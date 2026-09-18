use crate::{Attention, Consequence, Relation, StopReason};
use serde::Serialize;

/// A condition about knowledge actually reached by the running program.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EpistemicCondition {
    Observed { symbol: String },
    Examined { symbol: String },
}

impl EpistemicCondition {
    pub fn symbol(&self) -> &str {
        match self {
            Self::Observed { symbol } | Self::Examined { symbol } => symbol,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionStep {
    Inspect { entity: String },
    Operate { entity: String },
    Open { entity: String },
    Through { entity: String },
    Move { place: String },
    Observe { symbol: String },
    Stay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionalAction {
    Reopen {
        commitment: String,
        because: String,
    },
    Relate {
        from: String,
        relation: Relation,
        to: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Reactive(crate::reactive::Directive),
    Presentation(crate::presentation::Directive),
    Require {
        action: String,
        condition: EpistemicCondition,
    },
    Resolve {
        action: String,
        outcome: String,
        condition: Option<EpistemicCondition>,
    },
    Scene {
        text: String,
    },
    Display {
        symbol: String,
        text: String,
    },
    Place {
        name: String,
        kind: String,
    },
    Entity {
        name: String,
        kind: String,
        at: String,
    },
    Connect {
        from: String,
        to: String,
        via: Option<String>,
    },
    StartAt {
        place: String,
    },
    ActionPlan {
        action: String,
        from: String,
        to: String,
        requires_open: Vec<String>,
        steps: Vec<ActionStep>,
    },
    Budget {
        units: u64,
    },
    Claim {
        name: String,
    },
    Evidence {
        name: String,
        source: String,
    },
    Caveat {
        name: String,
        consequence: Consequence,
    },
    Relate {
        from: String,
        relation: Relation,
        to: String,
    },
    Attention {
        caveat: String,
        state: Attention,
    },
    Examine {
        caveat: String,
        cost: u64,
    },
    Investigate {
        name: String,
        options: Vec<String>,
    },
    Inspect {
        investigation: String,
        caveat: String,
        cost: u64,
    },
    Reveal {
        when_inspected: String,
        from: String,
        relation: Relation,
        to: String,
    },
    Rule {
        name: String,
        premises: Vec<String>,
        conclusion: String,
    },
    Infer {
        rule: String,
    },
    Choice {
        name: String,
        options: Vec<String>,
        retaining: Vec<String>,
    },
    Converge {
        choice: String,
    },
    Select {
        choice: String,
        option: String,
    },
    WhenCommitted {
        action: String,
        then: ConditionalAction,
    },
    Commit {
        action: String,
        reason: StopReason,
        retaining: Vec<String>,
    },
    Reopen {
        commitment: String,
        because: String,
    },
}

impl Program {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}
