use caveat_runtime::action_runtime::WorldCommand;
use caveat_runtime::game_session::{GamePending, GameSession, GameSnapshot};
use caveat_runtime::map::CaveatMap;

const SOURCE: &str = include_str!("../../game/the_last_beacon.cav");
const OPTIONS: [[&str; 3]; 6] = [
    ["lens_salt", "chart_age", "radio_echo"],
    ["trust_beam", "trust_chart", "trust_radio"],
    ["tide_lag", "reserve_unknown", "bearing_drift"],
    ["bridge_reserve", "mark_channel", "establish_contact"],
    ["beam_heat", "shoal_depth", "anchor_hold"],
    ["relight_beacon", "launch_pilot", "hold_offshore"],
];
const INTERACTIONS: [&str; 6] = [
    "first_watch",
    "first_signal",
    "changed_water",
    "revised_plan",
    "final_watch",
    "final_signal",
];
const DESTINATIONS: [&str; 3] = ["lantern_room", "harbor", "observatory"];
const CONCLUSIONS: [&str; 3] = ["harbor_arrival", "beach_rescue", "crossing_postponed"];
const OBSERVATIONS: [&str; 3] = ["beacon_arrival", "pilot_transfer", "offshore_hold"];

fn assert_pending(snapshot: &GameSnapshot, stage: usize) {
    let (name, options) = match &snapshot.pending {
        GamePending::Investigate { name, options, .. } if stage.is_multiple_of(2) => {
            (name, options)
        }
        GamePending::Choice { name, options, .. } if !stage.is_multiple_of(2) => (name, options),
        other => panic!("wrong pending state at stage {stage}: {other:?}"),
    };
    assert_eq!(name, INTERACTIONS[stage]);
    assert_eq!(options, &OPTIONS[stage]);
}

#[test]
fn all_729_routes_preserve_attention_reopening_and_distinct_outcomes() {
    let initial = GameSession::from_source(SOURCE).expect("beacon source should initialize");
    let mut ending_counts = [0_usize; 3];

    for route_number in 0..729_usize {
        let mut route_code = route_number;
        let mut route = [0_usize; 6];
        for selection in &mut route {
            *selection = route_code % 3;
            route_code /= 3;
        }
        let mut session = initial.clone();
        let mut snapshot = session.snapshot().unwrap();
        for (stage, &selection) in route.iter().enumerate() {
            assert_pending(&snapshot, stage);
            let action = OPTIONS[stage][selection];
            snapshot = session
                .apply(action)
                .unwrap_or_else(|error| panic!("route {route:?}, action {action}: {error}"));
            assert_eq!(snapshot.turn, stage + 1);
            assert_eq!(snapshot.selections[stage], action);
            let budget = snapshot.budget.as_ref().expect("attention budget exists");
            assert_eq!(budget.spent, ((stage + 2) / 2) as u64);
            assert_eq!(budget.remaining, 3 - budget.spent);

            let execution = snapshot.last_execution.as_ref().unwrap();
            assert_eq!(execution.action, action);
            assert_eq!(execution.from, "keeper_court");
            assert!(execution
                .commands
                .iter()
                .any(|command| { matches!(command, WorldCommand::Observe { .. }) }));
            if stage < 5 {
                assert_eq!(snapshot.state.current_place, "keeper_court");
                assert_eq!(execution.to, "keeper_court");
            }
        }

        let ending = route[5];
        ending_counts[ending] += 1;
        assert_eq!(snapshot.pending, GamePending::Complete);
        assert_eq!(snapshot.state.current_place, DESTINATIONS[ending]);
        assert_eq!(snapshot.commitments.len(), 3);
        for stage in [1, 3] {
            let selected = OPTIONS[stage][route[stage]];
            let commitment = snapshot
                .commitments
                .iter()
                .find(|commitment| commitment.action == selected)
                .expect("selected preparation remains in history");
            assert!(commitment.open, "{selected} should have reopened");
            assert_eq!(commitment.reopened_by.len(), 1);
            assert!(commitment.retained.contains(&commitment.reopened_by[0]));
        }
        let final_commitment = snapshot
            .commitments
            .iter()
            .find(|commitment| commitment.action == OPTIONS[5][ending])
            .unwrap();
        assert!(!final_commitment.open);
        assert_eq!(final_commitment.retained.len(), 9);
        assert_eq!(
            snapshot
                .symbols
                .iter()
                .filter(|symbol| symbol.attention.as_deref() == Some("examined"))
                .count(),
            3
        );
        assert_eq!(
            snapshot
                .symbols
                .iter()
                .filter(|symbol| symbol.attention.as_deref() == Some("unexamined"))
                .count(),
            6
        );
        for (index, conclusion) in CONCLUSIONS.iter().enumerate() {
            let realized = snapshot
                .relations
                .iter()
                .any(|relation| relation.relation == "supports" && relation.to == *conclusion);
            assert_eq!(realized, index == ending);
        }
        assert!(snapshot.last_execution.as_ref().unwrap().commands.contains(
            &WorldCommand::Observe {
                symbol: OBSERVATIONS[ending].into()
            }
        ));
    }
    assert_eq!(ending_counts, [243, 243, 243]);
}

