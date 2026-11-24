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
use super::{
    relation_name, BindingGroup, BindingValue, DecisionSeries, EffectReport, JournalEntry,
    NameTable, ReactiveSession,
};
use super::{DISPATCH_SCHEMA, REACTIVE_VIEW_SCHEMA};
use crate::chunked::Chunked;
use crate::map::{MapCommitment, MapRelation};
use crate::reactive_expr::Provenance;
use crate::{Edge, EpistemicGraph, NodeId, NodeKind, Relation};
use serde::Serialize;
use serde_json::value::RawValue;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
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
    effects: Vec<EffectReport>,
    commitments: BTreeMap<String, MapCommitment>,
    commitment_grounds: Arc<BTreeMap<String, Provenance>>,
    decision_series: Arc<BTreeMap<String, DecisionSeries>>,
    decision_journal: Chunked<JournalEntry>,
    graph: Arc<EpistemicGraph>,
    symbols: NameTable,
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

    /// `dispatch_view_patch_outcome_json_from` serialized straight to JSON,
    /// avoiding intermediate `RawValue` strings and JSON re-validation on the
    /// hot WebAssembly path. Byte-for-byte identical to serializing
    /// `dispatch_view_patch_outcome_json_from`.
    pub fn dispatch_view_patch_outcome_text_from(
        &mut self,
        base: &mut ViewPatchBase,
        held: Option<u64>,
        event: &str,
        payload_json: &str,
        agent: Option<&str>,
    ) -> Result<String, DispatchFatal> {
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
                let mut out = Vec::with_capacity(2048);
                out.extend_from_slice(br#"{"schema":""#);
                out.extend_from_slice(DISPATCH_SCHEMA.as_bytes());
                out.extend_from_slice(br#"","outcome":"accepted","patch":{"schema":""#);
                out.extend_from_slice(REACTIVE_VIEW_PATCH_SCHEMA.as_bytes());
                out.extend_from_slice(br#"","since":"#);
                match given {
                    Some(seq) => {
                        let _ = write!(out, "{seq}");
                    }
                    None => out.extend_from_slice(b"null"),
                }
                out.extend_from_slice(br#","view_schema":""#);
                out.extend_from_slice(REACTIVE_VIEW_SCHEMA.as_bytes());
                out.extend_from_slice(br#"","sequence":"#);
                let _ = write!(out, "{}", self.sequence);
                out.extend_from_slice(br#","last_event":"#);
                serde_json::to_writer(&mut out, &self.last_event).expect("finite last_event");
                out.extend_from_slice(br#","members":{"#);
                let parts = match before {
                    Some(mut parts) => {
                        parts.sync_json(self, false, &mut out);
                        parts
                    }
                    None => {
                        let mut parts = PatchParts::new(self);
                        parts.sync_json(self, true, &mut out);
                        parts
                    }
                };
                out.extend_from_slice(b"}}}");
                base.0 = Some((self.sequence, parts));
                Ok(String::from_utf8(out).expect("serde_json produces valid UTF-8"))
            }
            Err(failure) => {
                let outcome = patch_outcome(failure);
                if outcome.is_ok() {
                    if let Some(parts) = before {
                        base.0 = Some((since, parts));
                    }
                }
                outcome.map(|outcome| {
                    serde_json::to_string(&outcome).expect("serializable rejected outcome")
                })
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

fn same_opt_f64(a: Option<f64>, b: Option<f64>) -> bool {
    a.map(f64::to_bits) == b.map(f64::to_bits)
}

/// Equal `DecisionSeries` values, comparing outcome numbers by bits so `-0`
/// stays apart from `0` without serializing both sides to JSON.
fn same_decision_series(a: &DecisionSeries, b: &DecisionSeries) -> bool {
    a == b
        && a.revisions.iter().zip(&b.revisions).all(|(x, y)| {
            same_opt_f64(
                x.outcome.as_ref().map(|o| o.value),
                y.outcome.as_ref().map(|o| o.value),
            )
        })
}

/// Equal `JournalEntry` values, comparing numeric fields by bits so `-0` stays
/// apart from `0` without serializing both sides to JSON.
fn same_journal_entry(a: &JournalEntry, b: &JournalEntry) -> bool {
    a == b
        && same_opt_f64(a.elapsed, b.elapsed)
        && same_opt_f64(a.value, b.value)
        && same_opt_f64(a.outcome, b.outcome)
}

/// A number compares by its bits, so `-0` stays apart from `0`; every other
/// binding value has one form.
fn same_binding(a: &BindingValue, b: &BindingValue) -> bool {
    match (a, b) {
        (BindingValue::Number(x), BindingValue::Number(y)) => x.to_bits() == y.to_bits(),
        _ => a == b,
    }
}

/// Equal `EffectReport` values, comparing sample numbers by bits so `-0`
/// stays apart from `0` without serializing both sides to JSON.
fn same_effect(a: &EffectReport, b: &EffectReport) -> bool {
    match (a, b) {
        (
            EffectReport::Sample {
                stream: s1,
                id: i1,
                value: v1,
                relation: r1,
                target: t1,
                note: n1,
            },
            EffectReport::Sample {
                stream: s2,
                id: i2,
                value: v2,
                relation: r2,
                target: t2,
                note: n2,
            },
        ) => {
            v1.to_bits() == v2.to_bits() && s1 == s2 && i1 == i2 && r1 == r2 && t1 == t2 && n1 == n2
        }
        _ => a == b,
    }
}

fn same_effects(a: &[EffectReport], b: &[EffectReport]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| same_effect(x, y))
}

impl PatchParts {
    /// Parts that hold nothing yet: the first `sync`, with `full`, fills them.
    fn new(session: &ReactiveSession) -> Self {
        Self {
            bindings: BTreeMap::new(),
            binding_explanations: BTreeMap::new(),
            cues: String::new(),
            effects: Vec::new(),
            commitments: BTreeMap::new(),
            commitment_grounds: Arc::clone(&session.commitment_grounds),
            decision_series: Arc::clone(&session.decision_series),
            decision_journal: session.journal.clone(),
            graph: Arc::clone(&session.graph),
            symbols: session.symbols.clone(),
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
        if full {
            put(
                "bindings",
                sync_nested(&mut self.bindings, &session.bindings, true, &same_binding),
            );
            put(
                "binding_explanations",
                sync_nested(
                    &mut self.binding_explanations,
                    &session.binding_explanations,
                    true,
                    &|old, new| old == new,
                ),
            );
        } else if !session.last_binding_groups.is_empty() {
            put(
                "bindings",
                sync_nested_touched(
                    &mut self.bindings,
                    &session.bindings,
                    &session.binding_groups,
                    &session.last_binding_groups,
                    &same_binding,
                ),
            );
            put(
                "binding_explanations",
                sync_nested_touched(
                    &mut self.binding_explanations,
                    &session.binding_explanations,
                    &session.binding_groups,
                    &session.last_binding_groups,
                    &|old, new| old == new,
                ),
            );
        }
        put("cues", sync_whole_list(&mut self.cues, &session.cues, full));
        if full || !same_effects(&self.effects, &session.effects) {
            self.effects.clone_from(&session.effects);
            put(
                "effects",
                Some(MemberChange::Whole {
                    value: raw(&session.effects),
                }),
            );
        }

        let old_symbols = std::mem::replace(&mut self.symbols, session.symbols.clone());
        let symbols_changed = !old_symbols.shares_all(&self.symbols);
        let old_graph = std::mem::replace(&mut self.graph, Arc::clone(&session.graph));
        let graph_changed = !Arc::ptr_eq(&old_graph, &self.graph);

        if full || graph_changed || symbols_changed {
            let names_kept = !full && (!symbols_changed || old_symbols.kept_in(&self.symbols));
            let edges_appended = !full && self.graph.edges.starts_with(&old_graph.edges);

            // Commitments: in the common case (edges appended, existing names
            // and origins preserved), diff only changed node blocks and new
            // Retains/Reopens edges instead of rebuilding every commitment.
            let commitments_change = if names_kept
                && edges_appended
                && old_graph.origins.shares_all(&self.graph.origins)
            {
                let mut changed = Vec::new();
                let mut removed = false;
                old_graph.nodes.diff(&self.graph.nodes, |id, old, new| {
                    if matches!(old, Some(NodeKind::Commitment { .. }))
                        && !matches!(new, Some(NodeKind::Commitment { .. }))
                    {
                        removed = true;
                        return false;
                    }
                    if matches!(new, Some(NodeKind::Commitment { .. })) {
                        changed.push(id);
                    }
                    true
                });
                if removed {
                    let live = session
                        .commitment_records(|id| self.symbols.name_of(*id))
                        .into_iter()
                        .map(|commitment| (commitment.action.clone(), commitment))
                        .collect::<BTreeMap<_, _>>();
                    sync_map(&mut self.commitments, &live, false, &|old, new| old == new)
                        .map(MemberChange::Map)
                } else {
                    for edge in self.graph.edges.iter_from(old_graph.edges.len()) {
                        match edge.relation {
                            Relation::Retains => changed.push(edge.from),
                            Relation::Reopens => changed.push(edge.to),
                            _ => {}
                        }
                    }
                    if changed.is_empty() {
                        None
                    } else {
                        changed.sort_unstable();
                        changed.dedup();
                        let mut keys_changed = false;
                        let mut set = BTreeMap::new();
                        let name_of = |id: &NodeId| self.symbols.name_of(*id);
                        for id in changed {
                            let Some(record) = session.commitment_record(id, &name_of) else {
                                continue;
                            };
                            let action = record.action.clone();
                            match self.commitments.get_mut(&action) {
                                Some(known) if *known == record => {}
                                Some(known) => {
                                    *known = record;
                                    set.insert(action, Entry::Value(raw(known)));
                                }
                                None => {
                                    keys_changed = true;
                                    let rendered = raw(&record);
                                    self.commitments.insert(action.clone(), record);
                                    set.insert(action, Entry::Value(rendered));
                                }
                            }
                        }
                        (keys_changed || !set.is_empty()).then(|| {
                            MemberChange::Map(MapChange {
                                keys: keys_changed
                                    .then(|| self.commitments.keys().cloned().collect()),
                                set,
                            })
                        })
                    }
                }
            } else {
                let live = session
                    .commitment_records(|id| self.symbols.name_of(*id))
                    .into_iter()
                    .map(|commitment| (commitment.action.clone(), commitment))
                    .collect::<BTreeMap<_, _>>();
                sync_map(&mut self.commitments, &live, full, &|old, new| old == new)
                    .map(MemberChange::Map)
            };
            put("commitments", commitments_change);

            // Relations: fast append-only path skips collecting or scanning
            // shared edge chunks.
            let relations_change = if full {
                Some(MemberChange::List {
                    items: self
                        .graph
                        .edges
                        .iter()
                        .map(|edge| Item::New(raw(&relation(&self.symbols, edge))))
                        .collect(),
                })
            } else if edges_appended && names_kept {
                let old_len = old_graph.edges.len();
                let new_len = self.graph.edges.len();
                if old_len == new_len {
                    None
                } else {
                    let mut items =
                        Vec::with_capacity((old_len > 0) as usize + (new_len - old_len));
                    if old_len > 0 {
                        items.push(Item::Run([0, old_len]));
                    }
                    items.extend(
                        self.graph
                            .edges
                            .iter_from(old_len)
                            .map(|edge| Item::New(raw(&relation(&self.symbols, edge)))),
                    );
                    Some(MemberChange::List { items })
                }
            } else {
                let render = |edge: &&Edge| raw(&relation(&self.symbols, edge));
                let edges = |old: &&Edge, new: &&Edge| {
                    old.from == new.from && old.to == new.to && old.relation == new.relation
                };
                let old_edges = old_graph.edges.iter().collect::<Vec<_>>();
                let new_edges = self.graph.edges.iter().collect::<Vec<_>>();
                if !symbols_changed {
                    list_changes(Some(&old_edges), &new_edges, edges, render)
                } else {
                    list_changes(
                        Some(&old_edges),
                        &new_edges,
                        |old, new| {
                            edges(old, new)
                                && old_symbols.name_of(old.from) == self.symbols.name_of(new.from)
                                && old_symbols.name_of(old.to) == self.symbols.name_of(new.to)
                        },
                        render,
                    )
                }
            };
            put("relations", relations_change);
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
                    &same_decision_series,
                )
                .map(MemberChange::Map),
            );
        }
        let old_journal = std::mem::replace(&mut self.decision_journal, session.journal.clone());
        let journal_change = if full {
            Some(MemberChange::List {
                items: self
                    .decision_journal
                    .iter()
                    .map(|entry| Item::New(raw(entry)))
                    .collect(),
            })
        } else if self.decision_journal.shares_all(&old_journal) {
            None
        } else if self
            .decision_journal
            .starts_with_by(&old_journal, same_journal_entry)
        {
            let old_len = old_journal.len();
            let new_len = self.decision_journal.len();
            if old_len == new_len {
                None
            } else {
                let mut items = Vec::with_capacity((old_len > 0) as usize + (new_len - old_len));
                if old_len > 0 {
                    items.push(Item::Run([0, old_len]));
                }
                items.extend(
                    self.decision_journal
                        .iter_from(old_len)
                        .map(|entry| Item::New(raw(entry))),
                );
                Some(MemberChange::List { items })
            }
        } else {
            list_changes(
                Some(&old_journal.iter().collect::<Vec<_>>()),
                &self.decision_journal.iter().collect::<Vec<_>>(),
                |old, new| std::ptr::eq(*old, *new) || same_journal_entry(old, new),
                |entry| raw(*entry),
            )
        };
        put("decision_journal", journal_change);
        #[cfg(debug_assertions)]
        if !full {
            assert_eq!(
                text(&self.bindings),
                text(&session.bindings),
                "incremental view patch bindings drifted from session"
            );
            assert_eq!(
                self.binding_explanations, session.binding_explanations,
                "incremental view patch binding_explanations drifted from session"
            );
            let live_commitments = session
                .commitment_records(|id| self.symbols.name_of(*id))
                .into_iter()
                .map(|commitment| (commitment.action.clone(), commitment))
                .collect::<BTreeMap<_, _>>();
            assert_eq!(
                self.commitments, live_commitments,
                "incremental view patch commitments drifted from session"
            );
        }
        members
    }

    /// Bring the parts up to date with `session` and stream the `members`
    /// object body straight to `out` in strict `BTreeMap` member order:
    /// `binding_explanations`, `bindings`, `commitment_grounds`, `commitments`,
    /// `cues`, `decision_journal`, `decision_series`, `effects`, `relations`.
    fn sync_json(&mut self, session: &ReactiveSession, full: bool, out: &mut Vec<u8>) {
        let mut first = true;

        // 1. binding_explanations
        if full {
            write_member_head(out, &mut first, "binding_explanations");
            write_nested_full(
                out,
                &mut self.binding_explanations,
                &session.binding_explanations,
            );
        } else if !session.last_binding_groups.is_empty() {
            write_nested_touched(
                out,
                &mut first,
                "binding_explanations",
                &mut self.binding_explanations,
                &session.binding_explanations,
                &session.binding_groups,
                &session.last_binding_groups,
                &|old, new| old == new,
            );
        }

        // 2. bindings
        if full {
            write_member_head(out, &mut first, "bindings");
            write_nested_full(out, &mut self.bindings, &session.bindings);
        } else if !session.last_binding_groups.is_empty() {
            write_nested_touched(
                out,
                &mut first,
                "bindings",
                &mut self.bindings,
                &session.bindings,
                &session.binding_groups,
                &session.last_binding_groups,
                &same_binding,
            );
        }

        // 3. commitment_grounds
        let old_grounds = std::mem::replace(
            &mut self.commitment_grounds,
            Arc::clone(&session.commitment_grounds),
        );
        if full || !Arc::ptr_eq(&old_grounds, &self.commitment_grounds) {
            write_map_changes(
                out,
                &mut first,
                "commitment_grounds",
                (!full).then_some(old_grounds.as_ref()),
                &self.commitment_grounds,
                &|old, new| old == new,
            );
        }

        // Prepare graph / symbol deltas once for commitments (4) and relations (9).
        let old_symbols = std::mem::replace(&mut self.symbols, session.symbols.clone());
        let symbols_changed = !old_symbols.shares_all(&self.symbols);
        let old_graph = std::mem::replace(&mut self.graph, Arc::clone(&session.graph));
        let graph_changed = !Arc::ptr_eq(&old_graph, &self.graph);
        let names_kept = !full && (!symbols_changed || old_symbols.kept_in(&self.symbols));
        let edges_appended = !full && self.graph.edges.starts_with(&old_graph.edges);

        // 4. commitments
        if full || graph_changed || symbols_changed {
            if names_kept && edges_appended && old_graph.origins.shares_all(&self.graph.origins) {
                let mut changed = Vec::new();
                let mut removed = false;
                old_graph.nodes.diff(&self.graph.nodes, |id, old, new| {
                    if matches!(old, Some(NodeKind::Commitment { .. }))
                        && !matches!(new, Some(NodeKind::Commitment { .. }))
                    {
                        removed = true;
                        return false;
                    }
                    if matches!(new, Some(NodeKind::Commitment { .. })) {
                        changed.push(id);
                    }
                    true
                });
                if removed {
                    let live = session
                        .commitment_records(|id| self.symbols.name_of(*id))
                        .into_iter()
                        .map(|commitment| (commitment.action.clone(), commitment))
                        .collect::<BTreeMap<_, _>>();
                    write_sync_map(
                        out,
                        &mut first,
                        "commitments",
                        &mut self.commitments,
                        &live,
                        false,
                        &|old, new| old == new,
                    );
                } else {
                    for edge in self.graph.edges.iter_from(old_graph.edges.len()) {
                        match edge.relation {
                            Relation::Retains => changed.push(edge.from),
                            Relation::Reopens => changed.push(edge.to),
                            _ => {}
                        }
                    }
                    if !changed.is_empty() {
                        changed.sort_unstable();
                        changed.dedup();
                        let mut keys_changed = false;
                        let mut set_actions = BTreeSet::new();
                        let name_of = |id: &NodeId| self.symbols.name_of(*id);
                        for id in changed {
                            let Some(record) = session.commitment_record(id, &name_of) else {
                                continue;
                            };
                            let action = record.action.clone();
                            match self.commitments.get_mut(&action) {
                                Some(known) if *known == record => {}
                                Some(known) => {
                                    *known = record;
                                    set_actions.insert(action);
                                }
                                None => {
                                    keys_changed = true;
                                    self.commitments.insert(action.clone(), record);
                                    set_actions.insert(action);
                                }
                            }
                        }
                        if keys_changed || !set_actions.is_empty() {
                            write_member_head(out, &mut first, "commitments");
                            out.push(b'{');
                            if keys_changed {
                                write_keys_field(out, self.commitments.keys().map(String::as_str));
                                out.push(b',');
                            }
                            out.extend_from_slice(br#""set":{"#);
                            for (idx, action) in set_actions.iter().enumerate() {
                                if idx > 0 {
                                    out.push(b',');
                                }
                                serde_json::to_writer(&mut *out, action).expect("finite string");
                                out.push(b':');
                                serde_json::to_writer(&mut *out, &self.commitments[action])
                                    .expect("finite commitment");
                            }
                            out.extend_from_slice(b"}}");
                        }
                    }
                }
            } else {
                let live = session
                    .commitment_records(|id| self.symbols.name_of(*id))
                    .into_iter()
                    .map(|commitment| (commitment.action.clone(), commitment))
                    .collect::<BTreeMap<_, _>>();
                write_sync_map(
                    out,
                    &mut first,
                    "commitments",
                    &mut self.commitments,
                    &live,
                    full,
                    &|old, new| old == new,
                );
            }
        }

        // 5. cues
        if full || !(session.cues.is_empty() && self.cues == "[]") {
            let now = if session.cues.is_empty() {
                "[]".to_owned()
            } else {
                text(&session.cues)
            };
            if full || self.cues != now {
                write_member_head(out, &mut first, "cues");
                out.extend_from_slice(br#"{"value":"#);
                out.extend_from_slice(now.as_bytes());
                out.push(b'}');
                self.cues = now;
            }
        }

        // 6. decision_journal
        let old_journal = std::mem::replace(&mut self.decision_journal, session.journal.clone());
        if full {
            write_member_head(out, &mut first, "decision_journal");
            out.extend_from_slice(br#"{"items":["#);
            for (idx, entry) in self.decision_journal.iter().enumerate() {
                if idx > 0 {
                    out.push(b',');
                }
                serde_json::to_writer(&mut *out, entry).expect("finite journal entry");
            }
            out.extend_from_slice(b"]}");
        } else if !self.decision_journal.shares_all(&old_journal) {
            if self
                .decision_journal
                .starts_with_by(&old_journal, same_journal_entry)
            {
                let old_len = old_journal.len();
                let new_len = self.decision_journal.len();
                if old_len != new_len {
                    write_member_head(out, &mut first, "decision_journal");
                    out.extend_from_slice(br#"{"items":["#);
                    let mut first_item = true;
                    if old_len > 0 {
                        let _ = write!(out, "[0,{old_len}]");
                        first_item = false;
                    }
                    for entry in self.decision_journal.iter_from(old_len) {
                        if !first_item {
                            out.push(b',');
                        }
                        first_item = false;
                        serde_json::to_writer(&mut *out, entry).expect("finite journal entry");
                    }
                    out.extend_from_slice(b"]}");
                }
            } else {
                let old_entries = old_journal.iter().collect::<Vec<_>>();
                let new_entries = self.decision_journal.iter().collect::<Vec<_>>();
                write_list_changes(
                    out,
                    &mut first,
                    "decision_journal",
                    Some(&old_entries),
                    &new_entries,
                    |old, new| std::ptr::eq(*old, *new) || same_journal_entry(old, new),
                    |out, entry| {
                        serde_json::to_writer(&mut *out, *entry).expect("finite journal entry");
                    },
                );
            }
        }

        // 7. decision_series
        let old_series = std::mem::replace(
            &mut self.decision_series,
            Arc::clone(&session.decision_series),
        );
        if full || !Arc::ptr_eq(&old_series, &self.decision_series) {
            write_map_changes(
                out,
                &mut first,
                "decision_series",
                (!full).then_some(old_series.as_ref()),
                &self.decision_series,
                &same_decision_series,
            );
        }

        // 8. effects
        if full || !same_effects(&self.effects, &session.effects) {
            self.effects.clone_from(&session.effects);
            write_member_head(out, &mut first, "effects");
            out.extend_from_slice(br#"{"value":"#);
            serde_json::to_writer(&mut *out, &session.effects).expect("finite effects");
            out.push(b'}');
        }

        // 9. relations
        if full || graph_changed || symbols_changed {
            if full {
                write_member_head(out, &mut first, "relations");
                out.extend_from_slice(br#"{"items":["#);
                for (idx, edge) in self.graph.edges.iter().enumerate() {
                    if idx > 0 {
                        out.push(b',');
                    }
                    write_relation(out, &self.symbols, edge);
                }
                out.extend_from_slice(b"]}");
            } else if edges_appended && names_kept {
                let old_len = old_graph.edges.len();
                let new_len = self.graph.edges.len();
                if old_len != new_len {
                    write_member_head(out, &mut first, "relations");
                    out.extend_from_slice(br#"{"items":["#);
                    let mut first_item = true;
                    if old_len > 0 {
                        let _ = write!(out, "[0,{old_len}]");
                        first_item = false;
                    }
                    for edge in self.graph.edges.iter_from(old_len) {
                        if !first_item {
                            out.push(b',');
                        }
                        first_item = false;
                        write_relation(out, &self.symbols, edge);
                    }
                    out.extend_from_slice(b"]}");
                }
            } else {
                let edges = |old: &&Edge, new: &&Edge| {
                    old.from == new.from && old.to == new.to && old.relation == new.relation
                };
                let old_edges = old_graph.edges.iter().collect::<Vec<_>>();
                let new_edges = self.graph.edges.iter().collect::<Vec<_>>();
                if !symbols_changed {
                    write_list_changes(
                        out,
                        &mut first,
                        "relations",
                        Some(&old_edges),
                        &new_edges,
                        edges,
                        |out, edge| write_relation(out, &self.symbols, edge),
                    );
                } else {
                    write_list_changes(
                        out,
                        &mut first,
                        "relations",
                        Some(&old_edges),
                        &new_edges,
                        |old, new| {
                            edges(old, new)
                                && old_symbols.name_of(old.from) == self.symbols.name_of(new.from)
                                && old_symbols.name_of(old.to) == self.symbols.name_of(new.to)
                        },
                        |out, edge| write_relation(out, &self.symbols, edge),
                    );
                }
            }
        }

        #[cfg(debug_assertions)]
        if !full {
            debug_assert_eq!(
                text(&self.bindings),
                text(&session.bindings),
                "PatchParts::sync_json bindings drifted from session"
            );
            debug_assert_eq!(
                self.binding_explanations, session.binding_explanations,
                "PatchParts::sync_json binding_explanations drifted from session"
            );
            let live_commitments = session
                .commitment_records(|id| self.symbols.name_of(*id))
                .into_iter()
                .map(|c| (c.action.clone(), c))
                .collect::<BTreeMap<_, _>>();
            debug_assert_eq!(
                self.commitments, live_commitments,
                "PatchParts::sync_json commitments drifted from session"
            );
        }
    }
}

/// A list member a host replaces whole when its text changes, avoiding JSON
/// serialization in the common empty-to-empty case.
fn sync_whole_list<T: Serialize>(
    held: &mut String,
    live: &[T],
    full: bool,
) -> Option<MemberChange> {
    if !full && live.is_empty() && held == "[]" {
        return None;
    }
    let now = if live.is_empty() {
        "[]".to_owned()
    } else {
        text(live)
    };
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

/// Incremental update of a held nested map when only the `(target, property)`
/// entries named by `touched` in `groups` could have changed.
fn sync_nested_touched<V: Clone + Serialize>(
    held: &mut Nested<V>,
    live: &Nested<V>,
    groups: &[BindingGroup],
    touched: &[usize],
    same: &dyn Fn(&V, &V) -> bool,
) -> Option<MemberChange> {
    let mut existed_before: BTreeMap<&str, bool> = BTreeMap::new();
    for &group in touched {
        let target = groups[group].target.as_str();
        existed_before
            .entry(target)
            .or_insert_with(|| held.contains_key(target));
    }
    let mut per_target: BTreeMap<&str, (bool, BTreeMap<String, Entry>)> = BTreeMap::new();
    for &group in touched {
        let target = groups[group].target.as_str();
        let property = &groups[group].property;
        let live_value = live.get(target).and_then(|props| props.get(property));
        let held_value = held.get(target).and_then(|props| props.get(property));
        match (held_value, live_value) {
            (Some(known), Some(value)) if same(known, value) => {}
            (Some(_), Some(value)) => {
                *held
                    .get_mut(target)
                    .expect("target held")
                    .get_mut(property)
                    .expect("property held") = value.clone();
                per_target
                    .entry(target)
                    .or_default()
                    .1
                    .insert(property.clone(), Entry::Value(raw(value)));
            }
            (None, Some(value)) => {
                held.entry(target.to_owned())
                    .or_default()
                    .insert(property.clone(), value.clone());
                let entry = per_target.entry(target).or_default();
                entry.0 = true;
                entry.1.insert(property.clone(), Entry::Value(raw(value)));
            }
            (Some(_), None) => {
                let empty = {
                    let props = held.get_mut(target).expect("target held");
                    props.remove(property);
                    props.is_empty()
                };
                if empty {
                    held.remove(target);
                }
                per_target.entry(target).or_default().0 = true;
            }
            (None, None) => {}
        }
    }
    let mut outer_keys_changed = false;
    for (&target, &had) in &existed_before {
        if live.contains_key(target) != had {
            outer_keys_changed = true;
            break;
        }
    }
    let mut set = BTreeMap::new();
    for (target, (mut inner_keys_changed, prop_set)) in per_target {
        let Some(live_props) = live.get(target) else {
            continue;
        };
        if !existed_before[target] {
            inner_keys_changed = true;
        }
        if inner_keys_changed || !prop_set.is_empty() {
            set.insert(
                target.to_owned(),
                Entry::Map(MapChange {
                    keys: inner_keys_changed.then(|| live_props.keys().cloned().collect()),
                    set: prop_set,
                }),
            );
        }
    }
    (outer_keys_changed || !set.is_empty()).then(|| {
        MemberChange::Map(MapChange {
            keys: outer_keys_changed.then(|| live.keys().cloned().collect()),
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

fn relation(symbols: &NameTable, edge: &Edge) -> MapRelation {
    MapRelation {
        from: symbols.name_of(edge.from).expect("a named node").into(),
        to: symbols.name_of(edge.to).expect("a named node").into(),
        relation: relation_name(edge.relation).into(),
        origin: "live".into(),
    }
}

#[derive(Serialize)]
struct BorrowedRelation<'a> {
    from: &'a str,
    relation: &'static str,
    to: &'a str,
    origin: &'static str,
}

fn write_relation(out: &mut Vec<u8>, symbols: &NameTable, edge: &Edge) {
    let rel = BorrowedRelation {
        from: symbols.name_of(edge.from).expect("a named node"),
        to: symbols.name_of(edge.to).expect("a named node"),
        relation: relation_name(edge.relation),
        origin: "live",
    };
    serde_json::to_writer(&mut *out, &rel).expect("finite relation");
}

fn write_member_head(out: &mut Vec<u8>, first: &mut bool, name: &str) {
    if !*first {
        out.push(b',');
    }
    *first = false;
    out.push(b'"');
    out.extend_from_slice(name.as_bytes());
    out.extend_from_slice(br#"":"#);
}

fn write_keys_field<'a>(out: &mut Vec<u8>, keys: impl Iterator<Item = &'a str>) {
    out.extend_from_slice(br#""keys":["#);
    for (idx, key) in keys.enumerate() {
        if idx > 0 {
            out.push(b',');
        }
        serde_json::to_writer(&mut *out, key).expect("finite key");
    }
    out.push(b']');
}

fn write_nested_full<V: Clone + Serialize>(
    out: &mut Vec<u8>,
    held: &mut Nested<V>,
    live: &Nested<V>,
) {
    held.clone_from(live);
    out.push(b'{');
    write_keys_field(out, live.keys().map(String::as_str));
    out.extend_from_slice(br#","set":{"#);
    for (i, (target, props)) in live.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        serde_json::to_writer(&mut *out, target).expect("finite key");
        out.extend_from_slice(b":{");
        write_keys_field(out, props.keys().map(String::as_str));
        out.extend_from_slice(br#","set":{"#);
        for (j, (prop, val)) in props.iter().enumerate() {
            if j > 0 {
                out.push(b',');
            }
            serde_json::to_writer(&mut *out, prop).expect("finite key");
            out.push(b':');
            serde_json::to_writer(&mut *out, val).expect("finite value");
        }
        out.extend_from_slice(b"}}");
    }
    out.extend_from_slice(b"}}");
}

#[allow(clippy::too_many_arguments)]
fn write_nested_touched<V: Clone + Serialize>(
    out: &mut Vec<u8>,
    first: &mut bool,
    member: &str,
    held: &mut Nested<V>,
    live: &Nested<V>,
    groups: &[BindingGroup],
    touched: &[usize],
    same: &dyn Fn(&V, &V) -> bool,
) {
    // Fast path: all touched binding groups belong to one target (the common
    // case on routed events), avoiding intermediate BTreeMap allocations.
    if let Some((&first_group, rest)) = touched.split_first() {
        let target = groups[first_group].target.as_str();
        if rest.iter().all(|&g| groups[g].target == target) {
            let existed = held.contains_key(target);
            let mut inner_keys_changed = false;
            let mut prop_set: Vec<(&str, &V)> = Vec::with_capacity(touched.len());
            for &group in touched {
                let property = groups[group].property.as_str();
                let live_value = live.get(target).and_then(|props| props.get(property));
                let held_value = held.get(target).and_then(|props| props.get(property));
                match (held_value, live_value) {
                    (Some(known), Some(value)) if same(known, value) => {}
                    (Some(_), Some(value)) => {
                        *held
                            .get_mut(target)
                            .expect("target held")
                            .get_mut(property)
                            .expect("property held") = value.clone();
                        prop_set.push((property, value));
                    }
                    (None, Some(value)) => {
                        held.entry(target.to_owned())
                            .or_default()
                            .insert(property.to_owned(), value.clone());
                        inner_keys_changed = true;
                        prop_set.push((property, value));
                    }
                    (Some(_), None) => {
                        let empty = {
                            let props = held.get_mut(target).expect("target held");
                            props.remove(property);
                            props.is_empty()
                        };
                        if empty {
                            held.remove(target);
                        }
                        inner_keys_changed = true;
                    }
                    (None, None) => {}
                }
            }
            let live_props = live.get(target);
            let outer_keys_changed = live_props.is_some() != existed;
            if let Some(live_props) = live_props {
                if !existed {
                    inner_keys_changed = true;
                }
                if !outer_keys_changed && !inner_keys_changed && prop_set.is_empty() {
                    return;
                }
                if prop_set.len() > 1 {
                    prop_set.sort_unstable_by_key(|(k, _)| *k);
                    prop_set.dedup_by_key(|(k, _)| *k);
                }
                write_member_head(out, first, member);
                out.push(b'{');
                if outer_keys_changed {
                    write_keys_field(out, live.keys().map(String::as_str));
                    out.push(b',');
                }
                out.extend_from_slice(br#""set":{"#);
                if inner_keys_changed || !prop_set.is_empty() {
                    serde_json::to_writer(&mut *out, target).expect("finite key");
                    out.extend_from_slice(b":{");
                    if inner_keys_changed {
                        write_keys_field(out, live_props.keys().map(String::as_str));
                        out.push(b',');
                    }
                    out.extend_from_slice(br#""set":{"#);
                    for (j, (prop, val)) in prop_set.into_iter().enumerate() {
                        if j > 0 {
                            out.push(b',');
                        }
                        serde_json::to_writer(&mut *out, prop).expect("finite key");
                        out.push(b':');
                        serde_json::to_writer(&mut *out, val).expect("finite value");
                    }
                    out.extend_from_slice(b"}}");
                }
                out.extend_from_slice(b"}}");
            } else if outer_keys_changed {
                write_member_head(out, first, member);
                out.push(b'{');
                write_keys_field(out, live.keys().map(String::as_str));
                out.extend_from_slice(br#","set":{}}"#);
            }
            return;
        }
    }
    let mut existed_before: BTreeMap<&str, bool> = BTreeMap::new();
    for &group in touched {
        let target = groups[group].target.as_str();
        existed_before
            .entry(target)
            .or_insert_with(|| held.contains_key(target));
    }
    let mut per_target: BTreeMap<&str, (bool, BTreeMap<&str, &V>)> = BTreeMap::new();
    for &group in touched {
        let target = groups[group].target.as_str();
        let property = groups[group].property.as_str();
        let live_value = live.get(target).and_then(|props| props.get(property));
        let held_value = held.get(target).and_then(|props| props.get(property));
        match (held_value, live_value) {
            (Some(known), Some(value)) if same(known, value) => {}
            (Some(_), Some(value)) => {
                *held
                    .get_mut(target)
                    .expect("target held")
                    .get_mut(property)
                    .expect("property held") = value.clone();
                per_target
                    .entry(target)
                    .or_default()
                    .1
                    .insert(property, value);
            }
            (None, Some(value)) => {
                held.entry(target.to_owned())
                    .or_default()
                    .insert(property.to_owned(), value.clone());
                let entry = per_target.entry(target).or_default();
                entry.0 = true;
                entry.1.insert(property, value);
            }
            (Some(_), None) => {
                let empty = {
                    let props = held.get_mut(target).expect("target held");
                    props.remove(property);
                    props.is_empty()
                };
                if empty {
                    held.remove(target);
                }
                per_target.entry(target).or_default().0 = true;
            }
            (None, None) => {}
        }
    }
    let mut outer_keys_changed = false;
    for (&target, &had) in &existed_before {
        if live.contains_key(target) != had {
            outer_keys_changed = true;
            break;
        }
    }
    let mut active = Vec::new();
    for (target, (mut inner_keys_changed, prop_set)) in per_target {
        let Some(live_props) = live.get(target) else {
            continue;
        };
        if !existed_before[target] {
            inner_keys_changed = true;
        }
        if inner_keys_changed || !prop_set.is_empty() {
            active.push((target, inner_keys_changed, live_props, prop_set));
        }
    }
    if !outer_keys_changed && active.is_empty() {
        return;
    }
    write_member_head(out, first, member);
    out.push(b'{');
    if outer_keys_changed {
        write_keys_field(out, live.keys().map(String::as_str));
        out.push(b',');
    }
    out.extend_from_slice(br#""set":{"#);
    for (i, (target, inner_keys_changed, live_props, prop_set)) in active.into_iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        serde_json::to_writer(&mut *out, target).expect("finite key");
        out.extend_from_slice(b":{");
        if inner_keys_changed {
            write_keys_field(out, live_props.keys().map(String::as_str));
            out.push(b',');
        }
        out.extend_from_slice(br#""set":{"#);
        for (j, (prop, val)) in prop_set.into_iter().enumerate() {
            if j > 0 {
                out.push(b',');
            }
            serde_json::to_writer(&mut *out, prop).expect("finite key");
            out.push(b':');
            serde_json::to_writer(&mut *out, val).expect("finite value");
        }
        out.extend_from_slice(b"}}");
    }
    out.extend_from_slice(b"}}");
}

fn write_sync_map<V: Clone + Serialize>(
    out: &mut Vec<u8>,
    first: &mut bool,
    member: &str,
    held: &mut BTreeMap<String, V>,
    live: &BTreeMap<String, V>,
    full: bool,
    same: &dyn Fn(&V, &V) -> bool,
) {
    let keys_changed = full || !held.keys().eq(live.keys());
    if keys_changed {
        held.retain(|key, _| live.contains_key(key));
    }
    let mut changed: Vec<(&str, &V)> = Vec::new();
    for (key, value) in live {
        match held.get_mut(key) {
            Some(known) if same(known, value) => {}
            Some(known) => {
                *known = value.clone();
                changed.push((key.as_str(), value));
            }
            None => {
                held.insert(key.clone(), value.clone());
                changed.push((key.as_str(), value));
            }
        }
    }
    if !keys_changed && changed.is_empty() {
        return;
    }
    write_member_head(out, first, member);
    out.push(b'{');
    if keys_changed {
        write_keys_field(out, live.keys().map(String::as_str));
        out.push(b',');
    }
    out.extend_from_slice(br#""set":{"#);
    for (idx, (key, value)) in changed.into_iter().enumerate() {
        if idx > 0 {
            out.push(b',');
        }
        serde_json::to_writer(&mut *out, key).expect("finite key");
        out.push(b':');
        serde_json::to_writer(&mut *out, value).expect("finite value");
    }
    out.extend_from_slice(b"}}");
}

fn write_map_changes<V: Serialize>(
    out: &mut Vec<u8>,
    first: &mut bool,
    member: &str,
    before: Option<&BTreeMap<String, V>>,
    after: &BTreeMap<String, V>,
    same: &dyn Fn(&V, &V) -> bool,
) {
    let keys_changed = before.is_none_or(|old| !old.keys().eq(after.keys()));
    let mut changed: Vec<(&str, &V)> = Vec::new();
    for (key, new) in after {
        if !before
            .and_then(|old| old.get(key))
            .is_some_and(|known| same(known, new))
        {
            changed.push((key.as_str(), new));
        }
    }
    if !keys_changed && changed.is_empty() {
        return;
    }
    write_member_head(out, first, member);
    out.push(b'{');
    if keys_changed {
        write_keys_field(out, after.keys().map(String::as_str));
        out.push(b',');
    }
    out.extend_from_slice(br#""set":{"#);
    for (idx, (key, value)) in changed.into_iter().enumerate() {
        if idx > 0 {
            out.push(b',');
        }
        serde_json::to_writer(&mut *out, key).expect("finite key");
        out.push(b':');
        serde_json::to_writer(&mut *out, value).expect("finite value");
    }
    out.extend_from_slice(b"}}");
}

fn write_list_changes<T>(
    out: &mut Vec<u8>,
    first: &mut bool,
    member: &str,
    before: Option<&[T]>,
    after: &[T],
    same: impl Fn(&T, &T) -> bool,
    render: impl Fn(&mut Vec<u8>, &T),
) {
    let old = before.unwrap_or_default();
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut run: Option<(usize, usize)> = None;
    let mut kept = 0;
    for (position, entry) in old.iter().enumerate() {
        if kept < after.len() && same(entry, &after[kept]) {
            kept += 1;
            run = match run {
                Some((start, length)) if start + length == position => Some((start, length + 1)),
                Some(done) => {
                    runs.push(done);
                    Some((position, 1))
                }
                None => Some((position, 1)),
            };
        }
    }
    if before.is_some() && kept == old.len() && kept == after.len() {
        return;
    }
    if let Some(done) = run {
        runs.push(done);
    }
    write_member_head(out, first, member);
    out.extend_from_slice(br#"{"items":["#);
    let mut first_item = true;
    for (start, length) in runs {
        if !first_item {
            out.push(b',');
        }
        first_item = false;
        let _ = write!(out, "[{start},{length}]");
    }
    for entry in &after[kept..] {
        if !first_item {
            out.push(b',');
        }
        first_item = false;
        render(out, entry);
    }
    out.extend_from_slice(b"]}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn apply_map(held: Option<&Value>, change: &Value, nested: bool) -> Value {
        let set = change["set"].as_object().expect("map set");
        let keys: Vec<String> = match change.get("keys") {
            Some(keys) => keys
                .as_array()
                .expect("keys array")
                .iter()
                .map(|k| k.as_str().expect("key str").to_owned())
                .collect(),
            None => held
                .and_then(Value::as_object)
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default(),
        };
        let mut out = serde_json::Map::new();
        for key in keys {
            let value = if let Some(entry) = set.get(&key) {
                if nested {
                    apply_map(held.and_then(|h| h.get(&key)), entry, false)
                } else {
                    entry.clone()
                }
            } else {
                held.and_then(|h| h.get(&key))
                    .expect("held key present")
                    .clone()
            };
            out.insert(key, value);
        }
        Value::Object(out)
    }

    fn apply_list(held: Option<&Value>, change: &Value) -> Value {
        let held = held.and_then(Value::as_array);
        let mut out = Vec::new();
        for item in change["items"].as_array().expect("list items") {
            if let Some(run) = item.as_array() {
                let start = run[0].as_u64().unwrap() as usize;
                let len = run[1].as_u64().unwrap() as usize;
                let src = held.expect("held list");
                out.extend_from_slice(&src[start..start + len]);
            } else {
                out.push(item.clone());
            }
        }
        Value::Array(out)
    }

    fn apply_patch(held: Option<&Value>, patch: &Value) -> Value {
        let base = if patch["since"].is_null() { None } else { held };
        let members = patch["members"].as_object().expect("members");
        let mut view = serde_json::Map::new();
        view.insert("schema".into(), patch["view_schema"].clone());
        view.insert("sequence".into(), patch["sequence"].clone());
        view.insert("last_event".into(), patch["last_event"].clone());
        for name in [
            "bindings",
            "binding_explanations",
            "cues",
            "effects",
            "commitments",
            "commitment_grounds",
            "decision_series",
            "decision_journal",
            "relations",
        ] {
            let value = match members.get(name) {
                None => base.expect("base view")[name].clone(),
                Some(change) if matches!(name, "cues" | "effects") => change["value"].clone(),
                Some(change) if matches!(name, "decision_journal" | "relations") => {
                    apply_list(base.map(|b| &b[name]), change)
                }
                Some(change) if name == "commitments" => {
                    let held_map = base.map(|b| {
                        let mut m = serde_json::Map::new();
                        for c in b["commitments"].as_array().unwrap() {
                            m.insert(c["action"].as_str().unwrap().to_owned(), c.clone());
                        }
                        Value::Object(m)
                    });
                    let map = apply_map(held_map.as_ref(), change, false);
                    let obj = map.as_object().unwrap();
                    let order: Vec<String> = match change.get("keys") {
                        Some(k) => k
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|s| s.as_str().unwrap().to_owned())
                            .collect(),
                        None => base
                            .unwrap()
                            .get("commitments")
                            .unwrap()
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|c| c["action"].as_str().unwrap().to_owned())
                            .collect(),
                    };
                    Value::Array(order.into_iter().map(|k| obj[&k].clone()).collect())
                }
                Some(change) if matches!(name, "bindings" | "binding_explanations") => {
                    apply_map(base.map(|b| &b[name]), change, true)
                }
                Some(change) => apply_map(base.map(|b| &b[name]), change, false),
            };
            view.insert(name.into(), value);
        }
        Value::Object(view)
    }

    fn verify_sequence(source: &str, events: &[(&str, &str)]) {
        let mut plain = ReactiveSession::from_source(source).unwrap();
        let mut patched = ReactiveSession::from_source(source).unwrap();
        let mut streamed = ReactiveSession::from_source(source).unwrap();
        let mut base = ViewPatchBase::default();
        let mut base_text = ViewPatchBase::default();
        let mut held: Option<Value> = None;
        for &(event, payload) in events {
            let expected_raw =
                serde_json::to_string(&plain.dispatch_view_outcome_json(event, payload).unwrap())
                    .unwrap();
            let expected_val: Value = serde_json::from_str(&expected_raw).unwrap();
            let expected = serde_json::to_string(&expected_val).unwrap();
            let seq = held.as_ref().and_then(|v| v["sequence"].as_u64());
            let patch_raw = serde_json::to_string(
                &patched
                    .dispatch_view_patch_outcome_json_from(&mut base, seq, event, payload, None)
                    .unwrap(),
            )
            .unwrap();
            let streamed_raw = streamed
                .dispatch_view_patch_outcome_text_from(&mut base_text, seq, event, payload, None)
                .unwrap();
            assert_eq!(streamed_raw, patch_raw, "streamed != structured on {event} {payload}");
            let patch_outcome: Value = serde_json::from_str(&patch_raw).unwrap();
            if patch_outcome["outcome"] == "accepted" {
                let view = apply_patch(held.as_ref(), &patch_outcome["patch"]);
                let reconstructed = serde_json::to_string(&json!({
                    "schema": DISPATCH_SCHEMA,
                    "outcome": "accepted",
                    "view": view,
                }))
                .unwrap();
                assert_eq!(reconstructed, expected, "{event} {payload}");
                held = Some(view);
            } else {
                assert_eq!(
                    serde_json::to_string(&patch_outcome).unwrap(),
                    expected,
                    "{event} {payload}"
                );
            }
        }
    }

    #[test]
    fn incremental_patch_handles_binding_targets_zero_signs_commitments_and_departures() {
        let source = r#"
claim ok;
caveat k consequence material;
evidence tool from "tool";
evidence note from "note";
readings r from tool window 2;
decisions d limit 8 reopened by r;
journal window 2;
state x = 0 min -10 max 10;
state show_extra = 0 min 0 max 1;
event noop;
event set_x v min -10 max 10;
event toggle v min 0 max 1;
event sample_r v min -10 max 10;
event decide;
event flag;
event withdraw_latest;
on set_x set x = v;
on toggle set show_extra = v;
on sample_r sample r = v supports ok;
on decide commit d because enough using latest(r);
on flag qualify latest(r) with k;
on withdraw_latest reveal note;
on withdraw_latest withdraw latest(r) because note;
bind panel.x = x;
bind panel.extra = 1 when show_extra == 1;
bind ephemeral.only = x when show_extra == 1;
bind hist.count = history_count(r);
"#;
        verify_sequence(
            source,
            &[
                ("noop", "{}"),
                // 0.0 -> -0.0 and back, plus untouched noop
                ("set_x", r#"{"v":-0.0}"#),
                ("noop", "{}"),
                ("set_x", r#"{"v":0.0}"#),
                // Add inner property & brand-new outer target
                ("toggle", r#"{"v":1}"#),
                // Update property while outer target exists
                ("set_x", r#"{"v":2}"#),
                // Remove inner property & remove last property of outer target
                ("toggle", r#"{"v":0}"#),
                // Sample, commit, qualify, reopen, commit past window=2 so
                // commitments, relations, and journal entries depart
                ("sample_r", r#"{"v":1}"#),
                ("decide", "{}"),
                ("flag", "{}"),
                ("sample_r", r#"{"v":2}"#),
                ("decide", "{}"),
                ("sample_r", r#"{"v":3}"#),
                ("decide", "{}"),
                ("withdraw_latest", "{}"),
                ("sample_r", r#"{"v":4}"#),
                ("decide", "{}"),
                // Refused event leaves base valid for next accepted event
                ("set_x", r#"{"v":999}"#),
                ("set_x", r#"{"v":5}"#),
            ],
        );
    }

    #[test]
    fn streaming_view_patch_matches_structured_patch_byte_for_byte() {
        let source = r#"
claim ok;
place p kind site;
entity a kind item at p;
caveat c consequence material;
evidence e from "sensor";
readings r from e limit 4;
decisions d limit 4 reopened by r;
event sample v min -1 max 1;
event flag;
event unflag;
event decide;
on sample sample r = v supports ok;
on flag qualify latest(r) with c;
on unflag unqualify latest(r) with c;
on decide commit d because enough using latest(r);
bind a.count = history_count(r);
bind a.val = latest(r) when has_sample(r);
"#;
        let mut s_struct = ReactiveSession::from_source(source).unwrap();
        let mut s_text = ReactiveSession::from_source(source).unwrap();
        let mut base_struct = ViewPatchBase::default();
        let mut base_text = ViewPatchBase::default();
        let mut held = None;
        let events = [
            ("sample", r#"{"v":0}"#),
            ("sample", r#"{"v":-0.0}"#),
            ("decide", r#"{}"#),
            ("flag", r#"{}"#),
            ("unflag", r#"{}"#),
            ("decide", r#"{}"#), // rejected: not reopened
            ("sample", r#"{"v":1}"#),
            ("decide", r#"{}"#),
        ];
        for (event, payload) in events {
            let structured = serde_json::to_string(
                &s_struct
                    .dispatch_view_patch_outcome_json_from(
                        &mut base_struct,
                        held,
                        event,
                        payload,
                        None,
                    )
                    .unwrap(),
            )
            .unwrap();
            let streamed = s_text
                .dispatch_view_patch_outcome_text_from(&mut base_text, held, event, payload, None)
                .unwrap();
            assert_eq!(streamed, structured, "mismatch on event {event} {payload}");
            held = Some(s_text.sequence);
        }
    }
}
