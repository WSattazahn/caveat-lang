//! Public behavior across high-degree/shared collector adjacency changes.
//! The profile driver uses the same source; these checks assert reasons and
//! atomic continuation rather than any particular index representation.
use caveat_runtime::reactive::{ArchiveItem, ReactiveSession};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const HIGH_DEGREE: &str =
    include_str!("../../experiments/departure-gate/index-compaction/high-degree.cav");

fn send(game: &mut ReactiveSession, event: &str, payload: &BTreeMap<String, f64>) {
    game.apply(event, payload)
        .unwrap_or_else(|error| panic!("{event}: {error}"));
}

fn records(archive: &[ArchiveItem]) -> Vec<Value> {
    archive
        .iter()
        .filter_map(|item| match item {
            ArchiveItem::Record(entry) => Some(serde_json::to_value(entry).unwrap()),
            ArchiveItem::Provenance(_) => None,
        })
        .collect()
}

fn record_names(archive: &[ArchiveItem]) -> Vec<String> {
    records(archive)
        .iter()
        .map(|entry| entry["record"].as_str().unwrap().into())
        .collect()
}

fn archived(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.clone().drain_archive()).unwrap()
}

fn dynamic_retired(game: &ReactiveSession) -> Vec<String> {
    game.snapshot()
        .retired
        .keys()
        .filter(|name| name.contains('@'))
        .cloned()
        .collect()
}

fn paired(
    game: &mut ReactiveSession,
    restored: &mut ReactiveSession,
    event: &str,
    payload: &BTreeMap<String, f64>,
) {
    send(game, event, payload);
    send(restored, event, payload);
    assert_eq!(game.snapshot(), restored.snapshot(), "{event}");
    assert_eq!(game.save_json().unwrap(), restored.save_json().unwrap());
    assert_eq!(archived(game), archived(restored), "{event}");
}

fn rejected_without_change(game: &mut ReactiveSession, event: &str, payload: &str, origin: &str) {
    let before = game.clone();
    let save = game.save_json().unwrap();
    #[cfg(feature = "collector-metrics")]
    let metrics = game.withdrawal_collection_metrics();
    let outcome =
        serde_json::to_value(game.dispatch_outcome_json(event, payload).unwrap()).unwrap();
    assert_eq!(outcome["outcome"], "rejected", "{outcome}");
    assert_eq!(outcome["origin"], origin, "{outcome}");
    assert_eq!(game.save_json().unwrap(), save);
    assert_eq!(game.snapshot(), before.snapshot());
    assert_eq!(archived(game), archived(&before));
    #[cfg(feature = "collector-metrics")]
    assert_eq!(game.withdrawal_collection_metrics(), metrics);
}

