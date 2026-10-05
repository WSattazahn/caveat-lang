//! Retired records that nothing pins depart, and a save keeps the split
//! between a provenance's own reads and its inherited names
//! (spec/caveat-departure-0.1.md, spec/caveat-lineage-compaction-0.1.md).
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Stream `b` samples only while `latest(a)` is small, so a skipped sample
/// leaves `a`'s current reading in `b`'s selection qualifications. `copy`
/// inherits it through `latest(b)`, and `echo` through `copy`.
const TWO_STREAMS: &str = r#"
claim seen;
evidence glimpse from "a glimpse";
readings a from glimpse window 2;
readings b from glimpse window 2;
state copy = 0;
state echo = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event take; event pass;
on la sample a = v supports seen;
on lb when latest(a) < 5 sample b = v supports seen;
on take set copy = latest(b) because latest(b);
on pass set echo = copy because copy;
"#;

fn send(game: &mut ReactiveSession, event: &str, v: Option<f64>) -> Value {
    let payload = v
        .map(|v| BTreeMap::from([("v".to_string(), v)]))
        .unwrap_or_default();
    game.apply(event, &payload)
        .unwrap_or_else(|error| panic!("{event}: {error}"));
    serde_json::to_value(game.snapshot()).unwrap()
}

/// The records each event departed, in order.
fn departures(game: &mut ReactiveSession) -> Vec<Vec<String>> {
    [
        ("pass", None),
        ("la", Some(1.0)),
        ("la", Some(1.0)),
        ("la", Some(1.0)),
    ]
    .into_iter()
    .map(|(event, v)| {
        send(game, event, v)["effects"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|effect| effect["kind"] == "depart")
            .map(|effect| effect["record"].as_str().unwrap().to_string())
            .collect()
    })
    .collect()
}

fn inheriting() -> ReactiveSession {
    let mut game = ReactiveSession::from_source(TWO_STREAMS).unwrap();
    send(&mut game, "la", Some(1.0));
    send(&mut game, "lb", Some(3.0));
    send(&mut game, "la", Some(7.0));
    send(&mut game, "lb", Some(4.0));
    send(&mut game, "take", None);
    game
}

// The regression for the split travelling through reads: `copy`'s lineage
// inherited a@2, and a restore that read it as an own read would pin a@2 in
// `echo`'s grounds at the next `set`. Restored, the session departs exactly
// what the original departs.
#[test]
fn a_restored_lineage_keeps_what_it_inherited_and_departs_the_same_records() {
    let mut original = inheriting();
    let save: Value = serde_json::from_str(&original.save_json().unwrap()).unwrap();
    assert_eq!(
        save["states"]["copy"]["lineage"]["inherited"],
        json!(["a@2"]),
        "{save}"
    );
    let mut restored = ReactiveSession::restore_json(TWO_STREAMS, &save.to_string()).unwrap();
    let departed = departures(&mut original);
    assert_eq!(
        departed,
        [
            vec![],
            vec![],
            vec!["a@2".to_string()],
            vec!["a@3".to_string()]
        ]
    );
    assert_eq!(departures(&mut restored), departed);
    assert_eq!(restored.save_json().unwrap(), original.save_json().unwrap());
}

// The same save as rc.15 would have written it, with no `inherited` field:
// it restores, and keeps a@2 because it reads every name as an own read. The
// excess names only a record the save held; a@3, sampled after the restore,
// departs as in the original.
#[test]
fn a_save_without_inherited_restores_and_over_pins_only_records_it_held() {
    let save = inheriting().save_json().unwrap();
    let rc15 = save.replace(",\"inherited\":[\"a@2\"]", "");
    assert!(!rc15.contains("inherited"));
    let mut restored = ReactiveSession::restore_json(TWO_STREAMS, &rc15).unwrap();
    assert_eq!(
        departures(&mut restored),
        [vec![], vec![], vec![], vec!["a@3".to_string()]]
    );
    let later: Value = serde_json::from_str(&restored.save_json().unwrap()).unwrap();
    assert_eq!(later["retired"], json!({"a@1": 7, "a@2": 8}), "{later}");
    assert!(rc15.contains("\"a@2\""), "a@2 is a record the save held");
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert!(error.contains(expected), "{error}"),
        Ok(_) => panic!("accepted: {expected}"),
    }
}

// The 13 holders are every provenance the save schema has, so `inherited`
// anywhere else is an unknown field (spec/caveat-lineage-compaction-0.1.md).
#[test]
fn inherited_on_a_journal_entry_is_an_unknown_field() {
    let source = format!(
        "{TWO_STREAMS}\ndecisions trust limit 4;\nevent decide;\n\
         on decide commit trust because enough using latest(b);"
    );
    let mut game = ReactiveSession::from_source(&source).unwrap();
    send(&mut game, "la", Some(1.0));
    send(&mut game, "lb", Some(3.0));
    send(&mut game, "decide", None);
    let mut save: Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
    assert!(ReactiveSession::restore_json(&source, &save.to_string()).is_ok());
    save["decision_journal"][0]["inherited"] = json!(["b@1"]);
    refused(&source, &save, "unknown field `inherited`");
}

