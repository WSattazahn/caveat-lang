use caveat_runtime::action_runtime::{ActionExecution, WorldCommand};
use caveat_runtime::source_library::{FunctionValue, SourceLibrary};
use caveat_runtime::web::Web3DSession;
use caveat_runtime::world3d::{PlaceAnchor3D, Vec3, World3D, WorldEvent3D};
use serde_json::Value;

const DOOR: &str = include_str!("../../game/the_door_round2.cav");
const WEB_DOOR: &str = include_str!("../../web/the_door_round2.cav");

fn number(library: &SourceLibrary, name: &str, arguments: &[f32]) -> f32 {
    let arguments = arguments
        .iter()
        .map(|value| f64::from(*value))
        .collect::<Vec<_>>();
    let result = library.call_numbers(name, &arguments).unwrap();
    let FunctionValue::Number(number) = result.value else {
        panic!("expected numeric function {name}");
    };
    number as f32
}

fn boolean(library: &SourceLibrary, name: &str, arguments: &[f32]) -> bool {
    let arguments = arguments
        .iter()
        .map(|value| f64::from(*value))
        .collect::<Vec<_>>();
    let result = library.call_numbers(name, &arguments).unwrap();
    let FunctionValue::Bool(value) = result.value else {
        panic!("expected boolean function {name}");
    };
    value
}

fn world(session: &Web3DSession) -> Value {
    serde_json::from_str(&session.world()).unwrap()
}

fn after_inspection(source: &str) -> Web3DSession {
    let mut session = Web3DSession::new(source).unwrap();
    session.apply("latch_sensor_recently_serviced").unwrap();
    session
}

fn move_to(place: &str) -> ActionExecution {
    ActionExecution {
        action: "move".into(),
        from: "start_corridor".into(),
        to: place.into(),
        commands: vec![WorldCommand::Move {
            from: "start_corridor".into(),
            to: place.into(),
            via: None,
        }],
    }
}

#[test]
fn canonical_source_and_compatibility_constructor_use_the_same_motion_policy() {
    // The web build overwrites this legacy source copy with game/*.cav. Direct
    // web consumers and the bundled native default must still see one policy.
    assert_eq!(WEB_DOOR, DOOR);
    assert_eq!(
        World3D::the_door().to_json(),
        World3D::the_door_from_source(DOOR).unwrap().to_json()
    );
}

#[test]
fn source_motion_preserves_native_float_boundaries_and_signed_half_turns() {
    let library = SourceLibrary::from_source(DOOR).unwrap();
    // These reference operations are the replaced native policy. Compare at
    // the f32 command boundary, including both sides of each clamp/threshold.
    for distance in [
        0.0_f32, 0.01, 1.3499999, 1.35, 1.3500001, 3.0, 8.399999, 8.4, 8.400001, 30.0,
    ] {
        assert_eq!(
            number(&library, "door_move_duration", &[distance]),
            (distance / 3.0).clamp(0.45, 2.8)
        );
    }
    for delta in [
        -1_000_000.1_f32,
        -900.0,
        -540.0,
        -360.0,
        -180.00002,
        -180.0,
        -179.99998,
        -0.0,
        0.0,
        179.99998,
        180.0,
        180.00002,
        360.0,
        540.0,
        900.0,
        1_000_000.1,
    ] {
        let mut prior = delta;
        while prior > 180.0 {
            prior -= 360.0;
        }
        while prior < -180.0 {
            prior += 360.0;
        }
        assert_eq!(number(&library, "door_turn_delta", &[delta]), prior);
    }
    for delta in [
        -1.0000001_f32,
        -1.0,
        -0.99999994,
        0.0,
        0.99999994,
        1.0,
        1.0000001,
    ] {
        assert_eq!(
            boolean(&library, "door_animate_turn", &[delta]),
            delta.abs() >= 1.0
        );
    }
    for dx in [-0.001_f32, -0.0009999999, 0.0, 0.0009999999, 0.001] {
        for dz in [-0.001_f32, -0.0009999999, 0.0, 0.0009999999, 0.001] {
            assert_eq!(
                boolean(&library, "door_face_target", &[dx, dz]),
                !(dx.abs() < 0.001 && dz.abs() < 0.001)
            );
        }
    }
    for (name, expected) in [
        ("door_yaw_duration", 0.45_f32),
        ("door_pitch_duration", 0.35),
        ("door_open_duration", 0.9),
        ("door_open_degrees", -92.0),
    ] {
        assert_eq!(number(&library, name, &[]), expected);
    }
}