#[test]
fn overlapping_wide_owners_replace_release_and_roll_back_without_losing_shared_reasons() {
    let mut game = ReactiveSession::from_source(HIGH_DEGREE).unwrap();
    for event in ["create", "duplicate", "duplicate", "retire"] {
        send(&mut game, event, &BTreeMap::new());
    }
    let mut expected: Vec<String> = (0..16).map(|index| format!("s{index:02}@2")).collect();
    expected.push("zreason@2".into());
    assert_eq!(dynamic_retired(&game), expected);
    assert!(game.drain_archive().is_empty());
    let snapshot = serde_json::to_value(game.snapshot()).unwrap();
    let withdrawals = snapshot["withdrawals"].as_array().unwrap();
    assert_eq!(
        withdrawals.len(),
        17,
        "duplicate withdrawals retain the first record"
    );
    assert!(withdrawals
        .iter()
        .all(|entry| entry["sequence"] == 2 && entry["because"] == "zreason@2"));

    let mut restored =
        ReactiveSession::restore_json(HIGH_DEGREE, &game.save_json().unwrap()).unwrap();
    assert!(restored.drain_archive().is_empty());
    for event in ["same", "trim", "expand", "trim"] {
        paired(&mut game, &mut restored, event, &BTreeMap::new());
        assert_eq!(dynamic_retired(&game), expected);
    }
    for index in 1..15 {
        paired(
            &mut game,
            &mut restored,
            "clear",
            &BTreeMap::from([("index".into(), index as f64)]),
        );
        assert_eq!(
            dynamic_retired(&game),
            expected,
            "root15 still holds every subject"
        );
        assert_eq!(game.undrained(), 0);
    }
    for session in [&mut game, &mut restored] {
        rejected_without_change(session, "refuse", "{\"index\":15}", "policy");
        rejected_without_change(session, "fail", "{}", "evaluation");
    }
    paired(
        &mut game,
        &mut restored,
        "clear",
        &BTreeMap::from([("index".into(), 15.0)]),
    );
    let partial: Vec<String> = (1..16).map(|index| format!("s{index:02}@2")).collect();
    assert_eq!(record_names(&game.clone().drain_archive()), partial);
    assert_eq!(dynamic_retired(&game), ["s00@2", "zreason@2"]);
    paired(
        &mut game,
        &mut restored,
        "clear",
        &BTreeMap::from([("index".into(), 0.0)]),
    );
    assert_eq!(dynamic_retired(&game), ["s00@2", "zreason@2"]);
    assert_eq!(record_names(&game.clone().drain_archive()), partial);
    // Failed final collection must preserve the already-undrained first batch.
    for session in [&mut game, &mut restored] {
        rejected_without_change(session, "fail", "{}", "evaluation");
    }
    paired(&mut game, &mut restored, "release_single", &BTreeMap::new());
    assert!(dynamic_retired(&game).is_empty());
    let archive = game.drain_archive();
    let mut order = partial;
    order.extend(["s00@2".into(), "zreason@2".into()]);
    assert_eq!(
        record_names(&archive),
        order,
        "each event publishes a sorted batch"
    );
    assert_eq!(serde_json::to_value(&archive).unwrap(), archived(&restored));
    for entry in records(&archive) {
        assert_eq!(entry["withdrawal"]["because"], "zreason@2");
        assert_eq!(entry["withdrawal"]["sequence"], 2);
        assert_eq!(entry["withdrawal"]["event"], "duplicate");
    }
    let continued = ReactiveSession::restore_json(HIGH_DEGREE, &game.save_json().unwrap()).unwrap();
    assert_eq!(continued.snapshot(), game.snapshot());
}

#[test]
fn shared_reason_multiplicity_does_not_hide_a_scheduled_target_pin() {
    let source = format!(
        "{HIGH_DEGREE}\ncaveat later consequence material; event pin_reason; event tick dt min 0 max 0.1;\non pin_reason qualify zreason with later after 0.1;"
    );
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["create", "duplicate", "duplicate", "pin_reason", "retire"] {
        send(&mut game, event, &BTreeMap::new());
    }
    let mut restored = ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    paired(&mut game, &mut restored, "release", &BTreeMap::new());
    assert_eq!(dynamic_retired(&game), ["zreason@2"]);
    assert_eq!(
        record_names(&game.drain_archive()),
        (0..16)
            .map(|index| format!("s{index:02}@2"))
            .collect::<Vec<_>>()
    );
    restored.drain_archive();
    let snapshot = serde_json::to_value(game.snapshot()).unwrap();
    assert_eq!(
        snapshot["scheduled_qualifications"][0]["evidence"],
        "zreason@2"
    );
    assert_eq!(snapshot["withdrawals"].as_array().unwrap().len(), 1);
    assert_eq!(snapshot["withdrawals"][0]["because"], "zreason@2");
    paired(
        &mut game,
        &mut restored,
        "tick",
        &BTreeMap::from([("dt".into(), 0.1)]),
    );
    assert!(dynamic_retired(&game).is_empty());
    let archive = game.drain_archive();
    assert_eq!(record_names(&archive), ["zreason@2"]);
    assert_eq!(
        records(&archive)[0]["withdrawal"]["because"],
        json!("zreason@2")
    );
    assert_eq!(serde_json::to_value(&archive).unwrap(), archived(&restored));
    let continued = ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    assert_eq!(continued.snapshot(), game.snapshot());
}
