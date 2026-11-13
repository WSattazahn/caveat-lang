//! Which histories each part of what a session holds can carry records of
//! (finding D054). Departure turns the names of a windowed history's records
//! into one marker of that history, in the provenance that named them, so a
//! marker stands only where some rule or declaration can carry a record of
//! its history. Restore refuses a provenance that holds a marker or a
//! record's name, or a cause a series or a withdrawal keeps, that no rule
//! or declaration of the program can carry into it: in every program since
//! finding D061, and before it only in a program with a window. See
//! spec/caveat-save-0.1.md, "Restore validation".
//!
//! The bound is computed once per loaded program, from its rules,
//! procedures, reopening triggers and declarations: when restore first
//! needs it, and in debug builds as the program loads. Guards are not
//! evaluated: an effect that ran and one that was skipped carry the same. A
//! procedure's steps carry what any call of it passes, and a read carries
//! whatever it could find. So a marker some rule could carry in restores
//! whether or not one did: like the rest of restore, this checks what is
//! possible, not what happened.
//!
//! Each part of a thing has its own bound, from what writes it: a state's
//! lineage, and its grounds, which take what its initializer's and its sets'
//! values or citations ground on as `evaluate_grounds` reads them, so never
//! the guard of the rule that sets the state, though what a value reads can
//! hold the guard of the rule that wrote it; a stream's readings, their
//! observation and `observed` records, and its selection, which only a
//! skipped sample's guard writes; an evidence's observation and `observed`
//! records; the `withdrawn` records of an evidence's or a stream's records,
//! which take the guards of its skipped withdrawals; a series' bases, its
//! grounds (what its commits use, cite and give as reasons), its outcomes,
//! the evidence they name, which only what a resolve's `because` selects
//! gives, its reopening records, the evidence that reopened it or that a
//! hold defers, which only what a reopen's `because`, a reopening trigger or
//! a declaration selects gives, the reasons of its hold, which only what a
//! hold's `because` selects gives, its `committed` and `reopened` records,
//! which take the guards of its skipped commits and reopenings, and its
//! selection, which only a skipped commit's guard writes; the guards
//! scheduled on each evidence or stream, which take what its `qualify ...
//! after` effects evaluate; the withdrawals' reasons; and each cue. What a
//! release's `because` selects goes into the reopening record and is kept
//! as no cause.
//!
//! A procedure's parameter grounds on what its argument gives grounds. In a
//! program a release before rc.12 may have loaded it also grounds on what
//! the argument carries, its whole lineage, as those releases grounded it
//! (F268): a save of such a program can be that old (D061). A window rules
//! that out, and so does what else the language gained after rc.11 that the
//! loaded program keeps, as `may_load_before_rc12` lists it (D083). Each
//! parameter has its own bound, and a step's grounds take a call's argument
//! only through the parameter the step reads, as `evaluate_grounds` reads a
//! parameter by its name: a step that reads none, such as `set s = 5`, takes
//! nothing of the arguments, though its lineage takes all of them.
//!
//! A read takes the parts the runtime reads for it, and no other part of
//! what it names (`Read`). A read of evidence or of a stream takes the
//! record's name and what its observation and `observed` records hold, and
//! of a stream what its readings and its selection hold too; a value read
//! of a stream is given all of that, though the runtime reads only the
//! readings and the selection for it. Of a series or a commitment, a value
//! read takes the bases, `grounds_of` the grounds, `outcome` and `resolved`
//! the outcomes, `committed` and `in_basis` the bases, the reopening records
//! and the `committed` records, and `reopened` and `on_hold` those and the
//! `reopened` records, each with the selection. A withdrawal predicate
//! takes the `withdrawn` records of what it asks about and each
//! withdrawal's reason, read through its records, and nothing else of what
//! it asks about. In grounds, `qualified`, `observed` and `carries` give
//! only the name of the evidence they ask about, and a cause an outcome, a
//! series or a withdrawal keeps is the name its `because` selects. Two
//! reads take a name from what the session holds and read it through its
//! records: a `because caveated(STATE, CAVEAT)` reads, with the state's
//! lineage, each evidence the state's grounds name, and `reopened` and
//! `on_hold`, and with them the next revision's basis, read each evidence
//! that reopened the commitment again, as its records stand then. What the
//! second finds is bounded with the reopening records, so `committed` and
//! `in_basis` are given it too.
//!
//! A part's content also flows into what takes it whole, whatever wrote it:
//! grounds into their lineage or basis, a cause into its outcome or
//! reopening, a selection into what inherits it, and a reopening and a
//! `reopened` record into the next revision's basis. The guard of a
//! qualification or a withdrawal, and a scheduled guard when it applies,
//! joins what lists the record it qualifies then: a state's lineage and the
//! observation, examination and predicate records, never a part that keeps
//! what was read when it was written. A part holds a history's records only
//! where these flows lead to it from a read of the history or a name of
//! one, and a reading also holds its own name. So content moves only into
//! parts whose bound contains its own: a session that starts within the
//! bound stays within it, and a debug failure is a flow the bound misses.
use super::*;
use crate::reactive_expr::{Carried, Grounded, Read};
use std::collections::VecDeque;

/// What a provenance belongs to, for the bound: the declared thing, and the
/// part of it, whose content it holds. Each provenance a save holds has one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Owner {
    /// A state's lineage.
    State(String),
    /// A state's grounds: what its initializer's and its sets' values, or
    /// their citations, give grounds as `evaluate_grounds` reads them. The
    /// guard of the rule that sets it stays out, though a read can bring in
    /// what other rules' guards left in what it reads.
    StateGrounds(String),
    /// What a `caveated` selector reads besides the state's lineage: each
    /// evidence the state's grounds name, through its records. It holds no
    /// provenance itself.
    GroundsRecords(String),
    /// A reading stream's readings, which keep what their sample read.
    Stream(String),
    /// The observation and `observed` records of a stream's readings, which
    /// a later qualification or withdrawal can join.
    StreamRecords(String),
    /// A reading stream's selection: the guards of its skipped samples.
    StreamSelection(String),
    /// The observation and `observed` records of declared or renewable
    /// evidence and of its occurrences.
    Evidence(String),
    /// The `withdrawn` records of an evidence's or a stream's records: the
    /// guards of its skipped withdrawals, which only a withdrawal predicate
    /// reads.
    Withdrawn(String),
    /// The guards of the qualifications scheduled on an evidence, or on a
    /// stream's readings: what each `qualify ... after` of it evaluates.
    Scheduled(String),
    /// What joins, when a qualification or a withdrawal of a history's
    /// record applies, each state lineage and record that lists the record
    /// then: the guard of each. It holds no provenance itself.
    Qualifications(String),
    /// A series' or a commitment's bases.
    Basis(String),
    /// A commitment's `committed` records: the guards of its skipped
    /// commits.
    Committed(String),
    /// Its grounds: what its commits use, cite and give as reasons, as
    /// `evaluate_grounds` reads them. The commit's own guard, a permission
    /// and a predecessor's basis stay out.
    SeriesGrounds(String),
    /// A series' outcomes.
    Outcome(String),
    /// The evidence a series' outcomes name: what its resolves select.
    OutcomeCauses(String),
    /// Its reopening records.
    Reopening(String),
    /// Its `reopened` records: the guards of its skipped reopenings.
    Reopened(String),
    /// The evidence that reopened it, and that a hold defers: what its
    /// reopens select, the stream its reopening triggers watch, and what a
    /// declaration reopens it with. `reopened` reads each through its
    /// records, so what those hold flows into the reopening records.
    ReopeningCauses(String),
    /// The reasons of a series' hold: what its holds select.
    HoldCauses(String),
    /// A series' selection: the guards of its skipped commits.
    SeriesSelection(String),
    /// A caveat's examination record and its `examined` predicate's record.
    Caveat(String),
    /// What a procedure's steps run on: the guard and arguments of every
    /// call of it.
    Call(String),
    /// What a procedure's parameter grounds on: what the argument every call
    /// passes for it gives grounds, and in a program a release before rc.12
    /// may have loaded what that argument carries too (D061, D083).
    Parameter(String, String),
    /// What a withdrawal predicate's lineage takes: the reason of every
    /// withdrawal, read through its records.
    Withdrawals,
    /// The reason of every withdrawal, by name: what a withdrawal keeps and
    /// what a withdrawal predicate's grounds take.
    WithdrawalReasons,
    /// The guards of the last event's emits of a cue.
    Cue(String),
}

/// For each owner, the histories it can carry records of. A record belongs
/// to its reading stream or renewable evidence; any other evidence is its
/// own history here, so that every name is bounded.
#[derive(Debug, Default)]
pub(crate) struct HistoryReach {
    histories: HashMap<String, usize>,
    reach: HashMap<Owner, Vec<u64>>,
}

impl HistoryReach {
    /// Whether `owner` can hold records of `history`.
    pub(crate) fn carries(&self, owner: &Owner, history: &str) -> bool {
        let (Some(&bit), Some(bits)) = (self.histories.get(history), self.reach.get(owner)) else {
            return false;
        };
        bits[bit / 64] & (1 << (bit % 64)) != 0
    }
}

/// What one flow takes from: the owners whose content it carries, the
/// histories whose records it reads, the histories whose records it names
/// without reading them, and whether it could read anything at all.
#[derive(Debug, Default, Clone)]
struct Sources {
    owners: BTreeSet<usize>,
    /// A read through a record takes what the record's own owner holds.
    histories: BTreeSet<usize>,
    /// A name alone, as grounds and a cause list keep it, takes nothing
    /// else here. What reads a kept name through its records is in
    /// `Flows::through`.
    named: BTreeSet<usize>,
    everything: bool,
}

