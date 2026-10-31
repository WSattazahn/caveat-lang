//! The View 0.1 patch: how the whole View 0.1 view changed across one
//! accepted event, in a form a host applies to the view it already holds,
//! reusing every part the event left alone. See
//! spec/caveat-view-patch-0.1.md.
//!
//! A patch names only the members that changed. A map member gives its keys
//! in the view's order only when they changed, and the entries whose values
//! changed; a list member gives runs of the old list kept and new entries in
//! the new list's order. Applying it to the view as of `since` gives exactly
//! the view `dispatch_view_outcome_json` returns: the same values in the same
//! order, so the same JSON text.
use super::outcome::{DispatchFailure, DispatchFatal, RejectionCode, RejectionOrigin};
use super::{relation_name, BindingValue, DecisionSeries, JournalEntry, ReactiveSession};
use super::{DISPATCH_SCHEMA, REACTIVE_VIEW_SCHEMA};
use crate::chunked::Chunked;
use crate::fast_hash::HashMap;
use crate::map::{MapCommitment, MapRelation};
use crate::reactive_expr::Provenance;
use crate::{Edge, EpistemicGraph, NodeId};
use serde::Serialize;
use serde_json::value::RawValue;
use std::collections::BTreeMap;
use std::sync::Arc;

pub const REACTIVE_VIEW_PATCH_SCHEMA: &str = "caveat-reactive-view-patch/0.1";

/// What `dispatch_view_patch_outcome_json_from` returns: on acceptance the
/// patch from the view the host holds to the view after the event; a refusal
/// or fatal error exactly as `dispatch_view_outcome_json` reports it.
#[derive(Debug, Serialize)]
pub struct DispatchViewPatchOutcome {
    pub schema: &'static str,
    #[serde(flatten)]
    pub result: DispatchViewPatchResult,
}

#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum DispatchViewPatchResult {
    Accepted {
        patch: Box<ViewPatch>,
    },
    Rejected {
        origin: RejectionOrigin,
        code: RejectionCode,
        message: String,
    },
}

/// How the View 0.1 view changed. `since` is the sequence of the view it
/// applies to, or null when it applies to nothing and so gives every member.
#[derive(Debug, Serialize)]
pub struct ViewPatch {
    pub schema: &'static str,
    pub since: Option<u64>,
    pub view_schema: &'static str,
    pub sequence: u64,
    pub last_event: Option<String>,
    /// The members that changed, by name.
    pub members: BTreeMap<&'static str, MemberChange>,
}

/// How one member changed.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum MemberChange {
    /// The member's new value, whole.
    Whole { value: Box<RawValue> },
    /// A map member.
    Map(MapChange),
    /// A list member: `[start, length]` runs of the old list and new entries.
    List { items: Vec<Item> },
}

