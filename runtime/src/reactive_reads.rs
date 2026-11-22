//! Reads an event repeats, made once (speed plan item 6).
//!
//! One event often reads what qualifies the same evidence more than once:
//! `carries(latest(s), k)` for several caveats `k` of one stream, each in a
//! guard or a definition of its own, or two bindings that read whether the
//! same evidence is observed. Each such read walks the same caveats. That
//! lineage (`qualify_checked` of evidence alone) reads only the graph, the
//! names and the take-backs, and every write to those passes through the
//! write module, which counts it (`wrote`). So within one event, a lineage
//! read while the count stands is the same as when it was read, and it is
//! kept and handed out again until anything is written.
//!
//! A read also notes what it consulted in the record of the binding group
//! being evaluated, if there is one (`consult`), so that a later change to
//! anything else leaves the group as it is. A read made again must note the
//! same, so each kept lineage keeps what its read noted, and a read made
//! where nothing was being noted is made again where something is. Only
//! lineages are kept: a read that fails is made again, and fails again.
//!
//! The kept lineages live in a thread's own table, never in a session or a
//! transaction, and are dropped when the event ends, however it ends. They
//! belong to the session the event runs on, which stays borrowed until then:
//! another session read meanwhile, such as a copy a read makes to see what an
//! event would do, neither finds nor keeps any.
use super::{Consulted, ReactiveSession, CONSULTING, NOTING};
use crate::fast_hash::HashMap;
use crate::reactive_expr::{EvalError, Provenance};
use crate::NodeId;
use std::cell::{Cell, RefCell};

struct Kept {
    /// The count of writes when it was read.
    written: u64,
    lineage: Provenance,
    /// What the read noted, when something was being noted as it was made.
    notes: Option<Consulted>,
}

/// The lineages kept in the scope open now, by evidence. Its room is kept
/// from one scope to the next.
#[derive(Default)]
struct Reads {
    /// The session whose reads are kept, while a scope is open.
    open: Option<usize>,
    /// The evidence of each kept lineage, in `kept`'s order.
    evidence: Vec<NodeId>,
    /// Where each evidence is in `kept`, once more are kept than a walk
    /// along `evidence` finds quickly.
    at: HashMap<NodeId, usize>,
    kept: Vec<Kept>,
}

/// How many kept lineages are found by walking along them.
const WALKED: usize = 16;

impl Reads {
    fn find(&self, evidence: NodeId) -> Option<usize> {
        if self.kept.len() <= WALKED {
            self.evidence.iter().position(|&kept| kept == evidence)
        } else {
            self.at.get(&evidence).copied()
        }
    }

    fn insert(&mut self, evidence: NodeId, kept: Kept) {
        if let Some(index) = self.find(evidence) {
            self.kept[index] = kept;
            return;
        }
        self.evidence.push(evidence);
        self.kept.push(kept);
        match self.kept.len() {
            length if length == WALKED + 1 => self.at.extend(
                self.evidence
                    .iter()
                    .enumerate()
                    .map(|(index, &evidence)| (evidence, index)),
            ),
            length if length > WALKED + 1 => {
                self.at.insert(evidence, length - 1);
            }
            _ => {}
        }
    }
}

thread_local! {
    static READS: RefCell<Reads> = RefCell::new(Reads::default());
    /// Writes to the graph, the names or the take-backs of any session of
    /// this thread so far.
    static WRITTEN: Cell<u64> = const { Cell::new(0) };
}

/// Counts a write to the graph, the names or the take-backs: what was kept
/// before it is read again.
pub(super) fn wrote() {
    WRITTEN.set(WRITTEN.get().wrapping_add(1));
}

/// Where a session is: what the kept reads belong to.
fn place(session: &ReactiveSession) -> usize {
    std::ptr::from_ref(session) as usize
}

/// Keeps `session`'s reads while it lives, unless a scope is already open:
/// open while an event runs on the session.
pub(super) struct Scope {
    opened: bool,
}

impl Scope {
    pub(super) fn open(session: &ReactiveSession) -> Self {
        let opened = READS.with_borrow_mut(|reads| {
            reads.open.is_none() && {
                reads.open = Some(place(session));
                true
            }
        });
        Self { opened }
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if self.opened {
            READS.with_borrow_mut(|reads| {
                reads.open = None;
                reads.evidence.clear();
                reads.at.clear();
                reads.kept.clear();
            });
        }
    }
}

