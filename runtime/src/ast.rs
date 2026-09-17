use crate::{Attention, Consequence, Relation, StopReason};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program { pub statements: Vec<Statement> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Budget { units: u64 },
    Claim { name: String },
    Evidence { name: String, source: String },
    Caveat { name: String, consequence: Consequence },
    Relate { from: String, relation: Relation, to: String },
    Attention { caveat: String, state: Attention },
    Examine { caveat: String, cost: u64 },
    Commit { action: String, reason: StopReason, retaining: Vec<String> },
    Reopen { commitment: String, because: String },
}
impl Program { pub fn new(statements: Vec<Statement>) -> Self { Self { statements } } }