/// `keys`, every key in the view's order, only when they changed, and
/// `set`, the entries that are new or changed.
#[derive(Debug, Default, Serialize)]
pub struct MapChange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,
    pub set: BTreeMap<String, Entry>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Entry {
    Value(Box<RawValue>),
    Map(MapChange),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Item {
    Run([usize; 2]),
    New(Box<RawValue>),
}

/// The view as of one sequence, kept between calls so that each event builds
/// one view, not two. A caller keeps it only while nothing but
/// `dispatch_view_patch_outcome_json_from` changes the session.
#[derive(Debug, Default)]
pub struct ViewPatchBase(Option<(u64, PatchParts)>);

impl ViewPatchBase {
    /// Forget the kept view, as after anything else changes the session.
    pub fn clear(&mut self) {
        self.0 = None;
    }
}

/// View 0.1 as the host holds it: a copy of the session's members as of the
/// last patch, brought up to date by each patch, so that an event costs what
/// it changed rather than a copy of everything. Members the session shares
/// are held shared, so a member the event left alone is the same allocation
/// on both sides.
#[derive(Debug)]
pub(super) struct PatchParts {
    bindings: Nested<BindingValue>,
    binding_explanations: Nested<Provenance>,
    cues: String,
    effects: String,
    commitments: BTreeMap<String, MapCommitment>,
    commitment_grounds: Arc<BTreeMap<String, Provenance>>,
    decision_series: Arc<BTreeMap<String, DecisionSeries>>,
    decision_journal: Chunked<JournalEntry>,
    graph: Arc<EpistemicGraph>,
    symbols: Arc<HashMap<String, NodeId>>,
    /// Each node's name, as of `symbols`.
    names: Arc<HashMap<NodeId, String>>,
}

type Nested<V> = BTreeMap<String, BTreeMap<String, V>>;

impl ReactiveSession {
    /// Dispatch one event and return the View 0.1 patch from the view as of
    /// `held` to the view after the event. The same transaction as
    /// `dispatch_view_outcome_json`. When `base` does not hold the view as
    /// of `held`, or `held` is `None`, the patch applies to nothing and
    /// gives every member.
    pub fn dispatch_view_patch_outcome_json_from(
        &mut self,
        base: &mut ViewPatchBase,
        held: Option<u64>,
        event: &str,
        payload_json: &str,
        agent: Option<&str>,
    ) -> Result<DispatchViewPatchOutcome, DispatchFatal> {
        let since = self.sequence;
        let before = match (base.0.take(), held) {
            (Some((sequence, parts)), Some(held)) if sequence == since && held == since => {
                Some(parts)
            }
            _ => None,
        };
        match self.dispatch_classified(event, payload_json, agent) {
            Ok(()) => {
                let given = before.is_some().then_some(since);
                let (members, parts) = match before {
                    Some(mut parts) => (parts.sync(self, false), parts),
                    None => {
                        let mut parts = PatchParts::new(self);
                        (parts.sync(self, true), parts)
                    }
                };
                let patch = ViewPatch {
                    schema: REACTIVE_VIEW_PATCH_SCHEMA,
                    since: given,
                    view_schema: REACTIVE_VIEW_SCHEMA,
                    sequence: self.sequence,
                    last_event: self.last_event.clone(),
                    members,
                };
                base.0 = Some((self.sequence, parts));
                Ok(DispatchViewPatchOutcome {
                    schema: DISPATCH_SCHEMA,
                    result: DispatchViewPatchResult::Accepted {
                        patch: Box::new(patch),
                    },
                })
            }
            Err(failure) => {
                let outcome = patch_outcome(failure);
                // A refusal changes nothing, so the view still stands.
                if outcome.is_ok() {
                    if let Some(parts) = before {
                        base.0 = Some((since, parts));
                    }
                }
                outcome
            }
        }
    }
}

fn patch_outcome(failure: DispatchFailure) -> Result<DispatchViewPatchOutcome, DispatchFatal> {
    let (origin, code, message) = failure.classify()?;
    Ok(DispatchViewPatchOutcome {
        schema: DISPATCH_SCHEMA,
        result: DispatchViewPatchResult::Rejected {
            origin,
            code,
            message,
        },
    })
}

/// The value's JSON text, exactly as the whole view writes it.
fn raw<T: Serialize + ?Sized>(value: &T) -> Box<RawValue> {
    RawValue::from_string(text(value)).expect("serialized JSON")
}

fn text<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string(value).expect("finite reactive view")
}

/// The same JSON text. Sound for any value: equal text is what the view
/// shows, and unequal text is only ever sent again.
fn same<T: Serialize + ?Sized>(a: &T, b: &T) -> bool {
    text(a) == text(b)
}

/// A number compares by its bits, so `-0` stays apart from `0`; every other
/// binding value has one form.
fn same_binding(a: &BindingValue, b: &BindingValue) -> bool {
    match (a, b) {
        (BindingValue::Number(x), BindingValue::Number(y)) => x.to_bits() == y.to_bits(),
        _ => a == b,
    }
}

fn node_names(symbols: &HashMap<String, NodeId>) -> HashMap<NodeId, String> {
    symbols
        .iter()
        .map(|(name, id)| (*id, name.clone()))
        .collect()
}

impl PatchParts {
    /// Parts that hold nothing yet: the first `sync`, with `full`, fills them.
    fn new(session: &ReactiveSession) -> Self {
        Self {
            bindings: BTreeMap::new(),
            binding_explanations: BTreeMap::new(),
            cues: String::new(),
            effects: String::new(),
            commitments: BTreeMap::new(),
            commitment_grounds: Arc::clone(&session.commitment_grounds),
            decision_series: Arc::clone(&session.decision_series),
            decision_journal: session.journal.clone(),
            graph: Arc::clone(&session.graph),
            symbols: Arc::clone(&session.symbols),
            names: Arc::new(node_names(&session.symbols)),
        }
    }

