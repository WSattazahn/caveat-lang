//! Synthesized source-possible older-save boundary, not an authenticated run.
//! Uses the public validator and dispatch at the real 65,536-record threshold.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[test]
#[ignore = "large real-capacity contract gate; run explicitly in release mode"]
fn capacity_refusal_is_preappend_and_atomic_then_prior_collection_frees_room() {
    const HELD_LIMIT: usize = 65_536;
    let fixtures: Value =
        serde_json::from_str(include_str!("fixtures/collector-baseline-f5ec829.json")).unwrap();
    let case = &fixtures["cases"][0];
    let source = case["source"].as_str().unwrap();
    let mut save: Value = serde_json::from_str(case["save"].as_str().unwrap()).unwrap();
    let mut nodes = Vec::new();
    let mut relations = Vec::new();
    let mut withdrawals = Vec::new();
    let mut renewals = vec![json!("a")];
    let mut qualifications = serde_json::Map::new();
    let mut retired = serde_json::Map::new();
    retired.insert("a".into(), json!(1));
    for n in 2..=HELD_LIMIT {
        let record = format!("a@{n}");
        nodes.push(json!({"kind":"occurrence","name":record}));
        relations.push(json!([record, "supports", "seen"]));
        relations.push(json!(["withdrawn", "qualifies", record]));
        withdrawals
            .push(json!({"evidence":record,"because":record,"sequence":n-1,"event":"cycle"}));
        renewals.push(json!(record));
        qualifications.insert(record.clone(), json!({}));
        if n < HELD_LIMIT {
            retired.insert(record, json!(n));
        }
    }
    save["sequence"] = json!(HELD_LIMIT - 1);
    save["last_event"] = json!("collector_noop");
    save["effects"] = json!([]);
    save["graph"] = json!({"nodes":nodes,"relations":relations});
    save["renewals"]["a"] = json!(renewals);
    save["withdrawals"] = json!(withdrawals);
    save["observation_qualifications"] = Value::Object(qualifications);
    save["retired"] = Value::Object(retired);
    let saved = save.to_string();
    drop(save);
    eprintln!(
        "synthesized valid-input candidate: held={HELD_LIMIT}, bytes={}",
        saved.len()
    );
    assert!(
        saved.len() < 16 * 1024 * 1024,
        "ordinary save input byte limit unchanged"
    );
    let mut game = ReactiveSession::restore_json(source, &saved).unwrap();
    drop(saved);
    assert!(game.drain_archive().is_empty(), "restore never collects");
    let before = game.save_json().unwrap();
    let outcome = serde_json::to_value(game.dispatch_outcome_json("cycle", "{}").unwrap()).unwrap();
    assert_eq!(outcome["outcome"], "rejected", "{outcome}");
    assert_eq!(outcome["origin"], "limit");
    assert_eq!(outcome["code"], "renewal_limit");
    assert!(outcome["message"]
        .as_str()
        .unwrap()
        .contains("65536 records"));
    assert_eq!(game.save_json().unwrap(), before);
    drop(before);
    assert!(game.drain_archive().is_empty());
    // An accepted no-op can collect established unreachable groups. It does
    // not rescue the refused allocation; only a later retry has capacity.
    game.apply("collector_noop", &BTreeMap::new()).unwrap();
    let departed = game.drain_archive();
    assert_eq!(
        departed
            .iter()
            .filter(|item| item.record().is_some())
            .count(),
        HELD_LIMIT - 2
    );
    #[cfg(feature = "collector-metrics")]
    assert_eq!(
        game.withdrawal_collection_metrics().collected,
        HELD_LIMIT - 2
    );
    drop(departed);
    game.apply("cycle", &BTreeMap::new()).unwrap();
    assert_eq!(game.snapshot().sequence, HELD_LIMIT as u64 + 1);
    ReactiveSession::restore_json(source, &game.save_json().unwrap()).unwrap();
}
