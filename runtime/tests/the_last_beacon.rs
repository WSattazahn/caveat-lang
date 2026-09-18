use caveat_runtime::action_runtime::WorldCommand;
use caveat_runtime::ast::EpistemicCondition;
use caveat_runtime::game_session::{GamePending, GameSession, GameSnapshot};
use caveat_runtime::map::CaveatMap;
use std::collections::{BTreeMap, BTreeSet};

const SOURCE: &str = include_str!("../../game/the_last_beacon.cav");
const OPTIONS: [&[&str]; 6] = [
    &["lens_salt", "chart_age", "radio_echo"],
    &[
        "trust_beam",
        "trust_chart",
        "trust_radio",
        "compare_records",
    ],
    &["tide_lag", "reserve_unknown", "bearing_drift"],
    &[
        "bridge_reserve",
        "mark_channel",
        "establish_contact",
        "preserve_options",
    ],
    &["beam_heat", "shoal_depth", "anchor_hold"],
    &["relight_beacon", "launch_pilot", "hold_offshore"],
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
const OBSERVATIONS: [&str; 3] = ["beacon_signal", "pilot_dispatch", "holding_order"];
const OUTCOMES: [[&str; 3]; 3] = [
    ["beacon_guided", "beacon_limited", "beacon_unverified"],
    ["pilot_rescue", "pilot_delayed", "pilot_stalled"],
    ["hold_verified", "hold_repositioned", "hold_unlocated"],
];
const INVESTIGATIONS: [(&str, &str); 9] = [
    ("lens_salt", "salt_seam"),
    ("chart_age", "dated_survey"),
    ("radio_echo", "repeated_breath"),
    ("tide_lag", "sheltered_strip"),
    ("reserve_unknown", "reserve_charge"),
    ("bearing_drift", "present_bearing"),
    ("beam_heat", "measured_pulses"),
    ("shoal_depth", "leadline_clearance"),
    ("anchor_hold", "anchor_bearing"),
];

fn available(snapshot: &GameSnapshot) -> &[String] {
    let stage = snapshot.turn;
    let (name, options) = match &snapshot.pending {
        GamePending::Investigate { name, options, .. } if stage.is_multiple_of(2) => {
            (name, options)
        }
        GamePending::Choice { name, options, .. } if !stage.is_multiple_of(2) => (name, options),
        other => panic!(
            "wrong pending state after {:?}: {other:?}",
            snapshot.selections
        ),
    };
    assert_eq!(name, INTERACTIONS[stage]);
    assert!(!options.is_empty(), "a legal route must never deadlock");
    assert!(options
        .iter()
        .all(|option| OPTIONS[stage].contains(&option.as_str())));
    options
}

fn condition_holds(condition: &EpistemicCondition, snapshot: &GameSnapshot) -> bool {
    match condition {
        EpistemicCondition::Observed { symbol } => snapshot.relations.iter().any(|relation| {
            relation.from == *symbol
                && ["supports", "opposes"].contains(&relation.relation.as_str())
        }),
        EpistemicCondition::Examined { symbol } => snapshot
            .symbols
            .iter()
            .any(|entry| entry.name == *symbol && entry.attention.as_deref() == Some("examined")),
    }
}

fn assert_no_unearned_observations(snapshot: &GameSnapshot) {
    for (action, evidence) in INVESTIGATIONS {
        if !snapshot
            .selections
            .iter()
            .any(|selection| selection == action)
        {
            assert!(
                !snapshot
                    .discoveries
                    .iter()
                    .any(|entry| entry.evidence == evidence),
                "{evidence} leaked before {action} was chosen"
            );
            assert!(
                !snapshot.relations.iter().any(|relation| {
                    relation.from == evidence
                        && ["supports", "opposes"].contains(&relation.relation.as_str())
                }),
                "{evidence} became active knowledge before its investigation"
            );
        }
    }
    if snapshot.turn < 6 {
        assert!(
            snapshot.outcome.is_none(),
            "future outcome leaked into a live prefix"
        );
        assert!(!snapshot
            .discoveries
            .iter()
            .any(|entry| OBSERVATIONS.contains(&entry.evidence.as_str())));
    }
}

#[derive(Default)]
struct Coverage {
    complete: usize,
    by_action: BTreeMap<String, usize>,
    outcomes: BTreeMap<String, BTreeSet<String>>,
    outcome_counts: BTreeMap<String, usize>,
    gated_prefixes: usize,
    witnesses: BTreeMap<(String, String), Vec<String>>,
}

fn walk(session: &GameSession, coverage: &mut Coverage) {
    let before = session.snapshot().unwrap();
    assert_no_unearned_observations(&before);
    assert!(before.turn <= 6, "story exceeded its six-decision contract");
    if before.pending == GamePending::Complete {
        assert_eq!(before.turn, 6);
        let final_action = before.selections.last().unwrap();
        let ending = OPTIONS[5]
            .iter()
            .position(|action| *action == final_action)
            .unwrap();
        assert_eq!(before.state.current_place, DESTINATIONS[ending]);
        assert_eq!(before.commitments.len(), 3);
        for stage in [1, 3] {
            let commitment = before
                .commitments
                .iter()
                .find(|entry| entry.action == before.selections[stage])
                .unwrap();
            assert!(commitment.open);
            assert!(!commitment.reopened_by.is_empty());
            assert!(commitment
                .reopened_by
                .iter()
                .all(|reason| commitment.retained.contains(reason)));
        }
        let final_commitment = before
            .commitments
            .iter()
            .find(|entry| entry.action == *final_action)
            .unwrap();
        assert!(!final_commitment.open);
        assert_eq!(final_commitment.retained.len(), 9);
        assert_eq!(
            before
                .symbols
                .iter()
                .filter(|entry| entry.attention.as_deref() == Some("examined"))
                .count(),
            3
        );
        assert_eq!(
            before
                .symbols
                .iter()
                .filter(|entry| entry.attention.as_deref() == Some("unexamined"))
                .count(),
            6
        );
        let outcome = before
            .outcome
            .as_ref()
            .expect("every ending needs a source-resolved outcome");
        assert_eq!(outcome.action, *final_action);
        assert_eq!(outcome.turn, 6);
        assert!(outcome
            .basis
            .iter()
            .all(|condition| condition_holds(condition, &before)));
        assert!(before.last_execution.as_ref().unwrap().commands.contains(
            &WorldCommand::Observe {
                symbol: OBSERVATIONS[ending].into()
            }
        ));
        assert_eq!(before.blocked_actions.len(), 0);
        coverage.complete += 1;
        *coverage.by_action.entry(final_action.clone()).or_default() += 1;
        coverage
            .outcomes
            .entry(final_action.clone())
            .or_default()
            .insert(outcome.id.clone());
        *coverage
            .outcome_counts
            .entry(outcome.id.clone())
            .or_default() += 1;
        coverage
            .witnesses
            .entry((final_action.clone(), outcome.id.clone()))
            .or_insert(before.selections);
        return;
    }

    let options = available(&before);
    let expected_count = if [1, 3].contains(&before.turn) { 2 } else { 3 };
    assert_eq!(
        options.len(),
        expected_count,
        "knowledge should change the actual available actions"
    );
    if !before.blocked_actions.is_empty() {
        coverage.gated_prefixes += 1;
    }
    for blocked in &before.blocked_actions {
        assert!(!options.contains(&blocked.action));
        assert!(OPTIONS[before.turn].contains(&blocked.action.as_str()));
        assert!(!blocked.reasons.is_empty());
        assert!(blocked
            .reasons
            .iter()
            .all(|condition| !condition_holds(condition, &before)));
        let mut rejected = session.clone();
        assert!(
            rejected.apply(&blocked.action).is_err(),
            "a disabled action must also be rejected by the runtime"
        );
        assert_eq!(
            rejected.snapshot().unwrap(),
            before,
            "rejection must be atomic"
        );
        assert_eq!(
            rejected.save(),
            session.save(),
            "rejection must not enter the replay history"
        );
    }
    assert_eq!(
        options.len() + before.blocked_actions.len(),
        OPTIONS[before.turn].len()
    );
    for action in options {
        let mut next = session.clone();
        let after = next
            .apply(action)
            .unwrap_or_else(|error| panic!("route {:?}, {action}: {error}", before.selections));
        assert_eq!(after.turn, before.turn + 1);
        assert_eq!(after.selections.last(), Some(action));
        let budget = after.budget.as_ref().unwrap();
        assert_eq!(budget.spent, after.turn.div_ceil(2) as u64);
        assert_eq!(budget.remaining, 3 - budget.spent);
        let execution = after.last_execution.as_ref().unwrap();
        assert_eq!(execution.action, *action);
        assert_eq!(execution.from, "keeper_court");
        assert!(execution
            .commands
            .iter()
            .any(|command| matches!(command, WorldCommand::Observe { .. })));
        if after.turn < 6 {
            assert_eq!(after.state.current_place, "keeper_court");
            assert_eq!(execution.to, "keeper_court");
        } else {
            assert!(after
                .outcome
                .as_ref()
                .unwrap()
                .basis
                .iter()
                .all(|condition| condition_holds(condition, &before)));
        }
        walk(&next, coverage);
    }
}

#[test]
fn all_available_routes_finish_and_prior_knowledge_changes_every_final_action() {
    let initial = GameSession::from_source(SOURCE).unwrap();
    let mut coverage = Coverage::default();
    walk(&initial, &mut coverage);
    assert_eq!(coverage.complete, 324);
    assert!(coverage.gated_prefixes > 0);
    assert_eq!(
        coverage.outcomes.values().map(BTreeSet::len).sum::<usize>(),
        9
    );
    for action in OPTIONS[5] {
        assert_eq!(coverage.by_action[*action], 108);
        assert_eq!(
            coverage.outcomes[*action].len(),
            3,
            "the same physical choice must resolve differently from earlier knowledge"
        );
    }
    assert_eq!(coverage.witnesses.len(), 9);
    for endings in OUTCOMES {
        for (outcome, count) in endings.into_iter().zip([36, 22, 50]) {
            assert_eq!(
                coverage.outcome_counts[outcome], count,
                "unreachable or incorrectly resolved ending: {outcome}"
            );
        }
    }
}

#[test]
fn the_same_investigations_and_final_order_resolve_differently_after_preparation() {
    let mut prefix = GameSession::from_source(SOURCE).unwrap();
    for action in ["radio_echo", "compare_records", "reserve_unknown"] {
        prefix.apply(action).unwrap();
    }
    let mut prepared = prefix.clone();
    for action in ["bridge_reserve", "anchor_hold", "relight_beacon"] {
        prepared.apply(action).unwrap();
    }
    let mut unprepared = prefix.clone();
    for action in ["preserve_options", "anchor_hold", "relight_beacon"] {
        unprepared.apply(action).unwrap();
    }
    let mut verified = prefix;
    for action in ["preserve_options", "beam_heat", "relight_beacon"] {
        verified.apply(action).unwrap();
    }
    let prepared = prepared.snapshot().unwrap();
    let unprepared = unprepared.snapshot().unwrap();
    let verified = verified.snapshot().unwrap();
    assert_eq!(
        prepared.state, unprepared.state,
        "the physical destination is not the outcome selector"
    );
    assert_eq!(prepared.selections.last(), unprepared.selections.last());
    assert_eq!(prepared.budget, unprepared.budget);
    let partial = prepared.outcome.unwrap();
    assert_eq!(partial.id, "beacon_limited");
    assert_eq!(
        partial.basis,
        [EpistemicCondition::Observed {
            symbol: "hot_cable".into()
        }]
    );
    let fallback = unprepared.outcome.unwrap();
    assert_eq!(fallback.id, "beacon_unverified");
    assert!(fallback.basis.is_empty());
    let best = verified.outcome.unwrap();
    assert_eq!(best.id, "beacon_guided");
    assert_eq!(
        best.basis,
        [EpistemicCondition::Observed {
            symbol: "measured_pulses".into()
        }]
    );
}

#[test]
fn opening_state_does_not_run_future_inspections_or_endings() {
    let snapshot = GameSession::from_source(SOURCE)
        .unwrap()
        .snapshot()
        .unwrap();
    available(&snapshot);
    assert_eq!(snapshot.state.current_place, "keeper_court");
    assert_eq!(snapshot.budget.as_ref().unwrap().remaining, 3);
    assert!(snapshot.commitments.is_empty());
    assert!(snapshot.discoveries.is_empty());
    assert!(snapshot.last_execution.is_none());
    assert!(snapshot.blocked_actions.is_empty());
    assert!(snapshot.outcome.is_none());
    assert!(snapshot
        .symbols
        .iter()
        .all(|entry| entry.attention.as_deref() != Some("examined")));
    assert_no_unearned_observations(&snapshot);
}

#[test]
fn first_investigation_unlocks_only_the_action_it_can_justify() {
    let initial = GameSession::from_source(SOURCE).unwrap();
    for (index, action) in OPTIONS[0].iter().enumerate() {
        let mut session = initial.clone();
        let snapshot = session.apply(action).unwrap();
        assert_eq!(snapshot.discoveries.len(), 1);
        assert_eq!(snapshot.discoveries[0].because, *action);
        assert_eq!(snapshot.discoveries[0].evidence, INVESTIGATIONS[index].1);
        assert_eq!(snapshot.discoveries[0].relation, "opposes");
        assert_eq!(snapshot.budget.as_ref().unwrap().remaining, 2);
        assert_eq!(snapshot.state.current_place, "keeper_court");
        assert_eq!(available(&snapshot), [OPTIONS[1][index], "compare_records"]);
        assert_eq!(snapshot.blocked_actions.len(), 2);
        assert_no_unearned_observations(&snapshot);
    }
}

#[test]
fn save_restoration_keeps_gates_resolution_and_completed_world() {
    let mut session = GameSession::from_source(SOURCE).unwrap();
    for action in ["radio_echo", "trust_radio", "reserve_unknown"] {
        session.apply(action).unwrap();
    }
    let before = session.snapshot().unwrap();
    assert_eq!(before.blocked_actions.len(), 2);
    let mut restored = GameSession::restore(SOURCE, &session.save()).unwrap();
    assert_eq!(restored.snapshot().unwrap(), before);
    for action in ["bridge_reserve", "anchor_hold", "hold_offshore"] {
        restored.apply(action).unwrap();
    }
    let finished = restored.snapshot().unwrap();
    let mut replayed = GameSession::restore(SOURCE, &restored.save()).unwrap();
    assert_eq!(replayed.snapshot().unwrap(), finished);
    assert_eq!(finished.state.current_place, "observatory");
    assert!(finished.outcome.is_some());
    assert!(replayed.apply("relight_beacon").is_err());
    assert_eq!(replayed.snapshot().unwrap(), finished);
}

#[test]
fn every_option_and_resolved_ending_has_source_prose_and_a_physical_contract() {
    let map = CaveatMap::from_source(SOURCE).unwrap();
    let snapshot = GameSession::from_source(SOURCE)
        .unwrap()
        .snapshot()
        .unwrap();
    assert_eq!(map.world.action_plans.len(), 20);
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
            .find(|plan| plan.action == *action)
            .unwrap();
        assert!(plan.steps.iter().any(|step| step.kind == "observe"));
    }
    assert_eq!(map.requirements.len(), 6);
    assert_eq!(map.resolutions.len(), 12);
    for resolution in &map.resolutions {
        for suffix in ["title", "result"] {
            let key = format!("{}_{}", resolution.outcome, suffix);
            assert!(
                snapshot.labels.contains_key(&key),
                "missing resolved outcome prose: {key}"
            );
        }
    }
    for (index, action) in OPTIONS[5].iter().enumerate() {
        let plan = map
            .world
            .action_plans
            .iter()
            .find(|plan| plan.action == *action)
            .unwrap();
        assert_eq!(plan.to, DESTINATIONS[index]);
        assert!(plan.steps.iter().any(|step| step.kind == "operate"));
    }
    for stage in ["first_signal", "revised_plan"] {
        assert!(
            map.choices
                .iter()
                .find(|choice| choice.name == stage)
                .unwrap()
                .converging
        );
    }
    assert!(
        !map.choices
            .iter()
            .find(|choice| choice.name == "final_signal")
            .unwrap()
            .converging
    );
}