/// How long each list of a record was: what a read appends comes after.
type Lengths = [usize; 5];

fn lengths(record: &Consulted) -> Lengths {
    [
        record.reached.len(),
        record.listed.len(),
        record.withdrawals.len(),
        record.predicates.len(),
        record.renewals.len(),
    ]
}

/// What `record` gained past `from`.
fn gained(record: &Consulted, from: Lengths) -> Consulted {
    Consulted {
        reached: record.reached[from[0]..].to_vec(),
        listed: record.listed[from[1]..].to_vec(),
        withdrawals: record.withdrawals[from[2]..].to_vec(),
        predicates: record.predicates[from[3]..].to_vec(),
        renewals: record.renewals[from[4]..].to_vec(),
    }
}

/// What the open scope keeps of `evidence`: its lineage, or that the read
/// is to be made, and whether to keep it.
enum Found {
    /// The lineage, and in debug builds what its read noted.
    Kept(Provenance, Option<Consulted>),
    Read {
        keep: bool,
    },
}

/// The lineage `session` gives the evidence `evidence` with no extra
/// caveats: a kept one, noting again what its read noted, or `read`'s, kept
/// while a scope of `session`'s is open.
pub(super) fn qualified(
    session: &ReactiveSession,
    evidence: NodeId,
    read: impl FnOnce() -> Result<Provenance, EvalError>,
) -> Result<Provenance, EvalError> {
    let noting = NOTING.get();
    let found = READS.with_borrow(|reads| {
        if reads.open != Some(place(session)) {
            return Found::Read { keep: false };
        }
        let Some(index) = reads.find(evidence) else {
            return Found::Read { keep: true };
        };
        let kept = &reads.kept[index];
        if kept.written != WRITTEN.get() {
            return Found::Read { keep: true };
        }
        match (&kept.notes, noting) {
            (_, 0) => {}
            (Some(notes), _) => {
                CONSULTING.with_borrow_mut(|records| records[noting - 1].absorb(notes));
            }
            // Made where nothing was noted: made again, to note it.
            (None, _) => return Found::Read { keep: true },
        }
        // Debug builds check the read made again against what is kept.
        let notes = cfg!(debug_assertions).then(|| kept.notes.clone()).flatten();
        Found::Kept(kept.lineage.clone(), notes)
    });
    match found {
        Found::Kept(lineage, _notes) => {
            #[cfg(debug_assertions)]
            {
                let (again, mut noted) = super::noted_apart(read);
                assert_eq!(
                    again.as_ref(),
                    Ok(&lineage),
                    "a kept lineage differs from its read made again"
                );
                if noting != 0 {
                    let mut kept = _notes.expect("a read kept where something was noted");
                    kept.settle();
                    noted.settle();
                    assert_eq!(
                        kept, noted,
                        "a kept read noted what it consulted differently"
                    );
                }
            }
            Ok(lineage)
        }
        Found::Read { keep: false } => read(),
        Found::Read { keep: true } => keep(evidence, noting, read),
    }
}

/// Makes the read and keeps its lineage, with what it noted.
fn keep(
    evidence: NodeId,
    noting: usize,
    read: impl FnOnce() -> Result<Provenance, EvalError>,
) -> Result<Provenance, EvalError> {
    let from =
        (noting != 0).then(|| CONSULTING.with_borrow(|records| lengths(&records[noting - 1])));
    let written = WRITTEN.get();
    let lineage = read()?;
    // A read only reads.
    debug_assert_eq!(WRITTEN.get(), written, "a read wrote");
    // The read notes in the record that was being filled as it began, and
    // leaves that record being filled.
    debug_assert_eq!(NOTING.get(), noting, "a read changed the record noted in");
    let notes =
        from.map(|from| CONSULTING.with_borrow(|records| gained(&records[noting - 1], from)));
    READS.with_borrow_mut(|reads| {
        let kept = Kept {
            written,
            lineage: lineage.clone(),
            notes,
        };
        reads.insert(evidence, kept);
    });
    Ok(lineage)
}