    /// Bring the parts up to date with the session and say what changed;
    /// with `full`, every member, as to a host that holds nothing.
    fn sync(
        &mut self,
        session: &ReactiveSession,
        full: bool,
    ) -> BTreeMap<&'static str, MemberChange> {
        let mut members = BTreeMap::new();
        let mut put = |name: &'static str, change: Option<MemberChange>| {
            if let Some(change) = change {
                members.insert(name, change);
            }
        };
        put(
            "bindings",
            sync_nested(&mut self.bindings, &session.bindings, full, &same_binding),
        );
        // Provenance holds only names and counts, so equal values are equal
        // JSON.
        put(
            "binding_explanations",
            sync_nested(
                &mut self.binding_explanations,
                &session.binding_explanations,
                full,
                &|old, new| old == new,
            ),
        );
        put("cues", sync_whole(&mut self.cues, &session.cues, full));
        put(
            "effects",
            sync_whole(&mut self.effects, &session.effects, full),
        );

        let old_symbols = std::mem::replace(&mut self.symbols, Arc::clone(&session.symbols));
        let old_names = Arc::clone(&self.names);
        let symbols_changed = !Arc::ptr_eq(&old_symbols, &self.symbols);
        if symbols_changed {
            self.names = Arc::new(node_names(&self.symbols));
        }
        let old_graph = std::mem::replace(&mut self.graph, Arc::clone(&session.graph));
        let graph_changed = !Arc::ptr_eq(&old_graph, &self.graph);

        // A commitment holds only names and flags.
        if full || graph_changed || symbols_changed {
            let names = &self.names;
            let live = session
                .commitment_records(|id| names.get(id).map(String::as_str))
                .into_iter()
                .map(|commitment| (commitment.action.clone(), commitment))
                .collect::<BTreeMap<_, _>>();
            put(
                "commitments",
                sync_map(&mut self.commitments, &live, full, &|old, new| old == new)
                    .map(MemberChange::Map),
            );
        }
        let old_grounds = std::mem::replace(
            &mut self.commitment_grounds,
            Arc::clone(&session.commitment_grounds),
        );
        if full || !Arc::ptr_eq(&old_grounds, &self.commitment_grounds) {
            put(
                "commitment_grounds",
                map_changes(
                    (!full).then_some(old_grounds.as_ref()),
                    &self.commitment_grounds,
                    &|old, new| old == new,
                )
                .map(MemberChange::Map),
            );
        }
        let old_series = std::mem::replace(
            &mut self.decision_series,
            Arc::clone(&session.decision_series),
        );
        if full || !Arc::ptr_eq(&old_series, &self.decision_series) {
            put(
                "decision_series",
                map_changes(
                    (!full).then_some(old_series.as_ref()),
                    &self.decision_series,
                    &|old, new| same(old, new),
                )
                .map(MemberChange::Map),
            );
        }
        let old_journal = std::mem::replace(&mut self.decision_journal, session.journal.clone());
        put(
            "decision_journal",
            list_changes(
                (!full)
                    .then(|| old_journal.iter().collect::<Vec<_>>())
                    .as_deref(),
                &self.decision_journal.iter().collect::<Vec<_>>(),
                |old, new| std::ptr::eq(*old, *new) || same(*old, *new),
                |entry| raw(*entry),
            ),
        );
        if full || graph_changed || symbols_changed {
            let names = &self.names;
            let render = |edge: &Edge| raw(&relation(names, edge));
            let edges = |old: &Edge, new: &Edge| {
                old.from == new.from && old.to == new.to && old.relation == new.relation
            };
            put(
                "relations",
                if full {
                    list_changes(None, &self.graph.edges, edges, render)
                } else if !symbols_changed {
                    list_changes(Some(&old_graph.edges), &self.graph.edges, edges, render)
                } else {
                    list_changes(
                        Some(&old_graph.edges),
                        &self.graph.edges,
                        |old, new| {
                            edges(old, new)
                                && old_names.get(&old.from) == names.get(&new.from)
                                && old_names.get(&old.to) == names.get(&new.to)
                        },
                        render,
                    )
                },
            );
        }
        members
    }
}

/// A member a host replaces whole when its text changes.
fn sync_whole<T: Serialize + ?Sized>(
    held: &mut String,
    live: &T,
    full: bool,
) -> Option<MemberChange> {
    let now = text(live);
    if !full && *held == now {
        return None;
    }
    let value = RawValue::from_string(now.clone()).expect("serialized JSON");
    *held = now;
    Some(MemberChange::Whole { value })
}

