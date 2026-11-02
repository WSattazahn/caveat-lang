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

/// Finding: with several player-facing actions that have no world action plan,
/// the map named whichever came first in a hash set, so its load error changed
/// from process to process. It names the first in source order, as a game
/// without plans does. The options are chosen so that the set the old check
/// built would not walk the first of them first.
#[test]
fn several_unplanned_player_facing_actions_name_the_first_in_source_order_not_hash_order() {
    for salt in 0..64 {
        let options = (0..4)
            .map(|index| format!("go{salt}x{index}"))
            .collect::<Vec<_>>();
        let walked = std::iter::once(&options)
            .flat_map(|options| options.iter())
            .collect::<caveat_runtime::fast_hash::HashSet<_>>();
        if walked.iter().next() == Some(&&options[0]) {
            continue;
        }
        let source = format!(
            "place a kind corridor; start_at a; action wait from a to a steps stay; \
             choice route options {};",
            options.join(", ")
        );
        assert_eq!(
            CaveatMap::from_source(&source).map(|_| ()).unwrap_err(),
            format!(
                "player-facing action {} has no world action plan",
                options[0]
            )
        );
        return;
    }
    panic!("no options put a hash set's first against source order");
}
