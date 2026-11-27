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

    /// `dispatch_view_patch_outcome_text_from` encoded into a compact binary
    /// wire buffer so the WebAssembly host can construct frozen View 0.1
    /// objects directly without JSON formatting or `JSON.parse`.
    pub fn dispatch_view_patch_outcome_binary_from(
        &mut self,
        base: &mut ViewPatchBase,
        held: Option<u64>,
        event: &str,
        payload_json: &str,
        agent: Option<&str>,
    ) -> Result<Vec<u8>, DispatchFatal> {
        let since = self.sequence;
        let before = match (base.0.take(), held) {
            (Some((sequence, parts)), Some(held)) if sequence == since && held == since => {
                Some(parts)
            }
            _ => None,
        };
        match self.dispatch_classified(event, payload_json, agent) {
            Ok(()) => {
                let mut out = Vec::with_capacity(512);
                out.push(1); // accepted
                match before.is_some().then_some(since) {
                    Some(seq) => {
                        out.push(1);
                        bin_varint(&mut out, seq);
                    }
                    None => out.push(0),
                }
                bin_varint(&mut out, self.sequence);
                bin_opt_str(&mut out, self.last_event.as_deref());
                let mask_pos = out.len();
                out.extend_from_slice(&[0, 0]);
                let (mask, parts) = match before {
                    Some(mut parts) => {
                        let mask = parts.sync_binary(self, false, &mut out);
                        (mask, parts)
                    }
                    None => {
                        let mut parts = PatchParts::new(self);
                        let mask = parts.sync_binary(self, true, &mut out);
                        (mask, parts)
                    }
                };
                out[mask_pos..mask_pos + 2].copy_from_slice(&mask.to_le_bytes());
                base.0 = Some((self.sequence, parts));
                Ok(out)
            }
            Err(failure) => {
                let outcome = failure.classify();
                if outcome.is_ok() {
                    if let Some(parts) = before {
                        base.0 = Some((since, parts));
                    }
                }
                let (origin, code, message) = outcome?;
                let mut out = Vec::with_capacity(64 + message.len());
                out.push(0); // rejected
                out.push(origin.tag());
                bin_str(&mut out, code.as_str());
                bin_str(&mut out, &message);
                Ok(out)
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
            if let Some(key) = (!full)
                .then_some(session.last_grounds_written.as_deref())
                .flatten()
            {
                write_map_change_one(
                    out,
                    &mut first,
                    "commitment_grounds",
                    &old_grounds,
                    &self.commitment_grounds,
                    key,
                    &|old, new| old == new,
                );
            } else {
                write_map_changes(
                    out,
                    &mut first,
                    "commitment_grounds",
                    (!full).then_some(old_grounds.as_ref()),
                    &self.commitment_grounds,
                    &|old, new| old == new,
                );
            }
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
            if let Some(key) = (!full)
                .then_some(session.last_series_written.as_deref())
                .flatten()
            {
                write_map_change_one(
                    out,
                    &mut first,
                    "decision_series",
                    &old_series,
                    &self.decision_series,
                    key,
                    &same_decision_series,
                );
            } else {
                write_map_changes(
                    out,
                    &mut first,
                    "decision_series",
                    (!full).then_some(old_series.as_ref()),
                    &self.decision_series,
                    &same_decision_series,
                );
            }
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

    /// Bring the parts up to date with `session` and encode changed members
    /// into the compact binary patch buffer in VIEW_FIELDS bit order (0..8):
    /// 0:bindings, 1:binding_explanations, 2:cues, 3:effects, 4:commitments,
    /// 5:commitment_grounds, 6:decision_series, 7:decision_journal, 8:relations.
    fn sync_binary(&mut self, session: &ReactiveSession, full: bool, out: &mut Vec<u8>) -> u16 {
        let mut mask: u16 = 0;

        // Bit 0: bindings
        if full {
            mask |= 1 << 0;
            bin_nested_full(
                out,
                &mut self.bindings,
                &session.bindings,
                bin_binding_value,
            );
        } else if !session.last_binding_groups.is_empty()
            && bin_nested_touched(
                out,
                &mut self.bindings,
                &session.bindings,
                &session.binding_groups,
                &session.last_binding_groups,
                &same_binding,
                bin_binding_value,
            )
        {
            mask |= 1 << 0;
        }

        // Bit 1: binding_explanations
        if full {
            mask |= 1 << 1;
            bin_nested_full(
                out,
                &mut self.binding_explanations,
                &session.binding_explanations,
                bin_provenance,
            );
        } else if !session.last_binding_groups.is_empty()
            && bin_nested_touched(
                out,
                &mut self.binding_explanations,
                &session.binding_explanations,
                &session.binding_groups,
                &session.last_binding_groups,
                &|old, new| old == new,
                bin_provenance,
            )
        {
            mask |= 1 << 1;
        }

        // Bit 2: cues
        if full || !(session.cues.is_empty() && self.cues == "[]") {
            let now = if session.cues.is_empty() {
                "[]".to_owned()
            } else {
                text(&session.cues)
            };
            if full || self.cues != now {
                mask |= 1 << 2;
                if session.cues.is_empty() {
                    out.push(0);
                } else {
                    out.push(1);
                    bin_str(out, &now);
                }
                self.cues = now;
            }
        }

        // Bit 3: effects
        if full || !same_effects(&self.effects, &session.effects) {
            mask |= 1 << 3;
            self.effects.clone_from(&session.effects);
            bin_varint(out, session.effects.len() as u64);
            for effect in &session.effects {
                bin_effect(out, effect);
            }
        }

        // Prepare graph / symbol deltas once for commitments (4) and relations (8).
        let old_symbols = std::mem::replace(&mut self.symbols, session.symbols.clone());
        let symbols_changed = !old_symbols.shares_all(&self.symbols);
        let old_graph = std::mem::replace(&mut self.graph, Arc::clone(&session.graph));
        let graph_changed = !Arc::ptr_eq(&old_graph, &self.graph);
        let names_kept = !full && (!symbols_changed || old_symbols.kept_in(&self.symbols));
        let edges_appended = !full && self.graph.edges.starts_with(&old_graph.edges);

        // Bit 4: commitments
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
                    if bin_sync_map(
                        out,
                        &mut self.commitments,
                        &live,
                        false,
                        &|old, new| old == new,
                        bin_commitment,
                    ) {
                        mask |= 1 << 4;
                    }
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
                            mask |= 1 << 4;
                            if keys_changed {
                                out.push(1);
                                bin_varint(out, self.commitments.len() as u64);
                                for key in self.commitments.keys() {
                                    bin_str(out, key);
                                }
                            } else {
                                out.push(0);
                            }
                            bin_varint(out, set_actions.len() as u64);
                            for action in &set_actions {
                                bin_str(out, action);
                                bin_commitment(out, &self.commitments[action]);
                            }
                        }
                    }
                }
            } else {
                let live = session
                    .commitment_records(|id| self.symbols.name_of(*id))
                    .into_iter()
                    .map(|commitment| (commitment.action.clone(), commitment))
                    .collect::<BTreeMap<_, _>>();
                if bin_sync_map(
                    out,
                    &mut self.commitments,
                    &live,
                    full,
                    &|old, new| old == new,
                    bin_commitment,
                ) {
                    mask |= 1 << 4;
                }
            }
        }

        // Bit 5: commitment_grounds
        let old_grounds = std::mem::replace(
            &mut self.commitment_grounds,
            Arc::clone(&session.commitment_grounds),
        );
        if full || !Arc::ptr_eq(&old_grounds, &self.commitment_grounds) {
            let wrote = if let Some(key) = (!full)
                .then_some(session.last_grounds_written.as_deref())
                .flatten()
            {
                bin_map_change_one(
                    out,
                    &old_grounds,
                    &self.commitment_grounds,
                    key,
                    &|old, new| old == new,
                    bin_provenance,
                )
            } else {
                bin_map_changes(
                    out,
                    (!full).then_some(old_grounds.as_ref()),
                    &self.commitment_grounds,
                    &|old, new| old == new,
                    bin_provenance,
                )
            };
            if wrote {
                mask |= 1 << 5;
            }
        }

        // Bit 6: decision_series
        let old_series = std::mem::replace(
            &mut self.decision_series,
            Arc::clone(&session.decision_series),
        );
        if full || !Arc::ptr_eq(&old_series, &self.decision_series) {
            let wrote = if let Some(key) = (!full)
                .then_some(session.last_series_written.as_deref())
                .flatten()
            {
                bin_map_change_one(
                    out,
                    &old_series,
                    &self.decision_series,
                    key,
                    &same_decision_series,
                    bin_decision_series,
                )
            } else {
                bin_map_changes(
                    out,
                    (!full).then_some(old_series.as_ref()),
                    &self.decision_series,
                    &same_decision_series,
                    bin_decision_series,
                )
            };
            if wrote {
                mask |= 1 << 6;
            }
        }

        // Bit 7: decision_journal
        let old_journal = std::mem::replace(&mut self.decision_journal, session.journal.clone());
        if full {
            mask |= 1 << 7;
            bin_varint(out, self.decision_journal.len() as u64);
            for entry in &self.decision_journal {
                out.push(1);
                bin_journal_entry(out, entry);
            }
        } else if !self.decision_journal.shares_all(&old_journal) {
            if self
                .decision_journal
                .starts_with_by(&old_journal, same_journal_entry)
            {
                let old_len = old_journal.len();
                let new_len = self.decision_journal.len();
                if old_len != new_len {
                    mask |= 1 << 7;
                    let item_count = usize::from(old_len > 0) + (new_len - old_len);
                    bin_varint(out, item_count as u64);
                    if old_len > 0 {
                        out.push(0);
                        bin_varint(out, 0);
                        bin_varint(out, old_len as u64);
                    }
                    for entry in self.decision_journal.iter_from(old_len) {
                        out.push(1);
                        bin_journal_entry(out, entry);
                    }
                }
            } else {
                let old_entries = old_journal.iter().collect::<Vec<_>>();
                let new_entries = self.decision_journal.iter().collect::<Vec<_>>();
                if bin_list_changes(
                    out,
                    Some(&old_entries),
                    &new_entries,
                    |old, new| std::ptr::eq(*old, *new) || same_journal_entry(old, new),
                    |out, entry| bin_journal_entry(out, entry),
                ) {
                    mask |= 1 << 7;
                }
            }
        }

        // Bit 8: relations
        if full || graph_changed || symbols_changed {
            if full {
                mask |= 1 << 8;
                bin_varint(out, self.graph.edges.len() as u64);
                for edge in &self.graph.edges {
                    out.push(1);
                    bin_relation(out, &self.symbols, edge);
                }
            } else if edges_appended && names_kept {
                let old_len = old_graph.edges.len();
                let new_len = self.graph.edges.len();
                if old_len != new_len {
                    mask |= 1 << 8;
                    let item_count = usize::from(old_len > 0) + (new_len - old_len);
                    bin_varint(out, item_count as u64);
                    if old_len > 0 {
                        out.push(0);
                        bin_varint(out, 0);
                        bin_varint(out, old_len as u64);
                    }
                    for edge in self.graph.edges.iter_from(old_len) {
                        out.push(1);
                        bin_relation(out, &self.symbols, edge);
                    }
                }
            } else {
                let edges = |old: &&Edge, new: &&Edge| {
                    old.from == new.from && old.to == new.to && old.relation == new.relation
                };
                let old_edges = old_graph.edges.iter().collect::<Vec<_>>();
                let new_edges = self.graph.edges.iter().collect::<Vec<_>>();
                let wrote = if !symbols_changed {
                    bin_list_changes(out, Some(&old_edges), &new_edges, edges, |out, edge| {
                        bin_relation(out, &self.symbols, edge)
                    })
                } else {
                    bin_list_changes(
                        out,
                        Some(&old_edges),
                        &new_edges,
                        |old, new| {
                            edges(old, new)
                                && old_symbols.name_of(old.from) == self.symbols.name_of(new.from)
                                && old_symbols.name_of(old.to) == self.symbols.name_of(new.to)
                        },
                        |out, edge| bin_relation(out, &self.symbols, edge),
                    )
                };
                if wrote {
                    mask |= 1 << 8;
                }
            }
        }

        #[cfg(debug_assertions)]
        if !full {
            debug_assert_eq!(
                text(&self.bindings),
                text(&session.bindings),
                "PatchParts::sync_binary bindings drifted from session"
            );
            debug_assert_eq!(
                self.binding_explanations, session.binding_explanations,
                "PatchParts::sync_binary binding_explanations drifted from session"
            );
            let live_commitments = session
                .commitment_records(|id| self.symbols.name_of(*id))
                .into_iter()
                .map(|c| (c.action.clone(), c))
                .collect::<BTreeMap<_, _>>();
            debug_assert_eq!(
                self.commitments, live_commitments,
                "PatchParts::sync_binary commitments drifted from session"
            );
        }

        mask
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

