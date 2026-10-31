//! View 0.2 and its delta. See spec/caveat-view-0.2.md.
//!
//! The full view orders each commitment's grounds evidence as it was first
//! observed, frozen when the commitment was made, orders each binding
//! explanation's evidence the same way as of the view, and says `reopened`
//! where View 0.1 says `open`. A delta is built by comparing the full view before
//! and after one accepted event, so applying it gives exactly the next view.
use super::outcome::{DispatchFailure, DispatchFatal, RejectionCode, RejectionOrigin};
use super::{
    relation_name, BindingValue, Cue, DecisionSeries, EffectReport, JournalEntry, ObservationRanks,
    ReactiveSession, TakeBack, DISPATCH_SCHEMA,
};
use crate::chunked::Chunked;
use crate::fast_hash::HashMap;
use crate::map::{MapCommitment, MapRelation, RetainedCaveat};
use crate::reactive_expr::{departure_markers, Marker, Name, NameSet, Provenance};
use crate::{Edge, EpistemicGraph, NodeId};
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub const REACTIVE_VIEW_V2_SCHEMA: &str = "caveat-reactive-view/0.2";
pub const REACTIVE_VIEW_DELTA_SCHEMA: &str = "caveat-reactive-view-delta/0.2";

/// View 0.2: View 0.1 with ordered grounds evidence and `reopened`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReactiveViewV2<'a> {
    pub schema: &'static str,
    pub sequence: u64,
    pub last_event: Option<&'a str>,
    pub bindings: &'a BTreeMap<String, BTreeMap<String, BindingValue>>,
    pub binding_explanations: BTreeMap<&'a str, BTreeMap<&'a str, OrderedGrounds<'a>>>,
    pub cues: &'a [Cue],
    pub effects: &'a [EffectReport],
    pub commitments: Vec<ViewCommitment>,
    pub commitment_grounds: BTreeMap<&'a str, OrderedGrounds<'a>>,
    pub decision_series: &'a BTreeMap<String, DecisionSeries>,
    pub decision_journal: &'a crate::chunked::Chunked<JournalEntry>,
    pub relations: Vec<MapRelation>,
    /// The snapshot's take-back records, only in a program that takes back.
    /// See spec/caveat-take-back-0.1.md, "Views".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taken_back: Option<&'a [TakeBack]>,
}

/// A commitment as View 0.2 shows it: `reopened` in place of 0.1's `open`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ViewCommitment {
    pub action: String,
    pub reopened: bool,
    pub retained: Vec<String>,
    pub retained_authorship: Vec<RetainedCaveat>,
    pub reopened_by: Vec<String>,
}

impl From<MapCommitment> for ViewCommitment {
    fn from(commitment: MapCommitment) -> Self {
        Self {
            action: commitment.action,
            reopened: commitment.open,
            retained: commitment.retained,
            retained_authorship: commitment.retained_authorship,
            reopened_by: commitment.reopened_by,
        }
    }
}

/// A commitment's grounds, or a binding explanation, with evidence in
/// first-observed order. Every other member serializes as `Provenance` does.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OrderedGrounds<'a> {
    pub evidence: Vec<&'a str>,
    pub caveats: &'a NameSet,
    #[serde(
        skip_serializing_if = "no_markers",
        serialize_with = "serialize_markers"
    )]
    pub departed: &'a BTreeMap<String, Marker>,
    #[serde(skip_serializing_if = "no_names")]
    pub inherited: &'a NameSet,
    #[serde(skip_serializing_if = "no_names")]
    pub kept: &'a NameSet,
}

impl<'a> OrderedGrounds<'a> {
    fn new(evidence: Vec<&'a str>, provenance: &'a Provenance) -> Self {
        Self {
            evidence,
            caveats: &provenance.caveats,
            departed: &provenance.departed,
            inherited: &provenance.inherited,
            kept: &provenance.kept,
        }
    }
}

