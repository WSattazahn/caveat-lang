//! A renderer-independent, transactional CAVEAT game session.
//!
//! Source owns the world, interactions, evidence and consequences. This module
//! couples the existing epistemic evaluator to the existing action interpreter
//! and publishes only the state reached by actual player selections.

use crate::action_runtime::{ActionExecution, ActionRuntime, WorldStateSnapshot};
use crate::ast::{EpistemicCondition, Program, Statement};
use crate::eval::Evaluation;
use crate::map::{CaveatMap, MapBudget, MapCommitment, MapRelation, MapWorld};
use crate::session::{Discovery, PendingInteraction, Session};
use crate::{Attention, NodeId, NodeKind, Relation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const GAME_SCHEMA: &str = "caveat-game/0.1";
pub const SAVE_SCHEMA: &str = "caveat-save/0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GamePending {
    Investigate {
        name: String,
        options: Vec<String>,
        cost: u64,
        budget: u64,
    },
    Choice {
        name: String,
        options: Vec<String>,
        budget: u64,
    },
    Blocked {
        name: String,
        options: Vec<String>,
        reason: String,
        budget: u64,
    },
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameSymbol {
    pub name: String,
    pub kind: String,
    /// Where the claim came from in the world: an evidence's `from`.
    pub source: Option<String>,
    /// Where the assertion came from in the program: the part that declared
    /// it. A different question from `source`, and both are recorded.
    pub origin: Option<String>,
    pub consequence: Option<String>,
    pub display: Option<String>,
    pub attention: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameSnapshot {
    pub schema: String,
    pub source_id: String,
    pub turn: usize,
    pub scenes: Vec<String>,
    pub labels: BTreeMap<String, String>,
    pub world: MapWorld,
    pub state: WorldStateSnapshot,
    pub pending: GamePending,
    pub budget: Option<MapBudget>,
    pub symbols: Vec<GameSymbol>,
    pub relations: Vec<MapRelation>,
    pub commitments: Vec<MapCommitment>,
    pub discoveries: Vec<Discovery>,
    pub selections: Vec<String>,
    pub last_execution: Option<ActionExecution>,
    pub blocked_actions: Vec<BlockedAction>,
    pub outcome: Option<GameOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BlockedAction {
    pub action: String,
    pub reasons: Vec<EpistemicCondition>,
}

/// The source rule selected at the moment an action commits. Later knowledge
/// cannot rewrite its basis; replay recomputes the same result from source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameOutcome {
    pub action: String,
    pub id: String,
    pub basis: Vec<EpistemicCondition>,
    pub turn: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedGame {
    schema: String,
    source_id: String,
    // Exact equality protects identity even if the display fingerprint collides.
    source: String,
    selections: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GameSession {
    source: String,
    source_id: String,
    session: Session,
    map: CaveatMap,
    actions: ActionRuntime,
    labels: BTreeMap<String, String>,
    selections: Vec<String>,
    discoveries: Vec<Discovery>,
    last_execution: Option<ActionExecution>,
    outcome: Option<GameOutcome>,
}

impl GameSession {
    pub fn from_source(source: &str) -> Result<Self, String> {
        let session = Session::from_source(source)?;
        // Building a normal CaveatMap evaluates the entire script. For a live
        // game, compile only declarations: future selects, inspections, reveals
        // and conditionals must never run during initialization.
        let declarations = Program::new(
            session
                .program()
                .statements
                .iter()
                .filter(|statement| is_declaration(statement))
                .cloned()
                .collect(),
        );
        let map = CaveatMap::build(&declarations)?;
        // CaveatMap also supports purely epistemic programs, so its validator
        // intentionally allows an entirely absent action layer. A game cannot.
        for option in map
            .choices
            .iter()
            .flat_map(|choice| choice.options.iter())
            .chain(map.investigations.iter().flat_map(|investigation| {
                investigation.options.iter().map(|option| &option.symbol)
            }))
        {
            if !map
                .world
                .action_plans
                .iter()
                .any(|plan| &plan.action == option)
            {
                return Err(format!(
                    "player-facing action {option} has no world action plan"
                ));
            }
        }
        let actions = ActionRuntime::new(&map)?;
        let labels = session
            .program()
            .statements
            .iter()
            .filter_map(|statement| match statement {
                Statement::Display { symbol, text } => Some((symbol.clone(), text.clone())),
                _ => None,
            })
            .collect();
        let result = Self {
            source: source.into(),
            source_id: source_identity(source),
            session,
            map,
            actions,
            labels,
            selections: Vec::new(),
            discoveries: Vec::new(),
            last_execution: None,
            outcome: None,
        };
        result.snapshot()?;
        Ok(result)
    }

    pub fn pending(&self) -> Result<GamePending, String> {
        let (name, options, cost, budget) = match self.session.pending()? {
            PendingInteraction::Investigate {
                name,
                options,
                cost,
                budget,
            } => (name, options, Some(cost), budget),
            PendingInteraction::Choice {
                name,
                options,
                budget,
            } => (name, options, None, budget),
            PendingInteraction::Complete => return Ok(GamePending::Complete),
        };
        if cost.is_some_and(|value| value > budget) {
            return Ok(GamePending::Blocked {
                name,
                options: Vec::new(),
                reason: "insufficient investigation budget".into(),
                budget,
            });
        }
        let evaluation = self.session.current_evaluation()?;
        let options = self
            .actions
            .available_actions(&self.map, &options)
            .into_iter()
            .filter(|action| self.unmet_requirements(action, &evaluation).is_empty())
            .collect::<Vec<_>>();
        if options.is_empty() {
            return Ok(GamePending::Blocked {
                name,
                options,
                reason: "no action is available from the current world or knowledge state".into(),
                budget,
            });
        }
        Ok(match cost {
            Some(cost) => GamePending::Investigate {
                name,
                options,
                cost,
                budget,
            },
            None => GamePending::Choice {
                name,
                options,
                budget,
            },
        })
    }

    /// Commit physical and epistemic effects together, or preserve the session
    /// byte-for-byte if any action, evaluation or resulting snapshot fails.
    pub fn apply(&mut self, selection: &str) -> Result<GameSnapshot, String> {
        match self.pending()? {
            GamePending::Investigate { options, .. } | GamePending::Choice { options, .. } => {
                if !options.iter().any(|option| option == selection) {
                    return Err(format!("selection is not currently available: {selection}"));
                }
            }
            GamePending::Blocked { reason, .. } => return Err(reason),
            GamePending::Complete => return Err("session is complete".into()),
        }
        let before = self.session.current_evaluation()?;
        let (actions, execution) = self.actions.preview(&self.map, selection)?;
        let mut next = self.clone();
        next.session.apply(selection)?;
        next.actions = actions;
        next.last_execution = Some(execution);
        next.selections.push(selection.into());
        if let Some(rule) = self.map.resolutions.iter().find(|rule| {
            rule.action == selection
                && rule
                    .condition
                    .as_ref()
                    .is_none_or(|condition| condition_holds(condition, &before))
        }) {
            next.outcome = Some(GameOutcome {
                action: selection.into(),
                id: rule.outcome.clone(),
                basis: rule.condition.iter().cloned().collect(),
                turn: next.selections.len(),
            });
        }

        for discovery in next.session.discoveries() {
            if !next.discoveries.contains(discovery) {
                next.discoveries.push(discovery.clone());
            }
        }
        let after = next.session.current_evaluation()?;
        // Conditional relations are discoveries too; the older evaluator emits
        // Revealed events only for explicit `reveal` statements.
        for edge in &after.graph.edges {
            if before.graph.edges.contains(edge)
                || !matches!(edge.relation, Relation::Supports | Relation::Opposes)
                || !matches!(
                    after.graph.nodes.get(&edge.from),
                    Some(NodeKind::Evidence { .. })
                )
            {
                continue;
            }
            let evidence = symbol_name(&after, edge.from);
            let relation = relation_name(edge.relation);
            let target = symbol_name(&after, edge.to);
            if !next.discoveries.iter().any(|existing| {
                existing.evidence == evidence
                    && existing.relation == relation
                    && existing.target == target
            }) {
                next.discoveries.push(Discovery {
                    because: selection.into(),
                    evidence,
                    relation,
                    target,
                });
            }
        }
        let snapshot = next.snapshot()?;
        *self = next;
        Ok(snapshot)
    }

    pub fn snapshot(&self) -> Result<GameSnapshot, String> {
        let evaluation = self.session.current_evaluation()?;
        let mut names = evaluation.symbols.iter().collect::<Vec<_>>();
        names.sort_by_key(|(name, _)| *name);
        let mut symbols = Vec::new();
        let mut commitments = Vec::new();
        for (name, id) in names {
            let node = &evaluation.graph.nodes[id];
            let (kind, source, consequence, attention) = match node {
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
                            &evaluation.graph,
                            *id,
                            |node| Some(symbol_name(&evaluation, node)),
                        ),
                        retained: evaluation
                            .graph
                            .edges
                            .iter()
                            .filter(|edge| edge.from == *id && edge.relation == Relation::Retains)
                            .map(|edge| symbol_name(&evaluation, edge.to))
                            .collect(),
                        reopened_by: evaluation
                            .graph
                            .edges
                            .iter()
                            .filter(|edge| edge.to == *id && edge.relation == Relation::Reopens)
                            .map(|edge| symbol_name(&evaluation, edge.from))
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
                origin: evaluation.graph.origin(*id).map(str::to_string),
                consequence,
                display: self.labels.get(name).cloned(),
                attention,
            });
        }
        let scenes = evaluation
            .history
            .iter()
            .filter_map(|event| match &event.kind {
                crate::eval::EventKind::Scene { text } => Some(text.clone()),
                _ => None,
            })
            .collect();
        let relations = evaluation
            .graph
            .edges
            .iter()
            .map(|edge| MapRelation {
                from: symbol_name(&evaluation, edge.from),
                relation: relation_name(edge.relation),
                to: symbol_name(&evaluation, edge.to),
                origin: "live".into(),
            })
            .collect();
        Ok(GameSnapshot {
            schema: GAME_SCHEMA.into(),
            source_id: self.source_id.clone(),
            turn: self.selections.len(),
            scenes,
            labels: self.labels.clone(),
            world: self.map.world.clone(),
            state: self.actions.snapshot(),
            pending: self.pending()?,
            budget: evaluation.resources.as_ref().map(|ledger| MapBudget {
                initial: ledger.initial,
                spent: ledger.spent,
                remaining: ledger.remaining,
                exhausted: ledger.exhausted,
            }),
            symbols,
            relations,
            commitments,
            discoveries: self.discoveries.clone(),
            selections: self.selections.clone(),
            last_execution: self.last_execution.clone(),
            blocked_actions: self.blocked_actions(&evaluation)?,
            outcome: self.outcome.clone(),
        })
    }

    fn unmet_requirements(&self, action: &str, evaluation: &Evaluation) -> Vec<EpistemicCondition> {
        self.map
            .requirements
            .iter()
            .filter(|rule| rule.action == action && !condition_holds(&rule.condition, evaluation))
            .map(|rule| rule.condition.clone())
            .collect()
    }

    fn blocked_actions(&self, evaluation: &Evaluation) -> Result<Vec<BlockedAction>, String> {
        let options = match self.session.pending()? {
            PendingInteraction::Investigate { options, .. }
            | PendingInteraction::Choice { options, .. } => options,
            PendingInteraction::Complete => return Ok(Vec::new()),
        };
        // Describe epistemic blockers only for the current interaction; future
        // scripted selections and future outcome rules never become live facts.
        Ok(options
            .into_iter()
            .filter_map(|action| {
                let reasons = self.unmet_requirements(&action, evaluation);
                (!reasons.is_empty()).then_some(BlockedAction { action, reasons })
            })
            .collect())
    }

    pub fn save(&self) -> String {
        serde_json::to_string(&SavedGame {
            schema: SAVE_SCHEMA.into(),
            source_id: self.source_id.clone(),
            source: self.source.clone(),
            selections: self.selections.clone(),
        })
        .expect("a string-only save is JSON serializable")
    }

    pub fn restore(source: &str, save: &str) -> Result<Self, String> {
        let saved: SavedGame =
            serde_json::from_str(save).map_err(|error| format!("invalid game save: {error}"))?;
        if saved.schema != SAVE_SCHEMA {
            return Err(format!("unsupported game save schema: {}", saved.schema));
        }
        if saved.source_id != source_identity(source) || saved.source != source {
            return Err("game save belongs to a different CAVEAT source".into());
        }
        let mut session = Self::from_source(source)?;
        for (index, selection) in saved.selections.iter().enumerate() {
            session
                .apply(selection)
                .map_err(|error| format!("cannot replay turn {}: {error}", index + 1))?;
        }
        Ok(session)
    }
}

fn is_declaration(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::Scene { .. }
            | Statement::Presentation(_)
            | Statement::Display { .. }
            | Statement::Place { .. }
            | Statement::Entity { .. }
            | Statement::Connect { .. }
            | Statement::StartAt { .. }
            | Statement::ActionPlan { .. }
            | Statement::Require { .. }
            | Statement::Resolve { .. }
            | Statement::Claim { .. }
            | Statement::Evidence { .. }
            | Statement::Caveat { .. }
            | Statement::Investigate { .. }
            | Statement::Choice { .. }
            | Statement::Converge { .. }
    )
}

fn condition_holds(condition: &EpistemicCondition, evaluation: &Evaluation) -> bool {
    let Some(id) = evaluation.symbols.get(condition.symbol()) else {
        return false;
    };
    match condition {
        EpistemicCondition::Observed { .. } => {
            matches!(
                evaluation.graph.nodes.get(id),
                Some(NodeKind::Evidence { .. })
            ) && evaluation.graph.edges.iter().any(|edge| {
                edge.from == *id && matches!(edge.relation, Relation::Supports | Relation::Opposes)
            })
        }
        EpistemicCondition::Examined { .. } => matches!(
            evaluation.graph.nodes.get(id),
            Some(NodeKind::Caveat {
                attention: Attention::Examined,
                ..
            })
        ),
    }
}

fn symbol_name(evaluation: &Evaluation, id: NodeId) -> String {
    evaluation
        .symbols
        .iter()
        .find_map(|(name, value)| (*value == id).then(|| name.clone()))
        .unwrap_or_else(|| id.to_string())
}

fn relation_name(relation: Relation) -> String {
    match relation {
        Relation::Supports => "supports",
        Relation::Opposes => "opposes",
        Relation::Qualifies => "qualifies",
        Relation::InContext => "in_context",
        Relation::Retains => "retains",
        Relation::Reopens => "reopens",
        Relation::ReliesOn => "relies_on",
    }
    .into()
}

// A stable display identifier, not an authentication mechanism. Saves also
// carry the exact source bytes, which restore always checks for equality.
fn source_identity(source: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in source.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}:{}", source.len())
}