/// The flows of one program, before they are closed over.
struct Flows<'a> {
    session: &'a ReactiveSession,
    histories: HashMap<String, usize>,
    /// The owner of each history's records, by the history's bit.
    record_owners: Vec<usize>,
    /// What joins each history's records later, by the history's bit.
    qualifications: Vec<usize>,
    owners: HashMap<Owner, usize>,
    keys: Vec<Owner>,
    /// For each owner, the owners its content flows into.
    into: Vec<Vec<usize>>,
    /// For each owner, the histories whose records it names itself.
    names: Vec<Vec<usize>>,
    /// The owners a read of anything reaches.
    everything: Vec<bool>,
    /// The owners whose provenances a later qualification joins.
    joined: Vec<bool>,
    /// For each owner, the owners that read through its records each record
    /// the owner can name.
    through: Vec<Vec<usize>>,
    /// The procedure whose steps are being walked, if any: a name one of its
    /// steps reads that is no state is its parameter of that name, as
    /// `evaluate` and `evaluate_grounds` look names up.
    procedure: Option<&'a Procedure>,
    /// Whether a parameter grounds on what its argument carries as well,
    /// as releases before rc.12 grounded it (F268): only in a program such
    /// a release may have loaded (`may_load_before_rc12`).
    wide_parameters: bool,
}

impl<'a> Flows<'a> {
    fn new(session: &'a ReactiveSession) -> Self {
        let mut flows = Self {
            wide_parameters: session.may_load_before_rc12(),
            session,
            histories: HashMap::default(),
            record_owners: Vec::new(),
            qualifications: Vec::new(),
            owners: HashMap::default(),
            keys: Vec::new(),
            into: Vec::new(),
            names: Vec::new(),
            everything: Vec::new(),
            joined: Vec::new(),
            through: Vec::new(),
            procedure: None,
        };
        // Every stream and every declared evidence, renewable or not, in name
        // order. The graph holds only declared nodes while the program loads.
        let mut histories = session
            .symbols
            .iter()
            .filter(|(_, id)| {
                matches!(session.graph.nodes.get(id), Some(NodeKind::Evidence { .. }))
            })
            .map(|(name, _)| name.to_string())
            .chain(session.reading_streams.keys().cloned())
            .collect::<Vec<_>>();
        histories.sort_unstable();
        histories.dedup();
        for history in histories {
            let owner = flows.owner(session.record_owner(&history));
            let qualifications = flows.owner(Owner::Qualifications(history.clone()));
            flows.histories.insert(history, flows.record_owners.len());
            flows.record_owners.push(owner);
            flows.qualifications.push(qualifications);
        }
        flows
    }

    /// The owner's index. A part made here flows into what takes it whole,
    /// and what reads the names it keeps through their records is noted.
    fn owner(&mut self, owner: Owner) -> usize {
        if let Some(&index) = self.owners.get(&owner) {
            return index;
        }
        let index = self.keys.len();
        let wholes = self.wholes(&owner);
        let read_through = Self::read_through(&owner);
        // A state's lineage and the observation, examination and predicate
        // records are what a qualification or a withdrawal joins when it
        // applies (`apply_qualification`). Readings, bases, reopening
        // records, grounds, outcomes, selections, scheduled guards and cues
        // keep what was read when they were written, so nothing joins them
        // later (D054).
        let joined = matches!(
            owner,
            Owner::State(_)
                | Owner::StreamRecords(_)
                | Owner::Evidence(_)
                | Owner::Withdrawn(_)
                | Owner::Caveat(_)
                | Owner::Committed(_)
                | Owner::Reopened(_)
        );
        self.owners.insert(owner.clone(), index);
        self.keys.push(owner);
        self.into.push(Vec::new());
        self.names.push(Vec::new());
        self.everything.push(false);
        self.joined.push(joined);
        self.through.push(Vec::new());
        for whole in wholes {
            let whole = self.owner(whole);
            self.into[index].push(whole);
        }
        if let Some((names, reader)) = read_through {
            let names = self.owner(names);
            let reader = self.owner(reader);
            self.through[names].push(reader);
        }
        index
    }

    /// The two reads that take a name from what the session holds and read
    /// it through its records, as (the part that keeps the names, the part
    /// that takes what their records hold), for the part that brings the
    /// read (D054). `select_because` reads each evidence a `caveated`
    /// selector takes from the state's grounds with `qualify`, when a
    /// resolve, a reopen, a hold or a release runs. `predicate_tracked`
    /// reads each evidence that reopened a commitment the same way, whenever
    /// `reopened` or `on_hold` is read and when the next revision is
    /// committed, so later than the reopening record was written. What the
    /// names are is known only once the flows are closed, so `close` adds
    /// these flows as the part that keeps the names gains histories.
    fn read_through(owner: &Owner) -> Option<(Owner, Owner)> {
        match owner {
            Owner::GroundsRecords(state) => {
                Some((Owner::StateGrounds(state.clone()), owner.clone()))
            }
            Owner::ReopeningCauses(series) => {
                Some((owner.clone(), Owner::Reopening(series.clone())))
            }
            _ => None,
        }
    }

    /// Where a part's content goes whatever wrote it, so that a part's bound
    /// lies within the bound of each provenance its content joins (D054). A
    /// read of a series takes the parts it reads, each by itself (`read`).
    fn wholes(&self, owner: &Owner) -> Vec<Owner> {
        match owner {
            // A set's grounds lie within its lineage.
            Owner::StateGrounds(state) => vec![Owner::State(state.clone())],
            // A reading's records start with what the reading holds, and a
            // read through a reading takes both.
            Owner::Stream(stream) => vec![Owner::StreamRecords(stream.clone())],
            // A read of the stream inherits its selection.
            Owner::StreamSelection(stream) => vec![Owner::Stream(stream.clone())],
            // A scheduled guard joins, when it applies, what an immediate
            // qualification of the same records joins.
            Owner::Scheduled(history) => vec![Owner::Qualifications(history.clone())],
            // A commit's grounds lie within its basis.
            Owner::SeriesGrounds(series) => vec![Owner::Basis(series.clone())],
            // What a resolve selects lies within the outcome's lineage.
            Owner::OutcomeCauses(series) => vec![Owner::Outcome(series.clone())],
            // A revision's basis takes its predecessor's reopening, and the
            // guards of skipped reopenings, as reading `reopened` gives them.
            // A plain commitment is made once and takes neither.
            Owner::Reopening(series) | Owner::Reopened(series)
                if self.session.decision_series.contains_key(series) =>
            {
                vec![Owner::Basis(series.clone())]
            }
            // What reopened a commitment is named in its reopening record,
            // and by `reopened` when it is read.
            Owner::ReopeningCauses(series) => vec![Owner::Reopening(series.clone())],
            // A revision's basis, a resolve and a reopening inherit the
            // series' selection, as a read of the series does.
            Owner::SeriesSelection(series) => vec![
                Owner::Basis(series.clone()),
                Owner::Outcome(series.clone()),
                Owner::Reopening(series.clone()),
            ],
            // A parameter's grounds lie within its lineage.
            Owner::Parameter(procedure, _) => vec![Owner::Call(procedure.clone())],
            // A withdrawal predicate's lineage names each reason too.
            Owner::WithdrawalReasons => vec![Owner::Withdrawals],
            _ => Vec::new(),
        }
    }

    /// A name a record goes by, as a provenance lists it, alone.
    fn record(&self, record: &str, sources: &mut Sources) {
        if let Some(&bit) = self.histories.get(self.session.record_history(record)) {
            sources.named.insert(bit);
        }
    }

    /// The name of a record of `name`'s history, with nothing its records
    /// hold: what `qualified`, `observed` and `carries` give grounds, and
    /// what an outcome, a series or a withdrawal keeps of what a `because`
    /// selects. A name that is no history is read whole.
    fn named(&mut self, name: &str, sources: &mut Sources) {
        match self.histories.get(name) {
            Some(&bit) => {
                sources.named.insert(bit);
            }
            None => self.name(name, sources),
        }
    }

    /// A read of `name` through its records: the records of a stream or of
    /// evidence, a caveat's, or every part of a commitment or a series.
    fn name(&mut self, name: &str, sources: &mut Sources) {
        self.read(Read::Records, name, sources);
    }

