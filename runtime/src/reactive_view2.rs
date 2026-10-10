//! View 0.2 and its delta. See spec/caveat-view-0.2.md.
//!
//! The full view orders each commitment's grounds evidence as it was first
//! observed, frozen when the commitment was made, and says `reopened` where
//! View 0.1 says `open`. A delta is built by comparing the full view before
//! and after one accepted event, so applying it gives exactly the next view.
use super::outcome::{DispatchFailure, DispatchFatal, RejectionCode, RejectionOrigin};
use super::{
    BindingValue, Cue, DecisionSeries, EffectReport, JournalEntry, Provenance, ReactiveSession,
    DISPATCH_SCHEMA,
};
use crate::map::{MapCommitment, MapRelation, RetainedCaveat};
use crate::reactive_expr::{departure_markers, Marker};
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub const REACTIVE_VIEW_V2_SCHEMA: &str = "caveat-reactive-view/0.2";
pub const REACTIVE_VIEW_DELTA_SCHEMA: &str = "caveat-reactive-view-delta/0.2";

/// View 0.2: View 0.1 with ordered grounds evidence and `reopened`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReactiveViewV2<'a> {
    pub schema: &'static str,
    pub sequence: u64,
    pub last_event: Option<&'a str>,
    pub bindings: &'a BTreeMap<String, BTreeMap<String, BindingValue>>,
    pub binding_explanations: &'a BTreeMap<String, BTreeMap<String, Provenance>>,
    pub cues: &'a [Cue],
    pub effects: &'a [EffectReport],
    pub commitments: Vec<ViewCommitment>,
    pub commitment_grounds: BTreeMap<&'a str, OrderedGrounds<'a>>,
    pub decision_series: &'a BTreeMap<String, DecisionSeries>,
    pub decision_journal: &'a [JournalEntry],
    pub relations: Vec<MapRelation>,
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

/// A commitment's grounds with evidence in first-observed order. Every other
/// member serializes as `Provenance` does.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OrderedGrounds<'a> {
    pub evidence: Vec<&'a str>,
    pub caveats: &'a BTreeSet<String>,
    #[serde(
        skip_serializing_if = "no_markers",
        serialize_with = "serialize_markers"
    )]
    pub departed: &'a BTreeMap<String, Marker>,
    #[serde(skip_serializing_if = "no_names")]
    pub inherited: &'a BTreeSet<String>,
}

fn no_markers(markers: &&BTreeMap<String, Marker>) -> bool {
    markers.is_empty()
}

fn no_names(names: &&BTreeSet<String>) -> bool {
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
pub(super) fn ordered<'a>(
    evidence: &'a BTreeSet<String>,
    order: Option<&[String]>,
) -> Vec<&'a str> {
    let Some(order) = order else {
        return evidence.iter().map(String::as_str).collect();
    };
    let mut names = order
        .iter()
        .filter_map(|name| evidence.get(name).map(String::as_str))
        .collect::<Vec<_>>();
    if names.len() < evidence.len() {
        let listed = order.iter().map(String::as_str).collect::<BTreeSet<_>>();
        names.extend(
            evidence
                .iter()
                .map(String::as_str)
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
        let names = self
            .symbols
            .iter()
            .map(|(name, id)| (*id, name.as_str()))
            .collect::<HashMap<_, _>>();
        ReactiveViewV2 {
            schema: REACTIVE_VIEW_V2_SCHEMA,
            sequence: self.sequence,
            last_event: self.last_event.as_deref(),
            bindings: &self.bindings,
            binding_explanations: &self.binding_explanations,
            cues: &self.cues,
            effects: &self.effects,
            commitments: self
                .commitment_records(&names)
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
                        OrderedGrounds {
                            evidence: ordered(&grounds.evidence, order),
                            caveats: &grounds.caveats,
                            departed: &grounds.departed,
                            inherited: &grounds.inherited,
                        },
                    )
                })
                .collect(),
            decision_series: &self.decision_series,
            decision_journal: &self.journal,
            relations: self.relation_records(&names),
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
        let since = self.sequence;
        let before = view_value(self);
        match self.dispatch_classified(event, payload_json) {
            Ok(()) => {
                let delta = view_delta(since, &before, &view_value(self));
                Ok(DispatchViewDeltaOutcome {
                    schema: DISPATCH_SCHEMA,
                    result: DispatchViewDeltaResult::Accepted {
                        delta: Box::new(delta),
                    },
                })
            }
            Err(failure) => delta_outcome(failure),
        }
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