/// An explanation's evidence in first-observed order as of now: observed
/// names by where they were first observed, then any never observed, by name.
fn ranked<'a>(ranks: &ObservationRanks<'_>, explanation: &'a Provenance) -> Vec<&'a Name> {
    let mut evidence = explanation
        .evidence
        .iter()
        .map(|name| (ranks.rank(name), name))
        .collect::<Vec<_>>();
    evidence.sort_unstable();
    evidence.into_iter().map(|(_, name)| name).collect()
}

fn no_markers(markers: &&BTreeMap<String, Marker>) -> bool {
    markers.is_empty()
}

fn no_names(names: &&NameSet) -> bool {
    names.is_empty()
}

fn serialize_markers<S: Serializer>(
    markers: &&BTreeMap<String, Marker>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    departure_markers::serialize(markers, serializer)
}

/// Evidence in the frozen order: names the order lists first, in its order,
/// then any it does not list, by name. Departed names have left `evidence`
/// and so drop out; the rest keep their places.
pub(super) fn ordered<'a>(evidence: &'a NameSet, order: Option<&[String]>) -> Vec<&'a str> {
    let Some(order) = order else {
        return evidence.iter().map(Name::as_str).collect();
    };
    let mut names = order
        .iter()
        .filter_map(|name| evidence.get(name.as_str()).map(Name::as_str))
        .collect::<Vec<_>>();
    if names.len() < evidence.len() {
        let listed = order.iter().map(String::as_str).collect::<BTreeSet<_>>();
        names.extend(
            evidence
                .iter()
                .map(Name::as_str)
                .filter(|name| !listed.contains(name)),
        );
    }
    names
}

/// What `dispatch_view_delta_outcome_json` returns: on acceptance the delta
/// from the view before the event to the view after it; a refusal or fatal
/// error exactly as `dispatch_outcome_json` reports it.
#[derive(Debug, Serialize)]
pub struct DispatchViewDeltaOutcome {
    pub schema: &'static str,
    #[serde(flatten)]
    pub result: DispatchViewDeltaResult,
}

#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum DispatchViewDeltaResult {
    Accepted {
        delta: Box<ViewDelta>,
    },
    Rejected {
        origin: RejectionOrigin,
        code: RejectionCode,
        message: String,
    },
}

/// How the full 0.2 view changed across one accepted event.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ViewDelta {
    pub schema: &'static str,
    pub since: u64,
    pub sequence: u64,
    pub last_event: Value,
    pub bindings: PropertyChanges,
    pub binding_explanations: PropertyChanges,
    pub cues: Value,
    pub effects: Value,
    pub commitments: CommitmentChanges,
    pub commitment_grounds: NamedChanges,
    pub decision_series: NamedChanges,
    pub decision_journal: ListChanges,
    pub relations: ListChanges,
    /// Only in a program that takes back, as the view's member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taken_back: Option<ListChanges>,
}

/// Changed `[target, property]` values, and the pairs that went.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PropertyChanges {
    pub set: Map<String, Value>,
    pub removed: Vec<[String; 2]>,
}

/// Changed or added commitments, whole, and the actions that went.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CommitmentChanges {
    pub set: Vec<Value>,
    pub removed: Vec<String>,
}

/// Changed or added members by name, whole, and the names that went.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct NamedChanges {
    pub set: Map<String, Value>,
    pub removed: Vec<String>,
}

/// Positions dropped from the old list, ascending, then entries appended.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ListChanges {
    pub removed: Vec<usize>,
    pub appended: Vec<Value>,
}