    /// A graph or history read of `name`, with what the read takes of it
    /// (D054). Evidence and a stream are read through their records: the
    /// record's name, and what its observation and `observed` records hold.
    /// A withdrawal predicate takes none of that: only the `withdrawn`
    /// records of what it asks about and the reason of each withdrawal,
    /// through its records. Of a commitment or a series a read takes the
    /// parts `history_read` and `predicate_tracked` read for it; a read
    /// that names none of them takes every part.
    fn read(&mut self, read: Read, name: &str, sources: &mut Sources) {
        if read == Read::Withdrawn {
            sources.owners.insert(self.owner(Owner::Withdrawals));
            // A skipped withdrawal leaves its guard under the record's own
            // name, which is where `withdrawn(E)` looks; the other forms
            // look under the name they ask about, where none is kept, and
            // are given the same.
            if self.histories.contains_key(name) {
                sources
                    .owners
                    .insert(self.owner(Owner::Withdrawn(name.into())));
            }
            return;
        }
        if let Some(&bit) = self.histories.get(name) {
            sources.histories.insert(bit);
            return;
        }
        let session = self.session;
        let node = session
            .symbols
            .get(name)
            .and_then(|id| session.graph.nodes.get(id));
        if matches!(node, Some(NodeKind::Caveat { .. })) {
            sources
                .owners
                .insert(self.owner(Owner::Caveat(name.into())));
            return;
        }
        if session.states.contains_key(name) {
            sources.owners.insert(self.owner(Owner::State(name.into())));
            return;
        }
        // A commitment, a decision series, or an action a rule commits.
        // Grounds lie within the basis, what a resolve selects within the
        // outcome and what reopened it within the reopening record, and
        // the basis, the outcome and the reopening record inherit the
        // selection (`wholes`). A withdrawal predicate returned above, and
        // no form that loads reads a commitment as `Records` today: one
        // that did would take every part.
        let parts: &[fn(String) -> Owner] = match read {
            Read::Value => &[Owner::Basis, Owner::SeriesSelection],
            Read::Grounds => &[Owner::SeriesGrounds, Owner::SeriesSelection],
            Read::Outcome => &[Owner::Outcome, Owner::SeriesSelection],
            Read::Committed => &[
                Owner::Basis,
                Owner::Reopening,
                Owner::Committed,
                Owner::SeriesSelection,
            ],
            Read::Reopened => &[
                Owner::Basis,
                Owner::Reopening,
                Owner::Committed,
                Owner::Reopened,
                Owner::SeriesSelection,
            ],
            Read::Records | Read::Withdrawn => &[
                Owner::Basis,
                Owner::Outcome,
                Owner::Reopening,
                Owner::HoldCauses,
                Owner::Committed,
                Owner::Reopened,
                Owner::SeriesSelection,
            ],
        };
        let series = session.series_of(name);
        for part in parts {
            sources.owners.insert(self.owner(part(series.into())));
        }
    }

    /// What an expression's value can carry. A parameter it reads carries
    /// what the parameter grounds on; the rest of what the call passes, its
    /// guard and every argument, reaches each step through the step's guard
    /// (`program`).
    fn expression(&mut self, expression: &Expr, sources: &mut Sources) {
        let mut carried = Carried::default();
        expression.collect_carried(&mut carried);
        sources.everything |= carried.anything;
        for state in &carried.states {
            if self.session.states.contains_key(state) {
                sources
                    .owners
                    .insert(self.owner(Owner::State(state.clone())));
            } else {
                self.parameter(state, sources);
            }
        }
        for (read, name) in &carried.graph {
            self.read(*read, name, sources);
        }
    }

    /// What an expression gives a value's grounds, as `evaluate_grounds`
    /// reads it (D054): a state's grounds, a decision's grounds, the names
    /// of what reopened a commitment for `reopened`, of a hold's reasons for
    /// `on_hold` and of the withdrawals' reasons for a withdrawal predicate,
    /// and the name of the evidence `qualified`, `observed` and `carries`
    /// ask about. A read through a history or an outcome gives grounds what
    /// it gives the lineage. So grounds take no more than what the value
    /// reads grounds on: never the guard of the rule that writes them, nor
    /// the guard behind a state or a commitment the value reads, though a
    /// record read whole brings what its own writer read, guard included.
    /// In a procedure's step, a parameter gives what it grounds on, and a
    /// step that reads none takes nothing of the call's arguments (D061).
    fn grounds(&mut self, expression: &Expr, sources: &mut Sources) {
        let mut grounded = Grounded::default();
        expression.collect_grounded(&mut grounded);
        sources.everything |= grounded.anything;
        let session = self.session;
        for state in &grounded.states {
            if session.states.contains_key(state) {
                sources
                    .owners
                    .insert(self.owner(Owner::StateGrounds(state.clone())));
            } else {
                self.parameter(state, sources);
            }
        }
        for decision in &grounded.decisions {
            let series = session.series_of(decision).to_string();
            sources
                .owners
                .insert(self.owner(Owner::SeriesGrounds(series)));
        }
        for decision in &grounded.causes {
            let series = session.series_of(decision).to_string();
            sources
                .owners
                .insert(self.owner(Owner::ReopeningCauses(series)));
        }
        for decision in &grounded.holds {
            let series = session.series_of(decision).to_string();
            sources.owners.insert(self.owner(Owner::HoldCauses(series)));
        }
        if grounded.withdrawals {
            sources.owners.insert(self.owner(Owner::WithdrawalReasons));
        }
        for name in &grounded.named {
            self.named(name, sources);
        }
        for (read, name) in &grounded.graph {
            self.read(*read, name, sources);
        }
    }

    /// A name that is no state, read by a step of the procedure being
    /// walked: what the procedure's parameter of that name grounds on. A
    /// name that is not one of its parameters is a constant, which carries
    /// nothing, and outside a procedure a payload field is plain too.
    fn parameter(&mut self, name: &str, sources: &mut Sources) {
        let Some(procedure) = self.procedure else {
            return;
        };
        if procedure
            .parameters
            .iter()
            .any(|parameter| parameter == name)
        {
            let owner = self.owner(Owner::Parameter(procedure.name.clone(), name.to_string()));
            sources.owners.insert(owner);
        }
    }

    /// The evidence a `because` or a grant selects, with what reading it
    /// carries: each is read through its records (`select_because`).
    fn selector(&mut self, selector: &EvidenceSelector, sources: &mut Sources) {
        match selector {
            EvidenceSelector::Named(name)
            | EvidenceSelector::Latest(name)
            | EvidenceSelector::Every(name) => self.name(name, sources),
            EvidenceSelector::Occurrence { evidence, index } => {
                self.name(evidence, sources);
                self.expression(index, sources);
            }
            // The state's lineage, and each evidence its grounds name read
            // through its records, as they stand when the effect runs: they
            // can hold what the lineage does not, where the grounds name
            // evidence by a declaration or a citation.
            EvidenceSelector::Caveated { state, .. } => {
                sources
                    .owners
                    .insert(self.owner(Owner::State(state.clone())));
                sources
                    .owners
                    .insert(self.owner(Owner::GroundsRecords(state.clone())));
            }
        }
    }

    /// The names a `because` selects, as an outcome, a series or a
    /// withdrawal keeps them: a record of the evidence or the stream it
    /// names, or evidence the state's grounds name.
    fn cause(&mut self, selector: &EvidenceSelector, causes: &mut Sources) {
        match selector {
            EvidenceSelector::Named(name)
            | EvidenceSelector::Latest(name)
            | EvidenceSelector::Every(name)
            | EvidenceSelector::Occurrence { evidence: name, .. } => self.named(name, causes),
            EvidenceSelector::Caveated { state, .. } => {
                causes
                    .owners
                    .insert(self.owner(Owner::StateGrounds(state.clone())));
            }
        }
    }

    /// `sources` flow into `target`. A read through a record of a history
    /// also finds what the record's own owner holds: its observation record
    /// and its `observed` record. So each history read here brings its
    /// records' owner along; a history only named brings nothing else.
    fn connect(&mut self, sources: &Sources, target: Owner) {
        let target = self.owner(target);
        self.everything[target] |= sources.everything;
        for &owner in &sources.owners {
            self.into[owner].push(target);
        }
        for &bit in &sources.histories {
            self.names[target].push(bit);
            let owner = self.record_owners[bit];
            self.into[owner].push(target);
        }
        self.names[target].extend(&sources.named);
    }