fn view_value(session: &ReactiveSession) -> Map<String, Value> {
    match serde_json::to_value(session.view_v2()).expect("finite reactive view") {
        Value::Object(members) => members,
        _ => unreachable!("a view serializes as an object"),
    }
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

fn member<'a>(view: &'a Map<String, Value>, name: &str) -> &'a Value {
    view.get(name).unwrap_or(&Value::Null)
}

fn object(value: &Value) -> Option<&Map<String, Value>> {
    value.as_object()
}

fn array(value: &Value) -> &[Value] {
    value.as_array().map(Vec::as_slice).unwrap_or_default()
}

fn view_delta(since: u64, before: &Map<String, Value>, after: &Map<String, Value>) -> ViewDelta {
    ViewDelta {
        schema: REACTIVE_VIEW_DELTA_SCHEMA,
        since,
        sequence: member(after, "sequence").as_u64().unwrap_or_default(),
        last_event: member(after, "last_event").clone(),
        bindings: property_changes(member(before, "bindings"), member(after, "bindings")),
        binding_explanations: property_changes(
            member(before, "binding_explanations"),
            member(after, "binding_explanations"),
        ),
        cues: member(after, "cues").clone(),
        effects: member(after, "effects").clone(),
        commitments: commitment_changes(
            member(before, "commitments"),
            member(after, "commitments"),
        ),
        commitment_grounds: named_changes(
            member(before, "commitment_grounds"),
            member(after, "commitment_grounds"),
        ),
        decision_series: named_changes(
            member(before, "decision_series"),
            member(after, "decision_series"),
        ),
        decision_journal: list_changes(
            array(member(before, "decision_journal")),
            array(member(after, "decision_journal")),
        ),
        relations: list_changes(
            array(member(before, "relations")),
            array(member(after, "relations")),
        ),
    }
}

fn property_changes(before: &Value, after: &Value) -> PropertyChanges {
    let empty = Map::new();
    let (before, after) = (
        object(before).unwrap_or(&empty),
        object(after).unwrap_or(&empty),
    );
    let mut changes = PropertyChanges::default();
    for (target, properties) in after {
        let old = before.get(target).and_then(object).unwrap_or(&empty);
        let mut set = Map::new();
        for (property, value) in object(properties).unwrap_or(&empty) {
            if !old
                .get(property)
                .is_some_and(|known| identical(known, value))
            {
                set.insert(property.clone(), value.clone());
            }
        }
        if !set.is_empty() {
            changes.set.insert(target.clone(), Value::Object(set));
        }
    }
    for (target, properties) in before {
        let new = after.get(target).and_then(object);
        for property in object(properties).unwrap_or(&empty).keys() {
            if !new.is_some_and(|new| new.contains_key(property)) {
                changes.removed.push([target.clone(), property.clone()]);
            }
        }
    }
    changes
}

fn commitment_changes(before: &Value, after: &Value) -> CommitmentChanges {
    let action = |commitment: &Value| {
        commitment
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let old = array(before)
        .iter()
        .map(|commitment| (action(commitment), commitment))
        .collect::<BTreeMap<_, _>>();
    let new = array(after)
        .iter()
        .map(|commitment| (action(commitment), commitment))
        .collect::<BTreeMap<_, _>>();
    CommitmentChanges {
        set: new
            .iter()
            .filter(|(name, commitment)| {
                !old.get(*name)
                    .is_some_and(|known| identical(known, commitment))
            })
            .map(|(_, commitment)| (*commitment).clone())
            .collect(),
        removed: old
            .keys()
            .filter(|name| !new.contains_key(*name))
            .cloned()
            .collect(),
    }
}

fn named_changes(before: &Value, after: &Value) -> NamedChanges {
    let empty = Map::new();
    let (before, after) = (
        object(before).unwrap_or(&empty),
        object(after).unwrap_or(&empty),
    );
    NamedChanges {
        set: after
            .iter()
            .filter(|(name, value)| {
                !before
                    .get(*name)
                    .is_some_and(|known| identical(known, value))
            })
            .map(|(name, value)| (name.clone(), value.clone()))
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
fn list_changes(before: &[Value], after: &[Value]) -> ListChanges {
    let mut kept = 0;
    let mut removed = Vec::new();
    for (position, entry) in before.iter().enumerate() {
        if kept < after.len() && identical(entry, &after[kept]) {
            kept += 1;
        } else {
            removed.push(position);
        }
    }
    ListChanges {
        removed,
        appended: after[kept..].to_vec(),
    }
}