fn write_map_change_one<V: Serialize>(
    out: &mut Vec<u8>,
    first: &mut bool,
    member: &str,
    before: &BTreeMap<String, V>,
    after: &BTreeMap<String, V>,
    key: &str,
    same: &dyn Fn(&V, &V) -> bool,
) {
    #[cfg(debug_assertions)]
    let start_len = out.len();
    #[cfg(debug_assertions)]
    let start_first = *first;
    let old_val = before.get(key);
    let new_val = after.get(key);
    let keys_changed = old_val.is_some() != new_val.is_some();
    let changed_val = match (old_val, new_val) {
        (Some(old), Some(new)) if same(old, new) => None,
        (_, Some(new)) => Some(new),
        (_, None) => None,
    };
    if !keys_changed && changed_val.is_none() {
        #[cfg(debug_assertions)]
        {
            let mut expected = Vec::new();
            let mut expected_first = start_first;
            write_map_changes(
                &mut expected,
                &mut expected_first,
                member,
                Some(before),
                after,
                same,
            );
            assert!(
                expected.is_empty(),
                "single-key patch diff missed change for {member} (key {key:?})"
            );
        }
        return;
    }
    write_member_head(out, first, member);
    out.push(b'{');
    if keys_changed {
        write_keys_field(out, after.keys().map(String::as_str));
        out.push(b',');
    }
    out.extend_from_slice(br#""set":{"#);
    if let Some(value) = changed_val {
        serde_json::to_writer(&mut *out, key).expect("finite key");
        out.push(b':');
        serde_json::to_writer(&mut *out, value).expect("finite value");
    }
    out.extend_from_slice(b"}}");
    #[cfg(debug_assertions)]
    {
        let mut expected = Vec::new();
        let mut expected_first = start_first;
        write_map_changes(
            &mut expected,
            &mut expected_first,
            member,
            Some(before),
            after,
            same,
        );
        assert_eq!(
            &out[start_len..],
            expected.as_slice(),
            "single-key patch diff drifted for {member} (key {key:?})"
        );
        assert_eq!(*first, expected_first);
    }
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

// ------------------------------------------------ binary wire helpers

#[inline]
fn bin_varint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

#[inline]
fn bin_f64(out: &mut Vec<u8>, value: f64) {
    out.extend_from_slice(&value.to_le_bytes());
}

#[inline]
fn bin_str(out: &mut Vec<u8>, s: &str) {
    let non_ascii = u64::from(!s.is_ascii());
    bin_varint(out, ((s.len() as u64) << 1) | non_ascii);
    out.extend_from_slice(s.as_bytes());
}

#[inline]
fn bin_opt_str(out: &mut Vec<u8>, s: Option<&str>) {
    match s {
        Some(text) => {
            out.push(1);
            bin_str(out, text);
        }
        None => out.push(0),
    }
}

fn bin_str_slice(out: &mut Vec<u8>, items: &[String]) {
    bin_varint(out, items.len() as u64);
    for item in items {
        bin_str(out, item);
    }
}

fn bin_nameset(out: &mut Vec<u8>, set: &crate::reactive_expr::NameSet) {
    bin_varint(out, set.len() as u64);
    for name in set {
        bin_str(out, name.as_str());
    }
}

fn bin_binding_value(out: &mut Vec<u8>, value: &BindingValue) {
    match value {
        BindingValue::Number(n) => {
            out.push(0);
            bin_f64(out, *n);
        }
        BindingValue::Bool(false) => out.push(1),
        BindingValue::Bool(true) => out.push(2),
        BindingValue::Text(s) => {
            out.push(3);
            bin_str(out, s);
        }
    }
}

fn bin_provenance(out: &mut Vec<u8>, prov: &Provenance) {
    let mut flags = 0u8;
    if !prov.departed.is_empty() {
        flags |= 1;
    }
    if !prov.inherited.is_empty() {
        flags |= 2;
    }
    if !prov.kept.is_empty() {
        flags |= 4;
    }
    out.push(flags);
    bin_nameset(out, &prov.evidence);
    bin_nameset(out, &prov.caveats);
    if flags & 1 != 0 {
        bin_varint(out, prov.departed.len() as u64);
        for (history, marker) in &prov.departed {
            bin_str(out, history);
            bin_varint(out, marker.read);
            bin_varint(out, marker.from);
            bin_varint(out, marker.through);
            bin_varint(out, marker.departed_at);
            bin_opt_str(out, marker.archive_ref.as_ref().map(|t| t.id()));
        }
    }
    if flags & 2 != 0 {
        bin_nameset(out, &prov.inherited);
    }
    if flags & 4 != 0 {
        bin_nameset(out, &prov.kept);
    }
}

fn bin_effect(out: &mut Vec<u8>, effect: &EffectReport) {
    match effect {
        EffectReport::Sample {
            stream,
            id,
            value,
            relation,
            target,
            note,
        } => {
            out.push(0);
            bin_str(out, stream);
            bin_str(out, id);
            bin_f64(out, *value);
            let flags = u8::from(relation.is_some())
                | (u8::from(target.is_some()) << 1)
                | (u8::from(note.is_some()) << 2);
            out.push(flags);
            if let Some(rel) = relation {
                bin_str(out, rel);
            }
            if let Some(t) = target {
                bin_str(out, t);
            }
            if let Some(n) = note {
                bin_str(out, n);
            }
        }
        EffectReport::Reveal {
            evidence,
            relation,
            target,
        } => {
            out.push(1);
            bin_str(out, evidence);
            let flags = u8::from(relation.is_some()) | (u8::from(target.is_some()) << 1);
            out.push(flags);
            if let Some(rel) = relation {
                bin_str(out, rel);
            }
            if let Some(t) = target {
                bin_str(out, t);
            }
        }
        EffectReport::Examine { caveat, cost } => {
            out.push(2);
            bin_str(out, caveat);
            bin_varint(out, *cost);
        }
        EffectReport::Commit { action, retained } => {
            out.push(3);
            bin_str(out, action);
            bin_str_slice(out, retained);
        }
        EffectReport::Reopen { action, because } => {
            out.push(4);
            bin_str(out, action);
            bin_str(out, because);
        }
        EffectReport::Qualify { evidence, caveat } => {
            out.push(5);
            bin_str(out, evidence);
            bin_str(out, caveat);
        }
        EffectReport::Unqualify {
            evidence,
            caveat,
            occurrences,
        } => {
            out.push(6);
            bin_str(out, evidence);
            bin_str(out, caveat);
            bin_str_slice(out, occurrences);
        }
        EffectReport::Renew {
            evidence,
            occurrence,
        } => {
            out.push(7);
            bin_str(out, evidence);
            bin_str(out, occurrence);
        }
        EffectReport::Withdraw { evidence, because } => {
            out.push(8);
            bin_str(out, evidence);
            bin_str(out, because);
        }
        EffectReport::Retire { history, record } => {
            out.push(9);
            bin_str(out, history);
            bin_str(out, record);
        }
        EffectReport::Depart { history, record } => {
            out.push(10);
            bin_str(out, history);
            bin_str(out, record);
        }
    }
}

fn bin_commitment(out: &mut Vec<u8>, c: &MapCommitment) {
    bin_str(out, &c.action);
    out.push(u8::from(c.open));
    bin_str_slice(out, &c.retained);
    bin_varint(out, c.retained_authorship.len() as u64);
    for a in &c.retained_authorship {
        bin_str(out, &a.caveat);
        bin_opt_str(out, a.written_by.as_deref());
        out.push(match a.written_elsewhere {
            None => 0,
            Some(false) => 1,
            Some(true) => 2,
        });
    }
    bin_str_slice(out, &c.reopened_by);
}

fn bin_decision_series(out: &mut Vec<u8>, s: &DecisionSeries) {
    bin_varint(out, s.limit as u64);
    bin_opt_str(out, s.current.as_deref());
    bin_varint(out, s.revisions.len() as u64);
    for rev in &s.revisions {
        bin_str(out, &rev.id);
        bin_opt_str(out, rev.previous.as_deref());
        bin_varint(out, rev.ordinal);
        bin_varint(out, rev.sequence);
        bin_str(out, &rev.event);
        match &rev.outcome {
            None => out.push(0),
            Some(outcome) => {
                out.push(1);
                bin_f64(out, outcome.value);
                bin_str_slice(out, &outcome.because);
                if outcome.materialized.is_empty() {
                    out.push(0);
                } else {
                    out.push(1);
                    bin_str_slice(out, &outcome.materialized);
                }
                bin_varint(out, outcome.sequence);
                bin_str(out, &outcome.event);
                bin_provenance(out, &outcome.lineage);
            }
        }
    }
    bin_provenance(out, &s.selection_qualifications);
    let flags = u8::from(!s.on_hold.is_empty()) | (u8::from(!s.deferred.is_empty()) << 1);
    out.push(flags);
    if flags & 1 != 0 {
        bin_str_slice(out, &s.on_hold);
    }
    if flags & 2 != 0 {
        bin_str_slice(out, &s.deferred);
    }
}

fn bin_journal_entry(out: &mut Vec<u8>, entry: &JournalEntry) {
    bin_str(out, &entry.decision);
    bin_str(out, &entry.commitment);
    bin_str(out, &entry.change);
    bin_varint(out, entry.sequence);
    bin_str(out, &entry.event);
    let flags = u8::from(entry.agent.is_some())
        | (u8::from(entry.elapsed.is_some()) << 1)
        | (u8::from(entry.value.is_some()) << 2)
        | (u8::from(entry.permitted_by.is_some()) << 3)
        | (u8::from(entry.outcome.is_some()) << 4)
        | (u8::from(!entry.materialized.is_empty()) << 5);
    out.push(flags);
    if let Some(agent) = &entry.agent {
        bin_str(out, agent);
    }
    if let Some(elapsed) = entry.elapsed {
        bin_f64(out, elapsed);
    }
    if let Some(value) = entry.value {
        bin_f64(out, value);
    }
    bin_str_slice(out, &entry.because);
    bin_str_slice(out, &entry.caveats);
    if let Some(permitted_by) = &entry.permitted_by {
        bin_str(out, permitted_by);
    }
    if let Some(outcome) = entry.outcome {
        bin_f64(out, outcome);
    }
    if flags & 32 != 0 {
        bin_str_slice(out, &entry.materialized);
    }
}

fn relation_tag(relation: Relation) -> u8 {
    match relation {
        Relation::Supports => 0,
        Relation::Opposes => 1,
        Relation::Qualifies => 2,
        Relation::InContext => 3,
        Relation::Retains => 4,
        Relation::Reopens => 5,
        Relation::ReliesOn => 6,
    }
}

fn bin_relation(out: &mut Vec<u8>, symbols: &NameTable, edge: &Edge) {
    bin_str(out, symbols.name_of(edge.from).expect("a named node"));
    out.push(relation_tag(edge.relation));
    bin_str(out, symbols.name_of(edge.to).expect("a named node"));
}

fn bin_nested_full<V: Clone>(
    out: &mut Vec<u8>,
    held: &mut Nested<V>,
    live: &Nested<V>,
    encode: fn(&mut Vec<u8>, &V),
) {
    held.clone_from(live);
    out.push(1); // outer keys present
    bin_varint(out, live.len() as u64);
    for key in live.keys() {
        bin_str(out, key);
    }
    bin_varint(out, live.len() as u64);
    for (target, props) in live {
        bin_str(out, target);
        out.push(1); // inner keys present
        bin_varint(out, props.len() as u64);
        for prop in props.keys() {
            bin_str(out, prop);
        }
        bin_varint(out, props.len() as u64);
        for (prop, val) in props {
            bin_str(out, prop);
            encode(out, val);
        }
    }
}

fn bin_nested_touched<V: Clone>(
    out: &mut Vec<u8>,
    held: &mut Nested<V>,
    live: &Nested<V>,
    groups: &[BindingGroup],
    touched: &[usize],
    same: &dyn Fn(&V, &V) -> bool,
    encode: fn(&mut Vec<u8>, &V),
) -> bool {
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
                    return false;
                }
                if prop_set.len() > 1 {
                    prop_set.sort_unstable_by_key(|(k, _)| *k);
                    prop_set.dedup_by_key(|(k, _)| *k);
                }
                if outer_keys_changed {
                    out.push(1);
                    bin_varint(out, live.len() as u64);
                    for k in live.keys() {
                        bin_str(out, k);
                    }
                } else {
                    out.push(0);
                }
                if inner_keys_changed || !prop_set.is_empty() {
                    bin_varint(out, 1);
                    bin_str(out, target);
                    if inner_keys_changed {
                        out.push(1);
                        bin_varint(out, live_props.len() as u64);
                        for k in live_props.keys() {
                            bin_str(out, k);
                        }
                    } else {
                        out.push(0);
                    }
                    bin_varint(out, prop_set.len() as u64);
                    for (prop, val) in prop_set {
                        bin_str(out, prop);
                        encode(out, val);
                    }
                } else {
                    bin_varint(out, 0);
                }
                return true;
            } else if outer_keys_changed {
                out.push(1);
                bin_varint(out, live.len() as u64);
                for k in live.keys() {
                    bin_str(out, k);
                }
                bin_varint(out, 0);
                return true;
            }
            return false;
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
        return false;
    }
    if outer_keys_changed {
        out.push(1);
        bin_varint(out, live.len() as u64);
        for k in live.keys() {
            bin_str(out, k);
        }
    } else {
        out.push(0);
    }
    bin_varint(out, active.len() as u64);
    for (target, inner_keys_changed, live_props, prop_set) in active {
        bin_str(out, target);
        if inner_keys_changed {
            out.push(1);
            bin_varint(out, live_props.len() as u64);
            for k in live_props.keys() {
                bin_str(out, k);
            }
        } else {
            out.push(0);
        }
        bin_varint(out, prop_set.len() as u64);
        for (prop, val) in prop_set {
            bin_str(out, prop);
            encode(out, val);
        }
    }
    true
}

