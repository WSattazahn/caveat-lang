use crate::ast::{ActionStep, ConditionalAction, EpistemicCondition, Program, Statement};
use crate::eval;
use crate::{NodeKind, Relation};
use serde::Serialize;
use std::collections::{HashSet, VecDeque};

pub const MAP_SCHEMA: &str = "caveat-map/0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaveatMap {
    pub schema: String,
    pub scenes: Vec<String>,
    pub symbols: Vec<MapSymbol>,
    pub relations: Vec<MapRelation>,
    pub reveals: Vec<MapReveal>,
    pub investigations: Vec<MapInvestigation>,
    pub rules: Vec<MapRule>,
    pub choices: Vec<MapChoice>,
    pub conditionals: Vec<MapConditional>,
    pub actions: Vec<MapAction>,
    pub execution: MapExecution,
    pub world: MapWorld,
    pub requirements: Vec<MapRequirement>,
    pub resolutions: Vec<MapResolution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapRequirement {
    pub action: String,
    pub condition: EpistemicCondition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapResolution {
    pub action: String,
    pub outcome: String,
    pub condition: Option<EpistemicCondition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapSymbol {
    pub name: String,
    pub kind: String,
    /// Where the claim came from in the world: an evidence's `from`.
    pub source: Option<String>,
    /// Which part of the program declared it. `None` when the program
    /// declared no origin. See spec/caveat-authorship-0.1.md.
    pub written_by: Option<String>,
    pub consequence: Option<String>,
    pub display: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapRelation {
    pub from: String,
    pub relation: String,
    pub to: String,
    pub origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapReveal {
    pub when_inspected: String,
    pub from: String,
    pub relation: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapInvestigation {
    pub name: String,
    pub options: Vec<MapInvestigationOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapInvestigationOption {
    pub symbol: String,
    pub cost: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapRule {
    pub name: String,
    pub premises: Vec<String>,
    pub conclusion: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapChoice {
    pub name: String,
    pub options: Vec<String>,
    pub retaining: Vec<String>,
    pub selected: Option<String>,
    pub converging: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapConditional {
    pub action: String,
    pub kind: String,
    pub from: Option<String>,
    pub relation: Option<String>,
    pub to: Option<String>,
    pub commitment: Option<String>,
    pub because: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapAction {
    pub id: String,
    pub display: Option<String>,
    pub choice: String,
    pub retaining: Vec<String>,
    pub conditional_effects: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapExecution {
    pub budget: Option<MapBudget>,
    pub commitments: Vec<MapCommitment>,
    pub event_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapBudget {
    pub initial: u64,
    pub spent: u64,
    pub remaining: u64,
    pub exhausted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapCommitment {
    pub action: String,
    pub open: bool,
    pub retained: Vec<String>,
    /// The same retained caveats, with who wrote each one. Parallel to
    /// `retained`, which stays a plain list of names so existing readers keep
    /// working. See spec/caveat-borrowed-uncertainty-0.1.md.
    pub retained_authorship: Vec<RetainedCaveat>,
    pub reopened_by: Vec<String>,
}

/// A caveat a commitment carried, and where it was written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetainedCaveat {
    pub caveat: String,
    /// The part that wrote the caveat, if it was written under an origin.
    pub written_by: Option<String>,
    /// Whether the caveat was written in a different part from the commitment
    /// that retained it. `None` when either origin is unrecorded: not knowing
    /// where something was written is not the same as having written it.
    ///
    /// This is a fact about location, not a judgement. A caveat written
    /// elsewhere is not weaker, less relevant, or less binding than one
    /// written here.
    pub written_elsewhere: Option<bool>,
}

/// Pair each caveat a commitment retained with the part that wrote it.
pub fn retained_authorship(
    graph: &crate::EpistemicGraph,
    commitment: crate::NodeId,
    name_of: impl Fn(crate::NodeId) -> Option<String>,
) -> Vec<RetainedCaveat> {
    let by = graph.origin(commitment);
    graph
        .edges
        .iter()
        .filter(|edge| edge.from == commitment && edge.relation == Relation::Retains)
        .filter_map(|edge| {
            let written_by = graph.origin(edge.to);
            Some(RetainedCaveat {
                caveat: name_of(edge.to)?,
                written_by: written_by.map(str::to_string),
                written_elsewhere: by.zip(written_by).map(|(by, wrote)| by != wrote),
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct MapWorld {
    pub start_at: Option<String>,
    pub places: Vec<MapPlace>,
    pub entities: Vec<MapEntity>,
    pub connections: Vec<MapConnection>,
    pub action_plans: Vec<MapActionPlan>,
    #[serde(skip_serializing_if = "crate::presentation::Presentation::is_empty")]
    pub presentation: crate::presentation::Presentation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapPlace {
    pub id: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapEntity {
    pub id: String,
    pub kind: String,
    pub at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapConnection {
    pub from: String,
    pub to: String,
    pub via: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapActionPlan {
    pub action: String,
    pub from: String,
    pub to: String,
    pub requires_open: Vec<String>,
    pub steps: Vec<MapActionStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapActionStep {
    pub kind: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapInspection {
    pub subject: String,
    pub symbol: Option<MapSymbol>,
    pub place: Option<MapPlace>,
    pub entity: Option<MapEntity>,
    pub connections: Vec<MapConnection>,
    pub actions: Vec<MapAction>,
    pub action_plan: Option<MapActionPlan>,
    pub outgoing: Vec<MapRelation>,
    pub incoming: Vec<MapRelation>,
    pub investigations: Vec<String>,
    pub choices: Vec<String>,
    pub conditionals: Vec<String>,
    pub reveals: Vec<String>,
    pub requirements: Vec<MapRequirement>,
    pub resolutions: Vec<MapResolution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapTrace {
    pub subject: String,
    pub nodes: Vec<MapTraceNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapTraceNode {
    pub symbol: String,
    pub depth: usize,
    pub from: Option<String>,
    pub via: Option<String>,
}

impl CaveatMap {
    pub fn build(program: &Program) -> Result<Self, String> {
        let evaluation = eval::evaluate(program)?;

        let mut scenes = Vec::new();
        let mut symbols = Vec::new();
        let mut relations = Vec::new();
        let mut reveals = Vec::new();
        let mut investigations = Vec::new();
        let mut rules = Vec::new();
        let mut choices = Vec::new();
        let mut conditionals = Vec::new();
        let mut world = MapWorld::default();
        let mut requirements = Vec::new();
        let mut resolutions = Vec::new();
        let mut presentation = Vec::new();

        for statement in &program.statements {
            match statement {
                Statement::Reactive(_) | Statement::Origin { .. } => {}
                Statement::Presentation(directive) => presentation.push(directive.clone()),
                Statement::Require { action, condition } => requirements.push(MapRequirement {
                    action: action.clone(),
                    condition: condition.clone(),
                }),
                Statement::Resolve {
                    action,
                    outcome,
                    condition,
                } => resolutions.push(MapResolution {
                    action: action.clone(),
                    outcome: outcome.clone(),
                    condition: condition.clone(),
                }),
                Statement::Scene { text } => scenes.push(text.clone()),
                Statement::Display { .. } => {}
                Statement::Place { name, kind } => world.places.push(MapPlace {
                    id: name.clone(),
                    kind: kind.clone(),
                }),
                Statement::Entity { name, kind, at } => world.entities.push(MapEntity {
                    id: name.clone(),
                    kind: kind.clone(),
                    at: at.clone(),
                }),
                Statement::Connect { from, to, via } => {
                    world.connections.push(MapConnection {
                        from: from.clone(),
                        to: to.clone(),
                        via: via.clone(),
                    });
                }
                Statement::StartAt { place } => {
                    if world.start_at.replace(place.clone()).is_some() {
                        return Err("world can declare only one start_at".into());
                    }
                }
                Statement::ActionPlan {
                    action,
                    from,
                    to,
                    requires_open,
                    steps,
                } => world.action_plans.push(MapActionPlan {
                    action: action.clone(),
                    from: from.clone(),
                    to: to.clone(),
                    requires_open: requires_open.clone(),
                    steps: steps.iter().map(map_action_step).collect(),
                }),
                Statement::Claim { name } => symbols.push(MapSymbol {
                    name: name.clone(),
                    kind: "claim".into(),
                    source: None,
                    written_by: written_by(&evaluation, name),
                    consequence: None,
                    display: evaluation.display.get(name).cloned(),
                }),
                Statement::Evidence { name, source } => symbols.push(MapSymbol {
                    name: name.clone(),
                    kind: "evidence".into(),
                    source: Some(source.clone()),
                    written_by: written_by(&evaluation, name),
                    consequence: None,
                    display: evaluation.display.get(name).cloned(),
                }),
                Statement::Caveat { name, consequence } => symbols.push(MapSymbol {
                    name: name.clone(),
                    kind: "caveat".into(),
                    source: None,
                    written_by: written_by(&evaluation, name),
                    consequence: Some(format!("{consequence:?}").to_lowercase()),
                    display: evaluation.display.get(name).cloned(),
                }),
                Statement::Relate { from, relation, to } => relations.push(MapRelation {
                    from: from.clone(),
                    relation: relation_name(*relation),
                    to: to.clone(),
                    origin: "source".into(),
                }),
                Statement::Investigate { name, options } => investigations.push(MapInvestigation {
                    name: name.clone(),
                    options: options
                        .iter()
                        .map(|symbol| MapInvestigationOption {
                            symbol: symbol.clone(),
                            cost: None,
                        })
                        .collect(),
                }),
                Statement::Inspect {
                    investigation,
                    cost,
                    ..
                } => {
                    if let Some(found) = investigations
                        .iter_mut()
                        .find(|candidate| candidate.name == *investigation)
                    {
                        for option in &mut found.options {
                            option.cost = Some(*cost);
                        }
                    }
                }
                Statement::Reveal {
                    when_inspected,
                    from,
                    relation,
                    to,
                } => reveals.push(MapReveal {
                    when_inspected: when_inspected.clone(),
                    from: from.clone(),
                    relation: relation_name(*relation),
                    to: to.clone(),
                }),
                Statement::Rule {
                    name,
                    premises,
                    conclusion,
                } => rules.push(MapRule {
                    name: name.clone(),
                    premises: premises.clone(),
                    conclusion: conclusion.clone(),
                }),
                Statement::Choice {
                    name,
                    options,
                    retaining,
                } => choices.push(MapChoice {
                    name: name.clone(),
                    options: options.clone(),
                    retaining: retaining.clone(),
                    selected: None,
                    converging: false,
                }),
                Statement::Converge { choice } => {
                    let found = choices
                        .iter_mut()
                        .find(|candidate| candidate.name == *choice)
                        .ok_or_else(|| format!("converge references unknown choice {choice}"))?;
                    found.converging = true;
                }
                Statement::Select { choice, option } => {
                    if let Some(found) = choices
                        .iter_mut()
                        .find(|candidate| candidate.name == *choice)
                    {
                        found.selected = Some(option.clone());
                    }
                }
                Statement::WhenCommitted { action, then } => {
                    conditionals.push(map_conditional(action, then));
                }
                Statement::Budget { .. }
                | Statement::Attention { .. }
                | Statement::Examine { .. }
                | Statement::Infer { .. }
                | Statement::Commit { .. }
                | Statement::Reopen { .. } => {}
            }
        }

        validate_world(&world, &symbols, &choices, &investigations)?;
        world.presentation = crate::presentation::build_presentation(&presentation, &world)?;
        validate_epistemic_actions(&world, &symbols, &requirements, &resolutions)?;

        for (name, id) in &evaluation.symbols {
            if matches!(
                evaluation.graph.nodes.get(id),
                Some(NodeKind::Commitment { .. })
            ) && !symbols.iter().any(|symbol| symbol.name == *name)
            {
                symbols.push(MapSymbol {
                    name: name.clone(),
                    kind: "commitment".into(),
                    source: None,
                    written_by: written_by(&evaluation, name),
                    consequence: None,
                    display: evaluation.display.get(name).cloned(),
                });
            }
        }

        let actions = choices
            .iter()
            .flat_map(|choice| {
                choice.options.iter().map(|option| MapAction {
                    id: option.clone(),
                    display: evaluation.display.get(option).cloned(),
                    choice: choice.name.clone(),
                    retaining: choice.retaining.clone(),
                    conditional_effects: conditionals
                        .iter()
                        .filter(|conditional| conditional.action == *option)
                        .map(describe_conditional)
                        .collect(),
                })
            })
            .collect();

        let budget = evaluation.resources.as_ref().map(|ledger| MapBudget {
            initial: ledger.initial,
            spent: ledger.spent,
            remaining: ledger.remaining,
            exhausted: ledger.exhausted,
        });

        let mut commitments = Vec::new();
        for (name, id) in &evaluation.symbols {
            let Some(NodeKind::Commitment { open, .. }) = evaluation.graph.nodes.get(id) else {
                continue;
            };

            let retained = evaluation
                .graph
                .edges
                .iter()
                .filter(|edge| edge.from == *id && edge.relation == Relation::Retains)
                .filter_map(|edge| symbol_name(&evaluation, edge.to))
                .collect();

            let reopened_by = evaluation
                .graph
                .edges
                .iter()
                .filter(|edge| edge.to == *id && edge.relation == Relation::Reopens)
                .filter_map(|edge| symbol_name(&evaluation, edge.from))
                .collect();

            commitments.push(MapCommitment {
                action: name.clone(),
                open: *open,
                retained,
                retained_authorship: retained_authorship(&evaluation.graph, *id, |node| {
                    symbol_name(&evaluation, node)
                }),
                reopened_by,
            });
        }
        commitments.sort_by(|left, right| left.action.cmp(&right.action));

        symbols.sort_by(|left, right| left.name.cmp(&right.name));

        Ok(Self {
            requirements,
            resolutions,
            schema: MAP_SCHEMA.into(),
            scenes,
            symbols,
            relations,
            reveals,
            investigations,
            rules,
            choices,
            conditionals,
            actions,
            execution: MapExecution {
                budget,
                commitments,
                event_count: evaluation.history.len(),
            },
            world,
        })
    }

    pub fn from_source(source: &str) -> Result<Self, String> {
        let program = crate::parser::parse(&crate::link::link(source)?)?;
        Self::build(&program)
    }

    pub fn inspect(&self, subject: &str) -> MapInspection {
        let symbol = self
            .symbols
            .iter()
            .find(|symbol| symbol.name == subject)
            .cloned();

        let place = self
            .world
            .places
            .iter()
            .find(|place| place.id == subject)
            .cloned();

        let entity = self
            .world
            .entities
            .iter()
            .find(|entity| entity.id == subject)
            .cloned();

        let connections = self
            .world
            .connections
            .iter()
            .filter(|connection| {
                connection.from == subject
                    || connection.to == subject
                    || connection.via.as_deref() == Some(subject)
            })
            .cloned()
            .collect();

        let actions = self
            .actions
            .iter()
            .filter(|action| action.id == subject)
            .cloned()
            .collect();

        let action_plan = self
            .world
            .action_plans
            .iter()
            .find(|plan| plan.action == subject)
            .cloned();

        let outgoing = self
            .relations
            .iter()
            .filter(|relation| relation.from == subject)
            .cloned()
            .collect();

        let incoming = self
            .relations
            .iter()
            .filter(|relation| relation.to == subject)
            .cloned()
            .collect();

        let investigations = self
            .investigations
            .iter()
            .filter(|investigation| {
                investigation
                    .options
                    .iter()
                    .any(|option| option.symbol == subject)
            })
            .map(|investigation| investigation.name.clone())
            .collect();

        let choices = self
            .choices
            .iter()
            .filter(|choice| {
                choice.options.iter().any(|option| option == subject)
                    || choice.retaining.iter().any(|retained| retained == subject)
            })
            .map(|choice| choice.name.clone())
            .collect();

        let conditionals = self
            .conditionals
            .iter()
            .filter(|conditional| conditional_mentions(conditional, subject))
            .map(describe_conditional)
            .collect();

        let reveals = self
            .reveals
            .iter()
            .filter(|reveal| {
                reveal.when_inspected == subject || reveal.from == subject || reveal.to == subject
            })
            .map(|reveal| {
                format!(
                    "{} -> {} {} {}",
                    reveal.when_inspected, reveal.from, reveal.relation, reveal.to
                )
            })
            .collect();

        MapInspection {
            requirements: self
                .requirements
                .iter()
                .filter(|rule| rule.action == subject || rule.condition.symbol() == subject)
                .cloned()
                .collect(),
            resolutions: self
                .resolutions
                .iter()
                .filter(|rule| {
                    rule.action == subject
                        || rule.outcome == subject
                        || rule
                            .condition
                            .as_ref()
                            .is_some_and(|condition| condition.symbol() == subject)
                })
                .cloned()
                .collect(),
            subject: subject.into(),
            symbol,
            place,
            entity,
            connections,
            actions,
            action_plan,
            outgoing,
            incoming,
            investigations,
            choices,
            conditionals,
            reveals,
        }
    }

    pub fn trace(&self, subject: &str) -> MapTrace {
        let mut nodes = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(subject.to_string());
        queue.push_back((subject.to_string(), 0_usize, None, None));

        while let Some((symbol, depth, from, via)) = queue.pop_front() {
            nodes.push(MapTraceNode {
                symbol: symbol.clone(),
                depth,
                from,
                via,
            });

            if depth >= 8 {
                continue;
            }

            for relation in self
                .relations
                .iter()
                .filter(|relation| relation.from == symbol)
            {
                if visited.insert(relation.to.clone()) {
                    queue.push_back((
                        relation.to.clone(),
                        depth + 1,
                        Some(symbol.clone()),
                        Some(relation.relation.clone()),
                    ));
                }
            }

            for rule in &self.rules {
                if rule.premises.iter().any(|premise| premise == &symbol)
                    && visited.insert(rule.conclusion.clone())
                {
                    queue.push_back((
                        rule.conclusion.clone(),
                        depth + 1,
                        Some(symbol.clone()),
                        Some(format!("rule:{}", rule.name)),
                    ));
                }
            }

            for connection in &self.world.connections {
                let next = if connection.from == symbol {
                    Some(connection.to.as_str())
                } else if connection.to == symbol {
                    Some(connection.from.as_str())
                } else {
                    None
                };

                if let Some(next) = next {
                    if visited.insert(next.to_string()) {
                        let route = connection
                            .via
                            .as_deref()
                            .map(|via| format!("world:{via}"))
                            .unwrap_or_else(|| "world:direct".into());
                        queue.push_back((
                            next.to_string(),
                            depth + 1,
                            Some(symbol.clone()),
                            Some(route),
                        ));
                    }
                }

                if connection.via.as_deref() == Some(symbol.as_str()) {
                    for endpoint in [&connection.from, &connection.to] {
                        if visited.insert(endpoint.clone()) {
                            queue.push_back((
                                endpoint.clone(),
                                depth + 1,
                                Some(symbol.clone()),
                                Some("world:connector".into()),
                            ));
                        }
                    }
                }
            }
        }

        MapTrace {
            subject: subject.into(),
            nodes,
        }
    }

    pub fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|error| error.to_string())
    }
}

pub fn to_json_pretty<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|error| error.to_string())
}

fn map_action_step(step: &ActionStep) -> MapActionStep {
    match step {
        ActionStep::Inspect { entity } => MapActionStep {
            kind: "inspect".into(),
            target: Some(entity.clone()),
        },
        ActionStep::Operate { entity } => MapActionStep {
            kind: "operate".into(),
            target: Some(entity.clone()),
        },
        ActionStep::Open { entity } => MapActionStep {
            kind: "open".into(),
            target: Some(entity.clone()),
        },
        ActionStep::Through { entity } => MapActionStep {
            kind: "through".into(),
            target: Some(entity.clone()),
        },
        ActionStep::Move { place } => MapActionStep {
            kind: "move".into(),
            target: Some(place.clone()),
        },
        ActionStep::Observe { symbol } => MapActionStep {
            kind: "observe".into(),
            target: Some(symbol.clone()),
        },
        ActionStep::Stay => MapActionStep {
            kind: "stay".into(),
            target: None,
        },
    }
}

fn validate_epistemic_actions(
    world: &MapWorld,
    symbols: &[MapSymbol],
    requirements: &[MapRequirement],
    resolutions: &[MapResolution],
) -> Result<(), String> {
    let validate_action = |action: &str| {
        if world.action_plans.iter().any(|plan| plan.action == action) {
            Ok(())
        } else {
            Err(format!("epistemic rule references unknown action {action}"))
        }
    };
    let validate_condition = |condition: &EpistemicCondition| {
        let (name, kind) = match condition {
            EpistemicCondition::Observed { symbol } => (symbol, "evidence"),
            EpistemicCondition::Examined { symbol } => (symbol, "caveat"),
        };
        match symbols.iter().find(|symbol| symbol.name == *name) {
            Some(symbol) if symbol.kind == kind => Ok(()),
            Some(_) => Err(format!("epistemic condition requires {kind} {name}")),
            None => Err(format!(
                "epistemic condition references unknown {kind} {name}"
            )),
        }
    };
    for (index, requirement) in requirements.iter().enumerate() {
        validate_action(&requirement.action)?;
        validate_condition(&requirement.condition)?;
        if requirements[..index].contains(requirement) {
            return Err(format!(
                "duplicate requirement for action {}",
                requirement.action
            ));
        }
    }
    let mut actions = HashSet::new();
    let mut fallback_seen = HashSet::new();
    for (index, resolution) in resolutions.iter().enumerate() {
        validate_action(&resolution.action)?;
        actions.insert(resolution.action.as_str());
        if fallback_seen.contains(resolution.action.as_str()) {
            return Err(format!(
                "otherwise must be the final resolution for action {}",
                resolution.action
            ));
        }
        if let Some(condition) = &resolution.condition {
            validate_condition(condition)?;
            if resolutions[..index].iter().any(|previous| {
                previous.action == resolution.action && previous.condition == resolution.condition
            }) {
                return Err(format!(
                    "duplicate resolution condition for action {}",
                    resolution.action
                ));
            }
        } else {
            fallback_seen.insert(resolution.action.as_str());
        }
    }
    for action in actions {
        if !fallback_seen.contains(action) {
            return Err(format!(
                "action {action} requires exactly one otherwise resolution"
            ));
        }
    }
    Ok(())
}

fn validate_world(
    world: &MapWorld,
    symbols: &[MapSymbol],
    choices: &[MapChoice],
    investigations: &[MapInvestigation],
) -> Result<(), String> {
    let mut identifiers = HashSet::new();
    let places = world
        .places
        .iter()
        .map(|place| place.id.as_str())
        .collect::<HashSet<_>>();
    let entities = world
        .entities
        .iter()
        .map(|entity| entity.id.as_str())
        .collect::<HashSet<_>>();
    let symbol_names = symbols
        .iter()
        .map(|symbol| symbol.name.as_str())
        .collect::<HashSet<_>>();

    for place in &world.places {
        if !identifiers.insert(place.id.as_str()) {
            return Err(format!("duplicate world identifier: {}", place.id));
        }
    }

    for entity in &world.entities {
        if !identifiers.insert(entity.id.as_str()) {
            return Err(format!("duplicate world identifier: {}", entity.id));
        }
        if !places.contains(entity.at.as_str()) {
            return Err(format!(
                "entity {} references unknown place {}",
                entity.id, entity.at
            ));
        }
    }

    if let Some(start) = &world.start_at {
        if !places.contains(start.as_str()) {
            return Err(format!("start_at references unknown place {start}"));
        }
    }

    let mut seen_connections = HashSet::new();
    for connection in &world.connections {
        if !places.contains(connection.from.as_str()) {
            return Err(format!(
                "connection references unknown place {}",
                connection.from
            ));
        }
        if !places.contains(connection.to.as_str()) {
            return Err(format!(
                "connection references unknown place {}",
                connection.to
            ));
        }
        if connection.from == connection.to {
            return Err(format!(
                "connection cannot connect {} to itself",
                connection.from
            ));
        }

        if let Some(via) = &connection.via {
            if !entities.contains(via.as_str()) {
                return Err(format!("connection references unknown entity {via}"));
            }
            let entity = world
                .entities
                .iter()
                .find(|entity| entity.id == *via)
                .expect("validated entity membership");
            if entity.at != connection.from && entity.at != connection.to {
                return Err(format!(
                    "connector {} is not located at either endpoint {} or {}",
                    via, connection.from, connection.to
                ));
            }
        }

        let mut endpoints = [connection.from.as_str(), connection.to.as_str()];
        endpoints.sort_unstable();
        let key = (
            endpoints[0].to_string(),
            endpoints[1].to_string(),
            connection.via.clone(),
        );
        if !seen_connections.insert(key) {
            return Err(format!(
                "duplicate connection between {} and {}",
                connection.from, connection.to
            ));
        }
    }

    if world.action_plans.is_empty() {
        return Ok(());
    }

    let Some(_) = &world.start_at else {
        return Err("world action plans require start_at".into());
    };

    let mut plan_names = HashSet::new();
    for plan in &world.action_plans {
        if !plan_names.insert(plan.action.as_str()) {
            return Err(format!("duplicate action plan: {}", plan.action));
        }
        if !places.contains(plan.from.as_str()) {
            return Err(format!(
                "action {} starts at unknown place {}",
                plan.action, plan.from
            ));
        }
        if !places.contains(plan.to.as_str()) {
            return Err(format!(
                "action {} ends at unknown place {}",
                plan.action, plan.to
            ));
        }

        let mut current = plan.from.clone();
        let mut open = plan
            .requires_open
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();

        for entity in &plan.requires_open {
            validate_openable_entity(world, entity)?;
        }

        for step in &plan.steps {
            let target = step.target.as_deref();
            match step.kind.as_str() {
                "inspect" | "operate" => {
                    let entity = target.expect("mapped entity step must have target");
                    validate_entity_access(world, &current, entity)?;
                }
                "open" => {
                    let entity = target.expect("mapped open step must have target");
                    validate_entity_access(world, &current, entity)?;
                    validate_openable_entity(world, entity)?;
                    open.insert(entity);
                }
                "through" => {
                    let entity = target.expect("mapped through step must have target");
                    if !open.contains(entity) {
                        return Err(format!(
                            "action {} traverses closed barrier {} without opening it or declaring requires_open",
                            plan.action, entity
                        ));
                    }
                    current = connected_other_side(world, &current, entity).ok_or_else(|| {
                        format!(
                            "action {} cannot traverse {} from {}",
                            plan.action, entity, current
                        )
                    })?;
                }
                "move" => {
                    let place = target.expect("mapped move step must have target");
                    if !places.contains(place) {
                        return Err(format!(
                            "action {} moves to unknown place {}",
                            plan.action, place
                        ));
                    }
                    if !world.connections.iter().any(|connection| {
                        connection.via.is_none()
                            && ((connection.from == current && connection.to == place)
                                || (connection.to == current && connection.from == place))
                    }) {
                        return Err(format!(
                            "action {} has no direct open connection from {} to {}",
                            plan.action, current, place
                        ));
                    }
                    current = place.to_string();
                }
                "observe" => {
                    let symbol = target.expect("mapped observe step must have target");
                    if !symbol_names.contains(symbol) {
                        return Err(format!(
                            "action {} observes unknown symbol {}",
                            plan.action, symbol
                        ));
                    }
                }
                "stay" => {}
                other => {
                    return Err(format!(
                        "action {} contains unknown mapped step {}",
                        plan.action, other
                    ));
                }
            }
        }

        if current != plan.to {
            return Err(format!(
                "action {} declares destination {} but its steps finish at {}",
                plan.action, plan.to, current
            ));
        }
    }

    let offered =
        choices
            .iter()
            .flat_map(|choice| choice.options.iter())
            .chain(investigations.iter().flat_map(|investigation| {
                investigation.options.iter().map(|option| &option.symbol)
            }))
            .collect::<HashSet<_>>();

    for action in offered {
        if !world.action_plans.iter().any(|plan| &plan.action == action) {
            return Err(format!(
                "player-facing action {action} has no world action plan"
            ));
        }
    }

    for choice in choices {
        if choice.converging {
            continue;
        }
        let mut destinations = HashSet::new();
        for option in &choice.options {
            let Some(plan) = world
                .action_plans
                .iter()
                .find(|plan| plan.action == *option)
            else {
                continue;
            };
            if !destinations.insert(plan.to.as_str()) {
                return Err(format!(
                    "choice {} has multiple actions ending at {} without explicit convergence",
                    choice.name, plan.to
                ));
            }
        }
    }

    Ok(())
}

fn validate_entity_access(world: &MapWorld, place: &str, entity_id: &str) -> Result<(), String> {
    let entity = world
        .entities
        .iter()
        .find(|entity| entity.id == entity_id)
        .ok_or_else(|| format!("action references unknown entity {entity_id}"))?;

    let connected_here = world.connections.iter().any(|connection| {
        connection.via.as_deref() == Some(entity_id)
            && (connection.from == place || connection.to == place)
    });

    if entity.at == place || connected_here {
        Ok(())
    } else {
        Err(format!(
            "entity {} is not accessible from place {}",
            entity_id, place
        ))
    }
}

fn validate_openable_entity(world: &MapWorld, entity_id: &str) -> Result<(), String> {
    let entity = world
        .entities
        .iter()
        .find(|entity| entity.id == entity_id)
        .ok_or_else(|| format!("action references unknown entity {entity_id}"))?;

    if matches!(entity.kind.as_str(), "door" | "fire_door" | "gate")
        || entity.kind.ends_with("_door")
    {
        Ok(())
    } else {
        Err(format!(
            "entity {} of kind {} is not openable",
            entity.id, entity.kind
        ))
    }
}

fn connected_other_side(world: &MapWorld, place: &str, entity_id: &str) -> Option<String> {
    world.connections.iter().find_map(|connection| {
        if connection.via.as_deref() != Some(entity_id) {
            return None;
        }
        if connection.from == place {
            Some(connection.to.clone())
        } else if connection.to == place {
            Some(connection.from.clone())
        } else {
            None
        }
    })
}

fn map_conditional(action: &str, then: &ConditionalAction) -> MapConditional {
    match then {
        ConditionalAction::Relate { from, relation, to } => MapConditional {
            action: action.into(),
            kind: "relate".into(),
            from: Some(from.clone()),
            relation: Some(relation_name(*relation)),
            to: Some(to.clone()),
            commitment: None,
            because: None,
        },
        ConditionalAction::Reopen {
            commitment,
            because,
        } => MapConditional {
            action: action.into(),
            kind: "reopen".into(),
            from: None,
            relation: None,
            to: None,
            commitment: Some(commitment.clone()),
            because: Some(because.clone()),
        },
    }
}

fn describe_conditional(conditional: &MapConditional) -> String {
    match conditional.kind.as_str() {
        "relate" => format!(
            "{} {} {}",
            conditional.from.as_deref().unwrap_or("?"),
            conditional.relation.as_deref().unwrap_or("?"),
            conditional.to.as_deref().unwrap_or("?")
        ),
        "reopen" => format!(
            "reopen {} because {}",
            conditional.commitment.as_deref().unwrap_or("?"),
            conditional.because.as_deref().unwrap_or("?")
        ),
        _ => conditional.kind.clone(),
    }
}

fn conditional_mentions(conditional: &MapConditional, subject: &str) -> bool {
    conditional.action == subject
        || conditional.from.as_deref() == Some(subject)
        || conditional.to.as_deref() == Some(subject)
        || conditional.commitment.as_deref() == Some(subject)
        || conditional.because.as_deref() == Some(subject)
}

fn relation_name(relation: Relation) -> String {
    match relation {
        Relation::ReliesOn => "relies_on".into(),
        _ => format!("{relation:?}").to_lowercase(),
    }
}

/// Which part declared a symbol, asked of the graph rather than recovered
/// from the shape of its linked name.
fn written_by(evaluation: &eval::Evaluation, name: &str) -> Option<String> {
    let id = evaluation.symbols.get(name)?;
    evaluation.graph.origin(*id).map(str::to_string)
}

fn symbol_name(evaluation: &eval::Evaluation, id: crate::NodeId) -> Option<String> {
    evaluation
        .symbols
        .iter()
        .find_map(|(name, candidate)| (*candidate == id).then(|| name.clone()))
}

#[cfg(test)]
mod tests {
    use super::{CaveatMap, MAP_SCHEMA};

    const SOURCE: &str = r#"
place start kind corridor;
place landing kind stairwell;
entity door_a kind fire_door at start;
entity camera kind camera at start;
connect start to landing via door_a;
start_at start;
action camera_gap from start to start steps inspect camera, observe camera_frame;
action open from start to landing steps operate door_a, open door_a, through door_a;
budget 2;
claim route_clear;
claim route_safe;
evidence camera_frame from "camera";
caveat camera_gap consequence material;
camera_frame supports route_clear;
camera_gap qualifies camera_frame;
investigate predoor options camera_gap;
inspect predoor camera_gap cost 1;
rule route_rule when route_clear => route_safe;
infer route_rule;
choice door options open retaining camera_gap;
select door open;
when_committed open reopen open because camera_gap;
"#;

    #[test]
    fn builds_machine_map() {
        let map = CaveatMap::from_source(SOURCE).expect("map should build");
        assert_eq!(map.schema, MAP_SCHEMA);
        assert!(map
            .symbols
            .iter()
            .any(|symbol| symbol.name == "camera_gap" && symbol.kind == "caveat"));
        assert_eq!(map.actions[0].id, "open");
        assert_eq!(map.world.places.len(), 2);
        assert_eq!(map.world.connections.len(), 1);
    }

    #[test]
    fn inspect_exposes_relevant_context() {
        let map = CaveatMap::from_source(SOURCE).expect("map should build");
        let inspection = map.inspect("camera_gap");
        assert_eq!(inspection.investigations, vec!["predoor"]);
        assert_eq!(inspection.choices, vec!["door"]);
        assert!(inspection
            .outgoing
            .iter()
            .any(|relation| relation.relation == "qualifies"));
        assert!(inspection
            .conditionals
            .iter()
            .any(|conditional| conditional.contains("reopen open")));
    }

    #[test]
    fn inspect_and_trace_include_world_topology() {
        let map = CaveatMap::from_source(SOURCE).expect("map should build");
        let inspection = map.inspect("door_a");
        assert_eq!(
            inspection
                .entity
                .as_ref()
                .map(|entity| entity.kind.as_str()),
            Some("fire_door")
        );
        assert_eq!(inspection.connections.len(), 1);

        let trace = map.trace("start");
        assert!(trace.nodes.iter().any(|node| node.symbol == "landing"));
    }

    #[test]
    fn invalid_world_reference_is_rejected() {
        let source = "place start kind corridor; entity door_a kind fire_door at nowhere;";
        let error = CaveatMap::from_source(source).expect_err("unknown place should fail");
        assert!(error.contains("entity door_a references unknown place nowhere"));
    }

    #[test]
    fn closed_barrier_traversal_is_rejected() {
        let source = r#"
place a kind corridor;
place b kind corridor;
entity door_a kind fire_door at a;
connect a to b via door_a;
start_at a;
action leave from a to b steps through door_a;
choice route options leave;
select route leave;
"#;
        let error = CaveatMap::from_source(source).expect_err("closed traversal should fail");
        assert!(error.contains("traverses closed barrier door_a"));
    }

    #[test]
    fn requires_open_allows_cross_action_return() {
        let source = r#"
place a kind corridor;
place b kind corridor;
entity door_a kind fire_door at a;
connect a to b via door_a;
start_at a;
action leave from a to b steps open door_a, through door_a;
action return from b to a requires_open door_a steps through door_a;
choice outbound options leave;
select outbound leave;
choice inbound options return;
select inbound return;
"#;
        let map = CaveatMap::from_source(source).expect("open precondition should validate");
        assert_eq!(map.world.action_plans.len(), 2);
    }

    #[test]
    fn explicit_convergence_allows_actions_to_return_to_one_place() {
        let source = r#"
place hub kind courtyard;
place left kind garden_path;
place right kind garden_path;
connect hub to left;
connect hub to right;
start_at hub;
action search_left from hub to hub steps move left, move hub;
action search_right from hub to hub steps move right, move hub;
choice search options search_left, search_right;
converge search;
select search search_left;
"#;
        let map = CaveatMap::from_source(source).expect("explicit convergence should validate");
        let choice = map
            .choices
            .iter()
            .find(|choice| choice.name == "search")
            .expect("choice should exist");
        assert!(choice.converging);
    }

    #[test]
    fn duplicate_action_destinations_still_require_explicit_convergence() {
        let source = r#"
place hub kind courtyard;
place left kind garden_path;
place right kind garden_path;
connect hub to left;
connect hub to right;
start_at hub;
action search_left from hub to hub steps move left, move hub;
action search_right from hub to hub steps move right, move hub;
choice search options search_left, search_right;
select search search_left;
"#;
        let error = CaveatMap::from_source(source).expect_err("implicit convergence should fail");
        assert!(error.contains("without explicit convergence"));
    }

    #[test]
    fn trace_follows_relations_and_rules() {
        let map = CaveatMap::from_source(SOURCE).expect("map should build");
        let trace = map.trace("camera_gap");
        let names = trace
            .nodes
            .iter()
            .map(|node| node.symbol.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec!["camera_gap", "camera_frame", "route_clear", "route_safe"]
        );
    }
}