#[test]
fn opening_state_does_not_run_future_inspections_or_endings() {
    let session = GameSession::from_source(SOURCE).unwrap();
    let snapshot = session.snapshot().unwrap();
    assert_pending(&snapshot, 0);
    assert_eq!(snapshot.state.current_place, "keeper_court");
    assert_eq!(snapshot.budget.as_ref().unwrap().remaining, 3);
    assert!(snapshot.commitments.is_empty());
    assert!(snapshot.discoveries.is_empty());
    assert!(snapshot.last_execution.is_none());
    assert!(snapshot
        .symbols
        .iter()
        .all(|symbol| symbol.attention.as_deref() != Some("examined")));
    assert!(!snapshot
        .relations
        .iter()
        .any(|relation| CONCLUSIONS.contains(&relation.to.as_str())));
}

#[test]
fn each_first_investigation_exposes_only_its_own_evidence() {
    let evidence = ["salt_seam", "dated_survey", "repeated_breath"];
    let initial = GameSession::from_source(SOURCE).unwrap();
    for (index, action) in OPTIONS[0].iter().enumerate() {
        let mut session = initial.clone();
        let snapshot = session.apply(action).unwrap();
        assert_eq!(snapshot.discoveries.len(), 1);
        assert_eq!(snapshot.discoveries[0].because, *action);
        assert_eq!(snapshot.discoveries[0].evidence, evidence[index]);
        assert_eq!(snapshot.discoveries[0].relation, "opposes");
        assert_eq!(snapshot.budget.unwrap().remaining, 2);
        for (candidate, symbol) in OPTIONS[0].iter().enumerate() {
            let caveat = snapshot
                .symbols
                .iter()
                .find(|entry| entry.name == *symbol)
                .unwrap();
            assert_eq!(
                caveat.attention.as_deref(),
                Some(if candidate == index {
                    "examined"
                } else {
                    "unexamined"
                })
            );
        }
    }
}

#[test]
fn save_restoration_keeps_the_revision_record_and_completed_world() {
    let mut session = GameSession::from_source(SOURCE).unwrap();
    for action in [
        "radio_echo",
        "trust_chart",
        "reserve_unknown",
        "mark_channel",
    ] {
        session.apply(action).unwrap();
    }
    let before = session.snapshot().unwrap();
    let mut restored = GameSession::restore(SOURCE, &session.save()).unwrap();
    assert_eq!(restored.snapshot().unwrap(), before);
    restored.apply("anchor_hold").unwrap();
    restored.apply("hold_offshore").unwrap();
    let finished = restored.snapshot().unwrap();
    let mut replayed = GameSession::restore(SOURCE, &restored.save()).unwrap();
    assert_eq!(replayed.snapshot().unwrap(), finished);
    assert_eq!(finished.state.current_place, "observatory");
    assert!(replayed.apply("relight_beacon").is_err());
    assert_eq!(replayed.snapshot().unwrap(), finished);
}

#[test]
fn every_visible_option_has_source_feedback_and_a_physical_contract() {
    let map = CaveatMap::from_source(SOURCE).unwrap();
    let snapshot = GameSession::from_source(SOURCE)
        .unwrap()
        .snapshot()
        .unwrap();
    assert_eq!(map.world.action_plans.len(), 18);
    assert_eq!(map.world.places.len(), 7);
    assert_eq!(map.world.entities.len(), 10);
    for stage in INTERACTIONS {
        assert!(snapshot.labels.contains_key(stage));
        assert!(snapshot.labels.contains_key(&format!("{stage}_body")));
    }
    for action in OPTIONS.into_iter().flatten() {
        for key in [
            action.to_string(),
            format!("{action}_hint"),
            format!("{action}_result"),
        ] {
            assert!(
                snapshot.labels.contains_key(&key),
                "missing source prose: {key}"
            );
        }
        let plan = map
            .world
            .action_plans
            .iter()
            .find(|plan| plan.action == action)
            .unwrap();
        assert!(plan.steps.iter().any(|step| step.kind == "observe"));
    }
    for (index, action) in OPTIONS[5].iter().enumerate() {
        assert!(snapshot.labels.contains_key(&format!("{action}_title")));
        let plan = map
            .world
            .action_plans
            .iter()
            .find(|plan| plan.action == *action)
            .unwrap();
        assert_eq!(plan.to, DESTINATIONS[index]);
        assert!(plan.steps.iter().any(|step| step.kind == "operate"));
    }
    let choices = &map.choices;
    assert!(
        choices
            .iter()
            .find(|choice| choice.name == "first_signal")
            .unwrap()
            .converging
    );
    assert!(
        choices
            .iter()
            .find(|choice| choice.name == "revised_plan")
            .unwrap()
            .converging
    );
    assert!(
        !choices
            .iter()
            .find(|choice| choice.name == "final_signal")
            .unwrap()
            .converging
    );
}
