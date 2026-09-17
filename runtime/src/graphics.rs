use std::fmt::Write as _;

use crate::session::{CommitmentFeedback, Discovery};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualStatus {
    Unknown,
    Supported,
    Opposed,
    Retained,
    Reopened,
    Settled,
}

impl VisualStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Supported => "supported",
            Self::Opposed => "opposed",
            Self::Retained => "retained",
            Self::Reopened => "reopened",
            Self::Settled => "settled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualEffect {
    None,
    Occlude,
    Pulse,
    Ghost,
    Reveal,
}

impl VisualEffect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Occlude => "occlude",
            Self::Pulse => "pulse",
            Self::Ghost => "ghost",
            Self::Reveal => "reveal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualNode {
    pub id: String,
    pub shape: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub status: VisualStatus,
    pub effect: VisualEffect,
    pub opacity: u8,
    pub visible: bool,
    pub label: String,
}

impl VisualNode {
    fn new(
        id: &str,
        shape: &str,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        label: &str,
    ) -> Self {
        Self {
            id: id.into(),
            shape: shape.into(),
            x,
            y,
            width,
            height,
            status: VisualStatus::Unknown,
            effect: VisualEffect::None,
            opacity: 100,
            visible: true,
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualCaveat {
    pub id: String,
    pub target: String,
    pub consequence: String,
    pub status: VisualStatus,
    pub effect: VisualEffect,
    pub active: bool,
}

impl VisualCaveat {
    fn new(id: &str, target: &str, consequence: &str, effect: VisualEffect) -> Self {
        Self {
            id: id.into(),
            target: target.into(),
            consequence: consequence.into(),
            status: VisualStatus::Unknown,
            effect,
            active: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualTrail {
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaveatScene {
    pub title: String,
    pub phase: String,
    pub message: String,
    pub nodes: Vec<VisualNode>,
    pub caveats: Vec<VisualCaveat>,
    pub trail: Vec<VisualTrail>,
}

impl CaveatScene {
    pub fn door() -> Self {
        let mut scene = Self {
            title: "CAVEAT Graphics".into(),
            phase: "initial".into(),
            message: "The scene is provisional. Investigate a caveat to change what can be seen."
                .into(),
            nodes: vec![
                VisualNode::new(
                    "corridor",
                    "corridor",
                    70,
                    90,
                    760,
                    390,
                    "the evacuation corridor",
                ),
                VisualNode::new("door", "door", 350, 150, 200, 270, "the sealed door"),
                VisualNode::new(
                    "blind_spot",
                    "occlusion",
                    555,
                    180,
                    245,
                    205,
                    "the camera blind spot",
                ),
                VisualNode::new(
                    "latch",
                    "caveat",
                    245,
                    330,
                    135,
                    120,
                    "the latch sensor caveat",
                ),
                VisualNode::new(
                    "retained_uncertainty",
                    "badge",
                    600,
                    30,
                    255,
                    48,
                    "retained uncertainty",
                ),
                VisualNode::new(
                    "old_commitment",
                    "history",
                    335,
                    135,
                    230,
                    300,
                    "the earlier open commitment",
                ),
                VisualNode::new(
                    "retreat_path",
                    "path",
                    555,
                    375,
                    260,
                    85,
                    "the retreat path",
                ),
            ],
            caveats: vec![
                VisualCaveat::new(
                    "latch_sensor_recently_serviced",
                    "latch",
                    "high",
                    VisualEffect::Pulse,
                ),
                VisualCaveat::new(
                    "camera_has_blind_spot",
                    "blind_spot",
                    "material",
                    VisualEffect::Occlude,
                ),
            ],
            trail: Vec::new(),
        };

        scene.set_node("blind_spot", VisualStatus::Unknown, VisualEffect::Occlude, 100, false);
        scene.set_node("latch", VisualStatus::Unknown, VisualEffect::Pulse, 100, false);
        scene.set_node(
            "retained_uncertainty",
            VisualStatus::Unknown,
            VisualEffect::None,
            100,
            false,
        );
        scene.set_node("old_commitment", VisualStatus::Unknown, VisualEffect::Ghost, 100, false);
        scene.set_node("retreat_path", VisualStatus::Unknown, VisualEffect::Reveal, 100, false);
        scene
    }

    pub fn apply(
        &mut self,
        selection: &str,
        discoveries: &[Discovery],
        commitment: Option<&CommitmentFeedback>,
    ) {
        match selection {
            "latch_sensor_recently_serviced" => {
                self.phase = "latch-evidence".into();
                self.message =
                    "The maintenance record undermines confidence in the latch.".into();
                self.push_trail("INVESTIGATED", label(selection));
                self.activate_caveat(
                    "latch_sensor_recently_serviced",
                    VisualStatus::Opposed,
                    VisualEffect::Pulse,
                );
            }
            "camera_has_blind_spot" => {
                self.phase = "camera-evidence".into();
                self.message =
                    "The blind spot is now part of the scene instead of an invisible assumption."
                        .into();
                self.push_trail("INVESTIGATED", label(selection));
                self.activate_caveat(
                    "camera_has_blind_spot",
                    VisualStatus::Opposed,
                    VisualEffect::Occlude,
                );
            }
            "wait" => {
                self.phase = "waited".into();
                self.message = "Waiting preserves the unresolved scene.".into();
            }
            "reroute" => {
                self.phase = "rerouted".into();
                self.message = "The corridor is abandoned while uncertainty remains visible.".into();
                self.set_node(
                    "retreat_path",
                    VisualStatus::Settled,
                    VisualEffect::Reveal,
                    100,
                    true,
                );
            }
            "continue" => {
                self.phase = "continued".into();
                self.message = "The earlier plan continues with uncertainty retained.".into();
                self.set_node(
                    "door",
                    VisualStatus::Retained,
                    VisualEffect::Ghost,
                    65,
                    true,
                );
            }
            "stairwell" => {
                self.phase = "stairwell".into();
                self.message = "The alternate route becomes the visible commitment.".into();
                self.set_node(
                    "retreat_path",
                    VisualStatus::Settled,
                    VisualEffect::Reveal,
                    100,
                    true,
                );
            }
            _ => {}
        }

        for discovery in discoveries {
            self.push_trail(
                "DISCOVERED",
                format!(
                    "{} {} {}",
                    label(&discovery.evidence),
                    discovery.relation,
                    label(&discovery.target)
                ),
            );

            match discovery.evidence.as_str() {
                "maintenance_record" => self.activate_caveat(
                    "latch_sensor_recently_serviced",
                    VisualStatus::Opposed,
                    VisualEffect::Pulse,
                ),
                "coverage_test" => self.activate_caveat(
                    "camera_has_blind_spot",
                    VisualStatus::Opposed,
                    VisualEffect::Occlude,
                ),
                _ => {}
            }
        }

        if let Some(feedback) = commitment {
            self.push_trail("COMMITTED", label(&feedback.action));
            for retained in &feedback.retained {
                self.push_trail("RETAINED", label(retained));
            }
            for reopened_by in &feedback.reopened_by {
                self.push_trail("REOPENED", format!("because {}", label(reopened_by)));
            }

            match feedback.action.as_str() {
                "open" if !feedback.reopened_by.is_empty() => {
                    self.phase = "reopened".into();
                    self.message =
                        "The open decision is visible, but new evidence has reopened it.".into();
                    self.set_node(
                        "door",
                        VisualStatus::Reopened,
                        VisualEffect::Ghost,
                        55,
                        true,
                    );
                    self.set_node(
                        "old_commitment",
                        VisualStatus::Reopened,
                        VisualEffect::Ghost,
                        100,
                        true,
                    );
                    self.set_node(
                        "retained_uncertainty",
                        VisualStatus::Retained,
                        VisualEffect::Pulse,
                        100,
                        true,
                    );
                }
                "open" => {
                    self.phase = "opened".into();
                    self.message = "The door is open, with retained uncertainty still visible.".into();
                    self.set_node(
                        "door",
                        VisualStatus::Retained,
                        VisualEffect::Ghost,
                        70,
                        true,
                    );
                    self.set_node(
                        "retained_uncertainty",
                        VisualStatus::Retained,
                        VisualEffect::Pulse,
                        100,
                        true,
                    );
                }
                "retreat" => {
                    self.phase = "retreated".into();
                    self.message =
                        "The revised decision settles the route without pretending uncertainty vanished."
                            .into();
                    self.set_node(
                        "door",
                        VisualStatus::Settled,
                        VisualEffect::None,
                        100,
                        true,
                    );
                    self.set_node(
                        "retreat_path",
                        VisualStatus::Settled,
                        VisualEffect::Reveal,
                        100,
                        true,
                    );
                }
                _ => {}
            }
        }
    }

    pub fn to_json(&self) -> String {
        let nodes = self
            .nodes
            .iter()
            .map(node_json)
            .collect::<Vec<_>>()
            .join(",");
        let caveats = self
            .caveats
            .iter()
            .map(caveat_json)
            .collect::<Vec<_>>()
            .join(",");
        let trail = self
            .trail
            .iter()
            .map(trail_json)
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{{\"schema\":1,\"title\":{},\"phase\":{},\"message\":{},\"nodes\":[{}],\"caveats\":[{}],\"trail\":[{}]}}",
            json_string(&self.title),
            json_string(&self.phase),
            json_string(&self.message),
            nodes,
            caveats,
            trail
        )
    }

    fn activate_caveat(&mut self, id: &str, status: VisualStatus, effect: VisualEffect) {
        let target = self.caveats.iter_mut().find(|c| c.id == id).map(|caveat| {
            caveat.active = true;
            caveat.status = status;
            caveat.effect = effect;
            caveat.target.clone()
        });

        if let Some(target) = target {
            self.set_node(&target, status, effect, 100, true);
        }
    }

    fn set_node(
        &mut self,
        id: &str,
        status: VisualStatus,
        effect: VisualEffect,
        opacity: u8,
        visible: bool,
    ) {
        if let Some(node) = self.nodes.iter_mut().find(|node| node.id == id) {
            node.status = status;
            node.effect = effect;
            node.opacity = opacity;
            node.visible = visible;
        }
    }

    fn push_trail(&mut self, kind: &str, text: impl Into<String>) {
        self.trail.push(VisualTrail {
            kind: kind.into(),
            text: text.into(),
        });
    }
}

fn label(id: &str) -> String {
    match id {
        "latch_sensor_recently_serviced" => "the latch sensor was recently serviced".into(),
        "camera_has_blind_spot" => "the camera has a blind spot".into(),
        "maintenance_record" => "the maintenance record".into(),
        "coverage_test" => "the camera diagnostic".into(),
        "latch_alarm_reliable" => "the latch alarm is reliable".into(),
        "corridor_clear" => "the corridor is clear".into(),
        "door_unsafe" => "the door may be unsafe".into(),
        "smoke_report" => "the hallway detector report".into(),
        _ => id.replace('_', " "),
    }
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');

    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(out, "\\u{:04x}", character as u32);
            }
            character => out.push(character),
        }
    }

    out.push('"');
    out
}

fn node_json(node: &VisualNode) -> String {
    format!(
        "{{\"id\":{},\"shape\":{},\"x\":{},\"y\":{},\"width\":{},\"height\":{},\"status\":{},\"effect\":{},\"opacity\":{},\"visible\":{},\"label\":{}}}",
        json_string(&node.id),
        json_string(&node.shape),
        node.x,
        node.y,
        node.width,
        node.height,
        json_string(node.status.as_str()),
        json_string(node.effect.as_str()),
        node.opacity,
        if node.visible { "true" } else { "false" },
        json_string(&node.label)
    )
}

fn caveat_json(caveat: &VisualCaveat) -> String {
    format!(
        "{{\"id\":{},\"target\":{},\"consequence\":{},\"status\":{},\"effect\":{},\"active\":{}}}",
        json_string(&caveat.id),
        json_string(&caveat.target),
        json_string(&caveat.consequence),
        json_string(caveat.status.as_str()),
        json_string(caveat.effect.as_str()),
        if caveat.active { "true" } else { "false" }
    )
}

fn trail_json(entry: &VisualTrail) -> String {
    format!(
        "{{\"kind\":{},\"text\":{}}}",
        json_string(&entry.kind),
        json_string(&entry.text)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Session;

    const SOURCE: &str = include_str!("../../web/the_door_round2.cav");

    fn step(session: &mut Session, scene: &mut CaveatScene, selection: &str) {
        session.apply(selection).expect("valid scenario selection");
        let discoveries = session.discoveries().to_vec();
        let commitment = session.commitment().cloned();
        scene.apply(selection, &discoveries, commitment.as_ref());
    }

    #[test]
    fn visual_state_preserves_and_reopens_uncertainty() {
        let mut session = Session::from_source(SOURCE).expect("scenario parses");
        let mut scene = CaveatScene::door();

        step(&mut session, &mut scene, "camera_has_blind_spot");
        assert_eq!(scene.phase, "camera-evidence");
        assert!(scene.caveats.iter().any(|c| c.active));
        assert!(scene.to_json().contains("\"effect\":\"occlude\""));

        step(&mut session, &mut scene, "open");
        assert_eq!(scene.phase, "reopened");
        assert!(scene
            .nodes
            .iter()
            .any(|node| node.id == "old_commitment" && node.visible));

        step(&mut session, &mut scene, "retreat");
        assert_eq!(scene.phase, "retreated");
        assert!(scene.trail.iter().any(|entry| entry.kind == "REOPENED"));
    }
}