    /// Where an effect's records go, and what they carry: its guard, every
    /// expression it evaluates, and what it selects. A skipped effect keeps
    /// its guard in the same owner as one that ran, or in a selection.
    fn effect(&mut self, effect: &Effect, guard: &Sources) {
        let session = self.session;
        let mut sources = guard.clone();
        let targets = match effect {
            Effect::Call {
                name,
                arguments: passed,
            } => {
                // A parameter grounds a value as its argument would inline:
                // on what the argument gives grounds, never on the call's
                // guard. Releases before rc.12 grounded it on the argument's
                // whole lineage (F268), and only a program such a release
                // may have loaded can have a save that old, so there it takes
                // what the argument carries too (D061, D083). Each parameter
                // has its own bound, which only a step that reads it takes
                // (`grounds`).
                for argument in passed {
                    self.expression(argument, &mut sources);
                }
                let parameters = session
                    .procedures
                    .get(name)
                    .map_or(&[][..], |procedure| &procedure.parameters[..]);
                for (parameter, argument) in parameters.iter().zip(passed) {
                    let mut grounds = Sources::default();
                    self.grounds(argument, &mut grounds);
                    if self.wide_parameters {
                        self.expression(argument, &mut grounds);
                    }
                    self.connect(&grounds, Owner::Parameter(name.clone(), parameter.clone()));
                }
                vec![Owner::Call(name.clone())]
            }
            Effect::Sample {
                stream,
                value,
                with,
                ..
            } => {
                // A reading lists its own name, which the check lets it
                // hold (`unreached`); it holds another record of its stream,
                // or a marker of one, only when its value, its guard or its
                // condition reads the stream (D054).
                self.expression(value, &mut sources);
                if let Some(condition) = with.as_ref().and_then(|with| with.condition.as_ref()) {
                    self.expression(condition, &mut sources);
                }
                // A skipped sample leaves its guard alone in the stream's
                // selection.
                self.connect(guard, Owner::StreamSelection(stream.clone()));
                vec![Owner::Stream(stream.clone())]
            }
            Effect::Emit { name } => vec![Owner::Cue(name.clone())],
            Effect::Reject { .. } | Effect::Unqualify { .. } | Effect::Unset { .. } => Vec::new(),
            Effect::Qualify {
                evidence,
                after,
                index,
                ..
            } => {
                for expression in after.iter().chain(index.as_deref()) {
                    self.expression(expression, &mut sources);
                }
                // When it applies, the guard joins what lists the record then
                // (`close`). A scheduled guard holds what its qualification
                // evaluates, and nothing scheduled on other evidence, until
                // it applies (D054).
                vec![if after.is_some() {
                    Owner::Scheduled(evidence.clone())
                } else {
                    Owner::Qualifications(evidence.clone())
                }]
            }
            Effect::Withdraw { target, because } => {
                // The withdrawal keeps the reason's name, which a withdrawal
                // predicate's grounds take; its lineage reads the reason
                // through its records.
                let mut reason = Sources::default();
                self.named(because, &mut reason);
                self.connect(&reason, Owner::WithdrawalReasons);
                let mut read = Sources::default();
                self.name(because, &mut read);
                self.connect(&read, Owner::Withdrawals);
                // A withdrawal joins what lists the record as a qualification
                // does; a skipped one leaves its guard in the record's
                // `withdrawn` record, which only a withdrawal predicate
                // reads: a read of the record does not find it (D054).
                let withdrawn = |history: &str| {
                    [
                        Owner::Qualifications(history.into()),
                        Owner::Withdrawn(history.into()),
                    ]
                };
                match target {
                    EvidenceSelector::Occurrence { evidence, index } => {
                        self.expression(index, &mut sources);
                        withdrawn(evidence).into()
                    }
                    EvidenceSelector::Named(name) | EvidenceSelector::Latest(name) => {
                        withdrawn(name).into()
                    }
                    // What `withdraw every` lists: a stream's readings, or the
                    // evidence's occurrences and the readings of each stream
                    // that reads from it.
                    EvidenceSelector::Every(name) => {
                        let mut targets = withdrawn(name).to_vec();
                        if !session.reading_streams.contains_key(name) {
                            for (stream, readings) in session.reading_streams.iter() {
                                if readings.template == *name {
                                    targets.extend(withdrawn(stream));
                                }
                            }
                        }
                        targets
                    }
                    // A program cannot withdraw `caveated(…)`.
                    EvidenceSelector::Caveated { .. } => Vec::new(),
                }
            }
            Effect::Renew { evidence }
            | Effect::Observe { evidence }
            | Effect::Reveal { evidence, .. } => vec![session.record_owner(evidence)],
            Effect::Set {
                name,
                value,
                because,
            } => {
                // The lineage is the value's and the guard's. The grounds are
                // what the value, or with `because` the citations alone, give
                // grounds as `evaluate_grounds` reads them: never the set's
                // guard, and of a state the value reads only that state's
                // grounds (D054), and of a parameter only what it grounds on
                // (D061).
                self.expression(value, &mut sources);
                let mut grounds = Sources::default();
                match because {
                    None => self.grounds(value, &mut grounds),
                    Some(citations) => {
                        for citation in citations {
                            self.grounds(citation, &mut grounds);
                        }
                    }
                }
                self.connect(&grounds, Owner::StateGrounds(name.clone()));
                vec![Owner::State(name.clone())]
            }
            Effect::Examine { caveat, .. } => vec![Owner::Caveat(caveat.clone())],
            // A revision's basis also takes its predecessor's, through
            // `reopened`: what flows into the series' reopenings and its
            // selection flows into its bases.
            Effect::Commit {
                action,
                using,
                citing,
                permission,
                reasons,
                ..
            } => {
                // The grounds are what the commit uses, cites and gives as
                // reasons give grounds; its guard, its permission and a
                // predecessor's basis stay in the basis (D054).
                let mut grounds = Sources::default();
                for expression in using
                    .iter()
                    .chain(citing)
                    .chain(reasons.iter().map(|(_, _, reading, _)| reading))
                {
                    self.expression(expression, &mut sources);
                    self.grounds(expression, &mut grounds);
                }
                if let Some(clause) = permission {
                    self.selector(&clause.grant, &mut sources);
                    if let Some(scope) = &clause.scope {
                        self.expression(scope, &mut sources);
                    }
                }
                self.connect(&grounds, Owner::SeriesGrounds(action.clone()));
                // A skipped commit leaves its guard alone in the series'
                // selection, or in the commitment's `committed` record.
                if session.decision_series.contains_key(action) {
                    self.connect(guard, Owner::SeriesSelection(action.clone()));
                } else {
                    self.connect(guard, Owner::Committed(action.clone()));
                }
                vec![Owner::Basis(action.clone())]
            }
            Effect::Resolve(resolution) => {
                for expression in resolution
                    .revision
                    .iter()
                    .chain(std::iter::once(&resolution.value))
                {
                    self.expression(expression, &mut sources);
                }
                self.selector(&resolution.because, &mut sources);
                // The outcome keeps what it selects by name (D054).
                let mut causes = Sources::default();
                self.cause(&resolution.because, &mut causes);
                self.connect(&causes, Owner::OutcomeCauses(resolution.action.clone()));
                vec![Owner::Outcome(resolution.action.clone())]
            }
            // A reopening, a hold and a release: each joins what it
            // selects, read through its records, and its guard to the
            // reopening record.
            Effect::Reopen {
                action,
                because,
                form,
            } => {
                self.selector(because, &mut sources);
                // The series keeps by name what a reopen selects, as the
                // cause of a reopening or of one a hold defers, and what a
                // hold selects, as the hold's reason. It keeps nothing a
                // release selects: the reopening that follows a release
                // names the deferred causes (D054).
                let kept = match form {
                    DecisionChange::Reopen => Some(Owner::ReopeningCauses(action.clone())),
                    DecisionChange::Hold => Some(Owner::HoldCauses(action.clone())),
                    DecisionChange::Release => None,
                };
                if let Some(kept) = kept {
                    let mut causes = Sources::default();
                    self.cause(because, &mut causes);
                    self.connect(&causes, kept);
                }
                // A skipped one leaves its guard alone in the `reopened`
                // record of what it would have reopened.
                self.connect(guard, Owner::Reopened(action.clone()));
                vec![Owner::Reopening(action.clone())]
            }
        };
        for target in targets {
            self.connect(&sources, target);
        }
    }

    /// Every flow of the program as it loaded.
    fn program(&mut self) {
        let session = self.session;
        let mut called = Vec::new();
        for rule in session.rules.iter() {
            let mut guard = Sources::default();
            self.expression(&rule.condition, &mut guard);
            self.effect(&rule.effect, &guard);
            if let Effect::Call { name, .. } = &rule.effect {
                called.push(name.as_str());
            }
        }
        // Each procedure a rule reaches, once: its steps run on what any call
        // passes, so a call's guard and arguments flow into every step's
        // lineage, and an argument's grounds into what a step grounds on
        // where the step reads that argument's parameter (D061).
        let mut reached = BTreeSet::new();
        while let Some(name) = called.pop() {
            if !reached.insert(name) {
                continue;
            }
            let Some(procedure) = session.procedures.get(name) else {
                continue;
            };
            let call = self.owner(Owner::Call(name.to_string()));
            self.procedure = Some(procedure);
            for step in &procedure.body {
                let mut guard = Sources::default();
                guard.owners.insert(call);
                self.expression(&step.condition, &mut guard);
                self.effect(&step.effect, &guard);
                if let Effect::Call { name, .. } = &step.effect {
                    called.push(name.as_str());
                }
            }
            self.procedure = None;
        }
        // A declared reopening trigger reopens its series because of the
        // reading just taken, under the sample's guard, which the stream's
        // records carry; a skipped one leaves that guard and its condition's
        // in the `reopened` record. Its filters are not read, and its cause
        // is the reading's name alone.
        for (series, watching, condition) in session.reopening_triggers.iter() {
            let mut sources = Sources::default();
            let mut causes = Sources::default();
            self.expression(condition, &mut sources);
            for trigger in watching {
                self.name(&trigger.stream, &mut sources);
                self.named(&trigger.stream, &mut causes);
            }
            self.connect(&sources, Owner::Reopening(series.clone()));
            self.connect(&sources, Owner::Reopened(series.clone()));
            self.connect(&causes, Owner::ReopeningCauses(series.clone()));
        }
        // What the declarations leave: each state's initial lineage and
        // grounds, which hold only the names they list, though a `caveated`
        // selector reads what the grounds name through its records
        // (`selector`); and the evidence a declaration reopens a commitment
        // with, which `reopened` reads through its records (`read_through`).
        // A declared commitment relies on nothing: only a rule's commit
        // relates `relies_on`, and no rule has run as the program loads.
        for name in session.states.names() {
            let cell = session.states.get(name).expect("a declared state");
            let mut lineage = Sources::default();
            for record in cell.value.provenance.evidence.iter() {
                self.record(record.as_str(), &mut lineage);
            }
            self.connect(&lineage, Owner::State(name.clone()));
            let mut grounds = Sources::default();
            for record in cell.grounds.evidence.iter() {
                self.record(record.as_str(), &mut grounds);
            }
            self.connect(&grounds, Owner::StateGrounds(name.clone()));
        }
        for edge in session.graph.edges.iter().take(session.loaded.edges) {
            if edge.relation != Relation::Reopens {
                continue;
            }
            let (Some(evidence), Some(commitment)) = (
                session.symbols.name_of(edge.from),
                session.symbols.name_of(edge.to),
            ) else {
                continue;
            };
            let series = session.series_of(commitment).to_string();
            let mut causes = Sources::default();
            self.record(evidence, &mut causes);
            self.connect(&causes, Owner::ReopeningCauses(series));
        }
    }

