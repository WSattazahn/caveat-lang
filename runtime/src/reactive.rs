//! Bounded, deterministic event programs over the CAVEAT epistemic graph.
//!
//! The host dispatches declared inputs. Source owns numeric updates and graph
//! effects; an entire event publishes atomically or leaves the session intact.

use crate::ast::{Program, Statement};
use crate::eval::ResourceLedger;
use crate::game_session::GameSymbol;
use crate::map::{CaveatMap, MapBudget, MapCommitment, MapRelation, MapWorld};
use crate::presentation::Number;
use crate::reactive_expr::{self, Expr, FunctionDef, Value, ValueType};
pub use crate::reactive_expr::{Provenance, Tracked};
use crate::{Attention, EpistemicGraph, NodeId, NodeKind, Relation, StopReason};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

const LIMIT: f64 = 1_000_000_000_000.0;
pub const REACTIVE_SCHEMA: &str = "caveat-reactive/0.1";
pub const REACTIVE_PRELUDE_SOURCE: &str = include_str!("../prelude.cav");

/// Compile the standard library through the same parser and function checker
/// as application source. Nothing in the host implements these algorithms.
pub(crate) fn prelude_functions() -> Result<BTreeMap<String, FunctionDef>, String> {
    source_functions(REACTIVE_PRELUDE_SOURCE)
}

fn source_functions(source: &str) -> Result<BTreeMap<String, FunctionDef>, String> {
    let mut functions = BTreeMap::new();
    for statement in crate::parser::parse(source)?.statements {
        let Statement::Reactive(Directive::Function(function)) = statement else {
            return Err("reactive prelude may contain only pure function definitions".into());
        };
        let name = function.name.clone();
        if functions.insert(name.clone(), function).is_some() {
            return Err(format!("duplicate prelude function {name}"));
        }
    }
    reactive_expr::validate_functions(&functions)?;
    Ok(functions)
}

#[cfg(test)]
mod prelude_tests {
    use super::*;