impl ReactiveSession {
    /// The per-event view in schema 0.2. See spec/caveat-view-0.2.md.
    pub fn view_v2(&self) -> ReactiveViewV2<'_> {
        let names = self.node_names();
        ReactiveViewV2 {
            schema: REACTIVE_VIEW_V2_SCHEMA,
            sequence: self.sequence,
            last_event: self.last_event.as_deref(),
            bindings: &self.bindings,
            binding_explanations: self.ordered_explanations(),
            cues: &self.cues,
            effects: &self.effects,
            commitments: self
                .commitment_records(|id| names.get(id).copied())
                .into_iter()
                .map(ViewCommitment::from)
                .collect(),
            commitment_grounds: self
                .commitment_grounds
                .iter()
                .map(|(name, grounds)| {
                    let order = self.evidence_order.get(name).map(Vec::as_slice);
                    (
                        name.as_str(),
                        OrderedGrounds::new(ordered(&grounds.evidence, order), grounds),
                    )
                })
                .collect(),
            decision_series: &self.decision_series,
            decision_journal: &self.journal,
            relations: self.relation_records(&names),
            taken_back: (!self.take_back_caveats.is_empty()).then_some(self.taken_back.as_slice()),
        }
    }

    /// Each binding explanation with its evidence in first-observed order as
    /// of now: observed names by where they were first observed, then any
    /// never observed, by name. Caveats stay sorted.
    fn ordered_explanations(&self) -> BTreeMap<&str, BTreeMap<&str, OrderedGrounds<'_>>> {
        let ranks = self.observation_ranks();
        self.binding_explanations
            .iter()
            .map(|(target, properties)| {
                let properties = properties
                    .iter()
                    .map(|(property, explanation)| {
                        let evidence = ranked(&ranks, explanation)
                            .into_iter()
                            .map(Name::as_str)
                            .collect();
                        (
                            property.as_str(),
                            OrderedGrounds::new(evidence, explanation),
                        )
                    })
                    .collect();
                (target.as_str(), properties)
            })
            .collect()
    }

    fn node_names(&self) -> HashMap<NodeId, &str> {
        names(&self.symbols)
    }

    /// What a delta compares: the view's members as values, shared with the
    /// session where it shares them.
    fn view_parts(&self) -> ViewParts {
        let ranks = self.observation_ranks();
        let names = self.node_names();
        ViewParts {
            bindings: self.bindings.clone(),
            binding_explanations: self
                .binding_explanations
                .iter()
                .map(|(target, properties)| {
                    let properties = properties
                        .iter()
                        .map(|(property, explanation)| {
                            let evidence =
                                ranked(&ranks, explanation).into_iter().cloned().collect();
                            (property.clone(), (evidence, explanation.clone()))
                        })
                        .collect();
                    (target.clone(), properties)
                })
                .collect(),
            commitments: self
                .commitment_records(|id| names.get(id).copied())
                .into_iter()
                .map(|commitment| (commitment.action.clone(), ViewCommitment::from(commitment)))
                .collect(),
            commitment_grounds: Arc::clone(&self.commitment_grounds),
            evidence_order: Arc::clone(&self.evidence_order),
            decision_series: Arc::clone(&self.decision_series),
            decision_journal: self.journal.clone(),
            graph: Arc::clone(&self.graph),
            symbols: Arc::clone(&self.symbols),
            taken_back: (!self.take_back_caveats.is_empty()).then(|| Arc::clone(&self.taken_back)),
        }
    }

    /// Dispatch one event and return the View 0.2 delta it caused. The same
    /// transaction as `dispatch_outcome_json`: a refusal changes nothing and
    /// is reported as there, with no delta, and so is a fatal error.
    pub fn dispatch_view_delta_outcome_json(
        &mut self,
        event: &str,
        payload_json: &str,
    ) -> Result<DispatchViewDeltaOutcome, DispatchFatal> {
        self.dispatch_view_delta_outcome_json_from(
            &mut ViewDeltaBase::default(),
            event,
            payload_json,
        )
    }

    /// `dispatch_view_delta_outcome_json`, comparing against `base` when it
    /// holds the view as of now rather than building that view again, and
    /// leaving in it the view after the event. A caller keeps `base` only
    /// while nothing but this method changes the session, and starts again
    /// from `ViewDeltaBase::default()` otherwise.
    pub fn dispatch_view_delta_outcome_json_from(
        &mut self,
        base: &mut ViewDeltaBase,
        event: &str,
        payload_json: &str,
    ) -> Result<DispatchViewDeltaOutcome, DispatchFatal> {
        self.dispatch_view_delta_outcome_as_json_from(base, event, payload_json, None)
    }

    /// `dispatch_view_delta_outcome_json_from` for an event the host says
    /// `agent` sent. See spec/caveat-agents-0.1.md.
    pub fn dispatch_view_delta_outcome_as_json_from(
        &mut self,
        base: &mut ViewDeltaBase,
        event: &str,
        payload_json: &str,
        agent: Option<&str>,
    ) -> Result<DispatchViewDeltaOutcome, DispatchFatal> {
        let since = self.sequence;
        let before = match base.0.take() {
            Some((sequence, parts)) if sequence == since => parts,
            _ => self.view_parts(),
        };
        match self.dispatch_classified(event, payload_json, agent) {
            Ok(()) => {
                let after = self.view_parts();
                let delta = view_delta(since, self, &before, &after);
                base.0 = Some((self.sequence, after));
                Ok(DispatchViewDeltaOutcome {
                    schema: DISPATCH_SCHEMA,
                    result: DispatchViewDeltaResult::Accepted {
                        delta: Box::new(delta),
                    },
                })
            }
            Err(failure) => {
                let outcome = delta_outcome(failure);
                // A refusal changes nothing, so the view still stands.
                if outcome.is_ok() {
                    base.0 = Some((since, before));
                }
                outcome
            }
        }
    }
}