fn bin_sync_map<V: Clone>(
    out: &mut Vec<u8>,
    held: &mut BTreeMap<String, V>,
    live: &BTreeMap<String, V>,
    full: bool,
    same: &dyn Fn(&V, &V) -> bool,
    encode: fn(&mut Vec<u8>, &V),
) -> bool {
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
        return false;
    }
    if keys_changed {
        out.push(1);
        bin_varint(out, live.len() as u64);
        for k in live.keys() {
            bin_str(out, k);
        }
    } else {
        out.push(0);
    }
    bin_varint(out, changed.len() as u64);
    for (key, val) in changed {
        bin_str(out, key);
        encode(out, val);
    }
    true
}

fn bin_map_change_one<V>(
    out: &mut Vec<u8>,
    before: &BTreeMap<String, V>,
    after: &BTreeMap<String, V>,
    key: &str,
    same: &dyn Fn(&V, &V) -> bool,
    encode: fn(&mut Vec<u8>, &V),
) -> bool {
    #[cfg(debug_assertions)]
    let start_len = out.len();
    let old_val = before.get(key);
    let new_val = after.get(key);
    let keys_changed = old_val.is_some() != new_val.is_some();
    let changed_val = match (old_val, new_val) {
        (Some(old), Some(new)) if same(old, new) => None,
        (_, Some(new)) => Some(new),
        (_, None) => None,
    };
    if !keys_changed && changed_val.is_none() {
        #[cfg(debug_assertions)]
        {
            let mut expected = Vec::new();
            let wrote = bin_map_changes(&mut expected, Some(before), after, same, encode);
            assert!(
                !wrote && expected.is_empty(),
                "single-key binary patch diff missed change for key {key:?}"
            );
        }
        return false;
    }
    if keys_changed {
        out.push(1);
        bin_varint(out, after.len() as u64);
        for k in after.keys() {
            bin_str(out, k);
        }
    } else {
        out.push(0);
    }
    if let Some(val) = changed_val {
        bin_varint(out, 1);
        bin_str(out, key);
        encode(out, val);
    } else {
        bin_varint(out, 0);
    }
    #[cfg(debug_assertions)]
    {
        let mut expected = Vec::new();
        let wrote = bin_map_changes(&mut expected, Some(before), after, same, encode);
        assert!(wrote);
        assert_eq!(
            &out[start_len..],
            expected.as_slice(),
            "single-key binary patch diff drifted for key {key:?}"
        );
    }
    true
}

