use caveat_runtime::reactive::{ReactiveSession, ReactiveSnapshot};
use std::collections::BTreeMap;

const SOURCE: &str = include_str!("../../game/light_the_way.cav");
const FRAME: f64 = 1.0 / 30.0;

fn value(snapshot: &ReactiveSnapshot, name: &str) -> f64 {
    *snapshot
        .values
        .get(name)
        .unwrap_or_else(|| panic!("missing game state {name}"))
}

fn dispatch(
    session: &mut ReactiveSession,
    event: &str,
    parameters: &[(&str, f64)],
) -> ReactiveSnapshot {
    let values = parameters
        .iter()
        .map(|(name, number)| (name.to_string(), *number))
        .collect::<BTreeMap<_, _>>();
    session
        .dispatch(event, &values)
        .unwrap_or_else(|error| panic!("{event} failed: {error}"))
}

fn started(source: &str) -> ReactiveSession {
    let mut session = ReactiveSession::from_source(source).expect("game source should compile");
    assert_eq!(value(&session.snapshot(), "phase"), 0.0);
    let state = dispatch(&mut session, "start", &[]);
    assert_eq!(value(&state, "phase"), 1.0);
    session
}

fn tick(session: &mut ReactiveSession) -> ReactiveSnapshot {
    dispatch(session, "tick", &[("dt", FRAME)])
}

fn aim(session: &mut ReactiveSession, x: f64, z: f64) -> ReactiveSnapshot {
    dispatch(session, "aim", &[("x", x), ("z", z), ("active", 1.0)])
}

// This is a player input script, not a state mutation or an alternate simulator.
// It steers right, left, then right through the three visible reef rows.
fn guided_run(session: &mut ReactiveSession) -> ReactiveSnapshot {
    for _ in 0..(30 * 92) {
        let before = session.snapshot();
        if value(&before, "phase") != 1.0 {
            return before;
        }
        let z = value(&before, "boat_z");
        let x = if z > 41.0 {
            5.5
        } else if z > 32.0 {
            -4.0
        } else {
            5.3
        };
        aim(session, x, z - 8.0);
        tick(session);
    }
    panic!("the rescue did not terminate within its source time limit");
}

#[test]
fn opening_moves_immediately_and_gives_five_seconds_without_damage() {
    let mut session = started(SOURCE);
    let before = session.snapshot();
    for _ in 0..150 {
        let state = tick(&mut session);
        assert_eq!(value(&state, "phase"), 1.0);
        assert_eq!(value(&state, "hull"), 3.0);
        assert_eq!(value(&state, "hit_count"), 0.0);
        assert_eq!(value(&state, "rescued"), 0.0);
    }
    let after = session.snapshot();
    assert!(value(&after, "boat_z") < value(&before, "boat_z") - 1.0);
    assert!(value(&after, "progress") > 0.0);
    assert!((value(&after, "elapsed") - 5.0).abs() < 0.000_001);
}

#[test]
fn steering_through_open_water_rescues_everyone_in_one_short_run() {
    let mut session = started(SOURCE);
    let end = guided_run(&mut session);
    assert_eq!(
        value(&end, "phase"),
        2.0,
        "guided run failed: {:?}",
        end.values
    );
    assert_eq!(value(&end, "rescued"), value(&end, "total_passengers"));
    assert_eq!(value(&end, "rescued"), 32.0);
    assert_eq!(
        value(&end, "hull"),
        3.0,
        "the open-water route should be navigable without damage"
    );
    assert_eq!(value(&end, "hit_count"), 0.0);
    assert!(
        (60.0..=90.0).contains(&value(&end, "elapsed")),
        "the game should last 60–90 seconds, not a lengthy reading session: {:?}",
        end.values
    );
    assert!(
        value(&end, "boat_z") <= 22.0,
        "success must happen at the harbor"
    );
    assert!((value(&end, "boat_x") - 5.3).abs() < 2.9);
}

#[test]
fn doing_nothing_cannot_win_and_a_collision_cannot_remove_three_lives() {
    let mut session = started(SOURCE);
    let mut previous_hull = 3.0;
    let mut previous_hit_at = None::<f64>;
    let mut end = session.snapshot();
    for _ in 0..(30 * 92) {
        end = tick(&mut session);
        let hull = value(&end, "hull");
        if hull != previous_hull {
            let at = value(&end, "elapsed");
            assert_eq!(
                previous_hull - hull,
                1.0,
                "one contact consumed several lives"
            );
            assert!(at >= 5.0, "the player needs a safe opening");
            if let Some(previous) = previous_hit_at {
                assert!(
                    at - previous >= 1.95,
                    "the collision cooldown did not protect the boat"
                );
            }
            previous_hit_at = Some(at);
            previous_hull = hull;
        }
        if value(&end, "phase") != 1.0 {
            break;
        }
    }
    assert_eq!(
        value(&end, "phase"),
        3.0,
        "the untouched game must end in failure: {:?}",
        end.values
    );
    assert_eq!(value(&end, "rescued"), 0.0);
    assert!(value(&end, "elapsed") <= 90.1);
    assert!(
        previous_hit_at.is_some(),
        "a player drifting toward a visible reef needs collision feedback"
    );
}

#[test]
fn holding_the_old_open_flank_no_longer_bypasses_all_the_obstacles() {
    let mut session = started(SOURCE);
    for _ in 0..(30 * 92) {
        let before = session.snapshot();
        if value(&before, "phase") != 1.0 {
            break;
        }
        aim(&mut session, -7.0, value(&before, "boat_z") - 8.0);
        tick(&mut session);
    }
    let end = session.snapshot();
    assert!(
        value(&end, "hit_count") > 0.0,
        "holding one open flank bypassed every reef row"
    );
    assert!(value(&end, "hull") < 3.0);
}