#[test]
fn default_policy_retains_parent_hinges_waypoints_and_precise_camera_geometry() {
    let mut session = after_inspection(DOOR);
    session.apply("open").unwrap();
    let opened = world(&session);
    let events = opened["events"].as_array().unwrap();
    let opening = events
        .iter()
        .find(|event| event["kind"] == "rotate_y")
        .unwrap();
    assert_eq!(opening["object"], "door_hinge");
    assert_eq!(opening["degrees"], -92.0);
    assert_eq!(opening["duration"], 0.9);
    let travel = events
        .iter()
        .find(|event| event["kind"] == "move_path")
        .unwrap();
    assert_eq!(
        travel["points"],
        serde_json::json!([[0.0, 1.7, -3.9], [0.0, 1.7, -5.5]])
    );
    assert_eq!(travel["duration"], 2.8);

    let mut scene = World3D::the_door();
    let target = Vec3::new(-2.2, 1.7, 6.6);
    scene.anchors.push(PlaceAnchor3D {
        place: "target".into(),
        position: target,
        heading_degrees: -90.0,
        pitch_degrees: 30.0,
    });
    let origin = scene.camera.transform.position;
    scene.apply_execution(&move_to("target"), false).unwrap();
    let desired =
        (target.x - origin.x).atan2(-(target.z - origin.z)) * 180.0 / std::f32::consts::PI;
    assert!(
        matches!(&scene.events[0], WorldEvent3D::LookYaw { degrees, duration, .. } if *degrees == desired && *duration == 0.45)
    );
    let distance = ((target.x - origin.x).powi(2)
        + (target.y - origin.y).powi(2)
        + (target.z - origin.z).powi(2))
    .sqrt();
    assert!(
        matches!(&scene.events[1], WorldEvent3D::MovePath { points, duration, .. } if points == &[target] && *duration == (distance / 3.0).clamp(0.45, 2.8))
    );
    assert!(
        matches!(&scene.events[2], WorldEvent3D::LookYaw { degrees, duration, .. } if *degrees == -90.0 - desired && *duration == 0.45)
    );
    assert!(
        matches!(&scene.events[3], WorldEvent3D::LookPitch { degrees, duration, .. } if *degrees == 30.0 && *duration == 0.35)
    );
}

#[test]
fn editing_only_caveat_changes_real_commands_without_changing_evidence_or_materials() {
    let changed = DOOR
        .replace(
            "fn door_move_duration(distance) = clamp(distance / 3, 0.45, 2.8);",
            "fn door_move_duration(distance) = clamp(distance / 6, 0.45, 2.8);",
        )
        .replace(
            "fn door_open_degrees() = -92;",
            "fn door_open_degrees() = -70;",
        )
        .replace(
            "fn door_open_duration() = 0.9;",
            "fn door_open_duration() = 1.25;",
        )
        .replace(
            "fn door_yaw_duration() = 0.45;",
            "fn door_yaw_duration() = 0.75;",
        )
        .replace(
            "fn door_pitch_duration() = 0.35;",
            "fn door_pitch_duration() = 0.5;",
        );
    let mut original = after_inspection(DOOR);
    let mut variant = after_inspection(&changed);
    for action in ["open", "stairwell"] {
        original.apply(action).unwrap();
        variant.apply(action).unwrap();
        assert_eq!(original.pending(), variant.pending());
        assert_eq!(original.discoveries(), variant.discoveries());
        assert_eq!(original.commitment(), variant.commitment());
        let normal = world(&original);
        let modified = world(&variant);
        assert_eq!(normal["objects"], modified["objects"]);
        assert_eq!(normal["camera"], modified["camera"]);
        assert_eq!(normal["lights"], modified["lights"]);
        let events = modified["events"].as_array().unwrap();
        let opening = events
            .iter()
            .find(|event| event["kind"] == "rotate_y")
            .unwrap();
        assert_eq!(opening["degrees"], -70.0);
        assert_eq!(opening["duration"], 1.25);
        let travel = events
            .iter()
            .find(|event| event["kind"] == "move_path")
            .unwrap();
        assert_eq!(travel["duration"], 1.75);
        if action == "stairwell" {
            assert!(events
                .iter()
                .any(|event| event["kind"] == "look_yaw" && event["duration"] == 0.75));
            assert!(events
                .iter()
                .any(|event| event["kind"] == "look_pitch" && event["duration"] == 0.5));
        }
    }
}

