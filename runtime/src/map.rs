use crate::ast::{ActionStep, ConditionalAction, Program, Statement};
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapSymbol {
    pub name: String,
    pub kind: String,
    pub source: Option<String>,
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
    pub reopened_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct MapWorld {
    pub start_at: Option<String>,
    pub places: Vec<MapPlace>,
    pub entities: Vec<MapEntity>,
    pub connections: Vec<MapConnection>,
    pub action_plans: Vec<MapActionPlan>,
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

        for statement in &program.statements {
            match statement {
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
                    consequence: None,
                    display: evaluation.display.get(name).cloned(),
                }),
                Statement::Evidence { name, source } => symbols.push(MapSymbol {
                    name: name.clone(),
                    kind: "evidence".into(),
                    source: Some(source.clone()),
                    consequence: None,
                    display: evaluation.display.get(name).cloned(),
                }),
                Statement::Caveat { name, consequence } => symbols.push(MapSymbol {
                    name: name.clone(),
                    kind: "caveat".into(),
                    source: None,
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
                }),
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

        validate_world(&world)?;

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
                reopened_by,
            });
        }
        commitments.sort_by(|left, right| left.action.cmp(&right.action));

        symbols.sort_by(|left, right| left.name.cmp(&right.name));

        Ok(Self {
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
        let program = crate::parser::parse(source)?;
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
            subject: subject.into(),
            symbol,
            place,
            entity,
            connections,
            actions,
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

fn validate_world(world: &MapWorld) -> Result<(), String> {
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

    Ok(())
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
    format!("{relation:?}").to_lowercase()
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
connect start to landing via door_a;
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
