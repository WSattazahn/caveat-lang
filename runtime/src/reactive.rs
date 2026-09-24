//! Bounded, deterministic event programs over the CAVEAT epistemic graph.
//!
//! The host dispatches declared inputs. Source owns numeric updates and graph
//! effects; an entire event publishes atomically or leaves the session intact.

use crate::ast::{Program, Statement};
use crate::eval::ResourceLedger;
use crate::game_session::GameSymbol;
use crate::map::{CaveatMap, MapBudget, MapCommitment, MapRelation, MapWorld};
use crate::presentation::Number;
use crate::reactive_expr::{self, Expr, FunctionDef, HistoryRead, Reads, Value, ValueType};
pub use crate::reactive_expr::{Provenance, Tracked};
use crate::{Attention, Consequence, EpistemicGraph, NodeId, NodeKind, Relation, StopReason};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Arc;

const LIMIT: f64 = 1_000_000_000_000.0;
/// The caveat a withdrawal applies. See spec/caveat-withdrawal-0.1.md.
const WITHDRAWN: &str = "withdrawn";
const MAX_HISTORY_LIMIT: usize = 256;
const MAX_DECLARED_HISTORY_CAPACITY: usize = 1024;
const MAX_PROCEDURES: usize = 128;
const MAX_PROCEDURE_PARAMETERS: usize = 32;
const MAX_PROCEDURE_STEPS: usize = 4096;
const MAX_PROCEDURE_DEPTH: usize = 64;
const MAX_EVENT_STEPS: usize = 4096;
const MAX_RENEWAL_LIMIT: usize = 1024;
const MAX_SCHEDULED_QUALIFICATIONS: usize = 4096;
/// `carries(EVIDENCE, CAVEAT)` is a predicate on EVIDENCE named with this
/// prefix and the caveat, so it travels through every predicate path.
const CARRIES: &str = "carries:";
/// A predicate on a live state's grounds, not on an evidence alias.
const HAS_CAVEAT: &str = "has_caveat:";
pub const REACTIVE_SCHEMA: &str = "caveat-reactive/0.1";
pub const REACTIVE_VIEW_SCHEMA: &str = "caveat-reactive-view/0.1";
pub const REACTIVE_PRELUDE_SOURCE: &str = include_str!("../prelude.cav");

#[path = "reactive_save.rs"]
mod save;
pub use save::{
    Compact, ReactiveSave, SavedGraph, SavedNode, SavedResources, SavedState, REACTIVE_SAVE_SCHEMA,
};

#[path = "reactive_outcome.rs"]
mod outcome;

#[path = "reactive_identifiers.rs"]
mod identifiers;
use identifiers::{Identifiers, MAX_IDENTIFIER_LIMIT};
use outcome::DispatchFailure;
pub use outcome::{
    DispatchFatal, DispatchOutcome, DispatchResult, RejectionCode, RejectionOrigin, DISPATCH_SCHEMA,
};

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
    #[serde(skip_serializing_if = "ParameterDomain::is_numeric")]
    pub domain: ParameterDomain,
}

/// What a host may send for a parameter. See spec/caveat-typed-parameters-0.1.md.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterDomain {
    Numeric,
    /// `NAME kind KIND`: an entity of that kind, sent by name. Its value is the
    /// entity's position in the kind, counted from 1 as `$index` counts.
    Entity {
        kind: String,
        members: Vec<String>,
    },
    /// `NAME in A B C`: one of these names, sent as text. Its value is the
    /// name's position, counted from 1.
    Member {
        members: Vec<String>,
    },
    /// `NAME id`: any text, learned at run time. Its value is the text's
    /// handle in the session. See spec/caveat-identifiers-0.1.md.
    Identifier {
        limit: usize,
    },
}

impl ParameterDomain {
    fn is_numeric(&self) -> bool {
        matches!(self, Self::Numeric)
    }

    fn members(&self) -> &[String] {
        match self {
            Self::Numeric | Self::Identifier { .. } => &[],
            Self::Entity { members, .. } | Self::Member { members } => members,
        }
    }
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
    /// A caveat learned late: every current value that depends on the evidence
    /// gains it. Decisions already made keep what they were made on. See
    /// spec/caveat-late-qualification-0.1.md.
    Qualify {
        evidence: String,
        caveat: String,
        /// `after SECONDS`: apply once that much time has passed, to the
        /// occurrence current now. See spec/caveat-renewal-0.1.md.
        after: Option<Expr>,
    },
    /// Give renewable evidence a new occurrence: from now on its name means a
    /// new, unobserved piece of evidence. See spec/caveat-renewal-0.1.md.
    Renew {
        evidence: String,
    },
    /// `withdraw EVIDENCE because REASON;` or `withdraw latest(STREAM) …`:
    /// record that an observation is no longer stood behind. The target is
    /// `Named` or `Latest`. See spec/caveat-withdrawal-0.1.md.
    Withdraw {
        target: EvidenceSelector,
        because: String,
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
    /// Observed evidence in a state's grounds that currently bears a caveat.
    Caveated {
        state: String,
        caveat: String,
    },
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
    /// Numeric parameters, in order.
    pub parameters: Vec<String>,
    /// Parameters that name a graph symbol. A procedure with any is a
    /// template: each call is specialized with the names it passes. See
    /// spec/caveat-procedure-symbols-0.1.md.
    pub symbol_parameters: Vec<SymbolParameter>,
    pub body: Vec<GuardedEffect>,
}

/// `NAME evidence`, `NAME claim`, `NAME caveat`, `NAME readings` or
/// `NAME decisions` in a parameter list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolParameter {
    /// Its position among all the procedure's parameters, from 0.
    pub position: usize,
    pub name: String,
    pub kind: String,
}

/// How many procedures specialization may add, over the declared ones.
const MAX_SPECIALIZED_PROCEDURES: usize = 1024;

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
        }
        | Effect::Qualify {
            after: Some(value), ..
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
    fn spend(&mut self) -> Result<(), DispatchFailure> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            DispatchFailure::rejected(
                RejectionOrigin::Limit,
                RejectionCode::WorkLimit,
                format!("event exceeds work limit {MAX_EVENT_STEPS}"),
            )
        })?;
        Ok(())
    }

    fn enter(&mut self) -> Result<(), DispatchFailure> {
        if self.depth >= MAX_PROCEDURE_DEPTH {
            return Err(DispatchFailure::rejected(
                RejectionOrigin::Limit,
                RejectionCode::DepthLimit,
                format!("procedure exceeds call depth limit {MAX_PROCEDURE_DEPTH}"),
            ));
        }
        self.depth += 1;
        Ok(())
    }
}

#[cfg(test)]
mod dispatch_budget_tests {
    use super::*;

