use caveat_runtime::map::{CaveatMap, MAP_SCHEMA};

const DOOR: &str = include_str!("../../game/the_door_round2.cav");

#[test]
fn the_door_has_ai_readable_semantic_map() {
    let map = CaveatMap::from_source(DOOR).expect("The Door should map");

    assert_eq!(map.schema, MAP_SCHEMA);
    assert!(map
        .symbols
        .iter()
        .any(|symbol| symbol.name == "camera_has_blind_spot"));
    assert!(map
        .relations
        .iter()
        .any(|relation| relation.from == "camera_has_blind_spot"
            && relation.relation == "qualifies"
            && relation.to == "camera_frame"));
    assert!(map
        .actions
        .iter()
        .any(|action| action.id == "stairwell" && action.choice == "revised_decision"));
    assert!(map
        .world
        .places
        .iter()
        .any(|place| place.id == "stair_landing" && place.kind == "stairwell"));
    assert!(map.world.connections.iter().any(|connection| {
        connection.from == "cross_corridor"
            && connection.to == "stair_landing"
            && connection.via.as_deref() == Some("stair_door_b")
    }));
}

#[test]
fn inspect_camera_gap_finds_retention_and_reopening() {
    let map = CaveatMap::from_source(DOOR).expect("The Door should map");
    let inspection = map.inspect("camera_has_blind_spot");

    assert!(inspection
        .choices
        .iter()
        .any(|choice| choice == "door_decision"));
    assert!(inspection
        .choices
        .iter()
        .any(|choice| choice == "revised_decision"));
    assert!(inspection
        .conditionals
        .iter()
        .any(|effect| effect.contains("reopen open because camera_has_blind_spot")));
}

#[test]
fn trace_camera_gap_reaches_corridor_clear() {
    let map = CaveatMap::from_source(DOOR).expect("The Door should map");
    let trace = map.trace("camera_has_blind_spot");

    assert!(trace
        .nodes
        .iter()
        .any(|node| node.symbol == "corridor_clear"));
}

#[test]
fn map_json_is_stable_machine_readable_json() {
    let map = CaveatMap::from_source(DOOR).expect("The Door should map");
    let json = map.to_json_pretty().expect("map should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("map should be valid JSON");

    assert_eq!(value["schema"], MAP_SCHEMA);
    assert!(value["actions"].is_array());
    assert!(value["world"]["places"].is_array());
}


#[test]
fn world_trace_reaches_stairwell_through_declared_topology() {
    let map = CaveatMap::from_source(DOOR).expect("The Door should map");
    let trace = map.trace("start_corridor");

    assert!(trace
        .nodes
        .iter()
        .any(|node| node.symbol == "stair_landing"));
    assert!(trace
        .nodes
        .iter()
        .any(|node| node.symbol == "stair_flight_down"));
}
