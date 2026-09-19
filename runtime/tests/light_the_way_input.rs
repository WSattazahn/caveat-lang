use caveat_runtime::reactive::{ReactiveSession, ReactiveSnapshot};

const SOURCE: &str = include_str!("../../game/light_the_way.cav");

fn started(source: &str) -> ReactiveSession {
    let mut game = ReactiveSession::from_source(source).unwrap();
    game.dispatch_json("start", "{}").unwrap();
    game
}

fn key(game: &mut ReactiveSession, code: &str, active: bool) -> ReactiveSnapshot {
    let event = game.snapshot().controls[&format!("key_{code}")]
        .event
        .clone();
    game.dispatch_json(
        &event,
        if active {
            r#"{"active":1}"#
        } else {
            r#"{"active":0}"#
        },
    )
    .unwrap()
}

fn tick(game: &mut ReactiveSession) -> ReactiveSnapshot {
    game.dispatch_json("tick", r#"{"dt":0.03333333333333333}"#)
        .unwrap()
}

#[test]
fn source_combines_independent_aliases_and_opposing_keys() {
    let mut game = started(SOURCE);
    let initial_x = game.snapshot().values["aim_x"];
    key(&mut game, "ArrowLeft", true);
    key(&mut game, "KeyA", true);
    let both = tick(&mut game);
    assert!((both.values["aim_x"] - (initial_x - 0.4)).abs() < 1e-9);

    key(&mut game, "KeyA", false);
    let arrow_only = tick(&mut game);
    assert!((arrow_only.values["aim_x"] - (initial_x - 0.8)).abs() < 1e-9);
    assert_eq!(arrow_only.values["light_on"], 1.0);

    key(&mut game, "ArrowRight", true);
    let cancelled = tick(&mut game);
    assert_eq!(cancelled.values["aim_x"], arrow_only.values["aim_x"]);
    assert_eq!(cancelled.values["light_on"], 1.0);
    key(&mut game, "ArrowLeft", false);
    let right = tick(&mut game);
    assert!((right.values["aim_x"] - (initial_x - 0.4)).abs() < 1e-9);
    let released = key(&mut game, "ArrowRight", false);
    assert_eq!(released.values["light_on"], 0.0);
    assert_eq!(tick(&mut game).values["aim_x"], right.values["aim_x"]);
}

#[test]
fn pause_and_pointer_takeover_clear_source_held_keys() {
    let mut game = started(SOURCE);
    key(&mut game, "ArrowLeft", true);
    tick(&mut game);
    let paused = game.dispatch_json("pause", "{}").unwrap();
    key(&mut game, "ArrowRight", true);
    game.dispatch_json("resume", "{}").unwrap();
    let resumed = tick(&mut game);
    assert_eq!(resumed.values["aim_x"], paused.values["aim_x"]);
    assert_eq!(resumed.values["light_on"], 0.0);
    assert_eq!(resumed.commitments, paused.commitments);
    assert_eq!(resumed.relations, paused.relations);

    key(&mut game, "ArrowLeft", true);
    tick(&mut game);
    game.dispatch_json("aim", r#"{"x":5,"z":55,"active":1}"#)
        .unwrap();
    let pointer = tick(&mut game);
    assert_eq!(pointer.values["aim_x"], 5.0);
    assert_eq!(pointer.values["light_on"], 1.0);
    let stale_release = key(&mut game, "ArrowLeft", false);
    assert_eq!(stale_release.values["light_on"], 1.0);
    assert_eq!(stale_release.values["aim_x"], 5.0);
    game.dispatch_json("aim", r#"{"x":5,"z":55,"active":0}"#)
        .unwrap();
    assert_eq!(tick(&mut game).values["light_on"], 0.0);
}

#[test]
fn key_control_remapping_is_authored_entirely_in_source() {
    let changed = SOURCE.replace(
        "control key_KeyA = key_a_input;",
        "control key_KeyJ = key_a_input;",
    );
    assert_ne!(changed, SOURCE);
    let mut ordinary = started(SOURCE);
    let mut remapped = started(&changed);
    assert!(!remapped.snapshot().controls.contains_key("key_KeyA"));
    key(&mut ordinary, "KeyA", true);
    key(&mut remapped, "KeyJ", true);
    assert_eq!(tick(&mut ordinary).values, tick(&mut remapped).values);
}

#[test]
fn source_escape_control_toggles_pause_only_on_press() {
    let mut game = started(SOURCE);
    key(&mut game, "ArrowLeft", true);
    tick(&mut game);
    let paused = key(&mut game, "Escape", true);
    assert_eq!(paused.values["paused"], 1.0);
    assert_eq!(paused.values["light_on"], 0.0);
    assert_eq!(key(&mut game, "Escape", false).values["paused"], 1.0);
    key(&mut game, "ArrowRight", true);
    let resumed = key(&mut game, "Escape", true);
    assert_eq!(resumed.values["paused"], 0.0);
    assert_eq!(key(&mut game, "Escape", false).values["paused"], 0.0);
    let after = tick(&mut game);
    assert_eq!(after.values["aim_x"], paused.values["aim_x"]);
    assert_eq!(after.values["light_on"], 0.0);
    assert_eq!(after.commitments, paused.commitments);
    assert_eq!(after.relations, paused.relations);
}

#[test]
fn keyboard_sampling_preserves_observation_cost_and_retained_uncertainty() {
    let mut game = started(SOURCE);
    game.dispatch_json("aim", r#"{"x":-3.8,"z":33.5,"active":0}"#)
        .unwrap();
    let initial = game.snapshot();
    key(&mut game, "Space", true);
    assert!(game.snapshot().reading_streams["flow"]
        .occurrences
        .is_empty());
    for _ in 0..15 {
        tick(&mut game);
    }
    assert!(game.snapshot().reading_streams["flow"]
        .occurrences
        .is_empty());
    key(&mut game, "Space", false);
    tick(&mut game);
    assert_eq!(game.snapshot().values["sample_charge"], 0.0);

    key(&mut game, "Space", true);
    for _ in 0..35 {
        tick(&mut game);
    }
    let sampled = game.snapshot();
    let readings = &sampled.reading_streams["flow"].occurrences;
    assert_eq!(readings.len(), 1);
    assert!(
        sampled.budget.as_ref().unwrap().remaining < initial.budget.as_ref().unwrap().remaining
    );
    assert_eq!(sampled.decision_series["navigation"].revisions.len(), 2);
    let original = initial.decision_series["navigation"]
        .current
        .as_ref()
        .unwrap();
    assert_eq!(
        sampled.commitment_bases[original],
        initial.commitment_bases[original]
    );
    let current = sampled.decision_series["navigation"]
        .current
        .as_ref()
        .unwrap();
    let provenance = &sampled.commitment_bases[current].provenance;
    assert!(provenance.evidence.contains(&readings[0].id));
    assert!(provenance.caveats.contains("surge_unmeasured"));
    assert!(provenance.caveats.contains("reading_may_age"));

    for _ in 0..35 {
        tick(&mut game);
    }
    assert_eq!(game.snapshot().reading_streams["flow"].occurrences.len(), 1);
    assert_eq!(game.snapshot().budget, sampled.budget);
    assert_eq!(
        game.snapshot().commitment_bases[original],
        initial.commitment_bases[original]
    );
}

#[test]
fn malformed_raw_input_is_atomic_and_cannot_leave_a_held_key() {
    let mut game = started(SOURCE);
    let before = game.snapshot();
    let event = before.controls["key_ArrowLeft"].event.clone();
    assert!(game.dispatch_json(&event, r#"{"active":2}"#).is_err());
    assert_eq!(game.snapshot(), before);
    assert_eq!(tick(&mut game).values["aim_x"], before.values["aim_x"]);
}