// Without a window nothing departs and no split is kept, so a save that
// records one is refused, though the field is known.
#[test]
fn inherited_in_a_program_without_a_window_is_refused() {
    let source = TWO_STREAMS.replace("window 2", "limit 8");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    send(&mut game, "la", Some(1.0));
    send(&mut game, "lb", Some(3.0));
    send(&mut game, "la", Some(7.0));
    send(&mut game, "lb", Some(4.0));
    send(&mut game, "take", None);
    let mut save: Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
    assert!(!save.to_string().contains("inherited"), "{save}");
    save["states"]["copy"]["lineage"]["inherited"] = json!(["a@2"]);
    refused(&source, &save, "cannot record inherited names");
}

#[test]
fn an_inherited_name_must_be_one_the_provenance_holds() {
    let mut save: Value = serde_json::from_str(&inheriting().save_json().unwrap()).unwrap();
    save["states"]["copy"]["lineage"]["inherited"] = json!(["a@9"]);
    refused(TWO_STREAMS, &save, "which it does not hold");
}

/// The windows fixture's departure of sighting@1 into a marker on every
/// revision basis that read it.
const WORLD: &str = r#"
claim seen;
evidence glimpse from "a glimpse";
readings sighting from glimpse window 3;
decisions trust limit 64;
journal window 3;
event look v min 0 max 10;
event decide;
on look sample sighting = v supports seen;
on decide when committed(trust) and not reopened(trust) reopen trust because latest(sighting);
on decide commit trust because enough using history_count(sighting);
"#;

fn marked() -> (ReactiveSession, Value) {
    let mut game = ReactiveSession::from_source(WORLD).unwrap();
    for v in 0..6 {
        send(&mut game, "look", Some(f64::from(v)));
        send(&mut game, "decide", None);
    }
    let save = serde_json::from_str(&game.save_json().unwrap()).unwrap();
    (game, save)
}

#[test]
fn departed_readings_become_one_marker_per_history() {
    let (game, save) = marked();
    let snapshot = serde_json::to_value(game.snapshot()).unwrap();
    assert_eq!(
        snapshot["commitment_bases"]["trust@6"]["provenance"]["departed"],
        json!([{"history": "sighting", "read": 2, "from": 1, "through": 2, "departed_at": 12}])
    );
    let restored = ReactiveSession::restore_json(WORLD, &save.to_string()).unwrap();
    assert_eq!(restored.snapshot(), game.snapshot());
    let again: Value = serde_json::from_str(&restored.save_json().unwrap()).unwrap();
    assert_eq!(again, save);
}

#[test]
fn a_forged_marker_is_refused() {
    let (_, save) = marked();
    let edit = |change: &dyn Fn(&mut Value)| {
        let mut forged = save.clone();
        change(&mut forged["commitment_bases"]["trust@6"]["provenance"]["departed"][0]);
        forged
    };
    for (forged, why) in [
        (
            edit(&|m| m["departed_at"] = json!(0)),
            "departed out of sequence",
        ),
        (
            edit(&|m| m["departed_at"] = json!(10_000)),
            "departed out of sequence",
        ),
        (
            edit(&|m| m["from"] = json!(3)),
            "is not of departed records",
        ),
        (
            edit(&|m| m["through"] = json!(3)),
            "is not of departed records",
        ),
        (
            edit(&|m| m["read"] = json!(3)),
            "counts none, or more records",
        ),
        (
            edit(&|m| m["read"] = json!(0)),
            "counts none, or more records",
        ),
        (
            edit(&|m| m["history"] = json!("trust")),
            "which has no window",
        ),
    ] {
        refused(WORLD, &forged, why);
    }
}

#[test]
fn the_archive_hands_each_departed_record_over_once() {
    let mut game = ReactiveSession::from_source(WORLD).unwrap();
    for v in [1.0, 2.0, 3.0, 4.0] {
        send(&mut game, "look", Some(v));
    }
    let archive = serde_json::to_value(game.drain_archive()).unwrap();
    assert_eq!(archive.as_array().unwrap().len(), 1, "{archive}");
    assert_eq!(archive[0]["record"], "sighting@1");
    assert_eq!(archive[0]["retired_at"], 4);
    assert_eq!(archive[0]["departed_at"], 4);
    assert_eq!(archive[0]["reading"]["value"], 1.0);
    assert!(game.drain_archive().is_empty(), "drained once");
    // The archive is not saved, so a restored session starts with none.
    let mut restored = ReactiveSession::restore_json(WORLD, &game.save_json().unwrap()).unwrap();
    assert!(restored.drain_archive().is_empty());
}

#[test]
fn journal_departed_needs_a_journal_window() {
    let (_, mut save) = marked();
    assert_eq!(save["journal_departed"], 8, "{save}");
    let source = WORLD.replace("journal window 3;\n", "");
    save["journal_departed"] = json!(8);
    assert!(ReactiveSession::restore_json(&source, &save.to_string()).is_err());
}
