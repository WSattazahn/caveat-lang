use caveat_runtime::action_runtime::ActionRuntime;
use caveat_runtime::map::CaveatMap;
use caveat_runtime::web::Web3DSession;

const DOOR: &str = include_str!("../../game/the_door_round2.cav");

#[test]
fn the_door_actions_are_source_authored_and_reachable() {
    let map = CaveatMap::from_source(DOOR).expect("The Door map should validate");
    let names = map
        .world
        .action_plans
        .iter()
        .map(|plan| plan.action.as_str())
        .collect::<Vec<_>>();

    for required in [
        "latch_sensor_recently_serviced",
        "camera_has_blind_spot",
        "open",
        "wait",
        "reroute",
        "continue",
        "retreat",
        "stairwell",
    ] {
        assert!(names.contains(&required), "missing plan for {required}");
    }

    let stairwell = map
        .world
        .action_plans
        .iter()
        .find(|plan| plan.action == "stairwell")
        .expect("stairwell plan should exist");
    assert_eq!(stairwell.from, "cross_corridor");
    assert_eq!(stairwell.to, "stair_flight_down");
    assert!(stairwell
        .steps
        .iter()
        .any(|step| step.kind == "through" && step.target.as_deref() == Some("stair_door_b")));
}

#[test]
fn open_then_stairwell_changes_logical_place_without_action_name_special_cases() {
    let map = CaveatMap::from_source(DOOR).expect("The Door map should validate");
    let runtime = ActionRuntime::new(&map).expect("The Door should have start_at");

    let (runtime, inspected) = runtime
        .preview(&map, "latch_sensor_recently_serviced")
        .expect("inspection should execute");
    assert_eq!(inspected.to, "start_corridor");
    assert_eq!(runtime.current_place(), "start_corridor");

    let (runtime, opened) = runtime.preview(&map, "open").expect("open should execute");
    assert_eq!(opened.to, "cross_corridor");
    assert_eq!(runtime.current_place(), "cross_corridor");
    assert!(runtime.is_open("door_a"));

    let (runtime, stairs) = runtime
        .preview(&map, "stairwell")
        .expect("stairwell should execute");
    assert_eq!(stairs.to, "stair_flight_down");
    assert_eq!(runtime.current_place(), "stair_flight_down");
    assert!(runtime.is_open("stair_door_b"));
}

#[test]
fn wait_and_reroute_do_not_make_cross_corridor_actions_available() {
    let map = CaveatMap::from_source(DOOR).expect("The Door map should validate");
    let runtime = ActionRuntime::new(&map).expect("The Door should have start_at");

    let (waited, _) = runtime.preview(&map, "wait").expect("wait should execute");
    assert_eq!(waited.current_place(), "start_corridor");
    assert!(!waited.is_available(&map, "continue"));
    assert!(!waited.is_available(&map, "retreat"));
    assert!(!waited.is_available(&map, "stairwell"));

    let (rerouted, _) = runtime
        .preview(&map, "reroute")
        .expect("reroute should execute");
    assert_eq!(rerouted.current_place(), "alternate_route");
    assert!(!rerouted.is_available(&map, "continue"));
}

#[test]
fn web_session_initially_exposes_the_investigation_actions() {
    let session = Web3DSession::new(DOOR).expect("Web3D session should initialize");
    let pending = session.pending();

    assert!(pending.contains("\"kind\":\"investigate\""));
    assert!(pending.contains("latch_sensor_recently_serviced"));
    assert!(pending.contains("camera_has_blind_spot"));
}

#[test]
fn web_session_stops_offering_impossible_revised_routes_after_wait() {
    let mut session = Web3DSession::new(DOOR).expect("Web3D session should initialize");
    session
        .apply("camera_has_blind_spot")
        .expect("camera inspection should apply");
    session.apply("wait").expect("wait should apply");

    assert_eq!(session.pending(), "{\"kind\":\"complete\"}");
}

#[test]
fn web_3d_uses_entity_parent_hinges_and_endpoint_pitch() {
    let mut session = Web3DSession::new(DOOR).expect("Web3D session should initialize");
    session
        .apply("latch_sensor_recently_serviced")
        .expect("latch inspection should apply");
    session.apply("open").expect("Door A should open");

    let after_open: serde_json::Value =
        serde_json::from_str(&session.world()).expect("world JSON should parse");
    assert!(after_open["events"].as_array().is_some_and(|events| {
        events
            .iter()
            .any(|event| event["kind"] == "rotate_y" && event["object"] == "door_hinge")
    }));

    session
        .apply("stairwell")
        .expect("stairwell route should execute");
    let after_stairs: serde_json::Value =
        serde_json::from_str(&session.world()).expect("world JSON should parse");
    let events = after_stairs["events"]
        .as_array()
        .expect("world events should be an array");

    assert!(events
        .iter()
        .any(|event| { event["kind"] == "rotate_y" && event["object"] == "stair_door_hinge" }));
    assert!(events.iter().any(|event| event["kind"] == "look_pitch"));
    assert_eq!(
        after_stairs["camera"]["transform"]["position"],
        serde_json::json!([4.55, 1.05, -9.45])
    );
}
