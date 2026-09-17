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
        Self { position, rotation, scale }
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

#[derive(Debug, Clone, PartialEq)]
pub struct Object3D {
    pub id: String,
    pub primitive: Primitive3D,
    pub transform: Transform3D,
    pub interactive: Option<String>,
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
    RotateY { object: String, degrees: f32, duration: f32 },
    MoveTo { object: String, position: Vec3, duration: f32 },
    SetVisible { object: String, visible: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub struct World3D {
    pub id: String,
    pub camera: Camera3D,
    pub objects: Vec<Object3D>,
    pub lights: Vec<Light3D>,
    pub events: Vec<WorldEvent3D>,
}

impl World3D {
    pub fn the_door() -> Self {
        let transform = |position, scale| {
            Transform3D::new(position, Vec3::new(0.0, 0.0, 0.0), scale)
        };

        Self {
            id: "evacuation_corridor".into(),
            camera: Camera3D {
                id: "player".into(),
                transform: transform(Vec3::new(0.0, 1.7, 5.0), Vec3::new(1.0, 1.0, 1.0)),
                fov: 70.0,
            },
            objects: vec![
                Object3D { id: "floor".into(), primitive: Primitive3D::Box, transform: transform(Vec3::new(0.0, -0.1, 0.0), Vec3::new(8.0, 0.2, 18.0)), interactive: None },
                Object3D { id: "door".into(), primitive: Primitive3D::Box, transform: transform(Vec3::new(0.0, 1.2, -4.0), Vec3::new(2.2, 2.4, 0.2)), interactive: Some("open".into()) },
                Object3D { id: "latch".into(), primitive: Primitive3D::Box, transform: transform(Vec3::new(1.0, 1.2, -3.8), Vec3::new(0.15, 0.25, 0.12)), interactive: Some("latch_sensor_recently_serviced".into()) },
                Object3D { id: "security_camera".into(), primitive: Primitive3D::Box, transform: transform(Vec3::new(-2.5, 2.6, -2.0), Vec3::new(0.3, 0.2, 0.5)), interactive: Some("camera_has_blind_spot".into()) },
                Object3D { id: "stairwell".into(), primitive: Primitive3D::Box, transform: transform(Vec3::new(3.0, 1.0, -7.0), Vec3::new(2.0, 2.0, 0.2)), interactive: Some("stairwell".into()) },
            ],
            lights: vec![
                Light3D { id: "emergency".into(), kind: LightKind3D::Point, position: Vec3::new(0.0, 2.7, 1.0), intensity: 0.8 },
                Light3D { id: "ambient".into(), kind: LightKind3D::Ambient, position: Vec3::new(0.0, 0.0, 0.0), intensity: 0.18 },
            ],
            events: Vec::new(),
        }
    }

    pub fn apply_action(&mut self, action: &str, reopened: bool) {
        match action {
            "open" => self.events.push(WorldEvent3D::RotateY { object: "door".into(), degrees: 90.0, duration: 1.2 }),
            "stairwell" => self.events.push(WorldEvent3D::MoveTo { object: "player".into(), position: Vec3::new(3.0, 1.7, -6.0), duration: 1.5 }),
            "retreat" => self.events.push(WorldEvent3D::MoveTo { object: "player".into(), position: Vec3::new(0.0, 1.7, 7.0), duration: 1.2 }),
            _ => {}
        }
        if reopened {
            self.events.push(WorldEvent3D::SetVisible { object: "reopened_marker".into(), visible: true });
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"schema\":1,\"id\":{},\"camera\":{},\"objects\":[{}],\"lights\":[{}],\"events\":[{}]}}",
            json_string(&self.id),
            camera_json(&self.camera),
            self.objects.iter().map(object_json).collect::<Vec<_>>().join(","),
            self.lights.iter().map(light_json).collect::<Vec<_>>().join(","),
            self.events.iter().map(event_json).collect::<Vec<_>>().join(",")
        )
    }
}

fn vec3_json(value: Vec3) -> String { format!("[{:?},{:?},{:?}]", value.x, value.y, value.z) }
fn transform_json(value: Transform3D) -> String { format!("{{\"position\":{},\"rotation\":{},\"scale\":{}}}", vec3_json(value.position), vec3_json(value.rotation), vec3_json(value.scale)) }
fn primitive_name(value: &Primitive3D) -> &'static str { match value { Primitive3D::Box => "box", Primitive3D::Plane => "plane", Primitive3D::Sphere => "sphere" } }
fn camera_json(value: &Camera3D) -> String { format!("{{\"id\":{},\"transform\":{},\"fov\":{:?}}}", json_string(&value.id), transform_json(value.transform), value.fov) }
fn object_json(value: &Object3D) -> String { format!("{{\"id\":{},\"primitive\":{},\"transform\":{},\"interactive\":{}}}", json_string(&value.id), json_string(primitive_name(&value.primitive)), transform_json(value.transform), value.interactive.as_ref().map(|x| json_string(x)).unwrap_or_else(|| "null".into())) }
fn light_json(value: &Light3D) -> String { let kind = match value.kind { LightKind3D::Point => "point", LightKind3D::Directional => "directional", LightKind3D::Ambient => "ambient" }; format!("{{\"id\":{},\"kind\":{},\"position\":{},\"intensity\":{:?}}}", json_string(&value.id), json_string(kind), vec3_json(value.position), value.intensity) }
fn event_json(value: &WorldEvent3D) -> String { match value { WorldEvent3D::RotateY { object, degrees, duration } => format!("{{\"kind\":\"rotate_y\",\"object\":{},\"degrees\":{:?},\"duration\":{:?}}}", json_string(object), degrees, duration), WorldEvent3D::MoveTo { object, position, duration } => format!("{{\"kind\":\"move_to\",\"object\":{},\"position\":{},\"duration\":{:?}}}", json_string(object), vec3_json(*position), duration), WorldEvent3D::SetVisible { object, visible } => format!("{{\"kind\":\"visible\",\"object\":{},\"visible\":{}}}", json_string(object), visible) } }

fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            c if c.is_control() => { let _ = write!(output, "\\u{:04x}", c as u32); }
            c => output.push(c),
        }
    }
    output.push('"');
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn door_world_has_interactions() {
        let world = World3D::the_door();
        assert_eq!(world.objects.iter().find(|object| object.id == "security_camera").unwrap().interactive.as_deref(), Some("camera_has_blind_spot"));
        assert!(world.to_json().contains("evacuation_corridor"));
    }

    #[test]
    fn open_emits_door_rotation_and_reopen_marker() {
        let mut world = World3D::the_door();
        world.apply_action("open", true);
        assert!(matches!(world.events[0], WorldEvent3D::RotateY { ref object, .. } if object == "door"));
        assert!(matches!(world.events[1], WorldEvent3D::SetVisible { ref object, visible: true } if object == "reopened_marker"));
    }
}