/// The View 0.2 view as of a sequence, kept between calls to
/// `dispatch_view_delta_outcome_json_from` so each builds one view, not two.
#[derive(Debug, Default)]
pub struct ViewDeltaBase(Option<(u64, ViewParts)>);

impl ViewDeltaBase {
    /// Forget the kept view, as after anything else changes the session.
    pub fn clear(&mut self) {
        self.0 = None;
    }
}

fn delta_outcome(failure: DispatchFailure) -> Result<DispatchViewDeltaOutcome, DispatchFatal> {
    let (origin, code, message) = failure.classify()?;
    Ok(DispatchViewDeltaOutcome {
        schema: DISPATCH_SCHEMA,
        result: DispatchViewDeltaResult::Rejected {
            origin,
            code,
            message,
        },
    })
}

/// The view's members as of one sequence, held as values: what a delta
/// compares. Members the session shares are held shared, so a member the
/// event left alone is the same allocation on both sides and is skipped, and
/// only what changed is rendered.
#[derive(Debug)]
pub(super) struct ViewParts {
    bindings: BTreeMap<String, BTreeMap<String, BindingValue>>,
    /// Each explanation with its evidence in the order the view shows it.
    binding_explanations: BTreeMap<String, BTreeMap<String, (Vec<Name>, Provenance)>>,
    /// By action: a later commitment with an action already seen replaces
    /// the earlier one.
    commitments: BTreeMap<String, ViewCommitment>,
    commitment_grounds: Arc<BTreeMap<String, Provenance>>,
    evidence_order: Arc<BTreeMap<String, Vec<String>>>,
    decision_series: Arc<BTreeMap<String, DecisionSeries>>,
    decision_journal: Chunked<JournalEntry>,
    /// The relations are the graph's edges, named through the symbols.
    graph: Arc<EpistemicGraph>,
    symbols: Arc<HashMap<String, NodeId>>,
    taken_back: Option<Arc<Vec<TakeBack>>>,
}

fn text<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string(value).expect("finite reactive view")
}

fn value<T: Serialize + ?Sized>(value: &T) -> Value {
    serde_json::to_value(value).expect("finite reactive view")
}

fn parse(text: &str) -> Value {
    serde_json::from_str(text).expect("a view part is JSON")
}

/// Equal as the spec counts equality: as JSON values, with `-0` distinct
/// from `0`, which `Value`'s own `==` does not keep apart.
fn identical(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(p), Some(q)) if x.is_f64() || y.is_f64() => p.to_bits() == q.to_bits(),
            _ => x == y,
        },
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| identical(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(key, p)| y.get(key).is_some_and(|q| identical(p, q)))
        }
        _ => a == b,
    }
}

/// The same text is the same value; different texts are compared as values.
fn same<T: Serialize + ?Sized>(a: &T, b: &T) -> bool {
    let (a, b) = (text(a), text(b));
    a == b || identical(&parse(&a), &parse(&b))
}