#[test]
fn a_real_sighting_reopens_the_route_without_erasing_the_old_chart() {
    let mut session = started(SOURCE);
    let before = session.snapshot();
    assert_eq!(value(&before, "reef_one_seen"), 0.0);
    assert!(!before
        .relations
        .iter()
        .any(|relation| { relation.from == "reef_one_reading" && relation.relation == "opposes" }));
    aim(&mut session, -1.0, 45.0);
    let after = tick(&mut session);
    assert_eq!(value(&after, "reef_one_seen"), 1.0);
    assert!(after.relations.iter().any(|relation| {
        relation.from == "reef_one_reading"
            && relation.relation == "opposes"
            && relation.to == "clear_course"
    }));
    assert!(
        after.relations.iter().any(|relation| {
            relation.from == "keeper_chart"
                && relation.relation == "supports"
                && relation.to == "clear_course"
        }),
        "contrary observations must not silently remove the original claim's support"
    );
    let commitment = after
        .commitments
        .iter()
        .find(|entry| entry.action == "follow_light")
        .expect("steering should carry an explicit route commitment");
    assert!(commitment.open);
    assert!(commitment
        .retained
        .iter()
        .any(|entry| entry == "reef_one_shadow"));
    assert!(after.symbols.iter().any(|entry| {
        entry.name == "reef_one_shadow" && entry.attention.as_deref() == Some("examined")
    }));
}

#[test]
fn discovering_a_reef_changes_the_ferrys_behavior_before_a_collision() {
    let mut known = started(SOURCE);
    let mut unknown = started(SOURCE);
    aim(&mut known, -1.0, 45.0);
    // A zero-duration observation changes knowledge without moving either boat.
    dispatch(&mut known, "tick", &[("dt", 0.0)]);
    for session in [&mut known, &mut unknown] {
        dispatch(session, "aim", &[("x", 0.0), ("z", 45.0), ("active", 0.0)]);
    }
    assert_eq!(
        value(&known.snapshot(), "boat_z"),
        value(&unknown.snapshot(), "boat_z")
    );
    for _ in 0..750 {
        let informed = tick(&mut known);
        let uninformed = tick(&mut unknown);
        if value(&informed, "boat_speed") < value(&uninformed, "boat_speed") {
            assert_eq!(value(&informed, "warning"), 2.0);
            assert_eq!(value(&uninformed, "warning"), 1.0);
            assert_eq!(
                value(&informed, "boat_speed"),
                value(&informed, "cautious_speed")
            );
            assert_eq!(
                value(&uninformed, "boat_speed"),
                value(&uninformed, "forward_speed")
            );
            assert_eq!(value(&informed, "hull"), 3.0);
            assert_eq!(value(&uninformed, "hull"), 3.0);
            assert!(value(&informed, "boat_z") > value(&uninformed, "boat_z"));
            return;
        }
    }
    panic!("observing a reef changed the graph but never changed the simulation");
}

#[test]
fn source_only_speed_edit_changes_the_running_simulation() {
    let declaration = "state forward_speed = 0.6;";
    assert!(
        SOURCE.contains(declaration),
        "keep the authored speed visible in the source"
    );
    let slower_source = SOURCE.replacen(declaration, "state forward_speed = 0.3;", 1);
    let mut normal = started(SOURCE);
    let mut slower = started(&slower_source);
    let origin = value(&normal.snapshot(), "boat_z");
    for _ in 0..150 {
        tick(&mut normal);
        tick(&mut slower);
    }
    let normal_distance = origin - value(&normal.snapshot(), "boat_z");
    let slower_distance = origin - value(&slower.snapshot(), "boat_z");
    assert!(normal_distance > slower_distance + 1.0);
    assert!((normal_distance - slower_distance * 2.0).abs() < 0.01);
}

#[test]
fn bounded_events_are_atomic_and_identical_inputs_are_deterministic() {
    let mut session = started(SOURCE);
    let before = session.snapshot();
    for (event, payload) in [
        ("tick", r#"{"dt":0.2}"#),
        ("tick", r#"{"dt":-0.1}"#),
        ("tick", r#"{"dt":0.01,"force_win":1}"#),
        ("missing_event", "{}"),
        ("aim", r#"{"x":0,"z":40}"#),
    ] {
        assert!(session.dispatch_json(event, payload).is_err());
        assert_eq!(
            session.snapshot(),
            before,
            "rejected {event} changed the game"
        );
    }
    let invalid = BTreeMap::from([("dt".to_string(), f64::NAN)]);
    assert!(session.dispatch("tick", &invalid).is_err());
    assert_eq!(session.snapshot(), before);

    let mut replay = session.clone();
    for frame in 0..300 {
        if frame % 30 == 0 {
            let x = if frame % 60 == 0 { -7.0 } else { -5.0 };
            aim(&mut session, x, 47.0);
            aim(&mut replay, x, 47.0);
        }
        assert_eq!(tick(&mut session), tick(&mut replay));
    }
    assert!(
        value(&session.snapshot(), "boat_x") < -1.0,
        "aim input did not actually steer the boat"
    );
}

#[test]
fn a_completed_rescue_does_not_keep_running_behind_the_result_screen() {
    let mut session = started(SOURCE);
    let end = guided_run(&mut session);
    assert_eq!(value(&end, "phase"), 2.0);
    for _ in 0..60 {
        let after = tick(&mut session);
        assert_eq!(after.values, end.values);
        assert_eq!(after.relations, end.relations);
        assert_eq!(after.commitments, end.commitments);
    }
}
