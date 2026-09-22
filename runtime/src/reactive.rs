//! Bounded, deterministic event programs over the CAVEAT epistemic graph.
//!
//! The host dispatches declared inputs. Source owns numeric updates and graph
//! effects; an entire event publishes atomically or leaves the session intact.

use crate::ast::{Program, Statement};
use crate::eval::ResourceLedger;
use crate::game_session::GameSymbol;
use crate::map::{CaveatMap, MapBudget, MapCommitment, MapRelation, MapWorld};
use crate::presentation::Number;
use crate::reactive_expr::{self, Expr, FunctionDef, HistoryRead, Value, ValueType};
pub use crate::reactive_expr::{Provenance, Tracked};
use crate::{Attention, EpistemicGraph, NodeId, NodeKind, Relation, StopReason};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

const LIMIT: f64 = 1_000_000_000_000.0;
const MAX_HISTORY_LIMIT: usize = 256;
const MAX_DECLARED_HISTORY_CAPACITY: usize = 1024;
const MAX_PROCEDURES: usize = 128;
const MAX_PROCEDURE_PARAMETERS: usize = 32;
const MAX_PROCEDURE_STEPS: usize = 4096;
const MAX_PROCEDURE_DEPTH: usize = 64;
const MAX_EVENT_STEPS: usize = 4096;
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
    Call {
        name: String,
        arguments: Vec<Expr>,
    },
    Sample {
        stream: String,
        value: Expr,
        relation: Relation,
        claim: String,
    },
    Emit {
        name: String,
    },
    /// Fail the event with this message. Nothing the event did is kept.
    Reject {
        message: String,
    },
    Set {
        name: String,
        value: Expr,
        /// Authored grounds; `None` grounds the state on its value expression.
        /// See spec/caveat-explanations-0.2.md.
        because: Option<Vec<Expr>>,
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
        because: EvidenceSelector,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceSelector {
    Named(String),
    Latest(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub event: String,
    pub condition: Expr,
    pub effect: Effect,
}

/// An ordered effect program. Parameters are frozen numeric values; global
/// state and graph reads observe earlier effects in the same transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Procedure {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<GuardedEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardedEffect {
    pub condition: Expr,
    pub effect: Effect,
}

fn expand_effect(
    effect: &mut Effect,
    functions: &BTreeMap<String, FunctionDef>,
    defines: &BTreeMap<String, Expr>,
) -> Result<(), String> {
    match effect {
        Effect::Set { value, because, .. } => {
            *value = reactive_expr::expand_with(value, functions, defines)?;
            for citation in because.iter_mut().flatten() {
                *citation = reactive_expr::expand_with(citation, functions, defines)?;
            }
        }
        Effect::Sample { value, .. }
        | Effect::Commit {
            using: Some(value), ..
        } => {
            *value = reactive_expr::expand_with(value, functions, defines)?;
        }
        Effect::Call { arguments, .. } => {
            for argument in arguments {
                *argument = reactive_expr::expand_with(argument, functions, defines)?;
            }
        }
        _ => {}
    }
    Ok(())
}

struct ExecutionBudget {
    remaining: usize,
    depth: usize,
}

impl ExecutionBudget {
    fn spend(&mut self) -> Result<(), String> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or_else(|| format!("event exceeds work limit {MAX_EVENT_STEPS}"))?;
        Ok(())
    }

    fn enter(&mut self) -> Result<(), String> {
        if self.depth >= MAX_PROCEDURE_DEPTH {
            return Err(format!(
                "procedure exceeds call depth limit {MAX_PROCEDURE_DEPTH}"
            ));
        }
        self.depth += 1;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    Procedure(Procedure),
    Readings {
        name: String,
        template: String,
        limit: usize,
    },
    Decisions {
        name: String,
        limit: usize,
    },
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
    /// `define NAME = EXPRESSION;`, inlined where NAME is read.
    Define {
        name: String,
        expression: Expr,
    },
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
    /// What this binding cites when it wins: `None` cites its whole lineage,
    /// `Some(vec![])` is `because nothing`. See spec/caveat-explanations-0.1.md.
    pub because: Option<Vec<Expr>>,
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReadingOccurrence {
    pub id: String,
    pub ordinal: u64,
    pub sequence: u64,
    pub event: String,
    pub value: f64,
    pub provenance: Provenance,
    pub relation: String,
    pub claim: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReadingStream {
    pub template: String,
    pub limit: usize,
    pub current: Option<String>,
    pub occurrences: Vec<ReadingOccurrence>,
    pub selection_qualifications: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecisionRevision {
    pub id: String,
    pub previous: Option<String>,
    pub ordinal: u64,
    pub sequence: u64,
    pub event: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionSeries {
    pub limit: usize,
    pub current: Option<String>,
    pub revisions: Vec<DecisionRevision>,
    pub selection_qualifications: Provenance,
}

#[derive(Debug, Clone)]
struct StateRange {
    min: f64,
    max: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EffectReport {
    Sample {
        stream: String,
        id: String,
        value: f64,
        relation: String,
        target: String,
    },
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
    pub reading_streams: BTreeMap<String, ReadingStream>,
    pub decision_series: BTreeMap<String, DecisionSeries>,
    pub observation_qualifications: BTreeMap<String, Provenance>,
    pub examination_qualifications: BTreeMap<String, Provenance>,
    pub reopening_qualifications: BTreeMap<String, Provenance>,
    pub predicate_qualifications: BTreeMap<String, BTreeMap<String, Provenance>>,
    pub events: Vec<EventSignature>,
    pub bindings: BTreeMap<String, BTreeMap<String, BindingValue>>,
    pub binding_qualifications: BTreeMap<String, BTreeMap<String, Provenance>>,
    /// What each shown binding cites: a subset of its qualifications.
    pub binding_explanations: BTreeMap<String, BTreeMap<String, Provenance>>,
    /// What each state is grounded on: its content, never its control. A
    /// subset of `qualified_values`. See spec/caveat-explanations-0.2.md.
    pub value_grounds: BTreeMap<String, Provenance>,
    /// What each commitment was grounded on: its `using` content and retained
    /// caveats, without its guard or a predecessor's basis.
    pub commitment_grounds: BTreeMap<String, Provenance>,
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
    reading_streams: BTreeMap<String, ReadingStream>,
    decision_series: BTreeMap<String, DecisionSeries>,
    observation_qualifications: BTreeMap<String, Provenance>,
    examination_qualifications: BTreeMap<String, Provenance>,
    reopening_qualifications: BTreeMap<String, Provenance>,
    predicate_qualifications: BTreeMap<String, BTreeMap<String, Provenance>>,
    ranges: BTreeMap<String, StateRange>,
    constants: BTreeMap<String, f64>,
    events: BTreeMap<String, Vec<Parameter>>,
    rules: Arc<Vec<Rule>>,
    procedures: Arc<BTreeMap<String, Procedure>>,
    binding_rules: Arc<Vec<Binding>>,
    define_rules: Arc<Vec<(String, Expr)>>,
    bindings: BTreeMap<String, BTreeMap<String, BindingValue>>,
    binding_qualifications: BTreeMap<String, BTreeMap<String, Provenance>>,
    binding_explanations: BTreeMap<String, BTreeMap<String, Provenance>>,
    grounds: BTreeMap<String, Provenance>,
    commitment_grounds: BTreeMap<String, Provenance>,
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
        // Draft 0.5: a bundle links to one program, but its identity stays the
        // bundle bytes the author shipped, so saves and byte-pinned receipts
        // keep referring to that. A source without the bundle marker links to
        // itself, so single-file programs are unaffected.
        let program = crate::parser::parse(&crate::link::link(source)?)?;
        let mut declarations = Vec::new();
        let mut directives = Vec::new();
        for statement in &program.statements {
            match statement {
                Statement::Reactive(directive) => directives.push(directive.clone()),
                Statement::Origin { .. }
                | Statement::Scene { .. }
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
        let mut defines = BTreeMap::new();
        for directive in &directives {
            if let Directive::Define { name, expression } = directive {
                if functions.contains_key(name) {
                    return Err(format!("define {name} collides with a function"));
                }
                if defines.insert(name.clone(), expression.clone()).is_some() {
                    return Err(format!("duplicate define {name}"));
                }
            }
        }
        for directive in &mut directives {
            match directive {
                Directive::State { initial, .. } => {
                    *initial = reactive_expr::expand_with(initial, &functions, &defines)?
                }
                Directive::Rule(rule) => {
                    rule.condition =
                        reactive_expr::expand_with(&rule.condition, &functions, &defines)?;
                    expand_effect(&mut rule.effect, &functions, &defines)?;
                }
                Directive::Procedure(procedure) => {
                    for step in &mut procedure.body {
                        step.condition =
                            reactive_expr::expand_with(&step.condition, &functions, &defines)?;
                        expand_effect(&mut step.effect, &functions, &defines)?;
                    }
                }
                Directive::Binding(binding) => {
                    binding.condition =
                        reactive_expr::expand_with(&binding.condition, &functions, &defines)?;
                    if let BindingExpression::Expression(value) = &mut binding.value {
                        *value = reactive_expr::expand_with(value, &functions, &defines)?;
                    }
                    for citation in binding.because.iter_mut().flatten() {
                        *citation = reactive_expr::expand_with(citation, &functions, &defines)?;
                    }
                }
                // Expanded on its own too, so a cycle or bad call in an unused
                // define is still an error.
                Directive::Define { expression, .. } => {
                    *expression = reactive_expr::expand_with(expression, &functions, &defines)?;
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
            reading_streams: BTreeMap::new(),
            decision_series: BTreeMap::new(),
            observation_qualifications: BTreeMap::new(),
            examination_qualifications: BTreeMap::new(),
            reopening_qualifications: BTreeMap::new(),
            predicate_qualifications: BTreeMap::new(),
            ranges: BTreeMap::new(),
            constants,
            events: BTreeMap::new(),
            rules: Arc::new(Vec::new()),
            procedures: Arc::new(BTreeMap::new()),
            binding_rules: Arc::new(Vec::new()),
            define_rules: Arc::new(Vec::new()),
            bindings: BTreeMap::new(),
            binding_qualifications: BTreeMap::new(),
            binding_explanations: BTreeMap::new(),
            grounds: BTreeMap::new(),
            commitment_grounds: BTreeMap::new(),
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
        let mut history_capacity = 0_usize;
        for directive in &directives {
            let (name, limit) = match directive {
                Directive::Readings { name, limit, .. } | Directive::Decisions { name, limit } => {
                    (name, *limit)
                }
                _ => continue,
            };
            if limit == 0 || limit > MAX_HISTORY_LIMIT {
                return Err(format!(
                    "history {name} limit must be in 1..{MAX_HISTORY_LIMIT}"
                ));
            }
            history_capacity += limit;
            if history_capacity > MAX_DECLARED_HISTORY_CAPACITY {
                return Err(format!(
                    "declared history capacity exceeds {MAX_DECLARED_HISTORY_CAPACITY}"
                ));
            }
            if session.symbols.contains_key(name)
                || session.symbols.keys().any(|symbol| symbol.starts_with(&format!("{name}@")))
                || session.reading_streams.contains_key(name)
                || session.decision_series.contains_key(name)
                || directives.iter().any(|directive| matches!(directive, Directive::State { name: state, .. } if state == name))
            {
                return Err(format!("history {name} conflicts with another declared symbol"));
            }
            match directive {
                Directive::Readings { template, .. } => {
                    session.require_kind(template, "evidence")?;
                    session.reading_streams.insert(
                        name.clone(),
                        ReadingStream {
                            template: template.clone(),
                            limit,
                            current: None,
                            occurrences: Vec::new(),
                            selection_qualifications: Provenance::default(),
                        },
                    );
                }
                Directive::Decisions { .. } => {
                    session.decision_series.insert(
                        name.clone(),
                        DecisionSeries {
                            limit,
                            current: None,
                            revisions: Vec::new(),
                            selection_qualifications: Provenance::default(),
                        },
                    );
                }
                _ => unreachable!(),
            }
        }
        for directive in &directives {
            match directive {
                Directive::Define { name, expression } => Arc::make_mut(&mut session.define_rules)
                    .push((name.clone(), expression.clone())),
                Directive::Procedure(procedure) => {
                    if functions.contains_key(&procedure.name) {
                        return Err(format!(
                            "procedure {} shadows a source function",
                            procedure.name
                        ));
                    }
                    if Arc::make_mut(&mut session.procedures)
                        .insert(procedure.name.clone(), procedure.clone())
                        .is_some()
                    {
                        return Err(format!("duplicate procedure {}", procedure.name));
                    }
                }
                Directive::Function(_)
                | Directive::Readings { .. }
                | Directive::Decisions { .. } => {}
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
                        "numeric_history" => session.require_history(name),
                        "has_sample" => session.require_readings(name),
                        _ => Err("state initializers cannot query live graph predicates".into()),
                    })? != ValueType::Number
                    {
                        return Err(format!("state {name} initializer must be numeric"));
                    }
                    let value = initial.evaluate_tracked_with_histories(
                        &|name| {
                            Ok(session.values.get(name).cloned().or_else(|| {
                                session.constants.get(name).copied().map(Tracked::plain)
                            }))
                        },
                        &|kind, name| session.predicate_tracked(kind, name),
                        &|evidence, caveats| session.qualify(evidence, caveats),
                        &|name, query| session.history_read(name, query),
                    )?;
                    let Value::Number(number) = value.value else {
                        unreachable!("validated number initializer")
                    };
                    check_range(name, number, min.value(), max.value())?;
                    let grounds = session
                        .evaluate_grounds(initial, &BTreeMap::new())?
                        .provenance;
                    session.grounds.insert(name.clone(), grounds);
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
        session.validate_procedures()?;
        session.validate_rules()?;
        session.evaluate_bindings()?;
        Ok(session)
    }

    fn validate_procedures(&self) -> Result<(), String> {
        if self.procedures.len() > MAX_PROCEDURES {
            return Err(format!("source exceeds procedure limit {MAX_PROCEDURES}"));
        }
        let mut declared_steps = 0_usize;
        for procedure in self.procedures.values() {
            if procedure.parameters.len() > MAX_PROCEDURE_PARAMETERS {
                return Err(format!(
                    "procedure {} exceeds parameter limit {MAX_PROCEDURE_PARAMETERS}",
                    procedure.name
                ));
            }
            declared_steps = declared_steps
                .checked_add(procedure.body.len())
                .ok_or("procedure step count overflow")?;
            if declared_steps > MAX_PROCEDURE_STEPS {
                return Err(format!(
                    "source exceeds procedure step limit {MAX_PROCEDURE_STEPS}"
                ));
            }
            let mut names = HashSet::new();
            for parameter in &procedure.parameters {
                if !names.insert(parameter) {
                    return Err(format!(
                        "duplicate parameter {parameter} for procedure {}",
                        procedure.name
                    ));
                }
                if self.values.contains_key(parameter) || self.constants.contains_key(parameter) {
                    return Err(format!(
                        "procedure {} parameter {parameter} shadows state or a coordinate",
                        procedure.name
                    ));
                }
            }
        }
        // Check every definition, even unused ones. Memoized expanded work
        // counts stop acyclic fanout from creating unbounded execution.
        let mut costs = BTreeMap::new();
        let mut active = Vec::new();
        for name in self.procedures.keys() {
            self.procedure_cost(name, &mut costs, &mut active)?;
        }
        Ok(())
    }

    fn procedure_cost(
        &self,
        name: &str,
        costs: &mut BTreeMap<String, (usize, usize)>,
        active: &mut Vec<String>,
    ) -> Result<(usize, usize), String> {
        if active.iter().any(|parent| parent == name) {
            return Err(format!("recursive procedure cycle involving {name}"));
        }
        if active.len() >= MAX_PROCEDURE_DEPTH {
            return Err(format!(
                "procedure exceeds call depth limit {MAX_PROCEDURE_DEPTH}"
            ));
        }
        if let Some(&(cost, depth)) = costs.get(name) {
            if active.len() + depth > MAX_PROCEDURE_DEPTH {
                return Err(format!(
                    "procedure exceeds call depth limit {MAX_PROCEDURE_DEPTH}"
                ));
            }
            return Ok((cost, depth));
        }
        let procedure = self
            .procedures
            .get(name)
            .ok_or_else(|| format!("unknown procedure {name}"))?;
        active.push(name.into());
        let mut cost = 0_usize;
        let mut depth = 1;
        for step in &procedure.body {
            cost += 1;
            if let Effect::Call { name, .. } = &step.effect {
                let (nested_cost, nested_depth) = self.procedure_cost(name, costs, active)?;
                cost += nested_cost;
                depth = depth.max(nested_depth + 1);
            }
            if cost > MAX_EVENT_STEPS {
                return Err(format!(
                    "procedure {} exceeds expanded work limit {MAX_EVENT_STEPS}",
                    procedure.name
                ));
            }
        }
        active.pop();
        costs.insert(name.into(), (cost, depth));
        Ok((cost, depth))
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
        commitments.extend(self.decision_series.keys().cloned());
        for effect in self.rules.iter().map(|rule| &rule.effect).chain(
            self.procedures
                .values()
                .flat_map(|procedure| procedure.body.iter().map(|step| &step.effect)),
        ) {
            if let Effect::Commit { action, .. } = effect {
                if self.reading_streams.contains_key(action) {
                    return Err(format!(
                        "commitment {action} conflicts with a reading stream"
                    ));
                }
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
                "numeric_history" => self.require_history(symbol),
                "has_sample" => self.require_readings(symbol),
                "committed" | "reopened" if commitments.contains(symbol) => Ok(()),
                "committed" | "reopened" => Err(format!("unknown reactive commitment {symbol}")),
                _ => Err(format!("unknown epistemic predicate {kind}")),
            }
        };
        let mut steps = Vec::new();
        for (index, rule) in self.rules.iter().enumerate() {
            let parameters = self.events.get(&rule.event).ok_or_else(|| {
                format!(
                    "rule {} references undeclared event {}",
                    index + 1,
                    rule.event
                )
            })?;
            steps.push((
                format!("rule {}", index + 1),
                &rule.condition,
                &rule.effect,
                parameters
                    .iter()
                    .map(|parameter| parameter.name.as_str())
                    .collect::<Vec<_>>(),
            ));
        }
        for procedure in self.procedures.values() {
            for (index, step) in procedure.body.iter().enumerate() {
                steps.push((
                    format!("procedure {}, step {}", procedure.name, index + 1),
                    &step.condition,
                    &step.effect,
                    procedure.parameters.iter().map(String::as_str).collect(),
                ));
            }
        }
        for (context, condition, effect, parameters) in steps {
            let numeric = self
                .values
                .keys()
                .chain(self.constants.keys())
                .cloned()
                .chain(parameters.into_iter().map(str::to_owned))
                .collect();
            if condition.validate(&numeric, &validate_predicate)? != ValueType::Bool {
                return Err(format!("{context} condition must be boolean"));
            }
            match effect {
                Effect::Call { name, arguments } => {
                    let procedure = self
                        .procedures
                        .get(name)
                        .ok_or_else(|| format!("unknown procedure {name}"))?;
                    if arguments.len() != procedure.parameters.len() {
                        return Err(format!(
                            "procedure {name} requires {} arguments, got {}",
                            procedure.parameters.len(),
                            arguments.len()
                        ));
                    }
                    for argument in arguments {
                        if argument.validate(&numeric, &validate_predicate)? != ValueType::Number {
                            return Err(format!("procedure {name} requires numeric arguments"));
                        }
                    }
                }
                Effect::Sample {
                    stream,
                    value,
                    claim,
                    ..
                } => {
                    self.require_readings(stream)?;
                    self.require_kind(claim, "claim")?;
                    if value.validate(&numeric, &validate_predicate)? != ValueType::Number {
                        return Err(format!("sample {stream} requires a numeric expression"));
                    }
                }
                Effect::Reject { .. } => {}
                Effect::Emit { name } => {
                    if !self.cue_definitions.contains_key(name) {
                        return Err(format!("emit references undeclared cue {name}"));
                    }
                }
                Effect::Set {
                    name,
                    value,
                    because,
                } => {
                    if !self.values.contains_key(name) {
                        return Err(format!("set references undeclared state {name}"));
                    }
                    if value.validate(&numeric, &validate_predicate)? != ValueType::Number {
                        return Err(format!("state {name} requires a numeric expression"));
                    }
                    for citation in because.iter().flatten() {
                        citation
                            .validate(&numeric, &validate_predicate)
                            .map_err(|error| format!("set {name} because: {error}"))?;
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
                    match because {
                        EvidenceSelector::Named(name) => self.require_kind(name, "evidence")?,
                        EvidenceSelector::Latest(stream) => self.require_readings(stream)?,
                    }
                }
            }
        }
        let numeric = self
            .values
            .keys()
            .chain(self.constants.keys())
            .cloned()
            .collect();
        for (name, expression) in self.define_rules.iter() {
            if self.values.contains_key(name) || self.constants.contains_key(name) {
                return Err(format!("define {name} collides with a state"));
            }
            if self.reading_streams.contains_key(name) || self.decision_series.contains_key(name) {
                return Err(format!("define {name} collides with a history"));
            }
            if self
                .events
                .values()
                .flatten()
                .any(|parameter| parameter.name == *name)
            {
                return Err(format!("define {name} collides with an event parameter"));
            }
            expression
                .validate(&numeric, &validate_predicate)
                .map_err(|error| format!("define {name}: {error}"))?;
        }
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
            // A citation may be of any type; only its provenance is used.
            for citation in binding.because.iter().flatten() {
                citation
                    .validate(&numeric, &validate_predicate)
                    .map_err(|error| {
                        format!(
                            "binding {}.{} because: {error}",
                            binding.target, binding.property
                        )
                    })?;
            }
        }
        Ok(())
    }

    fn evaluate_bindings(&mut self) -> Result<(), String> {
        let mut bindings = BTreeMap::<String, BTreeMap<String, BindingValue>>::new();
        let mut data = BTreeMap::<String, BTreeMap<String, Provenance>>::new();
        let mut guards = BTreeMap::<String, BTreeMap<String, Provenance>>::new();
        // The declaration that supplied each shown value: the last one to match.
        let mut winners = BTreeMap::<(String, String), usize>::new();
        let parameters = BTreeMap::new();
        for (index, binding) in self.binding_rules.iter().enumerate() {
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
            winners.insert((binding.target.clone(), binding.property.clone()), index);
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
        let mut explanations = BTreeMap::<String, BTreeMap<String, Provenance>>::new();
        for ((target, property), index) in winners {
            let lineage = &data[&target][&property];
            let explanation = match &self.binding_rules[index].because {
                None => lineage.clone(),
                Some(citations) => self.grounded_citation(
                    &format!("binding {target}.{property}"),
                    citations,
                    lineage,
                    &parameters,
                )?,
            };
            explanations
                .entry(target)
                .or_default()
                .insert(property, explanation);
        }
        self.bindings = bindings;
        self.binding_qualifications = data;
        self.binding_explanations = explanations;
        Ok(())
    }

    /// Evaluate `because` citations and check that they cite only what the
    /// binding or state actually depended on. An explanation may leave
    /// dependencies out; it may never introduce one. Citations are read for
    /// their grounds, so citing a state cites what it is grounded on rather
    /// than every guard that ever touched it.
    fn grounded_citation(
        &self,
        subject: &str,
        citations: &[Expr],
        lineage: &Provenance,
        parameters: &BTreeMap<String, Tracked<f64>>,
    ) -> Result<Provenance, String> {
        let mut cited = Provenance::default();
        for citation in citations {
            let value = self
                .evaluate_grounds(citation, parameters)
                .map_err(|error| format!("{subject} because: {error}"))?;
            cited.merge(&value.provenance)?;
        }
        let ungrounded = cited
            .evidence
            .difference(&lineage.evidence)
            .map(|name| format!("evidence {name}"))
            .chain(
                cited
                    .caveats
                    .difference(&lineage.caveats)
                    .map(|name| format!("caveat {name}")),
            )
            .collect::<Vec<_>>();
        if !ungrounded.is_empty() {
            return Err(format!(
                "{subject} cites {} that its value and conditions never read",
                ungrounded.join(", ")
            ));
        }
        Ok(cited)
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

    fn require_readings(&self, name: &str) -> Result<(), String> {
        if self.reading_streams.contains_key(name) {
            Ok(())
        } else {
            Err(format!("{name} must name a declared reading stream"))
        }
    }

    fn require_history(&self, name: &str) -> Result<(), String> {
        if self.reading_streams.contains_key(name) || self.decision_series.contains_key(name) {
            Ok(())
        } else {
            Err(format!(
                "{name} must name a declared reading or decision history"
            ))
        }
    }

    fn history_read(&self, name: &str, query: HistoryRead) -> Result<Tracked<f64>, String> {
        if query == HistoryRead::Latest {
            return self.latest(name);
        }
        // Counting records is a qualified observation of membership. Include
        // every reached record's basis and guards for leaving history unchanged.
        // Indexed reads retain the selected record plus current selection guards.
        if let Some(stream) = self.reading_streams.get(name) {
            let mut provenance = stream.selection_qualifications.clone();
            let value = match query {
                HistoryRead::Count => {
                    for record in &stream.occurrences {
                        provenance.merge(&record.provenance)?;
                    }
                    stream.occurrences.len() as f64
                }
                HistoryRead::At(index) => {
                    let record = stream.occurrences.get(index).ok_or_else(|| {
                        format!("history index {index} is out of range for {name}")
                    })?;
                    provenance.merge(&record.provenance)?;
                    record.value
                }
                HistoryRead::Latest => unreachable!(),
            };
            return Tracked::new(value, provenance);
        }
        if let Some(series) = self.decision_series.get(name) {
            let mut provenance = series.selection_qualifications.clone();
            let value = match query {
                HistoryRead::Count => {
                    for revision in &series.revisions {
                        provenance.merge(&self.commitment_bases[&revision.id].provenance)?;
                    }
                    series.revisions.len() as f64
                }
                HistoryRead::At(index) => {
                    let revision = series.revisions.get(index).ok_or_else(|| {
                        format!("history index {index} is out of range for {name}")
                    })?;
                    let basis = &self.commitment_bases[&revision.id];
                    provenance.merge(&basis.provenance)?;
                    basis.value.ok_or_else(|| {
                        format!("decision {} has no numeric using value", revision.id)
                    })?
                }
                HistoryRead::Latest => unreachable!(),
            };
            return Tracked::new(value, provenance);
        }
        Err(format!("unknown history {name}"))
    }

    fn latest(&self, name: &str) -> Result<Tracked<f64>, String> {
        if let Some(stream) = self.reading_streams.get(name) {
            let reading = stream
                .occurrences
                .last()
                .ok_or_else(|| format!("reading stream {name} has no reached sample"))?;
            return Tracked::new(
                reading.value,
                reading.provenance.union(&stream.selection_qualifications)?,
            );
        }
        if let Some(series) = self.decision_series.get(name) {
            let current = series
                .current
                .as_ref()
                .ok_or_else(|| format!("decision series {name} has no commitment"))?;
            let basis = &self.commitment_bases[current];
            let value = basis
                .value
                .ok_or_else(|| format!("current decision {current} has no numeric using value"))?;
            return Tracked::new(
                value,
                basis.provenance.union(&series.selection_qualifications)?,
            );
        }
        Err(format!("unknown history {name}"))
    }

    fn retain_skipped_effect(
        &mut self,
        effect: &Effect,
        guard: &Provenance,
        budget: &mut ExecutionBudget,
    ) -> Result<(), String> {
        if let Effect::Call { name, .. } = effect {
            budget.enter()?;
            let procedures = Arc::clone(&self.procedures);
            for step in &procedures[name].body {
                budget.spend()?;
                self.retain_skipped_effect(&step.effect, guard, budget)?;
            }
            budget.depth -= 1;
            return Ok(());
        }
        if guard.is_empty() {
            return Ok(());
        }
        let target = match effect {
            Effect::Set { name, .. } => {
                return self
                    .values
                    .get_mut(name)
                    .expect("validated state")
                    .provenance
                    .merge(guard);
            }
            Effect::Sample { stream, .. } => {
                return self
                    .reading_streams
                    .get_mut(stream)
                    .expect("validated stream")
                    .selection_qualifications
                    .merge(guard);
            }
            Effect::Commit { action, .. } if self.decision_series.contains_key(action) => {
                return self
                    .decision_series
                    .get_mut(action)
                    .unwrap()
                    .selection_qualifications
                    .merge(guard);
            }
            Effect::Reveal { evidence, .. } => Some(("observed", evidence.clone())),
            Effect::Examine { caveat, .. } => Some(("examined", caveat.clone())),
            Effect::Commit { action, .. } => Some(("committed", action.clone())),
            Effect::Reopen { action, .. } => {
                let current = self
                    .decision_series
                    .get(action)
                    .and_then(|series| series.current.as_ref())
                    .unwrap_or(action);
                Some(("reopened", current.clone()))
            }
            _ => None,
        };
        if let Some((kind, name)) = target {
            self.predicate_qualifications
                .entry(kind.into())
                .or_default()
                .entry(name)
                .or_default()
                .merge(guard)?;
        }
        Ok(())
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
        next.sequence = next
            .sequence
            .checked_add(1)
            .ok_or("reactive event sequence exhausted")?;
        next.last_event = Some(event.into());
        let parameters = parameters
            .iter()
            .map(|(name, value)| (name.clone(), Tracked::plain(*value)))
            .collect();
        let mut budget = ExecutionBudget {
            remaining: MAX_EVENT_STEPS,
            depth: 0,
        };
        for (index, rule) in self
            .rules
            .iter()
            .enumerate()
            .filter(|(_, rule)| rule.event == event)
        {
            let result = next.execute_guarded_effect(
                &rule.condition,
                &rule.effect,
                &parameters,
                &Provenance::default(),
                &mut budget,
            );
            result.map_err(|error| format!("event {event}, rule {}: {error}", index + 1))?;
        }
        // Binding failures roll back the same numeric/graph/cue transaction.
        next.evaluate_bindings()?;
        let snapshot = next.snapshot();
        *self = next;
        Ok(snapshot)
    }

    fn execute_guarded_effect(
        &mut self,
        condition: &Expr,
        effect: &Effect,
        parameters: &BTreeMap<String, Tracked<f64>>,
        inherited: &Provenance,
        budget: &mut ExecutionBudget,
    ) -> Result<(), String> {
        budget.spend()?;
        let condition = self.evaluate(condition, parameters)?;
        let guard = inherited.union(&condition.provenance)?;
        if condition.value == Value::Bool(true) {
            self.apply_effect(effect, parameters, &guard, budget)
        } else {
            self.retain_skipped_effect(effect, &guard, budget)
        }
    }

    fn evaluate(
        &self,
        expression: &Expr,
        parameters: &BTreeMap<String, Tracked<f64>>,
    ) -> Result<Tracked<Value>, String> {
        expression.evaluate_tracked_with_histories(
            &|name| {
                Ok(self.values.get(name).cloned().or_else(|| {
                    parameters
                        .get(name)
                        .cloned()
                        .or_else(|| self.constants.get(name).copied().map(Tracked::plain))
                }))
            },
            &|kind, name| self.predicate_tracked(kind, name),
            &|evidence, caveats| self.qualify(evidence, caveats),
            &|name, query| self.history_read(name, query),
        )
    }

    /// Evaluate an expression for its grounds: the same value, with each read
    /// supplying content only. States give their grounds, `qualified` and
    /// `observed` give the evidence and the caveats that qualify it, and
    /// commitment predicates give the commitment's grounds. Guard, selection
    /// and absence dependencies stay in lineage. Every read here is a subset
    /// of the corresponding lineage read, so grounds are a subset of lineage.
    fn evaluate_grounds(
        &self,
        expression: &Expr,
        parameters: &BTreeMap<String, Tracked<f64>>,
    ) -> Result<Tracked<Value>, String> {
        expression.evaluate_tracked_with_histories(
            &|name| {
                if let Some(value) = self.values.get(name) {
                    let grounds = self.grounds.get(name).cloned().unwrap_or_default();
                    return Ok(Some(Tracked::new(value.value, grounds)?));
                }
                Ok(parameters
                    .get(name)
                    .cloned()
                    .or_else(|| self.constants.get(name).copied().map(Tracked::plain)))
            },
            &|kind, name| self.predicate_grounds(kind, name),
            &|evidence, caveats| self.qualify_core(evidence, caveats),
            &|name, query| self.history_read(name, query),
        )
    }

    fn predicate_grounds(&self, kind: &str, name: &str) -> Result<Tracked<bool>, String> {
        if matches!(kind, "committed" | "reopened") {
            if let Some(series) = self.decision_series.get(name) {
                return match &series.current {
                    Some(current) => self.predicate_grounds(kind, current),
                    None => Ok(Tracked::plain(false)),
                };
            }
        }
        let value = match kind {
            "has_sample" => return self.predicate_tracked(kind, name),
            _ => self.predicate(kind, name)?,
        };
        let provenance = match kind {
            "observed" if value => self.qualify_core(name, &[])?,
            "examined" => Provenance::from_names(
                [],
                std::iter::once(name.into()).chain(self.incoming_caveats([self.symbols[name]])),
            )?,
            "committed" | "reopened" if value => {
                let mut grounds = self
                    .commitment_grounds
                    .get(name)
                    .cloned()
                    .unwrap_or_default();
                if kind == "reopened" {
                    let id = self.symbols[name];
                    let by_id = self
                        .symbols
                        .iter()
                        .map(|(name, id)| (*id, name))
                        .collect::<HashMap<_, _>>();
                    for edge in &self.graph.edges {
                        if edge.to == id && edge.relation == Relation::Reopens {
                            grounds.merge(&self.qualify_core(by_id[&edge.from], &[])?)?;
                        }
                    }
                }
                grounds
            }
            _ => Provenance::default(),
        };
        Tracked::new(value, provenance)
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
        let mut provenance = self.qualify_core(evidence, extras)?;
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

    /// The evidence, the caveats that qualify it and the claims it bears on,
    /// and any explicit extras: what a qualified value is *about*, without the
    /// guard that revealed the evidence.
    fn qualify_core(&self, evidence: &str, extras: &[String]) -> Result<Provenance, String> {
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
        Provenance::from_names([evidence.into()], caveats)
    }

    fn predicate_tracked(&self, kind: &str, name: &str) -> Result<Tracked<bool>, String> {
        if kind == "has_sample" {
            self.require_readings(name)?;
            let stream = &self.reading_streams[name];
            let provenance = if stream.current.is_some() {
                self.latest(name)?.provenance
            } else {
                stream.selection_qualifications.clone()
            };
            return Tracked::new(stream.current.is_some(), provenance);
        }
        if matches!(kind, "committed" | "reopened") {
            if let Some(series) = self.decision_series.get(name) {
                let mut result = if let Some(current) = &series.current {
                    self.predicate_tracked(kind, current)?
                } else {
                    Tracked::plain(false)
                };
                result.provenance.merge(&series.selection_qualifications)?;
                if let Some(dependency) = self
                    .predicate_qualifications
                    .get(kind)
                    .and_then(|targets| targets.get(name))
                {
                    result.provenance.merge(dependency)?;
                }
                return Ok(result);
            }
        }
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
        parameters: &BTreeMap<String, Tracked<f64>>,
        guard: &Provenance,
        budget: &mut ExecutionBudget,
    ) -> Result<(), String> {
        match effect {
            Effect::Call { name, arguments } => {
                let procedures = Arc::clone(&self.procedures);
                let procedure = &procedures[name];
                let mut locals = BTreeMap::new();
                let mut inherited = guard.clone();
                // Freeze all arguments before the first body effect, including
                // unused arguments. Their qualifications cannot be laundered
                // by selecting a constant result or skipping a nested effect.
                for (parameter, argument) in procedure.parameters.iter().zip(arguments) {
                    let value = self.evaluate(argument, parameters)?;
                    let Value::Number(number) = value.value else {
                        return Err(format!("procedure {name} requires numeric arguments"));
                    };
                    inherited.merge(&value.provenance)?;
                    locals.insert(parameter.clone(), Tracked::new(number, value.provenance)?);
                }
                budget.enter()?;
                for (index, step) in procedure.body.iter().enumerate() {
                    self.execute_guarded_effect(
                        &step.condition,
                        &step.effect,
                        &locals,
                        &inherited,
                        budget,
                    )
                    .map_err(|error| format!("procedure {name}, step {}: {error}", index + 1))?;
                }
                budget.depth -= 1;
            }
            Effect::Sample {
                stream,
                value,
                relation,
                claim,
            } => {
                let readings = &self.reading_streams[stream];
                if readings.occurrences.len() >= readings.limit {
                    return Err(format!(
                        "reading stream {stream} reached its history limit {}",
                        readings.limit
                    ));
                }
                let ordinal = readings.occurrences.len() as u64 + 1;
                let name = format!("{stream}@{ordinal}");
                if self.symbols.contains_key(&name) {
                    return Err(format!("generated reading identity {name} already exists"));
                }
                let template = self.symbols[&readings.template];
                let value = self.evaluate(value, parameters)?;
                let Value::Number(number) = value.value else {
                    return Err("sample requires a numeric value".into());
                };
                let NodeKind::Evidence {
                    description,
                    source,
                } = self.graph.nodes[&template].clone()
                else {
                    unreachable!("validated evidence template")
                };
                let inherited = self
                    .graph
                    .edges
                    .iter()
                    .filter(|edge| edge.to == template && edge.relation == Relation::Qualifies)
                    .filter(|edge| {
                        matches!(
                            self.graph.nodes.get(&edge.from),
                            Some(NodeKind::Caveat { .. })
                        )
                    })
                    .map(|edge| edge.from)
                    .collect::<Vec<_>>();
                let id = self.graph.add(NodeKind::Evidence {
                    description: format!("{description} ({stream} reading {ordinal})"),
                    source,
                });
                self.symbols.insert(name.clone(), id);
                self.graph.relate(id, *relation, self.symbols[claim]);
                for caveat in inherited {
                    self.graph.relate(caveat, Relation::Qualifies, id);
                }
                self.observation_qualifications
                    .insert(name.clone(), value.provenance.union(guard)?);
                let provenance = self.qualify(&name, &[])?;
                let readings = self.reading_streams.get_mut(stream).unwrap();
                readings.current = Some(name.clone());
                readings.selection_qualifications = Provenance::default();
                readings.occurrences.push(ReadingOccurrence {
                    id: name.clone(),
                    ordinal,
                    sequence: self.sequence,
                    event: self
                        .last_event
                        .clone()
                        .expect("sampling occurs during dispatch"),
                    value: number,
                    provenance,
                    relation: relation_name(*relation).into(),
                    claim: claim.clone(),
                });
                self.effects.push(EffectReport::Sample {
                    stream: stream.clone(),
                    id: name,
                    value: number,
                    relation: relation_name(*relation).into(),
                    target: claim.clone(),
                });
            }
            Effect::Reject { message } => return Err(format!("rejected: {message}")),
            Effect::Emit { name } => {
                self.cues.push(self.cue_definitions[name].clone());
                self.cue_qualifications.push(guard.clone());
            }
            Effect::Set {
                name,
                value: expression,
                because,
            } => {
                let value = self.evaluate(expression, parameters)?;
                let Value::Number(number) = value.value else {
                    return Err("numeric state expression returned a boolean".into());
                };
                let range = &self.ranges[name];
                check_range(name, number, range.min, range.max)?;
                let lineage = value.provenance.union(guard)?;
                // Grounds are read against the state before this write, like
                // the value itself, and never take the rule's guard.
                let grounds = match because {
                    None => self.evaluate_grounds(expression, parameters)?.provenance,
                    Some(citations) => self.grounded_citation(
                        &format!("state {name}"),
                        citations,
                        &lineage,
                        parameters,
                    )?,
                };
                self.grounds.insert(name.clone(), grounds);
                self.values
                    .insert(name.clone(), Tracked::new(number, lineage)?);
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
                let previous = self
                    .decision_series
                    .get(action)
                    .and_then(|series| series.current.clone());
                if self.symbols.contains_key(action) {
                    return Err(format!("commitment {action} already exists"));
                }
                let mut provenance = guard.clone();
                let mut ordinal = None;
                if let Some(series) = self.decision_series.get(action) {
                    if series.revisions.len() >= series.limit {
                        return Err(format!(
                            "decision series {action} reached its history limit {}",
                            series.limit
                        ));
                    }
                    ordinal = Some(series.revisions.len() as u64 + 1);
                    if previous.is_some() {
                        let reopened = self.predicate_tracked("reopened", action)?;
                        if !reopened.value {
                            return Err(format!("current decision in {action} must be explicitly reopened before revision"));
                        }
                        provenance.merge(&reopened.provenance)?;
                    }
                }
                let name = ordinal
                    .map(|ordinal| format!("{action}@{ordinal}"))
                    .unwrap_or_else(|| action.clone());
                if self.symbols.contains_key(&name) {
                    return Err(format!(
                        "generated commitment identity {name} already exists"
                    ));
                }
                // Grounds: what the decision was made on, i.e. its `using`
                // content and retained caveats. The guard and a predecessor's
                // basis stay in the lineage recorded below.
                let mut grounds = Provenance::from_names([], retaining.iter().cloned())?;
                let used_value = if let Some(expression) = using {
                    let value = self.evaluate(expression, parameters)?;
                    let Value::Number(number) = value.value else {
                        return Err("commit using requires a numeric value".into());
                    };
                    provenance.merge(&value.provenance)?;
                    grounds.merge(&self.evaluate_grounds(expression, parameters)?.provenance)?;
                    Some(number)
                } else {
                    None
                };
                self.commitment_grounds.insert(name.clone(), grounds);
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
                let id = self.graph.commit_because(&name, &retained, reason.clone());
                self.clear_predicate_dependency("committed", action);
                if ordinal.is_some() {
                    self.clear_predicate_dependency("reopened", action);
                }
                self.symbols.insert(name.clone(), id);
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
                    name.clone(),
                    CommitmentBasis {
                        value: used_value,
                        provenance,
                    },
                );
                if let Some(ordinal) = ordinal {
                    let series = self.decision_series.get_mut(action).unwrap();
                    series.current = Some(name.clone());
                    series.selection_qualifications = Provenance::default();
                    series.revisions.push(DecisionRevision {
                        id: name.clone(),
                        previous,
                        ordinal,
                        sequence: self.sequence,
                        event: self
                            .last_event
                            .clone()
                            .expect("revision occurs during dispatch"),
                    });
                }
                self.effects.push(EffectReport::Commit {
                    action: name,
                    retained: retained_names,
                });
            }
            Effect::Reopen { action, because } => {
                let current = self
                    .decision_series
                    .get(action)
                    .and_then(|series| series.current.as_ref())
                    .unwrap_or(action)
                    .clone();
                let id = *self
                    .symbols
                    .get(&current)
                    .ok_or_else(|| format!("cannot reopen uncommitted action {action}"))?;
                let (because, cause) = match because {
                    EvidenceSelector::Named(name) => (name.clone(), self.qualify(name, &[])?),
                    EvidenceSelector::Latest(stream) => {
                        let reading = self.latest(stream)?;
                        let name = self.reading_streams[stream]
                            .current
                            .as_ref()
                            .expect("latest required an occurrence")
                            .clone();
                        (name, reading.provenance)
                    }
                };
                let from = self.symbols[&because];
                if !self.graph.edges.iter().any(|edge| {
                    edge.from == from && edge.to == id && edge.relation == Relation::Reopens
                }) {
                    let mut basis = cause.union(guard)?;
                    if let Some(series) = self.decision_series.get(action) {
                        basis.merge(&series.selection_qualifications)?;
                    }
                    self.clear_predicate_dependency("reopened", &current);
                    self.reopening_qualifications
                        .entry(current.clone())
                        .or_default()
                        .merge(&basis)?;
                    self.graph.reopen(id, from);
                    self.effects.push(EffectReport::Reopen {
                        action: current,
                        because,
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
                        retained_authorship: crate::map::retained_authorship(
                            &self.graph,
                            *id,
                            |node| names.get(&node).map(|name| (*name).to_string()),
                        ),
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
                written_by: self.graph.origin(*id).map(str::to_string),
                consequence,
                attention,
                display: self.labels.get(name).cloned(),
            });
        }
        ReactiveSnapshot {
            bindings: self.bindings.clone(),
            binding_qualifications: self.binding_qualifications.clone(),
            binding_explanations: self.binding_explanations.clone(),
            value_grounds: self.grounds.clone(),
            commitment_grounds: self.commitment_grounds.clone(),
            cues: self.cues.clone(),
            cue_qualifications: self.cue_qualifications.clone(),
            qualified_values: self.values.clone(),
            commitment_bases: self.commitment_bases.clone(),
            reading_streams: self.reading_streams.clone(),
            decision_series: self.decision_series.clone(),
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
    parse_directive_at(line, crate::parser::Position { line: 1, column: 1 })
}

pub(crate) fn parse_directive_at(
    line: &str,
    position: crate::parser::Position,
) -> Option<Result<Directive, String>> {
    let words = syntax_words(line);
    let keyword = words.first().copied()?;
    if !matches!(
        keyword,
        "state"
            | "event"
            | "on"
            | "fn"
            | "bind"
            | "cue"
            | "control"
            | "clock"
            | "readings"
            | "decisions"
            | "proc"
            | "define"
    ) {
        return None;
    }
    Some((|| match keyword {
        "proc" => parse_procedure(line, position),
        "define" => parse_define(line),
        "fn" => parse_function(line),
        "readings" => match words.as_slice() {
            ["readings", name, "from", template, "limit", limit] => Ok(Directive::Readings {
                name: identifier(name)?,
                template: identifier(template)?,
                limit: limit
                    .parse()
                    .map_err(|_| "reading limit must be an unsigned integer")?,
            }),
            _ => Err("readings expects NAME from EVIDENCE limit CAPACITY".into()),
        },
        "decisions" => match words.as_slice() {
            ["decisions", name, "limit", limit] => Ok(Directive::Decisions {
                name: identifier(name)?,
                limit: limit
                    .parse()
                    .map_err(|_| "decision limit must be an unsigned integer")?,
            }),
            _ => Err("decisions expects NAME limit CAPACITY".into()),
        },
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
            let GuardedEffect { condition, effect } = parse_guarded_effect(&words[2..])?;
            Ok(Directive::Rule(Rule {
                event,
                condition,
                effect,
            }))
        }
        _ => unreachable!(),
    })())
}

fn parse_guarded_effect(words: &[&str]) -> Result<GuardedEffect, String> {
    // Effect words are legal identifiers inside expressions. Select a complete
    // effect outside parentheses, rather than splitting at the first keyword.
    let mut depth = 0_i64;
    let mut candidates = Vec::new();
    for (index, token) in words.iter().enumerate() {
        if depth == 0
            && matches!(
                *token,
                "set"
                    | "reveal"
                    | "examine"
                    | "commit"
                    | "reopen"
                    | "emit"
                    | "sample"
                    | "call"
                    | "reject"
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
        .ok_or(
            "rule requires set, reveal, examine, commit, reopen, emit, sample, call, or reject effect",
        )?;
    let condition = if effect_index == 0 {
        reactive_expr::parse("true")?
    } else if words.first() == Some(&"when") {
        reactive_expr::parse_unresolved(&words[1..effect_index].join(" "))?
    } else {
        return Err("rule requires when CONDITION before its effect".into());
    };
    Ok(GuardedEffect {
        condition,
        effect: parse_effect(&words[effect_index..])?,
    })
}

fn parse_procedure(line: &str, mut position: crate::parser::Position) -> Result<Directive, String> {
    let open = line.find('{').ok_or("proc requires a braced body")?;
    let header = line[4..open].trim();
    let (name, parameters) = header
        .split_once('(')
        .ok_or("proc expects NAME(PARAMETERS) { EFFECTS }")?;
    let parameters = parameters
        .trim()
        .strip_suffix(')')
        .ok_or("proc parameter list requires closing parenthesis")?;
    let parameters = if parameters.trim().is_empty() {
        Vec::new()
    } else {
        parameters
            .split(',')
            .map(|name| identifier(name.trim()))
            .collect::<Result<_, _>>()?
    };
    let body = line[open + 1..]
        .trim_end()
        .strip_suffix('}')
        .ok_or("proc requires a closing brace")?;
    for ch in line[..open + 1].chars() {
        position.advance(ch);
    }
    let body = crate::parser::scan_statements_at(body, position)?
        .into_iter()
        .map(|(step, position)| {
            parse_guarded_effect(&syntax_words(step.trim())).map_err(|error| position.error(error))
        })
        .collect::<Result<_, _>>()?;
    Ok(Directive::Procedure(Procedure {
        name: identifier(name.trim())?,
        parameters,
        body,
    }))
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

/// Byte offsets of every `separator` character, and of every whitespace-
/// delimited `keyword`, that lies outside strings and expression parentheses.
fn top_level_positions(value: &str, separator: Option<char>, keyword: &str) -> Vec<usize> {
    let mut quoted = false;
    let mut escaped = false;
    let mut depth = 0_i64;
    let mut found = Vec::new();
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
        } else if depth == 0 && Some(ch) == separator {
            found.push(index);
        } else if depth == 0
            && !keyword.is_empty()
            && value[index..].starts_with(keyword)
            && index > 0
            && value[..index].ends_with(char::is_whitespace)
            && value[index + keyword.len()..]
                .chars()
                .next()
                .is_none_or(char::is_whitespace)
        {
            // A keyword ending the text still counts, so `bind x = 1 because`
            // reports a missing citation rather than a stray token.
            found.push(index);
        }
    }
    found
}

/// Split at the last top-level `keyword`: the text before it and, if the
/// keyword is present, the text after it.
fn trailing_clause<'a>(value: &'a str, keyword: &str) -> (&'a str, Option<&'a str>) {
    match top_level_positions(value, None, keyword).last() {
        Some(&index) => (
            value[..index].trim(),
            Some(value[index + keyword.len()..].trim()),
        ),
        None => (value.trim(), None),
    }
}

/// Locate a trailing `when` outside strings and expression parentheses.
fn binding_parts(value: &str) -> Result<(&str, Option<&str>), String> {
    let (expression, condition) = trailing_clause(value, "when");
    if condition.is_some_and(|condition| expression.is_empty() || condition.is_empty()) {
        return Err("bind requires a value and a condition after when".into());
    }
    Ok((expression, condition))
}

/// `because nothing`, or one or more comma-separated expressions whose
/// provenance a binding cites.
fn parse_citations(citations: &str) -> Result<Vec<Expr>, String> {
    if matches!(citations, "nothing") {
        return Ok(Vec::new());
    }
    let mut parsed = Vec::new();
    let mut start = 0;
    for end in top_level_positions(citations, Some(','), "")
        .into_iter()
        .chain([citations.len()])
    {
        let citation = citations[start..end].trim();
        if citation.is_empty() {
            return Err("because expects expressions separated by commas, or nothing".into());
        }
        parsed.push(reactive_expr::parse_unresolved(citation)?);
        start = end + 1;
    }
    Ok(parsed)
}

fn parse_define(line: &str) -> Result<Directive, String> {
    let (name, expression) = line
        .strip_prefix("define")
        .unwrap()
        .split_once('=')
        .ok_or("define expects NAME = EXPRESSION")?;
    Ok(Directive::Define {
        name: identifier(name.trim())?,
        expression: reactive_expr::parse_unresolved(expression.trim())?,
    })
}

fn parse_binding(line: &str) -> Result<Directive, String> {
    let (path, value) = line
        .strip_prefix("bind")
        .unwrap()
        .trim()
        .split_once('=')
        .ok_or("bind expects TARGET.PROPERTY = VALUE [when CONDITION] [because CITATIONS]")?;
    let (target, property) = path
        .trim()
        .split_once('.')
        .ok_or("bind requires TARGET.PROPERTY")?;
    let target = identifier(target)?;
    for segment in property.split('.') {
        identifier(segment)?;
    }
    // `because` comes last, so it is split off before looking for `when`.
    let (value, citations) = trailing_clause(value, "because");
    let because = match citations {
        Some("") => {
            return Err("because expects expressions separated by commas, or nothing".into())
        }
        Some(citations) => Some(parse_citations(citations)?),
        None => None,
    };
    let (value, condition) = binding_parts(value)?;
    let value = BindingExpression::Expression(reactive_expr::parse_unresolved(value)?);
    Ok(Directive::Binding(Binding {
        target,
        property: property.into(),
        value,
        condition: reactive_expr::parse_unresolved(condition.unwrap_or("true"))?,
        because,
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
        ["reject", rest @ ..] if !rest.is_empty() => {
            let text = rest.join(" ");
            let (message, trailing) = quoted_prefix(&text)?;
            if !trailing.is_empty() {
                return Err("reject expects one quoted message".into());
            }
            Ok(Effect::Reject { message })
        }
        ["call", rest @ ..] if !rest.is_empty() => {
            let (name, arguments) = reactive_expr::parse_procedure_call(&rest.join(" "))?;
            Ok(Effect::Call { name, arguments })
        }
        ["sample", stream, "=", rest @ ..] if rest.len() >= 3 => {
            let end = rest.len() - 2;
            let relation = match rest[end] {
                "supports" => Relation::Supports,
                "opposes" => Relation::Opposes,
                _ => return Err("sample requires supports CLAIM or opposes CLAIM".into()),
            };
            Ok(Effect::Sample {
                stream: identifier(stream)?,
                value: reactive_expr::parse_unresolved(&rest[..end].join(" "))?,
                relation,
                claim: identifier(rest[end + 1])?,
            })
        }
        ["emit", name] => Ok(Effect::Emit {
            name: identifier(name)?,
        }),
        ["set", name, "=", rest @ ..] if !rest.is_empty() => {
            let text = rest.join(" ");
            let (value, citations) = trailing_clause(&text, "because");
            let because = match citations {
                Some("") => {
                    return Err(
                        "because expects expressions separated by commas, or nothing".into(),
                    )
                }
                Some(citations) => Some(parse_citations(citations)?),
                None => None,
            };
            Ok(Effect::Set {
                name: identifier(name)?,
                value: reactive_expr::parse_unresolved(value)?,
                because,
            })
        }
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
        ["reopen", action, "because", selector @ ..] if !selector.is_empty() => {
            let selector = selector.join(" ");
            let because = if let Some(arguments) = selector
                .strip_prefix("latest")
                .map(str::trim)
                .and_then(|rest| rest.strip_prefix('('))
            {
                let stream = arguments
                    .trim()
                    .strip_suffix(')')
                    .ok_or("latest reading selector requires closing parenthesis")?;
                EvidenceSelector::Latest(identifier(stream.trim())?)
            } else {
                EvidenceSelector::Named(identifier(&selector)?)
            };
            Ok(Effect::Reopen {
                action: identifier(action)?,
                because,
            })
        }
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
