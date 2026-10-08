//! Retired records that nothing pins depart, and a save keeps the split
//! between a provenance's own reads and its inherited names
//! (spec/caveat-departure-0.1.md, spec/caveat-lineage-compaction-0.1.md).
use caveat_runtime::reactive::{ArchiveEntry, ArchiveItem, ReactiveSession};
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

fn records(archive: Vec<ArchiveItem>) -> Vec<ArchiveEntry> {
    archive
        .into_iter()
        .filter_map(|item| match item {
            ArchiveItem::Record(entry) => Some(*entry),
            ArchiveItem::Provenance(_) => None,
        })
        .collect()
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
    let mut marker = snapshot["commitment_bases"]["trust@6"]["provenance"]["departed"][0].clone();
    assert_eq!(
        marker
            .as_object_mut()
            .unwrap()
            .remove("archive_ref")
            .unwrap()
            .as_str()
            .unwrap()
            .len(),
        64
    );
    assert_eq!(
        marker,
        json!({"history": "sighting", "read": 2, "from": 1, "through": 2, "departed_at": 12})
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
    assert_eq!(archive.as_array().unwrap().len(), 2, "{archive}");
    assert_eq!(archive[1]["operation"], "record");
    assert_eq!(archive[1]["record"], "sighting@1");
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

const WITHDRAWAL_CASCADE: &str = r#"
claim seen;
evidence a_reason from "correction";
renewable a_reason window 1;
evidence glimpse from "a glimpse";
readings z_subject from glimpse window 1;
event reason; event look; event retract;
on reason renew a_reason;
on reason reveal a_reason supports seen;
on look sample z_subject = 1 supports seen;
on retract withdraw latest(z_subject) because a_reason;
"#;

#[test]
fn a_withdrawal_releases_its_retired_reason_in_the_same_event() {
    let mut game = ReactiveSession::from_source(WITHDRAWAL_CASCADE).unwrap();
    for event in ["reason", "look", "retract", "reason"] {
        send(&mut game, event, None);
    }
    let before = game.save_json().unwrap();
    let mut restored = ReactiveSession::restore_json(WITHDRAWAL_CASCADE, &before).unwrap();
    let shot = send(&mut game, "look", None);
    send(&mut restored, "look", None);
    let saved = game.save_json().unwrap();
    ReactiveSession::restore_json(WITHDRAWAL_CASCADE, &saved)
        .unwrap_or_else(|error| panic!("immediate save must restore: {error}"));
    assert_eq!(restored.save_json().unwrap(), saved);
    assert_eq!(shot["retired"], json!({"a_reason": 1}));
    assert!(game.snapshot().withdrawals.is_empty());
    let departed = shot["effects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|effect| effect["kind"] == "depart")
        .map(|effect| effect["record"].clone())
        .collect::<Vec<_>>();
    assert_eq!(departed, vec![json!("a_reason@2"), json!("z_subject@1")]);
    let archive = records(game.drain_archive());
    assert_eq!(
        archive
            .iter()
            .map(|entry| entry.record.as_str())
            .collect::<Vec<_>>(),
        ["a_reason@2", "z_subject@1"]
    );
    assert_eq!(
        archive[1].withdrawal.as_ref().unwrap().because,
        "a_reason@2"
    );
}

#[test]
fn shared_state_pins_release_only_after_the_last_holder_and_roll_back_on_refusal() {
    let source = r#"
claim seen; evidence glimpse from "glimpse";
readings sighting from glimpse window 1;
state a = 0; state b = 0;
event look; event take; event release_a; event release_b; event refused;
on look sample sighting = 1 supports seen;
on take set a = latest(sighting);
on take set b = latest(sighting);
on release_a set a = 0;
on release_b set b = 0;
on refused set b = 0;
on refused reject "keep it";
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    for event in ["look", "take", "look", "release_a"] {
        send(&mut game, event, None);
    }
    assert!(game.snapshot().retired.contains_key("sighting@1"));
    let before = game.save_json().unwrap();
    assert!(game.apply("refused", &BTreeMap::new()).is_err());
    assert_eq!(game.save_json().unwrap(), before);
    assert!(game.drain_archive().is_empty());
    let mut restored = ReactiveSession::restore_json(source, &before).unwrap();
    send(&mut game, "release_b", None);
    send(&mut restored, "release_b", None);
    assert_eq!(game.save_json().unwrap(), restored.save_json().unwrap());
    assert!(!game.snapshot().retired.contains_key("sighting@1"));
    assert_eq!(records(game.drain_archive())[0].record, "sighting@1");
}

#[test]
fn each_scheduled_target_pins_until_its_own_qualification_applies() {
    let source = r#"
claim seen; evidence reason from "reason";
renewable reason window 1;
caveat stale consequence material;
event renew; event later; event tick dt min 0 max 0.1;
on renew renew reason;
on renew reveal reason supports seen;
on later qualify reason with stale after 0.1;
on later qualify reason with stale after 0.2;
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    for event in ["renew", "later", "renew"] {
        send(&mut game, event, None);
    }
    let mut restored = ReactiveSession::restore_json(source, &game.save_json().unwrap()).unwrap();
    let tick = BTreeMap::from([("dt".into(), 0.1)]);
    for session in [&mut game, &mut restored] {
        session.apply("tick", &tick).unwrap();
        assert!(session.snapshot().retired.contains_key("reason@2"));
        session.apply("tick", &tick).unwrap();
        assert!(!session.snapshot().retired.contains_key("reason@2"));
        assert_eq!(records(session.drain_archive())[0].record, "reason@2");
        ReactiveSession::restore_json(source, &session.save_json().unwrap()).unwrap();
    }
    assert_eq!(game.save_json().unwrap(), restored.save_json().unwrap());
}

#[test]
fn permission_grants_remain_pinned_after_a_revision_is_superseded() {
    let source = r#"
claim seen; evidence glimpse from "glimpse";
readings grants from glimpse window 1;
decisions trust limit 4; journal window 1;
event look; event decide;
on look sample grants = 1 supports seen;
on decide when committed(trust) and not reopened(trust) reopen trust because latest(grants);
on decide commit trust because enough permitted by latest(grants);
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    for event in ["look", "decide", "look", "decide", "look"] {
        send(&mut game, event, None);
    }
    assert_eq!(
        game.snapshot()
            .retired
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["grants@1", "grants@2"]
    );
    let mut restored = ReactiveSession::restore_json(source, &game.save_json().unwrap()).unwrap();
    send(&mut game, "decide", None);
    send(&mut restored, "decide", None);
    assert_eq!(game.save_json().unwrap(), restored.save_json().unwrap());
}

#[test]
fn a_binding_failure_rolls_back_the_whole_departure_cascade_and_archive() {
    let source = format!(
        "{WITHDRAWAL_CASCADE}\nstate divisor = 1;\nevent fail;\n\
        on fail sample z_subject = 1 supports seen;\non fail set divisor = 0;\n\
        bind hud.value = 1 / divisor;"
    );
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["reason", "look", "retract", "reason"] {
        send(&mut game, event, None);
    }
    let before = game.save_json().unwrap();
    assert!(game.apply("fail", &BTreeMap::new()).is_err());
    assert_eq!(game.save_json().unwrap(), before);
    assert!(game.drain_archive().is_empty());
    send(&mut game, "look", None);
    assert_eq!(records(game.drain_archive()).len(), 2);
    ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
}

const SAME_EVENT_DEPARTURE: &str = r#"
claim seen; evidence glimpse from "glimpse";
evidence reason from "correction"; renewable reason window 1;
caveat stale consequence material; caveat unrelated consequence low;
readings readings from glimpse window 1;
event burst;
on burst sample readings = 1 supports seen;
on burst renew reason;
on burst reveal reason;
on burst qualify reason with stale;
on burst withdraw latest(readings) because reason;
on burst renew reason;
on burst reveal reason supports seen;
on burst sample readings = 2 supports seen;
"#;

#[test]
fn effects_can_report_records_that_depart_later_in_the_same_event() {
    let mut game = ReactiveSession::from_source(SAME_EVENT_DEPARTURE).unwrap();
    send(&mut game, "burst", None);
    let saved = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(SAME_EVENT_DEPARTURE, &saved).unwrap();
    assert_eq!(game.snapshot(), restored.snapshot());
    assert_eq!(records(game.drain_archive()).len(), 2);
    let save: Value = serde_json::from_str(&saved).unwrap();
    let mut forged = save.clone();
    forged["effects"]
        .as_array_mut()
        .unwrap()
        .retain(|effect| !(effect["kind"] == "depart" && effect["record"] == "reason@2"));
    refused(SAME_EVENT_DEPARTURE, &forged, "unknown");
    let mut forged = save.clone();
    forged["effects"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|effect| effect["kind"] == "qualify")
        .unwrap()["evidence"] = json!("reason@99");
    refused(SAME_EVENT_DEPARTURE, &forged, "unknown");
    let mut forged = save;
    forged["effects"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|effect| effect["kind"] == "qualify")
        .unwrap()["caveat"] = json!("unrelated");
    refused(
        SAME_EVENT_DEPARTURE,
        &forged,
        "not one the last event can make",
    );
}

#[test]
fn a_reopening_report_keeps_a_cause_that_departs_later_in_the_event() {
    let source = r#"
claim seen; evidence glimpse from "glimpse";
readings readings from glimpse window 1;
decisions trust limit 4; journal window 1;
event decide;
on decide sample readings = 1 supports seen;
on decide commit trust because enough;
on decide reopen trust because latest(readings);
on decide commit trust because enough;
on decide sample readings = 2 supports seen;
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    send(&mut game, "decide", None);
    assert!(!game.snapshot().retired.contains_key("readings@1"));
    let restored = ReactiveSession::restore_json(source, &game.save_json().unwrap()).unwrap();
    assert_eq!(restored.snapshot(), game.snapshot());
}

// left read {s@1,s@2,s@4}, right read {s@1,s@3,s@4}. Range/count markers
// alone cannot distinguish these copies or a reused holder's replacement.
#[test]
fn archive_roots_reconstruct_post_departure_copies_replacements_and_overlap() {
    let source = r#"
claim seen; evidence glimpse from "glimpse";
readings s from glimpse window 1;
state left = 0; state right = 0; state chosen = 0; state reused = 0;
event look v min 1 max 5; event pick side min 0 max 1;
on look sample s = v supports seen;
on look when v < 5 and v != 3 and latest(s) > 0 set left = left;
on look when v < 5 and v != 2 and latest(s) > 0 set right = right;
on look when v < 5 and v != 3 and latest(s) > 0 set reused = reused;
on pick when side == 0 set chosen = left;
on pick when side == 0 set reused = left;
on pick when side == 1 set chosen = right;
on pick when side == 1 set reused = right;
bind hud.chosen = chosen;
bind hud.union = left + right;
bind hud.overlap = left + reused;
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    for value in 1..=5 {
        send(&mut game, "look", Some(f64::from(value)));
    }
    let mut full_archive = game.drain_archive();
    let archive = records(full_archive.clone());
    for (name, expected) in [
        ("left", vec!["s@1", "s@2", "s@4"]),
        ("right", vec!["s@1", "s@3", "s@4"]),
        ("reused", vec!["s@1", "s@2", "s@4"]),
    ] {
        let held = archive
            .iter()
            .filter(|entry| {
                entry.holders.iter().any(|holder| {
                    holder.kind == "state" && holder.name == name && holder.field == "lineage"
                })
            })
            .map(|entry| entry.record.as_str())
            .collect::<Vec<_>>();
        assert_eq!(held, expected);
    }
    assert!(archive
        .iter()
        .all(|entry| entry.holders.iter().all(|holder| holder.name != "chosen")));
    let mut other = game.clone();
    game.apply("pick", &BTreeMap::from([("side".into(), 0.0)]))
        .unwrap();
    other
        .apply("pick", &BTreeMap::from([("side".into(), 1.0)]))
        .unwrap();
    assert_ne!(game.save_json().unwrap(), other.save_json().unwrap());
    full_archive.extend(game.drain_archive());
    full_archive.extend(other.drain_archive());
    let left = serde_json::to_value(game.snapshot()).unwrap();
    let right = serde_json::to_value(other.snapshot()).unwrap();
    for (shot, exact) in [
        (&left, vec!["s@1", "s@2", "s@4"]),
        (&right, vec!["s@1", "s@3", "s@4"]),
    ] {
        for state in ["chosen", "reused"] {
            let marker = &shot["qualified_values"][state]["provenance"]["departed"][0];
            assert_eq!(resolve_archive(&full_archive, marker), exact);
        }
        assert_eq!(
            resolve_archive(
                &full_archive,
                &shot["binding_qualifications"]["hud"]["union"]["departed"][0]
            ),
            ["s@1", "s@2", "s@3", "s@4"]
        );
    }
    assert_eq!(
        resolve_archive(
            &full_archive,
            &left["binding_qualifications"]["hud"]["overlap"]["departed"][0]
        ),
        ["s@1", "s@2", "s@4"]
    );
    assert_eq!(
        resolve_archive(
            &full_archive,
            &right["binding_qualifications"]["hud"]["overlap"]["departed"][0]
        ),
        ["s@1", "s@2", "s@3", "s@4"]
    );
    let mut restored = ReactiveSession::restore_json(source, &other.save_json().unwrap()).unwrap();
    assert!(restored.drain_archive().is_empty());
    assert_eq!(restored.snapshot(), other.snapshot());
    restored
        .apply("pick", &BTreeMap::from([("side".into(), 0.0)]))
        .unwrap();
    full_archive.extend(restored.drain_archive());
    let resumed = serde_json::to_value(restored.snapshot()).unwrap();
    assert_eq!(
        resolve_archive(
            &full_archive,
            &resumed["binding_qualifications"]["hud"]["chosen"]["departed"][0]
        ),
        ["s@1", "s@2", "s@4"]
    );
    let chosen = &game.snapshot().qualified_values["chosen"]
        .provenance
        .departed["s"];
    assert_eq!((chosen.from, chosen.through, chosen.read), (1, 4, 3));
    assert!(game.drain_archive().is_empty());
    assert!(other.drain_archive().is_empty());
}

/// The runtime regression checks membership; the kit additionally verifies
/// hashes, conflicts, cycles, scope and incomplete archives before disclosure.
fn resolve_archive(archive: &[ArchiveItem], marker: &Value) -> Vec<String> {
    let json = serde_json::to_value(archive).unwrap();
    let mut todo = vec![marker["archive_ref"]
        .as_str()
        .expect("marker has a root")
        .to_string()];
    let nodes = json
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["kind"] == "provenance")
        .map(|entry| (entry["id"].as_str().unwrap(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut seen = std::collections::BTreeSet::new();
    let mut exact = std::collections::BTreeSet::new();
    while let Some(id) = todo.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        let entry = nodes
            .get(id.as_str())
            .unwrap_or_else(|| panic!("missing node {id}"));
        if entry["operation"] == "record" {
            let record = entry["record"].as_str().unwrap();
            assert!(archive
                .iter()
                .filter_map(ArchiveItem::record)
                .any(|item| item.record == record));
            exact.insert(record.to_string());
        } else {
            todo.extend(
                entry["parents"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|parent| parent.as_str().unwrap().to_string()),
            );
        }
    }
    exact.into_iter().collect()
}

#[test]
fn transfer_metadata_is_transactional_and_read_queries_do_not_publish_it() {
    let source = r#"
claim seen; evidence glimpse from "glimpse";
readings s from glimpse window 1;
state accumulated = 0; state copied = 0; state divisor = 1;
event look; event refuse; event fail;
on look sample s = 1 supports seen;
on look when latest(s) > 0 set accumulated = accumulated;
on refuse set copied = accumulated;
on refuse sample s = 2 supports seen;
on refuse reject "rollback";
on fail set copied = accumulated;
on fail sample s = 2 supports seen;
on fail set divisor = 0;
bind hud.accumulated = accumulated;
bind hud.check = 1 / divisor;
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    for _ in 0..8 {
        send(&mut game, "look", None);
    }
    let mut archive = game.drain_archive();
    let before = game.save_json().unwrap();
    for event in ["refuse", "fail"] {
        assert!(game.apply(event, &BTreeMap::new()).is_err());
        assert_eq!(game.save_json().unwrap(), before);
        assert!(game.drain_archive().is_empty());
    }
    for _ in 0..3 {
        let _ = game.snapshot();
        assert_eq!(game.save_json().unwrap(), before);
        assert_eq!(game.undrained(), 0);
    }
    let restored = ReactiveSession::restore_json(source, &before).unwrap();
    assert_eq!(restored.snapshot(), game.snapshot());
    for _ in 0..16 {
        send(&mut game, "look", None);
        archive.extend(game.drain_archive());
    }
    let shot = serde_json::to_value(game.snapshot()).unwrap();
    let marker = &shot["qualified_values"]["accumulated"]["provenance"]["departed"][0];
    assert_eq!(marker["archive_ref"].as_str().unwrap().len(), 64);
    assert_eq!(
        marker.as_object().unwrap().len(),
        6,
        "fixed marker shape, no DAG descendants in save"
    );
    assert_eq!(resolve_archive(&archive, marker).len(), 23);
}

#[test]
fn legacy_unknown_marker_membership_stays_unknown_after_copy_and_merge() {
    let (game, mut save) = marked();
    fn strip(value: &mut Value) {
        match value {
            Value::Object(fields) => {
                fields.remove("archive_ref");
                for value in fields.values_mut() {
                    strip(value);
                }
            }
            Value::Array(values) => {
                for value in values {
                    strip(value);
                }
            }
            _ => {}
        }
    }
    strip(&mut save);
    let mut restored = ReactiveSession::restore_json(WORLD, &save.to_string()).unwrap();
    assert!(restored.drain_archive().is_empty());
    send(&mut restored, "look", Some(8.0));
    send(&mut restored, "decide", None);
    let shot = serde_json::to_value(restored.snapshot()).unwrap();
    // A new commitment inherits the prior unknown basis. Exactness cannot be
    // manufactured from an ambiguous legacy range, even with later departures.
    assert!(
        shot["commitment_bases"]["trust@7"]["provenance"]["departed"][0]["archive_ref"].is_null()
    );
    assert_ne!(game.save_json().unwrap(), restored.save_json().unwrap());
}

#[test]
fn a_departed_journal_entry_cannot_masquerade_as_effect_evidence() {
    let source = r#"
claim seen; evidence glimpse from "glimpse";
evidence journal from "separate source evidence";
caveat stale consequence material;
readings s from glimpse window 1;
decisions trust limit 4; journal window 1;
event decide;
on decide sample s = 1 supports seen;
on decide commit trust because enough;
on decide reopen trust because latest(s);
on decide commit trust because enough;
on decide reveal journal supports seen;
on decide when 0 == 1 qualify journal with stale;
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    send(&mut game, "decide", None);
    let mut forged: Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
    forged["effects"].as_array_mut().unwrap().push(json!({
        "kind":"qualify", "evidence":"journal@1", "caveat":"stale"
    }));
    assert!(
        ReactiveSession::restore_json(source, &forged.to_string()).is_err(),
        "a report of a departed journal record is not a missing evidence node"
    );
}