/// Bring a held map up to date: `keys` when they changed, and the entries
/// that are new or changed. `None` when nothing changed.
fn sync_map<V: Clone + Serialize>(
    held: &mut BTreeMap<String, V>,
    live: &BTreeMap<String, V>,
    full: bool,
    same: &dyn Fn(&V, &V) -> bool,
) -> Option<MapChange> {
    let keys_changed = full || !held.keys().eq(live.keys());
    if keys_changed {
        held.retain(|key, _| live.contains_key(key));
    }
    let mut set = BTreeMap::new();
    for (key, value) in live {
        match held.get_mut(key) {
            Some(known) if same(known, value) => {}
            Some(known) => {
                *known = value.clone();
                set.insert(key.clone(), Entry::Value(raw(value)));
            }
            None => {
                held.insert(key.clone(), value.clone());
                set.insert(key.clone(), Entry::Value(raw(value)));
            }
        }
    }
    (keys_changed || !set.is_empty()).then(|| MapChange {
        keys: keys_changed.then(|| live.keys().cloned().collect()),
        set,
    })
}

/// A held map of maps: as `sync_map`, with each changed outer entry itself a
/// `MapChange`, every key given for an entry the host has not got.
fn sync_nested<V: Clone + Serialize>(
    held: &mut Nested<V>,
    live: &Nested<V>,
    full: bool,
    same: &dyn Fn(&V, &V) -> bool,
) -> Option<MemberChange> {
    let keys_changed = full || !held.keys().eq(live.keys());
    if keys_changed {
        held.retain(|key, _| live.contains_key(key));
    }
    let mut set = BTreeMap::new();
    for (key, properties) in live {
        let change = match held.get_mut(key) {
            Some(known) => sync_map(known, properties, full, same),
            None => {
                let mut known = BTreeMap::new();
                let change = sync_map(&mut known, properties, true, same);
                held.insert(key.clone(), known);
                change
            }
        };
        if let Some(change) = change {
            set.insert(key.clone(), Entry::Map(change));
        }
    }
    (keys_changed || !set.is_empty()).then(|| {
        MemberChange::Map(MapChange {
            keys: keys_changed.then(|| live.keys().cloned().collect()),
            set,
        })
    })
}

/// A shared map against the one held before it: `keys` when they changed,
/// and the entries that are new or changed; every key and entry when
/// nothing was held. `None` when nothing changed.
fn map_changes<V: Serialize>(
    before: Option<&BTreeMap<String, V>>,
    after: &BTreeMap<String, V>,
    same: &dyn Fn(&V, &V) -> bool,
) -> Option<MapChange> {
    let keys_changed = before.is_none_or(|old| !old.keys().eq(after.keys()));
    let set = after
        .iter()
        .filter(|(key, new)| {
            !before
                .and_then(|old| old.get(*key))
                .is_some_and(|known| same(known, new))
        })
        .map(|(key, new)| (key.clone(), Entry::Value(raw(new))))
        .collect::<BTreeMap<_, _>>();
    (keys_changed || !set.is_empty()).then(|| MapChange {
        keys: keys_changed.then(|| after.keys().cloned().collect()),
        set,
    })
}

/// A list member as runs of the old list kept and new entries. The runtime
/// only drops entries from these lists and appends new ones, so the old
/// entries still present are a subsequence of the new list's head; matching
/// greedily from the front keeps each old entry that is next in the new
/// list. Every entry when nothing was held; `None` when nothing changed.
fn list_changes<T>(
    before: Option<&[T]>,
    after: &[T],
    same: impl Fn(&T, &T) -> bool,
    render: impl Fn(&T) -> Box<RawValue>,
) -> Option<MemberChange> {
    let old = before.unwrap_or_default();
    let mut items = Vec::new();
    let mut run: Option<(usize, usize)> = None;
    let mut kept = 0;
    for (position, entry) in old.iter().enumerate() {
        if kept < after.len() && same(entry, &after[kept]) {
            kept += 1;
            run = match run {
                Some((start, length)) if start + length == position => Some((start, length + 1)),
                Some((start, length)) => {
                    items.push(Item::Run([start, length]));
                    Some((position, 1))
                }
                None => Some((position, 1)),
            };
        }
    }
    if before.is_some() && kept == old.len() && kept == after.len() {
        return None;
    }
    if let Some((start, length)) = run {
        items.push(Item::Run([start, length]));
    }
    items.extend(after[kept..].iter().map(|entry| Item::New(render(entry))));
    Some(MemberChange::List { items })
}

fn relation(names: &HashMap<NodeId, String>, edge: &Edge) -> MapRelation {
    MapRelation {
        from: names[&edge.from].clone(),
        to: names[&edge.to].clone(),
        relation: relation_name(edge.relation).into(),
        origin: "live".into(),
    }
}