/// A number compares by its bits, so `-0` stays apart from `0`; every other
/// binding value has one form.
fn same_binding(a: &BindingValue, b: &BindingValue) -> bool {
    match (a, b) {
        (BindingValue::Number(x), BindingValue::Number(y)) => x.to_bits() == y.to_bits(),
        _ => a == b,
    }
}

/// Grounds as the view shows them, `evidence` already in the view's order.
fn shown<'a>(evidence: &'a [Name], grounds: &'a Provenance) -> OrderedGrounds<'a> {
    OrderedGrounds::new(evidence.iter().map(Name::as_str).collect(), grounds)
}

fn view_delta(
    since: u64,
    session: &ReactiveSession,
    before: &ViewParts,
    after: &ViewParts,
) -> ViewDelta {
    ViewDelta {
        schema: REACTIVE_VIEW_DELTA_SCHEMA,
        since,
        sequence: session.sequence,
        last_event: value(&session.last_event),
        bindings: property_changes(&before.bindings, &after.bindings, same_binding, |binding| {
            value(binding)
        }),
        binding_explanations: property_changes(
            &before.binding_explanations,
            &after.binding_explanations,
            |(old_order, old), (new_order, new)| old == new && old_order == new_order,
            |(order, grounds)| value(&shown(order, grounds)),
        ),
        cues: value(&session.cues),
        effects: value(&session.effects),
        commitments: commitment_changes(&before.commitments, &after.commitments),
        commitment_grounds: grounds_changes(before, after),
        decision_series: if Arc::ptr_eq(&before.decision_series, &after.decision_series) {
            NamedChanges::default()
        } else {
            named_changes(
                &before.decision_series,
                &after.decision_series,
                |_, old, new| same(old, new),
                |_, series| value(series),
            )
        },
        decision_journal: list_changes(
            &before.decision_journal.iter().collect::<Vec<_>>(),
            &after.decision_journal.iter().collect::<Vec<_>>(),
            |old, new| std::ptr::eq(*old, *new) || same(*old, *new),
            |entry| value(*entry),
        ),
        relations: relation_changes(before, after),
        taken_back: after
            .taken_back
            .as_deref()
            .map(|records| match &before.taken_back {
                Some(old) if std::ptr::eq(old.as_slice(), records.as_slice()) => {
                    ListChanges::default()
                }
                old => list_changes(
                    old.as_deref().map(Vec::as_slice).unwrap_or_default(),
                    records,
                    same,
                    value,
                ),
            }),
    }
}

fn names(symbols: &HashMap<String, NodeId>) -> HashMap<NodeId, &str> {
    symbols
        .iter()
        .map(|(name, id)| (*id, name.as_str()))
        .collect()
}

fn relation(names: &HashMap<NodeId, &str>, edge: &Edge) -> MapRelation {
    MapRelation {
        from: names[&edge.from].into(),
        to: names[&edge.to].into(),
        relation: relation_name(edge.relation).into(),
        origin: "live".into(),
    }
}

/// One relation per edge, as `relation_records` names them. While the
/// symbols are the same, an edge with the same ends and relation is the same
/// relation, so only the edges appended are named; otherwise the ends are
/// also compared by name.
fn relation_changes(before: &ViewParts, after: &ViewParts) -> ListChanges {
    let named = OnceCell::new();
    let named = || named.get_or_init(|| names(&after.symbols));
    let edges = |old: &Edge, new: &Edge| {
        old.from == new.from && old.to == new.to && old.relation == new.relation
    };
    if Arc::ptr_eq(&before.symbols, &after.symbols) {
        list_changes(&before.graph.edges, &after.graph.edges, edges, |edge| {
            value(&relation(named(), edge))
        })
    } else {
        let old_names = names(&before.symbols);
        list_changes(
            &before.graph.edges,
            &after.graph.edges,
            |old, new| {
                edges(old, new)
                    && old_names[&old.from] == named()[&new.from]
                    && old_names[&old.to] == named()[&new.to]
            },
            |edge| value(&relation(named(), edge)),
        )
    }
}

