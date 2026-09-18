use crate::ast::{ConditionalAction, Program, Statement};
use crate::{Attention, Edge, EpistemicGraph, NodeId, NodeKind, Relation, StopReason};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLedger {
    pub initial: u64,
    pub remaining: u64,
    pub spent: u64,
    pub exhausted: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpistemicSnapshot {
    pub nodes: HashMap<NodeId, NodeKind>,
    pub edges: Vec<Edge>,
    pub resources: Option<ResourceLedger>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferenceRule {
    pub premises: Vec<NodeId>,
    pub conclusion: NodeId,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoicePoint {
    pub options: Vec<String>,
    pub retaining: Vec<NodeId>,
    pub selected: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Investigation {
    pub options: Vec<String>,
    pub inspected: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    Scene {
        text: String,
    },
    BudgetDeclared {
        units: u64,
    },
    Examined {
        caveat: NodeId,
        cost: u64,
        remaining: u64,
    },
    InvestigationOffered {
        name: String,
        options: Vec<String>,
    },
    Investigated {
        name: String,
        caveat: NodeId,
        cost: u64,
        remaining: u64,
    },
    Revealed {
        because: NodeId,
        from: NodeId,
        relation: Relation,
        to: NodeId,
    },
    Inferred {
        rule: String,
        conclusion: NodeId,
        premises: Vec<NodeId>,
    },
    ChoiceOffered {
        name: String,
        options: Vec<String>,
    },
    ChoiceSelected {
        name: String,
        option: String,
        commitment: NodeId,
    },
    ConditionalApplied {
        because_action: String,
    },
    Committed {
        commitment: NodeId,
        retained: Vec<NodeId>,
        reason: StopReason,
        snapshot: EpistemicSnapshot,
    },
    Reopened {
        commitment: NodeId,
        because: NodeId,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub sequence: u64,
    pub kind: EventKind,
}
#[derive(Debug)]
pub struct Evaluation {
    pub graph: EpistemicGraph,
    pub symbols: HashMap<String, NodeId>,
    pub resources: Option<ResourceLedger>,
    pub history: Vec<Event>,
    pub rules: HashMap<String, InferenceRule>,
    pub choices: HashMap<String, ChoicePoint>,
    pub investigations: HashMap<String, Investigation>,
    pub display: HashMap<String, String>,
}

struct Evaluator {
    graph: EpistemicGraph,
    symbols: HashMap<String, NodeId>,
    resources: Option<ResourceLedger>,
    history: Vec<Event>,
    rules: HashMap<String, InferenceRule>,
    choices: HashMap<String, ChoicePoint>,
    investigations: HashMap<String, Investigation>,
    display: HashMap<String, String>,
    inspected: Vec<String>,
    next_sequence: u64,
}

impl Evaluator {
    fn new() -> Self {
        Self {
            graph: EpistemicGraph::new(),
            symbols: HashMap::new(),
            resources: None,
            history: Vec::new(),
            rules: HashMap::new(),
            choices: HashMap::new(),
            investigations: HashMap::new(),
            display: HashMap::new(),
            inspected: Vec::new(),
            next_sequence: 1,
        }
    }
    fn event(&mut self, kind: EventKind) {
        self.history.push(Event {
            sequence: self.next_sequence,
            kind,
        });
        self.next_sequence += 1;
    }
    fn resolve(&self, name: &str) -> Result<NodeId, String> {
        self.symbols
            .get(name)
            .copied()
            .ok_or_else(|| format!("unknown symbol: {name}"))
    }
    fn define(&mut self, name: &str, id: NodeId) -> Result<(), String> {
        if self.symbols.insert(name.into(), id).is_some() {
            Err(format!("duplicate symbol: {name}"))
        } else {
            Ok(())
        }
    }
    fn snapshot(&self) -> EpistemicSnapshot {
        EpistemicSnapshot {
            nodes: self.graph.nodes.clone(),
            edges: self.graph.edges.clone(),
            resources: self.resources.clone(),
        }
    }
    fn charge(&mut self, caveat: &str, cost: u64, context: &str) -> Result<(NodeId, u64), String> {
        let id = self.resolve(caveat)?;
        let ledger = self
            .resources
            .as_mut()
            .ok_or_else(|| format!("{context} requires budget"))?;
        if cost > ledger.remaining {
            return Err(format!("insufficient {context} budget"));
        }
        ledger.remaining -= cost;
        ledger.spent += cost;
        ledger.exhausted = ledger.remaining == 0;
        self.graph.set_attention(id, Attention::Examined);
        Ok((id, ledger.remaining))
    }
    fn apply(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Scene { text } => self.event(EventKind::Scene { text: text.clone() }),
            Statement::Display { symbol, text } => {
                self.display.insert(symbol.clone(), text.clone());
            }
            Statement::Place { .. }
            | Statement::Entity { .. }
            | Statement::Connect { .. }
            | Statement::StartAt { .. }
            | Statement::ActionPlan { .. } => {}
            Statement::Budget { units } => {
                self.resources = Some(ResourceLedger {
                    initial: *units,
                    remaining: *units,
                    spent: 0,
                    exhausted: *units == 0,
                });
                self.event(EventKind::BudgetDeclared { units: *units });
            }
            Statement::Claim { name } => {
                let id = self.graph.add(NodeKind::Claim {
                    proposition: name.clone(),
                });
                self.define(name, id)?;
            }
            Statement::Evidence { name, source } => {
                let id = self.graph.add(NodeKind::Evidence {
                    description: name.clone(),
                    source: source.clone(),
                });
                self.define(name, id)?;
            }
            Statement::Caveat { name, consequence } => {
                let id = self.graph.add_caveat(name, *consequence);
                self.define(name, id)?;
            }
            Statement::Relate { from, relation, to } => {
                let from = self.resolve(from)?;
                let to = self.resolve(to)?;
                self.graph.relate(from, *relation, to);
            }
            Statement::Attention { caveat, state } => {
                let id = self.resolve(caveat)?;
                self.graph.set_attention(id, *state);
            }
            Statement::Examine { caveat, cost } => {
                let (id, remaining) = self.charge(caveat, *cost, "examine")?;
                self.event(EventKind::Examined {
                    caveat: id,
                    cost: *cost,
                    remaining,
                });
            }
            Statement::Investigate { name, options } => {
                for option in options {
                    self.resolve(option)?;
                }
                self.investigations.insert(
                    name.clone(),
                    Investigation {
                        options: options.clone(),
                        inspected: Vec::new(),
                    },
                );
                self.event(EventKind::InvestigationOffered {
                    name: name.clone(),
                    options: options.clone(),
                });
            }
            Statement::Inspect {
                investigation,
                caveat,
                cost,
            } => {
                let valid = self
                    .investigations
                    .get(investigation)
                    .ok_or_else(|| format!("unknown investigation: {investigation}"))?
                    .options
                    .contains(caveat);
                if !valid {
                    return Err(format!("invalid investigation option: {caveat}"));
                }
                let (id, remaining) = self.charge(caveat, *cost, "investigation")?;
                self.investigations
                    .get_mut(investigation)
                    .expect("validated investigation")
                    .inspected
                    .push(caveat.clone());
                self.inspected.push(caveat.clone());
                self.event(EventKind::Investigated {
                    name: investigation.clone(),
                    caveat: id,
                    cost: *cost,
                    remaining,
                });
            }
            Statement::Reveal {
                when_inspected,
                from,
                relation,
                to,
            } if self.inspected.contains(when_inspected) => {
                let because = self.resolve(when_inspected)?;
                let from = self.resolve(from)?;
                let to = self.resolve(to)?;
                self.graph.relate(from, *relation, to);
                self.event(EventKind::Revealed {
                    because,
                    from,
                    relation: *relation,
                    to,
                });
            }
            Statement::Reveal { .. } => {}
            Statement::Rule {
                name,
                premises,
                conclusion,
            } => {
                let premises = premises
                    .iter()
                    .map(|name| self.resolve(name))
                    .collect::<Result<Vec<_>, _>>()?;
                let conclusion = self.resolve(conclusion)?;
                self.rules.insert(
                    name.clone(),
                    InferenceRule {
                        premises,
                        conclusion,
                    },
                );
            }
            Statement::Infer { rule } => {
                let rule_value = self
                    .rules
                    .get(rule)
                    .ok_or_else(|| format!("unknown rule: {rule}"))?
                    .clone();
                for premise in &rule_value.premises {
                    self.graph
                        .relate(*premise, Relation::Supports, rule_value.conclusion);
                }
                self.event(EventKind::Inferred {
                    rule: rule.clone(),
                    conclusion: rule_value.conclusion,
                    premises: rule_value.premises,
                });
            }
            Statement::Choice {
                name,
                options,
                retaining,
            } => {
                let retaining = retaining
                    .iter()
                    .map(|name| self.resolve(name))
                    .collect::<Result<Vec<_>, _>>()?;
                self.choices.insert(
                    name.clone(),
                    ChoicePoint {
                        options: options.clone(),
                        retaining,
                        selected: None,
                    },
                );
                self.event(EventKind::ChoiceOffered {
                    name: name.clone(),
                    options: options.clone(),
                });
            }
            Statement::Select { choice, option } => {
                let point = self
                    .choices
                    .get(choice)
                    .ok_or_else(|| format!("unknown choice: {choice}"))?
                    .clone();
                if !point.options.contains(option) {
                    return Err(format!("invalid option {option}"));
                }
                let id = self
                    .graph
                    .commit_because(option, &point.retaining, StopReason::Enough);
                self.define(option, id)?;
                self.choices
                    .get_mut(choice)
                    .expect("validated choice")
                    .selected = Some(option.clone());
                self.event(EventKind::ChoiceSelected {
                    name: choice.clone(),
                    option: option.clone(),
                    commitment: id,
                });
                let snapshot = self.snapshot();
                self.event(EventKind::Committed {
                    commitment: id,
                    retained: point.retaining,
                    reason: StopReason::Enough,
                    snapshot,
                });
            }
            Statement::WhenCommitted { action, then }
                if self.symbols.get(action).is_some_and(|id| {
                    matches!(self.graph.nodes.get(id), Some(NodeKind::Commitment { .. }))
                }) =>
            {
                match then {
                    ConditionalAction::Reopen {
                        commitment,
                        because,
                    } => {
                        let commitment = self.resolve(commitment)?;
                        let because = self.resolve(because)?;
                        self.graph.reopen(commitment, because);
                        self.event(EventKind::Reopened {
                            commitment,
                            because,
                        });
                    }
                    ConditionalAction::Relate { from, relation, to } => {
                        let from = self.resolve(from)?;
                        let to = self.resolve(to)?;
                        self.graph.relate(from, *relation, to);
                    }
                }
                self.event(EventKind::ConditionalApplied {
                    because_action: action.clone(),
                });
            }
            Statement::WhenCommitted { .. } => {}
            Statement::Commit {
                action,
                reason,
                retaining,
            } => {
                let retained = retaining
                    .iter()
                    .map(|name| self.resolve(name))
                    .collect::<Result<Vec<_>, _>>()?;
                let id = self.graph.commit_because(action, &retained, reason.clone());
                self.define(action, id)?;
                let snapshot = self.snapshot();
                self.event(EventKind::Committed {
                    commitment: id,
                    retained,
                    reason: reason.clone(),
                    snapshot,
                });
            }
            Statement::Reopen {
                commitment,
                because,
            } => {
                let commitment = self.resolve(commitment)?;
                let because = self.resolve(because)?;
                self.graph.reopen(commitment, because);
                self.event(EventKind::Reopened {
                    commitment,
                    because,
                });
            }
        }
        Ok(())
    }
    fn finish(self) -> Evaluation {
        Evaluation {
            graph: self.graph,
            symbols: self.symbols,
            resources: self.resources,
            history: self.history,
            rules: self.rules,
            choices: self.choices,
            investigations: self.investigations,
            display: self.display,
        }
    }
}

pub fn evaluate(program: &Program) -> Result<Evaluation, String> {
    let mut evaluator = Evaluator::new();
    for statement in &program.statements {
        evaluator.apply(statement)?;
    }
    Ok(evaluator.finish())
}