#[test]
fn invalid_motion_signatures_are_rejected_before_creating_a_session() {
    for (before, after) in [
        ("fn door_open_degrees() = -92;", "fn other_degrees() = -92;"),
        (
            "fn door_open_degrees() = -92;",
            "fn door_open_degrees(value) = value;",
        ),
        (
            "fn door_face_target(dx, dz) = abs(dx) >= 0.001 or abs(dz) >= 0.001;",
            "fn door_face_target(dx, dz) = 1;",
        ),
    ] {
        let source = DOOR.replace(before, after);
        assert_ne!(source, DOOR);
        assert!(Web3DSession::new(&source).is_err());
    }
}

#[test]
fn a_late_source_policy_failure_rolls_back_the_entire_web_action() {
    for replacement in [
        "fn door_move_duration(distance) = require(distance < 4, 1);",
        "fn door_move_duration(distance) = 0;",
        "fn door_move_duration(distance) = -1;",
        "fn door_move_duration(distance) = 1e-90;",
    ] {
        let source = DOOR.replace(
            "fn door_move_duration(distance) = clamp(distance / 3, 0.45, 2.8);",
            replacement,
        );
        let mut session = after_inspection(&source);
        let before = (
            session.world(),
            session.pending(),
            session.discoveries(),
            session.commitment(),
        );
        // The logical commitment and opening event both precede movement.
        // Their effects must disappear when the movement policy fails.
        assert!(session.apply("open").is_err());
        assert_eq!(
            (
                session.world(),
                session.pending(),
                session.discoveries(),
                session.commitment()
            ),
            before
        );
        session.apply("wait").unwrap();
        assert!(!world(&session)["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["kind"] == "rotate_y"));
    }
}

#[test]
fn a_late_source_policy_failure_rolls_back_direct_native_world_execution() {
    let execution = ActionExecution {
        action: "open_and_cross".into(),
        from: "start_corridor".into(),
        to: "cross_corridor".into(),
        commands: vec![
            WorldCommand::Open {
                entity: "door_a".into(),
            },
            WorldCommand::Move {
                from: "start_corridor".into(),
                to: "cross_corridor".into(),
                via: Some("door_a".into()),
            },
        ],
    };
    for replacement in [
        "fn door_move_duration(distance) = require(distance < 4, 1);",
        "fn door_move_duration(distance) = 0;",
        "fn door_move_duration(distance) = 1e-90;",
    ] {
        let source = DOOR.replace(
            "fn door_move_duration(distance) = clamp(distance / 3, 0.45, 2.8);",
            replacement,
        );
        let mut scene = World3D::the_door_from_source(&source).unwrap();
        let before = scene.clone();
        assert!(scene.apply_execution(&execution, true).is_err());
        // This checks internal headings and the compiled policy as well as
        // all public geometry, object materials, visibility and queued events.
        assert_eq!(scene, before);
        assert_eq!(scene.to_json(), before.to_json());
    }
}
