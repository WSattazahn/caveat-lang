//! A program's interface: what a host can send it and what it can show, read
//! from the loaded program. See spec/caveat-interface-0.1.md. Nothing is
//! evaluated beyond what loading the program evaluates.

use super::{ParameterDomain, ReactiveSession};
use crate::reactive_expr::parse_unresolved;
use crate::NodeKind;
use serde::Serialize;
use std::collections::BTreeMap;

pub const INTERFACE_SCHEMA: &str = "caveat-interface/0.1";

/// Every list is in name order, except an event's parameters, which keep
/// their declared order.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProgramInterface {
    pub schema: &'static str,
    pub events: Vec<InterfaceEvent>,
    pub states: Vec<InterfaceState>,
    pub bindings: Vec<InterfaceBinding>,
    pub cues: Vec<InterfaceCue>,
    pub decisions: Vec<InterfaceDecisions>,
    pub readings: Vec<InterfaceReadings>,
    pub evidence: Vec<String>,
    pub caveats: Vec<String>,
    pub claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InterfaceEvent {
    pub name: String,
    pub parameters: Vec<InterfaceParameter>,
}

/// One payload field. `type` is `number`, `entity`, `member` or `id`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InterfaceParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// The entity kind, for an `entity` parameter.
    #[serde(rename = "kind", skip_serializing_if = "Option::is_none")]
    pub entity_kind: Option<String>,
    /// The names it accepts, in position order, for `entity` and `member`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InterfaceState {
    pub name: String,
    pub min: f64,
    pub max: f64,
}

/// A bound property. `type` is `number`, `boolean` or `text`. `always` is
/// true when a declaration has no condition, so every view shows it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InterfaceBinding {
    pub target: String,
    pub property: String,
    #[serde(rename = "type")]
    pub value_type: &'static str,
    pub always: bool,
}

/// A declared cue. `kind` is `sound`, `toast`, `flash` or `ring`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InterfaceCue {
    pub name: String,
    pub kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InterfaceDecisions {
    pub name: String,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InterfaceReadings {
    pub name: String,
    pub evidence: String,
    pub limit: usize,
}

/// Loads the program, as `ReactiveSession::from_source` does, and returns its
/// interface. A program that does not load is that load error.
pub fn interface_source(source: &str) -> Result<ProgramInterface, String> {
    Ok(ReactiveSession::from_source(source)?.interface())
}

impl ReactiveSession {
    /// The interface of the program this session runs. It depends only on the
    /// program, so it is the same after any events.
    pub fn interface(&self) -> ProgramInterface {
        let events = self
            .events
            .iter()
            .map(|(name, parameters)| InterfaceEvent {
                name: name.clone(),
                parameters: parameters
                    .iter()
                    .map(|parameter| {
                        let (kind, entity_kind, members) = match &parameter.domain {
                            ParameterDomain::Numeric => ("number", None, None),
                            ParameterDomain::Entity { kind, members } => {
                                ("entity", Some(kind.clone()), Some(members.clone()))
                            }
                            ParameterDomain::Member { members } => {
                                ("member", None, Some(members.clone()))
                            }
                            ParameterDomain::Identifier { .. } => ("id", None, None),
                        };
                        let numeric = kind == "number";
                        InterfaceParameter {
                            name: parameter.name.clone(),
                            kind,
                            min: numeric.then(|| parameter.min.value()),
                            max: numeric.then(|| parameter.max.value()),
                            entity_kind,
                            members,
                        }
                    })
                    .collect(),
            })
            .collect();

        let mut states = self
            .states
            .names()
            .map(|name| InterfaceState {
                name: name.clone(),
                min: self.ranges[name].min,
                max: self.ranges[name].max,
            })
            .collect::<Vec<_>>();
        states.sort_by(|a, b| a.name.cmp(&b.name));

        // A declaration without `when` has the condition `true`.
        let unconditional = parse_unresolved("true").expect("the literal true parses");
        let mut bindings = self
            .binding_groups
            .iter()
            .enumerate()
            .map(|(group, binding)| InterfaceBinding {
                target: binding.target.clone(),
                property: binding.property.clone(),
                value_type: binding.value_type,
                always: self
                    .binding_rules
                    .iter()
                    .zip(self.binding_group_of.iter())
                    .any(|(rule, of)| *of == group && rule.condition == unconditional),
            })
            .collect::<Vec<_>>();
        bindings.sort_by(|a, b| (&a.target, &a.property).cmp(&(&b.target, &b.property)));

        let cues = self
            .cue_definitions
            .iter()
            .map(|(name, cue)| InterfaceCue {
                name: name.clone(),
                kind: match cue {
                    super::Cue::Sound { .. } => "sound",
                    super::Cue::Toast { .. } => "toast",
                    super::Cue::Flash { .. } => "flash",
                    super::Cue::Ring { .. } => "ring",
                },
            })
            .collect();

        let decisions = self
            .decision_series
            .iter()
            .map(|(name, series)| InterfaceDecisions {
                name: name.clone(),
                limit: series.limit,
            })
            .collect();
        let readings = self
            .reading_streams
            .iter()
            .map(|(name, stream)| InterfaceReadings {
                name: name.clone(),
                evidence: stream.template.clone(),
                limit: stream.limit,
            })
            .collect();

        // Names the program declared; events add occurrences after these.
        let mut symbols = BTreeMap::<&str, Vec<String>>::new();
        for (name, id) in self.symbols.iter() {
            if *id > self.loaded.last_node {
                continue;
            }
            let kind = match self.graph.nodes.get(id) {
                Some(NodeKind::Evidence { .. }) => "evidence",
                Some(NodeKind::Caveat { .. }) => "caveat",
                Some(NodeKind::Claim { .. }) => "claim",
                _ => continue,
            };
            symbols.entry(kind).or_default().push(name.clone());
        }
        let mut names = |kind| {
            let mut names = symbols.remove(kind).unwrap_or_default();
            names.sort();
            names
        };

        ProgramInterface {
            schema: INTERFACE_SCHEMA,
            events,
            states,
            bindings,
            cues,
            decisions,
            readings,
            evidence: names("evidence"),
            caveats: names("caveat"),
            claims: names("claim"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_without_when_is_always_shown() {
        let interface = interface_source(
            "state level = 0 min 0 max 10;\n\
             event poke;\n\
             bind gauge.value = level;\n\
             bind gauge.alarm = \"high\" when level > 5;\n",
        )
        .unwrap();
        let always = interface
            .bindings
            .iter()
            .map(|binding| (binding.property.as_str(), binding.always))
            .collect::<Vec<_>>();
        assert_eq!(always, [("alarm", false), ("value", true)]);
    }
}
