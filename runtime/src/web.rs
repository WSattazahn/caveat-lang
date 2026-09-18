use crate::action_runtime::ActionRuntime;
use crate::graphics::CaveatScene;
use crate::map::{to_json_pretty, CaveatMap};
use crate::session::{CommitmentFeedback, Discovery, PendingInteraction, Session};
use crate::world3d::World3D;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
fn json_string(value: &str) -> String {
    format!("\"{}\"", escape_json(value))
}
fn string_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn error_json(message: &str) -> String {
    format!(
        "{{\"kind\":\"error\",\"message\":{}}}",
        json_string(message)
    )
}

fn pending_json(pending: PendingInteraction) -> String {
    match pending {
        PendingInteraction::Investigate {
            name,
            options,
            cost,
            budget,
        } => format!(
            "{{\"kind\":\"investigate\",\"name\":{},\"options\":{},\"cost\":{},\"budget\":{}}}",
            json_string(&name),
            string_array(&options),
            cost,
            budget
        ),
        PendingInteraction::Choice {
            name,
            options,
            budget,
        } => format!(
            "{{\"kind\":\"choice\",\"name\":{},\"options\":{},\"budget\":{}}}",
            json_string(&name),
            string_array(&options),
            budget
        ),
        PendingInteraction::Complete => "{\"kind\":\"complete\"}".into(),
    }
}
fn session_pending(session: &Session) -> String {
    session
        .pending()
        .map(pending_json)
        .unwrap_or_else(|error| error_json(&error))
}
fn discoveries_json(discoveries: &[Discovery]) -> String {
    format!(
        "[{}]",
        discoveries
            .iter()
            .map(|discovery| format!(
                "{{\"because\":{},\"evidence\":{},\"relation\":{},\"target\":{}}}",
                json_string(&discovery.because),
                json_string(&discovery.evidence),
                json_string(&discovery.relation),
                json_string(&discovery.target)
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn commitment_json(commitment: Option<&CommitmentFeedback>) -> String {
    commitment
        .map(|commitment| {
            format!(
                "{{\"action\":{},\"retained\":{},\"reopened_by\":{}}}",
                json_string(&commitment.action),
                string_array(&commitment.retained),
                string_array(&commitment.reopened_by)
            )
        })
        .unwrap_or_else(|| "null".into())
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn caveat_map(source: &str) -> String {
    CaveatMap::from_source(source)
        .and_then(|map| map.to_json_pretty())
        .unwrap_or_else(|error| error_json(&error))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn caveat_inspect(source: &str, symbol: &str) -> String {
    CaveatMap::from_source(source)
        .and_then(|map| to_json_pretty(&map.inspect(symbol)))
        .unwrap_or_else(|error| error_json(&error))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn caveat_trace(source: &str, symbol: &str) -> String {
    CaveatMap::from_source(source)
        .and_then(|map| to_json_pretty(&map.trace(symbol)))
        .unwrap_or_else(|error| error_json(&error))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn caveat_actions(source: &str) -> String {
    CaveatMap::from_source(source)
        .and_then(|map| to_json_pretty(&map.actions))
        .unwrap_or_else(|error| error_json(&error))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn evaluate_summary(source: &str) -> String {
    match crate::parser::parse(source).and_then(|program| crate::eval::evaluate(&program)) {
        Ok(evaluation) => format!(
            "CAVEAT|nodes={}|edges={}|events={}",
            evaluation.graph.nodes.len(),
            evaluation.graph.edges.len(),
            evaluation.history.len()
        ),
        Err(error) => format!("CAVEAT_ERROR|{error}"),
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WebSession {
    inner: Session,
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WebSession {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(source: &str) -> Result<WebSession, String> {
        Ok(Self {
            inner: Session::from_source(source)?,
        })
    }
    pub fn pending(&self) -> String {
        session_pending(&self.inner)
    }
    pub fn apply(&mut self, selection: &str) -> Result<(), String> {
        self.inner.apply(selection)
    }
    pub fn discoveries(&self) -> String {
        discoveries_json(self.inner.discoveries())
    }
    pub fn commitment(&self) -> String {
        commitment_json(self.inner.commitment())
    }
    pub fn summary(&self) -> String {
        match crate::eval::evaluate(self.inner.program()) {
            Ok(evaluation) => {
                let budget = evaluation
                    .resources
                    .as_ref()
                    .map(|ledger| format!("{}/{}", ledger.remaining, ledger.initial))
                    .unwrap_or_else(|| "none".into());
                format!(
                    "CAVEAT|nodes={}|edges={}|events={}|budget={}",
                    evaluation.graph.nodes.len(),
                    evaluation.graph.edges.len(),
                    evaluation.history.len(),
                    budget
                )
            }
            Err(error) => format!("CAVEAT_ERROR|{error}"),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WebGraphicsSession {
    inner: Session,
    scene: CaveatScene,
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WebGraphicsSession {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(source: &str) -> Result<WebGraphicsSession, String> {
        Ok(Self {
            inner: Session::from_source(source)?,
            scene: CaveatScene::door(),
        })
    }
    pub fn pending(&self) -> String {
        session_pending(&self.inner)
    }
    pub fn apply(&mut self, selection: &str) -> Result<(), String> {
        self.inner.apply(selection)?;
        let discoveries = self.inner.discoveries().to_vec();
        let commitment = self.inner.commitment().cloned();
        self.scene
            .apply(selection, &discoveries, commitment.as_ref());
        Ok(())
    }
    pub fn scene(&self) -> String {
        self.scene.to_json()
    }
    pub fn summary(&self) -> String {
        format!("CAVEAT_GRAPHICS|phase={}", self.scene.phase)
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Web3DSession {
    inner: Session,
    map: CaveatMap,
    actions: ActionRuntime,
    world: World3D,
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Web3DSession {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(source: &str) -> Result<Web3DSession, String> {
        let map = CaveatMap::from_source(source)?;
        let actions = ActionRuntime::new(&map)?;
        Ok(Self {
            inner: Session::from_source(source)?,
            map,
            actions,
            world: World3D::the_door(),
        })
    }
    pub fn pending(&self) -> String {
        match self.inner.pending() {
            Ok(PendingInteraction::Investigate {
                name,
                options,
                cost,
                budget,
            }) => {
                let options = self.actions.available_actions(&self.map, &options);
                if options.is_empty() {
                    pending_json(PendingInteraction::Complete)
                } else {
                    pending_json(PendingInteraction::Investigate {
                        name,
                        options,
                        cost,
                        budget,
                    })
                }
            }
            Ok(PendingInteraction::Choice {
                name,
                options,
                budget,
            }) => {
                let options = self.actions.available_actions(&self.map, &options);
                if options.is_empty() {
                    pending_json(PendingInteraction::Complete)
                } else {
                    pending_json(PendingInteraction::Choice {
                        name,
                        options,
                        budget,
                    })
                }
            }
            Ok(PendingInteraction::Complete) => pending_json(PendingInteraction::Complete),
            Err(error) => error_json(&error),
        }
    }
    pub fn world(&self) -> String {
        self.world.to_json()
    }
    pub fn apply(&mut self, selection: &str) -> Result<(), String> {
        let (next_actions, execution) = self.actions.preview(&self.map, selection)?;

        let mut next_inner = self.inner.clone();
        next_inner.apply(selection)?;
        let reopened = next_inner
            .commitment()
            .is_some_and(|commitment| !commitment.reopened_by.is_empty());

        let mut next_world = self.world.clone();
        next_world.apply_execution(&execution, reopened)?;

        self.inner = next_inner;
        self.actions = next_actions;
        self.world = next_world;
        Ok(())
    }
    pub fn discoveries(&self) -> String {
        discoveries_json(self.inner.discoveries())
    }
    pub fn commitment(&self) -> String {
        commitment_json(self.inner.commitment())
    }
}
