use crate::action_runtime::{ActionExecution, WorldCommand};
use std::f32::consts::PI;
use std::fmt::Write as _;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform3D {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}
impl Transform3D {
    pub const fn new(position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Camera3D {
    pub id: String,
    pub transform: Transform3D,
    pub fov: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive3D {
    Box,
    Plane,
    Sphere,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpistemicVisualState {
    Neutral,
    Uncertain,
    Evidence,
    Retained,
    Reopened,
    Committed,
}
impl EpistemicVisualState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Uncertain => "uncertain",
            Self::Evidence => "evidence",
            Self::Retained => "retained",
            Self::Reopened => "reopened",
            Self::Committed => "committed",
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Material3D {
    pub color: Vec3,
    pub emissive: f32,
    pub opacity: f32,
    pub state: EpistemicVisualState,
}
impl Material3D {
    pub const fn new(
        color: Vec3,
        emissive: f32,
        opacity: f32,
        state: EpistemicVisualState,
    ) -> Self {
        Self {
            color,
            emissive,
            opacity,
            state,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Object3D {
    pub id: String,
    pub primitive: Primitive3D,
    pub transform: Transform3D,
    pub parent: Option<String>,
    pub interactive: Option<String>,
    pub material: Material3D,
    pub visible: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub enum LightKind3D {
    Point,
    Directional,
    Ambient,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Light3D {
    pub id: String,
    pub kind: LightKind3D,
    pub position: Vec3,
    pub intensity: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub enum WorldEvent3D {
    RotateY {
        object: String,
        degrees: f32,
        duration: f32,
    },
    MoveTo {
        object: String,
        position: Vec3,
        duration: f32,
    },
    MovePath {
        object: String,
        points: Vec<Vec3>,
        duration: f32,
    },
    LookYaw {
        object: String,
        degrees: f32,
        duration: f32,
    },
    LookPitch {
        object: String,
        degrees: f32,
        duration: f32,
    },
    SetVisible {
        object: String,
        visible: bool,
    },
    SetEpistemicState {
        object: String,
        state: EpistemicVisualState,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct PlaceAnchor3D {
    pub place: String,
    pub position: Vec3,
    pub heading_degrees: f32,
    pub pitch_degrees: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PassageAnchor3D {
    pub entity: String,
    pub position: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct World3D {
    pub id: String,
    pub camera: Camera3D,
    pub objects: Vec<Object3D>,
    pub lights: Vec<Light3D>,
    pub events: Vec<WorldEvent3D>,
    pub anchors: Vec<PlaceAnchor3D>,
    pub passages: Vec<PassageAnchor3D>,
    heading_degrees: f32,
    pitch_degrees: f32,
}

impl World3D {
    pub fn the_door() -> Self {
        let t = |p, s| Transform3D::new(p, Vec3::new(0.0, 0.0, 0.0), s);
        let obj =
            |id: &str, p, s, parent: Option<&str>, interactive: Option<&str>, color, state| {
                Object3D {
                    id: id.into(),
                    primitive: Primitive3D::Box,
                    transform: t(p, s),
                    parent: parent.map(Into::into),
                    interactive: interactive.map(Into::into),
                    material: Material3D::new(
                        color,
                        if state == EpistemicVisualState::Uncertain {
                            0.22
                        } else {
                            0.0
                        },
                        1.0,
                        state,
                    ),
                    visible: true,
                }
            };
        let mut w = Self {
            id: "evacuation_corridor".into(),
            camera: Camera3D {
                id: "player".into(),
                transform: t(Vec3::new(0.0, 1.7, 5.0), Vec3::new(1.0, 1.0, 1.0)),
                fov: 70.0,
            },
            objects: vec![
                obj(
                    "floor",
                    Vec3::new(0.0, -0.1, 0.0),
                    Vec3::new(8.0, 0.2, 18.0),
                    None,
                    None,
                    Vec3::new(0.065, 0.075, 0.095),
                    EpistemicVisualState::Neutral,
                ),
                obj(
                    "wall_left",
                    Vec3::new(-3.3, 1.5, -4.1),
                    Vec3::new(4.2, 3.0, 0.3),
                    None,
                    None,
                    Vec3::new(0.18, 0.19, 0.21),
                    EpistemicVisualState::Neutral,
                ),
                obj(
                    "wall_right",
                    Vec3::new(3.3, 1.5, -4.1),
                    Vec3::new(4.2, 3.0, 0.3),
                    None,
                    None,
                    Vec3::new(0.18, 0.19, 0.21),
                    EpistemicVisualState::Neutral,
                ),
                obj(
                    "header",
                    Vec3::new(0.0, 3.15, -4.1),
                    Vec3::new(2.4, 0.3, 0.3),
                    None,
                    None,
                    Vec3::new(0.18, 0.19, 0.21),
                    EpistemicVisualState::Neutral,
                ),
                obj(
                    "frame_left",
                    Vec3::new(-1.25, 1.5, -3.9),
                    Vec3::new(0.18, 3.0, 0.35),
                    None,
                    None,
                    Vec3::new(0.36, 0.37, 0.39),
                    EpistemicVisualState::Neutral,
                ),
                obj(
                    "frame_right",
                    Vec3::new(1.25, 1.5, -3.9),
                    Vec3::new(0.18, 3.0, 0.35),
                    None,
                    None,
                    Vec3::new(0.36, 0.37, 0.39),
                    EpistemicVisualState::Neutral,
                ),
                obj(
                    "door_hinge",
                    Vec3::new(-1.12, 1.5, -3.72),
                    Vec3::new(0.001, 0.001, 0.001),
                    None,
                    None,
                    Vec3::new(0.2, 0.2, 0.2),
                    EpistemicVisualState::Neutral,
                ),
                obj(
                    "door_a",
                    Vec3::new(1.1, 0.0, 0.0),
                    Vec3::new(2.2, 2.8, 0.16),
                    Some("door_hinge"),
                    Some("open"),
                    Vec3::new(0.30, 0.055, 0.045),
                    EpistemicVisualState::Uncertain,
                ),
                obj(
                    "latch_sensor",
                    Vec3::new(2.0, 0.0, 0.13),
                    Vec3::new(0.14, 0.22, 0.12),
                    Some("door_hinge"),
                    Some("latch_sensor_recently_serviced"),
                    Vec3::new(0.55, 0.32, 0.06),
                    EpistemicVisualState::Uncertain,
                ),
                obj(
                    "camera_7",
                    Vec3::new(-2.5, 2.65, -2.5),
                    Vec3::new(0.32, 0.22, 0.5),
                    None,
                    Some("camera_has_blind_spot"),
                    Vec3::new(0.13, 0.28, 0.38),
                    EpistemicVisualState::Uncertain,
                ),
                obj(
                    "stair_door_hinge",
                    Vec3::new(2.42, 1.45, -8.8),
                    Vec3::new(0.001, 0.001, 0.001),
                    None,
                    None,
                    Vec3::new(0.2, 0.2, 0.2),
                    EpistemicVisualState::Neutral,
                ),
                obj(
                    "stair_door_b",
                    Vec3::new(0.0, 0.0, 1.0),
                    Vec3::new(0.16, 2.7, 2.0),
                    Some("stair_door_hinge"),
                    Some("stairwell"),
                    Vec3::new(0.18, 0.22, 0.24),
                    EpistemicVisualState::Neutral,
                ),
                obj(
                    "stairwell",
                    Vec3::new(4.6, 1.2, -8.8),
                    Vec3::new(0.1, 0.1, 0.1),
                    None,
                    Some("stairwell"),
                    Vec3::new(0.08, 0.28, 0.20),
                    EpistemicVisualState::Neutral,
                ),
                obj(
                    "reopened_marker",
                    Vec3::new(0.0, 3.45, -3.7),
                    Vec3::new(0.8, 0.07, 0.07),
                    None,
                    None,
                    Vec3::new(0.65, 0.08, 0.08),
                    EpistemicVisualState::Reopened,
                ),
            ],
            lights: vec![
                Light3D {
                    id: "emergency".into(),
                    kind: LightKind3D::Point,
                    position: Vec3::new(0.0, 2.7, 1.0),
                    intensity: 0.8,
                },
                Light3D {
                    id: "ambient".into(),
                    kind: LightKind3D::Ambient,
                    position: Vec3::new(0.0, 0.0, 0.0),
                    intensity: 0.18,
                },
            ],
            events: vec![],
            anchors: vec![
                PlaceAnchor3D {
                    place: "start_corridor".into(),
                    position: Vec3::new(0.0, 1.7, 5.0),
                    heading_degrees: 0.0,
                    pitch_degrees: 0.0,
                },
                PlaceAnchor3D {
                    place: "cross_corridor".into(),
                    position: Vec3::new(0.0, 1.7, -5.5),
                    heading_degrees: 0.0,
                    pitch_degrees: 0.0,
                },
                PlaceAnchor3D {
                    place: "continuation_corridor".into(),
                    position: Vec3::new(0.0, 1.7, -10.4),
                    heading_degrees: 0.0,
                    pitch_degrees: 0.0,
                },
                PlaceAnchor3D {
                    place: "alternate_route".into(),
                    position: Vec3::new(-2.2, 1.7, 6.6),
                    heading_degrees: -90.0,
                    pitch_degrees: 0.0,
                },
                PlaceAnchor3D {
                    place: "stair_landing".into(),
                    position: Vec3::new(4.15, 1.55, -8.8),
                    heading_degrees: 90.0,
                    pitch_degrees: 15.0,
                },
                PlaceAnchor3D {
                    place: "stair_flight_down".into(),
                    position: Vec3::new(3.65, 1.55, -9.45),
                    heading_degrees: 90.0,
                    pitch_degrees: 42.0,
                },
            ],
            passages: vec![
                PassageAnchor3D {
                    entity: "door_a".into(),
                    position: Vec3::new(0.0, 1.7, -3.9),
                },
                PassageAnchor3D {
                    entity: "stair_door_b".into(),
                    position: Vec3::new(2.8, 1.7, -8.8),
                },
            ],
            heading_degrees: 0.0,
            pitch_degrees: 0.0,
        };
        if let Some(o) = w.objects.iter_mut().find(|o| o.id == "reopened_marker") {
            o.visible = false
        }
        w
    }
    fn set_state(&mut self, id: &str, state: EpistemicVisualState) {
        if let Some(o) = self.objects.iter_mut().find(|o| o.id == id) {
            o.material.state = state;
            o.material.emissive = match state {
                EpistemicVisualState::Uncertain => 0.22,
                EpistemicVisualState::Evidence => 0.55,
                EpistemicVisualState::Retained => 0.3,
                EpistemicVisualState::Reopened => 0.8,
                EpistemicVisualState::Committed => 0.15,
                EpistemicVisualState::Neutral => 0.0,
            };
        }
        self.events.push(WorldEvent3D::SetEpistemicState {
            object: id.into(),
            state,
        });
    }
    pub fn apply_execution(
        &mut self,
        execution: &ActionExecution,
        reopened: bool,
    ) -> Result<(), String> {
        let mut opened = Vec::new();

        for command in &execution.commands {
            match command {
                WorldCommand::Inspect { entity } => {
                    self.set_state(entity, EpistemicVisualState::Evidence);
                }
                WorldCommand::Operate { .. } | WorldCommand::Observe { .. } => {}
                WorldCommand::Open { entity } => {
                    let target = self
                        .objects
                        .iter()
                        .find(|object| object.id == *entity)
                        .and_then(|object| object.parent.clone())
                        .unwrap_or_else(|| entity.clone());
                    self.events.push(WorldEvent3D::RotateY {
                        object: target,
                        degrees: -92.0,
                        duration: 0.9,
                    });
                    self.set_state(entity, EpistemicVisualState::Retained);
                    opened.push(entity.clone());
                }
                WorldCommand::Move { to, via, .. } => {
                    self.enqueue_move(to, via.as_deref())?;
                }
                WorldCommand::Stay { .. } => {}
            }
        }

        let final_state = if reopened {
            EpistemicVisualState::Reopened
        } else {
            EpistemicVisualState::Committed
        };
        for entity in opened {
            self.set_state(&entity, final_state);
        }

        if reopened {
            if let Some(object) = self
                .objects
                .iter_mut()
                .find(|object| object.id == "reopened_marker")
            {
                object.visible = true;
            }
            self.events.push(WorldEvent3D::SetVisible {
                object: "reopened_marker".into(),
                visible: true,
            });
        }

        Ok(())
    }

    fn enqueue_move(&mut self, place: &str, via: Option<&str>) -> Result<(), String> {
        let destination = self.anchor(place)?.clone();
        let origin = self.camera.transform.position;
        let facing_target = via
            .and_then(|entity| self.passage_anchor(entity))
            .unwrap_or(destination.position);
        self.face_toward(facing_target);

        let mut points = Vec::new();
        if let Some(entity) = via {
            if let Some(passage) = self.passage_anchor(entity) {
                points.push(passage);
            }
        }
        points.push(destination.position);

        let mut distance = 0.0;
        let mut previous = origin;
        for point in &points {
            distance += vec3_distance(previous, *point);
            previous = *point;
        }
        let duration = (distance / 3.0).clamp(0.45, 2.8);

        self.events.push(WorldEvent3D::MovePath {
            object: "player".into(),
            points,
            duration,
        });
        self.camera.transform.position = destination.position;
        self.face_heading(destination.heading_degrees);
        self.face_pitch(destination.pitch_degrees);
        Ok(())
    }

    fn face_toward(&mut self, target: Vec3) {
        let origin = self.camera.transform.position;
        let dx = target.x - origin.x;
        let dz = target.z - origin.z;
        if dx.abs() < 0.001 && dz.abs() < 0.001 {
            return;
        }

        let desired = dx.atan2(-dz) * 180.0 / PI;
        self.face_heading(desired);
    }

    fn face_heading(&mut self, desired: f32) {
        let delta = normalize_degrees(desired - self.heading_degrees);
        if delta.abs() >= 1.0 {
            self.events.push(WorldEvent3D::LookYaw {
                object: "player".into(),
                degrees: delta,
                duration: 0.45,
            });
        }
        self.heading_degrees = desired;
    }

    fn face_pitch(&mut self, desired: f32) {
        let delta = desired - self.pitch_degrees;
        if delta.abs() >= 1.0 {
            self.events.push(WorldEvent3D::LookPitch {
                object: "player".into(),
                degrees: delta,
                duration: 0.35,
            });
        }
        self.pitch_degrees = desired;
    }

    fn anchor(&self, place: &str) -> Result<&PlaceAnchor3D, String> {
        self.anchors
            .iter()
            .find(|anchor| anchor.place == place)
            .ok_or_else(|| format!("3D presentation has no anchor for place {place}"))
    }

    fn passage_anchor(&self, entity: &str) -> Option<Vec3> {
        self.passages
            .iter()
            .find(|anchor| anchor.entity == entity)
            .map(|anchor| anchor.position)
    }

    pub fn to_json(&self) -> String {
        format!("{{\"schema\":4,\"id\":{},\"camera\":{},\"objects\":[{}],\"lights\":[{}],\"events\":[{}]}}",json_string(&self.id),camera_json(&self.camera),self.objects.iter().map(object_json).collect::<Vec<_>>().join(","),self.lights.iter().map(light_json).collect::<Vec<_>>().join(","),self.events.iter().map(event_json).collect::<Vec<_>>().join(","))
    }
}
fn vec3_distance(a: Vec3, b: Vec3) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let dz = b.z - a.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn normalize_degrees(mut degrees: f32) -> f32 {
    while degrees > 180.0 {
        degrees -= 360.0;
    }
    while degrees < -180.0 {
        degrees += 360.0;
    }
    degrees
}

fn vec3_json(v: Vec3) -> String {
    format!("[{:?},{:?},{:?}]", v.x, v.y, v.z)
}
fn transform_json(v: Transform3D) -> String {
    format!(
        "{{\"position\":{},\"rotation\":{},\"scale\":{}}}",
        vec3_json(v.position),
        vec3_json(v.rotation),
        vec3_json(v.scale)
    )
}
fn primitive_name(v: &Primitive3D) -> &'static str {
    match v {
        Primitive3D::Box => "box",
        Primitive3D::Plane => "plane",
        Primitive3D::Sphere => "sphere",
    }
}
fn camera_json(v: &Camera3D) -> String {
    format!(
        "{{\"id\":{},\"transform\":{},\"fov\":{:?}}}",
        json_string(&v.id),
        transform_json(v.transform),
        v.fov
    )
}
fn material_json(v: &Material3D) -> String {
    format!(
        "{{\"color\":{},\"emissive\":{:?},\"opacity\":{:?},\"state\":{}}}",
        vec3_json(v.color),
        v.emissive,
        v.opacity,
        json_string(v.state.as_str())
    )
}
fn object_json(v: &Object3D) -> String {
    format!("{{\"id\":{},\"primitive\":{},\"transform\":{},\"parent\":{},\"interactive\":{},\"material\":{},\"visible\":{}}}",json_string(&v.id),json_string(primitive_name(&v.primitive)),transform_json(v.transform),v.parent.as_ref().map(|x|json_string(x)).unwrap_or_else(||"null".into()),v.interactive.as_ref().map(|x|json_string(x)).unwrap_or_else(||"null".into()),material_json(&v.material),v.visible)
}
fn light_json(v: &Light3D) -> String {
    let kind = match v.kind {
        LightKind3D::Point => "point",
        LightKind3D::Directional => "directional",
        LightKind3D::Ambient => "ambient",
    };
    format!(
        "{{\"id\":{},\"kind\":{},\"position\":{},\"intensity\":{:?}}}",
        json_string(&v.id),
        json_string(kind),
        vec3_json(v.position),
        v.intensity
    )
}

fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            c if c.is_control() => {
                let _ = write!(output, "\\u{:04x}", c as u32);
            }
            c => output.push(c),
        }
    }
    output.push('"');
    output
}

fn event_json(v: &WorldEvent3D) -> String {
    match v {
        WorldEvent3D::RotateY {
            object,
            degrees,
            duration,
        } => format!(
            "{{\"kind\":\"rotate_y\",\"object\":{},\"degrees\":{:?},\"duration\":{:?}}}",
            json_string(object),
            degrees,
            duration
        ),
        WorldEvent3D::MoveTo {
            object,
            position,
            duration,
        } => format!(
            "{{\"kind\":\"move_to\",\"object\":{},\"position\":{},\"duration\":{:?}}}",
            json_string(object),
            vec3_json(*position),
            duration
        ),
        WorldEvent3D::MovePath {
            object,
            points,
            duration,
        } => format!(
            "{{\"kind\":\"move_path\",\"object\":{},\"points\":[{}],\"duration\":{:?}}}",
            json_string(object),
            points
                .iter()
                .map(|p| vec3_json(*p))
                .collect::<Vec<_>>()
                .join(","),
            duration
        ),
        WorldEvent3D::LookYaw {
            object,
            degrees,
            duration,
        } => format!(
            "{{\"kind\":\"look_yaw\",\"object\":{},\"degrees\":{:?},\"duration\":{:?}}}",
            json_string(object),
            degrees,
            duration
        ),
        WorldEvent3D::LookPitch {
            object,
            degrees,
            duration,
        } => format!(
            "{{\"kind\":\"look_pitch\",\"object\":{},\"degrees\":{:?},\"duration\":{:?}}}",
            json_string(object),
            degrees,
            duration
        ),
        WorldEvent3D::SetVisible { object, visible } => format!(
            "{{\"kind\":\"visible\",\"object\":{},\"visible\":{}}}",
            json_string(object),
            visible
        ),
        WorldEvent3D::SetEpistemicState { object, state } => format!(
            "{{\"kind\":\"epistemic_state\",\"object\":{},\"state\":{}}}",
            json_string(object),
            json_string(state.as_str())
        ),
    }
}
