//! Bounded, deterministic event programs over the CAVEAT epistemic graph.
//!
//! The host dispatches declared inputs. Source owns numeric updates and graph
//! effects; an entire event publishes atomically or leaves the session intact.

use crate::ast::{Program, Statement};
use crate::eval::ResourceLedger;
use crate::fast_hash::{HashMap, HashSet};
use crate::game_session::GameSymbol;
use crate::map::{CaveatMap, MapBudget, MapCommitment, MapRelation, MapWorld};
use crate::presentation::Number;
use crate::reactive_expr::{
    self, EvalError, EvalFailure, Expr, FunctionDef, HistoryRead, Reads, Value, ValueType,
};
pub use crate::reactive_expr::{Marker, Provenance, Tracked};
use crate::{Attention, Consequence, EpistemicGraph, NodeId, NodeKind, Relation, StopReason};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
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
/// The records a windowed history holds, live and retired, until departure.
/// See spec/caveat-windows-0.1.md.
pub(crate) const MAX_WINDOWED_RECORDS: usize = 65_536;
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

#[path = "reactive_replay_save.rs"]
mod replay_save;
pub use replay_save::{POSITIONAL_REPLAY_SAVE_SCHEMA, REPLAY_SAVE_MAX_BYTES, REPLAY_SAVE_SCHEMA};

#[path = "reactive_outcome.rs"]
mod outcome;

#[path = "reactive_view2.rs"]
mod view2;
pub use view2::{
    CommitmentChanges, DispatchViewDeltaOutcome, DispatchViewDeltaResult, ListChanges,
    NamedChanges, OrderedGrounds, PropertyChanges, ReactiveViewV2, ViewCommitment, ViewDelta,
    ViewDeltaBase, REACTIVE_VIEW_DELTA_SCHEMA, REACTIVE_VIEW_V2_SCHEMA,
};

#[path = "reactive_collector.rs"]
mod collector;
#[path = "reactive_departure.rs"]
mod departure;
pub use crate::reactive_archive::{ArchiveOperation, ArchiveProvenance};
#[cfg(feature = "collector-metrics")]
pub use collector::WithdrawalCollectionMetrics;
use collector::{CollectorOwner, WithdrawalCollector};
#[cfg(all(feature = "withdrawal-extraction-profile", not(target_arch = "wasm32")))]
pub use departure::WithdrawalExtractionProfile;
pub use departure::{ArchiveEntry, ArchiveItem, Holder};

#[cfg(all(feature = "renewal-removal-profile", not(target_arch = "wasm32")))]
#[path = "reactive_renewal_profile.rs"]
mod renewal_profile;
#[cfg(all(feature = "renewal-removal-profile", not(target_arch = "wasm32")))]
pub use renewal_profile::{RemovalProfile, RenewalRemovalProfile};

#[cfg(all(feature = "event-phase-profile", not(target_arch = "wasm32")))]
#[path = "reactive_event_profile.rs"]
mod event_profile;
#[cfg(all(feature = "event-phase-profile", not(target_arch = "wasm32")))]
pub use event_profile::{
    current_event_phase_index, EventPhase, EventPhaseProfile, EVENT_PHASE_COUNT, EVENT_PHASE_NAMES,
};

#[path = "reactive_identifiers.rs"]
mod identifiers;

#[path = "reactive_check.rs"]
mod check;
pub use check::{
    check_save_with, check_source, check_source_with, CheckOptions, CheckReport, Diagnostic,
    LateCaveatGap, Related, CHECK_SCHEMA,
};

#[path = "reactive_interface.rs"]
mod interface;
use identifiers::{Identifiers, MAX_IDENTIFIER_LIMIT};
pub use interface::{
    interface_source, InterfaceBinding, InterfaceCue, InterfaceDecisions, InterfaceEvent,
    InterfaceParameter, InterfaceReadings, InterfaceState, ProgramInterface, INTERFACE_SCHEMA,
};
use outcome::DispatchFailure;
pub use outcome::{
    DispatchFatal, DispatchOutcome, DispatchResult, DispatchViewOutcome, DispatchViewResult,
    RejectionCode, RejectionOrigin, DISPATCH_SCHEMA,
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
    fn the_prelude_defines_the_functions_the_function_limit_excludes() {
        assert_eq!(
            prelude_functions().unwrap().len(),
            reactive_expr::PRELUDE_FUNCTIONS
        );
    }

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
    /// `stance` is absent for a neutral sample. See
    /// spec/caveat-neutral-sample-0.1.md.
    Sample {
        stream: String,
        value: Expr,
        stance: Option<(Relation, String)>,
        /// `noting EXPRESSION`: the text of the identifier it names, kept
        /// with the reading. See spec/caveat-reading-notes-0.1.md.
        note: Option<Box<Expr>>,
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
        /// `qualify occurrence(EVIDENCE, INDEX) …`: the held occurrence the
        /// index names instead of the current one. See
        /// spec/caveat-earlier-occurrence-0.1.md.
        index: Option<Box<Expr>>,
        /// `qualify latest(NAME) …`: a reading stream's current reading, or
        /// the evidence's current occurrence. See spec/caveat-latest-0.1.md.
        latest: bool,
    },
    /// Record that the live qualifications of every held occurrence of the
    /// evidence by the caveat no longer stand. See
    /// spec/caveat-take-back-0.1.md.
    Unqualify {
        evidence: String,
        caveat: String,
        /// `unqualify occurrence(EVIDENCE, INDEX) …`: only the held occurrence
        /// the index names. See spec/caveat-earlier-occurrence-0.1.md.
        index: Option<Box<Expr>>,
        /// `unqualify latest(NAME) …`: only a reading stream's current
        /// reading, or the evidence's current occurrence. See
        /// spec/caveat-latest-0.1.md.
        latest: bool,
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
    /// Record an observation without a stance toward any claim.
    Observe {
        evidence: String,
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
        /// `citing X, Y`: grounds without a value. See spec/caveat-citing-0.1.md.
        citing: Vec<Expr>,
        /// `permitted by GRANT [for X]`. See spec/caveat-permission-0.1.md.
        /// Boxed so that adding `citing` leaves `Effect` smaller than before.
        permission: Option<Box<PermissionClause>>,
        retaining: Vec<String>,
        /// `retaining C because E`: each caveat that names a reason, with that
        /// evidence and `qualified(1, E)` to read it. See
        /// spec/caveat-retention-reasons-0.1.md.
        reasons: Vec<(String, String, Expr)>,
    },
    /// `reopen D because E`, and from 1.2 `hold D because E` and
    /// `release D because E`, which share its target and selector. See
    /// spec/caveat-holds-0.1.md.
    Reopen {
        action: String,
        because: EvidenceSelector,
        form: DecisionChange,
    },
    /// From 1.3, `resolve D = VALUE because E [materialized C, …]`, or of
    /// `revision(D, K)`. See spec/caveat-outcomes-0.1.md.
    Resolve(Box<Resolution>),
}

/// A `resolve` effect: the outcome it records and the revision it records
/// it on. See spec/caveat-outcomes-0.1.md.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub action: String,
    /// `revision(D, K)`: K. Otherwise the current revision.
    pub revision: Option<Expr>,
    pub value: Expr,
    pub because: EvidenceSelector,
    pub materialized: Vec<String>,
}

/// What `resolve` recorded on a revision. See spec/caveat-outcomes-0.1.md.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub value: f64,
    /// The evidence the outcome was read from, in observation order.
    pub because: Vec<String>,
    /// The revision's retained caveats that came true, in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub materialized: Vec<String>,
    pub sequence: u64,
    pub event: String,
    /// The value's lineage, the evidence's and the guard's, frozen.
    pub lineage: Provenance,
}

/// What a `reopen`, `hold` or `release` effect does to a decision. See
/// spec/caveat-holds-0.1.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionChange {
    Reopen,
    Hold,
    Release,
}

