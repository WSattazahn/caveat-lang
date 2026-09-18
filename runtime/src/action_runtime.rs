use crate::map::{CaveatMap, MapActionPlan};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorldCommand {
    Inspect {
        entity: String,
    },
    Operate {
        entity: String,
    },
    Open {
        entity: String,
    },
    Move {
        from: String,
        to: String,
        via: Option<String>,
    },
    Observe {
        symbol: String,
    },
    Stay {
        place: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionExecution {
    pub action: String,
    pub from: String,
    pub to: String,
    pub commands: Vec<WorldCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldStateSnapshot {
    pub current_place: String,
    pub open_entities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SimulationResult {
    pub executions: Vec<ActionExecution>,
    pub state: WorldStateSnapshot,
}

pub fn simulate(map: &CaveatMap, actions: &[String]) -> Result<SimulationResult, String> {
    let mut runtime = ActionRuntime::new(map)?;
    let mut executions = Vec::new();

    for action in actions {
        let (next, execution) = runtime.preview(map, action)?;
        runtime = next;
        executions.push(execution);
    }

    Ok(SimulationResult {
        executions,
        state: runtime.snapshot(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRuntime {
    current_place: String,
    open_entities: HashSet<String>,
}

impl ActionRuntime {
    pub fn new(map: &CaveatMap) -> Result<Self, String> {
        let current_place = map
            .world
            .start_at
            .clone()
            .ok_or_else(|| "CAVEAT world has no start_at".to_string())?;

        Ok(Self {
            current_place,
            open_entities: HashSet::new(),
        })
    }

    pub fn current_place(&self) -> &str {
        &self.current_place
    }

    pub fn is_open(&self, entity: &str) -> bool {
        self.open_entities.contains(entity)
    }

    pub fn snapshot(&self) -> WorldStateSnapshot {
        let mut open_entities = self.open_entities.iter().cloned().collect::<Vec<_>>();
        open_entities.sort();
        WorldStateSnapshot {
            current_place: self.current_place.clone(),
            open_entities,
        }
    }

    pub fn is_available(&self, map: &CaveatMap, action: &str) -> bool {
        map.world
            .action_plans
            .iter()
            .find(|plan| plan.action == action)
            .is_some_and(|plan| {
                plan.from == self.current_place
                    && plan
                        .requires_open
                        .iter()
                        .all(|entity| self.open_entities.contains(entity))
            })
    }

    pub fn available_actions(&self, map: &CaveatMap, options: &[String]) -> Vec<String> {
        options
            .iter()
            .filter(|action| self.is_available(map, action))
            .cloned()
            .collect()
    }

    pub fn preview(
        &self,
        map: &CaveatMap,
        action: &str,
    ) -> Result<(ActionRuntime, ActionExecution), String> {
        let mut next = self.clone();
        let execution = next.apply(map, action)?;
        Ok((next, execution))
    }

    fn apply(&mut self, map: &CaveatMap, action: &str) -> Result<ActionExecution, String> {
        let plan = map
            .world
            .action_plans
            .iter()
            .find(|plan| plan.action == action)
            .ok_or_else(|| format!("no world action plan for {action}"))?;

        self.check_preconditions(plan)?;

        let from = self.current_place.clone();
        let mut commands = Vec::new();

        for step in &plan.steps {
            let target = step.target.as_deref();
            match step.kind.as_str() {
                "inspect" => commands.push(WorldCommand::Inspect {
                    entity: required_target(action, "inspect", target)?.into(),
                }),
                "operate" => commands.push(WorldCommand::Operate {
                    entity: required_target(action, "operate", target)?.into(),
                }),
                "open" => {
                    let entity = required_target(action, "open", target)?;
                    self.open_entities.insert(entity.into());
                    commands.push(WorldCommand::Open {
                        entity: entity.into(),
                    });
                }
                "through" => {
                    let entity = required_target(action, "through", target)?;
                    if !self.open_entities.contains(entity) {
                        return Err(format!(
                            "action {action} cannot traverse closed barrier {entity}"
                        ));
                    }
                    let destination =
                        other_side(map, &self.current_place, entity).ok_or_else(|| {
                            format!(
                                "action {action} cannot traverse {entity} from {}",
                                self.current_place
                            )
                        })?;
                    let origin = std::mem::replace(&mut self.current_place, destination.clone());
                    commands.push(WorldCommand::Move {
                        from: origin,
                        to: destination,
                        via: Some(entity.into()),
                    });
                }
                "move" => {
                    let destination = required_target(action, "move", target)?;
                    if !direct_connection(map, &self.current_place, destination) {
                        return Err(format!(
                            "action {action} has no direct open connection from {} to {destination}",
                            self.current_place
                        ));
                    }
                    let origin =
                        std::mem::replace(&mut self.current_place, destination.to_string());
                    commands.push(WorldCommand::Move {
                        from: origin,
                        to: destination.into(),
                        via: None,
                    });
                }
                "observe" => commands.push(WorldCommand::Observe {
                    symbol: required_target(action, "observe", target)?.into(),
                }),
                "stay" => commands.push(WorldCommand::Stay {
                    place: self.current_place.clone(),
                }),
                other => {
                    return Err(format!("action {action} contains unknown step {other}"));
                }
            }
        }

        if self.current_place != plan.to {
            return Err(format!(
                "action {action} finished at {} instead of {}",
                self.current_place, plan.to
            ));
        }

        Ok(ActionExecution {
            action: action.into(),
            from,
            to: self.current_place.clone(),
            commands,
        })
    }

    fn check_preconditions(&self, plan: &MapActionPlan) -> Result<(), String> {
        if plan.from != self.current_place {
            return Err(format!(
                "action {} requires place {} but player is at {}",
                plan.action, plan.from, self.current_place
            ));
        }

        for entity in &plan.requires_open {
            if !self.open_entities.contains(entity) {
                return Err(format!(
                    "action {} requires {} to already be open",
                    plan.action, entity
                ));
            }
        }

        Ok(())
    }
}

fn required_target<'a>(
    action: &str,
    step: &str,
    target: Option<&'a str>,
) -> Result<&'a str, String> {
    target.ok_or_else(|| format!("action {action} step {step} has no target"))
}

fn other_side(map: &CaveatMap, place: &str, entity: &str) -> Option<String> {
    map.world.connections.iter().find_map(|connection| {
        if connection.via.as_deref() != Some(entity) {
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

fn direct_connection(map: &CaveatMap, from: &str, to: &str) -> bool {
    map.world.connections.iter().any(|connection| {
        connection.via.is_none()
            && ((connection.from == from && connection.to == to)
                || (connection.to == from && connection.from == to))
    })
}

#[cfg(test)]
mod tests {
    use super::ActionRuntime;
    use crate::map::CaveatMap;

    const SOURCE: &str = r#"
place a kind corridor;
place b kind corridor;
place c kind corridor;
entity door_a kind fire_door at a;
connect a to b via door_a;
connect b to c;
start_at a;
action leave from a to b steps operate door_a, open door_a, through door_a;
action continue from b to c steps move c;
action return from b to a requires_open door_a steps through door_a;
choice outbound options leave;
select outbound leave;
choice revised options continue, return;
select revised continue;
"#;

    #[test]
    fn action_runtime_preserves_world_state_between_commitments() {
        let map = CaveatMap::from_source(SOURCE).expect("map should validate");
        let runtime = ActionRuntime::new(&map).expect("world should have a start");

        assert_eq!(runtime.current_place(), "a");
        assert!(runtime.is_available(&map, "leave"));
        assert!(!runtime.is_available(&map, "return"));

        let (runtime, leave) = runtime
            .preview(&map, "leave")
            .expect("leave should execute");
        assert_eq!(leave.from, "a");
        assert_eq!(leave.to, "b");
        assert_eq!(runtime.current_place(), "b");
        assert!(runtime.is_open("door_a"));
        assert!(runtime.is_available(&map, "return"));

        let (runtime, returned) = runtime
            .preview(&map, "return")
            .expect("return should use the already-open door");
        assert_eq!(returned.to, "a");
        assert_eq!(runtime.current_place(), "a");
    }

    #[test]
    fn unavailable_action_does_not_mutate_state() {
        let map = CaveatMap::from_source(SOURCE).expect("map should validate");
        let runtime = ActionRuntime::new(&map).expect("world should have a start");

        let error = runtime
            .preview(&map, "continue")
            .expect_err("continue should require b");

        assert!(error.contains("requires place b"));
        assert_eq!(runtime.current_place(), "a");
    }
}