    /// What each owner can carry: what flows into it, closed over every flow.
    /// What joins a history's records later, the guard of a qualification
    /// or a withdrawal of one, joins every provenance that lists such a
    /// record when it applies (`apply_qualification`): so each part that
    /// holds such provenances takes it as soon as the part can hold a record
    /// of the history, and from then on, as its bound grows. In the same
    /// way, what reads through their records the names a part keeps
    /// (`read_through`) takes what a history's records hold as soon as the
    /// part can name a record of the history.
    fn close(self) -> HistoryReach {
        let words = self.histories.len().div_ceil(64).max(1);
        let mut bits = vec![vec![0u64; words]; self.keys.len()];
        for (owner, names) in self.names.iter().enumerate() {
            for &bit in names {
                bits[owner][bit / 64] |= 1 << (bit % 64);
            }
        }
        for (owner, _) in self.everything.iter().enumerate().filter(|(_, all)| **all) {
            for bit in 0..self.histories.len() {
                bits[owner][bit / 64] |= 1 << (bit % 64);
            }
        }
        let mut into = self.into;
        for targets in &mut into {
            targets.sort_unstable();
            targets.dedup();
        }
        let set = |words: &[u64]| {
            words
                .iter()
                .enumerate()
                .flat_map(|(word, &value)| {
                    (0..64usize)
                        .filter(move |&bit| value & (1u64 << bit) != 0)
                        .map(move |bit| word * 64 + bit)
                })
                .collect::<Vec<_>>()
        };
        for owner in (0..self.keys.len()).filter(|&owner| self.joined[owner]) {
            for bit in set(&bits[owner]) {
                into[self.qualifications[bit]].push(owner);
            }
        }
        for owner in (0..self.keys.len()).filter(|&owner| !self.through[owner].is_empty()) {
            for bit in set(&bits[owner]) {
                into[self.record_owners[bit]].extend(&self.through[owner]);
            }
        }
        let mut queue = (0..self.keys.len()).collect::<VecDeque<_>>();
        let mut queued = vec![true; self.keys.len()];
        while let Some(from) = queue.pop_front() {
            queued[from] = false;
            let carried = bits[from].clone();
            for to in into[from].clone() {
                let mut added = vec![0u64; words];
                for ((word, add), new) in bits[to].iter_mut().zip(&carried).zip(&mut added) {
                    *new = add & !*word;
                    *word |= add;
                }
                if added.iter().all(|&new| new == 0) {
                    continue;
                }
                // Each history `to` can now hold a record of brings what
                // joins that history's records, once.
                if self.joined[to] {
                    for bit in set(&added) {
                        let qualifications = self.qualifications[bit];
                        into[qualifications].push(to);
                        if !queued[qualifications] {
                            queued[qualifications] = true;
                            queue.push_back(qualifications);
                        }
                    }
                }
                // Each history `to` can now name brings what that history's
                // records hold to what reads `to`'s names through them.
                if !self.through[to].is_empty() {
                    for bit in set(&added) {
                        let records = self.record_owners[bit];
                        into[records].extend(&self.through[to]);
                        if !queued[records] {
                            queued[records] = true;
                            queue.push_back(records);
                        }
                    }
                }
                if !queued[to] {
                    queued[to] = true;
                    queue.push_back(to);
                }
            }
        }
        HistoryReach {
            histories: self.histories,
            reach: self.keys.into_iter().zip(bits).collect(),
        }
    }
}