type Properties<V> = BTreeMap<String, BTreeMap<String, V>>;

fn property_changes<V>(
    before: &Properties<V>,
    after: &Properties<V>,
    same: impl Fn(&V, &V) -> bool,
    value: impl Fn(&V) -> Value,
) -> PropertyChanges {
    let empty = BTreeMap::new();
    let mut changes = PropertyChanges::default();
    for (target, properties) in after {
        let old = before.get(target).unwrap_or(&empty);
        let mut set = Map::new();
        for (property, new) in properties {
            if !old.get(property).is_some_and(|known| same(known, new)) {
                set.insert(property.clone(), value(new));
            }
        }
        if !set.is_empty() {
            changes.set.insert(target.clone(), Value::Object(set));
        }
    }
    for (target, properties) in before {
        let new = after.get(target);
        for property in properties.keys() {
            if !new.is_some_and(|new| new.contains_key(property)) {
                changes.removed.push([target.clone(), property.clone()]);
            }
        }
    }
    changes
}

/// A commitment holds only names and flags, so equal values are equal JSON.
fn commitment_changes(
    before: &BTreeMap<String, ViewCommitment>,
    after: &BTreeMap<String, ViewCommitment>,
) -> CommitmentChanges {
    let NamedChanges { set, removed } =
        named_changes(before, after, |_, old, new| old == new, |_, new| value(new));
    CommitmentChanges {
        set: set.into_iter().map(|(_, commitment)| commitment).collect(),
        removed,
    }
}

/// Commitment grounds with evidence in each commitment's frozen order. Equal
/// provenance in the same order is the same JSON.
fn grounds_changes(before: &ViewParts, after: &ViewParts) -> NamedChanges {
    if Arc::ptr_eq(&before.commitment_grounds, &after.commitment_grounds)
        && Arc::ptr_eq(&before.evidence_order, &after.evidence_order)
    {
        return NamedChanges::default();
    }
    let evidence = |parts: &ViewParts, name: &str, grounds: &Provenance| {
        let order = parts.evidence_order.get(name).map(Vec::as_slice);
        ordered(&grounds.evidence, order)
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    named_changes(
        &before.commitment_grounds,
        &after.commitment_grounds,
        |name, old, new| old == new && evidence(before, name, old) == evidence(after, name, new),
        |name, grounds| {
            let order = after.evidence_order.get(name).map(Vec::as_slice);
            value(&OrderedGrounds::new(
                ordered(&grounds.evidence, order),
                grounds,
            ))
        },
    )
}

fn named_changes<V>(
    before: &BTreeMap<String, V>,
    after: &BTreeMap<String, V>,
    same: impl Fn(&str, &V, &V) -> bool,
    value: impl Fn(&str, &V) -> Value,
) -> NamedChanges {
    NamedChanges {
        set: after
            .iter()
            .filter(|(name, new)| {
                !before
                    .get(*name)
                    .is_some_and(|known| same(name.as_str(), known, new))
            })
            .map(|(name, new)| (name.clone(), value(name, new)))
            .collect(),
        removed: before
            .keys()
            .filter(|name| !after.contains_key(*name))
            .cloned()
            .collect(),
    }
}

/// The runtime only drops entries from these lists and appends new ones, so
/// the old entries still present are a subsequence of the new list's head.
/// Matching greedily from the front keeps each old entry that is next in the
/// new list and drops the rest; whatever is left of the new list is appended.
/// Applying the result gives the new list whatever the matching chose.
fn list_changes<T>(
    before: &[T],
    after: &[T],
    same: impl Fn(&T, &T) -> bool,
    value: impl Fn(&T) -> Value,
) -> ListChanges {
    let mut kept = 0;
    let mut removed = Vec::new();
    for (position, entry) in before.iter().enumerate() {
        if kept < after.len() && same(entry, &after[kept]) {
            kept += 1;
        } else {
            removed.push(position);
        }
    }
    ListChanges {
        removed,
        appended: after[kept..].iter().map(value).collect(),
    }
}