    #[test]
    fn changing_only_prelude_source_changes_numeric_and_formatting_algorithms() {
        let evaluate = |source: &str, expression: &str| {
            let functions = source_functions(source).unwrap();
            let expression = reactive_expr::expand(
                &reactive_expr::parse_unresolved(expression).unwrap(),
                &functions,
            )
            .unwrap();
            expression
                .evaluate(&|_| None, &|_, _| Err("pure library has no graph".into()))
                .unwrap()
        };
        let alternative = REACTIVE_PRELUDE_SOURCE
            .replace(
                "fn abs(value) = if(value < 0, -value, value + 0);",
                "fn abs(value) = value * value;",
            )
            .replace("text(round(value))", "text(floor(value))");
        assert_eq!(
            evaluate(REACTIVE_PRELUDE_SOURCE, "abs(-8)"),
            Value::Number(8.0)
        );
        assert_eq!(evaluate(&alternative, "abs(-8)"), Value::Number(64.0));
        assert_eq!(
            evaluate(REACTIVE_PRELUDE_SOURCE, "number_text(12.6)"),
            Value::Text("13".into())
        );
        assert_eq!(
            evaluate(&alternative, "number_text(12.6)"),
            Value::Text("12".into())
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Parameter {
    pub name: String,
    pub min: Number,
    pub max: Number,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Emit {
        name: String,
    },
    Set {
        name: String,
        value: Expr,
    },
    Reveal {
        evidence: String,
        relation: Relation,
        claim: String,
    },
    Examine {
        caveat: String,
        cost: u64,
    },
    Commit {
        action: String,
        reason: StopReason,
        using: Option<Expr>,
        retaining: Vec<String>,
    },
    Reopen {
        action: String,
        because: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub event: String,
    pub condition: Expr,
    pub effect: Effect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    Function(FunctionDef),
    Binding(Binding),
    Cue(Cue),
    Control {
        name: String,
        control: Control,
    },
    Clock(Clock),
    State {
        name: String,
        initial: Expr,
        min: Number,
        max: Number,
    },
    Event {
        name: String,
        parameters: Vec<Parameter>,
    },
    Rule(Rule),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingExpression {
    Expression(Expr),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub target: String,
    pub property: String,
    pub value: BindingExpression,
    pub condition: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum BindingValue {
    Number(f64),
    Bool(bool),
    Text(String),
}

impl BindingValue {
    pub fn as_number(&self) -> Option<f64> {
        if let Self::Number(value) = self {
            Some(*value)
        } else {
            None
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        if let Self::Text(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum CueColor {
    Number(u32),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Cue {
    Sound {
        id: String,
        frequency: Number,
        duration: Number,
        gain: Number,
    },
    Toast {
        id: String,
        text: String,
        duration: Number,
    },
    Flash {
        id: String,
        target: String,
        duration: Number,
    },
    Ring {
        id: String,
        target: String,
        color: CueColor,
        duration: Number,
    },
}

impl Cue {
    pub fn id(&self) -> &str {
        match self {
            Self::Sound { id, .. }
            | Self::Toast { id, .. }
            | Self::Flash { id, .. }
            | Self::Ring { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Control {
    pub event: String,
    pub reset: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Clock {
    pub event: String,
    pub step: Number,
}

pub type QualifiedValue = Tracked<f64>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CommitmentBasis {
    pub value: Option<f64>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone)]
struct StateRange {
    min: f64,
    max: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EffectReport {
    Reveal {
        evidence: String,
        relation: String,
        target: String,
    },
    Examine {
        caveat: String,
        cost: u64,
    },
    Commit {
        action: String,
        retained: Vec<String>,
    },
    Reopen {
        action: String,
        because: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReactiveSnapshot {
    pub schema: String,
    pub source_id: String,
    pub sequence: u64,
    pub last_event: Option<String>,
    pub values: BTreeMap<String, f64>,
    pub qualified_values: BTreeMap<String, QualifiedValue>,
    pub commitment_bases: BTreeMap<String, CommitmentBasis>,
    pub observation_qualifications: BTreeMap<String, Provenance>,
    pub examination_qualifications: BTreeMap<String, Provenance>,
    pub reopening_qualifications: BTreeMap<String, Provenance>,
    pub predicate_qualifications: BTreeMap<String, BTreeMap<String, Provenance>>,
    pub events: Vec<EventSignature>,
    pub bindings: BTreeMap<String, BTreeMap<String, BindingValue>>,
    pub binding_qualifications: BTreeMap<String, BTreeMap<String, Provenance>>,
    pub cues: Vec<Cue>,
    pub cue_qualifications: Vec<Provenance>,
    pub controls: BTreeMap<String, Control>,
    pub clock: Option<Clock>,
    pub world: MapWorld,
    pub scenes: Vec<String>,
    pub labels: BTreeMap<String, String>,
    pub symbols: Vec<GameSymbol>,
    pub relations: Vec<MapRelation>,
    pub commitments: Vec<MapCommitment>,
    pub budget: Option<MapBudget>,
    pub effects: Vec<EffectReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub struct ReactiveSession {
    source_id: String,
    sequence: u64,
    last_event: Option<String>,
    values: BTreeMap<String, QualifiedValue>,
    commitment_bases: BTreeMap<String, CommitmentBasis>,
    observation_qualifications: BTreeMap<String, Provenance>,
    examination_qualifications: BTreeMap<String, Provenance>,
    reopening_qualifications: BTreeMap<String, Provenance>,
    predicate_qualifications: BTreeMap<String, BTreeMap<String, Provenance>>,
    ranges: BTreeMap<String, StateRange>,
    constants: BTreeMap<String, f64>,
    events: BTreeMap<String, Vec<Parameter>>,
    rules: Arc<Vec<Rule>>,
    binding_rules: Arc<Vec<Binding>>,
    bindings: BTreeMap<String, BTreeMap<String, BindingValue>>,
    binding_qualifications: BTreeMap<String, BTreeMap<String, Provenance>>,
    cue_definitions: Arc<BTreeMap<String, Cue>>,
    cues: Vec<Cue>,
    cue_qualifications: Vec<Provenance>,
    controls: BTreeMap<String, Control>,
    clock: Option<Clock>,
    graph: EpistemicGraph,
    symbols: HashMap<String, NodeId>,
    resources: Option<ResourceLedger>,
    world: MapWorld,
    labels: BTreeMap<String, String>,
    scenes: Vec<String>,
    effects: Vec<EffectReport>,
}

impl ReactiveSession {
    pub fn from_source(source: &str) -> Result<Self, String> {
        let program = crate::parser::parse(source)?;
        let mut declarations = Vec::new();
        let mut directives = Vec::new();
        for statement in &program.statements {
            match statement {
                Statement::Reactive(directive) => directives.push(directive.clone()),
                Statement::Scene { .. }
                | Statement::Display { .. }
                | Statement::Budget { .. }
                | Statement::Claim { .. }
                | Statement::Evidence { .. }
                | Statement::Caveat { .. }
                | Statement::Relate { .. }
                | Statement::Attention { .. }
                | Statement::Examine { .. }
                | Statement::Commit { .. }
                | Statement::Reopen { .. }
                | Statement::Place { .. }
                | Statement::Entity { .. }
                | Statement::Connect { .. }
                | Statement::StartAt { .. }
                | Statement::Presentation(_) => declarations.push(statement.clone()),
                _ => {
                    return Err(
                        "reactive programs cannot contain sequential choice/action statements"
                            .into(),
                    )
                }
            }
        }
        let mut functions = prelude_functions()?;
        for directive in &directives {
            if let Directive::Function(function) = directive {
                if functions
                    .insert(function.name.clone(), function.clone())
                    .is_some()
                {
                    return Err(format!(
                        "duplicate reactive function {} (standard library names are reserved)",
                        function.name
                    ));
                }
            }
        }
        reactive_expr::validate_functions(&functions)?;
        for directive in &mut directives {
            match directive {
                Directive::State { initial, .. } => {
                    *initial = reactive_expr::expand(initial, &functions)?
                }
                Directive::Rule(rule) => {
                    rule.condition = reactive_expr::expand(&rule.condition, &functions)?;
                    if let Effect::Set { value, .. } = &mut rule.effect {
                        *value = reactive_expr::expand(value, &functions)?;
                    }
                    if let Effect::Commit {
                        using: Some(value), ..
                    } = &mut rule.effect
                    {
                        *value = reactive_expr::expand(value, &functions)?;
                    }
                }
                Directive::Binding(binding) => {
                    binding.condition = reactive_expr::expand(&binding.condition, &functions)?;
                    if let BindingExpression::Expression(value) = &mut binding.value {
                        *value = reactive_expr::expand(value, &functions)?;
                    }
                }
                _ => {}
            }
        }
        let declarations = Program::new(declarations);
        let map = CaveatMap::build(&declarations)?;
        let evaluation = crate::eval::evaluate(&declarations)?;
        let mut constants = BTreeMap::new();
        for position in &map.world.presentation.positions {
            for (axis, coordinate) in ["x", "y", "z"].into_iter().zip(&position.position) {
                constants.insert(format!("{}.{axis}", position.target), coordinate.value());
            }
        }
        let mut session = Self {
            source_id: source_identity(source),
            sequence: 0,
            last_event: None,
            values: BTreeMap::new(),
            commitment_bases: BTreeMap::new(),
            observation_qualifications: BTreeMap::new(),
            examination_qualifications: BTreeMap::new(),
            reopening_qualifications: BTreeMap::new(),
            predicate_qualifications: BTreeMap::new(),
            ranges: BTreeMap::new(),
            constants,
            events: BTreeMap::new(),
            rules: Arc::new(Vec::new()),
            binding_rules: Arc::new(Vec::new()),
            bindings: BTreeMap::new(),
            binding_qualifications: BTreeMap::new(),
            cue_definitions: Arc::new(BTreeMap::new()),
            cues: Vec::new(),
            cue_qualifications: Vec::new(),
            controls: BTreeMap::new(),
            clock: None,
            graph: evaluation.graph,
            symbols: evaluation.symbols,
            resources: evaluation.resources,
            world: map.world,
            labels: evaluation.display.into_iter().collect(),
            scenes: map.scenes,
            effects: Vec::new(),
        };
        for directive in &directives {
            match directive {
                Directive::Function(_) => {}
                Directive::Binding(binding) => {
                    Arc::make_mut(&mut session.binding_rules).push(binding.clone())
                }
                Directive::Cue(cue) => {
                    if Arc::make_mut(&mut session.cue_definitions)
                        .insert(cue.id().into(), cue.clone())
                        .is_some()
                    {
                        return Err(format!("duplicate cue {}", cue.id()));
                    }
                    if let Cue::Ring { target, .. } = cue {
                        if !session
                            .world
                            .entities
                            .iter()
                            .any(|entity| entity.id == *target)
                        {
                            return Err(format!(
                                "ring cue {} references unknown entity {target}",
                                cue.id()
                            ));
                        }
                    }
                }
                Directive::Control { name, control } => {
                    if session
                        .controls
                        .insert(name.clone(), control.clone())
                        .is_some()
                    {
                        return Err(format!("duplicate control {name}"));
                    }
                }
                Directive::Clock(clock) => {
                    if session.clock.replace(clock.clone()).is_some() {
                        return Err("reactive program can declare only one clock".into());
                    }
                }
                Directive::State {
                    name,
                    initial,
                    min,
                    max,
                } => {
                    if session.values.contains_key(name) {
                        return Err(format!("duplicate reactive state {name}"));
                    }
                    let numeric = session
                        .values
                        .keys()
                        .chain(session.constants.keys())
                        .cloned()
                        .collect();
                    if initial.validate(&numeric, &|kind, name| match kind {
                        "qualification_evidence" => session.require_kind(name, "evidence"),
                        "qualification_caveat" => session.require_kind(name, "caveat"),
                        _ => Err("state initializers cannot query live graph predicates".into()),
                    })? != ValueType::Number
                    {
                        return Err(format!("state {name} initializer must be numeric"));
                    }
                    let value = initial.evaluate_tracked(
                        &|name| {
                            Ok(session.values.get(name).cloned().or_else(|| {
                                session.constants.get(name).copied().map(Tracked::plain)
                            }))
                        },
                        &|_, _| Err("state initializer cannot query graph".into()),
                        &|evidence, caveats| session.qualify(evidence, caveats),
                    )?;
                    let Value::Number(number) = value.value else {
                        unreachable!("validated number initializer")
                    };
                    check_range(name, number, min.value(), max.value())?;
                    session
                        .values
                        .insert(name.clone(), Tracked::new(number, value.provenance)?);
                    session.ranges.insert(
                        name.clone(),
                        StateRange {
                            min: min.value(),
                            max: max.value(),
                        },
                    );
                }
                Directive::Event { name, parameters } => {
                    if session
                        .events
                        .insert(name.clone(), parameters.clone())
                        .is_some()
                    {
                        return Err(format!("duplicate reactive event {name}"));
                    }
                    let mut names = HashSet::new();
                    for parameter in parameters {
                        if !names.insert(&parameter.name) {
                            return Err(format!(
                                "duplicate parameter {} for event {name}",
                                parameter.name
                            ));
                        }
                    }
                    if name == "tick"
                        && (parameters.len() != 1
                            || parameters[0].name != "dt"
                            || parameters[0].min.value() < 0.0
                            || parameters[0].max.value() > 0.1)
                    {
                        return Err(
                            "tick must declare only dt with bounds inside 0..0.1 seconds".into(),
                        );
                    }
                }
                Directive::Rule(rule) => Arc::make_mut(&mut session.rules).push(rule.clone()),
            }
        }
        if session.events.is_empty() {
            return Err("reactive program must declare an event".into());
        }
        for (event, parameters) in &session.events {
            for parameter in parameters {
                if session.values.contains_key(&parameter.name)
                    || session.constants.contains_key(&parameter.name)
                {
                    return Err(format!(
                        "event {event} parameter {} shadows state or a coordinate",
                        parameter.name
                    ));
                }
            }
        }
        for (name, control) in &session.controls {
            if !session.events.contains_key(&control.event) {
                return Err(format!(
                    "control {name} references undeclared event {}",
                    control.event
                ));
            }
        }
        if let Some(clock) = &session.clock {
            let parameters = session
                .events
                .get(&clock.event)
                .ok_or_else(|| format!("clock references undeclared event {}", clock.event))?;
            if parameters.len() != 1 || parameters[0].name != "dt" {
                return Err("clock event must declare only a dt parameter".into());
            }
            if clock.step.value() <= 0.0 {
                return Err("clock step must be positive".into());
            }
            check_range(
                "clock step",
                clock.step.value(),
                parameters[0].min.value(),
                parameters[0].max.value(),
            )?;
        }
        session.validate_rules()?;
        session.evaluate_bindings()?;
        Ok(session)
    }

    fn validate_rules(&self) -> Result<(), String> {
        let mut commitments = self
            .symbols
            .iter()
            .filter_map(|(name, id)| {
                matches!(self.graph.nodes.get(id), Some(NodeKind::Commitment { .. }))
                    .then_some(name.clone())
            })
            .collect::<HashSet<_>>();
        for rule in self.rules.iter() {
            if let Effect::Commit { action, .. } = &rule.effect {
                if self.symbols.get(action).is_some_and(|id| {
                    !matches!(self.graph.nodes.get(id), Some(NodeKind::Commitment { .. }))
                }) {
                    return Err(format!(
                        "reactive commitment {action} conflicts with a declared symbol"
                    ));
                }
                commitments.insert(action.clone());
            }
        }
        let validate_predicate = |kind: &str, symbol: &str| -> Result<(), String> {
            match kind {
                "observed" => self.require_kind(symbol, "evidence"),
                "examined" => self.require_kind(symbol, "caveat"),
                "qualification_evidence" => self.require_kind(symbol, "evidence"),
                "qualification_caveat" => self.require_kind(symbol, "caveat"),
                "committed" | "reopened" if commitments.contains(symbol) => Ok(()),
                "committed" | "reopened" => Err(format!("unknown reactive commitment {symbol}")),
                _ => Err(format!("unknown epistemic predicate {kind}")),
            }
        };
        for (index, rule) in self.rules.iter().enumerate() {
            let parameters = self.events.get(&rule.event).ok_or_else(|| {
                format!(
                    "rule {} references undeclared event {}",
                    index + 1,
                    rule.event
                )
            })?;
            let numeric = self
                .values
                .keys()
                .chain(self.constants.keys())
                .cloned()
                .chain(parameters.iter().map(|parameter| parameter.name.clone()))
                .collect();
            if rule.condition.validate(&numeric, &validate_predicate)? != ValueType::Bool {
                return Err(format!("rule {} condition must be boolean", index + 1));
            }
            match &rule.effect {
                Effect::Emit { name } => {
                    if !self.cue_definitions.contains_key(name) {
                        return Err(format!("emit references undeclared cue {name}"));
                    }
                }
                Effect::Set { name, value } => {
                    if !self.values.contains_key(name) {
                        return Err(format!("set references undeclared state {name}"));
                    }
                    if value.validate(&numeric, &validate_predicate)? != ValueType::Number {
                        return Err(format!("state {name} requires a numeric expression"));
                    }
                }
                Effect::Reveal {
                    evidence, claim, ..
                } => {
                    self.require_kind(evidence, "evidence")?;
                    self.require_kind(claim, "claim")?;
                }
                Effect::Examine { caveat, .. } => {
                    self.require_kind(caveat, "caveat")?;
                    if self.resources.is_none() {
                        return Err("reactive examination requires an attention budget".into());
                    }
                }
                Effect::Commit {
                    using, retaining, ..
                } => {
                    if let Some(value) = using {
                        if value.validate(&numeric, &validate_predicate)? != ValueType::Number {
                            return Err("commit using requires a numeric expression".into());
                        }
                    }
                    let mut retained = HashSet::new();
                    for caveat in retaining {
                        self.require_kind(caveat, "caveat")?;
                        if !retained.insert(caveat) {
                            return Err(format!("duplicate retained caveat {caveat}"));
                        }
                    }
                }
                Effect::Reopen { action, because } => {
                    if !commitments.contains(action) {
                        return Err(format!("unknown reactive commitment {action}"));
                    }
                    self.require_kind(because, "evidence")?;
                }
            }
        }
        let numeric = self
            .values
            .keys()
            .chain(self.constants.keys())
            .cloned()
            .collect();
        let mut types = BTreeMap::new();
        for binding in self.binding_rules.iter() {
            if binding.condition.validate(&numeric, &validate_predicate)? != ValueType::Bool {
                return Err(format!(
                    "binding {}.{} condition must be boolean",
                    binding.target, binding.property
                ));
            }
            let kind = match &binding.value {
                BindingExpression::Text(_) => "text",
                BindingExpression::Expression(value) => {
                    match value.validate(&numeric, &validate_predicate)? {
                        ValueType::Number => "number",
                        ValueType::Bool => "boolean",
                        ValueType::Text => "text",
                    }
                }
            };
            if let Some(previous) = types.insert((&binding.target, &binding.property), kind) {
                if previous != kind {
                    return Err(format!(
                        "binding {}.{} changes type from {previous} to {kind}",
                        binding.target, binding.property
                    ));
                }
            }
        }
        Ok(())
    }

    fn evaluate_bindings(&mut self) -> Result<(), String> {
        let mut bindings = BTreeMap::<String, BTreeMap<String, BindingValue>>::new();
        let mut data = BTreeMap::<String, BTreeMap<String, Provenance>>::new();
        let mut guards = BTreeMap::<String, BTreeMap<String, Provenance>>::new();
        let parameters = BTreeMap::new();
        for binding in self.binding_rules.iter() {
            let context = || format!("binding {}.{}", binding.target, binding.property);
            let condition = self
                .evaluate(&binding.condition, &parameters)
                .map_err(|error| format!("{}: {error}", context()))?;
            guards
                .entry(binding.target.clone())
                .or_default()
                .entry(binding.property.clone())
                .or_default()
                .merge(&condition.provenance)?;
            if condition.value != Value::Bool(true) {
                continue;
            }
            let value = match &binding.value {
                BindingExpression::Text(value) => Tracked::plain(BindingValue::Text(value.clone())),
                BindingExpression::Expression(expression) => {
                    let value = self
                        .evaluate(expression, &parameters)
                        .map_err(|error| format!("{}: {error}", context()))?;
                    let primitive = match value.value {
                        Value::Number(value) => BindingValue::Number(value),
                        Value::Bool(value) => BindingValue::Bool(value),
                        Value::Text(value) => BindingValue::Text(value),
                    };
                    Tracked::new(primitive, value.provenance)?
                }
            };
            bindings
                .entry(binding.target.clone())
                .or_default()
                .insert(binding.property.clone(), value.value);
            data.entry(binding.target.clone())
                .or_default()
                .insert(binding.property.clone(), value.provenance);
        }
        for (target, properties) in guards {
            for (property, provenance) in properties {
                data.entry(target.clone())
                    .or_default()
                    .entry(property)
                    .or_default()
                    .merge(&provenance)?;
            }
        }
        self.bindings = bindings;
        self.binding_qualifications = data;
        Ok(())
    }

    fn require_kind(&self, name: &str, kind: &str) -> Result<(), String> {
        let node = self
            .symbols
            .get(name)
            .and_then(|id| self.graph.nodes.get(id));
        if matches!(
            (kind, node),
            ("evidence", Some(NodeKind::Evidence { .. }))
                | ("caveat", Some(NodeKind::Caveat { .. }))
                | ("claim", Some(NodeKind::Claim { .. }))
        ) {
            Ok(())
        } else {
            Err(format!("{name} must name a declared {kind}"))
        }
    }

    pub fn dispatch_json(
        &mut self,
        event: &str,
        payload_json: &str,
    ) -> Result<ReactiveSnapshot, String> {
        // Typed map parsing rejects duplicate fields instead of last-write wins.
        let payload: NumericPayload = serde_json::from_str(payload_json)
            .map_err(|error| format!("invalid event payload: {error}"))?;
        self.dispatch(event, &payload.0)
    }

    pub fn dispatch(
        &mut self,
        event: &str,
        parameters: &BTreeMap<String, f64>,
    ) -> Result<ReactiveSnapshot, String> {
        let signature = self
            .events
            .get(event)
            .ok_or_else(|| format!("undeclared event {event}"))?;
        if signature.len() != parameters.len()
            || parameters
                .keys()
                .any(|name| !signature.iter().any(|parameter| &parameter.name == name))
        {
            return Err(format!(
                "event {event} requires exactly its declared parameters"
            ));
        }
        for parameter in signature {
            let value = parameters
                .get(&parameter.name)
                .ok_or_else(|| format!("missing event parameter {}", parameter.name))?;
            check_range(
                &parameter.name,
                *value,
                parameter.min.value(),
                parameter.max.value(),
            )?;
        }
        let mut next = self.clone();
        next.effects.clear();
        next.cues.clear();
        next.cue_qualifications.clear();
        for (index, rule) in self
            .rules
            .iter()
            .enumerate()
            .filter(|(_, rule)| rule.event == event)
        {
            let result = (|| {
                let condition = next.evaluate(&rule.condition, parameters)?;
                if condition.value == Value::Bool(true) {
                    next.apply_effect(&rule.effect, parameters, &condition.provenance)?;
                } else if let Effect::Set { name, .. } = &rule.effect {
                    // Retaining a value after a failed qualified guard is itself
                    // dependent on the guard; do not silently discard that basis.
                    next.values
                        .get_mut(name)
                        .expect("validated state")
                        .provenance
                        .merge(&condition.provenance)?;
                } else {
                    let target = match &rule.effect {
                        Effect::Reveal { evidence, .. } => Some(("observed", evidence)),
                        Effect::Examine { caveat, .. } => Some(("examined", caveat)),
                        Effect::Commit { action, .. } => Some(("committed", action)),
                        Effect::Reopen { action, .. } => Some(("reopened", action)),
                        _ => None,
                    };
                    if let Some((kind, name)) = target {
                        if !condition.provenance.is_empty() {
                            next.predicate_qualifications
                                .entry(kind.into())
                                .or_default()
                                .entry(name.clone())
                                .or_default()
                                .merge(&condition.provenance)?;
                        }
                    }
                }
                Ok::<_, String>(())
            })();
            result.map_err(|error| format!("event {event}, rule {}: {error}", index + 1))?;
        }
        next.sequence = next
            .sequence
            .checked_add(1)
            .ok_or("reactive event sequence exhausted")?;
        next.last_event = Some(event.into());
        // Binding failures roll back the same numeric/graph/cue transaction.
        next.evaluate_bindings()?;
        let snapshot = next.snapshot();
        *self = next;
        Ok(snapshot)
    }

    fn evaluate(
        &self,
        expression: &Expr,
        parameters: &BTreeMap<String, f64>,
    ) -> Result<Tracked<Value>, String> {
        expression.evaluate_tracked(
            &|name| {
                Ok(self.values.get(name).cloned().or_else(|| {
                    parameters
                        .get(name)
                        .or_else(|| self.constants.get(name))
                        .copied()
                        .map(Tracked::plain)
                }))
            },
            &|kind, name| self.predicate_tracked(kind, name),
            &|evidence, caveats| self.qualify(evidence, caveats),
        )
    }

    fn predicate(&self, kind: &str, name: &str) -> Result<bool, String> {
        let Some(id) = self.symbols.get(name) else {
            return if matches!(kind, "committed" | "reopened") {
                Ok(false)
            } else {
                Err(format!("unknown graph symbol {name}"))
            };
        };
        Ok(match kind {
            "observed" => {
                matches!(self.graph.nodes.get(id), Some(NodeKind::Evidence { .. }))
                    && self.graph.edges.iter().any(|edge| {
                        edge.from == *id
                            && matches!(edge.relation, Relation::Supports | Relation::Opposes)
                    })
            }
            "examined" => matches!(
                self.graph.nodes.get(id),
                Some(NodeKind::Caveat {
                    attention: Attention::Examined,
                    ..
                })
            ),
            "committed" => matches!(self.graph.nodes.get(id), Some(NodeKind::Commitment { .. })),
            "reopened" => matches!(
                self.graph.nodes.get(id),
                Some(NodeKind::Commitment { open: true, .. })
            ),
            _ => return Err(format!("unknown graph predicate {kind}")),
        })
    }

    fn incoming_caveats(&self, roots: impl IntoIterator<Item = NodeId>) -> Vec<String> {
        let mut frontier = roots.into_iter().collect::<Vec<_>>();
        let mut visited = HashSet::new();
        let mut caveats = HashSet::new();
        while let Some(id) = frontier.pop() {
            if !visited.insert(id) {
                continue;
            }
            for edge in self
                .graph
                .edges
                .iter()
                .filter(|edge| edge.to == id && edge.relation == Relation::Qualifies)
            {
                if matches!(
                    self.graph.nodes.get(&edge.from),
                    Some(NodeKind::Caveat { .. })
                ) {
                    caveats.insert(edge.from);
                    frontier.push(edge.from);
                }
            }
        }
        self.symbols
            .iter()
            .filter_map(|(name, id)| caveats.contains(id).then_some(name.clone()))
            .collect()
    }

    fn qualify(&self, evidence: &str, extras: &[String]) -> Result<Provenance, String> {
        self.require_kind(evidence, "evidence")?;
        if !self.predicate("observed", evidence)? {
            return Err(format!(
                "cannot qualify a value with unobserved evidence {evidence}"
            ));
        }
        for caveat in extras {
            self.require_kind(caveat, "caveat")?;
        }
        let id = self.symbols[evidence];
        let mut roots = vec![id];
        roots.extend(extras.iter().map(|caveat| self.symbols[caveat]));
        roots.extend(
            self.graph
                .edges
                .iter()
                .filter(|edge| {
                    edge.from == id
                        && matches!(edge.relation, Relation::Supports | Relation::Opposes)
                        && matches!(self.graph.nodes.get(&edge.to), Some(NodeKind::Claim { .. }))
                })
                .map(|edge| edge.to),
        );
        let caveats = self
            .incoming_caveats(roots)
            .into_iter()
            .chain(extras.iter().cloned());
        let mut provenance = Provenance::from_names([evidence.into()], caveats)?;
        if let Some(observation) = self.observation_qualifications.get(evidence) {
            provenance.merge(observation)?;
        }
        if let Some(dependency) = self
            .predicate_qualifications
            .get("observed")
            .and_then(|targets| targets.get(evidence))
        {
            provenance.merge(dependency)?;
        }
        Ok(provenance)
    }

    fn predicate_tracked(&self, kind: &str, name: &str) -> Result<Tracked<bool>, String> {
        let value = self.predicate(kind, name)?;
        let mut provenance = match kind {
            "observed" if value => self.qualify(name, &[])?,
            "examined" => {
                let id = self.symbols[name];
                let mut provenance = Provenance::from_names(
                    [],
                    std::iter::once(name.into()).chain(self.incoming_caveats([id])),
                )?;
                if let Some(examination) = self.examination_qualifications.get(name) {
                    provenance.merge(examination)?;
                }
                provenance
            }
            "committed" | "reopened"
                if self.symbols.get(name).is_some_and(|id| {
                    matches!(self.graph.nodes.get(id), Some(NodeKind::Commitment { .. }))
                }) =>
            {
                let id = self.symbols[name];
                let mut provenance = self
                    .commitment_bases
                    .get(name)
                    .map(|basis| basis.provenance.clone())
                    .unwrap_or_default();
                if let Some(reopening) = self.reopening_qualifications.get(name) {
                    provenance.merge(reopening)?;
                }
                let by_id = self
                    .symbols
                    .iter()
                    .map(|(name, id)| (*id, name))
                    .collect::<HashMap<_, _>>();
                for edge in &self.graph.edges {
                    if edge.from == id && edge.relation == Relation::Retains {
                        provenance
                            .merge(&Provenance::from_names([], [by_id[&edge.to].clone()])?)?;
                    } else if edge.from == id && edge.relation == Relation::ReliesOn {
                        // Commitment basis is historical. Do not re-query its
                        // original value or replace its frozen qualifications.
                        provenance
                            .merge(&Provenance::from_names([by_id[&edge.to].clone()], [])?)?;
                    } else if kind == "reopened"
                        && edge.to == id
                        && edge.relation == Relation::Reopens
                    {
                        provenance.merge(&self.qualify(by_id[&edge.from], &[])?)?;
                    }
                }
                provenance
            }
            _ => Provenance::default(),
        };
        if let Some(dependency) = self
            .predicate_qualifications
            .get(kind)
            .and_then(|targets| targets.get(name))
        {
            provenance.merge(dependency)?;
        }
        if kind == "reopened" {
            // Whether a future action can be reopened also depends on the
            // earlier decision that left its commitment absent.
            if let Some(dependency) = self
                .predicate_qualifications
                .get("committed")
                .and_then(|targets| targets.get(name))
            {
                provenance.merge(dependency)?;
            }
        }
        Tracked::new(value, provenance)
    }

    fn clear_predicate_dependency(&mut self, kind: &str, name: &str) {
        if let Some(targets) = self.predicate_qualifications.get_mut(kind) {
            targets.remove(name);
            if targets.is_empty() {
                self.predicate_qualifications.remove(kind);
            }
        }
    }

    fn apply_effect(
        &mut self,
        effect: &Effect,
        parameters: &BTreeMap<String, f64>,
        guard: &Provenance,
    ) -> Result<(), String> {
        match effect {
            Effect::Emit { name } => {
                self.cues.push(self.cue_definitions[name].clone());
                self.cue_qualifications.push(guard.clone());
            }
            Effect::Set { name, value } => {
                let value = self.evaluate(value, parameters)?;
                let Value::Number(number) = value.value else {
                    return Err("numeric state expression returned a boolean".into());
                };
                let range = &self.ranges[name];
                check_range(name, number, range.min, range.max)?;
                self.values.insert(
                    name.clone(),
                    Tracked::new(number, value.provenance.union(guard)?)?,
                );
            }
            Effect::Reveal {
                evidence,
                relation,
                claim,
            } => {
                let from = self.symbols[evidence];
                let to = self.symbols[claim];
                if !self
                    .graph
                    .edges
                    .iter()
                    .any(|edge| edge.from == from && edge.to == to && edge.relation == *relation)
                {
                    self.clear_predicate_dependency("observed", evidence);
                    self.observation_qualifications
                        .entry(evidence.clone())
                        .or_default()
                        .merge(guard)?;
                    self.graph.relate(from, *relation, to);
                    self.effects.push(EffectReport::Reveal {
                        evidence: evidence.clone(),
                        relation: relation_name(*relation).into(),
                        target: claim.clone(),
                    });
                }
            }
            Effect::Examine { caveat, cost } => {
                // This guard is part of the successful attention decision. It
                // survives a later `examined(...)` query without changing any
                // underlying claim or removing a retained caveat.
                let attention_basis = self
                    .examination_qualifications
                    .get(caveat)
                    .cloned()
                    .unwrap_or_default()
                    .union(guard)?;
                let resources = self
                    .resources
                    .as_mut()
                    .ok_or("examination requires attention budget")?;
                if *cost > resources.remaining {
                    return Err("insufficient examination budget".into());
                }
                resources.remaining -= cost;
                resources.spent += cost;
                resources.exhausted = resources.remaining == 0;
                self.graph
                    .set_attention(self.symbols[caveat], Attention::Examined);
                self.clear_predicate_dependency("examined", caveat);
                self.examination_qualifications
                    .insert(caveat.clone(), attention_basis);
                self.effects.push(EffectReport::Examine {
                    caveat: caveat.clone(),
                    cost: *cost,
                });
            }
            Effect::Commit {
                action,
                reason,
                using,
                retaining,
            } => {
                if self.symbols.contains_key(action) {
                    return Err(format!("commitment {action} already exists"));
                }
                let mut provenance = guard.clone();
                let used_value = if let Some(expression) = using {
                    let value = self.evaluate(expression, parameters)?;
                    let Value::Number(number) = value.value else {
                        return Err("commit using requires a numeric value".into());
                    };
                    provenance.merge(&value.provenance)?;
                    Some(number)
                } else {
                    None
                };
                provenance.merge(&Provenance::from_names([], retaining.iter().cloned())?)?;
                let mut retained_names = retaining.clone();
                for caveat in &provenance.caveats {
                    self.require_kind(caveat, "caveat")?;
                    if !retained_names.contains(caveat) {
                        retained_names.push(caveat.clone());
                    }
                }
                let retained = retained_names
                    .iter()
                    .map(|name| self.symbols[name])
                    .collect::<Vec<_>>();
                let id = self.graph.commit_because(action, &retained, reason.clone());
                self.clear_predicate_dependency("committed", action);
                self.symbols.insert(action.clone(), id);
                for evidence in &provenance.evidence {
                    self.require_kind(evidence, "evidence")?;
                    if !self.predicate("observed", evidence)? {
                        return Err(format!(
                            "commitment basis includes unobserved evidence {evidence}"
                        ));
                    }
                    self.graph
                        .relate(id, Relation::ReliesOn, self.symbols[evidence]);
                }
                self.commitment_bases.insert(
                    action.clone(),
                    CommitmentBasis {
                        value: used_value,
                        provenance,
                    },
                );
                self.effects.push(EffectReport::Commit {
                    action: action.clone(),
                    retained: retained_names,
                });
            }
            Effect::Reopen { action, because } => {
                let id = *self
                    .symbols
                    .get(action)
                    .ok_or_else(|| format!("cannot reopen uncommitted action {action}"))?;
                if !self.predicate("observed", because)? {
                    return Err(format!("cannot reopen from unobserved evidence {because}"));
                }
                let from = self.symbols[because];
                if !self.graph.edges.iter().any(|edge| {
                    edge.from == from && edge.to == id && edge.relation == Relation::Reopens
                }) {
                    let basis = self.qualify(because, &[])?.union(guard)?;
                    self.clear_predicate_dependency("reopened", action);
                    self.reopening_qualifications
                        .entry(action.clone())
                        .or_default()
                        .merge(&basis)?;
                    self.graph.reopen(id, from);
                    self.effects.push(EffectReport::Reopen {
                        action: action.clone(),
                        because: because.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn snapshot(&self) -> ReactiveSnapshot {
        let names = self
            .symbols
            .iter()
            .map(|(name, id)| (*id, name.as_str()))
            .collect::<HashMap<_, _>>();
        let mut sorted = self.symbols.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|(name, _)| *name);
        let mut symbols = Vec::new();
        let mut commitments = Vec::new();
        for (name, id) in sorted {
            let (kind, source, consequence, attention) = match &self.graph.nodes[id] {
                NodeKind::Claim { .. } => ("claim", None, None, None),
                NodeKind::Evidence { source, .. } => ("evidence", Some(source.clone()), None, None),
                NodeKind::Caveat {
                    consequence,
                    attention,
                    ..
                } => (
                    "caveat",
                    None,
                    Some(format!("{consequence:?}").to_lowercase()),
                    Some(format!("{attention:?}").to_lowercase()),
                ),
                NodeKind::Commitment { open, .. } => {
                    commitments.push(MapCommitment {
                        action: name.clone(),
                        open: *open,
                        retained: self
                            .graph
                            .edges
                            .iter()
                            .filter(|edge| edge.from == *id && edge.relation == Relation::Retains)
                            .map(|edge| names[&edge.to].to_string())
                            .collect(),
                        reopened_by: self
                            .graph
                            .edges
                            .iter()
                            .filter(|edge| edge.to == *id && edge.relation == Relation::Reopens)
                            .map(|edge| names[&edge.from].to_string())
                            .collect(),
                    });
                    ("commitment", None, None, None)
                }
                NodeKind::Context { .. } => ("context", None, None, None),
                NodeKind::Observation { .. } => ("observation", None, None, None),
            };
            symbols.push(GameSymbol {
                name: name.clone(),
                kind: kind.into(),
                source,
                consequence,
                attention,
                display: self.labels.get(name).cloned(),
            });
        }
        ReactiveSnapshot {
            bindings: self.bindings.clone(),
            binding_qualifications: self.binding_qualifications.clone(),
            cues: self.cues.clone(),
            cue_qualifications: self.cue_qualifications.clone(),
            qualified_values: self.values.clone(),
            commitment_bases: self.commitment_bases.clone(),
            observation_qualifications: self.observation_qualifications.clone(),
            examination_qualifications: self.examination_qualifications.clone(),
            reopening_qualifications: self.reopening_qualifications.clone(),
            predicate_qualifications: self.predicate_qualifications.clone(),
            controls: self.controls.clone(),
            clock: self.clock.clone(),
            schema: REACTIVE_SCHEMA.into(),
            source_id: self.source_id.clone(),
            sequence: self.sequence,
            last_event: self.last_event.clone(),
            values: self
                .values
                .iter()
                .map(|(name, value)| (name.clone(), value.value))
                .collect(),
            world: self.world.clone(),
            events: self
                .events
                .iter()
                .map(|(name, parameters)| EventSignature {
                    name: name.clone(),
                    parameters: parameters.clone(),
                })
                .collect(),
            scenes: self.scenes.clone(),
            labels: self.labels.clone(),
            symbols,
            commitments,
            relations: self
                .graph
                .edges
                .iter()
                .map(|edge| MapRelation {
                    from: names[&edge.from].into(),
                    to: names[&edge.to].into(),
                    relation: relation_name(edge.relation).into(),
                    origin: "live".into(),
                })
                .collect(),
            budget: self.resources.as_ref().map(|resources| MapBudget {
                initial: resources.initial,
                spent: resources.spent,
                remaining: resources.remaining,
                exhausted: resources.exhausted,
            }),
            effects: self.effects.clone(),
        }
    }
}

fn check_range(name: &str, value: f64, min: f64, max: f64) -> Result<(), String> {
    if !value.is_finite() || value < min || value > max || value.abs() > LIMIT {
        Err(format!("{name} must be finite and in {min}..{max}"))
    } else {
        Ok(())
    }
}

fn relation_name(relation: Relation) -> &'static str {
    match relation {
        Relation::Supports => "supports",
        Relation::Opposes => "opposes",
        Relation::Qualifies => "qualifies",
        Relation::InContext => "in_context",
        Relation::Retains => "retains",
        Relation::Reopens => "reopens",
        Relation::ReliesOn => "relies_on",
    }
}

fn source_identity(source: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in source.as_bytes() {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}:{}", source.len())
}

fn identifier(name: &str) -> Result<String, String> {
    let mut chars = name.chars();
    if !chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(format!("invalid reactive identifier {name}"));
    }
    if matches!(name, "true" | "false" | "and" | "or" | "not") {
        return Err(format!("reserved reactive identifier {name}"));
    }
    Ok(name.into())
}

fn bounds(min: &str, max: &str) -> Result<(Number, Number), String> {
    let min = Number::parse(min)?;
    let max = Number::parse(max)?;
    if min.value() > max.value() || min.value().abs() > LIMIT || max.value().abs() > LIMIT {
        return Err("reactive bounds must be ordered and within +/-1e12".into());
    }
    Ok((min, max))
}

/// Split declaration syntax without changing whitespace inside text values.
fn syntax_words(input: &str) -> Vec<&str> {
    let mut words = Vec::new();
    let mut start = None;
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in input.char_indices() {
        if !quoted && ch.is_whitespace() {
            if let Some(start) = start.take() {
                words.push(&input[start..index]);
            }
            continue;
        }
        start.get_or_insert(index);
        if escaped {
            escaped = false;
        } else if quoted && ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            quoted = !quoted;
        }
    }
    if let Some(start) = start {
        words.push(&input[start..]);
    }
    words
}

fn expression_depth_delta(input: &str) -> i64 {
    let mut depth = 0;
    let mut quoted = false;
    let mut escaped = false;
    for ch in input.chars() {
        if escaped {
            escaped = false;
        } else if quoted && ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            quoted = !quoted;
        } else if !quoted {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
            }
        }
    }
    depth
}

pub fn parse_directive(line: &str) -> Option<Result<Directive, String>> {
    let words = syntax_words(line);
    let keyword = words.first().copied()?;
    if !matches!(
        keyword,
        "state" | "event" | "on" | "fn" | "bind" | "cue" | "control" | "clock"
    ) {
        return None;
    }
    Some((|| match keyword {
        "fn" => parse_function(line),
        "bind" => parse_binding(line),
        "cue" => parse_cue(line),
        "control" => match words.as_slice() {
            ["control", name, "=", event] => Ok(Directive::Control {
                name: identifier(name)?,
                control: Control {
                    event: identifier(event)?,
                    reset: false,
                },
            }),
            ["control", name, "=", event, "reset"] => Ok(Directive::Control {
                name: identifier(name)?,
                control: Control {
                    event: identifier(event)?,
                    reset: true,
                },
            }),
            _ => Err("control expects NAME = EVENT [reset]".into()),
        },
        "clock" => match words.as_slice() {
            ["clock", event, "every", step] => Ok(Directive::Clock(Clock {
                event: identifier(event)?,
                step: Number::parse(step)?,
            })),
            _ => Err("clock expects EVENT every STEP".into()),
        },
        "state" => {
            let (name, initial) = line
                .trim_start_matches("state")
                .trim()
                .split_once('=')
                .ok_or("state declaration requires NAME = EXPRESSION")?;
            let tokens = syntax_words(initial);
            let (end, min, max) = if tokens.len() >= 5
                && tokens[tokens.len() - 4] == "min"
                && tokens[tokens.len() - 2] == "max"
            {
                let (min, max) = bounds(tokens[tokens.len() - 3], tokens[tokens.len() - 1])?;
                (tokens.len() - 4, min, max)
            } else {
                (
                    tokens.len(),
                    Number::parse("-1000000000000")?,
                    Number::parse("1000000000000")?,
                )
            };
            Ok(Directive::State {
                name: identifier(name.trim())?,
                initial: reactive_expr::parse_unresolved(&tokens[..end].join(" "))?,
                min,
                max,
            })
        }
        "event" => {
            let name = words.get(1).ok_or("event declaration requires a name")?;
            let rest = words[2..].join(" ");
            let mut parameters = Vec::new();
            if !rest.is_empty() {
                for parameter in rest.split(',') {
                    let tokens = parameter.split_whitespace().collect::<Vec<_>>();
                    match tokens.as_slice() {
                        [name, "min", min, "max", max] => {
                            let (min, max) = bounds(min, max)?;
                            parameters.push(Parameter {
                                name: identifier(name)?,
                                min,
                                max,
                            });
                        }
                        _ => {
                            return Err("event parameter must be NAME min NUMBER max NUMBER".into())
                        }
                    }
                }
            }
            Ok(Directive::Event {
                name: identifier(name)?,
                parameters,
            })
        }
        "on" => {
            let event = identifier(words.get(1).ok_or("on requires an event name")?)?;
            // Effect words may also be identifiers, including inside spaced
            // graph predicates. Find a syntactically complete effect outside
            // expression parentheses, rather than splitting at the first word.
            let mut depth = 0_i64;
            let mut candidates = Vec::new();
            for (index, token) in words.iter().enumerate().skip(2) {
                if depth == 0
                    && matches!(
                        *token,
                        "set" | "reveal" | "examine" | "commit" | "reopen" | "emit"
                    )
                {
                    candidates.push(index);
                }
                depth += expression_depth_delta(token);
            }
            let effect_index = candidates
                .iter()
                .copied()
                .find(|index| parse_effect(&words[*index..]).is_ok())
                .or_else(|| candidates.last().copied())
                .ok_or("on rule requires set, reveal, examine, commit, reopen, or emit effect")?;
            let condition = if effect_index == 2 {
                reactive_expr::parse("true")?
            } else if words.get(2) == Some(&"when") {
                reactive_expr::parse_unresolved(&words[3..effect_index].join(" "))?
            } else {
                return Err("on rule requires when CONDITION before its effect".into());
            };
            let effect = parse_effect(&words[effect_index..])?;
            Ok(Directive::Rule(Rule {
                event,
                condition,
                effect,
            }))
        }
        _ => unreachable!(),
    })())
}

fn parse_function(line: &str) -> Result<Directive, String> {
    let rest = line.strip_prefix("fn").unwrap().trim();
    let (header, body) = rest
        .split_once('=')
        .ok_or("fn expects NAME(PARAMETERS) = EXPRESSION")?;
    let (name, parameters) = header
        .trim()
        .split_once('(')
        .ok_or("fn requires a parameter list")?;
    let parameters = parameters
        .trim()
        .strip_suffix(')')
        .ok_or("fn parameter list requires closing parenthesis")?;
    let parameters = if parameters.trim().is_empty() {
        Vec::new()
    } else {
        parameters
            .split(',')
            .map(|parameter| identifier(parameter.trim()))
            .collect::<Result<Vec<_>, _>>()?
    };
    Ok(Directive::Function(FunctionDef {
        name: identifier(name.trim())?,
        parameters,
        body: reactive_expr::parse_unresolved(body.trim())?,
    }))
}

/// Locate a trailing `when` outside strings and expression parentheses.
fn binding_parts(value: &str) -> Result<(&str, Option<&str>), String> {
    let mut quoted = false;
    let mut escaped = false;
    let mut depth = 0_i64;
    let mut separator = None;
    for (index, ch) in value.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
        } else if ch == '"' {
            quoted = true;
        } else if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
        } else if depth == 0
            && value[index..].starts_with("when")
            && index > 0
            && value[..index].ends_with(char::is_whitespace)
            && value[index + 4..].starts_with(char::is_whitespace)
        {
            separator = Some(index);
        }
    }
    if let Some(index) = separator {
        let expression = value[..index].trim();
        let condition = value[index + 4..].trim();
        if expression.is_empty() || condition.is_empty() {
            return Err("bind requires a value and a condition after when".into());
        }
        Ok((expression, Some(condition)))
    } else {
        Ok((value.trim(), None))
    }
}

fn parse_binding(line: &str) -> Result<Directive, String> {
    let (path, value) = line
        .strip_prefix("bind")
        .unwrap()
        .trim()
        .split_once('=')
        .ok_or("bind expects TARGET.PROPERTY = VALUE [when CONDITION]")?;
    let (target, property) = path
        .trim()
        .split_once('.')
        .ok_or("bind requires TARGET.PROPERTY")?;
    let target = identifier(target)?;
    for segment in property.split('.') {
        identifier(segment)?;
    }
    let (value, condition) = binding_parts(value)?;
    let value = BindingExpression::Expression(reactive_expr::parse_unresolved(value)?);
    Ok(Directive::Binding(Binding {
        target,
        property: property.into(),
        value,
        condition: reactive_expr::parse_unresolved(condition.unwrap_or("true"))?,
    }))
}

fn quoted_prefix(value: &str) -> Result<(String, &str), String> {
    let value = value.trim_start();
    if !value.starts_with('"') {
        return Err("expected quoted text".into());
    }
    let mut escaped = false;
    for (index, ch) in value.char_indices().skip(1) {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Ok((
                crate::parser::quoted(&value[..index + 1])?,
                value[index + 1..].trim(),
            ));
        }
    }
    Err("unterminated quoted text".into())
}

fn cue_number(name: &str, value: &str, min: f64, max: f64) -> Result<Number, String> {
    let number = Number::parse(value)?;
    check_range(name, number.value(), min, max)?;
    Ok(number)
}

fn duration(value: &str) -> Result<Number, String> {
    let value = cue_number("cue duration", value, 0.0, 30.0)?;
    if value.value() == 0.0 {
        return Err("cue duration must be positive".into());
    }
    Ok(value)
}

fn parse_cue(line: &str) -> Result<Directive, String> {
    let mut rest = line;
    let mut header = Vec::new();
    for _ in 0..3 {
        rest = rest.trim_start();
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        header.push(&rest[..end]);
        rest = &rest[end..];
    }
    let id = identifier(header[1])?;
    let rest = rest.trim();
    let cue = match header[2] {
        "sound" => {
            let tokens = rest.split_whitespace().collect::<Vec<_>>();
            let [frequency, seconds, gain] = tokens.as_slice() else {
                return Err("sound cue expects FREQUENCY DURATION GAIN".into());
            };
            Cue::Sound {
                id,
                frequency: cue_number("sound frequency", frequency, 20.0, 20_000.0)?,
                duration: duration(seconds)?,
                gain: cue_number("sound gain", gain, 0.0, 1.0)?,
            }
        }
        "toast" => {
            let (text, seconds) = quoted_prefix(rest)?;
            Cue::Toast {
                id,
                text,
                duration: duration(seconds)?,
            }
        }
        "flash" => {
            let tokens = rest.split_whitespace().collect::<Vec<_>>();
            let [target, seconds] = tokens.as_slice() else {
                return Err("flash cue expects TARGET DURATION".into());
            };
            Cue::Flash {
                id,
                target: identifier(target)?,
                duration: duration(seconds)?,
            }
        }
        "ring" => {
            let (target, rest) = rest
                .split_once(char::is_whitespace)
                .ok_or("ring cue expects TARGET COLOR DURATION")?;
            let rest = rest.trim();
            let (color, seconds) = if rest.starts_with('"') {
                let (color, seconds) = quoted_prefix(rest)?;
                let digits = color
                    .strip_prefix('#')
                    .ok_or("ring color must be quoted #RGB or #RRGGBB")?;
                if !matches!(digits.len(), 3 | 6)
                    || !digits.chars().all(|ch| ch.is_ascii_hexdigit())
                {
                    return Err("ring color must be quoted #RGB or #RRGGBB".into());
                }
                (CueColor::Text(color), seconds)
            } else {
                let (color, seconds) = rest
                    .split_once(char::is_whitespace)
                    .ok_or("ring cue expects COLOR DURATION")?;
                let color = color
                    .parse::<u32>()
                    .map_err(|_| "numeric ring color must be an integer in 0..16777215")?;
                if color > 0xff_ffff {
                    return Err("numeric ring color must be an integer in 0..16777215".into());
                }
                (CueColor::Number(color), seconds.trim())
            };
            Cue::Ring {
                id,
                target: identifier(target)?,
                color,
                duration: duration(seconds)?,
            }
        }
        _ => return Err("cue kind must be sound, toast, flash, or ring".into()),
    };
    Ok(Directive::Cue(cue))
}

fn parse_effect(words: &[&str]) -> Result<Effect, String> {
    match words {
        ["emit", name] => Ok(Effect::Emit {
            name: identifier(name)?,
        }),
        ["set", name, "=", rest @ ..] if !rest.is_empty() => Ok(Effect::Set {
            name: identifier(name)?,
            value: reactive_expr::parse_unresolved(&rest.join(" "))?,
        }),
        ["reveal", evidence, relation, claim] if matches!(*relation, "supports" | "opposes") => {
            Ok(Effect::Reveal {
                evidence: identifier(evidence)?,
                relation: if *relation == "supports" {
                    Relation::Supports
                } else {
                    Relation::Opposes
                },
                claim: identifier(claim)?,
            })
        }
        ["examine", caveat, "cost", cost] => Ok(Effect::Examine {
            caveat: identifier(caveat)?,
            cost: cost
                .parse()
                .map_err(|_| "examination cost must be an unsigned integer")?,
        }),
        ["commit", action, "because", reason, rest @ ..] => {
            let reason = match *reason {
                "enough" => StopReason::Enough,
                "budget" => StopReason::BudgetExhausted,
                "deadline" => StopReason::Deadline,
                _ => return Err(format!("unknown commit reason {reason}")),
            };
            let parse_retained = |names: &[&str]| -> Result<Vec<String>, String> {
                if names.is_empty() {
                    return Err("retaining requires at least one caveat".into());
                }
                names
                    .join(" ")
                    .split(',')
                    .map(|name| identifier(name.trim()))
                    .collect()
            };
            let (using, retaining) = match rest {
                [] => (None, Vec::new()),
                ["retaining", names @ ..] => (None, parse_retained(names)?),
                ["using", expression @ ..] if !expression.is_empty() => {
                    let mut depth = 0_i64;
                    let mut boundary = None;
                    for (index, token) in expression.iter().enumerate() {
                        if index > 0 && depth == 0 && *token == "retaining" {
                            boundary = Some(index);
                            break;
                        }
                        depth += expression_depth_delta(token);
                    }
                    let end = boundary.unwrap_or(expression.len());
                    let using = reactive_expr::parse_unresolved(&expression[..end].join(" "))?;
                    let retaining = boundary
                        .map(|index| parse_retained(&expression[index + 1..]))
                        .transpose()?
                        .unwrap_or_default();
                    (Some(using), retaining)
                }
                _ => {
                    return Err(
                        "commit expects [using NUMERIC_EXPRESSION] [retaining CAVEAT, CAVEAT]"
                            .into(),
                    )
                }
            };
            Ok(Effect::Commit {
                action: identifier(action)?,
                reason,
                using,
                retaining,
            })
        }
        ["reopen", action, "because", evidence] => Ok(Effect::Reopen {
            action: identifier(action)?,
            because: identifier(evidence)?,
        }),
        _ => Err(format!("invalid reactive effect: {}", words.join(" "))),
    }
}

struct NumericPayload(BTreeMap<String, f64>);

impl<'de> serde::Deserialize<'de> for NumericPayload {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = NumericPayload;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an object of unique numeric event parameters")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, f64>()? {
                    if values.insert(key.clone(), value).is_some() {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate parameter {key}"
                        )));
                    }
                }
                Ok(NumericPayload(values))
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}