impl DecisionChange {
    pub fn verb(self) -> &'static str {
        match self {
            Self::Reopen => "reopen",
            Self::Hold => "hold",
            Self::Release => "release",
        }
    }
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
    /// `occurrence(EVIDENCE, INDEX)`: a held occurrence of renewable evidence,
    /// chosen when the effect runs. See spec/caveat-earlier-occurrence-0.1.md.
    Occurrence {
        evidence: String,
        index: Box<Expr>,
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
        Effect::Sample { value, note, .. } => {
            *value = reactive_expr::expand_with(value, functions, defines)?;
            if let Some(note) = note {
                **note = reactive_expr::expand_with(note, functions, defines)?;
            }
        }
        Effect::Qualify { after, index, .. } => {
            for value in after
                .iter_mut()
                .chain(index.iter_mut().map(|index| &mut **index))
            {
                *value = reactive_expr::expand_with(value, functions, defines)?;
            }
        }
        Effect::Unqualify {
            index: Some(index), ..
        }
        | Effect::Withdraw {
            target: EvidenceSelector::Occurrence { index, .. },
            ..
        }
        | Effect::Reopen {
            because: EvidenceSelector::Occurrence { index, .. },
            ..
        } => {
            **index = reactive_expr::expand_with(index, functions, defines)?;
        }
        Effect::Commit {
            using,
            citing,
            permission,
            reasons,
            ..
        } => {
            let scope = permission.as_mut().and_then(|clause| clause.scope.as_mut());
            let reasons = reasons.iter_mut().map(|(_, _, reading)| reading);
            for value in using
                .iter_mut()
                .chain(citing.iter_mut())
                .chain(scope)
                .chain(reasons)
            {
                *value = reactive_expr::expand_with(value, functions, defines)?;
            }
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

/// The parameters an effect runs with. An event's parameters are plain. A
/// procedure's are its evaluated arguments, read two ways: `evaluate` reads
/// each argument's lineage and `evaluate_grounds` its grounds, so a parameter
/// grounds a value as its argument would inline (F268).
#[derive(Default)]
struct Parameters {
    lineage: BTreeMap<String, Tracked<f64>>,
    grounds: BTreeMap<String, Tracked<f64>>,
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

#[cfg(test)]
mod elapsed_guard_tests {
    use super::*;

    // F151: the accumulator cannot become nonfinite through declared bounds,
    // whose magnitude is at most 1e12, nor through restore, which refuses a
    // nonfinite clock. The guard is reached here only by setting the clock
    // directly, which no public entry point can do.
    #[test]
    fn f151_nonfinite_clock_refuses_the_event_as_bound_exceeded() {
        let source = "event advance dt min 0 max 1; clock advance every 1; event idle;";
        let mut session = ReactiveSession::from_source(source).unwrap();
        session.elapsed = f64::INFINITY;
        let before = session.snapshot();
        let outcome = session
            .dispatch_outcome_json("advance", r#"{"dt":1}"#)
            .unwrap();
        let report = serde_json::to_value(outcome).unwrap();
        assert_eq!(report["outcome"], "rejected");
        assert_eq!(report["origin"], "evaluation");
        assert_eq!(report["code"], "bound_exceeded");
        assert_eq!(session.snapshot(), before, "the refusal rolls back");
        let shown = session.dispatch_json("idle", "{}").unwrap();
        assert_eq!(shown.sequence, before.sequence + 1, "the session continues");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    Procedure(Procedure),
    Readings {
        name: String,
        template: String,
        limit: usize,
        /// `window N` in place of `limit N`: spec/caveat-windows-0.1.md.
        window: bool,
    },
    Decisions {
        name: String,
        limit: usize,
        /// `reopened by …`. See spec/caveat-reopening-triggers-0.1.md.
        triggers: Vec<ReopeningTrigger>,
        /// `irreversible [forecloses ACTION, …]`: the actions it forecloses.
        /// See spec/caveat-foreclosure-0.1.md.
        irreversible: Option<Vec<String>>,
    },
    /// `renewable EVIDENCE limit N;` or `renewable EVIDENCE window N;`
    Renewable {
        evidence: String,
        limit: usize,
        window: bool,
    },
    /// `journal window N;`
    JournalWindow {
        limit: usize,
    },
    /// `identifiers limit N;`
    Identifiers {
        limit: usize,
    },
    /// `agent NAME [sends EVENT, …];` See spec/caveat-agents-0.1.md.
    Agent {
        name: String,
        sends: Vec<String>,
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
    /// `clock EVENT every STEP integer`: time counts whole units
    /// (spec/caveat-elapsed-0.1.md, "Integer clocks"). Published only when
    /// set, so a clock without it keeps its metadata bytes.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub integer: bool,
}

/// The largest whole number of units an integer clock reads, 2^53 - 1: the
/// integers binary64 and a JSON host hold exactly.
pub(crate) const MAX_INTEGER_CLOCK: f64 = 9_007_199_254_740_991.0;

/// A whole number of integer-clock units within +-(2^53 - 1).
pub(crate) fn whole_clock_reading(value: f64) -> bool {
    value.fract() == 0.0 && value.abs() <= MAX_INTEGER_CLOCK
}

/// A whole delay an integer clock can wait: 0..2^53 - 1.
pub(crate) fn whole_clock_delay(value: f64) -> bool {
    whole_clock_reading(value) && value >= 0.0
}

pub type QualifiedValue = Tracked<f64>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitmentBasis {
    pub value: Option<f64>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingOccurrence {
    pub id: String,
    pub ordinal: u64,
    pub sequence: u64,
    pub event: String,
    /// The agent the host said sent the event, in a program that declares
    /// agents. See spec/caveat-agents-0.1.md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    pub value: f64,
    pub provenance: Provenance,
    /// Both absent for a neutral reading: spec/caveat-neutral-sample-0.1.md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim: Option<String>,
    /// What was observed, as text: spec/caveat-reading-notes-0.1.md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingStream {
    pub template: String,
    pub limit: usize,
    pub current: Option<String>,
    pub occurrences: Vec<ReadingOccurrence>,
    pub selection_qualifications: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRevision {
    pub id: String,
    pub previous: Option<String>,
    pub ordinal: u64,
    pub sequence: u64,
    pub event: String,
    /// How it turned out, once a `resolve` says. Left out until then. See
    /// spec/caveat-outcomes-0.1.md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<Outcome>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionSeries {
    pub limit: usize,
    pub current: Option<String>,
    pub revisions: Vec<DecisionRevision>,
    pub selection_qualifications: Provenance,
    /// While the current revision is on hold, the evidence each `hold` cited,
    /// in the order cited. Empty, and left out, otherwise. See
    /// spec/caveat-holds-0.1.md.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_hold: Vec<String>,
    /// The causes of the reopenings deferred while on hold, in the order
    /// deferred. Empty, and left out, when none waits.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deferred: Vec<String>,
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
#[serde(deny_unknown_fields)]
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
    /// the states still the same as these, to the bit.
    states: Arc<Vec<Arc<StateCell>>>,
}

#[derive(Debug, Clone)]
struct StateRange {
    min: f64,
    max: f64,
}

/// A state's current value, with its lineage, and what it is grounded on.
#[derive(Debug, Clone)]
struct StateCell {
    value: QualifiedValue,
    grounds: Provenance,
}

impl StateCell {
    /// The same value to the bit, lineage and grounds. Not `==` on the
    /// number: `-0.0 == 0.0`, but the sign is observable, for example through
    /// atan2, so a state that only changed its zero's sign has changed.
    fn same(&self, other: &Self) -> bool {
        self.value.value.to_bits() == other.value.value.to_bits()
            && self.value.provenance == other.value.provenance
            && self.grounds == other.grounds
    }
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
            return HashSet::default();
        }
        self.cells
            .iter()
            .zip(old.cells.iter())
            .enumerate()
            .filter(|(_, (new, old))| !Arc::ptr_eq(new, old) && !new.same(old))
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
            Effect::Reveal { evidence, .. } | Effect::Observe { evidence } => {
                Some(evidence.clone())
            }
            _ => None,
        }
    }

    fn reveals(&self, evidence: &str) -> bool {
        matches!(self.effect, Effect::Reveal { evidence: revealed, .. } | Effect::Observe { evidence: revealed } if revealed == evidence)
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
            Effect::Sample { value, note, .. } => {
                expressions.push(value);
                expressions.extend(note.as_deref());
            }
            Effect::Commit {
                using,
                citing,
                permission,
                reasons,
                ..
            } => {
                expressions.extend(using);
                expressions.extend(citing);
                expressions.extend(reasons.iter().map(|(_, _, reading)| reading));
                if let Some(clause) = permission {
                    if let EvidenceSelector::Named(grant) = &clause.grant {
                        uses.push((
                            grant.clone(),
                            guard.clone(),
                            format!("permitted by {grant}"),
                        ));
                    }
                    expressions.extend(&clause.scope);
                }
            }
            Effect::Call { arguments, .. } => expressions.extend(arguments),
            Effect::Qualify {
                evidence,
                after,
                index,
                latest,
                ..
            } => {
                // An earlier occurrence, or a stream's latest reading, is
                // checked when the effect runs.
                if index.is_none() && !latest {
                    uses.push((
                        evidence.clone(),
                        guard.clone(),
                        format!("qualify {evidence}"),
                    ));
                }
                expressions.extend(after);
            }
            Effect::Reopen {
                because: EvidenceSelector::Named(evidence),
                form,
                ..
            } => uses.push((
                evidence.clone(),
                guard.clone(),
                format!("{} because {evidence}", form.verb()),
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
    /// When every graph read names a reading stream or decision series:
    /// those histories. A graph change that touches none of them, and none
    /// of the records every value's caveats are read from, leaves the group
    /// as it was. `None` for any other graph read.
    histories: Option<BTreeSet<String>>,
    /// `number`, `boolean` or `text`: one type for every declaration.
    value_type: &'static str,
}

/// Each bound property's value type, by target and property.
type BindingTypes = BTreeMap<(String, String), &'static str>;

/// What one event changed that an expression can read: states whose value,
/// lineage or grounds differ, and whether anything the graph queries read
/// (graph, observation and predicate records, commitments, histories) did.
struct Changes {
    names: HashSet<String>,
    graph: bool,
    clock: bool,
    /// When the graph changed: the reading streams and decision series whose
    /// records changed, if every change belongs to one of them or to nothing
    /// a history read reaches. `None` when a change could reach any of them.
    histories: Option<BTreeSet<String>>,
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
            || !same(&old.observations, &new.observations)
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
            || !same(&old.renewals, &new.renewals)
            || !same(&old.retired, &new.retired)
            || !same(&old.taken_back, &new.taken_back);
        Self {
            names,
            graph,
            // Signed zero is observable, for example through atan2.
            clock: old.elapsed.to_bits() != new.elapsed.to_bits(),
            histories: if graph {
                Self::histories_between(old, new)
            } else {
                Some(BTreeSet::new())
            },
        }
    }

    /// The histories whose records an event changed, or `None` if it changed
    /// something a read of any history can reach: a qualification, take-back,
    /// withdrawal, renewal, retirement or permission, or a record it cannot
    /// place in one history. A change to a record no history read consults
    /// (a claim, or evidence outside every history) places nowhere.
    fn histories_between(old: &ReactiveSession, new: &ReactiveSession) -> Option<BTreeSet<String>> {
        fn same<T: PartialEq>(old: &Arc<T>, new: &Arc<T>) -> bool {
            Arc::ptr_eq(old, new) || **old == **new
        }
        if !same(
            &old.examination_qualifications,
            &new.examination_qualifications,
        ) || !same(&old.predicate_qualifications, &new.predicate_qualifications)
            || !same(&old.renewals, &new.renewals)
            || !same(&old.retired, &new.retired)
            || !same(&old.taken_back, &new.taken_back)
            || !same(&old.withdrawals, &new.withdrawals)
            || !same(&old.late_caveats, &new.late_caveats)
            || !same(&old.take_back_caveats, &new.take_back_caveats)
            || !same(&old.commitment_permissions, &new.commitment_permissions)
        {
            return None;
        }
        // The history a record belongs to: a history itself, or `NAME@N`.
        let history_of = |name: &str| -> Option<String> {
            let is_history = |name: &str| {
                new.reading_streams.contains_key(name) || new.decision_series.contains_key(name)
            };
            if is_history(name) {
                return Some(name.to_string());
            }
            let (history, ordinal) = name.rsplit_once('@')?;
            (is_history(history) && ordinal.bytes().all(|byte| byte.is_ascii_digit()))
                .then(|| history.to_string())
        };
        let mut changed = BTreeSet::new();
        // Keyed records: every changed key must place in a history.
        fn keyed<V: PartialEq>(
            old: &BTreeMap<String, V>,
            new: &BTreeMap<String, V>,
            added_only: bool,
            place: &dyn Fn(&str) -> Option<String>,
            changed: &mut BTreeSet<String>,
        ) -> Option<()> {
            if old.keys().any(|key| !new.contains_key(key)) {
                return None;
            }
            for (key, value) in new {
                match old.get(key) {
                    Some(previous) if previous == value => {}
                    Some(_) if added_only => return None,
                    _ => {
                        changed.insert(place(key)?);
                    }
                }
            }
            Some(())
        }
        if !Arc::ptr_eq(&old.reading_streams, &new.reading_streams) {
            keyed(
                &old.reading_streams,
                &new.reading_streams,
                false,
                &history_of,
                &mut changed,
            )?;
        }
        if !Arc::ptr_eq(&old.decision_series, &new.decision_series) {
            keyed(
                &old.decision_series,
                &new.decision_series,
                false,
                &history_of,
                &mut changed,
            )?;
        }
        if !Arc::ptr_eq(&old.commitment_bases, &new.commitment_bases) {
            keyed(
                &old.commitment_bases,
                &new.commitment_bases,
                false,
                &history_of,
                &mut changed,
            )?;
        }
        if !Arc::ptr_eq(&old.commitment_grounds, &new.commitment_grounds) {
            keyed(
                &old.commitment_grounds,
                &new.commitment_grounds,
                false,
                &history_of,
                &mut changed,
            )?;
        }
        if !Arc::ptr_eq(&old.reopening_qualifications, &new.reopening_qualifications) {
            keyed(
                &old.reopening_qualifications,
                &new.reopening_qualifications,
                false,
                &history_of,
                &mut changed,
            )?;
        }
        // A new reading's qualification is new; one that changes could be
        // read through any value that holds that reading.
        if !Arc::ptr_eq(
            &old.observation_qualifications,
            &new.observation_qualifications,
        ) {
            let anywhere = |name: &str| Some(history_of(name).unwrap_or_default());
            keyed(
                &old.observation_qualifications,
                &new.observation_qualifications,
                true,
                &anywhere,
                &mut changed,
            )?;
        }
        // Names only gain symbols.
        if !Arc::ptr_eq(&old.symbols, &new.symbols)
            && old
                .symbols
                .iter()
                .any(|(name, id)| new.symbols.get(name) != Some(id))
        {
            return None;
        }
        if !Arc::ptr_eq(&old.observations, &new.observations)
            && !new.observations.starts_with(&old.observations)
        {
            return None;
        }
        if !Arc::ptr_eq(&old.graph, &new.graph) {
            // Edges are only added here; a `qualifies` edge reaches every
            // value whose lineage lists its evidence.
            let added = new.graph.edges.strip_prefix(old.graph.edges.as_slice())?;
            if added
                .iter()
                .any(|edge| edge.relation == Relation::Qualifies)
            {
                return None;
            }
            let mut touched = added
                .iter()
                .flat_map(|edge| [edge.from, edge.to])
                .collect::<HashSet<_>>();
            if old
                .graph
                .nodes
                .keys()
                .any(|id| !new.graph.nodes.contains_key(id))
            {
                return None;
            }
            touched.extend(
                new.graph
                    .nodes
                    .iter()
                    .filter(|(id, node)| old.graph.nodes.get(id) != Some(*node))
                    .map(|(id, _)| *id),
            );
            if !touched.is_empty() {
                for (name, id) in new.symbols.iter() {
                    if touched.contains(id) {
                        if let Some(history) = history_of(name) {
                            changed.insert(history);
                        }
                    }
                }
            }
        }
        changed.remove("");
        Some(changed)
    }

    fn touch(&self, reads: &Reads, histories: Option<&BTreeSet<String>>) -> bool {
        reads.anything
            || (reads.graph
                && self.graph
                && match (histories, &self.histories) {
                    (Some(read), Some(changed)) => read.iter().any(|name| changed.contains(name)),
                    _ => true,
                })
            || (reads.clock && self.clock)
            || reads.names.iter().any(|name| self.names.contains(name))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EffectReport {
    Sample {
        stream: String,
        id: String,
        value: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        relation: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        /// `noting`: spec/caveat-reading-notes-0.1.md.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    Reveal {
        evidence: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        relation: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<String>,
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
    /// The occurrences whose qualification a take-back marked, in
    /// occurrence order. See spec/caveat-take-back-0.1.md.
    Unqualify {
        evidence: String,
        caveat: String,
        occurrences: Vec<String>,
    },
    Renew {
        evidence: String,
        occurrence: String,
    },
    Withdraw {
        evidence: String,
        because: String,
    },
    /// The oldest live record of a windowed history left the live graph.
    /// See spec/caveat-windows-0.1.md.
    Retire {
        history: String,
        record: String,
    },
    /// A retired record that nothing pins left the session for the archive.
    /// See spec/caveat-departure-0.1.md.
    Depart {
        history: String,
        record: String,
    },
}

/// One `reopened by` trigger: a reading stream, optionally only its readings
/// that bear on a claim in one relation, and, when `changed`, only readings
/// whose value differs from the stream's previous reading. See
/// spec/caveat-changed-triggers-0.1.md.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReopeningTrigger {
    pub stream: String,
    pub filter: Option<(Relation, String)>,
    pub changed: bool,
}

impl ReopeningTrigger {
    /// A neutral reading, with no stance, matches only an unfiltered
    /// trigger. See spec/caveat-neutral-sample-0.1.md.
    fn matches(&self, stream: &str, stance: Option<&(Relation, String)>) -> bool {
        self.stream == stream
            && match (&self.filter, stance) {
                (None, _) => true,
                (Some((want, about)), Some((relation, claim))) => {
                    want == relation && about == claim
                }
                (Some(_), None) => false,
            }
    }
}

/// `permitted by GRANT [for X]` on a commit: the grant is named evidence or
/// `latest(STREAM)`, and `for X` only takes a stream's reading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionClause {
    pub grant: EvidenceSelector,
    pub scope: Option<Expr>,
}

/// What a commitment was permitted by, frozen when it was made. Not its
/// grounds; the grant is in its lineage. See spec/caveat-permission-0.1.md.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionRecord {
    /// The concrete grant occurrence.
    pub grant: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<PermissionScope>,
    /// The caveats the grant carried when the commitment was made.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub caveats: Vec<String>,
}

/// The two values `for X` compared: the grant's and the required one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionScope {
    pub granted: f64,
    pub required: f64,
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
    /// The agent the host said sent the event, in a program that declares
    /// agents. See spec/caveat-agents-0.1.md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
}

/// One taken-back qualification: the occurrence whose live `qualifies`
/// relation by the caveat it marked, and the event that marked it. See
/// spec/caveat-take-back-0.1.md.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TakeBack {
    pub evidence: String,
    pub caveat: String,
    pub sequence: u64,
    pub event: String,
    /// The agent the host said sent the event, in a program that declares
    /// agents. See spec/caveat-agents-0.1.md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
}

/// One change to a decision, in the order it happened. See
/// spec/caveat-decision-journal-0.1.md.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JournalEntry {
    /// The declared commitment or decision series.
    pub decision: String,
    /// The concrete commitment, `name@N` for a series revision.
    pub commitment: String,
    /// `committed` or `reopened`.
    pub change: String,
    pub sequence: u64,
    pub event: String,
    /// The agent the host said sent the event, in a program that declares
    /// agents. See spec/caveat-agents-0.1.md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
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
    /// For a commitment made with `permitted by`, its grant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permitted_by: Option<String>,
    /// For `resolved`, the outcome's value. See spec/caveat-outcomes-0.1.md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<f64>,
    /// For `resolved`, the retained caveats it says came true.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub materialized: Vec<String>,
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

/// The parts of `ReactiveView` a host can ask for after an event, in the
/// view's own order. `schema`, `sequence` and `last_event` always come.
pub const VIEW_FIELDS: [&str; 9] = [
    "bindings",
    "binding_explanations",
    "cues",
    "effects",
    "commitments",
    "commitment_grounds",
    "decision_series",
    "decision_journal",
    "relations",
];

/// A selection of `VIEW_FIELDS`, one bit each in that order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewFields(u16);

impl ViewFields {
    /// Every field: the selection that gives `ReactiveView` itself.
    pub const ALL: Self = Self((1 << VIEW_FIELDS.len()) - 1);

    /// The named fields. A name outside `VIEW_FIELDS`, or a repeated one, is
    /// an error naming it.
    pub fn from_names<'n>(names: impl IntoIterator<Item = &'n str>) -> Result<Self, String> {
        let mut bits = 0u16;
        for name in names {
            let Some(index) = VIEW_FIELDS.iter().position(|field| *field == name) else {
                return Err(format!(
                    "unknown view field `{name}`; the fields are {}",
                    VIEW_FIELDS.join(", ")
                ));
            };
            if bits & (1 << index) != 0 {
                return Err(format!("view field `{name}` is named twice"));
            }
            bits |= 1 << index;
        }
        Ok(Self(bits))
    }

    /// The fields named by a JSON array of strings.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let names: Vec<String> = serde_json::from_str(text)
            .map_err(|_| "view fields must be a JSON array of field names".to_string())?;
        Self::from_names(names.iter().map(String::as_str))
    }

    fn has(self, field: &str) -> bool {
        let index = VIEW_FIELDS
            .iter()
            .position(|name| *name == field)
            .expect("a view field");
        self.0 & (1 << index) != 0
    }
}

/// `ReactiveView` with only the selected fields. Each field it holds is
/// exactly the view's, in the view's order; the others are left out.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReactiveViewSelection<'a> {
    pub schema: &'static str,
    pub sequence: u64,
    pub last_event: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bindings: Option<&'a BTreeMap<String, BTreeMap<String, BindingValue>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_explanations: Option<&'a BTreeMap<String, BTreeMap<String, Provenance>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cues: Option<&'a [Cue]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<&'a [EffectReport]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitments: Option<Vec<MapCommitment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment_grounds: Option<&'a BTreeMap<String, Provenance>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_series: Option<&'a BTreeMap<String, DecisionSeries>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_journal: Option<&'a [JournalEntry]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<MapRelation>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReactiveSnapshot {
    pub schema: String,
    pub source_id: String,
    /// SHA-256 of the exact source text, in lowercase hex. It identifies the
    /// source; it does not authenticate a save or its history.
    pub source_sha256: String,
    /// Events at or before this sequence ran before this session's lineage
    /// recorded its source digest: their decisions show the source as not
    /// recorded. Zero, and left out, when every event's source is recorded.
    #[serde(skip_serializing_if = "save::is_zero")]
    pub source_unrecorded_through: u64,
    pub sequence: u64,
    /// Seconds counted from the time event's `dt`.
    pub elapsed: f64,
    pub renewals: BTreeMap<String, Renewal>,
    /// First observations in acquisition order, present once a neutral reveal occurs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<String>,
    /// Texts received for `id` parameters: the entry at index 0 has handle 1.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub identifiers: Vec<String>,
    /// Withdrawn observations, in the order they were withdrawn.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub withdrawals: Vec<Withdrawal>,
    /// Taken-back qualifications, in the order they were taken back.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub taken_back: Vec<TakeBack>,
    /// What each permitted commitment was permitted by, frozen.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub commitment_permissions: BTreeMap<String, PermissionRecord>,
    /// By commitment, each retained caveat that named a reason, with the
    /// reason's occurrence. See spec/caveat-retention-reasons-0.1.md.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub retention_reasons: BTreeMap<String, BTreeMap<String, String>>,
    pub scheduled_qualifications: Vec<ScheduledQualification>,
    pub last_event: Option<String>,
    /// The agent the host said sent the last event; left out when it named
    /// none. See spec/caveat-agents-0.1.md.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_agent: Option<String>,
    pub values: BTreeMap<String, f64>,
    pub qualified_values: BTreeMap<String, QualifiedValue>,
    pub commitment_bases: BTreeMap<String, CommitmentBasis>,
    pub reading_streams: BTreeMap<String, ReadingStream>,
    pub decision_series: BTreeMap<String, DecisionSeries>,
    /// Irreversible series and the actions each forecloses, in source order.
    /// See spec/caveat-foreclosure-0.1.md.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub irreversible: BTreeMap<String, Vec<String>>,
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
    /// Each retired record of a windowed history, with the sequence of the
    /// event that retired it. See spec/caveat-windows-0.1.md.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub retired: BTreeMap<String, u64>,
    /// The histories declared with a window, `journal` for `journal window`;
    /// their `limit` is the window. Absent when none is.
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub windows: BTreeSet<String>,
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
    source_sha256: String,
    source_unrecorded_through: u64,
    sequence: u64,
    last_event: Option<String>,
    states: States,
    // Copy-on-write: a transaction's copy shares these until an effect
    // changes one (`Arc::make_mut`), so an event pays for what it touches and
    // an untouched structure is recognised by pointer.
    commitment_bases: Arc<BTreeMap<String, CommitmentBasis>>,
    reading_streams: Arc<BTreeMap<String, ReadingStream>>,
    decision_series: Arc<BTreeMap<String, DecisionSeries>>,
    /// Irreversible series and the actions each forecloses, from the source.
    /// See spec/caveat-foreclosure-0.1.md.
    irreversible: Arc<BTreeMap<String, Vec<String>>>,
    renewals: Arc<BTreeMap<String, Renewal>>,
    /// Empty retains legacy edge-derived observation ordering. The first neutral
    /// reveal materializes the complete order; all later observations append.
    observations: Arc<Vec<String>>,
    /// Texts received for `id` parameters, in handle order.
    identifiers: Arc<Identifiers>,
    /// Withdrawn observations, in the order they were withdrawn.
    withdrawals: Arc<Vec<Withdrawal>>,
    /// Taken-back qualifications, in the order they were taken back.
    taken_back: Arc<Vec<TakeBack>>,
    /// The caveats some `unqualify` names, fixed at load; empty in a program
    /// that does not take back.
    take_back_caveats: Arc<BTreeSet<String>>,
    /// The caveats a `qualify` effect names: the only relations added while
    /// the session runs, so the only ones a record may lack. See
    /// spec/caveat-caveat-sources-0.1.md.
    late_caveats: Arc<BTreeSet<String>>,
    /// Each permitted commitment's frozen permission record.
    commitment_permissions: Arc<BTreeMap<String, PermissionRecord>>,
    retention_reasons: Arc<BTreeMap<String, BTreeMap<String, String>>>,
    /// Declared reopening triggers, by series in name order, each with the
    /// condition its reopening step runs under. Fixed at load.
    reopening_triggers: Arc<Vec<(String, Vec<ReopeningTrigger>, Expr)>>,
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
    /// Each commitment's grounds evidence in first-observed order, frozen when
    /// it was made; only where that differs from name order. View 0.2 shows
    /// grounds in this order. See spec/caveat-view-0.2.md.
    evidence_order: Arc<BTreeMap<String, Vec<String>>>,
    journal: Arc<Vec<JournalEntry>>,
    /// The histories declared with `window N` instead of `limit N`, and the
    /// journal's window. Fixed at load. See spec/caveat-windows-0.1.md.
    windows: Arc<BTreeSet<String>>,
    journal_window: Option<usize>,
    /// Each retired record, with the sequence of the event that retired it.
    /// A journal entry is named `journal@K`, K counting from 1.
    retired: Arc<BTreeMap<String, u64>>,
    /// How many of the oldest journal entries have departed: entry `i` held
    /// is `journal@(journal_departed + i + 1)`. See spec/caveat-departure-0.1.md.
    journal_departed: usize,
    /// Departed records the host has not drained, in the order they left. A
    /// handover buffer, not session state: nothing reads it, and a save does
    /// not hold it.
    archive: Vec<ArchiveItem>,
    /// Source-specific reference counts and withdrawal edges, derived from
    /// session state. Shared by transaction clones; never serialized.
    pin_counts: Arc<BTreeMap<String, usize>>,
    withdrawal_reasons: Arc<BTreeMap<String, String>>,
    /// New retirements and retired records whose final pin was released.
    departure_candidates: Arc<BTreeSet<String>>,
    /// Decision series a rule or binding reads through
    /// `rests_on_withdrawn(revision(SERIES, INDEX))`. Every revision of these
    /// keeps its grounds pinned, not only the current one. Fixed at load. See
    /// spec/caveat-past-revisions-0.1.md.
    revision_series: Arc<BTreeSet<String>>,
    collector: WithdrawalCollector,
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
    /// The events accepted since the program loaded, for a replay save: none
    /// once they outgrow one, or in a session restored from a state save.
    replay: Option<Arc<replay_save::ReplayLog>>,
    /// Each declared agent and the events it `sends`, in source order. Empty
    /// in a program that declares none. See spec/caveat-agents-0.1.md.
    agents: Arc<BTreeMap<String, Vec<String>>>,
    /// Each event some agent `sends`, and every agent that sends it. An event
    /// not listed here may be sent by any agent, or by none.
    senders: Arc<BTreeMap<String, BTreeSet<String>>>,
    /// The agent the host said sent the last accepted event, if it named one.
    last_agent: Option<String>,
}

/// How many loaded programs `from_source_cached` keeps.
const LOADED_PROGRAMS: usize = 4;

thread_local! {
    /// Programs this thread loaded recently, by source text, least recent first.
    static LOADED: std::cell::RefCell<Vec<(String, ReactiveSession)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

impl ReactiveSession {
    /// `from_source`, reusing a program this thread loaded recently. Loading
    /// depends only on the source text, and a clone of a freshly loaded
    /// session is that session (its shared parts are copied on write), so a
    /// host that opens or restores one program again pays its load once. The
    /// last few programs that loaded are kept; a source that does not load is
    /// not, and fails the same way each time.
    pub fn from_source_cached(source: &str) -> Result<Self, String> {
        let kept = LOADED.with_borrow_mut(|loaded| {
            let index = loaded.iter().position(|(text, _)| text == source)?;
            let entry = loaded.remove(index);
            let session = entry.1.clone();
            loaded.push(entry);
            Some(session)
        });
        if let Some(session) = kept {
            return Ok(session);
        }
        let session = Self::from_source(source)?;
        LOADED.with_borrow_mut(|loaded| {
            if loaded.len() == LOADED_PROGRAMS {
                loaded.remove(0);
            }
            loaded.push((source.to_string(), session.clone()));
        });
        Ok(session)
    }

    /// Whether the program declares an integer clock.
    pub(crate) fn integer_clock(&self) -> bool {
        self.clock.as_ref().is_some_and(|clock| clock.integer)
    }

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
        let mut declared = 0;
        for directive in &directives {
            if let Directive::Function(function) = directive {
                declared += 1;
                if declared > reactive_expr::MAX_FUNCTIONS {
                    return Err(format!(
                        "source exceeds function limit {}",
                        reactive_expr::MAX_FUNCTIONS
                    ));
                }
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
            source_sha256: source_digest(source),
            source_unrecorded_through: 0,
            sequence: 0,
            last_event: None,
            states: States::default(),
            commitment_bases: Arc::default(),
            reading_streams: Arc::default(),
            decision_series: Arc::default(),
            irreversible: Arc::default(),
            renewals: Arc::default(),
            observations: Arc::default(),
            identifiers: Arc::default(),
            withdrawals: Arc::default(),
            taken_back: Arc::default(),
            take_back_caveats: Arc::default(),
            late_caveats: Arc::default(),
            commitment_permissions: Arc::default(),
            retention_reasons: Arc::default(),
            reopening_triggers: Arc::default(),
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
            evidence_order: Arc::default(),
            journal: Arc::default(),
            windows: Arc::default(),
            journal_window: None,
            retired: Arc::default(),
            journal_departed: 0,
            archive: Vec::new(),
            pin_counts: Arc::default(),
            withdrawal_reasons: Arc::default(),
            departure_candidates: Arc::default(),
            revision_series: Arc::default(),
            collector: WithdrawalCollector::default(),
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
            replay: Some(Arc::default()),
            agents: Arc::default(),
            senders: Arc::default(),
            last_agent: None,
        };
        // Reopening triggers: declared streams and claims, by series in name
        // order, each with its reopening condition.
        let mut reopening_triggers = directives
            .iter()
            .filter_map(|directive| match directive {
                Directive::Decisions { name, triggers, .. } if !triggers.is_empty() => {
                    Some((name.clone(), triggers.clone()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        reopening_triggers.sort_by(|left, right| left.0.cmp(&right.0));
        let mut declared_triggers = Vec::new();
        // Caveats state initializers name in `qualified(…, CAVEAT)`, checked
        // against the take-back caveats once procedures are specialized.
        let initializer_extras = std::cell::RefCell::new(BTreeSet::<String>::new());
        for (series, triggers) in reopening_triggers {
            let condition = reactive_expr::parse_unresolved(&format!(
                "committed({series}) and not reopened({series})"
            ))?;
            declared_triggers.push((series, triggers, condition));
        }
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
            let (name, limit, window) = match directive {
                Directive::Readings {
                    name,
                    limit,
                    window,
                    ..
                } => (name, *limit, *window),
                Directive::Decisions { name, limit, .. } => (name, *limit, false),
                _ => continue,
            };
            if limit == 0 || limit > MAX_HISTORY_LIMIT {
                let clause = if window { "window" } else { "limit" };
                return Err(format!(
                    "history {name} {clause} must be in 1..{MAX_HISTORY_LIMIT}"
                ));
            }
            if window {
                Arc::make_mut(&mut session.windows).insert(name.clone());
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
                Directive::Decisions { irreversible, .. } => {
                    if let Some(forecloses) = irreversible {
                        Arc::make_mut(&mut session.irreversible)
                            .insert(name.clone(), forecloses.clone());
                    }
                    Arc::make_mut(&mut session.decision_series).insert(
                        name.clone(),
                        DecisionSeries {
                            limit,
                            current: None,
                            revisions: Vec::new(),
                            selection_qualifications: Provenance::default(),
                            on_hold: Vec::new(),
                            deferred: Vec::new(),
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
                Directive::Agent { name, sends } => {
                    if Arc::make_mut(&mut session.agents)
                        .insert(name.clone(), sends.clone())
                        .is_some()
                    {
                        return Err(format!("duplicate agent {name}"));
                    }
                    for event in sends {
                        Arc::make_mut(&mut session.senders)
                            .entry(event.clone())
                            .or_default()
                            .insert(name.clone());
                    }
                }
                Directive::JournalWindow { limit } => {
                    if session.journal_window.is_some() {
                        return Err("duplicate journal window".into());
                    }
                    if *limit == 0 || *limit > MAX_HISTORY_LIMIT {
                        return Err(format!("journal window must be in 1..{MAX_HISTORY_LIMIT}"));
                    }
                    session.journal_window = Some(*limit);
                }
                Directive::Renewable {
                    evidence,
                    limit,
                    window,
                } => {
                    session.require_kind(evidence, "evidence")?;
                    if *window {
                        if *limit == 0 || *limit > MAX_HISTORY_LIMIT {
                            return Err(format!(
                                "renewable {evidence} window must be in 1..{MAX_HISTORY_LIMIT}"
                            ));
                        }
                        Arc::make_mut(&mut session.windows).insert(evidence.clone());
                    } else if *limit == 0 || *limit > MAX_RENEWAL_LIMIT {
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
                        "qualification_caveat" => {
                            initializer_extras.borrow_mut().insert(name.to_string());
                            session.require_kind(name, "caveat")
                        }
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
                        .evaluate_grounds(initial, &Parameters::default())?
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
                    let mut names = HashSet::default();
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
        for (agent, sends) in session.agents.iter() {
            if let Some(event) = sends
                .iter()
                .find(|event| !session.events.contains_key(*event))
            {
                return Err(format!("agent {agent} sends undeclared event {event}"));
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
            if clock.integer {
                if clock.event == "tick" {
                    return Err(
                        "an integer clock cannot be tick, whose dt lies within 0..0.1; declare a clock event of the program's own".into(),
                    );
                }
                for (what, value) in [
                    ("step", clock.step.value()),
                    ("dt min", parameters[0].min.value()),
                    ("dt max", parameters[0].max.value()),
                ] {
                    if value.fract() != 0.0 {
                        return Err(format!(
                            "integer clock {}: {what} must be a whole number, not {value}",
                            clock.event
                        ));
                    }
                }
            }
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
        session.declare_take_backs(&initializer_extras.into_inner())?;
        // A retirement names its history; `journal` names the journal's.
        if session.journal_window.is_some() && session.windows.contains("journal") {
            return Err(
                "a windowed history named journal cannot share a program with journal window"
                    .into(),
            );
        }
        for (series, triggers, _) in &declared_triggers {
            for trigger in triggers {
                session
                    .require_readings(&trigger.stream)
                    .map_err(|error| format!("decisions {series} reopened by: {error}"))?;
                if let Some((_, claim)) = &trigger.filter {
                    session
                        .require_kind(claim, "claim")
                        .map_err(|error| format!("decisions {series} reopened by: {error}"))?;
                }
            }
        }
        session.reopening_triggers = Arc::new(declared_triggers);
        session.check_foreclosures()?;
        let (binding_types, revision_series) = session.validate_rules()?;
        session.revision_series = Arc::new(revision_series);
        session.check_observation_order()?;
        session.group_bindings(&binding_types);
        let mut rules_by_event = HashMap::<String, Vec<usize>>::default();
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
        session
            .evaluate_bindings(None)
            .map_err(|error| error.to_string())?;
        session.rebuild_departure_index();
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
            let mut names = HashSet::default();
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
        let mut names = HashMap::default();
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

    fn group_bindings(&mut self, types: &BindingTypes) {
        let mut groups = Vec::<BindingGroup>::new();
        let mut index_of = HashMap::<(String, String), usize>::default();
        let mut group_of = Vec::with_capacity(self.binding_rules.len());
        for binding in self.binding_rules.iter() {
            let key = (binding.target.clone(), binding.property.clone());
            let group = *index_of.entry(key).or_insert_with(|| {
                groups.push(BindingGroup {
                    target: binding.target.clone(),
                    property: binding.property.clone(),
                    reads: Reads::default(),
                    histories: None,
                    value_type: types[&(binding.target.clone(), binding.property.clone())],
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
        for group in &mut groups {
            let reads = &group.reads;
            let scoped = reads.graph
                && !reads.anything
                && !reads.graph_unnamed
                && reads.graph_names.iter().all(|name| {
                    self.reading_streams.contains_key(name)
                        || self.decision_series.contains_key(name)
                });
            group.histories = scoped.then(|| reads.graph_names.clone());
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
            let mut names = HashSet::default();
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

    /// Also returns each bound property's value type, which every one of its
    /// declarations must share.
    /// Also returns the decision series read through `revision(…)`.
    fn validate_rules(&self) -> Result<(BindingTypes, BTreeSet<String>), String> {
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
        let revision_series = std::cell::RefCell::new(BTreeSet::new());
        let validate_predicate = |kind: &str, symbol: &str| -> Result<(), String> {
            // `rests_on_withdrawn(revision(SERIES, INDEX))`. See
            // spec/caveat-past-revisions-0.1.md.
            if kind == reactive_expr::AT_REVISION {
                if !self.decision_series.contains_key(symbol) {
                    return Err(format!("revision({symbol}, …) names no decision series"));
                }
                revision_series.borrow_mut().insert(symbol.to_string());
                return Ok(());
            }
            // `in_basis(…)`, as `SERIES|R|KIND|NAME`. See
            // spec/caveat-in-basis-0.1.md.
            if kind == reactive_expr::IN_BASIS {
                let mut parts = symbol.splitn(4, '|');
                let (Some(series), Some(revision), Some(kind), Some(name)) =
                    (parts.next(), parts.next(), parts.next(), parts.next())
                else {
                    return Err(format!("in_basis received a malformed target {symbol}"));
                };
                if revision.is_empty() {
                    if !self.decision_series.contains_key(series) && !commitments.contains(series) {
                        return Err(format!("in_basis({series}, …) names no decision"));
                    }
                } else {
                    if !self.decision_series.contains_key(series) {
                        return Err(format!("revision({series}, …) names no decision series"));
                    }
                    revision_series.borrow_mut().insert(series.to_string());
                }
                return match kind {
                    "named" => self.require_kind(name, "evidence"),
                    "latest" => self.require_latest(name),
                    _ => self.require_renewable_occurrence(name),
                };
            }
            if let Some(kind) = kind.strip_prefix(reactive_expr::AT_LATEST) {
                self.require_latest(symbol)?;
                return match kind.strip_prefix(CARRIES) {
                    Some(caveat) => self.require_kind(caveat, "caveat"),
                    None => Err(format!("unknown epistemic predicate {kind}")),
                };
            }
            if let Some(kind) = kind.strip_prefix(reactive_expr::AT_OCCURRENCE) {
                self.require_renewable_occurrence(symbol)?;
                return match kind.strip_prefix(CARRIES) {
                    Some(caveat) => self.require_kind(caveat, "caveat"),
                    None => Ok(()),
                };
            }
            match kind {
                "runtime_clock" => Ok(()),
                "observed" => self.require_kind(symbol, "evidence"),
                "examined" => {
                    self.not_a_take_back_caveat(symbol, "examined")?;
                    self.require_kind(symbol, "caveat")
                }
                "qualification_evidence" => self.require_kind(symbol, "evidence"),
                "qualification_caveat" => {
                    self.not_the_withdrawal_caveat(symbol, "qualified")?;
                    self.not_a_take_back_caveat(symbol, "qualified")?;
                    self.require_kind(symbol, "caveat")
                }
                "numeric_history" => self.require_history(symbol),
                "has_sample" | "withdrawn_latest" => self.require_readings(symbol),
                "withdrawn" => self.require_kind(symbol, "evidence"),
                "rests_on_withdrawn" | "permission_withdrawn" if commitments.contains(symbol) => {
                    Ok(())
                }
                "rests_on_withdrawn" | "permission_withdrawn" => {
                    Err(format!("unknown reactive commitment {symbol}"))
                }
                "committed" | "reopened" if commitments.contains(symbol) => Ok(()),
                "committed" | "reopened" => Err(format!("unknown reactive commitment {symbol}")),
                reactive_expr::ON_HOLD if self.decision_series.contains_key(symbol) => Ok(()),
                reactive_expr::ON_HOLD => Err(format!(
                    "on_hold names a decision series; {symbol} is not one"
                )),
                reactive_expr::RESOLVED | reactive_expr::OUTCOME
                    if self.decision_series.contains_key(symbol) =>
                {
                    Ok(())
                }
                reactive_expr::RESOLVED | reactive_expr::OUTCOME => Err(format!(
                    "{kind} names a decision series; {symbol} is not one"
                )),
                reactive_expr::AT_REVISION_RESOLVED
                    if self.decision_series.contains_key(symbol) =>
                {
                    Ok(())
                }
                reactive_expr::AT_REVISION_RESOLVED => {
                    Err(format!("revision({symbol}, …) names no decision series"))
                }
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
        // The states and constants are built into the set once; each step's
        // parameters join it while that step is checked. Rebuilding the whole
        // set for every step made loading quadratic in program size.
        let mut numeric: HashSet<String> = self
            .states
            .names()
            .chain(self.constants.keys())
            .cloned()
            .collect();
        for (context, condition, effect, parameters) in steps {
            let added = parameters
                .into_iter()
                .filter(|name| !numeric.contains(*name))
                .map(str::to_owned)
                .collect::<Vec<_>>();
            numeric.extend(added.iter().cloned());
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
                    stance,
                    note,
                } => {
                    self.require_readings(stream)?;
                    if let Some((_, claim)) = stance {
                        self.require_kind(claim, "claim")?;
                    }
                    if value.validate(&numeric, &validate_predicate)? != ValueType::Number {
                        return Err(format!("sample {stream} requires a numeric expression"));
                    }
                    if let Some(note) = note {
                        if note.validate(&numeric, &validate_predicate)? != ValueType::Text {
                            return Err(format!(
                                "sample {stream} noting requires an identifier handle"
                            ));
                        }
                    }
                }
                Effect::Reject { .. } => {}
                Effect::Qualify {
                    evidence,
                    caveat,
                    after,
                    index,
                    latest,
                } => {
                    if *latest {
                        self.require_latest(evidence)?;
                    } else if index.is_none() {
                        // An index names evidence or a stream: below.
                        self.require_kind(evidence, "evidence")?;
                    }
                    if let Some(index) = index {
                        self.validate_occurrence(evidence, index, &numeric, &validate_predicate)?;
                    }
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
                        if let Some(delay) = after.literal_number() {
                            if self.integer_clock() && !whole_clock_delay(delay) {
                                return Err(format!(
                                    "qualify {evidence} with {caveat} after {delay}: an integer clock waits a whole number of units in 0..{MAX_INTEGER_CLOCK}"
                                ));
                            }
                        }
                    }
                }
                Effect::Unqualify {
                    evidence,
                    caveat,
                    index,
                    latest,
                } => {
                    if *latest {
                        self.require_latest(evidence)?;
                    } else if index.is_none() {
                        // An index names evidence or a stream: below.
                        self.require_kind(evidence, "evidence")?;
                    }
                    self.require_kind(caveat, "caveat")?;
                    if let Some(index) = index {
                        self.validate_occurrence(evidence, index, &numeric, &validate_predicate)?;
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
                        EvidenceSelector::Occurrence { evidence, index } => self
                            .validate_occurrence(evidence, index, &numeric, &validate_predicate)?,
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
                        self.validate_citation(citation, &numeric, &validate_predicate)
                            .map_err(|error| format!("set {name} because: {error}"))?;
                    }
                }
                Effect::Observe { evidence } => {
                    self.require_kind(evidence, "evidence")?;
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
                    self.not_a_take_back_caveat(caveat, "examine")?;
                    if self.resources.is_none() {
                        return Err("reactive examination requires an attention budget".into());
                    }
                }
                Effect::Commit {
                    using,
                    citing,
                    permission,
                    retaining,
                    reasons,
                    ..
                } => {
                    for (_, evidence, reading) in reasons {
                        self.require_kind(evidence, "evidence").map_err(|_| {
                            format!("retaining … because names evidence, not {evidence}")
                        })?;
                        reading.validate(&numeric, &validate_predicate)?;
                    }
                    if let Some(value) = using {
                        if value.validate(&numeric, &validate_predicate)? != ValueType::Number {
                            return Err("commit using requires a numeric expression".into());
                        }
                    }
                    for citation in citing {
                        citation.validate(&numeric, &validate_predicate)?;
                    }
                    if let Some(clause) = permission {
                        match &clause.grant {
                            EvidenceSelector::Named(grant) => {
                                self.require_kind(grant, "evidence")?
                            }
                            EvidenceSelector::Latest(stream) => self.require_readings(stream)?,
                            EvidenceSelector::Caveated { .. }
                            | EvidenceSelector::Occurrence { .. } => {
                                return Err("permitted by names evidence or latest(STREAM)".into())
                            }
                        }
                        if let Some(scope) = &clause.scope {
                            if !matches!(clause.grant, EvidenceSelector::Latest(_)) {
                                return Err("permitted by … for X needs a grant from latest(STREAM), which has a value".into());
                            }
                            if scope.validate(&numeric, &validate_predicate)? != ValueType::Number {
                                return Err(
                                    "permitted by … for requires a numeric expression".into()
                                );
                            }
                        }
                    }
                    let mut retained = HashSet::default();
                    for caveat in retaining {
                        self.require_kind(caveat, "caveat")?;
                        self.not_the_withdrawal_caveat(caveat, "retaining")?;
                        self.not_a_take_back_caveat(caveat, "retaining")?;
                        if !retained.insert(caveat) {
                            return Err(format!("duplicate retained caveat {caveat}"));
                        }
                    }
                }
                Effect::Reopen {
                    action,
                    because,
                    form,
                } => {
                    if !commitments.contains(action) {
                        return Err(format!("unknown reactive commitment {action}"));
                    }
                    // A hold is of a series' current revision, and is kept
                    // with the series (spec/caveat-holds-0.1.md).
                    if *form != DecisionChange::Reopen && !self.decision_series.contains_key(action)
                    {
                        return Err(format!(
                            "{} names a decision series; {action} is not one",
                            form.verb()
                        ));
                    }
                    match because {
                        EvidenceSelector::Named(name) => self.require_kind(name, "evidence")?,
                        EvidenceSelector::Latest(stream) => self.require_readings(stream)?,
                        EvidenceSelector::Caveated { state, caveat } => {
                            self.state_with_caveat(state, caveat)?;
                        }
                        EvidenceSelector::Occurrence { evidence, index } => self
                            .validate_occurrence(evidence, index, &numeric, &validate_predicate)?,
                    }
                }
                // An outcome is of a revision of a series
                // (spec/caveat-outcomes-0.1.md).
                Effect::Resolve(resolution) => {
                    let action = &resolution.action;
                    if !self.decision_series.contains_key(action) {
                        return Err(format!(
                            "resolve names a decision series; {action} is not one"
                        ));
                    }
                    if let Some(index) = &resolution.revision {
                        if index.validate(&numeric, &validate_predicate)? != ValueType::Number {
                            return Err(format!("revision({action}, …) requires a numeric index"));
                        }
                    }
                    if resolution.value.validate(&numeric, &validate_predicate)?
                        != ValueType::Number
                    {
                        return Err(format!("resolve {action} requires a numeric value"));
                    }
                    match &resolution.because {
                        EvidenceSelector::Named(name) => self.require_kind(name, "evidence")?,
                        EvidenceSelector::Latest(stream) => self.require_readings(stream)?,
                        EvidenceSelector::Caveated { state, caveat } => {
                            self.state_with_caveat(state, caveat)?;
                        }
                        EvidenceSelector::Occurrence { evidence, index } => self
                            .validate_occurrence(evidence, index, &numeric, &validate_predicate)?,
                    }
                    for caveat in &resolution.materialized {
                        if self.require_kind(caveat, "caveat").is_err() {
                            return Err(format!("unknown caveat {caveat}"));
                        }
                    }
                }
            }
            for name in &added {
                numeric.remove(name);
            }
        }
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
                self.validate_citation(citation, &numeric, &validate_predicate)
                    .map_err(|error| {
                        format!(
                            "binding {}.{} because: {error}",
                            binding.target, binding.property
                        )
                    })?;
            }
        }
        let types = types
            .into_iter()
            .map(|((target, property), kind)| ((target.clone(), property.clone()), kind))
            .collect();
        Ok((types, revision_series.into_inner()))
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
    fn evaluate_bindings(&mut self, changes: Option<&Changes>) -> Result<(), DispatchFailure> {
        let groups = Arc::clone(&self.binding_groups);
        let rules = Arc::clone(&self.binding_rules);
        let stale = groups
            .iter()
            .map(|group| {
                changes.is_none_or(|changes| changes.touch(&group.reads, group.histories.as_ref()))
            })
            .collect::<Vec<_>>();
        if !stale.contains(&true) {
            return Ok(());
        }
        let parameters = Parameters::default();
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
                .map_err(|error| error.context(context()))?;
            guards[group].merge(&condition.provenance)?;
            if condition.value != Value::Bool(true) {
                continue;
            }
            let value = match &binding.value {
                BindingExpression::Text(value) => Tracked::plain(BindingValue::Text(value.clone())),
                BindingExpression::Expression(expression) => {
                    let value = self
                        .evaluate(expression, &parameters)
                        .map_err(|error| error.context(context()))?;
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
    fn check_incremental_bindings(&self, incremental: &Result<(), DispatchFailure>) {
        let mut full = self.clone();
        let result = full.evaluate_bindings(None);
        assert_eq!(
            incremental, &result,
            "incremental and full binding evaluation disagree on the outcome"
        );
        if result.is_ok() {
            // As JSON text: `==` would take -0.0 and 0.0 for the same value.
            let text = |bindings| serde_json::to_string(bindings).expect("finite bindings");
            assert_eq!(
                text(&self.bindings),
                text(&full.bindings),
                "incremental bindings differ"
            );
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

    /// Citations are expressions, not claim/evidence/history names. Keep the
    /// expression validator's checks, but explain a failed numeric lookup in
    /// the citation's context. Other expression errors retain their diagnostic.
    fn validate_citation(
        &self,
        citation: &Expr,
        numeric: &HashSet<String>,
        predicate: &impl Fn(&str, &str) -> Result<(), String>,
    ) -> Result<(), String> {
        citation.validate(numeric, predicate).map(|_| ()).map_err(|error| {
            let Some(name) = error.strip_prefix("unknown numeric identifier ") else {
                return error;
            };
            let kind = if self.reading_streams.contains_key(name) {
                Some("reading stream")
            } else if self.decision_series.contains_key(name) {
                Some("decision series")
            } else if self.events.contains_key(name) {
                Some("event")
            } else if self.procedures.contains_key(name) {
                Some("procedure")
            } else {
                self.symbols.get(name).and_then(|id| self.graph.nodes.get(id)).map(|node| {
                    match node {
                        NodeKind::Claim { .. } => "claim",
                        NodeKind::Evidence { .. } => "evidence",
                        NodeKind::Caveat { .. } => "caveat",
                        NodeKind::Context { .. } => "context",
                        NodeKind::Commitment { .. } => "commitment",
                        NodeKind::Observation { .. } => "observation",
                    }
                })
            };
            match kind {
                Some(kind) => format!(
                    "{name} is a declared {kind}, not a value citation; cite a state, a history read such as latest(STREAM), or an evidence-bearing expression that the value or its conditions actually read"
                ),
                None => format!("unknown citation identifier {name}"),
            }
        })
    }

    /// Evaluate `because` citations and check that they cite only what the
    /// binding or state actually depended on. An explanation may leave
    /// dependencies out; it may never introduce one. Citations are read for
    /// their grounds, so citing a state cites what it is grounded on rather
    /// than every guard that ever touched it.
    ///
    /// Whether a citation holds depends on the session, not only on the text:
    /// a late caveat can make a declaration win for the first time on a clock
    /// tick, a renewal moves what an evidence name means, and a cited state
    /// can change apart from the value. So a citation that does not hold
    /// refuses the event it is reached on, classified, and the session goes
    /// on; the event is rolled back as for any failure.
    fn grounded_citation(
        &self,
        subject: &str,
        citations: &[Expr],
        lineage: &Provenance,
        parameters: &Parameters,
    ) -> Result<Provenance, DispatchFailure> {
        let mut cited = Provenance::default();
        for citation in citations {
            let value = self
                .evaluate_grounds(citation, parameters)
                .map_err(|error| error.context(format!("{subject} because")))?;
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
            return Err(DispatchFailure::rejected(
                RejectionOrigin::Evaluation,
                RejectionCode::UngroundedCitation,
                format!(
                    "{subject} cites {} that its value and conditions never read",
                    ungrounded.join(", ")
                ),
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

    /// `occurrence(EVIDENCE, INDEX)` names renewable evidence, and
    /// `reading(STREAM, INDEX)` a reading stream: see
    /// spec/caveat-earlier-occurrence-0.1.md and
    /// spec/caveat-reading-index-0.1.md.
    fn require_renewable_occurrence(&self, evidence: &str) -> Result<(), String> {
        if self.renewals.contains_key(evidence) || self.reading_streams.contains_key(evidence) {
            Ok(())
        } else {
            Err(format!(
                "occurrence({evidence}, …) names renewable evidence; declare renewable {evidence} limit N"
            ))
        }
    }

    /// `occurrence(EVIDENCE, INDEX)` in an effect: renewable evidence and a
    /// numeric index.
    fn validate_occurrence(
        &self,
        evidence: &str,
        index: &Expr,
        numeric: &HashSet<String>,
        predicate: &impl Fn(&str, &str) -> Result<(), String>,
    ) -> Result<(), String> {
        self.require_renewable_occurrence(evidence)?;
        if index.validate(numeric, predicate)? != ValueType::Number {
            return Err(format!(
                "occurrence({evidence}, …) requires a numeric index"
            ));
        }
        Ok(())
    }

    /// `latest(NAME)` in `qualify` or `unqualify`: a reading stream or
    /// declared evidence. See spec/caveat-latest-0.1.md.
    fn require_latest(&self, name: &str) -> Result<(), String> {
        if self.reading_streams.contains_key(name) || self.require_kind(name, "evidence").is_ok() {
            Ok(())
        } else {
            Err(format!(
                "latest({name}) must name a reading stream or declared evidence"
            ))
        }
    }

    /// What `latest(NAME)` names now: a reading stream's current reading,
    /// none before its first sample, or the evidence's current occurrence.
    pub(super) fn latest_target(&self, name: &str) -> Option<String> {
        match self.reading_streams.get(name) {
            Some(stream) => stream.current.clone(),
            None => Some(self.occurrence(name).to_string()),
        }
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

    /// An index past the history's records, `latest` of a history with none
    /// yet, or a decision read with no numeric `using` value is a classified
    /// expression failure; see
    /// spec/caveat-dispatch-0.1.md.
    fn history_read(&self, name: &str, query: HistoryRead) -> Result<Tracked<f64>, EvalError> {
        if let HistoryRead::Outcome(index) = query {
            return self.outcome_read(name, index);
        }
        if query == HistoryRead::Latest {
            if let Some(stream) = self.reading_streams.get(name) {
                if stream.occurrences.is_empty() {
                    return Err(EvalError::new(
                        EvalFailure::HistoryIndex,
                        format!("reading stream {name} has no reached sample"),
                    ));
                }
            } else if let Some(series) = self.decision_series.get(name) {
                let Some(current) = &series.current else {
                    return Err(EvalError::new(
                        EvalFailure::HistoryIndex,
                        format!("decision series {name} has no commitment"),
                    ));
                };
                if self.commitment_bases[current].value.is_none() {
                    return Err(EvalError::new(
                        EvalFailure::HistoryIndex,
                        format!("current decision {current} has no numeric using value"),
                    ));
                }
            }
            return Ok(self.latest(name)?);
        }
        // Counting records is a qualified observation of membership. Include
        // every reached record's basis and guards for leaving history unchanged.
        // Indexed reads retain the selected record plus current selection guards.
        if let Some(stream) = self.reading_streams.get(name) {
            let mut provenance = Provenance::default();
            self.inherit(&mut provenance, &stream.selection_qualifications)?;
            let live = &stream.occurrences[self.live_start(&stream.occurrences, |r| &r.id)..];
            let value = match query {
                HistoryRead::Count => {
                    for record in live {
                        provenance.merge(&record.provenance)?;
                    }
                    live.len() as f64
                }
                HistoryRead::At(index) => {
                    let record = live.get(index).ok_or_else(|| {
                        EvalError::new(
                            EvalFailure::HistoryIndex,
                            format!("history index {index} is out of range for {name}"),
                        )
                    })?;
                    provenance.merge(&record.provenance)?;
                    record.value
                }
                HistoryRead::Latest | HistoryRead::Outcome(_) => unreachable!(),
            };
            return Ok(Tracked::new(
                value,
                self.with_current_withdrawals(provenance)?,
            )?);
        }
        if let Some(series) = self.decision_series.get(name) {
            let mut provenance = Provenance::default();
            self.inherit(&mut provenance, &series.selection_qualifications)?;
            let live = &series.revisions[self.live_start(&series.revisions, |r| &r.id)..];
            let value = match query {
                HistoryRead::Count => {
                    for revision in live {
                        provenance.merge(&self.commitment_bases[&revision.id].provenance)?;
                    }
                    live.len() as f64
                }
                HistoryRead::At(index) => {
                    let revision = live.get(index).ok_or_else(|| {
                        EvalError::new(
                            EvalFailure::HistoryIndex,
                            format!("history index {index} is out of range for {name}"),
                        )
                    })?;
                    let basis = &self.commitment_bases[&revision.id];
                    provenance.merge(&basis.provenance)?;
                    basis.value.ok_or_else(|| {
                        EvalError::new(
                            EvalFailure::HistoryIndex,
                            format!("decision {} has no numeric using value", revision.id),
                        )
                    })?
                }
                HistoryRead::Latest | HistoryRead::Outcome(_) => unreachable!(),
            };
            return Ok(Tracked::new(
                value,
                self.with_current_withdrawals(provenance)?,
            )?);
        }
        Err(format!("unknown history {name}").into())
    }

    /// The outcome of a series' current revision, or of revision K. A
    /// revision with none refuses the event as `evaluation/expression`, as
    /// `history_at` past a history's records does. See
    /// spec/caveat-outcomes-0.1.md.
    fn outcome_read(&self, name: &str, index: Option<usize>) -> Result<Tracked<f64>, EvalError> {
        let series = &self.decision_series[name];
        let (label, revision) = match index {
            None => (
                name.to_string(),
                series
                    .current
                    .as_ref()
                    .and_then(|current| series.revisions.iter().find(|held| held.id == *current)),
            ),
            Some(k) => (
                format!("revision({name}, {k})"),
                series.revisions.get(k - 1),
            ),
        };
        let Some(revision) = revision else {
            return Err(EvalError::new(
                EvalFailure::HistoryIndex,
                format!("outcome({label}): {name} has no such revision"),
            ));
        };
        let Some(outcome) = &revision.outcome else {
            return Err(EvalError::new(
                EvalFailure::HistoryIndex,
                format!("outcome({label}): {} has no outcome", revision.id),
            ));
        };
        Ok(Tracked::new(outcome.value, self.outcome_lineage(outcome)?)?)
    }

    /// An outcome's frozen lineage, with the caveats its evidence carries
    /// now, so that a later qualification or withdrawal reaches it.
    fn outcome_lineage(&self, outcome: &Outcome) -> Result<Provenance, String> {
        let mut lineage = outcome.lineage.clone();
        for evidence in &outcome.because {
            lineage.merge(&self.qualify_core(evidence, &[])?)?;
        }
        self.with_current_withdrawals(lineage)
    }

    /// `resolved(SERIES)`, or `resolved` of a revision `SERIES@K`: true, on
    /// the outcome's lineage, when the revision has an outcome. See
    /// spec/caveat-outcomes-0.1.md.
    fn resolved_predicate(&self, name: &str) -> Result<Tracked<bool>, String> {
        let (series, revision) = match self.decision_series.get(name) {
            Some(series) => (series, series.current.clone()),
            None => {
                let (series, _) = name
                    .rsplit_once('@')
                    .ok_or_else(|| format!("resolved received a malformed target {name}"))?;
                (&self.decision_series[series], Some(name.to_string()))
            }
        };
        let outcome = revision
            .and_then(|id| series.revisions.iter().find(|held| held.id == id))
            .and_then(|held| held.outcome.as_ref());
        match outcome {
            Some(outcome) => Tracked::new(true, self.outcome_lineage(outcome)?),
            None => {
                let mut provenance = Provenance::default();
                self.inherit(&mut provenance, &series.selection_qualifications)?;
                Tracked::new(false, provenance)
            }
        }
    }

    fn latest(&self, name: &str) -> Result<Tracked<f64>, String> {
        if let Some(stream) = self.reading_streams.get(name) {
            let reading = stream
                .occurrences
                .last()
                .ok_or_else(|| format!("reading stream {name} has no reached sample"))?;
            return Tracked::new(
                reading.value,
                self.with_current_withdrawals({
                    let mut provenance = reading.provenance.clone();
                    self.inherit(&mut provenance, &stream.selection_qualifications)?;
                    provenance
                })?,
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
                self.with_current_withdrawals({
                    let mut provenance = basis.provenance.clone();
                    self.inherit(&mut provenance, &series.selection_qualifications)?;
                    provenance
                })?,
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
        self.collector_effect(effect);
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
                let guard = self.frozen(guard).into_owned();
                return Arc::make_mut(&mut self.reading_streams)
                    .get_mut(stream)
                    .expect("validated stream")
                    .selection_qualifications
                    .merge(&guard)
                    .map_err(Into::into);
            }
            Effect::Commit { action, .. } if self.decision_series.contains_key(action) => {
                let guard = self.frozen(guard).into_owned();
                return Arc::make_mut(&mut self.decision_series)
                    .get_mut(action)
                    .unwrap()
                    .selection_qualifications
                    .merge(&guard)
                    .map_err(Into::into);
            }
            Effect::Reveal { evidence, .. }
            | Effect::Observe { evidence }
            | Effect::Renew { evidence } => {
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
            let mut guard = guard.clone();
            self.freeze_observation(&mut guard);
            Arc::make_mut(&mut self.predicate_qualifications)
                .entry(kind.into())
                .or_default()
                .entry(name)
                .or_default()
                .merge(&guard)?;
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
        self.apply_classified(event, &payload, new_identifiers, None)
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
        self.dispatch_outcome_as_json(event, payload_json, None)
    }

    /// `dispatch_outcome_json` for an event the host says `agent` sent. See
    /// spec/caveat-agents-0.1.md. `None` is `dispatch_outcome_json`.
    pub fn dispatch_outcome_as_json(
        &mut self,
        event: &str,
        payload_json: &str,
        agent: Option<&str>,
    ) -> Result<DispatchOutcome, DispatchFatal> {
        match self.dispatch_classified(event, payload_json, agent) {
            Ok(()) => Ok(DispatchOutcome::accepted(self.snapshot())),
            Err(failure) => failure.outcome(),
        }
    }

    /// `dispatch_outcome_json` with the view after an accepted event in place
    /// of the snapshot, which is not built. The same transaction; a refusal
    /// or fatal error is exactly the one `dispatch_outcome_json` returns.
    pub fn dispatch_view_outcome_json(
        &mut self,
        event: &str,
        payload_json: &str,
    ) -> Result<DispatchViewOutcome<'_>, DispatchFatal> {
        self.dispatch_view_outcome_as_json(event, payload_json, None)
    }

    /// `dispatch_view_outcome_json` for an event the host says `agent` sent.
    pub fn dispatch_view_outcome_as_json(
        &mut self,
        event: &str,
        payload_json: &str,
        agent: Option<&str>,
    ) -> Result<DispatchViewOutcome<'_>, DispatchFatal> {
        match self.dispatch_classified(event, payload_json, agent) {
            Ok(()) => Ok(DispatchViewOutcome::accepted(self.view())),
            Err(failure) => failure.view_outcome(),
        }
    }

    /// `dispatch_view_outcome_json` with only the selected view fields. The
    /// same transaction and the same refusals; the records a field needs are
    /// built only when it is selected.
    pub fn dispatch_view_fields_outcome_json(
        &mut self,
        event: &str,
        payload_json: &str,
        fields: ViewFields,
    ) -> Result<DispatchViewOutcome<'_, ReactiveViewSelection<'_>>, DispatchFatal> {
        self.dispatch_view_fields_outcome_as_json(event, payload_json, fields, None)
    }

    /// `dispatch_view_fields_outcome_json` for an event the host says `agent` sent.
    pub fn dispatch_view_fields_outcome_as_json(
        &mut self,
        event: &str,
        payload_json: &str,
        fields: ViewFields,
        agent: Option<&str>,
    ) -> Result<DispatchViewOutcome<'_, ReactiveViewSelection<'_>>, DispatchFatal> {
        match self.dispatch_classified(event, payload_json, agent) {
            Ok(()) => Ok(DispatchViewOutcome::accepted(self.view_fields(fields))),
            Err(failure) => failure.view_outcome(),
        }
    }

    /// The transaction both outcome entry points run: resolve the payload,
    /// then apply the event. A failure leaves the session as it was.
    fn dispatch_classified(
        &mut self,
        event: &str,
        payload_json: &str,
        agent: Option<&str>,
    ) -> Result<(), DispatchFailure> {
        let (payload, new_identifiers) = self.resolve_payload(event, payload_json)?;
        self.apply_classified(event, &payload, new_identifiers, agent)
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
        #[cfg(all(feature = "event-phase-profile", not(target_arch = "wasm32")))]
        let _profile = event_profile::Scope::enter(EventPhase::ApplyWrapper);
        self.apply_classified(event, parameters, Vec::new(), None)
            .map_err(|error| error.to_string())
    }

    /// `new_identifiers` are texts the session does not hold, whose handles
    /// follow its own; the event adds them only if it succeeds.
    fn apply_classified(
        &mut self,
        event: &str,
        parameters: &BTreeMap<String, f64>,
        new_identifiers: Vec<String>,
        agent: Option<&str>,
    ) -> Result<(), DispatchFailure> {
        // Declared before staged state: its guard outlives implicit rollback
        // destruction, while successful assignment drops the old state in place.
        #[cfg(all(feature = "event-phase-profile", not(target_arch = "wasm32")))]
        let mut profile = event_profile::Scope::enter(EventPhase::TransactionPreparation);
        let signature = self.events.get(event).ok_or_else(|| {
            DispatchFailure::rejected(
                RejectionOrigin::Input,
                RejectionCode::UnknownEvent,
                format!("undeclared event {event}"),
            )
        })?;
        self.check_agent(event, agent)?;
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
            // A typed parameter's number is a member's position, so it must be
            // whole: a fraction inside the range names no member. The range
            // check above runs first, so an isolated out-of-range value keeps
            // its code. Exact, on the parsed number; no rounding.
            if !parameter.domain.is_numeric() && value.fract() != 0.0 {
                return Err(DispatchFailure::rejected(
                    RejectionOrigin::Input,
                    RejectionCode::PayloadInvalid,
                    format!(
                        "event {event} parameter {} takes a whole number from 1 to {} or a member's name, not {value}",
                        parameter.name,
                        parameter.max.value()
                    ),
                ));
            }
        }
        // An integer clock's dt counts whole units: a fraction inside the
        // range is malformed input. The range check above runs first, so an
        // out-of-range value keeps its code.
        if self.integer_clock() && self.time_event.as_deref() == Some(event) {
            let dt = parameters.get("dt").copied().unwrap_or(0.0);
            if dt.fract() != 0.0 {
                return Err(DispatchFailure::rejected(
                    RejectionOrigin::Input,
                    RejectionCode::PayloadInvalid,
                    format!(
                        "event {event} parameter dt takes a whole number of clock units, not {dt}"
                    ),
                ));
            }
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
        // The archive stays out of the transaction's copy: the event appends
        // to an empty one, kept only if it succeeds.
        let archive = std::mem::take(&mut self.archive);
        let mut next = self.clone();
        self.archive = archive;
        next.put_shown(shown);
        if !new_identifiers.is_empty() {
            let identifiers = Arc::make_mut(&mut next.identifiers);
            for text in new_identifiers {
                identifiers.push(text);
            }
        }
        next.last_agent = agent.map(str::to_string);
        match next.run_event(self, event, parameters) {
            Ok(()) => {
                #[cfg(all(feature = "event-phase-profile", not(target_arch = "wasm32")))]
                profile.switch(EventPhase::CommitCleanup);
                let mut archive = std::mem::take(&mut self.archive);
                archive.append(&mut next.archive);
                *self = next;
                self.archive = archive;
                if agent.is_some() {
                    // A replay save names no agent: spec/caveat-agents-0.1.md.
                    self.replay = None;
                } else {
                    self.record_replay(event, parameters);
                }
                Ok(())
            }
            Err(error) => {
                #[cfg(all(feature = "event-phase-profile", not(target_arch = "wasm32")))]
                profile.switch(EventPhase::RollbackCleanup);
                self.put_shown(next.take_shown());
                Err(error)
            }
        }
    }

    /// Whether the host may say `agent` sent `event`: an agent must be one
    /// the program declares, and an event some agent `sends` takes only those
    /// agents. See spec/caveat-agents-0.1.md.
    fn check_agent(&self, event: &str, agent: Option<&str>) -> Result<(), DispatchFailure> {
        let wrong = |message: String| {
            DispatchFailure::rejected(RejectionOrigin::Input, RejectionCode::WrongAgent, message)
        };
        if let Some(agent) = agent {
            if !self.agents.contains_key(agent) {
                return Err(wrong(if self.agents.is_empty() {
                    format!(
                        "this program declares no agents, so it cannot take an event from {agent}"
                    )
                } else {
                    format!("undeclared agent {agent}")
                }));
            }
        }
        if let Some(senders) = self.senders.get(event) {
            let listed = senders
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ");
            match agent {
                None => {
                    return Err(wrong(format!(
                        "event {event} is sent only by {listed}; say which agent sent it"
                    )))
                }
                Some(agent) if !senders.contains(agent) => {
                    return Err(wrong(format!(
                        "event {event} is sent only by {listed}, not {agent}"
                    )))
                }
                Some(_) => {}
            }
        }
        Ok(())
    }

    /// The body of `apply`, run on the transaction's copy: `old` is the
    /// session as it was before the event.
    fn run_event(
        &mut self,
        old: &Self,
        event: &str,
        parameters: &BTreeMap<String, f64>,
    ) -> Result<(), DispatchFailure> {
        #[cfg(all(feature = "event-phase-profile", not(target_arch = "wasm32")))]
        let mut profile = event_profile::Scope::enter(EventPhase::Evaluation);
        self.collector_begin();
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
            // Unreachable through declared bounds and restore; refused anyway.
            if !self.elapsed.is_finite() {
                return Err(DispatchFailure::rejected(
                    RejectionOrigin::Evaluation,
                    RejectionCode::BoundExceeded,
                    "elapsed time must remain a finite number",
                ));
            }
            if self.integer_clock() && !whole_clock_reading(self.elapsed) {
                return Err(DispatchFailure::rejected(
                    RejectionOrigin::Evaluation,
                    RejectionCode::BoundExceeded,
                    format!(
                        "an integer clock must stay within -{MAX_INTEGER_CLOCK}..{MAX_INTEGER_CLOCK}"
                    ),
                ));
            }
            self.apply_due_qualifications()?;
        }
        let lineage: BTreeMap<_, _> = parameters
            .iter()
            .map(|(name, value)| (name.clone(), Tracked::plain(*value)))
            .collect();
        let parameters = Parameters {
            grounds: lineage.clone(),
            lineage,
        };
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
        // Retired records nothing pins any more leave, before the bindings
        // read what cites them.
        self.depart_unpinned()?;
        // Binding failures roll back the same numeric/graph/cue transaction.
        #[cfg(all(feature = "event-phase-profile", not(target_arch = "wasm32")))]
        profile.switch(EventPhase::BindingEvaluation);
        let changes = Changes::between(old, self);
        let result = self.evaluate_bindings(Some(&changes));
        #[cfg(debug_assertions)]
        self.check_incremental_bindings(&result);
        result?;
        #[cfg(all(feature = "event-phase-profile", not(target_arch = "wasm32")))]
        profile.switch(EventPhase::FinalSettlement);
        self.settle_archive_provenance(Some(old), true);
        Ok(())
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
        self.apply_classified(event, &payload, new_identifiers, None)
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

    /// The view with only the selected fields.
    pub fn view_fields(&self, fields: ViewFields) -> ReactiveViewSelection<'_> {
        let needs_names = fields.has("commitments") || fields.has("relations");
        let names = if needs_names {
            self.symbols
                .iter()
                .map(|(name, id)| (*id, name.as_str()))
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::default()
        };
        let pick = |field: &str| fields.has(field);
        ReactiveViewSelection {
            schema: REACTIVE_VIEW_SCHEMA,
            sequence: self.sequence,
            last_event: self.last_event.as_deref(),
            bindings: pick("bindings").then_some(&self.bindings),
            binding_explanations: pick("binding_explanations")
                .then_some(&self.binding_explanations),
            cues: pick("cues").then_some(self.cues.as_slice()),
            effects: pick("effects").then_some(self.effects.as_slice()),
            commitments: pick("commitments").then(|| self.commitment_records(&names)),
            commitment_grounds: pick("commitment_grounds").then_some(&self.commitment_grounds),
            decision_series: pick("decision_series").then_some(&self.decision_series),
            decision_journal: pick("decision_journal").then_some(self.journal.as_slice()),
            relations: pick("relations").then(|| self.relation_records(&names)),
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
        parameters: &Parameters,
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
        parameters: &Parameters,
    ) -> Result<Tracked<Value>, EvalError> {
        expression.evaluate_tracked_with_identifiers(
            &|name| {
                if name == reactive_expr::ELAPSED_READ {
                    return Ok(Some(Tracked::plain(self.elapsed)));
                }
                Ok(self.states.value(name).cloned().or_else(|| {
                    parameters
                        .lineage
                        .get(name)
                        .cloned()
                        .or_else(|| self.constants.get(name).copied().map(Tracked::plain))
                }))
            },
            &|kind, name| match self.at_occurrence(kind, name) {
                _ if kind == reactive_expr::IN_BASIS => self.in_basis(name, false),
                Some((kind, Some(occurrence))) => self.predicate_tracked(kind, &occurrence),
                Some((_, None)) => Ok(Tracked::plain(false)),
                None => self.predicate_tracked(kind, self.predicate_target(kind, name)),
            },
            &|evidence, caveats| self.qualify_read(self.occurrence(evidence), caveats, false),
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
        parameters: &Parameters,
    ) -> Result<Tracked<Value>, EvalError> {
        expression.evaluate_tracked_with_identifiers(
            &|name| {
                if name == reactive_expr::ELAPSED_READ {
                    return Ok(Some(Tracked::plain(self.elapsed)));
                }
                if let Some(cell) = self.states.get(name) {
                    return Ok(Some(Tracked::new(cell.value.value, cell.grounds.clone())?));
                }
                Ok(parameters
                    .grounds
                    .get(name)
                    .cloned()
                    .or_else(|| self.constants.get(name).copied().map(Tracked::plain)))
            },
            &|kind, name| match self.at_occurrence(kind, name) {
                _ if kind == reactive_expr::IN_BASIS => self.in_basis(name, true),
                Some((kind, Some(occurrence))) => self.predicate_grounds(kind, &occurrence),
                Some((_, None)) => Ok(Tracked::plain(false)),
                None => self.predicate_grounds(kind, self.predicate_target(kind, name)),
            },
            &|evidence, caveats| self.qualify_read(self.occurrence(evidence), caveats, true),
            &|name, query| self.history_read(name, query),
            &|handle| self.identifier_text(handle),
        )
    }

    fn predicate_grounds(&self, kind: &str, name: &str) -> Result<Tracked<bool>, String> {
        if kind == reactive_expr::RESOLVED {
            return self.resolved_predicate(name);
        }
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
        // On hold, it rests on what the holds cited (spec/caveat-holds-0.1.md).
        if kind == reactive_expr::ON_HOLD {
            let series = &self.decision_series[name];
            let mut grounds = Provenance::default();
            for reason in &series.on_hold {
                grounds.merge(&self.qualify_core(reason, &[])?)?;
            }
            return Tracked::new(!series.on_hold.is_empty(), grounds);
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
            // Retired evidence has left the live graph (spec/caveat-windows-0.1.md).
            "observed" => {
                matches!(self.graph.nodes.get(id), Some(NodeKind::Evidence { .. }))
                    && !self.is_retired(name)
                    && (self.observations.iter().any(|observed| observed == name)
                        || self.graph.edges.iter().any(|edge| {
                            edge.from == *id
                                && matches!(edge.relation, Relation::Supports | Relation::Opposes)
                        }))
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
        self.caveats_reaching(roots, true)
    }

    /// Caveats reaching `roots` through `qualifies` relations. With
    /// `honor_take_back` false, a taken-back relation still counts: what a
    /// recorded entry could have carried when it was made.
    fn caveats_reaching(
        &self,
        roots: impl IntoIterator<Item = NodeId>,
        honor_take_back: bool,
    ) -> Vec<String> {
        let mut frontier = roots.into_iter().collect::<Vec<_>>();
        let mut visited = HashSet::default();
        let mut caveats = HashSet::default();
        let marked = if honor_take_back {
            self.marked_relations()
        } else {
            HashMap::default()
        };
        while let Some(id) = frontier.pop() {
            if !visited.insert(id) {
                continue;
            }
            let mut passed = HashMap::default();
            for edge in self
                .graph
                .edges
                .iter()
                .filter(|edge| edge.to == id && edge.relation == Relation::Qualifies)
            {
                // A taken-back relation no longer qualifies what it names.
                if let Some(count) = marked.get(&(edge.from, id)) {
                    let seen = passed.entry(edge.from).or_insert(0);
                    if *seen < *count {
                        *seen += 1;
                        continue;
                    }
                }
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

    /// Start explicit ordering only when an edge cannot represent observation.
    fn record_observation(&mut self, evidence: &str, neutral: bool) {
        if self.observations.is_empty() && neutral {
            let names = self
                .symbols
                .iter()
                .map(|(name, id)| (*id, name))
                .collect::<HashMap<_, _>>();
            let mut observations = Vec::new();
            for edge in &self.graph.edges {
                if matches!(edge.relation, Relation::Supports | Relation::Opposes)
                    && matches!(
                        self.graph.nodes.get(&edge.from),
                        Some(NodeKind::Evidence { .. })
                    )
                {
                    let name = names[&edge.from];
                    if !observations.contains(name) {
                        observations.push(name.clone());
                    }
                }
            }
            self.observations = Arc::new(observations);
        }
        if (neutral || !self.observations.is_empty())
            && !self.observations.iter().any(|name| name == evidence)
        {
            Arc::make_mut(&mut self.observations).push(evidence.to_string());
        }
    }

    /// Evidence names in first-observation order. Legacy sessions derive that
    /// order from their edges; neutral observations require an explicit ledger.
    fn in_observation_order<'a>(&self, evidence: impl Iterator<Item = &'a String>) -> Vec<String> {
        let ranks = self.observation_ranks();
        let mut named = evidence
            .map(|name| (ranks.rank(name), name.clone()))
            .collect::<Vec<_>>();
        named.sort();
        named.into_iter().map(|(_, name)| name).collect()
    }

    /// Where each name was first observed: its place in the observation
    /// ledger, or, for a session without one, the place of its first
    /// `supports` or `opposes` edge. Built once, read for many names.
    pub(crate) fn observation_ranks(&self) -> ObservationRanks<'_> {
        if self.observations.is_empty() {
            let mut first = HashMap::default();
            for (position, edge) in self.graph.edges.iter().enumerate() {
                if matches!(edge.relation, Relation::Supports | Relation::Opposes) {
                    first.entry(edge.from).or_insert(position);
                }
            }
            ObservationRanks::Edges {
                symbols: &self.symbols,
                first,
            }
        } else {
            let mut first = HashMap::default();
            for (position, name) in self.observations.iter().enumerate() {
                first.entry(name.as_str()).or_insert(position);
            }
            ObservationRanks::Ledger(first)
        }
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

    /// `qualified(VALUE, EVIDENCE)` in an expression: unobserved evidence is a
    /// classified failure; see spec/caveat-dispatch-0.1.md. `core` gives
    /// grounds rather than lineage.
    fn qualify_read(
        &self,
        evidence: &str,
        extras: &[String],
        core: bool,
    ) -> Result<Provenance, EvalError> {
        self.require_kind(evidence, "evidence")?;
        if !self.predicate("observed", evidence)? {
            return Err(EvalError::new(
                EvalFailure::UnobservedEvidence,
                format!("cannot qualify a value with unobserved evidence {evidence}"),
            ));
        }
        Ok(if core {
            self.qualify_core(evidence, extras)?
        } else {
            self.qualify(evidence, extras)?
        })
    }

    /// The evidence, the caveats that qualify it and the claims it bears on,
    /// and any explicit extras: what a qualified value is *about*, without the
    /// guard that revealed the evidence.
    fn qualify_core(&self, evidence: &str, extras: &[String]) -> Result<Provenance, String> {
        self.qualify_core_with(evidence, extras, true)
    }

    /// `qualify_core` with taken-back relations still counted: the caveats
    /// the evidence carried before any take-back, which a decision journal
    /// entry made then may cite.
    pub(crate) fn qualify_core_before_take_back(
        &self,
        evidence: &str,
    ) -> Result<Provenance, String> {
        self.qualify_core_with(evidence, &[], false)
    }

    fn qualify_core_with(
        &self,
        evidence: &str,
        extras: &[String],
        honor_take_back: bool,
    ) -> Result<Provenance, String> {
        self.require_kind(evidence, "evidence")?;
        // Retired evidence was observed when it was cited; what cites it
        // still carries it.
        if !self.is_retired(evidence) && !self.predicate("observed", evidence)? {
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
            .caveats_reaching(roots, honor_take_back)
            .into_iter()
            .chain(extras.iter().cloned());
        Provenance::from_names([evidence.into()], caveats)
    }

    fn predicate_tracked(&self, kind: &str, name: &str) -> Result<Tracked<bool>, String> {
        if kind == reactive_expr::RESOLVED {
            return self.resolved_predicate(name);
        }
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
                let mut provenance = Provenance::default();
                self.inherit(&mut provenance, &stream.selection_qualifications)?;
                provenance
            };
            return Tracked::new(stream.current.is_some(), provenance);
        }
        // Whether a series is on hold reads what its reopening reads: the
        // holds, the deferred reopenings and the guards of skipped ones
        // (spec/caveat-holds-0.1.md).
        if kind == reactive_expr::ON_HOLD {
            let value = !self.decision_series[name].on_hold.is_empty();
            let provenance = self.predicate_tracked("reopened", name)?.provenance;
            return Tracked::new(value, provenance);
        }
        if matches!(kind, "committed" | "reopened") {
            if let Some(series) = self.decision_series.get(name) {
                let mut result = if let Some(current) = &series.current {
                    self.predicate_tracked(kind, current)?
                } else {
                    Tracked::plain(false)
                };
                self.inherit(&mut result.provenance, &series.selection_qualifications)?;
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
                    self.inherit(&mut provenance, reopening)?;
                }
                // Only the names these edges reach: the session may hold many.
                let reached = self
                    .graph
                    .edges
                    .iter()
                    .filter_map(|edge| {
                        if edge.from == id
                            && matches!(edge.relation, Relation::Retains | Relation::ReliesOn)
                        {
                            Some(edge.to)
                        } else if kind == "reopened"
                            && edge.to == id
                            && edge.relation == Relation::Reopens
                        {
                            Some(edge.from)
                        } else {
                            None
                        }
                    })
                    .collect::<HashSet<_>>();
                let by_id = if reached.is_empty() {
                    HashMap::default()
                } else {
                    self.symbols
                        .iter()
                        .filter(|(_, id)| reached.contains(*id))
                        .map(|(name, id)| (*id, name))
                        .collect::<HashMap<_, _>>()
                };
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
                self.read_live(&mut provenance)?;
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
        self.collector_dirty(CollectorOwner::Record(name.into()));
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
        parameters: &Parameters,
        guard: &Provenance,
        budget: &mut ExecutionBudget,
    ) -> Result<(), DispatchFailure> {
        self.collector_effect(effect);
        match effect {
            Effect::Call { name, arguments } => {
                let procedures = Arc::clone(&self.procedures);
                let procedure = &procedures[name];
                let mut locals = Parameters::default();
                let mut inherited = guard.clone();
                // Freeze all arguments before the first body effect, including
                // unused arguments. Their qualifications cannot be laundered
                // by selecting a constant result or skipping a nested effect.
                // Each argument is also evaluated for its grounds, which
                // leave out the guards behind its reads (F268).
                for (parameter, argument) in procedure.parameters.iter().zip(arguments) {
                    let value = self.evaluate(argument, parameters)?;
                    let Value::Number(number) = value.value else {
                        return Err(format!("procedure {name} requires numeric arguments").into());
                    };
                    let grounds = self.evaluate_grounds(argument, parameters)?.provenance;
                    inherited.merge(&value.provenance)?;
                    locals
                        .lineage
                        .insert(parameter.clone(), Tracked::new(number, value.provenance)?);
                    locals
                        .grounds
                        .insert(parameter.clone(), Tracked::new(number, grounds)?);
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
                stance,
                note,
            } => {
                self.history_room(stream)?;
                // The previous reading's value, before taking this one can
                // retire it: spec/caveat-changed-triggers-0.1.md.
                let previous = {
                    let readings = &self.reading_streams[stream];
                    readings.current.as_ref().and_then(|current| {
                        readings
                            .occurrences
                            .iter()
                            .find(|occurrence| occurrence.id == *current)
                            .map(|occurrence| occurrence.value)
                    })
                };
                self.retire_oldest(stream);
                let ordinal = self.next_stream_ordinal(stream);
                let readings = &self.reading_streams[stream];
                let name = format!("{stream}@{ordinal}");
                if self.symbols.contains_key(&name) {
                    return Err(format!("generated reading identity {name} already exists").into());
                }
                let template = self.symbols[&readings.template];
                let value = self.evaluate(value, parameters)?;
                let Value::Number(number) = value.value else {
                    return Err("sample requires a numeric value".into());
                };
                // The note is text only: it joins neither the reading's
                // lineage nor its grounds.
                let note = match note {
                    Some(note) => match self.evaluate(note, parameters)?.value {
                        Value::Text(text) => Some(text),
                        _ => return Err("sample noting requires an identifier handle".into()),
                    },
                    None => None,
                };
                let NodeKind::Evidence {
                    description,
                    source,
                } = self.graph.nodes[&template].clone()
                else {
                    unreachable!("validated evidence template")
                };
                // A withdrawn template's `withdrawn` is about its own
                // observation. A reading is a new one, which only a withdrawal
                // of it can withdraw: see spec/caveat-withdrawal-0.1.md.
                let withdrawn = self
                    .withdrawal_of(&readings.template)
                    .map(|_| self.symbols[WITHDRAWN]);
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
                    .filter(|caveat| Some(*caveat) != withdrawn)
                    .collect::<Vec<_>>();
                let id = Arc::make_mut(&mut self.graph).add(NodeKind::Evidence {
                    description: format!("{description} ({stream} reading {ordinal})"),
                    source,
                });
                Arc::make_mut(&mut self.symbols).insert(name.clone(), id);
                self.record_observation(&name, stance.is_none());
                if let Some((relation, claim)) = stance {
                    Arc::make_mut(&mut self.graph).relate(id, *relation, self.symbols[claim]);
                }
                for caveat in inherited {
                    Arc::make_mut(&mut self.graph).relate(caveat, Relation::Qualifies, id);
                }
                let mut observation = value.provenance.union(guard)?;
                self.freeze_observation(&mut observation);
                Arc::make_mut(&mut self.observation_qualifications)
                    .insert(name.clone(), observation);
                let mut provenance = self.qualify(&name, &[])?;
                self.freeze(&mut provenance);
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
                    agent: self.last_agent.clone(),
                    value: number,
                    provenance,
                    relation: stance
                        .as_ref()
                        .map(|(relation, _)| relation_name(*relation).into()),
                    claim: stance.as_ref().map(|(_, claim)| claim.clone()),
                    note: note.clone(),
                });
                self.effects.push(EffectReport::Sample {
                    stream: stream.clone(),
                    id: name,
                    value: number,
                    relation: stance
                        .as_ref()
                        .map(|(relation, _)| relation_name(*relation).into()),
                    target: stance.as_ref().map(|(_, claim)| claim.clone()),
                    note,
                });
                let changed = previous.is_none_or(|previous| previous != number);
                self.run_reopening_triggers(
                    stream,
                    stance.as_ref(),
                    changed,
                    parameters,
                    guard,
                    budget,
                )?;
            }
            Effect::Reject { message } => return Err(DispatchFailure::policy(message)),
            Effect::Qualify {
                evidence,
                caveat,
                after,
                index,
                latest,
            } => {
                let (occurrence, guard) = match index {
                    Some(index) => {
                        let (occurrence, lineage) =
                            self.resolve_occurrence(evidence, index, parameters)?;
                        (occurrence, guard.union(&lineage)?)
                    }
                    None if *latest => match self.latest_target(evidence) {
                        Some(occurrence) => (occurrence, guard.clone()),
                        None => {
                            return Err(DispatchFailure::rejected(
                                RejectionOrigin::Evaluation,
                                RejectionCode::UnobservedEvidence,
                                format!(
                                    "cannot qualify latest({evidence}): its stream has no reading"
                                ),
                            ))
                        }
                    },
                    None => (self.occurrence(evidence).to_string(), guard.clone()),
                };
                let guard = &guard;
                if !self.predicate("observed", &occurrence)? {
                    return Err(DispatchFailure::rejected(
                        RejectionOrigin::Evaluation,
                        RejectionCode::UnobservedEvidence,
                        format!("cannot qualify unobserved evidence {occurrence}"),
                    ));
                }
                match after {
                    None => self.apply_qualification(&occurrence, caveat, guard)?,
                    Some(expression) => {
                        let delay = self.evaluate(expression, parameters)?;
                        let Value::Number(after) = delay.value else {
                            return Err("qualify after requires a number of seconds".into());
                        };
                        if self.integer_clock() && !whole_clock_delay(after) {
                            return Err(EvalError::new(
                                EvalFailure::Domain,
                                format!(
                                    "qualify {occurrence} with {caveat} after: an integer clock waits a whole number of units in 0..{MAX_INTEGER_CLOCK}, not {after}"
                                ),
                            )
                            .into());
                        }
                        if after < 0.0 {
                            return Err(
                                "qualify after requires a nonnegative number of seconds".into()
                            );
                        }
                        if self.scheduled.len() >= MAX_SCHEDULED_QUALIFICATIONS {
                            return Err(DispatchFailure::rejected(
                                RejectionOrigin::Limit,
                                RejectionCode::ScheduledLimit,
                                format!(
                                    "scheduled qualifications exceed limit {MAX_SCHEDULED_QUALIFICATIONS}"
                                ),
                            ));
                        }
                        let mut guard = guard.union(&delay.provenance)?;
                        self.freeze(&mut guard);
                        let scheduled = ScheduledQualification {
                            evidence: occurrence,
                            caveat: caveat.clone(),
                            scheduled_at: self.elapsed,
                            after,
                            guard,
                        };
                        self.add_pin(&scheduled.evidence);
                        self.collector_schedule(&scheduled, true);
                        Arc::make_mut(&mut self.scheduled).push(scheduled);
                    }
                }
            }
            Effect::Unqualify {
                evidence,
                caveat,
                index,
                latest,
            } => {
                let only = match index {
                    Some(index) => Some(self.resolve_occurrence(evidence, index, parameters)?.0),
                    None if *latest => self.latest_target(evidence),
                    None => None,
                };
                // A stream with no reading has nothing to take back.
                if !(*latest && only.is_none()) {
                    self.take_back(evidence, caveat, only.as_deref())?
                }
            }
            Effect::Withdraw { target, because } => {
                let unobserved = |message: String| {
                    DispatchFailure::rejected(
                        RejectionOrigin::Evaluation,
                        RejectionCode::UnobservedEvidence,
                        message,
                    )
                };
                let (evidence, guard) = match target {
                    EvidenceSelector::Occurrence { evidence, index } => {
                        let (occurrence, lineage) =
                            self.resolve_occurrence(evidence, index, parameters)?;
                        (Some(occurrence), guard.union(&lineage)?)
                    }
                    _ => (self.withdrawal_target(target), guard.clone()),
                };
                let guard = &guard;
                let evidence = evidence.ok_or_else(|| {
                    unobserved(format!(
                        "cannot withdraw {target:?}: its stream has no reading"
                    ))
                })?;
                let because = self.occurrence(because).to_string();
                for (name, role) in [(&evidence, "withdraw"), (&because, "withdraw because")] {
                    if !self.predicate("observed", name)? {
                        return Err(unobserved(format!(
                            "cannot {role} unobserved evidence {name}"
                        )));
                    }
                }
                if !self
                    .withdrawals
                    .iter()
                    .any(|withdrawal| withdrawal.evidence == evidence)
                {
                    self.add_pin(&because);
                    if self.windowed() {
                        Arc::make_mut(&mut self.withdrawal_reasons)
                            .insert(evidence.clone(), because.clone());
                        self.collector.withdrawal(&evidence, &because, true);
                    }
                    Arc::make_mut(&mut self.withdrawals).push(Withdrawal {
                        evidence: evidence.clone(),
                        because: because.clone(),
                        sequence: self.sequence,
                        event: self.last_event.clone().unwrap_or_default(),
                        agent: self.last_agent.clone(),
                    });
                    self.apply_qualification(&evidence, WITHDRAWN, guard)?;
                    // apply_qualification reported a qualify: this is a withdrawal.
                    self.effects.pop();
                    self.effects
                        .push(EffectReport::Withdraw { evidence, because });
                }
            }
            Effect::Renew { evidence } => {
                self.history_room(evidence)?;
                self.retire_oldest(evidence);
                let ordinal = self.next_renewal_ordinal(evidence);
                let renewal = &self.renewals[evidence];
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
                    let mut guard = guard.clone();
                    self.freeze_observation(&mut guard);
                    Arc::make_mut(&mut self.observation_qualifications).insert(name.clone(), guard);
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
                let previous_pins = self.states.cells[slot]
                    .grounds
                    .own_evidence()
                    .cloned()
                    .collect::<Vec<_>>();
                for record in previous_pins {
                    self.release_pin(&record);
                }
                for record in grounds.own_evidence() {
                    self.add_pin(record);
                }
                self.states.set(
                    slot,
                    StateCell {
                        value: Tracked::new(number, lineage)?,
                        grounds,
                    },
                );
            }
            Effect::Observe { evidence } => {
                let evidence = self.occurrence(evidence).to_string();
                if !self.predicate("observed", &evidence)? {
                    self.clear_predicate_dependency("observed", &evidence);
                    let mut guard = guard.clone();
                    self.freeze_observation(&mut guard);
                    Arc::make_mut(&mut self.observation_qualifications)
                        .entry(evidence.clone())
                        .or_default()
                        .merge(&guard)?;
                    self.record_observation(&evidence, true);
                    self.effects.push(EffectReport::Reveal {
                        evidence,
                        relation: None,
                        target: None,
                    });
                }
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
                    let mut guard = guard.clone();
                    self.freeze_observation(&mut guard);
                    Arc::make_mut(&mut self.observation_qualifications)
                        .entry(evidence.clone())
                        .or_default()
                        .merge(&guard)?;
                    self.record_observation(evidence, false);
                    Arc::make_mut(&mut self.graph).relate(from, *relation, to);
                    self.effects.push(EffectReport::Reveal {
                        evidence: evidence.clone(),
                        relation: Some(relation_name(*relation).into()),
                        target: Some(claim.clone()),
                    });
                }
            }
            Effect::Examine { caveat, cost } => {
                // This guard is part of the successful attention decision. It
                // survives a later `examined(...)` query without changing any
                // underlying claim or removing a retained caveat. Like any
                // stored guard, it keeps only what it did not take through a
                // live relation, so a take-back reaches the rest (D019).
                let mut guard = guard.clone();
                self.freeze_observation(&mut guard);
                let attention_basis = self
                    .examination_qualifications
                    .get(caveat)
                    .cloned()
                    .unwrap_or_default()
                    .union(&guard)?;
                let resources = self
                    .resources
                    .as_mut()
                    .ok_or("examination requires attention budget")?;
                if *cost > resources.remaining {
                    return Err(DispatchFailure::rejected(
                        RejectionOrigin::Limit,
                        RejectionCode::AttentionLimit,
                        "insufficient examination budget",
                    ));
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
                citing,
                permission,
                retaining,
                reasons,
            } => {
                // Foreclosure first: nothing else could let the commit through.
                // See spec/caveat-foreclosure-0.1.md.
                if let Some(by) = self.foreclosing(action) {
                    return Err(DispatchFailure::rejected(
                        RejectionOrigin::Evaluation,
                        RejectionCode::Foreclosed,
                        format!("{action} is foreclosed by {by}"),
                    ));
                }
                let previous = self
                    .decision_series
                    .get(action)
                    .and_then(|series| series.current.clone());
                if self.symbols.contains_key(action) {
                    let message = format!("commitment {action} already exists");
                    // A plain commitment is made once, reopened or not. See
                    // spec/caveat-dispatch-0.1.md, `already_committed`.
                    if self.commitment_bases.contains_key(action) {
                        return Err(DispatchFailure::rejected(
                            RejectionOrigin::Evaluation,
                            RejectionCode::AlreadyCommitted,
                            message,
                        ));
                    }
                    return Err(message.into());
                }
                let mut provenance = guard.clone();
                let mut ordinal = None;
                if let Some(series) = self.decision_series.get(action) {
                    self.history_room(action)?;
                    ordinal = Some(series.revisions.len() as u64 + 1);
                    if previous.is_some() {
                        let reopened = self.predicate_tracked("reopened", action)?;
                        if !reopened.value {
                            return Err(DispatchFailure::rejected(
                                RejectionOrigin::Evaluation,
                                RejectionCode::DecisionInForce,
                                format!("current decision in {action} must be explicitly reopened before revision"),
                            ));
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
                // Permission is checked before anything is recorded. Its grant
                // controls whether the decision happens: lineage, not grounds.
                let permission = match permission {
                    Some(clause) => {
                        let (record, lineage) =
                            self.check_permission(action, clause, parameters)?;
                        provenance.merge(&lineage)?;
                        Some(record)
                    }
                    None => None,
                };
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
                // `citing`: grounds and lineage as for `using`, with no value.
                // See spec/caveat-citing-0.1.md.
                for citation in citing {
                    provenance.merge(&self.evaluate(citation, parameters)?.provenance)?;
                    grounds.merge(&self.evaluate_grounds(citation, parameters)?.provenance)?;
                }
                // A retained caveat's reason is read as `qualified(1, E)` and
                // joins the grounds. See spec/caveat-retention-reasons-0.1.md.
                let mut retention = BTreeMap::new();
                for (caveat, evidence, reading) in reasons {
                    provenance.merge(&self.evaluate(reading, parameters)?.provenance)?;
                    grounds.merge(&self.evaluate_grounds(reading, parameters)?.provenance)?;
                    retention.insert(caveat.clone(), self.occurrence(evidence).to_string());
                }
                // An irreversible commit names each caveat of consequence
                // material or above that it rests on. See
                // spec/caveat-foreclosure-0.1.md, "An irreversible decision".
                if self.irreversible.contains_key(action) {
                    let unretained = grounds
                        .caveats
                        .iter()
                        .filter(|caveat| !retaining.contains(*caveat))
                        .filter_map(|caveat| {
                            let consequence = self.graph.consequence(self.symbols[caveat]);
                            (consequence >= Consequence::Material).then(|| {
                                format!("{caveat} ({})", format!("{consequence:?}").to_lowercase())
                            })
                        })
                        .collect::<Vec<_>>();
                    if !unretained.is_empty() {
                        return Err(DispatchFailure::rejected(
                            RejectionOrigin::Evaluation,
                            RejectionCode::CaveatedIrreversible,
                            format!(
                                "irreversible {action} rests on {}, which its commit does not retain",
                                unretained.join(", ")
                            ),
                        ));
                    }
                }
                // A series read through `revision(…)` keeps every revision's
                // grounds pinned. See spec/caveat-past-revisions-0.1.md.
                if let Some(previous) = previous
                    .as_ref()
                    .filter(|_| !self.revision_series.contains(action))
                {
                    let released = self.commitment_grounds[previous]
                        .own_evidence()
                        .cloned()
                        .collect::<Vec<_>>();
                    for record in released {
                        self.release_pin(&record);
                    }
                }
                for record in grounds.own_evidence() {
                    self.add_pin(record);
                }
                self.freeze(&mut grounds);
                Arc::make_mut(&mut self.commitment_grounds).insert(name.clone(), grounds);
                provenance.merge(&Provenance::from_names([], retaining.iter().cloned())?)?;
                self.freeze(&mut provenance);

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
                    // A retired record reaches a new basis only through one
                    // that already cites it, such as the predecessor's.
                    if !self.is_retired(evidence) && !self.predicate("observed", evidence)? {
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
                self.collector_reopens(
                    &name,
                    &provenance.evidence.iter().cloned().collect::<Vec<_>>(),
                );
                Arc::make_mut(&mut self.commitment_bases).insert(
                    name.clone(),
                    CommitmentBasis {
                        value: used_value,
                        provenance,
                    },
                );
                if let Some(record) = &permission {
                    self.add_pin(&record.grant);
                    Arc::make_mut(&mut self.commitment_permissions)
                        .insert(name.clone(), record.clone());
                }
                if !retention.is_empty() {
                    // Pinned like a permission's grant: the record outlives
                    // the revision's turn as current.
                    for evidence in retention.values() {
                        self.add_pin(evidence);
                    }
                    Arc::make_mut(&mut self.retention_reasons).insert(name.clone(), retention);
                }
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
                        outcome: None,
                    });
                }
                let grounds = &self.commitment_grounds[&name];
                let entry = JournalEntry {
                    decision: action.clone(),
                    commitment: name.clone(),
                    change: "committed".into(),
                    sequence: self.sequence,
                    event: self.last_event.clone().unwrap_or_default(),
                    agent: self.last_agent.clone(),
                    elapsed: Some(self.elapsed),
                    value: used_value,
                    because: self.in_observation_order(grounds.evidence.iter()),
                    caveats: grounds.caveats.iter().cloned().collect(),
                    permitted_by: permission.map(|record| record.grant),
                    outcome: None,
                    materialized: Vec::new(),
                };
                if !entry.because.iter().eq(grounds.evidence.iter()) {
                    Arc::make_mut(&mut self.evidence_order)
                        .insert(name.clone(), entry.because.clone());
                }
                self.retire_journal_entry();
                self.pin_journal_entry(&entry);
                Arc::make_mut(&mut self.journal).push(entry);
                self.effects.push(EffectReport::Commit {
                    action: name,
                    retained: retained_names,
                });
            }
            Effect::Resolve(resolution) => self.resolve(resolution, parameters, guard)?,
            Effect::Reopen {
                action,
                because,
                form,
            } => {
                let current = self
                    .decision_series
                    .get(action)
                    .and_then(|series| series.current.as_ref())
                    .unwrap_or(action)
                    .clone();
                let id = *self.symbols.get(&current).ok_or_else(|| {
                    DispatchFailure::rejected(
                        RejectionOrigin::Evaluation,
                        RejectionCode::NotCommitted,
                        format!("cannot {} uncommitted action {action}", form.verb()),
                    )
                })?;
                let (because, cause) =
                    self.select_because(because, form.verb(), action, parameters)?;
                let mut basis = cause.union(guard)?;
                if let Some(series) = self.decision_series.get(action) {
                    self.inherit(&mut basis, &series.selection_qualifications)?;
                }
                self.freeze(&mut basis);
                let open = matches!(
                    self.graph.nodes.get(&id),
                    Some(NodeKind::Commitment { open: true, .. })
                );
                let on_hold = self
                    .decision_series
                    .get(action)
                    .is_some_and(|series| !series.on_hold.is_empty());
                match form {
                    DecisionChange::Reopen if on_hold && !open => {
                        // A hold never keeps in force a revision whose
                        // grounds hold withdrawn evidence: the reopening goes
                        // ahead, and the deferred ones follow it
                        // (spec/caveat-holds-0.1.md).
                        let through = self
                            .withdrawal_predicate("rests_on_withdrawn", action, false)?
                            .is_some_and(|rests| rests.value);
                        if through {
                            let deferred = self.end_hold(action);
                            let mut causes = because;
                            for evidence in deferred {
                                if !causes.contains(&evidence) {
                                    causes.push(evidence);
                                }
                            }
                            self.reopen_revision(action, &current, id, causes, &basis)?;
                        } else {
                            let series = &self.decision_series[action];
                            let causes = self.new_causes(&current, id, because, &series.deferred);
                            if !causes.is_empty() {
                                self.join_reopening(&current, &basis)?;
                                for evidence in &causes {
                                    self.add_pin(evidence);
                                }
                                Arc::make_mut(&mut self.decision_series)
                                    .get_mut(action)
                                    .expect("on hold")
                                    .deferred
                                    .extend(causes.iter().cloned());
                                self.journal_change(action, &current, "deferred", causes)?;
                            }
                        }
                    }
                    DecisionChange::Reopen => {
                        self.reopen_revision(action, &current, id, because, &basis)?;
                    }
                    // A revision already reopened is not in force: nothing
                    // is held.
                    DecisionChange::Hold if open => {}
                    DecisionChange::Hold => {
                        let held = &self.decision_series[action].on_hold;
                        let reasons = because
                            .into_iter()
                            .filter(|evidence| !held.contains(evidence))
                            .collect::<Vec<_>>();
                        if !reasons.is_empty() {
                            self.join_reopening(&current, &basis)?;
                            for evidence in &reasons {
                                self.add_pin(evidence);
                            }
                            Arc::make_mut(&mut self.decision_series)
                                .get_mut(action)
                                .expect("a series")
                                .on_hold
                                .extend(reasons.iter().cloned());
                            self.journal_change(action, &current, "held", reasons)?;
                        }
                    }
                    DecisionChange::Release if !on_hold || open => {}
                    DecisionChange::Release => {
                        self.join_reopening(&current, &basis)?;
                        let deferred = self.end_hold(action);
                        self.journal_change(action, &current, "released", because)?;
                        self.reopen_revision(action, &current, id, deferred, &basis)?;
                    }
                }
            }
        }
        self.collector_effect(effect);
        Ok(())
    }

    /// `resolve`: records an outcome on a revision, changing nothing else.
    /// See spec/caveat-outcomes-0.1.md.
    #[inline(never)]
    fn resolve(
        &mut self,
        resolution: &Resolution,
        parameters: &Parameters,
        guard: &Provenance,
    ) -> Result<(), DispatchFailure> {
        let action = &resolution.action;
        let mut lineage = guard.clone();
        let revision = match &resolution.revision {
            None => self.decision_series[action]
                .current
                .clone()
                .ok_or_else(|| {
                    DispatchFailure::rejected(
                        RejectionOrigin::Evaluation,
                        RejectionCode::NotCommitted,
                        format!("cannot resolve {action}: it has no revision"),
                    )
                })?,
            Some(index) => {
                let index = self.evaluate(index, parameters)?;
                let Value::Number(k) = index.value else {
                    return Err(format!("revision({action}, …) requires a numeric index").into());
                };
                lineage.merge(&index.provenance)?;
                self.held_revision(action, k).ok_or_else(|| {
                    DispatchFailure::rejected(
                        RejectionOrigin::Evaluation,
                        RejectionCode::NotCommitted,
                        format!("cannot resolve {action}: it has no revision {k}"),
                    )
                })?
            }
        };
        let position = self.decision_series[action]
            .revisions
            .iter()
            .position(|held| held.id == revision)
            .expect("a held revision");
        let previous = self.decision_series[action].revisions[position]
            .outcome
            .clone();
        if let Some(previous) = &previous {
            // A withdrawn outcome may be corrected; any other stands.
            let withdrawn = previous
                .because
                .iter()
                .any(|evidence| self.withdrawal_of(evidence).is_some());
            if !withdrawn {
                return Err(DispatchFailure::rejected(
                    RejectionOrigin::Evaluation,
                    RejectionCode::AlreadyResolved,
                    format!(
                        "{revision} is already resolved at #{}; withdraw its evidence to correct it",
                        previous.sequence
                    ),
                ));
            }
        }
        let id = self.symbols[&revision];
        for caveat in &resolution.materialized {
            let caveat_id = self.symbols[caveat];
            let retained = self.graph.edges.iter().any(|edge| {
                edge.from == id && edge.to == caveat_id && edge.relation == Relation::Retains
            });
            if !retained {
                return Err(DispatchFailure::rejected(
                    RejectionOrigin::Evaluation,
                    RejectionCode::NotRetained,
                    format!("{revision} does not retain {caveat}, so it cannot have materialized"),
                ));
            }
        }
        let value = self.evaluate(&resolution.value, parameters)?;
        let Value::Number(number) = value.value else {
            return Err(format!("resolve {action} requires a numeric value").into());
        };
        lineage.merge(&value.provenance)?;
        let (because, cause) =
            self.select_because(&resolution.because, "resolve", action, parameters)?;
        lineage.merge(&cause)?;
        self.inherit(
            &mut lineage,
            &self.decision_series[action]
                .selection_qualifications
                .clone(),
        )?;
        self.freeze(&mut lineage);
        if let Some(previous) = &previous {
            for evidence in &previous.because {
                self.release_pin(evidence);
            }
        }
        for evidence in &because {
            self.add_pin(evidence);
        }
        let outcome = Outcome {
            value: number,
            because: because.clone(),
            materialized: resolution.materialized.clone(),
            sequence: self.sequence,
            event: self.last_event.clone().unwrap_or_default(),
            lineage,
        };
        Arc::make_mut(&mut self.decision_series)
            .get_mut(action)
            .expect("a series")
            .revisions[position]
            .outcome = Some(outcome);
        self.retire_journal_entry();
        let mut cause_caveats = BTreeSet::new();
        for evidence in &because {
            cause_caveats.extend(self.qualify_core(evidence, &[])?.caveats);
        }
        let entry = JournalEntry {
            decision: action.clone(),
            commitment: revision.clone(),
            change: "resolved".into(),
            sequence: self.sequence,
            event: self.last_event.clone().unwrap_or_default(),
            agent: self.last_agent.clone(),
            elapsed: Some(self.elapsed),
            value: self
                .commitment_bases
                .get(&revision)
                .and_then(|basis| basis.value),
            because,
            caveats: cause_caveats.into_iter().collect(),
            permitted_by: None,
            outcome: Some(number),
            materialized: resolution.materialized.clone(),
        };
        self.pin_journal_entry(&entry);
        Arc::make_mut(&mut self.journal).push(entry);
        Ok(())
    }

    /// The evidence a `because` selector names now, in observation order,
    /// and its lineage: of a reopening, a hold, a release or an outcome.
    #[inline(never)]
    fn select_because(
        &mut self,
        because: &EvidenceSelector,
        verb: &str,
        action: &str,
        parameters: &Parameters,
    ) -> Result<(Vec<String>, Provenance), DispatchFailure> {
        Ok(match because {
            EvidenceSelector::Named(name) => {
                let name = self.occurrence(name).to_string();
                if !self.predicate("observed", &name)? {
                    return Err(DispatchFailure::rejected(
                        RejectionOrigin::Evaluation,
                        RejectionCode::UnobservedEvidence,
                        format!(
                            "cannot {} {action} because unobserved evidence {name}",
                            verb
                        ),
                    ));
                }
                let cause = self.qualify(&name, &[])?;
                (vec![name], cause)
            }
            EvidenceSelector::Occurrence { evidence, index } => {
                let (name, lineage) = self.resolve_occurrence(evidence, index, parameters)?;
                if !self.predicate("observed", &name)? {
                    return Err(DispatchFailure::rejected(
                        RejectionOrigin::Evaluation,
                        RejectionCode::UnobservedEvidence,
                        format!(
                            "cannot {} {action} because unobserved evidence {name}",
                            verb
                        ),
                    ));
                }
                let mut cause = self.qualify(&name, &[])?;
                cause.merge(&lineage)?;
                (vec![name], cause)
            }
            EvidenceSelector::Latest(stream) => {
                if self.reading_streams[stream].current.is_none() {
                    return Err(DispatchFailure::rejected(
                        RejectionOrigin::Evaluation,
                        RejectionCode::UnobservedEvidence,
                        format!(
                            "cannot {} {action} because latest({stream}): its stream has no reading",
                            verb
                        ),
                    ));
                }
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
                    return Err(DispatchFailure::rejected(
                        RejectionOrigin::Evaluation,
                        RejectionCode::EmptyCaveatedSelection,
                        format!("caveated({state}, {caveat}) selects no observed evidence carrying {caveat} from the state's grounds"),
                    ));
                }
                (self.in_observation_order(selected.into_iter()), cause)
            }
        })
    }

    /// Ends `action`'s hold, if it has one, releasing what the hold pinned,
    /// and returns the reopenings it deferred.
    fn end_hold(&mut self, action: &str) -> Vec<String> {
        if self
            .decision_series
            .get(action)
            .is_none_or(|series| series.on_hold.is_empty() && series.deferred.is_empty())
        {
            return Vec::new();
        }
        let series = Arc::make_mut(&mut self.decision_series)
            .get_mut(action)
            .expect("a series");
        let held = std::mem::take(&mut series.on_hold);
        let deferred = std::mem::take(&mut series.deferred);
        for evidence in held.iter().chain(&deferred) {
            self.release_pin(evidence);
        }
        deferred
    }

    /// The causes in `because` that neither reopen `current` already nor
    /// are among `kept`.
    fn new_causes(
        &self,
        current: &str,
        id: NodeId,
        because: Vec<String>,
        kept: &[String],
    ) -> Vec<String> {
        let _ = current;
        because
            .into_iter()
            .filter(|evidence| {
                let from = self.symbols[evidence];
                !kept.contains(evidence)
                    && !self.graph.edges.iter().any(|edge| {
                        edge.from == from && edge.to == id && edge.relation == Relation::Reopens
                    })
            })
            .collect()
    }

    /// Adds `basis` to the record of what `current`'s reopening rests on: a
    /// reopening, and from 1.2 a hold, a deferred reopening or a release of
    /// it (spec/caveat-holds-0.1.md).
    fn join_reopening(&mut self, current: &str, basis: &Provenance) -> Result<(), DispatchFailure> {
        Arc::make_mut(&mut self.reopening_qualifications)
            .entry(current.to_string())
            .or_default()
            .merge(basis)?;
        Ok(())
    }

    /// One journal entry for a change to `current` that is not a commit,
    /// citing `because` in observation order with the caveats it carries now.
    fn journal_change(
        &mut self,
        action: &str,
        current: &str,
        change: &str,
        because: Vec<String>,
    ) -> Result<(), DispatchFailure> {
        self.retire_journal_entry();
        let mut caveats = BTreeSet::new();
        for evidence in &because {
            caveats.extend(self.qualify_core(evidence, &[])?.caveats);
        }
        let entry = JournalEntry {
            decision: action.to_string(),
            commitment: current.to_string(),
            change: change.into(),
            sequence: self.sequence,
            event: self.last_event.clone().unwrap_or_default(),
            agent: self.last_agent.clone(),
            elapsed: Some(self.elapsed),
            value: self
                .commitment_bases
                .get(current)
                .and_then(|basis| basis.value),
            because: self.in_observation_order(because.iter()),
            caveats: caveats.into_iter().collect(),
            permitted_by: None,
            outcome: None,
            materialized: Vec::new(),
        };
        self.pin_journal_entry(&entry);
        Arc::make_mut(&mut self.journal).push(entry);
        Ok(())
    }

    /// Reopens `current`, `id` in the graph, because of each of `because`
    /// that does not reopen it already, on `basis`: the cause's lineage, the
    /// guard and the series' selection, frozen. Nothing happens when every
    /// cause reopens it already. A reopening ends a hold, and the next
    /// revision starts with none.
    fn reopen_revision(
        &mut self,
        action: &str,
        current: &str,
        id: NodeId,
        because: Vec<String>,
        basis: &Provenance,
    ) -> Result<(), DispatchFailure> {
        let because = self.new_causes(current, id, because, &[]);
        if because.is_empty() {
            return Ok(());
        }
        self.end_hold(action);
        let current = current.to_string();
        self.collector_reopens(&current, &because);
        self.clear_predicate_dependency("reopened", &current);
        self.join_reopening(&current, basis)?;
        // Reported before the reopen that adds the entry.
        self.retire_journal_entry();
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
            decision: action.to_string(),
            commitment: current.clone(),
            change: "reopened".into(),
            sequence: self.sequence,
            event: self.last_event.clone().unwrap_or_default(),
            agent: self.last_agent.clone(),
            elapsed: Some(self.elapsed),
            value: self
                .commitment_bases
                .get(&current)
                .and_then(|basis| basis.value),
            because,
            caveats: caveats.into_iter().collect(),
            permitted_by: None,
            outcome: None,
            materialized: Vec::new(),
        };
        self.pin_journal_entry(&entry);
        Arc::make_mut(&mut self.journal).push(entry);
        Ok(())
    }

    /// How many `CAVEAT qualifies OCCURRENCE` relations a take-back has
    /// marked, by caveat and occurrence. The marked relations of a pair are
    /// its first ones in graph order, since a take-back marks the one live
    /// relation and a qualify adds one only when none is live. Empty unless
    /// the program has taken something back. See spec/caveat-take-back-0.1.md.
    fn marked_relations(&self) -> HashMap<(NodeId, NodeId), usize> {
        let mut marked = HashMap::default();
        for record in self.taken_back.iter() {
            if let (Some(&caveat), Some(&occurrence)) = (
                self.symbols.get(&record.caveat),
                self.symbols.get(&record.evidence),
            ) {
                *marked.entry((caveat, occurrence)).or_insert(0) += 1;
            }
        }
        marked
    }

    /// Whether `caveat qualifies occurrence` through a relation no take-back
    /// has marked.
    fn has_live_relation(&self, caveat: NodeId, occurrence: NodeId) -> bool {
        let relations = self
            .graph
            .edges
            .iter()
            .filter(|edge| {
                edge.from == caveat && edge.to == occurrence && edge.relation == Relation::Qualifies
            })
            .count();
        let marked = self
            .taken_back
            .iter()
            .filter(|record| {
                self.symbols.get(&record.caveat) == Some(&caveat)
                    && self.symbols.get(&record.evidence) == Some(&occurrence)
            })
            .count();
        relations > marked
    }

    /// A record holds the take-back caveats it carries: reading it back keeps
    /// them. Nothing in a program that does not take back.
    fn freeze(&self, provenance: &mut Provenance) {
        if !self.take_back_caveats.is_empty() {
            provenance.keep(self.take_back_caveats.iter());
        }
    }

    /// An observation's or predicate's qualification, as its record holds it.
    /// It keeps the take-back caveats it carries except those it takes
    /// through a live relation of evidence it holds: those stay live, so a
    /// take-back reaches them as it reaches the evidence (D012).
    fn freeze_observation(&self, provenance: &mut Provenance) {
        if self.take_back_caveats.is_empty() {
            return;
        }
        let live = self
            .take_back_caveats
            .iter()
            .filter(|caveat| {
                !provenance.kept.contains(*caveat)
                    && self.symbols.get(*caveat).is_some_and(|&from| {
                        provenance.evidence.iter().any(|name| {
                            self.symbols
                                .get(name)
                                .is_some_and(|&to| self.has_live_relation(from, to))
                        })
                    })
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        provenance.keep(self.take_back_caveats.difference(&live));
    }

    /// `provenance` as a record holds it, copied only when that differs.
    fn frozen<'a>(&self, provenance: &'a Provenance) -> std::borrow::Cow<'a, Provenance> {
        if self.take_back_caveats.is_empty() {
            return std::borrow::Cow::Borrowed(provenance);
        }
        let mut copy = provenance.clone();
        self.freeze(&mut copy);
        std::borrow::Cow::Owned(copy)
    }

    /// `unqualify EVIDENCE with CAVEAT`: mark the live relation of each held
    /// occurrence, then take the caveat from each current value that no
    /// longer has it live and does not keep it. See
    /// spec/caveat-take-back-0.1.md, "Meaning".
    fn take_back(
        &mut self,
        evidence: &str,
        caveat: &str,
        only: Option<&str>,
    ) -> Result<(), String> {
        let from = self.symbols[caveat];
        // `only` is one occurrence of the evidence, or a reading of the
        // stream `unqualify latest(…)` names.
        let occurrences = match (only, self.renewals.get(evidence)) {
            (Some(only), _) => vec![only.to_string()],
            (None, Some(renewal)) => renewal.occurrences.clone(),
            (None, None) => vec![evidence.to_string()],
        };
        let marked = occurrences
            .into_iter()
            .filter(|occurrence| {
                !self.is_retired(occurrence)
                    && self
                        .symbols
                        .get(occurrence)
                        .is_some_and(|&to| self.has_live_relation(from, to))
            })
            .collect::<Vec<_>>();
        if marked.is_empty() {
            return Ok(());
        }
        let event = self.last_event.clone().unwrap_or_default();
        let records = Arc::make_mut(&mut self.taken_back);
        for occurrence in &marked {
            records.push(TakeBack {
                evidence: occurrence.clone(),
                caveat: caveat.to_string(),
                sequence: self.sequence,
                event: event.clone(),
                agent: self.last_agent.clone(),
            });
        }
        // An observation's or predicate's record holds what it took through a
        // live relation only while that relation is live (D012).
        let unkept = |lineage: &Provenance| {
            lineage.caveats.contains(caveat)
                && !lineage.kept.contains(caveat)
                && marked
                    .iter()
                    .any(|occurrence| lineage.evidence.contains(occurrence))
                && !lineage.evidence.iter().any(|name| {
                    self.symbols
                        .get(name)
                        .is_some_and(|&id| self.has_live_relation(from, id))
                })
        };
        let observations = self
            .observation_qualifications
            .iter()
            .filter(|(_, lineage)| unkept(lineage))
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        let examinations = self
            .examination_qualifications
            .iter()
            .filter(|(_, lineage)| unkept(lineage))
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        let predicates = self
            .predicate_qualifications
            .iter()
            .flat_map(|(kind, targets)| {
                targets
                    .iter()
                    .filter(|(_, lineage)| unkept(lineage))
                    .map(move |(name, _)| (kind.clone(), name.clone()))
            })
            .collect::<Vec<_>>();
        // Current values only, as a qualify, and their grounds on the same
        // test over the grounds' own evidence. Commitment bases and grounds,
        // reading archives and the journal keep what they recorded.
        let states = (0..self.states.len())
            .map(|slot| {
                let cell = &self.states.cells[slot];
                (slot, unkept(&cell.value.provenance), unkept(&cell.grounds))
            })
            .filter(|&(_, in_value, in_grounds)| in_value || in_grounds)
            .collect::<Vec<_>>();
        for name in observations {
            Arc::make_mut(&mut self.observation_qualifications)
                .get_mut(&name)
                .expect("listed above")
                .caveats
                .remove(caveat);
        }
        for name in examinations {
            Arc::make_mut(&mut self.examination_qualifications)
                .get_mut(&name)
                .expect("listed above")
                .caveats
                .remove(caveat);
        }
        for (kind, name) in predicates {
            Arc::make_mut(&mut self.predicate_qualifications)
                .get_mut(&kind)
                .and_then(|targets| targets.get_mut(&name))
                .expect("listed above")
                .caveats
                .remove(caveat);
        }
        for (slot, in_value, in_grounds) in states {
            self.collector_dirty(CollectorOwner::State(self.states.names[slot].clone()));
            let cell = self.states.cell_mut(slot);
            if in_value {
                cell.value.provenance.caveats.remove(caveat);
            }
            if in_grounds {
                cell.grounds.caveats.remove(caveat);
            }
        }
        self.effects.push(EffectReport::Unqualify {
            evidence: evidence.to_string(),
            caveat: caveat.to_string(),
            occurrences: marked,
        });
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
        // Once per live relation: after a take-back, qualifying again adds a
        // new one. See spec/caveat-take-back-0.1.md.
        if !self.has_live_relation(from, to) {
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
            self.collector_dirty(CollectorOwner::State(self.states.names[slot].clone()));
            let cell = self.states.cell_mut(slot);
            if in_value {
                cell.value.provenance.merge(&added)?;
                cell.value.provenance.merge(guard)?;
            }
            if in_grounds {
                cell.grounds.merge(&added)?;
            }
        }
        // An observation's or predicate's record lists what its guard read,
        // so a later read through it sees the caveat a current value gains.
        // A take-back removes it the same way (D012, D014).
        let listed = |lineage: &Provenance| lineage.evidence.contains(evidence);
        if self.observation_qualifications.values().any(listed)
            || self.examination_qualifications.values().any(listed)
            || self
                .predicate_qualifications
                .values()
                .any(|targets| targets.values().any(listed))
        {
            // Every record merged into lists the evidence, so the caveat's
            // route through it is live: freeze against it as well as the
            // guard, or the caveat is wrongly kept (D020).
            let mut lineage = added
                .union(guard)?
                .union(&Provenance::from_names([evidence.to_string()], [])?)?;
            self.freeze_observation(&mut lineage);
            // Each record whose qualification gains the lineage's evidence
            // references it, so the withdrawal collector must recount that
            // record's references before departure runs (D016).
            let mut changed = BTreeSet::new();
            for (name, record) in Arc::make_mut(&mut self.observation_qualifications).iter_mut() {
                if listed(record) {
                    record.merge(&lineage)?;
                    changed.insert(name.clone());
                }
            }
            for (name, record) in Arc::make_mut(&mut self.examination_qualifications).iter_mut() {
                if listed(record) {
                    record.merge(&lineage)?;
                    changed.insert(name.clone());
                }
            }
            for targets in Arc::make_mut(&mut self.predicate_qualifications).values_mut() {
                for (name, record) in targets.iter_mut() {
                    if listed(record) {
                        record.merge(&lineage)?;
                        changed.insert(name.clone());
                    }
                }
            }
            for name in changed {
                self.collector_dirty(CollectorOwner::Record(name));
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
            self.collector_schedule(&scheduled, false);
            self.release_pin(&scheduled.evidence);
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

    /// Each foreclosed action is a series or a plain commitment some `commit`
    /// names. See spec/caveat-foreclosure-0.1.md, "Syntax".
    fn check_foreclosures(&self) -> Result<(), String> {
        let committed = self
            .rules
            .iter()
            .map(|rule| &rule.effect)
            .chain(
                self.procedures
                    .values()
                    .flat_map(|procedure| procedure.body.iter().map(|step| &step.effect)),
            )
            .filter_map(|effect| match effect {
                Effect::Commit { action, .. } => Some(action.as_str()),
                _ => None,
            })
            .collect::<HashSet<_>>();
        for (series, forecloses) in self.irreversible.iter() {
            for action in forecloses {
                if !self.decision_series.contains_key(action)
                    && !committed.contains(action.as_str())
                {
                    return Err(format!(
                        "{series} forecloses {action}, which is not a decision"
                    ));
                }
            }
        }
        Ok(())
    }

    /// The committed irreversible revision that forecloses `action`, if any:
    /// the earliest by sequence, then by series name.
    fn foreclosing(&self, action: &str) -> Option<&str> {
        self.irreversible
            .iter()
            .filter(|(_, forecloses)| forecloses.iter().any(|closed| closed == action))
            .filter_map(|(series, _)| self.decision_series.get(series)?.revisions.first())
            .min_by_key(|revision| revision.sequence)
            .map(|revision| revision.id.as_str())
    }

    /// Every `unqualify` of a program, after procedures are specialized.
    fn take_back_effects(&self) -> impl Iterator<Item = (&str, &str)> {
        self.program_effects().filter_map(|effect| match effect {
            Effect::Unqualify {
                evidence, caveat, ..
            } => Some((evidence.as_str(), caveat.as_str())),
            _ => None,
        })
    }

    fn program_effects(&self) -> impl Iterator<Item = &Effect> {
        self.rules.iter().map(|rule| &rule.effect).chain(
            self.procedures
                .values()
                .flat_map(|procedure| procedure.body.iter().map(|step| &step.effect)),
        )
    }

    /// The caveats a program takes back, and the rules that make a take-back
    /// exact: see spec/caveat-take-back-0.1.md, "What a take-back caveat may
    /// not do". Uses in rules, procedures and bindings are refused when they
    /// are validated; state initializers name theirs in `extras`.
    fn declare_take_backs(&mut self, extras: &BTreeSet<String>) -> Result<(), String> {
        let mut caveats = BTreeSet::new();
        for (evidence, caveat) in self.take_back_effects() {
            if let Some(stream) = self.reading_streams.get(evidence) {
                // Only `unqualify latest(STREAM)` names a stream. Its readings
                // copy the template's caveats, so a caveat it takes back is
                // never put on the template: see spec/caveat-latest-0.1.md.
                if self.program_effects().any(|effect| {
                    matches!(effect, Effect::Qualify { evidence: target, caveat: added, .. }
                        if *target == stream.template && added == caveat)
                }) {
                    return Err(format!(
                        "unqualify latest({evidence}) with {caveat}: qualify {} with {caveat} puts it on the template, whose caveats every reading copies",
                        stream.template
                    ));
                }
            } else {
                self.require_kind(evidence, "evidence")
                    .map_err(|error| format!("unqualify {evidence} with {caveat}: {error}"))?;
            }
            self.require_kind(caveat, "caveat")
                .map_err(|error| format!("unqualify {evidence} with {caveat}: {error}"))?;
            if let Some((stream, _)) = self
                .reading_streams
                .iter()
                .find(|(_, stream)| stream.template == evidence)
            {
                return Err(format!(
                    "unqualify {evidence} with {caveat}: {evidence} is the template of reading stream {stream}, whose readings copy its caveats"
                ));
            }
            if caveat == WITHDRAWN && self.withdraws() {
                return Err(
                    "unqualify cannot take back withdrawn: nothing undoes a withdrawal".into(),
                );
            }
            caveats.insert(caveat.to_string());
        }
        // Every edge is declared while the program loads.
        for caveat in &caveats {
            let id = self.symbols[caveat];
            for edge in self
                .graph
                .edges
                .iter()
                .filter(|edge| edge.relation == Relation::Qualifies)
            {
                if edge.from == id {
                    return Err(format!(
                        "{caveat} is taken back by unqualify, so a declaration cannot apply it; apply it with qualify"
                    ));
                }
                if edge.to == id {
                    return Err(format!(
                        "{caveat} is taken back by unqualify, so a declared caveat cannot qualify it"
                    ));
                }
            }
            if extras.contains(caveat) {
                return Err(format!(
                    "qualified cannot name {caveat}: it is taken back by unqualify, so only qualify applies it"
                ));
            }
        }
        self.take_back_caveats = Arc::new(caveats);
        self.late_caveats = Arc::new(
            self.program_effects()
                .filter_map(|effect| match effect {
                    Effect::Qualify { caveat, .. } => Some(caveat.clone()),
                    _ => None,
                })
                .collect(),
        );
        Ok(())
    }

    fn not_a_take_back_caveat(&self, caveat: &str, place: &str) -> Result<(), String> {
        if self.take_back_caveats.contains(caveat) {
            return Err(format!(
                "{place} cannot name {caveat}: it is taken back by unqualify, so only qualify applies it"
            ));
        }
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
            // Resolved when the effect runs; see `resolve_occurrence`.
            EvidenceSelector::Caveated { .. } | EvidenceSelector::Occurrence { .. } => None,
        }
    }

    /// Declared reopening triggers for a reading just recorded: each watching
    /// series, in name order, runs `when committed(D) and not reopened(D)
    /// reopen D because latest(STREAM)` with the sample's guard, as that
    /// authored step would. See spec/caveat-reopening-triggers-0.1.md.
    fn run_reopening_triggers(
        &mut self,
        stream: &str,
        stance: Option<&(Relation, String)>,
        changed: bool,
        parameters: &Parameters,
        guard: &Provenance,
        budget: &mut ExecutionBudget,
    ) -> Result<(), DispatchFailure> {
        let triggers = Arc::clone(&self.reopening_triggers);
        for (series, watching, condition) in triggers.iter() {
            if !watching
                .iter()
                .any(|trigger| trigger.matches(stream, stance) && (changed || !trigger.changed))
            {
                continue;
            }
            let reopen = Effect::Reopen {
                action: series.clone(),
                because: EvidenceSelector::Latest(stream.to_string()),
                form: DecisionChange::Reopen,
            };
            self.execute_guarded_effect(condition, &reopen, parameters, guard, budget)?;
        }
        Ok(())
    }

    /// A `permitted by` clause, when its commit runs. A missing, withdrawn or
    /// mismatched grant is a refusal; any other failure keeps its own
    /// classification. Returns the frozen record and the lineage the check
    /// read. See spec/caveat-permission-0.1.md.
    fn check_permission(
        &self,
        action: &str,
        clause: &PermissionClause,
        parameters: &Parameters,
    ) -> Result<(PermissionRecord, Provenance), DispatchFailure> {
        let refuse = |why: String| {
            DispatchFailure::rejected(
                RejectionOrigin::Policy,
                RejectionCode::NotPermitted,
                format!("commit {action} is not permitted: {why}"),
            )
        };
        let (grant, value, mut lineage) = match &clause.grant {
            EvidenceSelector::Named(evidence) => {
                let grant = self.occurrence(evidence).to_string();
                if !self.predicate("observed", &grant)? {
                    return Err(refuse(format!("{grant} is not observed")));
                }
                let lineage = self.qualify(&grant, &[])?;
                (grant, None, lineage)
            }
            EvidenceSelector::Latest(stream) => {
                // An empty stream is "no grant yet", recognised as such rather
                // than by catching the error an empty read raises.
                let Some(grant) = self.reading_streams[stream].current.clone() else {
                    return Err(refuse(format!("{stream} holds no grant yet")));
                };
                let reading = self.latest(stream)?;
                (grant, Some(reading.value), reading.provenance)
            }
            EvidenceSelector::Caveated { .. } | EvidenceSelector::Occurrence { .. } => {
                return Err("permitted by names evidence or latest(STREAM)".into())
            }
        };
        let caveats = lineage.caveats.iter().cloned().collect();
        if let Some(withdrawal) = self.withdrawal_of(&grant) {
            return Err(refuse(format!(
                "{grant} was withdrawn at #{} because {}",
                withdrawal.sequence, withdrawal.because
            )));
        }
        let scope = match &clause.scope {
            None => None,
            Some(expression) => {
                let required = self.evaluate(expression, parameters)?;
                let Value::Number(required_value) = required.value else {
                    return Err("permitted by … for requires a number".into());
                };
                lineage.merge(&required.provenance)?;
                let granted = value.expect("validated: for takes latest(STREAM)");
                if granted != required_value {
                    return Err(refuse(format!(
                        "{grant} permits {granted}, not {required_value}"
                    )));
                }
                Some(PermissionScope {
                    granted,
                    required: required_value,
                })
            }
        };
        Ok((
            PermissionRecord {
                grant,
                scope,
                caveats,
            },
            lineage,
        ))
    }

    fn withdrawal_of(&self, evidence: &str) -> Option<&Withdrawal> {
        self.withdrawals
            .iter()
            .find(|withdrawal| withdrawal.evidence == evidence)
    }

    /// Whether a record of a windowed history has retired.
    pub(crate) fn is_retired(&self, record: &str) -> bool {
        self.retired.contains_key(record)
    }

    /// The index of a history's oldest live record. Retirement always takes
    /// the oldest live record, so the retired records are a prefix.
    fn live_start<T>(&self, records: &[T], id: impl Fn(&T) -> &str) -> usize {
        if self.retired.is_empty() {
            return 0;
        }
        records.partition_point(|record| self.retired.contains_key(id(record)))
    }

    /// The record to retire before one more is added to a full window.
    fn oldest_live_beyond_window<T>(
        &self,
        records: &[T],
        id: impl Fn(&T) -> &str,
        window: usize,
    ) -> Option<String> {
        let start = self.live_start(records, &id);
        (records.len() - start >= window).then(|| id(&records[start]).to_string())
    }

    /// Room for one more record of a history: a full history without a window
    /// refuses; a windowed one refuses only past `MAX_WINDOWED_RECORDS`
    /// records, live and retired. Out of line, because `apply_effect` recurses
    /// through procedures and its frame bounds their depth.
    #[inline(never)]
    fn history_room(&self, history: &str) -> Result<(), DispatchFailure> {
        let windowed = self.windows.contains(history);
        let (held, limit, code, refusal) = if let Some(stream) = self.reading_streams.get(history) {
            let limit = stream.limit;
            (
                stream.occurrences.len(),
                limit,
                RejectionCode::HistoryLimit,
                format!("reading stream {history} reached its history limit {limit}"),
            )
        } else if let Some(series) = self.decision_series.get(history) {
            let limit = series.limit;
            (
                series.revisions.len(),
                limit,
                RejectionCode::HistoryLimit,
                format!("decision series {history} reached its history limit {limit}"),
            )
        } else {
            let renewal = &self.renewals[history];
            let limit = renewal.limit;
            (
                renewal.occurrences.len(),
                limit,
                RejectionCode::RenewalLimit,
                format!("renewable {history} reached its limit {limit}"),
            )
        };
        if windowed && held >= MAX_WINDOWED_RECORDS {
            return Err(DispatchFailure::rejected(
                RejectionOrigin::Limit,
                code,
                format!("{history} holds its {MAX_WINDOWED_RECORDS} records, live and retired"),
            ));
        }
        if !windowed && held >= limit {
            return Err(DispatchFailure::rejected(
                RejectionOrigin::Limit,
                code,
                refusal,
            ));
        }
        Ok(())
    }

    /// Before a windowed history adds a record, retire its oldest live one if
    /// the window is full. A history without a window retires nothing.
    #[inline(never)]
    fn retire_oldest(&mut self, history: &str) {
        if !self.windows.contains(history) {
            return;
        }
        let oldest = if let Some(stream) = self.reading_streams.get(history) {
            self.oldest_live_beyond_window(&stream.occurrences, |record| &record.id, stream.limit)
        } else {
            let renewal = &self.renewals[history];
            self.oldest_live_beyond_window(&renewal.occurrences, String::as_str, renewal.limit)
        };
        if let Some(oldest) = oldest {
            self.retire(history, oldest);
        }
    }

    fn retire(&mut self, history: &str, record: String) {
        self.collector.queue(&record);
        Arc::make_mut(&mut self.departure_candidates).insert(record.clone());
        Arc::make_mut(&mut self.retired).insert(record.clone(), self.sequence);
        self.effects.push(EffectReport::Retire {
            history: history.to_string(),
            record,
        });
    }

    /// Before a journal entry is added under `journal window N`.
    fn retire_journal_entry(&mut self) {
        let Some(window) = self.journal_window else {
            return;
        };
        let names = (0..self.journal.len())
            .map(|index| self.journal_name(index))
            .collect::<Vec<_>>();
        if let Some(oldest) = self.oldest_live_beyond_window(&names, String::as_str, window) {
            let index = names.iter().position(|name| name == &oldest).unwrap();
            let released = self.journal[index]
                .because
                .iter()
                .chain(&self.journal[index].permitted_by)
                .cloned()
                .collect::<Vec<_>>();
            for record in released {
                self.release_pin(&record);
            }
            self.retire("journal", oldest);
        }
    }

    /// A value newly read from history carries `withdrawn` if what it rests on
    /// has since been withdrawn, and the live caveats of the evidence the
    /// record lists. The record itself is unchanged.
    fn with_current_withdrawals(&self, mut provenance: Provenance) -> Result<Provenance, String> {
        self.read_live(&mut provenance)?;
        if provenance
            .evidence
            .iter()
            .any(|evidence| self.withdrawal_of(evidence).is_some())
        {
            provenance.merge(&Provenance::from_names([], [WITHDRAWN.to_string()])?)?;
        }
        Ok(provenance)
    }

    /// A record's own caveats are kept; the evidence it lists is read live
    /// (D015). A value read through a record gains each caveat that now
    /// qualifies that evidence, as a value that already held it gained the
    /// caveat when it was added, so when the value is computed does not
    /// matter. A caveat gained here is live: a take-back reaches it. See
    /// spec/caveat-caveat-sources-0.1.md.
    fn read_live(&self, provenance: &mut Provenance) -> Result<(), String> {
        let reached = self
            .live_late_relations(provenance)
            .into_iter()
            .map(|(caveat, _)| caveat)
            .collect::<BTreeSet<_>>();
        if reached.is_empty() {
            return Ok(());
        }
        // The caveat and whatever qualifies it, as `qualify` adds them.
        let names = self
            .symbols
            .iter()
            .filter(|(_, id)| reached.contains(id))
            .map(|(name, _)| name.clone())
            .chain(self.incoming_caveats(reached.iter().copied()))
            .collect::<Vec<_>>();
        provenance.merge(&Provenance::from_names([], names)?)
    }

    /// Each live `C qualifies E` relation for a late caveat C that
    /// `provenance` lacks, where it lists E: what `read_live` adds, as
    /// (C, E) pairs.
    fn live_late_relations(&self, provenance: &Provenance) -> BTreeSet<(NodeId, NodeId)> {
        if self.late_caveats.is_empty() {
            return BTreeSet::new();
        }
        // A handful of names each: sorted lists, not hash sets.
        let mut late = self
            .late_caveats
            .iter()
            .filter(|caveat| !provenance.caveats.contains(*caveat))
            .filter_map(|caveat| self.symbols.get(caveat).copied())
            .collect::<Vec<_>>();
        if late.is_empty() {
            return BTreeSet::new();
        }
        late.sort_unstable();
        let mut listed = provenance
            .evidence
            .iter()
            .filter_map(|name| self.symbols.get(name).copied())
            .collect::<Vec<_>>();
        listed.sort_unstable();
        self.graph
            .edges
            .iter()
            .filter(|edge| {
                edge.relation == Relation::Qualifies
                    && late.binary_search(&edge.from).is_ok()
                    && listed.binary_search(&edge.to).is_ok()
            })
            .map(|edge| (edge.from, edge.to))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|&(caveat, evidence)| self.has_live_relation(caveat, evidence))
            .collect()
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
            "permission_withdrawn" => {
                let current = match self.decision_series.get(name) {
                    Some(series) => series.current.as_deref(),
                    None => Some(name),
                };
                current
                    .and_then(|current| self.commitment_permissions.get(current))
                    .and_then(|record| self.withdrawal_of(&record.grant))
                    .into_iter()
                    .collect()
            }
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

    /// A predicate asked of `occurrence(EVIDENCE, INDEX)` arrives as kind
    /// `occurrence:KIND` and target `EVIDENCE#INDEX`, and one asked of
    /// `latest(NAME)` as kind `latest:KIND` and target `NAME`. Returns `None` for any
    /// other predicate, and otherwise the plain kind with the held occurrence
    /// the index names, or `None` when it names none: the predicate is then
    /// false. See spec/caveat-earlier-occurrence-0.1.md.
    fn at_occurrence<'a>(&self, kind: &'a str, name: &str) -> Option<(&'a str, Option<String>)> {
        if kind == reactive_expr::AT_REVISION || kind == reactive_expr::AT_REVISION_RESOLVED {
            let revision = name
                .rsplit_once('#')
                .and_then(|(series, index)| self.held_revision(series, index.parse().ok()?));
            let kind = if kind == reactive_expr::AT_REVISION {
                "rests_on_withdrawn"
            } else {
                reactive_expr::RESOLVED
            };
            return Some((kind, revision));
        }
        if let Some(kind) = kind.strip_prefix(reactive_expr::AT_LATEST) {
            return Some((kind, self.latest_target(name)));
        }
        let kind = kind.strip_prefix(reactive_expr::AT_OCCURRENCE)?;
        let held = name
            .rsplit_once('#')
            .and_then(|(evidence, index)| Some((evidence, index.parse::<f64>().ok()?)))
            .and_then(|(evidence, index)| self.held_occurrence(evidence, index));
        Some((kind, held))
    }

    /// `in_basis(…)`, whose target arrives as `SERIES#K|KIND|NAME#N`: whether
    /// the revision's grounds hold the evidence named. The answer rests on the
    /// revision's record, as `committed(…)` asked of it does; a revision or a
    /// reference that names nothing held answers false. See
    /// spec/caveat-in-basis-0.1.md.
    fn in_basis(&self, target: &str, grounds: bool) -> Result<Tracked<bool>, String> {
        let malformed = || format!("in_basis received a malformed target {target}");
        let mut parts = target.splitn(3, '|');
        let (Some(revision), Some(kind), Some(reference)) =
            (parts.next(), parts.next(), parts.next())
        else {
            return Err(malformed());
        };
        let (series, k) = revision.rsplit_once('#').ok_or_else(malformed)?;
        let (name, n) = reference.rsplit_once('#').ok_or_else(malformed)?;
        let revision = if k.is_empty() {
            match self.decision_series.get(series) {
                Some(series) => series.current.clone(),
                None => self
                    .predicate("committed", series)?
                    .then(|| series.to_string()),
            }
        } else {
            k.parse().ok().and_then(|k| self.held_revision(series, k))
        };
        let evidence = match kind {
            "named" => Some(self.occurrence(name).to_string()),
            "latest" => self.latest_target(name),
            "at" => n.parse().ok().and_then(|n| self.held_occurrence(name, n)),
            _ => return Err(malformed()),
        };
        let (Some(revision), Some(evidence)) = (revision, evidence) else {
            return Ok(Tracked::plain(false));
        };
        let held = self
            .commitment_grounds
            .get(&revision)
            .is_some_and(|grounds| grounds.evidence.contains(&evidence));
        let record = if grounds {
            self.predicate_grounds("committed", &revision)?
        } else {
            self.predicate_tracked("committed", &revision)?
        };
        Tracked::new(held, record.provenance)
    }

    /// The revision of a decision series that a 1-based index names: `k` is
    /// `SERIES@k`, the `k`th revision committed, current or past. `None` when
    /// the index is not a whole number naming a revision made so far. See
    /// spec/caveat-past-revisions-0.1.md.
    fn held_revision(&self, series: &str, index: f64) -> Option<String> {
        let revisions = &self.decision_series.get(series)?.revisions;
        if index.fract() != 0.0 || index < 1.0 || index > revisions.len() as f64 {
            return None;
        }
        Some(revisions[index as usize - 1].id.clone())
    }

    /// The held occurrence of renewable evidence that a 1-based index names:
    /// 1 is the declared evidence, `k` is `EVIDENCE@k`. `None` when the index
    /// is not a whole number naming an occurrence that exists and has not
    /// retired.
    fn held_occurrence(&self, evidence: &str, index: f64) -> Option<String> {
        if index < 1.0 || index.fract() != 0.0 {
            return None;
        }
        // A stream's `k`th reading is `STREAM@k`, while the stream holds it:
        // spec/caveat-reading-index-0.1.md.
        if let Some(stream) = self.reading_streams.get(evidence) {
            let name = format!("{evidence}@{index}");
            return stream
                .occurrences
                .iter()
                .any(|occurrence| occurrence.id == name)
                .then_some(name)
                .filter(|name| !self.is_retired(name));
        }
        let renewal = self.renewals.get(evidence)?;
        let name = if index == 1.0 {
            evidence.to_string()
        } else {
            format!("{evidence}@{index}")
        };
        (renewal.occurrences.contains(&name) && !self.is_retired(&name)).then_some(name)
    }

    /// `occurrence(EVIDENCE, INDEX)` in an effect: the held occurrence the
    /// index names, and the index's lineage. An index that names none refuses
    /// the event as `evaluation/expression`, as `history_at` does.
    fn resolve_occurrence(
        &mut self,
        evidence: &str,
        index: &Expr,
        parameters: &Parameters,
    ) -> Result<(String, Provenance), DispatchFailure> {
        let tracked = self.evaluate(index, parameters)?;
        let Value::Number(number) = tracked.value else {
            return Err(format!("occurrence({evidence}, …) requires a numeric index").into());
        };
        let Some(occurrence) = self.held_occurrence(evidence, number) else {
            return Err(EvalError::new(
                EvalFailure::HistoryIndex,
                format!("occurrence({evidence}, {number}) names no held occurrence of {evidence}"),
            )
            .into());
        };
        self.collector_dirty(CollectorOwner::Record(occurrence.clone()));
        Ok((occurrence, tracked.provenance))
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
            retired: (*self.retired).clone(),
            windows: self
                .windows
                .iter()
                .cloned()
                .chain(self.journal_window.map(|_| "journal".to_string()))
                .collect(),
            elapsed: self.elapsed,
            renewals: (*self.renewals).clone(),
            observations: (*self.observations).clone(),
            identifiers: self.identifiers.texts().to_vec(),
            withdrawals: (*self.withdrawals).clone(),
            taken_back: (*self.taken_back).clone(),
            commitment_permissions: (*self.commitment_permissions).clone(),
            retention_reasons: (*self.retention_reasons).clone(),
            scheduled_qualifications: (*self.scheduled).clone(),
            cues: self.cues.clone(),
            cue_qualifications: self.cue_qualifications.clone(),
            qualified_values: self.states.values(),
            commitment_bases: (*self.commitment_bases).clone(),
            reading_streams: (*self.reading_streams).clone(),
            decision_series: (*self.decision_series).clone(),
            irreversible: (*self.irreversible).clone(),
            observation_qualifications: (*self.observation_qualifications).clone(),
            examination_qualifications: (*self.examination_qualifications).clone(),
            reopening_qualifications: (*self.reopening_qualifications).clone(),
            predicate_qualifications: (*self.predicate_qualifications).clone(),
            controls: (*self.controls).clone(),
            clock: self.clock.clone(),
            schema: REACTIVE_SCHEMA.into(),
            source_id: self.source_id.clone(),
            source_sha256: self.source_sha256.clone(),
            source_unrecorded_through: self.source_unrecorded_through,
            sequence: self.sequence,
            last_event: self.last_event.clone(),
            last_agent: self.last_agent.clone(),
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
            stance,
            note,
        } => Effect::Sample {
            stream: rename(stream),
            value: value.rename_symbols(names),
            stance: stance
                .as_ref()
                .map(|(relation, claim)| (*relation, rename(claim))),
            note: note
                .as_ref()
                .map(|note| Box::new(note.rename_symbols(names))),
        },
        Effect::Emit { .. } | Effect::Reject { .. } => effect.clone(),
        Effect::Qualify {
            evidence,
            caveat,
            after,
            index,
            latest,
        } => Effect::Qualify {
            evidence: rename(evidence),
            caveat: rename(caveat),
            after: after.as_ref().map(|after| after.rename_symbols(names)),
            index: index
                .as_ref()
                .map(|index| Box::new(index.rename_symbols(names))),
            latest: *latest,
        },
        Effect::Withdraw { target, because } => Effect::Withdraw {
            target: match target {
                EvidenceSelector::Named(evidence) => EvidenceSelector::Named(rename(evidence)),
                EvidenceSelector::Latest(stream) => EvidenceSelector::Latest(rename(stream)),
                EvidenceSelector::Occurrence { evidence, index } => EvidenceSelector::Occurrence {
                    evidence: rename(evidence),
                    index: Box::new(index.rename_symbols(names)),
                },
                selector => selector.clone(),
            },
            because: rename(because),
        },
        Effect::Unqualify {
            evidence,
            caveat,
            index,
            latest,
        } => Effect::Unqualify {
            evidence: rename(evidence),
            caveat: rename(caveat),
            index: index
                .as_ref()
                .map(|index| Box::new(index.rename_symbols(names))),
            latest: *latest,
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
        Effect::Observe { evidence } => Effect::Observe {
            evidence: rename(evidence),
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
            citing,
            permission,
            retaining,
            reasons,
        } => Effect::Commit {
            action: rename(action),
            reason: reason.clone(),
            using: using.as_ref().map(|value| value.rename_symbols(names)),
            citing: citing
                .iter()
                .map(|value| value.rename_symbols(names))
                .collect(),
            permission: permission.as_ref().map(|clause| {
                Box::new(PermissionClause {
                    grant: match &clause.grant {
                        EvidenceSelector::Named(grant) => EvidenceSelector::Named(rename(grant)),
                        EvidenceSelector::Latest(stream) => {
                            EvidenceSelector::Latest(rename(stream))
                        }
                        selector => selector.clone(),
                    },
                    scope: clause
                        .scope
                        .as_ref()
                        .map(|scope| scope.rename_symbols(names)),
                })
            }),
            retaining: retaining.iter().map(rename).collect(),
            reasons: reasons
                .iter()
                .map(|(caveat, evidence, reading)| {
                    (
                        rename(caveat),
                        rename(evidence),
                        reading.rename_symbols(names),
                    )
                })
                .collect(),
        },
        Effect::Reopen {
            action,
            because,
            form,
        } => Effect::Reopen {
            form: *form,
            action: rename(action),
            because: rename_selector(because, names),
        },
        Effect::Resolve(resolution) => Effect::Resolve(Box::new(Resolution {
            action: rename(&resolution.action),
            revision: resolution
                .revision
                .as_ref()
                .map(|index| index.rename_symbols(names)),
            value: resolution.value.rename_symbols(names),
            because: rename_selector(&resolution.because, names),
            materialized: resolution.materialized.iter().map(rename).collect(),
        })),
    }
}

/// A `because` selector of a reopening or an outcome, with graph symbol and
/// history names replaced.
fn rename_selector(
    because: &EvidenceSelector,
    names: &HashMap<String, String>,
) -> EvidenceSelector {
    let rename = |name: &String| names.get(name).cloned().unwrap_or_else(|| name.clone());
    match because {
        EvidenceSelector::Named(evidence) => EvidenceSelector::Named(rename(evidence)),
        EvidenceSelector::Latest(stream) => EvidenceSelector::Latest(rename(stream)),
        EvidenceSelector::Caveated { state, caveat } => EvidenceSelector::Caveated {
            state: state.clone(),
            caveat: rename(caveat),
        },
        EvidenceSelector::Occurrence { evidence, index } => EvidenceSelector::Occurrence {
            evidence: rename(evidence),
            index: Box::new(index.rename_symbols(names)),
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

/// The source's SHA-256 (spec/caveat-save-0.1.md, "Source digest").
fn source_digest(source: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(source.as_bytes()))
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
    syntax_word_spans(input)
        .into_iter()
        .map(|(start, end)| &input[start..end])
        .collect()
}

/// The byte range of each word `syntax_words` splits `input` into.
pub(crate) fn syntax_word_spans(input: &str) -> Vec<(usize, usize)> {
    let mut words = Vec::new();
    let mut start = None;
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in input.char_indices() {
        if !quoted && ch.is_whitespace() {
            if let Some(start) = start.take() {
                words.push((start, index));
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
        words.push((start, input.len()));
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
            | "journal"
            | "identifiers"
            | "agent"
            | "proc"
            | "define"
    ) {
        return None;
    }
    Some((|| {
        match keyword {
        "proc" => parse_procedure(line, position),
        "define" => parse_define(line),
        "fn" => parse_function(line),
        "readings" => match words.as_slice() {
            ["readings", name, "from", template, clause @ ("limit" | "window"), limit] => {
                Ok(Directive::Readings {
                    name: identifier(name)?,
                    template: identifier(template)?,
                    limit: limit
                        .parse()
                        .map_err(|_| format!("reading {clause} must be an unsigned integer"))?,
                    window: *clause == "window",
                })
            }
            _ => Err("readings expects NAME from EVIDENCE limit CAPACITY or window N".into()),
        },
        "renewable" => match words.as_slice() {
            ["renewable", evidence, clause @ ("limit" | "window"), limit] => {
                Ok(Directive::Renewable {
                    evidence: identifier(evidence)?,
                    limit: limit
                        .parse()
                        .map_err(|_| format!("renewable {clause} must be an unsigned integer"))?,
                    window: *clause == "window",
                })
            }
            _ => Err("renewable expects EVIDENCE limit CAPACITY or window N".into()),
        },
        "journal" => match words.as_slice() {
            ["journal", "window", limit] => Ok(Directive::JournalWindow {
                limit: limit
                    .parse()
                    .map_err(|_| "journal window must be an unsigned integer")?,
            }),
            _ => Err("journal expects window N".into()),
        },
        "agent" => match words.as_slice() {
            ["agent", name] => Ok(Directive::Agent {
                name: identifier(name)?,
                sends: Vec::new(),
            }),
            ["agent", name, "sends", events @ ..] if !events.is_empty() => {
                let mut sends = Vec::<String>::new();
                for event in events.join(" ").split(',') {
                    let event = match event.split_whitespace().collect::<Vec<_>>()[..] {
                        [event] => identifier(event)?,
                        _ => {
                            return Err(format!(
                                "sends takes event names separated by commas, not {}",
                                event.trim()
                            ))
                        }
                    };
                    if sends.contains(&event) {
                        return Err(format!("agent {name} lists event {event} twice"));
                    }
                    sends.push(event);
                }
                Ok(Directive::Agent {
                    name: identifier(name)?,
                    sends,
                })
            }
            _ => Err("agent expects NAME [sends EVENT, …]".into()),
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
            // spec/caveat-windows-0.1.md, "Decision series: not in rc.15".
            ["decisions", _, "window", ..] => Err(
                "a decision series has no window yet: it waits on the revision-lineage rule (spec/caveat-windows-0.1.md)".into(),
            ),
            ["decisions", name, "limit", limit, rest @ ..] => {
                // spec/caveat-foreclosure-0.1.md: `irreversible [forecloses
                // ACTION, …]` comes before any `reopened by`.
                let (irreversible, rest) = match rest {
                    ["irreversible", tail @ ..] => {
                        if limit.parse::<usize>() != Ok(1) {
                            return Err(format!(
                                "an irreversible series holds one revision: decisions {name} limit 1 irreversible"
                            ));
                        }
                        let end = tail
                            .windows(2)
                            .position(|pair| pair == ["reopened", "by"])
                            .unwrap_or(tail.len());
                        let (foreclosure, rest) = tail.split_at(end);
                        let forecloses = match foreclosure {
                            [] => Vec::new(),
                            ["forecloses", actions @ ..] if !actions.is_empty() => {
                                let mut forecloses = Vec::<String>::new();
                                for action in actions.join(" ").split(',') {
                                    let action = match action.split_whitespace().collect::<Vec<_>>()[..] {
                                        [action] => identifier(action)?,
                                        _ => return Err(format!(
                                            "forecloses takes decision names separated by commas, not {}",
                                            action.trim()
                                        )),
                                    };
                                    if action == *name {
                                        return Err(format!("{name} cannot foreclose itself"));
                                    }
                                    if forecloses.contains(&action) {
                                        return Err(format!("duplicate foreclosed decision {action}"));
                                    }
                                    forecloses.push(action);
                                }
                                forecloses
                            }
                            _ => return Err(format!(
                                "decisions expects NAME limit 1 irreversible [forecloses ACTION, …] [reopened by STREAM, …], not {}",
                                foreclosure.join(" ")
                            )),
                        };
                        (Some(forecloses), rest)
                    }
                    _ if rest.contains(&"forecloses") && !rest.starts_with(&["reopened", "by"]) => {
                        return Err(format!(
                            "forecloses follows irreversible: decisions {name} limit 1 irreversible forecloses ACTION, …"
                        ));
                    }
                    _ => (None, rest),
                };
                let triggers = match rest {
                    [] => Vec::new(),
                    ["reopened", "by", triggers @ ..] if !triggers.is_empty() => triggers
                        .join(" ")
                        .split(',')
                        .map(|trigger| {
                            let words = trigger.split_whitespace().collect::<Vec<_>>();
                            // `changed` leads only before a stream, so a stream
                            // named `changed` keeps both of its forms.
                            let (changed, words) = match words[..] {
                                ["changed", _] | ["changed", _, _, _] => (true, &words[1..]),
                                _ => (false, &words[..]),
                            };
                            match *words {
                                [stream] => Ok(ReopeningTrigger {
                                    stream: identifier(stream)?,
                                    filter: None,
                                    changed,
                                }),
                                [stream, relation @ ("supporting" | "opposing"), claim] => {
                                    Ok(ReopeningTrigger {
                                        stream: identifier(stream)?,
                                        filter: Some((
                                            if relation == "supporting" {
                                                Relation::Supports
                                            } else {
                                                Relation::Opposes
                                            },
                                            identifier(claim)?,
                                        )),
                                        changed,
                                    })
                                }
                                _ => Err(format!(
                                    "reopened by takes [changed] STREAM [supporting|opposing CLAIM], not {}",
                                    trigger.trim()
                                )),
                            }
                        })
                        .collect::<Result<_, String>>()?,
                    _ => return Err("decisions expects NAME limit CAPACITY [reopened by STREAM, …]".into()),
                };
                Ok(Directive::Decisions {
                    name: identifier(name)?,
                    limit: limit
                        .parse()
                        .map_err(|_| "decision limit must be an unsigned integer".to_string())?,
                    triggers,
                    irreversible,
                })
            }
            _ => Err("decisions expects NAME limit CAPACITY [reopened by STREAM, …]".into()),
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
                integer: false,
            })),
            ["clock", event, "every", step, "integer"] => Ok(Directive::Clock(Clock {
                event: identifier(event)?,
                step: Number::parse(step)?,
                integer: true,
            })),
            _ => Err("clock expects EVENT every STEP [integer]".into()),
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
    }
    })())
}

fn parse_guarded_effect(words: &[&str]) -> Result<GuardedEffect, String> {
    // Effect words are legal identifiers inside expressions. Select a complete
    // effect outside parentheses, rather than splitting at the first keyword.
    let candidates = effect_candidates(words);
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

/// Where the loader finds the effect in a rule's words after `on EVENT`: the
/// first effect word outside parentheses that begins a complete effect. None
/// when no word does, and the rule cannot load.
pub(crate) fn complete_effect_at(words: &[&str]) -> Option<usize> {
    effect_candidates(words)
        .into_iter()
        .find(|index| parse_effect(&words[*index..]).is_ok())
}

/// The effect words outside parentheses, each where an effect may begin.
fn effect_candidates(words: &[&str]) -> Vec<usize> {
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
                    | "unqualify"
                    | "renew"
                    | "withdraw"
            )
        {
            candidates.push(index);
        }
        // `hold` and `release` came after 1.0 and are legal names: one is an
        // effect word only two words before a `because`, so a rule that does
        // not load reports the error it reported before
        // (spec/caveat-holds-0.1.md).
        if depth == 0
            && matches!(*token, "hold" | "release")
            && words.get(index + 2) == Some(&"because")
        {
            candidates.push(index);
        }
        // `resolve` came in 1.3 and is a legal name: it is an effect word only
        // before a name and `=`, or before `revision(`
        // (spec/caveat-outcomes-0.1.md).
        if depth == 0
            && *token == "resolve"
            && (words.get(index + 2) == Some(&"=")
                || words
                    .get(index + 1)
                    .is_some_and(|next| next.starts_with("revision(") || *next == "revision"))
        {
            candidates.push(index);
        }
        depth += expression_depth_delta(token);
    }
    candidates
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
    let name = name.trim();
    if let Some((function_name, _)) = name.split_once('(') {
        if let Ok(function_name) = identifier(function_name.trim()) {
            return Err(format!(
                "define declares a named expression and does not take parameters; use define {function_name} = EXPRESSION; use fn {function_name}(...) = EXPRESSION only for a pure function with explicit parameters, which cannot capture state, history or evidence"
            ));
        }
    }
    Ok(Directive::Define {
        name: identifier(name)?,
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

/// `occurrence(EVIDENCE, INDEX)` as an effect's target or reason, or `None`
/// for any other text. See spec/caveat-earlier-occurrence-0.1.md.
/// `occurrence(EVIDENCE, INDEX)`, or `reading(STREAM, INDEX)`, which names a
/// stream's reading the same way: spec/caveat-reading-index-0.1.md.
fn occurrence_selector(text: &str) -> Result<Option<(String, Box<Expr>)>, String> {
    let Some(arguments) = ["occurrence", "reading"].iter().find_map(|selector| {
        text.strip_prefix(selector)
            .map(str::trim_start)
            .and_then(|rest| rest.strip_prefix('('))
    }) else {
        return Ok(None);
    };
    let arguments = arguments
        .trim_end()
        .strip_suffix(')')
        .ok_or("occurrence selector requires a closing parenthesis")?;
    let (evidence, index) = arguments
        .split_once(',')
        .ok_or("occurrence selector requires EVIDENCE, INDEX")?;
    Ok(Some((
        identifier(evidence.trim())?,
        Box::new(reactive_expr::parse_unresolved(index.trim())?),
    )))
}

/// The evidence selector after a `because` of `reopen`, `hold`, `release`
/// or `resolve`: a name, an occurrence, `latest(STREAM)` or
/// `caveated(STATE, CAVEAT)`.
fn parse_selector(selector: &str) -> Result<EvidenceSelector, String> {
    Ok(
        if let Some(arguments) = selector
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
        } else if let Some((evidence, index)) = occurrence_selector(selector)? {
            EvidenceSelector::Occurrence { evidence, index }
        } else {
            EvidenceSelector::Named(identifier(selector)?)
        },
    )
}

/// `resolve TARGET = VALUE because SELECTOR [materialized C, …]`, with the
/// words after `resolve`. See spec/caveat-outcomes-0.1.md.
fn parse_resolution(words: &[&str]) -> Result<Effect, String> {
    let invalid = || format!("invalid reactive effect: resolve {}", words.join(" "));
    let equals = words
        .iter()
        .position(|word| *word == "=")
        .ok_or_else(invalid)?;
    let because = words
        .iter()
        .position(|word| *word == "because")
        .filter(|&because| because > equals + 1)
        .ok_or_else(invalid)?;
    let target = words[..equals].join(" ");
    let (action, revision) = match target
        .strip_prefix("revision")
        .map(str::trim_start)
        .and_then(|rest| rest.strip_prefix('('))
    {
        Some(arguments) => {
            let arguments = arguments
                .trim_end()
                .strip_suffix(')')
                .ok_or("revision selector requires a closing parenthesis")?;
            let (series, index) = arguments
                .split_once(',')
                .ok_or("revision selector requires SERIES, INDEX")?;
            (
                identifier(series.trim())?,
                Some(reactive_expr::parse_unresolved(index.trim())?),
            )
        }
        None => (identifier(&target)?, None),
    };
    let value = reactive_expr::parse_unresolved(&words[equals + 1..because].join(" "))?;
    let rest = &words[because + 1..];
    let (selector, materialized) = match rest.iter().position(|word| *word == "materialized") {
        Some(at) => (&rest[..at], Some(&rest[at + 1..])),
        None => (rest, None),
    };
    if selector.is_empty() {
        return Err(invalid());
    }
    let mut caveats = Vec::new();
    if let Some(named) = materialized {
        let named = named.join(" ");
        for caveat in named.split(',') {
            let caveat = identifier(caveat.trim())?;
            if caveats.contains(&caveat) {
                return Err(format!("duplicate materialized caveat {caveat}"));
            }
            caveats.push(caveat);
        }
    }
    Ok(Effect::Resolve(Box::new(Resolution {
        action,
        revision,
        value,
        because: parse_selector(&selector.join(" "))?,
        materialized: caveats,
    })))
}

fn parse_effect(words: &[&str]) -> Result<Effect, String> {
    let invalid = || format!("invalid reactive effect: {}", words.join(" "));
    match words {
        [verb @ ("qualify" | "unqualify"), rest @ ..]
            if ["occurrence", "reading"].iter().any(|selector| {
                rest.join(" ")
                    .strip_prefix(selector)
                    .is_some_and(|rest| rest.trim_start().starts_with('('))
            }) =>
        {
            let split = rest
                .iter()
                .position(|word| *word == "with")
                .ok_or_else(invalid)?;
            let (evidence, index) =
                occurrence_selector(&rest[..split].join(" "))?.ok_or_else(invalid)?;
            match (*verb, &rest[split + 1..]) {
                ("qualify", [caveat]) => Ok(Effect::Qualify {
                    evidence,
                    caveat: identifier(caveat)?,
                    after: None,
                    index: Some(index),
                    latest: false,
                }),
                ("qualify", [caveat, "after", delay @ ..]) if !delay.is_empty() => {
                    Ok(Effect::Qualify {
                        evidence,
                        caveat: identifier(caveat)?,
                        after: Some(reactive_expr::parse_unresolved(&delay.join(" "))?),
                        index: Some(index),
                        latest: false,
                    })
                }
                ("unqualify", [caveat]) => Ok(Effect::Unqualify {
                    evidence,
                    caveat: identifier(caveat)?,
                    index: Some(index),
                    latest: false,
                }),
                _ => Err(invalid()),
            }
        }
        // `qualify latest(NAME) …` and `unqualify latest(NAME) …`: see
        // spec/caveat-latest-0.1.md.
        [verb @ ("qualify" | "unqualify"), rest @ ..]
            if rest
                .join(" ")
                .strip_prefix("latest")
                .is_some_and(|rest| rest.trim_start().starts_with('(')) =>
        {
            let split = rest
                .iter()
                .position(|word| *word == "with")
                .ok_or_else(invalid)?;
            let target = rest[..split].join(" ");
            let name = identifier(
                target
                    .strip_prefix("latest")
                    .map(str::trim)
                    .and_then(|rest| rest.strip_prefix('('))
                    .and_then(|rest| rest.trim_end().strip_suffix(')'))
                    .ok_or_else(|| format!("{verb} latest requires latest(NAME)"))?
                    .trim(),
            )?;
            match (*verb, &rest[split + 1..]) {
                ("qualify", [caveat]) => Ok(Effect::Qualify {
                    evidence: name,
                    caveat: identifier(caveat)?,
                    after: None,
                    index: None,
                    latest: true,
                }),
                ("qualify", [caveat, "after", delay @ ..]) if !delay.is_empty() => {
                    Ok(Effect::Qualify {
                        evidence: name,
                        caveat: identifier(caveat)?,
                        after: Some(reactive_expr::parse_unresolved(&delay.join(" "))?),
                        index: None,
                        latest: true,
                    })
                }
                ("unqualify", [caveat]) => Ok(Effect::Unqualify {
                    evidence: name,
                    caveat: identifier(caveat)?,
                    index: None,
                    latest: true,
                }),
                _ => Err(invalid()),
            }
        }
        ["qualify", evidence, "with", caveat] => Ok(Effect::Qualify {
            evidence: identifier(evidence)?,
            caveat: identifier(caveat)?,
            after: None,
            index: None,
            latest: false,
        }),
        ["qualify", evidence, "with", caveat, "after", rest @ ..] if !rest.is_empty() => {
            Ok(Effect::Qualify {
                evidence: identifier(evidence)?,
                caveat: identifier(caveat)?,
                after: Some(reactive_expr::parse_unresolved(&rest.join(" "))?),
                index: None,
                latest: false,
            })
        }
        ["unqualify", evidence, "with", caveat] => Ok(Effect::Unqualify {
            evidence: identifier(evidence)?,
            caveat: identifier(caveat)?,
            index: None,
            latest: false,
        }),
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
                None => match occurrence_selector(&target)? {
                    Some((evidence, index)) => EvidenceSelector::Occurrence { evidence, index },
                    None => EvidenceSelector::Named(identifier(&target)?),
                },
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
        ["sample", stream, "=", rest @ ..] if !rest.is_empty() => {
            // The stance is optional: spec/caveat-neutral-sample-0.1.md.
            const FORM: &str =
                "sample expects STREAM = EXPRESSION [supports|opposes CLAIM] [noting EXPRESSION]";
            // `noting` is a word only where what follows parses: a 1.0
            // program may name a state `noting` (spec/caveat-reading-notes-0.1.md).
            let noting = rest
                .iter()
                .rposition(|word| *word == "noting")
                .and_then(|at| {
                    if at == 0 || at + 1 == rest.len() {
                        return None;
                    }
                    reactive_expr::parse_unresolved(&format!(
                        "id_text({})",
                        rest[at + 1..].join(" ")
                    ))
                    .ok()
                    .map(|note| (at, note))
                });
            let (rest, note) = match noting {
                Some((at, note)) => (&rest[..at], Some(note)),
                None => (rest, None),
            };
            let relation_of = |word: &str| match word {
                "supports" => Some(Relation::Supports),
                "opposes" => Some(Relation::Opposes),
                _ => None,
            };
            let (value, stance) = match rest {
                [value @ .., word, claim] if !value.is_empty() && relation_of(word).is_some() => (
                    value,
                    Some((relation_of(word).unwrap(), identifier(claim)?)),
                ),
                _ => (rest, None),
            };
            if value.iter().any(|word| relation_of(word).is_some()) {
                return Err(FORM.into());
            }
            Ok(Effect::Sample {
                stream: identifier(stream)?,
                value: reactive_expr::parse_unresolved(&value.join(" "))
                    .map_err(|error| format!("{FORM}: {error}"))?,
                stance,
                note: note.map(Box::new),
            })
        }
        ["sample", ..] => Err(
            "sample expects STREAM = EXPRESSION [supports|opposes CLAIM] [noting EXPRESSION]"
                .into(),
        ),
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
        ["reveal", evidence] => Ok(Effect::Observe {
            evidence: identifier(evidence)?,
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
            // `CAVEAT [because EVIDENCE], …`. See
            // spec/caveat-retention-reasons-0.1.md.
            type Retained = (Vec<String>, Vec<(String, String, Expr)>);
            let parse_retained = |names: &[&str]| -> Result<Retained, String> {
                if names.is_empty() {
                    return Err("retaining requires at least one caveat".into());
                }
                let (mut retained, mut reasons) = (Vec::new(), Vec::new());
                for item in names.join(" ").split(',') {
                    match item.split_whitespace().collect::<Vec<_>>()[..] {
                        [caveat] => retained.push(identifier(caveat)?),
                        [caveat, "because", evidence] => {
                            let (caveat, evidence) = (identifier(caveat)?, identifier(evidence)?);
                            let reading = reactive_expr::parse_unresolved(&format!(
                                "qualified(1, {evidence})"
                            ))?;
                            retained.push(caveat.clone());
                            reasons.push((caveat, evidence, reading));
                        }
                        _ => {
                            return Err(format!(
                                "retaining expects CAVEAT [because EVIDENCE], …, not {}",
                                item.trim()
                            ))
                        }
                    }
                }
                Ok((retained, reasons))
            };
            // Clauses, in order: using or citing, permitted by … [for …],
            // retaining. Their
            // keywords count only outside parentheses.
            let mut depth = 0_i64;
            let (mut permitted, mut retained_at) = (None, None);
            for (index, token) in rest.iter().enumerate() {
                if depth == 0 && retained_at.is_none() {
                    if *token == "permitted"
                        && rest.get(index + 1) == Some(&"by")
                        && permitted.is_none()
                    {
                        permitted = Some(index);
                    } else if *token == "retaining" {
                        retained_at = Some(index);
                    }
                }
                depth += expression_depth_delta(token);
            }
            let usage = &rest[..permitted.or(retained_at).unwrap_or(rest.len())];
            let (using, citing) = match usage {
                [] => (None, Vec::new()),
                ["using", expression @ ..] if !expression.is_empty() => (
                    Some(reactive_expr::parse_unresolved(&expression.join(" "))?),
                    Vec::new(),
                ),
                ["citing", citations @ ..] if !citations.is_empty() => {
                    let citations = citations.join(" ");
                    if citations == "nothing" {
                        return Err("commit citing expects expressions separated by commas; without grounds, leave citing out".into());
                    }
                    (None, parse_citations(&citations).map_err(|_| {
                        "commit citing expects expressions separated by commas"
                    })?)
                }
                _ => {
                    return Err(
                        "commit expects [using NUMERIC_EXPRESSION | citing EXPRESSION, …] [permitted by GRANT [for EXPRESSION]] [retaining CAVEAT, CAVEAT]"
                            .into(),
                    )
                }
            };
            let permission = match permitted {
                None => None,
                Some(start) => {
                    let clause = &rest[start + 2..retained_at.unwrap_or(rest.len())];
                    let mut depth = 0_i64;
                    let mut scope_at = None;
                    for (index, token) in clause.iter().enumerate() {
                        if depth == 0 && *token == "for" {
                            scope_at = Some(index);
                            break;
                        }
                        depth += expression_depth_delta(token);
                    }
                    let grant = clause[..scope_at.unwrap_or(clause.len())].join(" ");
                    if grant.is_empty() {
                        return Err("permitted by needs a grant: EVIDENCE or latest(STREAM)".into());
                    }
                    let grant = match grant
                        .strip_prefix("latest")
                        .map(str::trim)
                        .and_then(|rest| rest.strip_prefix('('))
                    {
                        Some(arguments) => EvidenceSelector::Latest(identifier(
                            arguments
                                .trim()
                                .strip_suffix(')')
                                .ok_or("permitted by latest requires a closing parenthesis")?
                                .trim(),
                        )?),
                        None => EvidenceSelector::Named(identifier(&grant)?),
                    };
                    let scope = match scope_at {
                        None => None,
                        Some(index) if index + 1 < clause.len() => Some(
                            reactive_expr::parse_unresolved(&clause[index + 1..].join(" "))?,
                        ),
                        Some(_) => return Err("permitted by … for needs an expression".into()),
                    };
                    Some(Box::new(PermissionClause { grant, scope }))
                }
            };
            let (retaining, reasons) = retained_at
                .map(|index| parse_retained(&rest[index + 1..]))
                .transpose()?
                .unwrap_or_default();
            Ok(Effect::Commit {
                action: identifier(action)?,
                reason,
                using,
                citing,
                permission,
                retaining,
                reasons,
            })
        }
        [verb @ ("reopen" | "hold" | "release"), action, "because", selector @ ..]
            if !selector.is_empty() =>
        {
            let form = match *verb {
                "hold" => DecisionChange::Hold,
                "release" => DecisionChange::Release,
                _ => DecisionChange::Reopen,
            };
            let because = parse_selector(&selector.join(" "))?;
            Ok(Effect::Reopen {
                action: identifier(action)?,
                because,
                form,
            })
        }
        ["resolve", rest @ ..] => parse_resolution(rest),
        _ => Err(invalid()),
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

/// First-observation ranks; see `ReactiveSession::observation_ranks`. A name
/// never observed ranks last, so a sort by `(rank, name)` puts it after the
/// observed ones, in name order.
pub(crate) enum ObservationRanks<'a> {
    Ledger(HashMap<&'a str, usize>),
    Edges {
        symbols: &'a HashMap<String, NodeId>,
        first: HashMap<NodeId, usize>,
    },
}

impl ObservationRanks<'_> {
    pub(crate) fn rank(&self, name: &str) -> usize {
        match self {
            Self::Ledger(first) => first.get(name).copied(),
            Self::Edges { symbols, first } => {
                symbols.get(name).and_then(|id| first.get(id).copied())
            }
        }
        .unwrap_or(usize::MAX)
    }
}
