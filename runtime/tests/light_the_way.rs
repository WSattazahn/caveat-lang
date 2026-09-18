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
    for frame in 0..(30 * 92) {
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
        if frame % 30 == 0 {
            // The browser test uses the same once-per-second steering policy;
            // every intervening 30 Hz simulation tick still runs in Caveat.
            aim(session, x, z - 8.0);
        }
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
        (50.0..=90.0).contains(&value(&end, "elapsed")),
        "the game should last 50–90 seconds, not a lengthy reading session: {:?}",
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
    assert!(
        after.relations.iter().any(|edge| {
            edge.from == "reef_one_shadow"
                && edge.relation == "qualifies"
                && edge.to == "keeper_chart"
        }),
        "examining uncertainty must not remove its qualification"
    );
    assert_eq!(
        after.budget.as_ref().unwrap().remaining + 1,
        before.budget.as_ref().unwrap().remaining
    );
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
fn pausing_keeps_the_live_evidence_and_commitments_then_resumes_motion() {
    let mut session = started(SOURCE);
    aim(&mut session, -1.0, 45.0);
    tick(&mut session);
    let before = session.snapshot();
    let pause_event = before
        .controls
        .get("pause")
        .expect("source pause control")
        .event
        .clone();
    let paused = dispatch(&mut session, &pause_event, &[]);
    assert_eq!(value(&paused, "paused"), 1.0);
    for _ in 0..90 {
        let after = tick(&mut session);
        for field in [
            "boat_x",
            "boat_z",
            "boat_vx",
            "elapsed",
            "hull",
            "hit_count",
        ] {
            assert_eq!(
                value(&after, field),
                value(&paused, field),
                "{field} changed while paused"
            );
        }
        assert_eq!(after.relations, before.relations);
        assert_eq!(after.commitments, before.commitments);
        assert_eq!(after.budget, before.budget);
    }
    let resume_event = paused
        .controls
        .get("resume")
        .expect("source resume control")
        .event
        .clone();
    dispatch(&mut session, &resume_event, &[]);
    let after = tick(&mut session);
    assert_eq!(value(&after, "paused"), 0.0);
    assert!(value(&after, "elapsed") > value(&paused, "elapsed"));
    assert!(value(&after, "boat_z") < value(&paused, "boat_z"));
    assert_eq!(after.relations, before.relations);
    assert_eq!(after.commitments, before.commitments);
}

#[test]
fn discovering_the_current_revises_navigation_without_changing_physical_truth() {
    let mut informed = started(SOURCE);
    let mut uninformed = started(SOURCE);
    aim(&mut informed, -3.8, 33.5);
    let observed = dispatch(&mut informed, "tick", &[("dt", 0.0)]);
    assert!(observed.relations.iter().any(|edge| {
        edge.from == "morning_forecast"
            && edge.relation == "supports"
            && edge.to == "crosscurrent_mild"
    }));
    assert!(observed.relations.iter().any(|edge| {
        edge.from == "crosscurrent_reading"
            && edge.relation == "opposes"
            && edge.to == "crosscurrent_mild"
    }));
    let earlier = observed
        .commitments
        .iter()
        .find(|entry| entry.action == "trust_forecast")
        .unwrap();
    assert!(earlier.open, "the provisional forecast should reopen");
    assert!(earlier
        .retained
        .iter()
        .any(|symbol| symbol == "surge_unmeasured"));
    let revised = observed
        .commitments
        .iter()
        .find(|entry| entry.action == "counter_steer")
        .unwrap();
    assert!(revised
        .retained
        .iter()
        .any(|symbol| symbol == "surge_unmeasured"));

    for _ in 0..(30 * 80) {
        let before = uninformed.snapshot();
        let z = value(&before, "boat_z");
        let x = if z > 41.0 {
            5.5
        } else if z > 32.0 {
            -4.0
        } else {
            5.3
        };
        // Identical legitimate steering inputs; keeping the light aft prevents
        // the uninformed run from accidentally scouting the current later.
        aim(&mut informed, x, 60.0);
        aim(&mut uninformed, x, 60.0);
        let known = tick(&mut informed);
        let unknown = tick(&mut uninformed);
        let force = value(&unknown, "current_force");
        if force.abs() > 0.000_001 {
            assert!(
                (value(&known, "current_force") - force).abs() < 0.000_001,
                "knowledge changed the current itself rather than the response to it"
            );
            assert_eq!(value(&unknown, "compensation"), 0.0);
            assert!(
                value(&known, "compensation") * force < 0.0,
                "the revised commitment must counter the real current"
            );
            assert!(
                (value(&known, "boat_x") - value(&unknown, "boat_x")).abs() > 0.000_000_1
                    || (value(&known, "boat_vx") - value(&unknown, "boat_vx")).abs() > 0.000_000_1,
                "the knowledge-based response never affected navigation"
            );
            assert!(unknown
                .commitments
                .iter()
                .all(|entry| entry.action != "counter_steer"));
            assert!(known
                .relations
                .iter()
                .any(|edge| edge.from == "morning_forecast" && edge.relation == "supports"));
            assert!(known
                .relations
                .iter()
                .any(|edge| edge.from == "crosscurrent_reading" && edge.relation == "opposes"));
            return;
        }
        assert_eq!(value(&known, "boat_x"), value(&unknown, "boat_x"));
        assert_eq!(value(&known, "boat_z"), value(&unknown, "boat_z"));
    }
    panic!("the source-authored crosscurrent never affected the route");
}

#[test]
fn source_presentation_overrides_do_not_replace_the_games_live_knowledge() {
    let variant = format!("{SOURCE}\nbind passengers_label.text = \"SOURCE VARIANT CREW\";\nbind crosscurrent_marker.scale = 1.5;\nbind crosscurrent_marker.ring.color = \"#ff00ff\";\nbind crosscurrent_marker.visible = true;\ncue qa_source_tone sound 523.25 0.07 0.02;\non start emit qa_source_tone;\n");
    let mut normal = started(SOURCE);
    let mut changed = started(&variant);
    for session in [&mut normal, &mut changed] {
        aim(session, -1.0, 45.0);
        tick(session);
    }
    let original = normal.snapshot();
    let edited = changed.snapshot();
    let json = serde_json::to_value(&edited).unwrap();
    assert_eq!(
        json["bindings"]["passengers_label"]["text"],
        "SOURCE VARIANT CREW"
    );
    assert_eq!(json["bindings"]["crosscurrent_marker"]["scale"], 1.5);
    assert_eq!(edited.values, original.values);
    assert_eq!(edited.relations, original.relations);
    assert_eq!(edited.commitments, original.commitments);
    assert_eq!(edited.budget, original.budget);
}

#[test]
fn an_observation_emits_feedback_once_without_spending_attention_again() {
    let mut session = started(SOURCE);
    aim(&mut session, -1.0, 45.0);
    let first = dispatch(&mut session, "tick", &[("dt", 0.0)]);
    assert!(
        !first.cues.is_empty(),
        "new evidence should have visible or audible feedback"
    );
    let second = dispatch(&mut session, "tick", &[("dt", 0.0)]);
    assert!(
        second.cues.is_empty(),
        "the same sighting replayed its feedback every frame"
    );
    assert_eq!(second.relations, first.relations);
    assert_eq!(second.commitments, first.commitments);
    assert_eq!(second.budget, first.budget);
}

#[test]
fn fully_examined_uncertainty_stays_in_the_graph_and_still_allows_rescue() {
    let mut session = started(SOURCE);
    let points = session
        .snapshot()
        .world
        .presentation
        .positions
        .into_iter()
        .filter(|entry| entry.target.starts_with("reef_") || entry.target == "crosscurrent_marker")
        .map(|entry| (entry.position[0].value(), entry.position[2].value()))
        .collect::<Vec<_>>();
    assert_eq!(points.len(), 7);
    for (x, z) in points {
        aim(&mut session, x, z);
        dispatch(&mut session, "tick", &[("dt", 0.0)]);
    }
    let known = session.snapshot();
    assert_eq!(known.budget.as_ref().unwrap().remaining, 0);
    assert_eq!(known.symbols.iter().filter(|entry| entry.kind == "caveat" && entry.attention.as_deref() == Some("examined")).count(), 7);
    let qualifications = known
        .relations
        .iter()
        .filter(|edge| edge.relation == "qualifies")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        qualifications.len(),
        7,
        "examined caveats must remain actual graph edges"
    );
    assert!(known
        .relations
        .iter()
        .any(|edge| edge.relation == "supports" && edge.to == "clear_course"));
    assert!(known
        .relations
        .iter()
        .any(|edge| edge.relation == "opposes" && edge.to == "clear_course"));
    let end = guided_run(&mut session);
    assert_eq!(
        value(&end, "phase"),
        2.0,
        "retained uncertainty must permit deliberate action: {:?}",
        end.values
    );
    assert_eq!(value(&end, "rescued"), 32.0);
    assert!(qualifications
        .iter()
        .all(|edge| end.relations.contains(edge)));
    assert_eq!(end.commitments, known.commitments);
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