    #[test]
    fn defensive_depth_failure_is_classified_without_changing_legacy_text() {
        let mut budget = ExecutionBudget {
            remaining: MAX_EVENT_STEPS,
            depth: MAX_PROCEDURE_DEPTH,
        };
        let failure = budget.enter().unwrap_err();
        assert_eq!(
            failure.to_string(),
            format!("procedure exceeds call depth limit {MAX_PROCEDURE_DEPTH}")
        );
        let report = serde_json::to_value(failure.outcome().unwrap()).unwrap();
        assert_eq!(report["origin"], "limit");
        assert_eq!(report["code"], "depth_limit");
        assert_eq!(budget.depth, MAX_PROCEDURE_DEPTH);
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
    /// `renewable EVIDENCE limit N;`
    Renewable {
        evidence: String,
        limit: usize,
    },
    /// `identifiers limit N;`
    Identifiers {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitmentBasis {
    pub value: Option<f64>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadingStream {
    pub template: String,
    pub limit: usize,
    pub current: Option<String>,
    pub occurrences: Vec<ReadingOccurrence>,
    pub selection_qualifications: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionRevision {
    pub id: String,
    pub previous: Option<String>,
    pub ordinal: u64,
    pub sequence: u64,
    pub event: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionSeries {
    pub limit: usize,
    pub current: Option<String>,
    pub revisions: Vec<DecisionRevision>,
    pub selection_qualifications: Provenance,
}

/// Evidence that can be renewed, and every occurrence it has had. See
/// spec/caveat-renewal-0.1.md.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Renewal {
    pub limit: usize,
    /// First to current: the declared name, then `NAME@2`, `NAME@3`, ...
    pub occurrences: Vec<String>,
    /// The caveats declared on the evidence. Every occurrence inherits them.
    #[serde(skip)]
    template_caveats: Vec<NodeId>,
}

/// `qualify EVIDENCE with CAVEAT after SECONDS`, waiting for its time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduledQualification {
    /// The occurrence that was current when it was scheduled.
    pub evidence: String,
    pub caveat: String,
    /// The elapsed time when it was scheduled, and how long after that it applies.
    pub scheduled_at: f64,
    pub after: f64,
    /// The scheduling rule's guard and the delay's lineage, which join the
    /// lineage of what it qualifies.
    pub guard: Provenance,
}

#[derive(Debug, Clone, Default)]
struct LoadedGraph {
    last_node: NodeId,
    edges: usize,
    /// Every state as the program set it when it loaded; a save leaves out
    /// the states still equal to these.
    states: Arc<Vec<Arc<StateCell>>>,
}

#[derive(Debug, Clone)]
struct StateRange {
    min: f64,
    max: f64,
}

/// A state's current value, with its lineage, and what it is grounded on.
#[derive(Debug, Clone, PartialEq)]
struct StateCell {
    value: QualifiedValue,
    grounds: Provenance,
}

/// Every declared state, in a slot fixed at load. A transaction's copy shares
/// every cell until an effect writes one, so an event copies only the states
/// it changes, and comparing before with after checks pointers first.
#[derive(Debug, Clone, Default)]
struct States {
    slots: Arc<HashMap<String, usize>>,
    names: Arc<Vec<String>>,
    cells: Arc<Vec<Arc<StateCell>>>,
}

impl States {
    fn contains_key(&self, name: &str) -> bool {
        self.slots.contains_key(name)
    }

    fn slot(&self, name: &str) -> Option<usize> {
        self.slots.get(name).copied()
    }

    fn get(&self, name: &str) -> Option<&StateCell> {
        self.slot(name).map(|slot| &*self.cells[slot])
    }

    fn value(&self, name: &str) -> Option<&QualifiedValue> {
        self.get(name).map(|cell| &cell.value)
    }

    fn names(&self) -> impl Iterator<Item = &String> {
        self.names.iter()
    }

    fn len(&self) -> usize {
        self.cells.len()
    }

    /// Load time: a new state. The caller has checked the name is unused.
    fn declare(&mut self, name: String, cell: StateCell) {
        let slot = self.names.len();
        Arc::make_mut(&mut self.slots).insert(name.clone(), slot);
        Arc::make_mut(&mut self.names).push(name);
        Arc::make_mut(&mut self.cells).push(Arc::new(cell));
    }

    fn set(&mut self, slot: usize, cell: StateCell) {
        Arc::make_mut(&mut self.cells)[slot] = Arc::new(cell);
    }

    fn cell_mut(&mut self, slot: usize) -> &mut StateCell {
        Arc::make_mut(&mut Arc::make_mut(&mut self.cells)[slot])
    }

    /// The states whose value, lineage or grounds differ from `old`'s.
    fn changed_since(&self, old: &Self) -> HashSet<String> {
        if Arc::ptr_eq(&self.cells, &old.cells) {
            return HashSet::new();
        }
        self.cells
            .iter()
            .zip(old.cells.iter())
            .enumerate()
            .filter(|(_, (new, old))| !Arc::ptr_eq(new, old) && new != old)
            .map(|(slot, _)| self.names[slot].clone())
            .collect()
    }

    fn values(&self) -> BTreeMap<String, QualifiedValue> {
        self.names
            .iter()
            .zip(self.cells.iter())
            .map(|(name, cell)| (name.clone(), cell.value.clone()))
            .collect()
    }

    fn grounds(&self) -> BTreeMap<String, Provenance> {
        self.names
            .iter()
            .zip(self.cells.iter())
            .map(|(name, cell)| (name.clone(), cell.grounds.clone()))
            .collect()
    }
}

/// A step an event runs, as the observation-order check sees it.
struct OrderStep<'a> {
    place: String,
    /// The guards of the calls that led here.
    inherited: Vec<&'a Expr>,
    /// This step's own guard, split at `and`.
    own: Vec<&'a Expr>,
    effect: &'a Effect,
}

impl<'a> OrderStep<'a> {
    fn guard(&self) -> impl Iterator<Item = &'a Expr> + '_ {
        self.inherited.iter().chain(&self.own).copied()
    }

    fn revealed(&self) -> Option<String> {
        match self.effect {
            Effect::Reveal { evidence, .. } => Some(evidence.clone()),
            _ => None,
        }
    }

    fn reveals(&self, evidence: &str) -> bool {
        matches!(self.effect, Effect::Reveal { evidence: revealed, .. } if revealed == evidence)
    }

    /// Evidence this step certainly needs observed once it runs as far as the
    /// use, the guard already passed by then, and the use in words.
    fn uses(&self) -> Vec<(String, Vec<&'a Expr>, String)> {
        let mut uses = Vec::new();
        for (position, conjunct) in self.own.iter().enumerate() {
            let mut evidence = BTreeSet::new();
            conjunct.qualified_unconditionally(&mut evidence);
            let guard = self
                .inherited
                .iter()
                .chain(&self.own[..position])
                .copied()
                .collect::<Vec<_>>();
            for name in evidence {
                uses.push((name.clone(), guard.clone(), format!("qualified(…, {name})")));
            }
        }
        let guard = self.guard().collect::<Vec<_>>();
        let mut expressions = Vec::new();
        match self.effect {
            Effect::Set { value, because, .. } => {
                expressions.push(value);
                expressions.extend(because.iter().flatten());
            }
            Effect::Sample { value, .. }
            | Effect::Commit {
                using: Some(value), ..
            } => expressions.push(value),
            Effect::Call { arguments, .. } => expressions.extend(arguments),
            Effect::Qualify {
                evidence, after, ..
            } => {
                uses.push((
                    evidence.clone(),
                    guard.clone(),
                    format!("qualify {evidence}"),
                ));
                expressions.extend(after);
            }
            Effect::Reopen {
                because: EvidenceSelector::Named(evidence),
                ..
            } => uses.push((
                evidence.clone(),
                guard.clone(),
                format!("reopen because {evidence}"),
            )),
            Effect::Withdraw { target, because } => {
                if let EvidenceSelector::Named(evidence) = target {
                    uses.push((
                        evidence.clone(),
                        guard.clone(),
                        format!("withdraw {evidence}"),
                    ));
                }
                uses.push((
                    because.clone(),
                    guard.clone(),
                    format!("withdraw because {because}"),
                ));
            }
            _ => {}
        }
        for expression in expressions {
            let mut evidence = BTreeSet::new();
            expression.qualified_unconditionally(&mut evidence);
            for name in evidence {
                uses.push((name.clone(), guard.clone(), format!("qualified(…, {name})")));
            }
        }
        uses
    }
}

/// What a session shows: bindings, their lineage and what each cites.
type Shown = (
    BTreeMap<String, BTreeMap<String, BindingValue>>,
    BTreeMap<String, BTreeMap<String, Provenance>>,
    BTreeMap<String, BTreeMap<String, Provenance>>,
);

/// Every `bind` declaration for one `TARGET.PROPERTY`, and what any of them
/// can read. The property is shown from the last declaration that matches,
/// with every declaration's guard in its lineage, so the group is the unit
/// that is evaluated again or kept.
#[derive(Debug, Clone)]
struct BindingGroup {
    target: String,
    property: String,
    reads: Reads,
}

/// What one event changed that an expression can read: states whose value,
/// lineage or grounds differ, and whether anything the graph queries read
/// (graph, observation and predicate records, commitments, histories) did.
struct Changes {
    names: HashSet<String>,
    graph: bool,
    clock: bool,
}

impl Changes {
    fn between(old: &ReactiveSession, new: &ReactiveSession) -> Self {
        let names = new.states.changed_since(&old.states);
        // Copy-on-write fields an event never wrote still share their pointer.
        fn same<T: PartialEq>(old: &Arc<T>, new: &Arc<T>) -> bool {
            Arc::ptr_eq(old, new) || **old == **new
        }
        let graph = !(Arc::ptr_eq(&old.graph, &new.graph)
            || (old.graph.edges == new.graph.edges && old.graph.nodes == new.graph.nodes))
            || !same(&old.symbols, &new.symbols)
            || !same(
                &old.observation_qualifications,
                &new.observation_qualifications,
            )
            || !same(
                &old.examination_qualifications,
                &new.examination_qualifications,
            )
            || !same(&old.reopening_qualifications, &new.reopening_qualifications)
            || !same(&old.predicate_qualifications, &new.predicate_qualifications)
            || !same(&old.commitment_bases, &new.commitment_bases)
            || !same(&old.commitment_grounds, &new.commitment_grounds)
            || !same(&old.reading_streams, &new.reading_streams)
            || !same(&old.decision_series, &new.decision_series)
            || !same(&old.renewals, &new.renewals);
        Self {
            names,
            graph,
            // Signed zero is observable, for example through atan2.
            clock: old.elapsed.to_bits() != new.elapsed.to_bits(),
        }
    }

    fn touch(&self, reads: &Reads) -> bool {
        reads.anything
            || (reads.graph && self.graph)
            || (reads.clock && self.clock)
            || reads.names.iter().any(|name| self.names.contains(name))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    Qualify {
        evidence: String,
        caveat: String,
    },
    Renew {
        evidence: String,
        occurrence: String,
    },
    Withdraw {
        evidence: String,
        because: String,
    },
}

/// One withdrawn observation, resolved to its occurrence when it was
/// withdrawn. See spec/caveat-withdrawal-0.1.md.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Withdrawal {
    pub evidence: String,
    pub because: String,
    pub sequence: u64,
    pub event: String,
}

/// One change to a decision, in the order it happened. See
/// spec/caveat-decision-journal-0.1.md.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalEntry {
    /// The declared commitment or decision series.
    pub decision: String,
    /// The concrete commitment, `name@N` for a series revision.
    pub commitment: String,
    /// `committed` or `reopened`.
    pub change: String,
    pub sequence: u64,
    pub event: String,
    /// Source-controlled elapsed time at this change. Absent in older saves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed: Option<f64>,
    /// The commitment's frozen `using` value, if it had one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// For a commitment its grounds' evidence, for a reopening the evidence
    /// that reopened it; either way in the order it was first observed.
    pub because: Vec<String>,
    pub caveats: Vec<String>,
}

/// What a host redraws after an event: bindings and what they cite, cues and
/// effects, commitments and their grounds, decision series and live relations.
/// Static declarations and full lineage stay in `ReactiveSnapshot`.
///
/// It borrows from the session: a view is for serializing, and copying the
/// bindings for every event cost more than building the records.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReactiveView<'a> {
    pub schema: &'static str,
    pub sequence: u64,
    pub last_event: Option<&'a str>,
    pub bindings: &'a BTreeMap<String, BTreeMap<String, BindingValue>>,
    pub binding_explanations: &'a BTreeMap<String, BTreeMap<String, Provenance>>,
    pub cues: &'a [Cue],
    pub effects: &'a [EffectReport],
    pub commitments: Vec<MapCommitment>,
    pub commitment_grounds: &'a BTreeMap<String, Provenance>,
    pub decision_series: &'a BTreeMap<String, DecisionSeries>,
    pub decision_journal: &'a [JournalEntry],
    pub relations: Vec<MapRelation>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReactiveSnapshot {
    pub schema: String,
    pub source_id: String,
    pub sequence: u64,
    /// Seconds counted from the time event's `dt`.
    pub elapsed: f64,
    pub renewals: BTreeMap<String, Renewal>,
    /// Texts received for `id` parameters: the entry at index 0 has handle 1.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub identifiers: Vec<String>,
    /// Withdrawn observations, in the order they were withdrawn.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub withdrawals: Vec<Withdrawal>,
    pub scheduled_qualifications: Vec<ScheduledQualification>,
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
    pub decision_journal: Vec<JournalEntry>,
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
    states: States,
    // Copy-on-write: a transaction's copy shares these until an effect
    // changes one (`Arc::make_mut`), so an event pays for what it touches and
    // an untouched structure is recognised by pointer.
    commitment_bases: Arc<BTreeMap<String, CommitmentBasis>>,
    reading_streams: Arc<BTreeMap<String, ReadingStream>>,
    decision_series: Arc<BTreeMap<String, DecisionSeries>>,
    renewals: Arc<BTreeMap<String, Renewal>>,
    /// Texts received for `id` parameters, in handle order.
    identifiers: Arc<Identifiers>,
    /// Withdrawn observations, in the order they were withdrawn.
    withdrawals: Arc<Vec<Withdrawal>>,
    /// Qualifications waiting for time to pass, in the order scheduled.
    scheduled: Arc<Vec<ScheduledQualification>>,
    /// Seconds counted from the time event's `dt`.
    elapsed: f64,
    /// The event whose `dt` counts time: the clock's, or else `tick`.
    time_event: Option<Arc<str>>,
    /// The graph as the program declared it: events add nodes after
    /// `last_node` and edges after `edges`.
    loaded: LoadedGraph,
    observation_qualifications: Arc<BTreeMap<String, Provenance>>,
    examination_qualifications: Arc<BTreeMap<String, Provenance>>,
    reopening_qualifications: Arc<BTreeMap<String, Provenance>>,
    predicate_qualifications: Arc<BTreeMap<String, BTreeMap<String, Provenance>>>,
    // Fixed at load and shared by every transaction.
    ranges: Arc<BTreeMap<String, StateRange>>,
    constants: Arc<BTreeMap<String, f64>>,
    events: Arc<BTreeMap<String, Vec<Parameter>>>,
    rules: Arc<Vec<Rule>>,
    /// Each event's rules, as indexes into `rules` in source order.
    rules_by_event: Arc<HashMap<String, Vec<usize>>>,
    procedures: Arc<BTreeMap<String, Procedure>>,
    binding_rules: Arc<Vec<Binding>>,
    binding_groups: Arc<Vec<BindingGroup>>,
    /// For each of `binding_rules`, its index in `binding_groups`.
    binding_group_of: Arc<Vec<usize>>,
    define_rules: Arc<Vec<(String, Expr)>>,
    bindings: BTreeMap<String, BTreeMap<String, BindingValue>>,
    binding_qualifications: BTreeMap<String, BTreeMap<String, Provenance>>,
    binding_explanations: BTreeMap<String, BTreeMap<String, Provenance>>,
    commitment_grounds: Arc<BTreeMap<String, Provenance>>,
    journal: Arc<Vec<JournalEntry>>,
    cue_definitions: Arc<BTreeMap<String, Cue>>,
    cues: Vec<Cue>,
    cue_qualifications: Vec<Provenance>,
    controls: Arc<BTreeMap<String, Control>>,
    clock: Option<Clock>,
    graph: Arc<EpistemicGraph>,
    symbols: Arc<HashMap<String, NodeId>>,
    resources: Option<ResourceLedger>,
    world: Arc<MapWorld>,
    labels: Arc<BTreeMap<String, String>>,
    scenes: Arc<Vec<String>>,
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
            states: States::default(),
            commitment_bases: Arc::default(),
            reading_streams: Arc::default(),
            decision_series: Arc::default(),
            renewals: Arc::default(),
            identifiers: Arc::default(),
            withdrawals: Arc::default(),
            scheduled: Arc::default(),
            elapsed: 0.0,
            time_event: None,
            loaded: LoadedGraph::default(),
            observation_qualifications: Arc::default(),
            examination_qualifications: Arc::default(),
            reopening_qualifications: Arc::default(),
            predicate_qualifications: Arc::default(),
            ranges: Arc::new(BTreeMap::new()),
            constants: Arc::new(constants),
            events: Arc::new(BTreeMap::new()),
            rules: Arc::new(Vec::new()),
            rules_by_event: Arc::default(),
            procedures: Arc::new(BTreeMap::new()),
            binding_rules: Arc::new(Vec::new()),
            binding_groups: Arc::new(Vec::new()),
            binding_group_of: Arc::new(Vec::new()),
            define_rules: Arc::new(Vec::new()),
            bindings: BTreeMap::new(),
            binding_qualifications: BTreeMap::new(),
            binding_explanations: BTreeMap::new(),
            commitment_grounds: Arc::default(),
            journal: Arc::default(),
            cue_definitions: Arc::new(BTreeMap::new()),
            cues: Vec::new(),
            cue_qualifications: Vec::new(),
            controls: Arc::default(),
            clock: None,
            graph: Arc::new(evaluation.graph),
            symbols: Arc::new(evaluation.symbols),
            resources: evaluation.resources,
            world: Arc::new(map.world),
            labels: Arc::new(evaluation.display.into_iter().collect()),
            scenes: Arc::new(map.scenes),
            effects: Vec::new(),
        };
        let mut declared_identifiers = false;
        for directive in &directives {
            if let Directive::Identifiers { limit } = directive {
                if declared_identifiers {
                    return Err("identifiers is declared more than once".into());
                }
                if *limit == 0 || *limit > MAX_IDENTIFIER_LIMIT {
                    return Err(format!(
                        "identifiers limit must be in 1..{MAX_IDENTIFIER_LIMIT}"
                    ));
                }
                declared_identifiers = true;
                session.identifiers = Arc::new(Identifiers::declared(*limit));
            }
        }
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
                    Arc::make_mut(&mut session.reading_streams).insert(
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
                    Arc::make_mut(&mut session.decision_series).insert(
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
                | Directive::Decisions { .. }
                | Directive::Identifiers { .. } => {}
                Directive::Renewable { evidence, limit } => {
                    session.require_kind(evidence, "evidence")?;
                    if *limit == 0 || *limit > MAX_RENEWAL_LIMIT {
                        return Err(format!(
                            "renewable {evidence} limit must be in 1..{MAX_RENEWAL_LIMIT}"
                        ));
                    }
                    let id = session.symbols[evidence];
                    let template_caveats = session
                        .graph
                        .edges
                        .iter()
                        .filter(|edge| edge.to == id && edge.relation == Relation::Qualifies)
                        .filter(|edge| {
                            matches!(
                                session.graph.nodes.get(&edge.from),
                                Some(NodeKind::Caveat { .. })
                            )
                        })
                        .map(|edge| edge.from)
                        .collect();
                    let renewal = Renewal {
                        limit: *limit,
                        occurrences: vec![evidence.clone()],
                        template_caveats,
                    };
                    if Arc::make_mut(&mut session.renewals)
                        .insert(evidence.clone(), renewal)
                        .is_some()
                    {
                        return Err(format!("duplicate renewable {evidence}"));
                    }
                }
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
                    if Arc::make_mut(&mut session.controls)
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
                    if session.states.contains_key(name) {
                        return Err(format!("duplicate reactive state {name}"));
                    }
                    let numeric = session
                        .states
                        .names()
                        .chain(session.constants.keys())
                        .cloned()
                        .collect();
                    if initial.validate(&numeric, &|kind, name| match kind {
                        "runtime_clock" => Ok(()),
                        "qualification_evidence" => session.require_kind(name, "evidence"),
                        "qualification_caveat" => session.require_kind(name, "caveat"),
                        "numeric_history" => session.require_history(name),
                        "has_sample" => session.require_readings(name),
                        _ if kind.starts_with(HAS_CAVEAT) => session
                            .state_with_caveat(name, &kind[HAS_CAVEAT.len()..])
                            .map(|_| ()),
                        _ => Err("state initializers cannot query live graph predicates".into()),
                    })? != ValueType::Number
                    {
                        return Err(format!("state {name} initializer must be numeric"));
                    }
                    let value = initial.evaluate_tracked_with_histories(
                        &|name| {
                            if name == reactive_expr::ELAPSED_READ {
                                return Ok(Some(Tracked::plain(session.elapsed)));
                            }
                            Ok(session.states.value(name).cloned().or_else(|| {
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
                    session.states.declare(
                        name.clone(),
                        StateCell {
                            value: Tracked::new(number, value.provenance)?,
                            grounds,
                        },
                    );
                    Arc::make_mut(&mut session.ranges).insert(
                        name.clone(),
                        StateRange {
                            min: min.value(),
                            max: max.value(),
                        },
                    );
                }
                Directive::Event { name, parameters } => {
                    let mut parameters = parameters.clone();
                    for parameter in &mut parameters {
                        if let ParameterDomain::Entity { kind, members } = &mut parameter.domain {
                            *members = session
                                .world
                                .entities
                                .iter()
                                .filter(|entity| entity.kind == *kind)
                                .map(|entity| entity.id.clone())
                                .collect();
                            if members.is_empty() {
                                return Err(format!(
                                    "parameter {} of event {name}: no entity is declared kind {kind}",
                                    parameter.name
                                ));
                            }
                            parameter.max = Number::parse(&members.len().to_string())?;
                        }
                        if let ParameterDomain::Identifier { limit } = &mut parameter.domain {
                            *limit = session.identifiers.limit();
                            if *limit == 0 {
                                return Err(format!(
                                    "parameter {} of event {name} is an identifier: declare identifiers limit N",
                                    parameter.name
                                ));
                            }
                            parameter.max = Number::parse(&limit.to_string())?;
                        }
                        // PARAMETER.MEMBER names each value in source.
                        for (position, member) in parameter.domain.members().iter().enumerate() {
                            let constant = format!("{}.{member}", parameter.name);
                            let value = (position + 1) as f64;
                            if Arc::make_mut(&mut session.constants)
                                .insert(constant.clone(), value)
                                .is_some_and(|previous| previous != value)
                            {
                                return Err(format!("{constant} already names a different value"));
                            }
                        }
                    }
                    let parameters = &parameters;
                    if Arc::make_mut(&mut session.events)
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
        for (event, parameters) in session.events.iter() {
            for parameter in parameters {
                if session.states.contains_key(&parameter.name)
                    || session.constants.contains_key(&parameter.name)
                {
                    return Err(format!(
                        "event {event} parameter {} shadows state or a coordinate",
                        parameter.name
                    ));
                }
            }
        }
        for (name, control) in session.controls.iter() {
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
        session.time_event = session
            .clock
            .as_ref()
            .map(|clock| clock.event.clone())
            .or_else(|| {
                session
                    .events
                    .contains_key("tick")
                    .then(|| "tick".to_string())
            })
            .map(Arc::from);
        session.validate_procedures()?;
        session.specialize_procedures()?;
        session.declare_withdrawal_caveat()?;
        session.validate_rules()?;
        session.check_observation_order()?;
        session.group_bindings();
        let mut rules_by_event = HashMap::<String, Vec<usize>>::new();
        for (index, rule) in session.rules.iter().enumerate() {
            rules_by_event
                .entry(rule.event.clone())
                .or_default()
                .push(index);
        }
        session.rules_by_event = Arc::new(rules_by_event);
        session.loaded = LoadedGraph {
            last_node: session.graph.nodes.keys().copied().max().unwrap_or(0),
            edges: session.graph.edges.len(),
            states: Arc::clone(&session.states.cells),
        };
        session.evaluate_bindings(None)?;
        Ok(session)
    }

    /// Replace every call of a procedure with symbol parameters by a call of
    /// its specialization for the names passed, created once per distinct set
    /// of names, and drop the templates. Names come from the declared symbols,
    /// so this ends even for calls that pass names on; a cycle among the
    /// templates was already rejected by `validate_procedures`.
    fn specialize_procedures(&mut self) -> Result<(), String> {
        let (templates, plain): (BTreeMap<_, _>, BTreeMap<_, _>) = self
            .procedures
            .iter()
            .map(|(name, procedure)| (name.clone(), procedure.clone()))
            .partition(|(_, procedure)| !procedure.symbol_parameters.is_empty());
        if templates.is_empty() {
            return Ok(());
        }
        for template in templates.values() {
            let mut names = HashSet::new();
            for parameter in &template.symbol_parameters {
                if !names.insert(&parameter.name) || template.parameters.contains(&parameter.name) {
                    return Err(format!(
                        "duplicate parameter {} for procedure {}",
                        parameter.name, template.name
                    ));
                }
                if self.symbols.contains_key(&parameter.name)
                    || self.reading_streams.contains_key(&parameter.name)
                    || self.decision_series.contains_key(&parameter.name)
                    || self.states.contains_key(&parameter.name)
                    || self.constants.contains_key(&parameter.name)
                {
                    return Err(format!(
                        "procedure {} parameter {} shadows a declared name",
                        template.name, parameter.name
                    ));
                }
            }
        }
        let mut procedures = plain;
        let mut pending = procedures.keys().cloned().collect::<Vec<_>>();
        let mut rules = (*self.rules).clone();
        for rule in &mut rules {
            self.specialize_call(&mut rule.effect, &templates, &mut procedures, &mut pending)?;
        }
        while let Some(name) = pending.pop() {
            let mut body = procedures[&name].body.clone();
            for step in &mut body {
                self.specialize_call(&mut step.effect, &templates, &mut procedures, &mut pending)?;
            }
            procedures.get_mut(&name).expect("pending procedure").body = body;
        }
        self.rules = Arc::new(rules);
        self.procedures = Arc::new(procedures);
        Ok(())
    }

    fn specialize_call(
        &self,
        effect: &mut Effect,
        templates: &BTreeMap<String, Procedure>,
        procedures: &mut BTreeMap<String, Procedure>,
        pending: &mut Vec<String>,
    ) -> Result<(), String> {
        let Effect::Call { name, arguments } = effect else {
            return Ok(());
        };
        let Some(template) = templates.get(name) else {
            return Ok(());
        };
        let expected = template.parameters.len() + template.symbol_parameters.len();
        if arguments.len() != expected {
            return Err(format!(
                "procedure {name} expects {expected} arguments, got {}",
                arguments.len()
            ));
        }
        let mut names = HashMap::new();
        let mut passed = Vec::new();
        for parameter in &template.symbol_parameters {
            let symbol = arguments[parameter.position]
                .as_name()
                .filter(|symbol| match parameter.kind.as_str() {
                    "readings" => self.reading_streams.contains_key(*symbol),
                    "decisions" => self.decision_series.contains_key(*symbol),
                    kind => self.require_kind(symbol, kind).is_ok(),
                })
                .ok_or_else(|| {
                    let kind = match parameter.kind.as_str() {
                        "readings" => "reading stream",
                        "decisions" => "decision series",
                        kind => kind,
                    };
                    format!(
                        "procedure {name} parameter {} takes the name of a declared {kind}",
                        parameter.name
                    )
                })?;
            names.insert(parameter.name.clone(), symbol.to_string());
            passed.push(symbol.to_string());
        }
        let specialized = format!("{name}[{}]", passed.join(", "));
        if !procedures.contains_key(&specialized) {
            if procedures.len() >= self.procedures.len() + MAX_SPECIALIZED_PROCEDURES {
                return Err(format!(
                    "procedure specialization exceeds limit {MAX_SPECIALIZED_PROCEDURES}"
                ));
            }
            let body = template
                .body
                .iter()
                .map(|step| GuardedEffect {
                    condition: step.condition.rename_symbols(&names),
                    effect: rename_effect(&step.effect, &names),
                })
                .collect();
            procedures.insert(
                specialized.clone(),
                Procedure {
                    name: specialized.clone(),
                    parameters: template.parameters.clone(),
                    symbol_parameters: Vec::new(),
                    body,
                },
            );
            pending.push(specialized.clone());
        }
        let symbols = template
            .symbol_parameters
            .iter()
            .map(|parameter| parameter.position)
            .collect::<HashSet<_>>();
        *arguments = std::mem::take(arguments)
            .into_iter()
            .enumerate()
            .filter(|(position, _)| !symbols.contains(position))
            .map(|(_, argument)| argument)
            .collect();
        *name = specialized;
        Ok(())
    }

    /// A use of evidence that can only fail is an error when the program
    /// loads, rather than on the first event that reaches it. See
    /// spec/caveat-observation-order-0.1.md.
    fn check_observation_order(&self) -> Result<(), String> {
        let mut by_event = BTreeMap::<&str, Vec<OrderStep>>::new();
        for (index, rule) in self.rules.iter().enumerate() {
            self.flatten_steps(
                format!("rule {}", index + 1),
                Vec::new(),
                &rule.condition,
                &rule.effect,
                by_event.entry(rule.event.as_str()).or_default(),
            );
        }
        let revealed = by_event
            .iter()
            .map(|(event, steps)| {
                let evidence = steps
                    .iter()
                    .filter_map(OrderStep::revealed)
                    .collect::<HashSet<_>>();
                (*event, evidence)
            })
            .collect::<BTreeMap<_, _>>();
        for (event, steps) in &by_event {
            let parameters = self.events[*event]
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<HashSet<_>>();
            // Fixed throughout the rules: parameters, constants, or the clock
            // (which advances before any rule, never between rules).
            let fixed = |conjunct: &Expr| {
                let mut reads = Reads::default();
                conjunct.collect_reads(&mut reads);
                !reads.graph
                    && !reads.anything
                    && reads.names.iter().all(|name| {
                        parameters.contains(name.as_str()) || self.constants.contains_key(name)
                    })
            };
            for (position, step) in steps.iter().enumerate() {
                let uses = step.uses();
                for (evidence, guard, what) in &uses {
                    let revealed_elsewhere = revealed.iter().any(|(other, evidence_set)| {
                        other != event && evidence_set.contains(evidence)
                    });
                    if revealed_elsewhere
                        || self.predicate("observed", evidence).unwrap_or(true)
                        || steps[..position]
                            .iter()
                            .any(|earlier| earlier.reveals(evidence))
                        || step.guard().any(|conjunct| {
                            let mut reads = Reads::default();
                            conjunct.collect_reads(&mut reads);
                            reads.observed.contains(evidence)
                        })
                    {
                        continue;
                    }
                    let later = steps[position + 1..]
                        .iter()
                        .filter(|step| step.reveals(evidence))
                        .collect::<Vec<_>>();
                    let Some(first) = later.first() else {
                        return Err(format!(
                            "event {event}, {}: {what} needs {evidence} observed, but nothing observes it: no rule reveals it and it is not observed when the program loads",
                            step.place
                        ));
                    };
                    let certain = guard.iter().all(|conjunct| fixed(conjunct))
                        && later.iter().all(|reveal| {
                            guard
                                .iter()
                                .all(|conjunct| reveal.guard().any(|other| other == *conjunct))
                        });
                    if certain {
                        return Err(format!(
                            "event {event}, {}: {what} runs before {} reveals {evidence} in the same event, so it can only fail; reveal {evidence} first, or guard the rule with observed({evidence})",
                            step.place, first.place
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// One rule, or one step of a procedure it calls with the call's guard
    /// carried along, in the order an event runs them. A call's own step
    /// comes first: its arguments are evaluated before its body runs.
    fn flatten_steps<'a>(
        &'a self,
        place: String,
        inherited: Vec<&'a Expr>,
        condition: &'a Expr,
        effect: &'a Effect,
        steps: &mut Vec<OrderStep<'a>>,
    ) {
        let own = condition.conjuncts();
        let mut guard = inherited.clone();
        guard.extend(own.iter().copied());
        steps.push(OrderStep {
            place: place.clone(),
            inherited,
            own,
            effect,
        });
        if let Effect::Call { name, .. } = effect {
            for (index, step) in self.procedures[name].body.iter().enumerate() {
                self.flatten_steps(
                    format!("{place} (procedure {name}, step {})", index + 1),
                    guard.clone(),
                    &step.condition,
                    &step.effect,
                    steps,
                );
            }
        }
    }

    fn group_bindings(&mut self) {
        let mut groups = Vec::<BindingGroup>::new();
        let mut index_of = HashMap::<(String, String), usize>::new();
        let mut group_of = Vec::with_capacity(self.binding_rules.len());
        for binding in self.binding_rules.iter() {
            let key = (binding.target.clone(), binding.property.clone());
            let group = *index_of.entry(key).or_insert_with(|| {
                groups.push(BindingGroup {
                    target: binding.target.clone(),
                    property: binding.property.clone(),
                    reads: Reads::default(),
                });
                groups.len() - 1
            });
            let reads = &mut groups[group].reads;
            binding.condition.collect_reads(reads);
            if let BindingExpression::Expression(value) = &binding.value {
                value.collect_reads(reads);
            }
            for citation in binding.because.iter().flatten() {
                citation.collect_reads(reads);
            }
            group_of.push(group);
        }
        self.binding_groups = Arc::new(groups);
        self.binding_group_of = Arc::new(group_of);
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
                if self.states.contains_key(parameter) || self.constants.contains_key(parameter) {
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
                "runtime_clock" => Ok(()),
                "observed" => self.require_kind(symbol, "evidence"),
                "examined" => self.require_kind(symbol, "caveat"),
                "qualification_evidence" => self.require_kind(symbol, "evidence"),
                "qualification_caveat" => {
                    self.not_the_withdrawal_caveat(symbol, "qualified")?;
                    self.require_kind(symbol, "caveat")
                }
                "numeric_history" => self.require_history(symbol),
                "has_sample" | "withdrawn_latest" => self.require_readings(symbol),
                "withdrawn" => self.require_kind(symbol, "evidence"),
                "rests_on_withdrawn" if commitments.contains(symbol) => Ok(()),
                "rests_on_withdrawn" => Err(format!("unknown reactive commitment {symbol}")),
                "committed" | "reopened" if commitments.contains(symbol) => Ok(()),
                "committed" | "reopened" => Err(format!("unknown reactive commitment {symbol}")),
                _ if kind.starts_with(CARRIES) => {
                    self.require_kind(symbol, "evidence")?;
                    self.require_kind(&kind[CARRIES.len()..], "caveat")
                }
                _ if kind.starts_with(HAS_CAVEAT) => self
                    .state_with_caveat(symbol, &kind[HAS_CAVEAT.len()..])
                    .map(|_| ()),
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
                .states
                .names()
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
                Effect::Qualify {
                    evidence,
                    caveat,
                    after,
                } => {
                    self.require_kind(evidence, "evidence")?;
                    self.require_kind(caveat, "caveat")?;
                    self.not_the_withdrawal_caveat(caveat, "qualify")?;
                    if let Some(after) = after {
                        if self.time_event.is_none() {
                            return Err(format!(
                                "qualify {evidence} with {caveat} after: nothing counts time; declare a tick event or a clock"
                            ));
                        }
                        if after.validate(&numeric, &validate_predicate)? != ValueType::Number {
                            return Err("qualify after requires a number of seconds".into());
                        }
                    }
                }
                Effect::Renew { evidence } => {
                    if !self.renewals.contains_key(evidence) {
                        return Err(format!(
                            "renew {evidence}: declare it renewable first (renewable {evidence} limit N;)"
                        ));
                    }
                }
                Effect::Withdraw { target, because } => {
                    match target {
                        EvidenceSelector::Named(evidence) => {
                            self.require_kind(evidence, "evidence")?
                        }
                        EvidenceSelector::Latest(stream) => self.require_readings(stream)?,
                        EvidenceSelector::Caveated { .. } => {
                            return Err("withdraw names evidence or latest(STREAM)".into())
                        }
                    }
                    self.require_kind(because, "evidence")?;
                }
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
                    if !self.states.contains_key(name) {
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
                    self.not_the_withdrawal_caveat(caveat, "examine")?;
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
                        self.not_the_withdrawal_caveat(caveat, "retaining")?;
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
                        EvidenceSelector::Caveated { state, caveat } => {
                            self.state_with_caveat(state, caveat)?;
                        }
                    }
                }
            }
        }
        let numeric = self
            .states
            .names()
            .chain(self.constants.keys())
            .cloned()
            .collect();
        for (name, expression) in self.define_rules.iter() {
            if self.states.contains_key(name) || self.constants.contains_key(name) {
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
            // On its own a define may name any event parameter; each place it
            // is used is validated again with that place's parameters.
            let names = self
                .states
                .names()
                .chain(self.constants.keys())
                .chain(
                    self.events
                        .values()
                        .flatten()
                        .map(|parameter| &parameter.name),
                )
                .cloned()
                .collect();
            expression
                .validate(&names, &validate_predicate)
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

    /// Show every bound property: the last matching declaration's value, its
    /// lineage (that value's and every declaration's guard) and what it cites.
    ///
    /// With `changes`, a property whose declarations read nothing the event
    /// changed keeps what it showed, because evaluating it again would give
    /// the same result: expressions are deterministic in what they read. Only
    /// the rest are evaluated, so an event costs what it touches rather than
    /// the size of the program. Debug builds check this against a full
    /// evaluation after every event.
    ///
    /// Everything is computed before anything is written, and properties are
    /// evaluated in declaration order and explained in name order, so an error
    /// is the same one a full evaluation reports.
    fn evaluate_bindings(&mut self, changes: Option<&Changes>) -> Result<(), String> {
        let groups = Arc::clone(&self.binding_groups);
        let rules = Arc::clone(&self.binding_rules);
        let stale = groups
            .iter()
            .map(|group| changes.is_none_or(|changes| changes.touch(&group.reads)))
            .collect::<Vec<_>>();
        if !stale.contains(&true) {
            return Ok(());
        }
        let parameters = BTreeMap::new();
        let mut guards = vec![Provenance::default(); groups.len()];
        // The declaration that supplied each shown value: the last one to match.
        let mut winners = vec![None::<(usize, Tracked<BindingValue>)>; groups.len()];
        for (index, binding) in rules.iter().enumerate() {
            let group = self.binding_group_of[index];
            if !stale[group] {
                continue;
            }
            let context = || format!("binding {}.{}", binding.target, binding.property);
            let condition = self
                .evaluate(&binding.condition, &parameters)
                .map_err(|error| format!("{}: {error}", context()))?;
            guards[group].merge(&condition.provenance)?;
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
            winners[group] = Some((index, value));
        }
        let mut order = (0..groups.len())
            .filter(|&group| stale[group])
            .collect::<Vec<_>>();
        order.sort_by(|&a, &b| {
            (&groups[a].target, &groups[a].property).cmp(&(&groups[b].target, &groups[b].property))
        });
        let mut shown = Vec::with_capacity(order.len());
        for group in order {
            let (target, property) = (&groups[group].target, &groups[group].property);
            let guard = std::mem::take(&mut guards[group]);
            let Some((index, value)) = winners[group].take() else {
                shown.push((group, None, guard));
                continue;
            };
            let mut lineage = value.provenance;
            lineage.merge(&guard)?;
            let explanation = match &rules[index].because {
                None => lineage.clone(),
                Some(citations) => self.grounded_citation(
                    &format!("binding {target}.{property}"),
                    citations,
                    &lineage,
                    &parameters,
                )?,
            };
            shown.push((group, Some((value.value, explanation)), lineage));
        }
        for (group, value, lineage) in shown {
            let (target, property) = (&groups[group].target, &groups[group].property);
            self.binding_qualifications
                .entry(target.clone())
                .or_default()
                .insert(property.clone(), lineage);
            match value {
                Some((value, explanation)) => {
                    self.bindings
                        .entry(target.clone())
                        .or_default()
                        .insert(property.clone(), value);
                    self.binding_explanations
                        .entry(target.clone())
                        .or_default()
                        .insert(property.clone(), explanation);
                }
                None => {
                    remove_shown(&mut self.bindings, target, property);
                    remove_shown(&mut self.binding_explanations, target, property);
                }
            }
        }
        Ok(())
    }

    /// Debug builds: an incremental evaluation must show exactly what a full
    /// one shows, or fail with the same error.
    #[cfg(debug_assertions)]
    fn check_incremental_bindings(&self, incremental: &Result<(), String>) {
        let mut full = self.clone();
        let result = full.evaluate_bindings(None);
        assert_eq!(
            incremental, &result,
            "incremental and full binding evaluation disagree on the outcome"
        );
        if result.is_ok() {
            assert_eq!(self.bindings, full.bindings, "incremental bindings differ");
            assert_eq!(
                self.binding_qualifications, full.binding_qualifications,
                "incremental binding lineage differs"
            );
            assert_eq!(
                self.binding_explanations, full.binding_explanations,
                "incremental binding explanations differ"
            );
        }
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

    fn state_with_caveat(&self, name: &str, caveat: &str) -> Result<&StateCell, String> {
        let state = self
            .states
            .get(name)
            .ok_or_else(|| format!("unknown reactive state {name}"))?;
        self.require_kind(caveat, "caveat")?;
        Ok(state)
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
            return Tracked::new(value, self.with_current_withdrawals(provenance)?);
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
            return Tracked::new(value, self.with_current_withdrawals(provenance)?);
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
                self.with_current_withdrawals(
                    reading.provenance.union(&stream.selection_qualifications)?,
                )?,
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
                self.with_current_withdrawals(
                    basis.provenance.union(&series.selection_qualifications)?,
                )?,
            );
        }
        Err(format!("unknown history {name}"))
    }

    fn retain_skipped_effect(
        &mut self,
        effect: &Effect,
        guard: &Provenance,
        budget: &mut ExecutionBudget,
    ) -> Result<(), DispatchFailure> {
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
                let slot = self.states.slot(name).expect("validated state");
                // A guard already in the lineage changes nothing; writing it
                // anyway would copy the shared cell.
                let current = &self
                    .states
                    .get(name)
                    .expect("validated state")
                    .value
                    .provenance;
                if guard.evidence.is_subset(&current.evidence)
                    && guard.caveats.is_subset(&current.caveats)
                {
                    return Ok(());
                }
                return self
                    .states
                    .cell_mut(slot)
                    .value
                    .provenance
                    .merge(guard)
                    .map_err(Into::into);
            }
            Effect::Sample { stream, .. } => {
                return Arc::make_mut(&mut self.reading_streams)
                    .get_mut(stream)
                    .expect("validated stream")
                    .selection_qualifications
                    .merge(guard)
                    .map_err(Into::into);
            }
            Effect::Commit { action, .. } if self.decision_series.contains_key(action) => {
                return Arc::make_mut(&mut self.decision_series)
                    .get_mut(action)
                    .unwrap()
                    .selection_qualifications
                    .merge(guard)
                    .map_err(Into::into);
            }
            Effect::Reveal { evidence, .. } | Effect::Renew { evidence } => {
                Some(("observed", self.occurrence(evidence).to_string()))
            }
            Effect::Withdraw { target, .. } => self
                .withdrawal_target(target)
                .map(|occurrence| (WITHDRAWN, occurrence)),
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
            Arc::make_mut(&mut self.predicate_qualifications)
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
        let (payload, new_identifiers) = self
            .resolve_payload(event, payload_json)
            .map_err(|error| error.to_string())?;
        self.apply_classified(event, &payload, new_identifiers)
            .map_err(|error| error.to_string())?;
        Ok(self.snapshot())
    }

    /// Dispatch through the same transaction as the legacy API, preserving
    /// classified refusals as values. Unclassified errors are fatal: callers
    /// must discard the session on Err or an escaped panic/trap.
    pub fn dispatch_outcome_json(
        &mut self,
        event: &str,
        payload_json: &str,
    ) -> Result<DispatchOutcome, DispatchFatal> {
        let result =
            self.resolve_payload(event, payload_json)
                .and_then(|(payload, new_identifiers)| {
                    self.apply_classified(event, &payload, new_identifiers)
                });
        match result {
            Ok(()) => Ok(DispatchOutcome::accepted(self.snapshot())),
            Err(failure) => failure.outcome(),
        }
    }

    /// Parse a JSON payload and turn each typed parameter's name into its
    /// position. Numeric parameters take numbers; typed parameters take the
    /// name of an entity or member, or its position as a number; identifier
    /// parameters take text only, and become handles. Bounds and the exact
    /// parameter set are checked by `apply` as for any payload.
    ///
    /// Also returns the identifiers the session does not hold yet, in the
    /// order the event declares its parameters. Their handles follow the
    /// session's; `apply_classified` adds them inside the event's transaction.
    fn resolve_payload(
        &self,
        event: &str,
        payload_json: &str,
    ) -> Result<(BTreeMap<String, f64>, Vec<String>), DispatchFailure> {
        // Typed map parsing rejects duplicate fields instead of last-write wins.
        let payload: Payload = serde_json::from_str(payload_json).map_err(|error| {
            DispatchFailure::rejected(
                RejectionOrigin::Input,
                RejectionCode::PayloadInvalid,
                format!("invalid event payload: {error}"),
            )
        })?;
        let signature = self.events.get(event);
        let payload_invalid = |message: String| {
            DispatchFailure::rejected(
                RejectionOrigin::Input,
                RejectionCode::PayloadInvalid,
                message,
            )
        };
        // Identifier texts wait until the others resolve, so that handles are
        // given in declaration order whatever order the payload lists them in.
        let mut texts = BTreeMap::new();
        let mut resolved = payload
            .0
            .into_iter()
            .filter_map(|(name, value)| {
                let parameter = signature
                    .and_then(|parameters| parameters.iter().find(|p| p.name == name));
                let number = match (value, parameter.map(|p| &p.domain)) {
                    (PayloadValue::Text(text), Some(ParameterDomain::Identifier { .. })) => {
                        if let Err(message) = Identifiers::check_text(&text) {
                            return Some(Err(payload_invalid(format!(
                                "event {event} parameter {name}: {message}"
                            ))));
                        }
                        texts.insert(name, text);
                        return None;
                    }
                    (PayloadValue::Number(_), Some(ParameterDomain::Identifier { .. })) => {
                        return Some(Err(payload_invalid(format!(
                            "event {event} parameter {name} is an identifier: send its text"
                        ))))
                    }
                    (PayloadValue::Number(number), _) => number,
                    (PayloadValue::Text(text), Some(domain)) if !domain.is_numeric() => {
                        let Some(position) =
                            domain.members().iter().position(|member| *member == text)
                        else {
                            return Some(Err(payload_invalid(format!(
                                "event {event} parameter {name} does not accept {text}; expected one of {}",
                                domain.members().join(", ")
                            ))));
                        };
                        (position + 1) as f64
                    }
                    (PayloadValue::Text(_), _) => {
                        return Some(Err(payload_invalid(format!(
                            "event {event} parameter {name} expects a number"
                        ))))
                    }
                };
                Some(Ok((name, number)))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let mut new_identifiers: Vec<String> = Vec::new();
        for parameter in signature.into_iter().flatten() {
            let Some(text) = texts.remove(&parameter.name) else {
                continue;
            };
            let handle = match self.identifiers.handle(&text) {
                Some(handle) => handle,
                None => match new_identifiers.iter().position(|new| *new == text) {
                    Some(index) => self.identifiers.len() + index + 1,
                    None => {
                        new_identifiers.push(text);
                        self.identifiers.len() + new_identifiers.len()
                    }
                },
            };
            resolved.insert(parameter.name.clone(), handle as f64);
        }
        Ok((resolved, new_identifiers))
    }

    /// Run one event as a transaction. Any failure leaves the session as it was.
    /// An identifier parameter takes the handle of an identifier the session
    /// already holds: numbers cannot name a new one.
    pub fn apply(&mut self, event: &str, parameters: &BTreeMap<String, f64>) -> Result<(), String> {
        self.apply_classified(event, parameters, Vec::new())
            .map_err(|error| error.to_string())
    }

    /// `new_identifiers` are texts the session does not hold, whose handles
    /// follow its own; the event adds them only if it succeeds.
    fn apply_classified(
        &mut self,
        event: &str,
        parameters: &BTreeMap<String, f64>,
        new_identifiers: Vec<String>,
    ) -> Result<(), DispatchFailure> {
        let signature = self.events.get(event).ok_or_else(|| {
            DispatchFailure::rejected(
                RejectionOrigin::Input,
                RejectionCode::UnknownEvent,
                format!("undeclared event {event}"),
            )
        })?;
        if signature.len() != parameters.len()
            || parameters
                .keys()
                .any(|name| !signature.iter().any(|parameter| &parameter.name == name))
        {
            return Err(DispatchFailure::rejected(
                RejectionOrigin::Input,
                RejectionCode::PayloadInvalid,
                format!("event {event} requires exactly its declared parameters"),
            ));
        }
        let handles = self.identifiers.len() + new_identifiers.len();
        for parameter in signature {
            let value = parameters
                .get(&parameter.name)
                .ok_or_else(|| format!("missing event parameter {}", parameter.name))?;
            if let ParameterDomain::Identifier { .. } = parameter.domain {
                if value.fract() != 0.0 || *value < 1.0 || *value > handles as f64 {
                    return Err(DispatchFailure::rejected(
                        RejectionOrigin::Input,
                        RejectionCode::PayloadInvalid,
                        format!(
                            "event {event} parameter {} is not the handle of an identifier",
                            parameter.name
                        ),
                    ));
                }
                continue;
            }
            check_range(
                &parameter.name,
                *value,
                parameter.min.value(),
                parameter.max.value(),
            )
            .map_err(|message| {
                DispatchFailure::rejected(
                    RejectionOrigin::Input,
                    RejectionCode::BoundExceeded,
                    message,
                )
            })?;
        }
        // The shown bindings move into the transaction rather than being
        // copied: rules never read them, and evaluation writes them only after
        // it has succeeded, so a failed event hands them back untouched.
        if !self.identifiers.has_room_for(&new_identifiers) {
            return Err(DispatchFailure::rejected(
                RejectionOrigin::Limit,
                RejectionCode::IdentifierLimit,
                format!(
                    "event {event} would hold more identifiers than the limit {} or 1 MiB of text",
                    self.identifiers.limit()
                ),
            ));
        }
        let shown = self.take_shown();
        let mut next = self.clone();
        next.put_shown(shown);
        if !new_identifiers.is_empty() {
            let identifiers = Arc::make_mut(&mut next.identifiers);
            for text in new_identifiers {
                identifiers.push(text);
            }
        }
        match next.run_event(self, event, parameters) {
            Ok(()) => {
                *self = next;
                Ok(())
            }
            Err(error) => {
                self.put_shown(next.take_shown());
                Err(error)
            }
        }
    }

    /// The body of `apply`, run on the transaction's copy: `old` is the
    /// session as it was before the event.
    fn run_event(
        &mut self,
        old: &Self,
        event: &str,
        parameters: &BTreeMap<String, f64>,
    ) -> Result<(), DispatchFailure> {
        self.effects.clear();
        self.cues.clear();
        self.cue_qualifications.clear();
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or("reactive event sequence exhausted")?;
        self.last_event = Some(event.into());
        if self.time_event.as_deref() == Some(event) {
            self.elapsed += parameters.get("dt").copied().unwrap_or(0.0);
            self.apply_due_qualifications()?;
        }
        let parameters = parameters
            .iter()
            .map(|(name, value)| (name.clone(), Tracked::plain(*value)))
            .collect();
        let mut budget = ExecutionBudget {
            remaining: MAX_EVENT_STEPS,
            depth: 0,
        };
        let rules = Arc::clone(&self.rules);
        let indexes = Arc::clone(&self.rules_by_event);
        for &index in indexes.get(event).into_iter().flatten() {
            let rule = &rules[index];
            let result = self.execute_guarded_effect(
                &rule.condition,
                &rule.effect,
                &parameters,
                &Provenance::default(),
                &mut budget,
            );
            result.map_err(|error| error.context(format!("event {event}, rule {}", index + 1)))?;
        }
        // Binding failures roll back the same numeric/graph/cue transaction.
        let changes = Changes::between(old, self);
        let result = self.evaluate_bindings(Some(&changes));
        #[cfg(debug_assertions)]
        self.check_incremental_bindings(&result);
        result.map_err(Into::into)
    }

    fn take_shown(&mut self) -> Shown {
        (
            std::mem::take(&mut self.bindings),
            std::mem::take(&mut self.binding_qualifications),
            std::mem::take(&mut self.binding_explanations),
        )
    }

    fn put_shown(&mut self, (bindings, qualifications, explanations): Shown) {
        self.bindings = bindings;
        self.binding_qualifications = qualifications;
        self.binding_explanations = explanations;
    }

    /// Dispatch and return the full snapshot.
    pub fn dispatch(
        &mut self,
        event: &str,
        parameters: &BTreeMap<String, f64>,
    ) -> Result<ReactiveSnapshot, String> {
        self.apply(event, parameters)?;
        Ok(self.snapshot())
    }

    /// Dispatch and return the per-event view: what a host redraws after an
    /// event, without the static world, symbols and full lineage.
    pub fn dispatch_view_json(
        &mut self,
        event: &str,
        payload_json: &str,
    ) -> Result<ReactiveView<'_>, String> {
        let (payload, new_identifiers) = self
            .resolve_payload(event, payload_json)
            .map_err(|error| error.to_string())?;
        self.apply_classified(event, &payload, new_identifiers)
            .map_err(|error| error.to_string())?;
        Ok(self.view())
    }

    pub fn view(&self) -> ReactiveView<'_> {
        let names = self
            .symbols
            .iter()
            .map(|(name, id)| (*id, name.as_str()))
            .collect::<HashMap<_, _>>();
        ReactiveView {
            schema: REACTIVE_VIEW_SCHEMA,
            sequence: self.sequence,
            last_event: self.last_event.as_deref(),
            bindings: &self.bindings,
            binding_explanations: &self.binding_explanations,
            cues: &self.cues,
            effects: &self.effects,
            commitments: self.commitment_records(&names),
            commitment_grounds: &self.commitment_grounds,
            decision_series: &self.decision_series,
            decision_journal: &self.journal,
            relations: self.relation_records(&names),
        }
    }

    fn commitment_records(&self, names: &HashMap<NodeId, &str>) -> Vec<MapCommitment> {
        let mut sorted = self.symbols.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|(name, _)| *name);
        sorted
            .into_iter()
            .filter_map(|(name, id)| match &self.graph.nodes[id] {
                NodeKind::Commitment { open, .. } => Some(MapCommitment {
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
                }),
                _ => None,
            })
            .collect()
    }

    fn relation_records(&self, names: &HashMap<NodeId, &str>) -> Vec<MapRelation> {
        self.graph
            .edges
            .iter()
            .map(|edge| MapRelation {
                from: names[&edge.from].into(),
                to: names[&edge.to].into(),
                relation: relation_name(edge.relation).into(),
                origin: "live".into(),
            })
            .collect()
    }

    fn execute_guarded_effect(
        &mut self,
        condition: &Expr,
        effect: &Effect,
        parameters: &BTreeMap<String, Tracked<f64>>,
        inherited: &Provenance,
        budget: &mut ExecutionBudget,
    ) -> Result<(), DispatchFailure> {
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
        expression.evaluate_tracked_with_identifiers(
            &|name| {
                if name == reactive_expr::ELAPSED_READ {
                    return Ok(Some(Tracked::plain(self.elapsed)));
                }
                Ok(self.states.value(name).cloned().or_else(|| {
                    parameters
                        .get(name)
                        .cloned()
                        .or_else(|| self.constants.get(name).copied().map(Tracked::plain))
                }))
            },
            &|kind, name| self.predicate_tracked(kind, self.predicate_target(kind, name)),
            &|evidence, caveats| self.qualify(self.occurrence(evidence), caveats),
            &|name, query| self.history_read(name, query),
            &|handle| self.identifier_text(handle),
        )
    }

    /// `id_text(handle)`: the text of an identifier the session holds.
    fn identifier_text(&self, handle: f64) -> Result<String, String> {
        self.identifiers
            .text(handle)
            .map(str::to_owned)
            .ok_or_else(|| format!("id_text: {handle} is not the handle of an identifier"))
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
        expression.evaluate_tracked_with_identifiers(
            &|name| {
                if name == reactive_expr::ELAPSED_READ {
                    return Ok(Some(Tracked::plain(self.elapsed)));
                }
                if let Some(cell) = self.states.get(name) {
                    return Ok(Some(Tracked::new(cell.value.value, cell.grounds.clone())?));
                }
                Ok(parameters
                    .get(name)
                    .cloned()
                    .or_else(|| self.constants.get(name).copied().map(Tracked::plain)))
            },
            &|kind, name| self.predicate_grounds(kind, self.predicate_target(kind, name)),
            &|evidence, caveats| self.qualify_core(self.occurrence(evidence), caveats),
            &|name, query| self.history_read(name, query),
            &|handle| self.identifier_text(handle),
        )
    }

    fn predicate_grounds(&self, kind: &str, name: &str) -> Result<Tracked<bool>, String> {
        if let Some(result) = self.withdrawal_predicate(kind, name, true)? {
            return Ok(result);
        }
        if let Some(caveat) = kind.strip_prefix(HAS_CAVEAT) {
            let state = self.state_with_caveat(name, caveat)?;
            return Tracked::new(
                state.grounds.caveats.contains(caveat),
                state.grounds.clone(),
            );
        }
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
            _ if kind.starts_with(CARRIES) => {
                if self.predicate("observed", name)? {
                    self.qualify_core(name, &[])?
                } else {
                    Provenance::default()
                }
            }
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
            _ if kind.starts_with(CARRIES) => {
                self.predicate("observed", name)?
                    && self
                        .qualify_core(name, &[])?
                        .caveats
                        .contains(&kind[CARRIES.len()..])
            }
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

    /// Evidence names in the order each was first observed: the position of
    /// its first supporting or opposing relation in the graph.
    fn in_observation_order<'a>(&self, evidence: impl Iterator<Item = &'a String>) -> Vec<String> {
        let mut named = evidence
            .map(|name| {
                let id = self.symbols.get(name).copied();
                let rank = self
                    .graph
                    .edges
                    .iter()
                    .position(|edge| {
                        Some(edge.from) == id
                            && matches!(edge.relation, Relation::Supports | Relation::Opposes)
                    })
                    .unwrap_or(usize::MAX);
                (rank, name.clone())
            })
            .collect::<Vec<_>>();
        named.sort();
        named.into_iter().map(|(_, name)| name).collect()
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
        if let Some(mut result) = self.withdrawal_predicate(kind, name, false)? {
            if let Some(dependency) = self
                .predicate_qualifications
                .get(WITHDRAWN)
                .and_then(|targets| targets.get(name))
            {
                result.provenance.merge(dependency)?;
            }
            return Ok(result);
        }
        if let Some(caveat) = kind.strip_prefix(HAS_CAVEAT) {
            let state = self.state_with_caveat(name, caveat)?;
            return Tracked::new(
                state.grounds.caveats.contains(caveat),
                state.value.provenance.clone(),
            );
        }
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
            // Whether it carries the caveat or not, the answer rests on the
            // evidence and what qualifies it.
            _ if kind.starts_with(CARRIES) => self.predicate_tracked("observed", name)?.provenance,
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
        // Checked first: most calls have nothing to clear, and a write would
        // copy the shared map.
        if !self
            .predicate_qualifications
            .get(kind)
            .is_some_and(|targets| targets.contains_key(name))
        {
            return;
        }
        let dependencies = Arc::make_mut(&mut self.predicate_qualifications);
        let targets = dependencies.get_mut(kind).expect("checked above");
        targets.remove(name);
        if targets.is_empty() {
            dependencies.remove(kind);
        }
    }

    fn apply_effect(
        &mut self,
        effect: &Effect,
        parameters: &BTreeMap<String, Tracked<f64>>,
        guard: &Provenance,
        budget: &mut ExecutionBudget,
    ) -> Result<(), DispatchFailure> {
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
                        return Err(format!("procedure {name} requires numeric arguments").into());
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
                    .map_err(|error| {
                        error.context(format!("procedure {name}, step {}", index + 1))
                    })?;
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
                    return Err(DispatchFailure::rejected(
                        RejectionOrigin::Limit,
                        RejectionCode::HistoryLimit,
                        format!(
                            "reading stream {stream} reached its history limit {}",
                            readings.limit
                        ),
                    ));
                }
                let ordinal = readings.occurrences.len() as u64 + 1;
                let name = format!("{stream}@{ordinal}");
                if self.symbols.contains_key(&name) {
                    return Err(format!("generated reading identity {name} already exists").into());
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
                let id = Arc::make_mut(&mut self.graph).add(NodeKind::Evidence {
                    description: format!("{description} ({stream} reading {ordinal})"),
                    source,
                });
                Arc::make_mut(&mut self.symbols).insert(name.clone(), id);
                Arc::make_mut(&mut self.graph).relate(id, *relation, self.symbols[claim]);
                for caveat in inherited {
                    Arc::make_mut(&mut self.graph).relate(caveat, Relation::Qualifies, id);
                }
                Arc::make_mut(&mut self.observation_qualifications)
                    .insert(name.clone(), value.provenance.union(guard)?);
                let provenance = self.qualify(&name, &[])?;
                let readings = Arc::make_mut(&mut self.reading_streams)
                    .get_mut(stream)
                    .unwrap();
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
            Effect::Reject { message } => return Err(DispatchFailure::policy(message)),
            Effect::Qualify {
                evidence,
                caveat,
                after,
            } => {
                let occurrence = self.occurrence(evidence).to_string();
                if !self.predicate("observed", &occurrence)? {
                    return Err(format!("cannot qualify unobserved evidence {occurrence}").into());
                }
                match after {
                    None => self.apply_qualification(&occurrence, caveat, guard)?,
                    Some(expression) => {
                        let delay = self.evaluate(expression, parameters)?;
                        let Value::Number(after) = delay.value else {
                            return Err("qualify after requires a number of seconds".into());
                        };
                        if after < 0.0 {
                            return Err(
                                "qualify after requires a nonnegative number of seconds".into()
                            );
                        }
                        if self.scheduled.len() >= MAX_SCHEDULED_QUALIFICATIONS {
                            return Err(format!(
                                "scheduled qualifications exceed limit {MAX_SCHEDULED_QUALIFICATIONS}"
                            ).into());
                        }
                        let scheduled = ScheduledQualification {
                            evidence: occurrence,
                            caveat: caveat.clone(),
                            scheduled_at: self.elapsed,
                            after,
                            guard: guard.union(&delay.provenance)?,
                        };
                        Arc::make_mut(&mut self.scheduled).push(scheduled);
                    }
                }
            }
            Effect::Withdraw { target, because } => {
                let evidence = self.withdrawal_target(target).ok_or_else(|| {
                    format!("cannot withdraw {target:?}: its stream has no reading")
                })?;
                let because = self.occurrence(because).to_string();
                for (name, role) in [(&evidence, "withdraw"), (&because, "withdraw because")] {
                    if !self.predicate("observed", name)? {
                        return Err(format!("cannot {role} unobserved evidence {name}").into());
                    }
                }
                if !self
                    .withdrawals
                    .iter()
                    .any(|withdrawal| withdrawal.evidence == evidence)
                {
                    Arc::make_mut(&mut self.withdrawals).push(Withdrawal {
                        evidence: evidence.clone(),
                        because: because.clone(),
                        sequence: self.sequence,
                        event: self.last_event.clone().unwrap_or_default(),
                    });
                    self.apply_qualification(&evidence, WITHDRAWN, guard)?;
                    // apply_qualification reported a qualify: this is a withdrawal.
                    self.effects.pop();
                    self.effects
                        .push(EffectReport::Withdraw { evidence, because });
                }
            }
            Effect::Renew { evidence } => {
                let renewal = &self.renewals[evidence];
                if renewal.occurrences.len() >= renewal.limit {
                    return Err(format!(
                        "renewable {evidence} reached its limit {}",
                        renewal.limit
                    )
                    .into());
                }
                let ordinal = renewal.occurrences.len() + 1;
                let name = format!("{evidence}@{ordinal}");
                if self.symbols.contains_key(&name) {
                    return Err(format!("generated occurrence {name} already exists").into());
                }
                let template_caveats = renewal.template_caveats.clone();
                let NodeKind::Evidence {
                    description,
                    source,
                } = self.graph.nodes[&self.symbols[evidence]].clone()
                else {
                    unreachable!("validated renewable evidence")
                };
                let graph = Arc::make_mut(&mut self.graph);
                let id = graph.add(NodeKind::Evidence {
                    description: format!("{description} (occurrence {ordinal})"),
                    source,
                });
                for caveat in template_caveats {
                    graph.relate(caveat, Relation::Qualifies, id);
                }
                Arc::make_mut(&mut self.symbols).insert(name.clone(), id);
                // The occurrence exists because this rule chose to renew.
                if !guard.is_empty() {
                    Arc::make_mut(&mut self.observation_qualifications)
                        .insert(name.clone(), guard.clone());
                }
                Arc::make_mut(&mut self.renewals)
                    .get_mut(evidence)
                    .expect("validated renewable evidence")
                    .occurrences
                    .push(name.clone());
                self.effects.push(EffectReport::Renew {
                    evidence: evidence.clone(),
                    occurrence: name,
                });
            }
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
                check_range(name, number, range.min, range.max).map_err(|message| {
                    DispatchFailure::rejected(
                        RejectionOrigin::Evaluation,
                        RejectionCode::BoundExceeded,
                        message,
                    )
                })?;
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
                let slot = self.states.slot(name).expect("validated state");
                self.states.set(
                    slot,
                    StateCell {
                        value: Tracked::new(number, lineage)?,
                        grounds,
                    },
                );
            }
            Effect::Reveal {
                evidence,
                relation,
                claim,
            } => {
                let evidence = &self.occurrence(evidence).to_string();
                let from = self.symbols[evidence];
                let to = self.symbols[claim];
                if !self
                    .graph
                    .edges
                    .iter()
                    .any(|edge| edge.from == from && edge.to == to && edge.relation == *relation)
                {
                    self.clear_predicate_dependency("observed", evidence);
                    Arc::make_mut(&mut self.observation_qualifications)
                        .entry(evidence.clone())
                        .or_default()
                        .merge(guard)?;
                    Arc::make_mut(&mut self.graph).relate(from, *relation, to);
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
                Arc::make_mut(&mut self.graph)
                    .set_attention(self.symbols[caveat], Attention::Examined);
                self.clear_predicate_dependency("examined", caveat);
                Arc::make_mut(&mut self.examination_qualifications)
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
                    return Err(format!("commitment {action} already exists").into());
                }
                let mut provenance = guard.clone();
                let mut ordinal = None;
                if let Some(series) = self.decision_series.get(action) {
                    if series.revisions.len() >= series.limit {
                        return Err(DispatchFailure::rejected(
                            RejectionOrigin::Limit,
                            RejectionCode::HistoryLimit,
                            format!(
                                "decision series {action} reached its history limit {}",
                                series.limit
                            ),
                        ));
                    }
                    ordinal = Some(series.revisions.len() as u64 + 1);
                    if previous.is_some() {
                        let reopened = self.predicate_tracked("reopened", action)?;
                        if !reopened.value {
                            return Err(format!("current decision in {action} must be explicitly reopened before revision").into());
                        }
                        provenance.merge(&reopened.provenance)?;
                    }
                }
                let name = ordinal
                    .map(|ordinal| format!("{action}@{ordinal}"))
                    .unwrap_or_else(|| action.clone());
                if self.symbols.contains_key(&name) {
                    return Err(
                        format!("generated commitment identity {name} already exists").into(),
                    );
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
                Arc::make_mut(&mut self.commitment_grounds).insert(name.clone(), grounds);
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
                let id =
                    Arc::make_mut(&mut self.graph).commit_because(&name, &retained, reason.clone());
                self.clear_predicate_dependency("committed", action);
                if ordinal.is_some() {
                    self.clear_predicate_dependency("reopened", action);
                }
                Arc::make_mut(&mut self.symbols).insert(name.clone(), id);
                for evidence in &provenance.evidence {
                    self.require_kind(evidence, "evidence")?;
                    if !self.predicate("observed", evidence)? {
                        return Err(format!(
                            "commitment basis includes unobserved evidence {evidence}"
                        )
                        .into());
                    }
                    Arc::make_mut(&mut self.graph).relate(
                        id,
                        Relation::ReliesOn,
                        self.symbols[evidence],
                    );
                }
                Arc::make_mut(&mut self.commitment_bases).insert(
                    name.clone(),
                    CommitmentBasis {
                        value: used_value,
                        provenance,
                    },
                );
                if let Some(ordinal) = ordinal {
                    let series = Arc::make_mut(&mut self.decision_series)
                        .get_mut(action)
                        .unwrap();
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
                let grounds = &self.commitment_grounds[&name];
                let entry = JournalEntry {
                    decision: action.clone(),
                    commitment: name.clone(),
                    change: "committed".into(),
                    sequence: self.sequence,
                    event: self.last_event.clone().unwrap_or_default(),
                    elapsed: Some(self.elapsed),
                    value: used_value,
                    because: self.in_observation_order(grounds.evidence.iter()),
                    caveats: grounds.caveats.iter().cloned().collect(),
                };
                Arc::make_mut(&mut self.journal).push(entry);
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
                    EvidenceSelector::Named(name) => {
                        let name = self.occurrence(name).to_string();
                        let cause = self.qualify(&name, &[])?;
                        (vec![name], cause)
                    }
                    EvidenceSelector::Latest(stream) => {
                        let reading = self.latest(stream)?;
                        let name = self.reading_streams[stream]
                            .current
                            .as_ref()
                            .expect("latest required an occurrence")
                            .clone();
                        (vec![name], reading.provenance)
                    }
                    EvidenceSelector::Caveated { state, caveat } => {
                        let cell = self.state_with_caveat(state, caveat)?;
                        let mut selected = Vec::new();
                        let mut cause = cell.value.provenance.clone();
                        for evidence in &cell.grounds.evidence {
                            // These are concrete occurrence identities, not
                            // renewable aliases. Never resolve them again.
                            if self.predicate("observed", evidence)?
                                && self.qualify_core(evidence, &[])?.caveats.contains(caveat)
                            {
                                selected.push(evidence);
                                cause.merge(&self.qualify(evidence, &[])?)?;
                            }
                        }
                        if selected.is_empty() {
                            return Err(format!(
                                "caveated({state}, {caveat}) selects no observed evidence carrying {caveat} from the state's grounds"
                            ).into());
                        }
                        (self.in_observation_order(selected.into_iter()), cause)
                    }
                };
                let because = because
                    .into_iter()
                    .filter(|evidence| {
                        let from = self.symbols[evidence];
                        !self.graph.edges.iter().any(|edge| {
                            edge.from == from && edge.to == id && edge.relation == Relation::Reopens
                        })
                    })
                    .collect::<Vec<_>>();
                if !because.is_empty() {
                    let mut basis = cause.union(guard)?;
                    if let Some(series) = self.decision_series.get(action) {
                        basis.merge(&series.selection_qualifications)?;
                    }
                    self.clear_predicate_dependency("reopened", &current);
                    Arc::make_mut(&mut self.reopening_qualifications)
                        .entry(current.clone())
                        .or_default()
                        .merge(&basis)?;
                    let mut caveats = BTreeSet::new();
                    for evidence in &because {
                        caveats.extend(self.qualify_core(evidence, &[])?.caveats);
                        Arc::make_mut(&mut self.graph).reopen(id, self.symbols[evidence]);
                        self.effects.push(EffectReport::Reopen {
                            action: current.clone(),
                            because: evidence.clone(),
                        });
                    }
                    let entry = JournalEntry {
                        decision: action.clone(),
                        commitment: current.clone(),
                        change: "reopened".into(),
                        sequence: self.sequence,
                        event: self.last_event.clone().unwrap_or_default(),
                        elapsed: Some(self.elapsed),
                        value: self
                            .commitment_bases
                            .get(&current)
                            .and_then(|basis| basis.value),
                        because,
                        caveats: caveats.into_iter().collect(),
                    };
                    Arc::make_mut(&mut self.journal).push(entry);
                }
            }
        }
        Ok(())
    }

    /// A caveat learned late, applied to one occurrence of evidence: see
    /// spec/caveat-late-qualification-0.1.md.
    fn apply_qualification(
        &mut self,
        evidence: &str,
        caveat: &str,
        guard: &Provenance,
    ) -> Result<(), String> {
        let (from, to) = (self.symbols[caveat], self.symbols[evidence]);
        if !self
            .graph
            .edges
            .iter()
            .any(|edge| edge.from == from && edge.to == to && edge.relation == Relation::Qualifies)
        {
            Arc::make_mut(&mut self.graph).relate(from, Relation::Qualifies, to);
        }
        // The caveat and whatever qualifies it, as `qualified` inherits.
        let added = Provenance::from_names(
            [],
            std::iter::once(caveat.to_string()).chain(self.incoming_caveats([from])),
        )?;
        // Current values only. Commitment bases and grounds, reading
        // archives and the journal record what was known then.
        for slot in 0..self.states.len() {
            let cell = &self.states.cells[slot];
            let in_value = cell.value.provenance.evidence.contains(evidence);
            let in_grounds = cell.grounds.evidence.contains(evidence);
            if !(in_value || in_grounds) {
                continue;
            }
            let cell = self.states.cell_mut(slot);
            if in_value {
                cell.value.provenance.merge(&added)?;
                cell.value.provenance.merge(guard)?;
            }
            if in_grounds {
                cell.grounds.merge(&added)?;
            }
        }
        self.effects.push(EffectReport::Qualify {
            evidence: evidence.to_string(),
            caveat: caveat.to_string(),
        });
        Ok(())
    }

    /// Apply every scheduled qualification whose time has come, in the order
    /// they were scheduled.
    fn apply_due_qualifications(&mut self) -> Result<(), String> {
        let elapsed = self.elapsed;
        let due = |scheduled: &ScheduledQualification| {
            elapsed - scheduled.scheduled_at >= scheduled.after
        };
        if !self.scheduled.iter().any(due) {
            return Ok(());
        }
        let (now, later): (Vec<_>, Vec<_>) = self.scheduled.iter().cloned().partition(due);
        self.scheduled = Arc::new(later);
        for scheduled in now {
            self.apply_qualification(&scheduled.evidence, &scheduled.caveat, &scheduled.guard)?;
        }
        Ok(())
    }

    /// What a source name means now: the current occurrence of renewable
    /// evidence, and the name itself for everything else.
    fn occurrence<'a>(&'a self, name: &'a str) -> &'a str {
        self.renewals
            .get(name)
            .and_then(|renewal| renewal.occurrences.last())
            .map_or(name, String::as_str)
    }

    /// A predicate's target as the source names it, resolved: predicates on
    /// evidence mean its current occurrence.
    /// A program that withdraws has the built-in caveat `withdrawn`, and may
    /// not declare, apply or examine it any other way.
    fn declare_withdrawal_caveat(&mut self) -> Result<(), String> {
        let withdraws = self
            .rules
            .iter()
            .map(|rule| &rule.effect)
            .chain(
                self.procedures
                    .values()
                    .flat_map(|procedure| procedure.body.iter().map(|step| &step.effect)),
            )
            .any(|effect| matches!(effect, Effect::Withdraw { .. }));
        if !withdraws {
            return Ok(());
        }
        if self.symbols.contains_key(WITHDRAWN)
            || self.states.contains_key(WITHDRAWN)
            || self.constants.contains_key(WITHDRAWN)
            || self.reading_streams.contains_key(WITHDRAWN)
            || self.decision_series.contains_key(WITHDRAWN)
        {
            return Err(
                "withdrawn is the caveat a withdrawal applies; a program that withdraws cannot declare it"
                    .into(),
            );
        }
        let graph = Arc::make_mut(&mut self.graph);
        let id = graph.add_caveat(
            "withdrawn: the observation is no longer stood behind",
            Consequence::Material,
        );
        Arc::make_mut(&mut self.symbols).insert(WITHDRAWN.into(), id);
        Ok(())
    }

    fn not_the_withdrawal_caveat(&self, caveat: &str, place: &str) -> Result<(), String> {
        if caveat == WITHDRAWN && self.withdraws() {
            return Err(format!(
                "{place} cannot apply withdrawn: only withdraw applies it"
            ));
        }
        Ok(())
    }

    /// Whether this program has the built-in `withdrawn` caveat.
    fn withdraws(&self) -> bool {
        self.symbols.get(WITHDRAWN).is_some_and(|id| {
            matches!(self.graph.nodes.get(id), Some(NodeKind::Caveat { .. }))
                && self
                    .rules
                    .iter()
                    .map(|rule| &rule.effect)
                    .chain(
                        self.procedures
                            .values()
                            .flat_map(|procedure| procedure.body.iter().map(|step| &step.effect)),
                    )
                    .any(|effect| matches!(effect, Effect::Withdraw { .. }))
        })
    }

    /// The occurrence a withdrawal names now: named evidence's current
    /// occurrence, or the stream's current reading.
    fn withdrawal_target(&self, target: &EvidenceSelector) -> Option<String> {
        match target {
            EvidenceSelector::Named(evidence) => Some(self.occurrence(evidence).to_string()),
            EvidenceSelector::Latest(stream) => self.reading_streams[stream].current.clone(),
            EvidenceSelector::Caveated { .. } => None,
        }
    }

    fn withdrawal_of(&self, evidence: &str) -> Option<&Withdrawal> {
        self.withdrawals
            .iter()
            .find(|withdrawal| withdrawal.evidence == evidence)
    }

    /// A value newly read from history carries `withdrawn` if what it rests on
    /// has since been withdrawn. The record itself is unchanged.
    fn with_current_withdrawals(&self, mut provenance: Provenance) -> Result<Provenance, String> {
        if provenance
            .evidence
            .iter()
            .any(|evidence| self.withdrawal_of(evidence).is_some())
        {
            provenance.merge(&Provenance::from_names([], [WITHDRAWN.to_string()])?)?;
        }
        Ok(provenance)
    }

    /// `withdrawn(E)`, `withdrawn(latest(S))` and `rests_on_withdrawn(D)`.
    /// Each answer carries the reasons of the withdrawals it found.
    fn withdrawal_predicate(
        &self,
        kind: &str,
        name: &str,
        grounds: bool,
    ) -> Result<Option<Tracked<bool>>, String> {
        let withdrawn: Vec<&Withdrawal> = match kind {
            WITHDRAWN => self.withdrawal_of(name).into_iter().collect(),
            "withdrawn_latest" => self
                .reading_streams
                .get(name)
                .and_then(|stream| stream.current.as_deref())
                .and_then(|current| self.withdrawal_of(current))
                .into_iter()
                .collect(),
            "rests_on_withdrawn" => {
                let current = match self.decision_series.get(name) {
                    Some(series) => series.current.as_deref(),
                    None => Some(name),
                };
                let evidence = current
                    .and_then(|current| self.commitment_grounds.get(current))
                    .map(|grounds| grounds.evidence.clone())
                    .unwrap_or_default();
                evidence
                    .iter()
                    .filter_map(|evidence| self.withdrawal_of(evidence))
                    .collect()
            }
            _ => return Ok(None),
        };
        let mut provenance = Provenance::default();
        for withdrawal in &withdrawn {
            provenance.merge(&if grounds {
                self.qualify_core(&withdrawal.because, &[])?
            } else {
                self.qualify(&withdrawal.because, &[])?
            })?;
        }
        Tracked::new(!withdrawn.is_empty(), provenance).map(Some)
    }

    fn predicate_target<'a>(&'a self, kind: &str, name: &'a str) -> &'a str {
        if kind == "observed" || kind == WITHDRAWN || kind.starts_with(CARRIES) {
            self.occurrence(name)
        } else {
            name
        }
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
            value_grounds: self.states.grounds(),
            commitment_grounds: (*self.commitment_grounds).clone(),
            decision_journal: (*self.journal).clone(),
            elapsed: self.elapsed,
            renewals: (*self.renewals).clone(),
            identifiers: self.identifiers.texts().to_vec(),
            withdrawals: (*self.withdrawals).clone(),
            scheduled_qualifications: (*self.scheduled).clone(),
            cues: self.cues.clone(),
            cue_qualifications: self.cue_qualifications.clone(),
            qualified_values: self.states.values(),
            commitment_bases: (*self.commitment_bases).clone(),
            reading_streams: (*self.reading_streams).clone(),
            decision_series: (*self.decision_series).clone(),
            observation_qualifications: (*self.observation_qualifications).clone(),
            examination_qualifications: (*self.examination_qualifications).clone(),
            reopening_qualifications: (*self.reopening_qualifications).clone(),
            predicate_qualifications: (*self.predicate_qualifications).clone(),
            controls: (*self.controls).clone(),
            clock: self.clock.clone(),
            schema: REACTIVE_SCHEMA.into(),
            source_id: self.source_id.clone(),
            sequence: self.sequence,
            last_event: self.last_event.clone(),
            values: self
                .states
                .names()
                .zip(self.states.cells.iter())
                .map(|(name, cell)| (name.clone(), cell.value.value))
                .collect(),
            world: (*self.world).clone(),
            events: self
                .events
                .iter()
                .map(|(name, parameters)| EventSignature {
                    name: name.clone(),
                    parameters: parameters.clone(),
                })
                .collect(),
            scenes: (*self.scenes).clone(),
            labels: (*self.labels).clone(),
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

/// An effect with graph symbol and history names replaced, for a specialized
/// procedure.
fn rename_effect(effect: &Effect, names: &HashMap<String, String>) -> Effect {
    let rename = |name: &String| names.get(name).cloned().unwrap_or_else(|| name.clone());
    match effect {
        Effect::Call { name, arguments } => Effect::Call {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| argument.rename_symbols(names))
                .collect(),
        },
        Effect::Sample {
            stream,
            value,
            relation,
            claim,
        } => Effect::Sample {
            stream: rename(stream),
            value: value.rename_symbols(names),
            relation: *relation,
            claim: rename(claim),
        },
        Effect::Emit { .. } | Effect::Reject { .. } => effect.clone(),
        Effect::Qualify {
            evidence,
            caveat,
            after,
        } => Effect::Qualify {
            evidence: rename(evidence),
            caveat: rename(caveat),
            after: after.as_ref().map(|after| after.rename_symbols(names)),
        },
        Effect::Withdraw { target, because } => Effect::Withdraw {
            target: match target {
                EvidenceSelector::Named(evidence) => EvidenceSelector::Named(rename(evidence)),
                EvidenceSelector::Latest(stream) => EvidenceSelector::Latest(rename(stream)),
                selector => selector.clone(),
            },
            because: rename(because),
        },
        Effect::Renew { evidence } => Effect::Renew {
            evidence: rename(evidence),
        },
        Effect::Set {
            name,
            value,
            because,
        } => Effect::Set {
            name: name.clone(),
            value: value.rename_symbols(names),
            because: because.as_ref().map(|citations| {
                citations
                    .iter()
                    .map(|citation| citation.rename_symbols(names))
                    .collect()
            }),
        },
        Effect::Reveal {
            evidence,
            relation,
            claim,
        } => Effect::Reveal {
            evidence: rename(evidence),
            relation: *relation,
            claim: rename(claim),
        },
        Effect::Examine { caveat, cost } => Effect::Examine {
            caveat: rename(caveat),
            cost: *cost,
        },
        Effect::Commit {
            action,
            reason,
            using,
            retaining,
        } => Effect::Commit {
            action: rename(action),
            reason: reason.clone(),
            using: using.as_ref().map(|value| value.rename_symbols(names)),
            retaining: retaining.iter().map(rename).collect(),
        },
        Effect::Reopen { action, because } => Effect::Reopen {
            action: rename(action),
            because: match because {
                EvidenceSelector::Named(evidence) => EvidenceSelector::Named(rename(evidence)),
                EvidenceSelector::Latest(stream) => EvidenceSelector::Latest(rename(stream)),
                EvidenceSelector::Caveated { state, caveat } => EvidenceSelector::Caveated {
                    state: state.clone(),
                    caveat: rename(caveat),
                },
            },
        },
    }
}

/// Stop showing `TARGET.PROPERTY`, and the target once it shows nothing.
fn remove_shown<T>(
    shown: &mut BTreeMap<String, BTreeMap<String, T>>,
    target: &str,
    property: &str,
) {
    if let Some(properties) = shown.get_mut(target) {
        properties.remove(property);
        if properties.is_empty() {
            shown.remove(target);
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
            | "renewable"
            | "identifiers"
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
        "renewable" => match words.as_slice() {
            ["renewable", evidence, "limit", limit] => Ok(Directive::Renewable {
                evidence: identifier(evidence)?,
                limit: limit
                    .parse()
                    .map_err(|_| "renewable limit must be an unsigned integer")?,
            }),
            _ => Err("renewable expects EVIDENCE limit CAPACITY".into()),
        },
        "identifiers" => match words.as_slice() {
            ["identifiers", "limit", limit] => Ok(Directive::Identifiers {
                limit: limit
                    .parse()
                    .map_err(|_| "identifiers limit must be an unsigned integer")?,
            }),
            _ => Err("identifiers expects limit CAPACITY".into()),
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
                                domain: ParameterDomain::Numeric,
                            });
                        }
                        // Bounds and members are filled in when the event is
                        // registered, from the world's entities of that kind.
                        [name, "kind", kind] => parameters.push(Parameter {
                            name: identifier(name)?,
                            min: Number::parse("1")?,
                            max: Number::parse("1")?,
                            domain: ParameterDomain::Entity {
                                kind: identifier(kind)?,
                                members: Vec::new(),
                            },
                        }),
                        // The limit is filled in when the event is registered,
                        // from the program's `identifiers` declaration.
                        [name, "id"] => parameters.push(Parameter {
                            name: identifier(name)?,
                            min: Number::parse("1")?,
                            max: Number::parse("1")?,
                            domain: ParameterDomain::Identifier { limit: 0 },
                        }),
                        [name, "in", members @ ..] if !members.is_empty() => {
                            let members = members
                                .iter()
                                .map(|member| identifier(member))
                                .collect::<Result<Vec<_>, _>>()?;
                            parameters.push(Parameter {
                                name: identifier(name)?,
                                min: Number::parse("1")?,
                                max: Number::parse(&members.len().to_string())?,
                                domain: ParameterDomain::Member { members },
                            });
                        }
                        _ => {
                            return Err("event parameter must be NAME min NUMBER max NUMBER, NAME kind KIND, NAME in NAME..., or NAME id".into())
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
                    | "qualify"
                    | "renew"
                    | "withdraw"
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
    let mut numeric = Vec::new();
    let mut symbol_parameters = Vec::new();
    if !parameters.trim().is_empty() {
        for (position, parameter) in parameters.split(',').enumerate() {
            match parameter.split_whitespace().collect::<Vec<_>>()[..] {
                [name] => numeric.push(identifier(name)?),
                [name, kind @ ("evidence" | "claim" | "caveat" | "readings" | "decisions")] => {
                    symbol_parameters.push(SymbolParameter {
                        position,
                        name: identifier(name)?,
                        kind: kind.into(),
                    })
                }
                _ => {
                    return Err(format!(
                    "proc parameter {} must be NAME, or NAME followed by evidence, claim, caveat, readings or decisions",
                    parameter.trim()
                ))
                }
            }
        }
    }
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
        parameters: numeric,
        symbol_parameters,
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
        ["qualify", evidence, "with", caveat] => Ok(Effect::Qualify {
            evidence: identifier(evidence)?,
            caveat: identifier(caveat)?,
            after: None,
        }),
        ["qualify", evidence, "with", caveat, "after", rest @ ..] if !rest.is_empty() => {
            Ok(Effect::Qualify {
                evidence: identifier(evidence)?,
                caveat: identifier(caveat)?,
                after: Some(reactive_expr::parse_unresolved(&rest.join(" "))?),
            })
        }
        ["renew", evidence] => Ok(Effect::Renew {
            evidence: identifier(evidence)?,
        }),
        ["withdraw", rest @ ..] if rest.contains(&"because") => {
            let split = rest.iter().position(|word| *word == "because").unwrap();
            let (target, reason) = (rest[..split].join(" "), &rest[split + 1..]);
            let [reason] = reason else {
                return Err("withdraw expects EVIDENCE because EVIDENCE".into());
            };
            let target = match target
                .strip_prefix("latest")
                .map(str::trim)
                .and_then(|rest| rest.strip_prefix('('))
            {
                Some(arguments) => EvidenceSelector::Latest(identifier(
                    arguments
                        .trim()
                        .strip_suffix(')')
                        .ok_or("withdraw latest requires a closing parenthesis")?
                        .trim(),
                )?),
                None => EvidenceSelector::Named(identifier(&target)?),
            };
            Ok(Effect::Withdraw {
                target,
                because: identifier(reason)?,
            })
        }
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
            } else if let Some(arguments) = selector
                .strip_prefix("caveated")
                .map(str::trim)
                .and_then(|rest| rest.strip_prefix('('))
            {
                let arguments = arguments
                    .trim()
                    .strip_suffix(')')
                    .ok_or("caveated selector requires closing parenthesis")?;
                let (state, caveat) = arguments
                    .split_once(',')
                    .ok_or("caveated selector requires STATE, CAVEAT")?;
                EvidenceSelector::Caveated {
                    state: identifier(state.trim())?,
                    caveat: identifier(caveat.trim())?,
                }
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

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum PayloadValue {
    Number(f64),
    Text(String),
}

struct Payload(BTreeMap<String, PayloadValue>);

impl<'de> serde::Deserialize<'de> for Payload {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Payload;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an object of unique event parameters")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, PayloadValue>()? {
                    if values.insert(key.clone(), value).is_some() {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate parameter {key}"
                        )));
                    }
                }
                Ok(Payload(values))
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}