fn bin_map_changes<V>(
    out: &mut Vec<u8>,
    before: Option<&BTreeMap<String, V>>,
    after: &BTreeMap<String, V>,
    same: &dyn Fn(&V, &V) -> bool,
    encode: fn(&mut Vec<u8>, &V),
) -> bool {
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
        return false;
    }
    if keys_changed {
        out.push(1);
        bin_varint(out, after.len() as u64);
        for k in after.keys() {
            bin_str(out, k);
        }
    } else {
        out.push(0);
    }
    bin_varint(out, changed.len() as u64);
    for (key, val) in changed {
        bin_str(out, key);
        encode(out, val);
    }
    true
}

fn bin_list_changes<T>(
    out: &mut Vec<u8>,
    before: Option<&[T]>,
    after: &[T],
    same: impl Fn(&T, &T) -> bool,
    render: impl Fn(&mut Vec<u8>, &T),
) -> bool {
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
        return false;
    }
    if let Some(done) = run {
        runs.push(done);
    }
    let item_count = runs.len() + (after.len() - kept);
    bin_varint(out, item_count as u64);
    for (start, length) in runs {
        out.push(0);
        bin_varint(out, start as u64);
        bin_varint(out, length as u64);
    }
    for entry in &after[kept..] {
        out.push(1);
        render(out, entry);
    }
    true
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
        verify_sequence_with_agents(
            source,
            &events
                .iter()
                .map(|&(event, payload)| (event, payload, None))
                .collect::<Vec<_>>(),
        );
    }

    struct BinReader<'a> {
        buf: &'a [u8],
        pos: usize,
    }

    impl<'a> BinReader<'a> {
        fn byte(&mut self) -> u8 {
            let b = self.buf[self.pos];
            self.pos += 1;
            b
        }

        fn varint(&mut self) -> u64 {
            let mut val = 0u64;
            let mut shift = 0u32;
            loop {
                let b = self.byte();
                val |= u64::from(b & 0x7f) << shift;
                if b < 0x80 {
                    return val;
                }
                shift += 7;
            }
        }

        fn f64(&mut self) -> Value {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&self.buf[self.pos..self.pos + 8]);
            self.pos += 8;
            let n = f64::from_le_bytes(bytes);
            serde_json::to_value(n).expect("finite f64")
        }

        fn str(&mut self) -> String {
            let tag = self.varint();
            let len = (tag >> 1) as usize;
            let bytes = &self.buf[self.pos..self.pos + len];
            self.pos += len;
            let s = std::str::from_utf8(bytes).expect("valid utf8");
            assert_eq!(tag & 1 == 0, s.is_ascii(), "ascii flag mismatch for {s:?}");
            s.to_owned()
        }

        fn opt_str(&mut self) -> Value {
            match self.byte() {
                0 => Value::Null,
                1 => Value::String(self.str()),
                other => panic!("unexpected opt_str tag {other}"),
            }
        }

        fn str_list(&mut self) -> Value {
            let len = self.varint() as usize;
            Value::Array((0..len).map(|_| Value::String(self.str())).collect())
        }

        fn binding_value(&mut self) -> Value {
            match self.byte() {
                0 => self.f64(),
                1 => Value::Bool(false),
                2 => Value::Bool(true),
                3 => Value::String(self.str()),
                other => panic!("unexpected binding tag {other}"),
            }
        }

        fn provenance(&mut self) -> Value {
            let flags = self.byte();
            let mut obj = serde_json::Map::new();
            obj.insert("evidence".into(), self.str_list());
            obj.insert("caveats".into(), self.str_list());
            if flags & 1 != 0 {
                let n = self.varint() as usize;
                let mut departed = Vec::with_capacity(n);
                for _ in 0..n {
                    let mut m = serde_json::Map::new();
                    m.insert("history".into(), Value::String(self.str()));
                    m.insert("read".into(), json!(self.varint()));
                    m.insert("from".into(), json!(self.varint()));
                    m.insert("through".into(), json!(self.varint()));
                    m.insert("departed_at".into(), json!(self.varint()));
                    match self.opt_str() {
                        Value::Null => {}
                        Value::String(r) => {
                            m.insert("archive_ref".into(), Value::String(r));
                        }
                        _ => unreachable!(),
                    }
                    departed.push(Value::Object(m));
                }
                obj.insert("departed".into(), Value::Array(departed));
            }
            if flags & 2 != 0 {
                obj.insert("inherited".into(), self.str_list());
            }
            if flags & 4 != 0 {
                obj.insert("kept".into(), self.str_list());
            }
            Value::Object(obj)
        }

        fn effect(&mut self) -> Value {
            let mut obj = serde_json::Map::new();
            match self.byte() {
                0 => {
                    obj.insert("kind".into(), json!("sample"));
                    obj.insert("stream".into(), Value::String(self.str()));
                    obj.insert("id".into(), Value::String(self.str()));
                    obj.insert("value".into(), self.f64());
                    let flags = self.byte();
                    if flags & 1 != 0 {
                        obj.insert("relation".into(), Value::String(self.str()));
                    }
                    if flags & 2 != 0 {
                        obj.insert("target".into(), Value::String(self.str()));
                    }
                    if flags & 4 != 0 {
                        obj.insert("note".into(), Value::String(self.str()));
                    }
                }
                1 => {
                    obj.insert("kind".into(), json!("reveal"));
                    obj.insert("evidence".into(), Value::String(self.str()));
                    let flags = self.byte();
                    if flags & 1 != 0 {
                        obj.insert("relation".into(), Value::String(self.str()));
                    }
                    if flags & 2 != 0 {
                        obj.insert("target".into(), Value::String(self.str()));
                    }
                }
                2 => {
                    obj.insert("kind".into(), json!("examine"));
                    obj.insert("caveat".into(), Value::String(self.str()));
                    obj.insert("cost".into(), json!(self.varint()));
                }
                3 => {
                    obj.insert("kind".into(), json!("commit"));
                    obj.insert("action".into(), Value::String(self.str()));
                    obj.insert("retained".into(), self.str_list());
                }
                4 => {
                    obj.insert("kind".into(), json!("reopen"));
                    obj.insert("action".into(), Value::String(self.str()));
                    obj.insert("because".into(), Value::String(self.str()));
                }
                5 => {
                    obj.insert("kind".into(), json!("qualify"));
                    obj.insert("evidence".into(), Value::String(self.str()));
                    obj.insert("caveat".into(), Value::String(self.str()));
                }
                6 => {
                    obj.insert("kind".into(), json!("unqualify"));
                    obj.insert("evidence".into(), Value::String(self.str()));
                    obj.insert("caveat".into(), Value::String(self.str()));
                    obj.insert("occurrences".into(), self.str_list());
                }
                7 => {
                    obj.insert("kind".into(), json!("renew"));
                    obj.insert("evidence".into(), Value::String(self.str()));
                    obj.insert("occurrence".into(), Value::String(self.str()));
                }
                8 => {
                    obj.insert("kind".into(), json!("withdraw"));
                    obj.insert("evidence".into(), Value::String(self.str()));
                    obj.insert("because".into(), Value::String(self.str()));
                }
                9 => {
                    obj.insert("kind".into(), json!("retire"));
                    obj.insert("history".into(), Value::String(self.str()));
                    obj.insert("record".into(), Value::String(self.str()));
                }
                10 => {
                    obj.insert("kind".into(), json!("depart"));
                    obj.insert("history".into(), Value::String(self.str()));
                    obj.insert("record".into(), Value::String(self.str()));
                }
                other => panic!("unexpected effect kind {other}"),
            }
            Value::Object(obj)
        }

        fn commitment(&mut self) -> Value {
            let mut obj = serde_json::Map::new();
            obj.insert("action".into(), Value::String(self.str()));
            obj.insert("open".into(), Value::Bool(self.byte() != 0));
            obj.insert("retained".into(), self.str_list());
            let auth_len = self.varint() as usize;
            let mut auth = Vec::with_capacity(auth_len);
            for _ in 0..auth_len {
                let mut a = serde_json::Map::new();
                a.insert("caveat".into(), Value::String(self.str()));
                a.insert("written_by".into(), self.opt_str());
                let we = match self.byte() {
                    0 => Value::Null,
                    1 => Value::Bool(false),
                    2 => Value::Bool(true),
                    other => panic!("unexpected written_elsewhere tag {other}"),
                };
                a.insert("written_elsewhere".into(), we);
                auth.push(Value::Object(a));
            }
            obj.insert("retained_authorship".into(), Value::Array(auth));
            obj.insert("reopened_by".into(), self.str_list());
            Value::Object(obj)
        }

        fn decision_series(&mut self) -> Value {
            let mut obj = serde_json::Map::new();
            obj.insert("limit".into(), json!(self.varint()));
            obj.insert("current".into(), self.opt_str());
            let rev_len = self.varint() as usize;
            let mut revs = Vec::with_capacity(rev_len);
            for _ in 0..rev_len {
                let mut r = serde_json::Map::new();
                r.insert("id".into(), Value::String(self.str()));
                r.insert("previous".into(), self.opt_str());
                r.insert("ordinal".into(), json!(self.varint()));
                r.insert("sequence".into(), json!(self.varint()));
                r.insert("event".into(), Value::String(self.str()));
                if self.byte() == 1 {
                    let mut o = serde_json::Map::new();
                    o.insert("value".into(), self.f64());
                    o.insert("because".into(), self.str_list());
                    if self.byte() == 1 {
                        o.insert("materialized".into(), self.str_list());
                    }
                    o.insert("sequence".into(), json!(self.varint()));
                    o.insert("event".into(), Value::String(self.str()));
                    o.insert("lineage".into(), self.provenance());
                    r.insert("outcome".into(), Value::Object(o));
                }
                revs.push(Value::Object(r));
            }
            obj.insert("revisions".into(), Value::Array(revs));
            obj.insert("selection_qualifications".into(), self.provenance());
            let flags = self.byte();
            if flags & 1 != 0 {
                obj.insert("on_hold".into(), self.str_list());
            }
            if flags & 2 != 0 {
                obj.insert("deferred".into(), self.str_list());
            }
            Value::Object(obj)
        }

        fn journal_entry(&mut self) -> Value {
            let mut obj = serde_json::Map::new();
            obj.insert("decision".into(), Value::String(self.str()));
            obj.insert("commitment".into(), Value::String(self.str()));
            obj.insert("change".into(), Value::String(self.str()));
            obj.insert("sequence".into(), json!(self.varint()));
            obj.insert("event".into(), Value::String(self.str()));
            let flags = self.byte();
            if flags & 1 != 0 {
                obj.insert("agent".into(), Value::String(self.str()));
            }
            if flags & 2 != 0 {
                obj.insert("elapsed".into(), self.f64());
            }
            if flags & 4 != 0 {
                obj.insert("value".into(), self.f64());
            }
            obj.insert("because".into(), self.str_list());
            obj.insert("caveats".into(), self.str_list());
            if flags & 8 != 0 {
                obj.insert("permitted_by".into(), Value::String(self.str()));
            }
            if flags & 16 != 0 {
                obj.insert("outcome".into(), self.f64());
            }
            if flags & 32 != 0 {
                obj.insert("materialized".into(), self.str_list());
            }
            Value::Object(obj)
        }

        fn relation(&mut self) -> Value {
            const NAMES: [&str; 7] = [
                "supports",
                "opposes",
                "qualifies",
                "in_context",
                "retains",
                "reopens",
                "relies_on",
            ];
            let from = self.str();
            let rel = NAMES[self.byte() as usize];
            let to = self.str();
            json!({ "from": from, "relation": rel, "to": to, "origin": "live" })
        }

        fn map(&mut self, held: Option<&Value>, read_val: fn(&mut Self) -> Value) -> Value {
            let has_keys = self.byte();
            let keys: Vec<String> = if has_keys == 1 {
                let n = self.varint() as usize;
                (0..n).map(|_| self.str()).collect()
            } else {
                held.unwrap().as_object().unwrap().keys().cloned().collect()
            };
            let set_len = self.varint() as usize;
            let mut updates = BTreeMap::new();
            for _ in 0..set_len {
                let k = self.str();
                updates.insert(k, read_val(self));
            }
            let mut out = serde_json::Map::new();
            for k in keys {
                let v = updates
                    .remove(&k)
                    .unwrap_or_else(|| held.unwrap()[&k].clone());
                out.insert(k, v);
            }
            Value::Object(out)
        }

        fn nested(&mut self, held: Option<&Value>, read_val: fn(&mut Self) -> Value) -> Value {
            let has_keys = self.byte();
            let keys: Vec<String> = if has_keys == 1 {
                let n = self.varint() as usize;
                (0..n).map(|_| self.str()).collect()
            } else {
                held.unwrap().as_object().unwrap().keys().cloned().collect()
            };
            let set_len = self.varint() as usize;
            let mut updates = BTreeMap::new();
            for _ in 0..set_len {
                let target = self.str();
                let inner = self.map(held.and_then(|h| h.get(&target)), read_val);
                updates.insert(target, inner);
            }
            let mut out = serde_json::Map::new();
            for k in keys {
                let v = updates
                    .remove(&k)
                    .unwrap_or_else(|| held.unwrap()[&k].clone());
                out.insert(k, v);
            }
            Value::Object(out)
        }

        fn commitments(&mut self, held: Option<&Value>) -> Value {
            let held_map = held.map(|h| {
                let mut m = BTreeMap::new();
                for c in h.as_array().unwrap() {
                    m.insert(c["action"].as_str().unwrap().to_owned(), c.clone());
                }
                m
            });
            let has_keys = self.byte();
            let keys: Vec<String> = if has_keys == 1 {
                let n = self.varint() as usize;
                (0..n).map(|_| self.str()).collect()
            } else {
                held.unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| c["action"].as_str().unwrap().to_owned())
                    .collect()
            };
            let set_len = self.varint() as usize;
            let mut updates = BTreeMap::new();
            for _ in 0..set_len {
                let k = self.str();
                updates.insert(k, self.commitment());
            }
            Value::Array(
                keys.into_iter()
                    .map(|k| {
                        updates
                            .remove(&k)
                            .unwrap_or_else(|| held_map.as_ref().unwrap()[&k].clone())
                    })
                    .collect(),
            )
        }

        fn list(&mut self, held: Option<&Value>, read_item: fn(&mut Self) -> Value) -> Value {
            let count = self.varint() as usize;
            let mut out = Vec::new();
            for _ in 0..count {
                match self.byte() {
                    0 => {
                        let start = self.varint() as usize;
                        let len = self.varint() as usize;
                        let src = held.unwrap().as_array().unwrap();
                        out.extend_from_slice(&src[start..start + len]);
                    }
                    1 => out.push(read_item(self)),
                    other => panic!("unexpected list tag {other}"),
                }
            }
            Value::Array(out)
        }
    }

    fn decode_binary_outcome(bytes: &[u8], held: Option<&Value>) -> Value {
        let mut r = BinReader { buf: bytes, pos: 0 };
        let outcome = r.byte();
        if outcome == 0 {
            const ORIGINS: [&str; 4] = ["policy", "input", "evaluation", "limit"];
            let origin = ORIGINS[r.byte() as usize];
            let code = r.str();
            let message = r.str();
            assert_eq!(r.pos, bytes.len());
            return json!({
                "schema": DISPATCH_SCHEMA,
                "outcome": "rejected",
                "origin": origin,
                "code": code,
                "message": message,
            });
        }
        assert_eq!(outcome, 1);
        let since = if r.byte() == 1 {
            Some(r.varint())
        } else {
            None
        };
        let base = since.map(|seq| {
            let h = held.expect("held view for since");
            assert_eq!(h["sequence"].as_u64(), Some(seq));
            h
        });
        let sequence = r.varint();
        let last_event = r.opt_str();
        let mask = u16::from_le_bytes([r.byte(), r.byte()]);
        let bindings = if mask & 1 != 0 {
            r.nested(base.map(|b| &b["bindings"]), BinReader::binding_value)
        } else {
            base.unwrap()["bindings"].clone()
        };
        let binding_explanations = if mask & 2 != 0 {
            r.nested(
                base.map(|b| &b["binding_explanations"]),
                BinReader::provenance,
            )
        } else {
            base.unwrap()["binding_explanations"].clone()
        };
        let cues = if mask & 4 != 0 {
            if r.byte() == 0 {
                Value::Array(Vec::new())
            } else {
                serde_json::from_str(&r.str()).unwrap()
            }
        } else {
            base.unwrap()["cues"].clone()
        };
        let effects = if mask & 8 != 0 {
            let n = r.varint() as usize;
            Value::Array((0..n).map(|_| r.effect()).collect())
        } else {
            base.unwrap()["effects"].clone()
        };
        let commitments = if mask & 16 != 0 {
            r.commitments(base.map(|b| &b["commitments"]))
        } else {
            base.unwrap()["commitments"].clone()
        };
        let commitment_grounds = if mask & 32 != 0 {
            r.map(
                base.map(|b| &b["commitment_grounds"]),
                BinReader::provenance,
            )
        } else {
            base.unwrap()["commitment_grounds"].clone()
        };
        let decision_series = if mask & 64 != 0 {
            r.map(
                base.map(|b| &b["decision_series"]),
                BinReader::decision_series,
            )
        } else {
            base.unwrap()["decision_series"].clone()
        };
        let decision_journal = if mask & 128 != 0 {
            r.list(
                base.map(|b| &b["decision_journal"]),
                BinReader::journal_entry,
            )
        } else {
            base.unwrap()["decision_journal"].clone()
        };
        let relations = if mask & 256 != 0 {
            r.list(base.map(|b| &b["relations"]), BinReader::relation)
        } else {
            base.unwrap()["relations"].clone()
        };
        assert_eq!(r.pos, bytes.len(), "trailing bytes in binary patch");
        let mut view = serde_json::Map::new();
        view.insert("schema".into(), json!(REACTIVE_VIEW_SCHEMA));
        view.insert("sequence".into(), json!(sequence));
        view.insert("last_event".into(), last_event);
        view.insert("bindings".into(), bindings);
        view.insert("binding_explanations".into(), binding_explanations);
        view.insert("cues".into(), cues);
        view.insert("effects".into(), effects);
        view.insert("commitments".into(), commitments);
        view.insert("commitment_grounds".into(), commitment_grounds);
        view.insert("decision_series".into(), decision_series);
        view.insert("decision_journal".into(), decision_journal);
        view.insert("relations".into(), relations);
        json!({
            "schema": DISPATCH_SCHEMA,
            "outcome": "accepted",
            "view": Value::Object(view),
        })
    }

    fn verify_sequence_with_agents(source: &str, events: &[(&str, &str, Option<&str>)]) {
        let mut plain = ReactiveSession::from_source(source).unwrap();
        let mut patched = ReactiveSession::from_source(source).unwrap();
        let mut streamed = ReactiveSession::from_source(source).unwrap();
        let mut binary = ReactiveSession::from_source(source).unwrap();
        let mut base = ViewPatchBase::default();
        let mut base_text = ViewPatchBase::default();
        let mut base_bin = ViewPatchBase::default();
        let mut held: Option<Value> = None;
        let mut held_bin: Option<Value> = None;
        for &(event, payload, agent) in events {
            let expected_raw = serde_json::to_string(
                &plain
                    .dispatch_view_outcome_as_json(event, payload, agent)
                    .unwrap(),
            )
            .unwrap();
            let expected_val: Value = serde_json::from_str(&expected_raw).unwrap();
            let expected = serde_json::to_string(&expected_val).unwrap();
            let seq = held.as_ref().and_then(|v| v["sequence"].as_u64());
            let patch_raw = serde_json::to_string(
                &patched
                    .dispatch_view_patch_outcome_json_from(&mut base, seq, event, payload, agent)
                    .unwrap(),
            )
            .unwrap();
            let streamed_raw = streamed
                .dispatch_view_patch_outcome_text_from(&mut base_text, seq, event, payload, agent)
                .unwrap();
            assert_eq!(
                streamed_raw, patch_raw,
                "streamed != structured on {event} {payload}"
            );
            let bin_bytes = binary
                .dispatch_view_patch_outcome_binary_from(&mut base_bin, seq, event, payload, agent)
                .unwrap();
            let bin_outcome = decode_binary_outcome(&bin_bytes, held_bin.as_ref());
            assert_eq!(
                serde_json::to_string(&bin_outcome).unwrap(),
                expected,
                "binary != expected on {event} {payload}"
            );
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
                held_bin = Some(bin_outcome["view"].clone());
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

    #[test]
    fn binary_view_patch_covers_outcomes_holds_authorship_agents_effects_and_cues() {
        let relation_variants = [
            Relation::Supports,
            Relation::Opposes,
            Relation::Qualifies,
            Relation::InContext,
            Relation::Retains,
            Relation::Reopens,
            Relation::ReliesOn,
        ];
        const NAMES: [&str; 7] = [
            "supports",
            "opposes",
            "qualifies",
            "in_context",
            "retains",
            "reopens",
            "relies_on",
        ];
        for relation in relation_variants {
            assert_eq!(
                NAMES[relation_tag(relation) as usize],
                relation_name(relation)
            );
        }

        let source = r#"
origin ext;
caveat remote_c consequence material;
origin main;
claim ok;
claim permit;
evidence sensor from "sensor";
evidence grant from "grant";
evidence pause_reason from "pause";
evidence recheck from "recheck";
caveat local_c consequence low;
caveat taken_c consequence low;
renewable grant limit 4;
identifiers limit 8;
readings r from sensor window 2;
decisions d limit 4 reopened by r;
journal window 4;
agent main sends decide, pause, resume, close, wrong;
agent worker sends sample_r, flag, unflag, renew_g, retract;
state label_on = 0 min 0 max 1;
cue ping sound 440 0.2 0.5;
event tick dt min 0 max 0.1;
event sample_r v min -10 max 10, note_id id;
event flag;
event unflag;
event renew_g;
event decide;
event pause;
event resume;
event close;
event retract;
event wrong;
on sample_r sample r = v supports ok noting note_id;
on sample_r emit ping;
on flag qualify latest(r) with taken_c;
on flag qualify latest(r) with remote_c;
on unflag unqualify latest(r) with taken_c;
on renew_g when not observed(grant) reveal grant supports permit;
on renew_g renew grant;
on decide when not observed(grant) reveal grant supports permit;
on decide commit d because enough using latest(r) permitted by grant retaining local_c, remote_c;
on pause reveal pause_reason supports ok;
on pause hold d because pause_reason;
on resume release d because pause_reason;
on close resolve d = 1 because latest(r) materialized remote_c;
on retract reveal recheck opposes ok;
on retract withdraw latest(r) because recheck;
on wrong reject "No.";
bind hud.flag = label_on == 1;
bind hud.note = "café — 東京" when label_on == 0;
"#;
        verify_sequence_with_agents(
            source,
            &[
                ("tick", r#"{"dt":0.05}"#, None),
                ("sample_r", r#"{"v":2.5,"note_id":"αβγ-1"}"#, Some("worker")),
                ("flag", "{}", Some("worker")),
                ("decide", "{}", Some("main")),
                ("pause", "{}", Some("main")),
                (
                    "sample_r",
                    r#"{"v":3.0,"note_id":"deferred-2"}"#,
                    Some("worker"),
                ),
                ("resume", "{}", Some("main")),
                ("decide", "{}", Some("main")),
                ("close", "{}", Some("main")),
                ("unflag", "{}", Some("worker")),
                ("renew_g", "{}", Some("worker")),
                ("retract", "{}", Some("worker")),
                (
                    "sample_r",
                    r#"{"v":4.0,"note_id":"window-3"}"#,
                    Some("worker"),
                ),
                (
                    "sample_r",
                    r#"{"v":5.0,"note_id":"window-4"}"#,
                    Some("worker"),
                ),
                ("wrong", "{}", Some("main")),
                ("decide", "{}", Some("worker")), // wrong_agent rejection
                ("tick", r#"{"dt":0.1}"#, None),
            ],
        );
    }
}