/// Where a session holds a provenance or a cause, as restore names it.
pub(crate) enum Place<'a> {
    State(&'a str),
    StateGrounds(&'a str),
    Commitment(&'a str),
    CommitmentGrounds(&'a str),
    Stream(&'a str),
    Reading(&'a str),
    Series(&'a str),
    Outcome(&'a str),
    /// The scheduled qualification of an evidence or a reading.
    Scheduled(&'a str),
    Record(&'static str, &'a str),
    Predicate(&'a str, &'a str),
    Cue(&'a str),
    /// The evidence an outcome names.
    OutcomeCause(&'a str),
    /// A series' hold and the reopenings it defers.
    Hold(&'a str),
    Deferral(&'a str),
    /// A cause of a reopening, and the commitment it reopened.
    Reopens(&'a str, &'a str),
    /// A withdrawal's reason, by the record withdrawn.
    Withdrawal(&'a str),
}

impl std::fmt::Display for Place<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::State(name) => write!(f, "state {name}"),
            Self::StateGrounds(name) => write!(f, "state {name} grounds"),
            Self::Commitment(name) => write!(f, "commitment {name}"),
            Self::CommitmentGrounds(name) => write!(f, "commitment {name} grounds"),
            Self::Stream(name) => write!(f, "reading stream {name}"),
            Self::Reading(id) => write!(f, "reading {id}"),
            Self::Series(name) => write!(f, "decision series {name}"),
            Self::Outcome(id) => write!(f, "outcome of {id}"),
            Self::Scheduled(evidence) => write!(f, "scheduled qualification of {evidence}"),
            Self::Record(what, name) => write!(f, "{what} of {name}"),
            Self::Predicate(kind, name) => write!(f, "{kind}({name})"),
            Self::Cue(id) => write!(f, "cue {id}"),
            Self::OutcomeCause(id) => write!(f, "outcome of {id} because"),
            Self::Hold(name) => write!(f, "decision series {name} hold"),
            Self::Deferral(name) => write!(f, "decision series {name} deferral"),
            Self::Reopens(cause, commitment) => write!(f, "relation {cause} reopens {commitment}"),
            Self::Withdrawal(evidence) => write!(f, "withdrawal of {evidence} because"),
        }
    }
}

/// Whether an effect, as the program loaded it, is a form or holds an
/// expression that no release before rc.12 reads (D083): `unqualify`,
/// `unset`, `resolve`, `hold` and `release`; a withdrawal of `every` stream
/// or of an occurrence, a reopening because of one, and a qualification of
/// one or of `latest(…)`; a neutral sample, one `noting` an identifier or
/// `with` caveats; and a commit `citing` grounds or `retaining` a caveat
/// `because` evidence. The match is exhaustive, so a new effect must say
/// whether those releases read it.
fn effect_after_rc11(effect: &Effect, only_if: bool) -> bool {
    let after = |expression: &Expr| expression.has_form_after_rc11(only_if);
    match effect {
        Effect::Unqualify { .. } | Effect::Unset { .. } | Effect::Resolve(_) => true,
        Effect::Withdraw { target, .. } => matches!(
            target,
            EvidenceSelector::Occurrence { .. } | EvidenceSelector::Every(_)
        ),
        Effect::Reopen { because, form, .. } => {
            *form != DecisionChange::Reopen
                || matches!(because, EvidenceSelector::Occurrence { .. })
        }
        Effect::Qualify {
            after: delay,
            index,
            latest,
            ..
        } => index.is_some() || *latest || delay.as_ref().is_some_and(after),
        Effect::Sample {
            value,
            stance,
            note,
            with,
            ..
        } => stance.is_none() || note.is_some() || with.is_some() || after(value),
        Effect::Commit {
            using,
            citing,
            permission,
            reasons,
            ..
        } => {
            !citing.is_empty()
                || !reasons.is_empty()
                || using.as_ref().is_some_and(after)
                || permission
                    .as_ref()
                    .and_then(|permission| permission.scope.as_ref())
                    .is_some_and(after)
        }
        Effect::Set { value, because, .. } => after(value) || because.iter().flatten().any(after),
        Effect::Call { arguments, .. } => arguments.iter().any(after),
        Effect::Emit { .. }
        | Effect::Reject { .. }
        | Effect::Renew { .. }
        | Effect::Observe { .. }
        | Effect::Reveal { .. }
        | Effect::Examine { .. } => false,
    }
}

impl ReactiveSession {
    /// Whether a release before rc.12 may have loaded this program, so that
    /// a save of it may be that old and ground a procedure's parameter as
    /// those releases did (F268). Not when the program declares a window, a
    /// threshold, an agent, `work skips other routes`, an irreversible
    /// series, one that forecloses, an integer clock, a state `unset`, or a
    /// reopening `changed` trigger, and not when a rule, a procedure as it
    /// loaded, a binding or a define holds an effect or an expression that
    /// no such release reads (`effect_after_rc11`,
    /// `Expr::has_form_after_rc11`): each came in a later release, so no
    /// release before rc.12 loads a program that has one (D083). `only_if`
    /// counts unless the program of a bundle declares a function
    /// `MODULE__only_if` for one of its modules (`flat_only_if`), which a
    /// release before rc.12 calls for it in that module when the module
    /// declares `only_if` as something other than a function. What the
    /// loaded program does not keep is not looked at, so a program whose
    /// only later construct is there still may: a state's initial value, a
    /// function that nothing the loaded program keeps calls, a template no
    /// call specializes, and an expansion larger than rc.11 allowed (D034).
    /// Nor are a nested `for` block and an across-members form, which leave
    /// nothing once expanded; what they expand to is looked at.
    pub(crate) fn may_load_before_rc12(&self) -> bool {
        let only_if = !self.flat_only_if;
        let after = |expression: &Expr| expression.has_form_after_rc11(only_if);
        let declares_later = self.windowed()
            || self.has_thresholds
            || self.routed_work
            || !self.agents.is_empty()
            || !self.irreversible.is_empty()
            || !self.foreclosures.is_empty()
            || self.integer_clock()
            || self.ranges.values().any(|range| range.unset)
            || self
                .reopening_triggers
                .iter()
                .any(|(_, triggers, _)| triggers.iter().any(|trigger| trigger.changed));
        let holds_later = || {
            self.rules.iter().any(|rule| {
                after(&rule.condition) || effect_after_rc11(&rule.effect, only_if)
            }) || self.procedures.values().any(|procedure| {
                procedure
                    .body
                    .iter()
                    .any(|step| after(&step.condition) || effect_after_rc11(&step.effect, only_if))
            }) || self.binding_rules.iter().any(|binding| {
                after(&binding.condition)
                    || matches!(&binding.value, BindingExpression::Expression(value) if after(value))
                    || binding.because.iter().flatten().any(after)
            }) || self
                .define_rules
                .iter()
                .any(|(_, expression)| after(expression))
        };
        !(declares_later || holds_later())
    }

    /// The bound of finding D054, from the program as it loaded.
    pub(crate) fn history_reach(&self) -> HistoryReach {
        let mut flows = Flows::new(self);
        flows.program();
        flows.close()
    }

    /// The program's bound, computed the first time it is asked for. The
    /// session that first asks computes it, so it must be one as its program
    /// loaded, before any event or save: restore asks before it applies a
    /// save, and debug builds as the program loads. A later caller, played
    /// or restored, reads what that one computed. Every clone of a loaded
    /// session shares it, so a program `from_source_cached` keeps computes
    /// it once (D061).
    pub(crate) fn history_reach_bound(&self) -> &HistoryReach {
        self.reach.get_or_init(|| {
            debug_assert_eq!(
                self.sequence, 0,
                "the bound is computed as the program loaded"
            );
            self.history_reach()
        })
    }

    /// The history a record belongs to: its reading stream or renewable
    /// evidence, or the record itself.
    pub(crate) fn record_history<'n>(&self, record: &'n str) -> &'n str {
        match save::occurrence_parts(record) {
            Some((base, _))
                if self.reading_streams.contains_key(base) || self.renewals.contains_key(base) =>
            {
                base
            }
            _ => record,
        }
    }

    /// The owner of a history's observation and `observed` records: what a
    /// read through one of its records finds, and what a qualification or a
    /// withdrawal of one joins where they list it.
    fn record_owner(&self, history: &str) -> Owner {
        if self.reading_streams.contains_key(history) {
            Owner::StreamRecords(history.into())
        } else {
            Owner::Evidence(history.into())
        }
    }

    /// The decision series a commitment is a revision of, or the
    /// commitment itself.
    fn series_of<'n>(&self, commitment: &'n str) -> &'n str {
        match save::occurrence_parts(commitment) {
            Some((series, _)) if self.decision_series.contains_key(series) => series,
            _ => commitment,
        }
    }

    /// The owner of a predicate's record: the evidence, caveat or decision
    /// the skipped effect was about. A decision's records have their own
    /// owners, apart from its bases and reopening records, because a later
    /// qualification joins them and not those. A record's `withdrawn`
    /// record has its own, apart from its observation and `observed`
    /// records, because a read of the record does not find it (D054).
    fn predicate_owner(&self, kind: &str, name: &str) -> Owner {
        match kind {
            "examined" => Owner::Caveat(name.into()),
            "committed" => Owner::Committed(self.series_of(name).into()),
            "reopened" => Owner::Reopened(self.series_of(name).into()),
            WITHDRAWN => Owner::Withdrawn(self.record_history(name).into()),
            _ => self.record_owner(self.record_history(name)),
        }
    }

    /// Every provenance the session holds, where it is held, and its owner.
    /// With `old`, only those of the kinds an event changed since then.
    pub(crate) fn held<'s>(
        &'s self,
        old: Option<&Self>,
    ) -> Vec<(Place<'s>, Owner, &'s Provenance)> {
        let mut held = Vec::new();
        if !old.is_some_and(|old| Arc::ptr_eq(&old.states.cells, &self.states.cells)) {
            let mut names = self.states.names().collect::<Vec<_>>();
            names.sort_unstable();
            for name in names {
                let cell = self.states.get(name).expect("a declared state");
                held.push((
                    Place::State(name),
                    Owner::State(name.clone()),
                    &cell.value.provenance,
                ));
                held.push((
                    Place::StateGrounds(name),
                    Owner::StateGrounds(name.clone()),
                    &cell.grounds,
                ));
            }
        }
        if !old.is_some_and(|old| Arc::ptr_eq(&old.commitment_bases, &self.commitment_bases)) {
            for (name, basis) in self.commitment_bases.iter() {
                let owner = Owner::Basis(self.series_of(name).into());
                held.push((Place::Commitment(name), owner, &basis.provenance));
            }
        }
        if !old.is_some_and(|old| Arc::ptr_eq(&old.commitment_grounds, &self.commitment_grounds)) {
            for (name, grounds) in self.commitment_grounds.iter() {
                let owner = Owner::SeriesGrounds(self.series_of(name).into());
                held.push((Place::CommitmentGrounds(name), owner, grounds));
            }
        }
        if !old.is_some_and(|old| Arc::ptr_eq(&old.reading_streams, &self.reading_streams)) {
            for (name, stream) in self.reading_streams.iter() {
                held.push((
                    Place::Stream(name),
                    Owner::StreamSelection(name.clone()),
                    &stream.selection_qualifications,
                ));
                for reading in &stream.occurrences {
                    held.push((
                        Place::Reading(&reading.id),
                        Owner::Stream(name.clone()),
                        &reading.provenance,
                    ));
                }
            }
        }
        if !old.is_some_and(|old| Arc::ptr_eq(&old.decision_series, &self.decision_series)) {
            for (name, series) in self.decision_series.iter() {
                held.push((
                    Place::Series(name),
                    Owner::SeriesSelection(name.clone()),
                    &series.selection_qualifications,
                ));
                for revision in &series.revisions {
                    if let Some(outcome) = &revision.outcome {
                        held.push((
                            Place::Outcome(&revision.id),
                            Owner::Outcome(name.clone()),
                            &outcome.lineage,
                        ));
                    }
                }
            }
        }
        if !old.is_some_and(|old| Arc::ptr_eq(&old.scheduled, &self.scheduled)) {
            for scheduled in self.scheduled.iter() {
                // Each evidence's scheduled guards have their own bound.
                let owner = Owner::Scheduled(self.record_history(&scheduled.evidence).into());
                held.push((
                    Place::Scheduled(&scheduled.evidence),
                    owner,
                    &scheduled.guard,
                ));
            }
        }
        for (what, records, before) in [
            (
                "observation",
                &self.observation_qualifications,
                old.map(|old| &old.observation_qualifications),
            ),
            (
                "examination",
                &self.examination_qualifications,
                old.map(|old| &old.examination_qualifications),
            ),
            (
                "reopening",
                &self.reopening_qualifications,
                old.map(|old| &old.reopening_qualifications),
            ),
        ] {
            if before.is_some_and(|before| Arc::ptr_eq(before, records)) {
                continue;
            }
            for (name, provenance) in records.iter() {
                let owner = match what {
                    "observation" => self.record_owner(self.record_history(name)),
                    "examination" => Owner::Caveat(name.to_string()),
                    _ => Owner::Reopening(self.series_of(name).into()),
                };
                held.push((Place::Record(what, name), owner, provenance));
            }
        }
        if !old.is_some_and(|old| {
            Arc::ptr_eq(
                &old.predicate_qualifications,
                &self.predicate_qualifications,
            )
        }) {
            for (kind, targets) in self.predicate_qualifications.iter() {
                for (name, provenance) in targets {
                    let owner = self.predicate_owner(kind, name);
                    held.push((Place::Predicate(kind, name), owner, provenance));
                }
            }
        }
        // Cleared and written again by every event. Each cue's guards are
        // those of the emits of it (D054).
        for (cue, provenance) in self.cues.iter().zip(&self.cue_qualifications) {
            held.push((
                Place::Cue(cue.id()),
                Owner::Cue(cue.id().into()),
                provenance,
            ));
        }
        held
    }

    /// The records a session names outside its provenances that a read
    /// carries into one, with their owners: the evidence an outcome names,
    /// the reasons a hold keeps and the reopenings it defers, the evidence
    /// that reopened a commitment, and a withdrawal's reason. A read of the
    /// outcome, the hold or the reopening, or a withdrawal predicate, takes
    /// each with what qualifies it, so each is bounded as a name in a
    /// provenance is (D054): an outcome's by what the `because` of a
    /// resolve can select; a hold's reasons by what the `because` of a hold
    /// can; the reopenings it defers and the evidence that reopened a
    /// commitment by what the `because` of a reopen can or what a trigger
    /// reads; and a withdrawal's reason by what the `because` of a
    /// withdrawal can. A release keeps nothing it selects. A relation a
    /// declaration makes is the program's, not the session's: it stands
    /// before every relation an event adds, which is all a save holds
    /// (`save`), so restore makes it again from the source, and it is not
    /// listed. Its cause can be a caveat, a claim or a commitment, which no
    /// history holds. With `old`, only those of the kinds an event changed
    /// since then.
    pub(crate) fn causes<'s>(&'s self, old: Option<&Self>) -> Vec<(Place<'s>, Owner, &'s str)> {
        let mut causes = Vec::new();
        if !old.is_some_and(|old| Arc::ptr_eq(&old.decision_series, &self.decision_series)) {
            for (name, series) in self.decision_series.iter() {
                for revision in &series.revisions {
                    for cause in revision.outcome.iter().flat_map(|outcome| &outcome.because) {
                        causes.push((
                            Place::OutcomeCause(&revision.id),
                            Owner::OutcomeCauses(name.clone()),
                            cause.as_str(),
                        ));
                    }
                }
                for reason in &series.on_hold {
                    causes.push((
                        Place::Hold(name),
                        Owner::HoldCauses(name.clone()),
                        reason.as_str(),
                    ));
                }
                for cause in &series.deferred {
                    causes.push((
                        Place::Deferral(name),
                        Owner::ReopeningCauses(name.clone()),
                        cause.as_str(),
                    ));
                }
            }
        }
        if !old.is_some_and(|old| Arc::ptr_eq(&*old.graph, &*self.graph)) {
            for edge in self
                .graph
                .edges
                .iter_from(self.loaded.edges)
                .filter(|edge| edge.relation == Relation::Reopens)
            {
                let (Some(cause), Some(commitment)) = (
                    self.symbols.name_of(edge.from),
                    self.symbols.name_of(edge.to),
                ) else {
                    continue;
                };
                causes.push((
                    Place::Reopens(cause, commitment),
                    Owner::ReopeningCauses(self.series_of(commitment).into()),
                    cause,
                ));
            }
        }
        if !old.is_some_and(|old| Arc::ptr_eq(&*old.withdrawals, &*self.withdrawals)) {
            for withdrawal in self.withdrawals.iter() {
                causes.push((
                    Place::Withdrawal(&withdrawal.evidence),
                    Owner::WithdrawalReasons,
                    withdrawal.because.as_str(),
                ));
            }
        }
        causes
    }

    /// Whether `owner` can hold departed records of `history`. A journal
    /// entry's name never enters a provenance, so nothing carries the
    /// journal's records.
    fn reaches(&self, owner: &Owner, history: &str) -> bool {
        !(self.journal_window.is_some() && history == "journal")
            && self.history_reach_bound().carries(owner, history)
    }

    /// The history of `record`, a name `owner` holds at `place`, when
    /// `owner` cannot carry its records. A reading lists its own name, which
    /// no flow carries: it holds it while it is held, and departs with it.
    fn unreached<'n>(&self, place: &Place<'_>, owner: &Owner, record: &'n str) -> Option<&'n str> {
        if matches!(place, Place::Reading(id) if *id == record) {
            return None;
        }
        let history = self.record_history(record);
        (!self.history_reach_bound().carries(owner, history)).then_some(history)
    }

    /// Restore's check of findings D054 and D061, after every other: each
    /// marker a provenance holds, each name it holds and each cause a series
    /// or a withdrawal holds is of a history some rule or declaration of the
    /// program can carry into it. A program without a window holds no
    /// marker, since restore refused one already, but its names and causes
    /// are read as in any other program (D061).
    pub(crate) fn check_history_reach(&self) -> Result<(), String> {
        let refusal = |place: Place<'_>, history: &str| {
            format!(
                "{place}: no rule or declaration of this program carries records of {history} into it"
            )
        };
        for (place, owner, provenance) in self.held(None) {
            let unreached = provenance
                .departed
                .keys()
                .map(String::as_str)
                .find(|history| !self.reaches(&owner, history))
                .or_else(|| {
                    provenance
                        .evidence
                        .iter()
                        .find_map(|record| self.unreached(&place, &owner, record.as_str()))
                });
            if let Some(history) = unreached {
                return Err(refusal(place, history));
            }
        }
        for (place, owner, cause) in self.causes(None) {
            if let Some(history) = self.unreached(&place, &owner, cause) {
                return Err(refusal(place, history));
            }
        }
        Ok(())
    }

    /// Debug builds: every marker, every name and every cause the session
    /// holds is of a history some rule or declaration carries into it. Each
    /// part's content moves only into provenances whose bound contains its
    /// own, so a session that starts within the bound stays within it, and
    /// a failure is a flow the bound misses, which would make restore refuse
    /// a save the runtime wrote: the bound is what to fix. It runs after a
    /// program loads and after every event, a restored session's included,
    /// since restore accepts only one within the bound (D061). With `old`,
    /// only the kinds an event changed are read.
    #[cfg(debug_assertions)]
    pub(crate) fn check_history_reach_held(&self, old: Option<&Self>) {
        for (place, owner, provenance) in self.held(old) {
            for history in provenance.departed.keys() {
                assert!(
                    self.reaches(&owner, history),
                    "D054: {place} holds departed records of {history}, which the program's history reach does not carry into it"
                );
            }
            for record in provenance.evidence.iter() {
                if let Some(history) = self.unreached(&place, &owner, record.as_str()) {
                    panic!(
                        "D054: {place} holds {record}, a record of {history}, which the program's history reach does not carry into it"
                    );
                }
            }
        }
        for (place, owner, cause) in self.causes(old) {
            if let Some(history) = self.unreached(&place, &owner, cause) {
                panic!(
                    "D054: {place} names {cause}, a record of {history}, which the program's history reach does not carry into it"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `st` sets `t` because of `d`, whose basis holds by then only a marker
    /// of `b`'s first reading, which its guard read: the grounds take that
    /// marker, which the lineage, set from a later reading, does not hold.
    const CITED: &str = "evidence gb from \"gb\";
readings b from gb window 1;
decisions d limit 4;
state t = 0;
event sb v min 0 max 10;
event cd v min 0 max 10;
event st v min 0 max 10;
on sb sample b = v;
on cd when latest(b) > 0 commit d because enough using 1;
on st set t = latest(b) because latest(d);
";

    /// The citation carries `b` into `t`'s grounds, so the marker is within
    /// the bound after every event. Restore refuses the save from `st` on in
    /// its earlier check that grounds lie within the lineage, a separate
    /// finding that this check leaves as it was.
    #[test]
    fn a_citation_carries_a_departed_marker_into_grounds_within_the_bound() {
        let mut session = ReactiveSession::from_source(CITED).unwrap();
        // The bound is the loaded program's, in release builds too, where
        // loading does not compute it.
        session.history_reach_bound();
        for (step, (event, payload)) in [
            ("sb", r#"{"v":5}"#),
            ("cd", r#"{"v":1}"#),
            ("sb", r#"{"v":6}"#),
            ("st", r#"{"v":1}"#),
            ("sb", r#"{"v":7}"#),
            ("sb", r#"{"v":8}"#),
        ]
        .into_iter()
        .enumerate()
        {
            let outcome = session.dispatch_outcome_json(event, payload).unwrap();
            let outcome = serde_json::to_value(outcome).unwrap();
            assert_eq!(outcome["outcome"], "accepted", "{step}: {event}");
            assert_eq!(session.check_history_reach(), Ok(()), "{step}: {event}");
            let save = session.save_json().unwrap();
            let restored = ReactiveSession::restore_json(CITED, &save).err();
            assert_eq!(
                restored.as_deref(),
                (step >= 3).then_some(
                    "cannot restore save: state t grounds hold departed b records outside its lineage"
                ),
                "{step}: {event}"
            );
        }
    }

    /// A state's initializer grounds it as a set's value does: `qualified(1,
    /// e)` grounds `s` on `e` as the program loads, so `s`'s grounds can
    /// carry `e`'s records though no rule sets `s`.
    #[test]
    fn a_declaration_carries_what_its_initializer_grounds_on() {
        let session = ReactiveSession::from_source(
            "claim seen;
evidence e from \"e\";
renewable e window 2;
e supports seen;
state s = qualified(1, e);
event go;
on go renew e;
",
        )
        .unwrap();
        let grounds = &session.states.get("s").unwrap().grounds;
        assert!(!grounds.evidence.is_empty());
        assert!(session
            .history_reach_bound()
            .carries(&Owner::StateGrounds("s".into()), "e"));
    }

    /// A program every release from rc.1 to rc.11 loads, to which each case
    /// below adds lines.
    const BASE: &str = "claim c;
evidence e from \"e\";
state s = 0;
event go;
on go set s = s + 1;
";

    /// Lines that make a program no release from rc.1 to rc.11 loads, each
    /// checked against those releases' binaries (D083): a declaration, an
    /// effect or an expression that came later, and such an expression in
    /// each place the loaded program keeps one.
    const LATER: [(&str, &str); 55] = [
        ("journal window", "journal window 5;"),
        ("readings window", "readings r from e window 3;"),
        ("renewable window", "renewable e window 3;"),
        ("threshold", "threshold t = 3;"),
        ("routed work", "work skips other routes;"),
        ("agent", "agent bot sends go;"),
        ("irreversible", "decisions d limit 1 irreversible;"),
        ("strict", "decisions d limit 1 irreversible strict;"),
        ("forecloses", "decisions d2 limit 3;\ndecisions d limit 3 forecloses d2;"),
        ("integer clock", "event advance dt min 0 max 1;\nclock advance every 1 integer;"),
        ("unset state", "state u unset;"),
        ("changed trigger", "readings r from e limit 3;\ndecisions d limit 3 reopened by changed r;"),
        ("unqualify", "caveat k consequence low;\non go reveal e supports c;\non go unqualify e with k;"),
        ("qualify occurrence", "renewable e limit 2;\ncaveat k consequence low;\non go qualify occurrence(e, 1) with k;"),
        ("qualify latest", "readings r from e limit 2;\ncaveat k consequence low;\non go sample r = s supports c;\non go qualify latest(r) with k;"),
        ("withdraw occurrence", "renewable e limit 2;\nevidence w from \"w\";\non go reveal w supports c;\non go withdraw occurrence(e, 1) because w;"),
        ("withdraw every", "readings r from e limit 2;\nevidence w from \"w\";\non go reveal w supports c;\non go withdraw every r because w;"),
        ("reopen because a reading", "readings r from e limit 2;\ndecisions d limit 3;\non go commit d because enough;\non go reopen d because reading(r, 1);"),
        ("hold", "decisions d limit 3;\non go reveal e supports c;\non go commit d because enough;\non go hold d because e;"),
        ("release", "decisions d limit 3;\non go reveal e supports c;\non go commit d because enough;\non go release d because e;"),
        ("resolve", "decisions d limit 3;\non go reveal e supports c;\non go commit d because enough;\non go resolve d = 1 because e;"),
        ("neutral sample", "readings r from e limit 2;\non go sample r = s;"),
        ("sample noting", "readings r from e limit 2;\non go sample r = s supports c noting s;"),
        ("sample with", "readings r from e limit 2;\ncaveat k consequence low;\non go sample r = s supports c with k;"),
        ("commit citing", "decisions d limit 3;\non go commit d because enough citing s;"),
        ("retaining because", "decisions d limit 3;\ncaveat k consequence low;\nk qualifies e;\non go reveal e supports c;\non go commit d because enough retaining k because e;"),
        ("only_if", "on go set s = only_if(s > 0, 1);"),
        ("in_basis", "decisions d limit 3;\non go when in_basis(d, e) set s = 1;"),
        ("observed occurrence", "renewable e limit 3;\non go when observed(occurrence(e, 1)) set s = 1;"),
        ("withdrawn reading", "readings r from e limit 3;\non go when withdrawn(reading(r, 1)) set s = 1;"),
        ("carries latest", "readings r from e limit 3;\ncaveat k consequence low;\non go when carries(latest(r), k) set s = 1;"),
        ("revision", "decisions d limit 3;\non go when rests_on_withdrawn(revision(d, 1)) set s = 1;"),
        ("resolved revision", "decisions d limit 3;\non go when resolved(revision(d, 1)) set s = 1;"),
        ("grounds_of", "decisions d limit 3;\non go when committed(d) set s = grounds_of(d);"),
        ("resolved", "decisions d limit 3;\non go when resolved(d) set s = 1;"),
        ("outcome", "decisions d limit 3;\non go when committed(d) set s = outcome(d);"),
        ("on_hold", "decisions d limit 3;\non go when on_hold(d) set s = 1;"),
        ("carries_any", "on go when carries_any(e) set s = 1;"),
        ("carries_any reading", "readings r from e limit 3;\non go when carries_any(reading(r, 1), negligible) set s = 1;"),
        ("only_if in a step", "proc p(v) { set s = only_if(v > 0, v); };\non go call p(1);"),
        ("only_if in an uncalled procedure", "proc p(v) { set s = only_if(v > 0, v); };"),
        ("only_if in a template", "proc t(x evidence) { set s = only_if(observed(x), 1); };\non go call t(e);"),
        ("only_if in a guard", "on go when only_if(s > 0, 1) > 0 set s = 1;"),
        ("only_if in a binding", "bind hud.score = only_if(s > 0, s);"),
        ("only_if in a define", "define dd = only_if(s > 0, 1);"),
        ("only_if in a for block", "place p kind room;\nentity a kind beast at p;\nfor beast as $x {\n    on go set s = only_if(s > 0, 1);\n};"),
        ("only_if in a call's argument", "proc p(v) { set s = v; };\non go call p(only_if(s > 0, 1));"),
        ("only_if in a step's guard", "proc p(v) { when only_if(v > 0, 1) > 0 set s = v; };\non go call p(1);"),
        ("only_if in a commit's using", "decisions d limit 3;\non go commit d because enough using only_if(s > 0, 1);"),
        ("only_if in a permission's scope", "readings r from e limit 2;\ndecisions d limit 3;\non go sample r = s supports c;\non go commit d because enough permitted by latest(r) for only_if(s > 0, s);"),
        ("only_if in a qualify's delay", "event tick dt min 0 max 0.1;\ncaveat k consequence low;\non go reveal e supports c;\non go qualify e with k after only_if(s > 0, 5);"),
        ("only_if in a sample's value", "readings r from e limit 2;\non go sample r = only_if(s > 0, 1) supports c;"),
        ("only_if in a set's because", "on go reveal e supports c;\non go set s = 1 because only_if(s > 0, 1);"),
        ("only_if in a binding's condition", "bind hud.score = s when only_if(s > 0, 1) > 0;"),
        ("only_if in a binding's because", "bind hud.score = s because only_if(s > 0, 1);"),
    ];

    /// Lines that keep a program rc.11 loads, each checked against its
    /// binary: names spelled like a later word, functions named like a
    /// later form, and forms rc.11 reads.
    const OLDER: [(&str, &str); 31] = [
        ("base", ""),
        ("procedure", "proc p(v) { set s = v; };\non go call p(1);"),
        ("state named unset", "state unset = 0;"),
        ("evidence named threshold", "evidence threshold from \"t\";\nthreshold supports c;"),
        ("evidence named work", "evidence work from \"w\";\nwork supports c;"),
        ("series named strict", "decisions strict limit 3;"),
        ("stream named changed", "readings changed from e limit 2;\ndecisions d limit 3 reopened by changed;"),
        ("evidence named every", "evidence every from \"x\";\non go reveal e supports c;\non go reveal every supports c;\non go withdraw every because e;"),
        ("fn only_if", "fn only_if(a, b) = a + b;\non go set s = only_if(s, 1);"),
        ("fn outcome", "fn outcome(a) = a * 2;\non go set s = outcome(s);"),
        ("fn carries_any", "fn carries_any(a) = a;\non go set s = carries_any(s);"),
        ("fn in_basis", "state t = 0;\nfn in_basis(a, b) = a + b;\non go set s = in_basis(s, t);"),
        ("fn is_set", "fn is_set(a) = a > 0;\non go when is_set(s) set s = 0;"),
        ("withdrawn latest", "readings r from e limit 3;\non go when withdrawn(latest(r)) set s = 1;"),
        ("rests_on_withdrawn", "decisions d limit 3;\non go when rests_on_withdrawn(d) set s = 1;"),
        ("history", "decisions d limit 3;\non go set s = history_count(d);"),
        ("fold", "readings r from e limit 3;\nfn add(a, b) = a + b;\non go set s = fold_history(r, 0, add);"),
        ("has_sample", "readings r from e limit 3;\non go when has_sample(r) set s = 1;"),
        ("qualify after", "event tick dt min 0 max 0.1;\ncaveat k consequence low;\non go reveal e supports c;\non go qualify e with k after 5;"),
        ("permitted", "decisions d limit 3;\nevidence g from \"g\";\non go reveal g supports c;\non go commit d because enough permitted by g;"),
        ("retaining", "decisions d limit 3;\ncaveat k consequence low;\nk qualifies e;\non go commit d because enough retaining k;"),
        ("reopen latest", "decisions d limit 3;\nreadings r from e limit 2;\non go sample r = s supports c;\non go commit d because enough;\non go reopen d because latest(r);"),
        ("withdraw latest", "evidence w from \"w\";\nreadings r from e limit 2;\non go reveal w supports c;\non go sample r = s supports c;\non go withdraw latest(r) because w;"),
        ("neutral reveal", "on go reveal e;"),
        ("step guard", "proc p(v) { when v > 0 set s = v; };\non go call p(1);"),
        ("commit using", "decisions d limit 3;\non go commit d because enough using s;"),
        ("permission scope", "readings r from e limit 2;\ndecisions d limit 3;\non go sample r = s supports c;\non go commit d because enough permitted by latest(r) for s;"),
        ("sample value", "readings r from e limit 2;\non go sample r = 1 supports c;"),
        ("set because", "on go reveal e supports c;\non go set s = 1 because s;"),
        ("binding condition", "bind hud.score = s when s > 0;"),
        ("binding because", "bind hud.score = s because s;"),
    ];

    /// A parameter grounds on what its argument carries only in a program a
    /// release before rc.12 may have loaded (D083): `LATER`'s lines each
    /// rule that out, `OLDER`'s do not.
    #[test]
    fn only_a_program_an_old_release_may_load_keeps_old_parameter_grounds() {
        for (lines, may) in [(&LATER[..], false), (&OLDER[..], true)] {
            for (case, line) in lines {
                let source = format!("{BASE}{line}\n");
                let session = ReactiveSession::from_source(&source)
                    .unwrap_or_else(|error| panic!("{case}: {error}"));
                assert_eq!(session.may_load_before_rc12(), may, "{case}");
            }
        }
    }

    /// In a bundle module that declares `only_if` as a state, this runtime
    /// reads `only_if(…)` as the form. Releases before rc.12 read it as a
    /// call of the root's function `m__only_if`, and load the bundle when
    /// the root declares one, so there the form is no sign of a later
    /// program; without that function they refuse the bundle (D083). There
    /// the call takes numbers, and here the form's condition is a truth
    /// value: this runtime loads the module only because its linker leaves
    /// `hold` bare two words before `because`, so that `hold` reads the
    /// root's define, where those releases flattened it to the module's
    /// numeric state (finding D088). A function named like that for no
    /// module, or a module's own `fn only_if`, leaves `only_if` counted, and
    /// so does a single file.
    #[test]
    fn only_if_beside_a_flattened_function_keeps_old_parameter_grounds() {
        const MODULE: &str = "module m;
state only_if = 0;
state hold = 0;
state u = 0;
event tick;
on tick set u = only_if( hold , because );
";
        let root = format!("use m;\n{BASE}state because = 1;\ndefine hold = s > 0;\n");
        let bundle = |module: &str, root: &str| {
            format!(
                "#caveat-bundle 1\n#module m {}\n{module}#module main {}\n{root}",
                module.len(),
                root.len()
            )
        };
        let may = |source: &str| {
            ReactiveSession::from_source(source)
                .unwrap_or_else(|error| panic!("{source}: {error}"))
                .may_load_before_rc12()
        };
        assert!(may(&bundle(
            MODULE,
            &format!("{root}fn m__only_if(c, v) = v;\n")
        )));
        assert!(!may(&bundle(MODULE, &root)));
        assert!(!may(&bundle(
            MODULE,
            &format!("{root}fn other__only_if(c, v) = v;\n")
        )));
        let form = format!("use m;\n{BASE}on go set s = only_if(s > 0, 1);\n");
        assert!(!may(&bundle(
            "module m;\nfn only_if(c, v) = c * v;\n",
            &form
        )));
        assert!(!may(&format!(
            "{BASE}fn m__only_if(c, v) = v;\non go set s = only_if(s > 0, 1);\n"
        )));
    }
}
