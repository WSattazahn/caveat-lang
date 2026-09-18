//! Bounded, deterministic event programs over the CAVEAT epistemic graph.
//!
//! The host dispatches declared inputs. Source owns numeric updates and graph
//! effects; an entire event publishes atomically or leaves the session intact.

use crate::ast::{Program, Statement};
use crate::eval::ResourceLedger;
use crate::game_session::GameSymbol;
use crate::map::{CaveatMap, MapBudget, MapCommitment, MapRelation, MapWorld};
use crate::presentation::Number;
use crate::reactive_expr::{self, Expr, Value, ValueType};
use crate::{Attention, EpistemicGraph, NodeId, NodeKind, Relation, StopReason};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

const LIMIT: f64 = 1_000_000_000_000.0;
pub const REACTIVE_SCHEMA: &str = "caveat-reactive/0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Parameter {
    pub name: String,
    pub min: Number,
    pub max: Number,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
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
    pub events: Vec<EventSignature>,
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
    values: BTreeMap<String, f64>,
    ranges: BTreeMap<String, StateRange>,
    constants: BTreeMap<String, f64>,
    events: BTreeMap<String, Vec<Parameter>>,
    rules: Arc<Vec<Rule>>,
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
            ranges: BTreeMap::new(),
            constants,
            events: BTreeMap::new(),
            rules: Arc::new(Vec::new()),
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
                    if initial.validate(&numeric, &|_, _| {
                        Err("state initializers cannot query live graph predicates".into())
                    })? != ValueType::Number
                    {
                        return Err(format!("state {name} initializer must be numeric"));
                    }
                    let value = initial.evaluate(
                        &|name| {
                            session
                                .values
                                .get(name)
                                .or_else(|| session.constants.get(name))
                                .copied()
                        },
                        &|_, _| Err("state initializer cannot query graph".into()),
                    )?;
                    let Value::Number(value) = value else {
                        unreachable!("validated number initializer")
                    };
                    check_range(name, value, min.value(), max.value())?;
                    session.values.insert(name.clone(), value);
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
        session.validate_rules()?;
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
                Effect::Commit { retaining, .. } => {
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
        for (index, rule) in self
            .rules
            .iter()
            .enumerate()
            .filter(|(_, rule)| rule.event == event)
        {
            let result = (|| {
                let condition = next.evaluate(&rule.condition, parameters)?;
                if condition == Value::Bool(true) {
                    next.apply_effect(&rule.effect, parameters)?;
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
        let snapshot = next.snapshot();
        *self = next;
        Ok(snapshot)
    }

    fn evaluate(
        &self,
        expression: &Expr,
        parameters: &BTreeMap<String, f64>,
    ) -> Result<Value, String> {
        expression.evaluate(
            &|name| {
                self.values
                    .get(name)
                    .or_else(|| parameters.get(name))
                    .or_else(|| self.constants.get(name))
                    .copied()
            },
            &|kind, name| self.predicate(kind, name),
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

    fn apply_effect(
        &mut self,
        effect: &Effect,
        parameters: &BTreeMap<String, f64>,
    ) -> Result<(), String> {
        match effect {
            Effect::Set { name, value } => {
                let Value::Number(value) = self.evaluate(value, parameters)? else {
                    return Err("numeric state expression returned a boolean".into());
                };
                let range = &self.ranges[name];
                check_range(name, value, range.min, range.max)?;
                self.values.insert(name.clone(), value);
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
                    self.graph.relate(from, *relation, to);
                    self.effects.push(EffectReport::Reveal {
                        evidence: evidence.clone(),
                        relation: relation_name(*relation).into(),
                        target: claim.clone(),
                    });
                }
            }
            Effect::Examine { caveat, cost } => {
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
                self.effects.push(EffectReport::Examine {
                    caveat: caveat.clone(),
                    cost: *cost,
                });
            }
            Effect::Commit {
                action,
                reason,
                retaining,
            } => {
                if self.symbols.contains_key(action) {
                    return Err(format!("commitment {action} already exists"));
                }
                let retained = retaining
                    .iter()
                    .map(|name| self.symbols[name])
                    .collect::<Vec<_>>();
                let id = self.graph.commit_because(action, &retained, reason.clone());
                self.symbols.insert(action.clone(), id);
                self.effects.push(EffectReport::Commit {
                    action: action.clone(),
                    retained: retaining.clone(),
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
            schema: REACTIVE_SCHEMA.into(),
            source_id: self.source_id.clone(),
            sequence: self.sequence,
            last_event: self.last_event.clone(),
            values: self.values.clone(),
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

pub fn parse_directive(line: &str) -> Option<Result<Directive, String>> {
    let words = line.split_whitespace().collect::<Vec<_>>();
    let keyword = words.first().copied()?;
    if !matches!(keyword, "state" | "event" | "on") {
        return None;
    }
    Some((|| match keyword {
        "state" => {
            let (name, initial) = line
                .trim_start_matches("state")
                .trim()
                .split_once('=')
                .ok_or("state declaration requires NAME = EXPRESSION")?;
            let tokens = initial.split_whitespace().collect::<Vec<_>>();
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
                initial: reactive_expr::parse(&tokens[..end].join(" "))?,
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
                    && matches!(*token, "set" | "reveal" | "examine" | "commit" | "reopen")
                {
                    candidates.push(index);
                }
                depth += token.chars().filter(|ch| *ch == '(').count() as i64;
                depth -= token.chars().filter(|ch| *ch == ')').count() as i64;
            }
            let effect_index = candidates
                .iter()
                .copied()
                .find(|index| parse_effect(&words[*index..]).is_ok())
                .or_else(|| candidates.last().copied())
                .ok_or("on rule requires set, reveal, examine, commit, or reopen effect")?;
            let condition = if effect_index == 2 {
                reactive_expr::parse("true")?
            } else if words.get(2) == Some(&"when") {
                reactive_expr::parse(&words[3..effect_index].join(" "))?
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

fn parse_effect(words: &[&str]) -> Result<Effect, String> {
    match words {
        ["set", name, "=", rest @ ..] if !rest.is_empty() => Ok(Effect::Set {
            name: identifier(name)?,
            value: reactive_expr::parse(&rest.join(" "))?,
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
            let retaining = match rest {
                [] => Vec::new(),
                ["retaining", names @ ..] if !names.is_empty() => names
                    .join(" ")
                    .split(',')
                    .map(|name| identifier(name.trim()))
                    .collect::<Result<_, _>>()?,
                _ => return Err("commit expects retaining CAVEAT, CAVEAT".into()),
            };
            Ok(Effect::Commit {
                action: identifier(action)?,
                reason,
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
