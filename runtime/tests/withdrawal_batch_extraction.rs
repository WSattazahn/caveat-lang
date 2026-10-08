//! Withdrawal handover preserves append order in the session and numeric
//! history order in each archive batch, including transactional retries.
use caveat_runtime::reactive::{ArchiveEntry, ArchiveItem, ReactiveSession, Withdrawal};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const INTERLEAVED: &str = r#"
claim seen;
evidence sensor from "sensor";
evidence reason from "correction";
readings a from sensor window 1;
readings z from sensor window 1;
readings m from sensor window 1;
state kept = 0; state released = 0;
event start; event plain;
event take_a hold min 0 max 2, mark min 0 max 1, value min 0 max 100;
event take_z hold min 0 max 2, mark min 0 max 1, value min 0 max 100;
event release_some; event release_all; event idle;
on start reveal reason supports seen;
on plain sample m = 0 supports seen;
on take_a sample a = value supports seen;
on take_a when hold == 1 set kept = kept + latest(a);
on take_a when hold == 2 set released = released + latest(a);
on take_a when mark == 1 withdraw latest(a) because reason;
on take_z sample z = value supports seen;
on take_z when hold == 1 set kept = kept + latest(z);
on take_z when hold == 2 set released = released + latest(z);
on take_z when mark == 1 withdraw latest(z) because reason;
on release_some set released = 0;
on release_all set kept = 0;
"#;

const MUTUAL: &str = r#"
claim seen;
evidence a from "a"; renewable a window 1;
evidence z from "z"; renewable z window 1;
evidence other from "a later correction";
state held = 0; state divisor = 1;
event cycle; event hold; event duplicate; event retire;
event release; event refuse; event fail;
on cycle reveal other supports seen;
on cycle renew a;
on cycle reveal a supports seen;
on cycle renew z;
on cycle reveal z supports seen;
# Withdrawal order deliberately opposes the archive's history order.
on cycle withdraw z because a;
on cycle withdraw a because z;
on hold set held = qualified(1, a);
on duplicate withdraw a because other;
on duplicate withdraw z because other;
on retire renew a;
on retire renew z;
on release set held = 0;
on refuse set held = 0;
on refuse reject "keep the root";
on fail set held = 0;
on fail set divisor = 0;
bind hud.guard = 1 / divisor;
"#;

fn send(game: &mut ReactiveSession, event: &str) {
    game.apply(event, &BTreeMap::new())
        .unwrap_or_else(|error| panic!("{event}: {error}"));
}

fn take(game: &mut ReactiveSession, event: &str, hold: u32, mark: u32, value: u32) {
    let payload = BTreeMap::from([
        ("hold".into(), f64::from(hold)),
        ("mark".into(), f64::from(mark)),
        ("value".into(), f64::from(value)),
    ]);
    game.apply(event, &payload)
        .unwrap_or_else(|error| panic!("{event}: {error}"));
}

fn records(archive: &[ArchiveItem]) -> Vec<&ArchiveEntry> {
    archive.iter().filter_map(ArchiveItem::record).collect()
}

fn names(archive: &[ArchiveItem]) -> Vec<&str> {
    records(archive)
        .into_iter()
        .map(|entry| entry.record.as_str())
        .collect()
}

fn pending(game: &ReactiveSession) -> Vec<ArchiveItem> {
    game.clone().drain_archive()
}

fn assert_withdrawals(game: &ReactiveSession, expected: &[Withdrawal]) {
    assert_eq!(game.snapshot().withdrawals, expected);
    let save: Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
    if expected.is_empty() {
        assert!(save.get("withdrawals").is_none());
    } else {
        assert_eq!(save["withdrawals"], json!(expected));
    }
}

fn assert_handover(archive: &[ArchiveItem], expected: &[&str], original: &[Withdrawal]) {
    assert_eq!(names(archive), expected);
    for entry in records(archive) {
        let (history, number) = entry.record.rsplit_once('@').unwrap();
        assert_eq!(entry.history, history);
        assert_eq!(entry.number, number.parse::<u64>().unwrap());
        // Compare the complete first record, not just its subject or reason.
        // Records that were never withdrawn must not take a neighbor's record.
        assert_eq!(
            entry.withdrawal.as_ref(),
            original
                .iter()
                .find(|record| record.evidence == entry.record),
            "{}",
            entry.record
        );
    }
}

fn assert_depart_effects(game: &ReactiveSession, expected: &[&str]) {
    let snapshot = serde_json::to_value(game.snapshot()).unwrap();
    let actual: Vec<_> = snapshot["effects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|effect| effect["kind"] == "depart")
        .map(|effect| effect["record"].as_str().unwrap())
        .collect();
    assert_eq!(actual, expected);
}

fn restore(source: &str, game: &ReactiveSession) -> ReactiveSession {
    let save = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(source, &save).unwrap();
    assert_eq!(restored.save_json().unwrap(), save);
    assert_eq!(restored.snapshot(), game.snapshot());
    assert_eq!(restored.undrained(), 0);
    restored
}

#[test]
fn interleaved_removal_preserves_survivor_order_and_numeric_archive_order() {
    let mut game = ReactiveSession::from_source(INTERLEAVED).unwrap();
    send(&mut game, "start");
    // Empty withdrawals, including a nonempty batch of unwithdrawn departures.
    send(&mut game, "plain");
    send(&mut game, "plain");
    assert_withdrawals(&game, &[]);
    assert_handover(&game.drain_archive(), &["m@1"], &[]);

    // Every old reading has an own-ground holder. The release set is interleaved
    // with survivors at the front, middle and end of the withdrawal vector.
    let trace = [
        ("take_a", "a@1", 1, 1),
        ("take_z", "z@1", 2, 1),
        ("take_a", "a@2", 2, 1),
        ("take_a", "a@3", 1, 1),
        ("take_z", "z@2", 1, 1),
        ("take_a", "a@4", 2, 0),
        ("take_a", "a@5", 2, 1),
        ("take_a", "a@6", 1, 1),
        ("take_a", "a@7", 2, 1),
        ("take_a", "a@8", 1, 1),
        ("take_a", "a@9", 2, 1),
        ("take_a", "a@10", 2, 1),
        ("take_a", "a@11", 1, 1),
        ("take_a", "a@12", 0, 0),
        ("take_z", "z@3", 0, 0),
    ];
    let mut original = Vec::new();
    for (index, &(event, evidence, hold, mark)) in trace.iter().enumerate() {
        take(&mut game, event, hold, mark, 1);
        if mark == 1 {
            original.push(Withdrawal {
                evidence: evidence.into(),
                because: "reason".into(),
                sequence: index as u64 + 4,
                event: event.into(),
            });
        }
        assert_withdrawals(&game, &original);
        assert_eq!(game.undrained(), 0);
    }
    assert_eq!(original.len(), 12);

    // A nonempty departure batch can have no matching withdrawal at all.
    send(&mut game, "plain");
    assert_withdrawals(&game, &original);
    assert_handover(&game.drain_archive(), &["m@2"], &original);
    let mut resumed = restore(INTERLEAVED, &game);

    for session in [&mut game, &mut resumed] {
        send(session, "release_some");
        let expected: Vec<_> = original
            .iter()
            .filter(|record| {
                ["a@1", "a@3", "z@2", "a@6", "a@8", "a@11"].contains(&record.evidence.as_str())
            })
            .cloned()
            .collect();
        assert_withdrawals(session, &expected);
        assert_eq!(
            expected
                .iter()
                .map(|record| record.evidence.as_str())
                .collect::<Vec<_>>(),
            ["a@1", "a@3", "z@2", "a@6", "a@8", "a@11"]
        );
        // a@4 has no withdrawal; a@10 sorts before a@2 lexically, but after it
        // numerically. Neither fact may change withdrawal-to-record association.
        let batch = ["a@2", "a@4", "a@5", "a@7", "a@9", "a@10", "z@1"];
        assert_handover(&pending(session), &batch, &original);
        assert_depart_effects(session, &batch);
    }
    assert_eq!(game.save_json().unwrap(), resumed.save_json().unwrap());
    assert_eq!(game.drain_archive(), resumed.drain_archive());

    // Restore after the partial extraction, then remove every remaining
    // withdrawal. The current, unwithdrawn readings remain live.
    let mut resumed = restore(INTERLEAVED, &game);
    for session in [&mut game, &mut resumed] {
        send(session, "release_all");
        assert_withdrawals(session, &[]);
        let batch = ["a@1", "a@3", "a@6", "a@8", "a@11", "z@2"];
        assert_handover(&pending(session), &batch, &original);
        assert_depart_effects(session, &batch);
    }
    assert_eq!(game.save_json().unwrap(), resumed.save_json().unwrap());
    assert_eq!(game.drain_archive(), resumed.drain_archive());
    let mut resumed = restore(INTERLEAVED, &game);
    for session in [&mut game, &mut resumed] {
        send(session, "idle");
        assert_withdrawals(session, &[]);
        assert_eq!(session.undrained(), 0);
    }
    assert_eq!(game.save_json().unwrap(), resumed.save_json().unwrap());
}

fn rejected_without_change(game: &mut ReactiveSession, event: &str, origin: &str, code: &str) {
    let save = game.save_json().unwrap();
    let snapshot = game.snapshot();
    let archive = pending(game);
    #[cfg(feature = "collector-metrics")]
    let metrics = game.withdrawal_collection_metrics();
    let outcome = serde_json::to_value(game.dispatch_outcome_json(event, "{}").unwrap()).unwrap();
    assert_eq!(outcome["outcome"], "rejected", "{outcome}");
    assert_eq!(outcome["origin"], origin, "{outcome}");
    assert_eq!(outcome["code"], code, "{outcome}");
    assert_eq!(game.save_json().unwrap(), save);
    assert_eq!(game.snapshot(), snapshot);
    assert_eq!(pending(game), archive);
    #[cfg(feature = "collector-metrics")]
    assert_eq!(game.withdrawal_collection_metrics(), metrics);
}

#[test]
fn mutual_handover_preserves_first_reasons_undrained_archive_and_failed_retry() {
    let mut game = ReactiveSession::from_source(MUTUAL).unwrap();
    let mut original = Vec::new();
    for event in [
        "cycle",
        "cycle",
        "hold",
        "duplicate",
        "duplicate",
        "cycle",
        "duplicate",
        "retire",
    ] {
        let previous = game.snapshot().withdrawals;
        send(&mut game, event);
        if event == "cycle" {
            for record in game.snapshot().withdrawals {
                if !original
                    .iter()
                    .any(|old: &Withdrawal| old.evidence == record.evidence)
                {
                    original.push(record);
                }
            }
        } else if event == "duplicate" {
            assert_withdrawals(&game, &previous);
        }
    }
    let remaining = vec![
        Withdrawal {
            evidence: "z@3".into(),
            because: "a@3".into(),
            sequence: 2,
            event: "cycle".into(),
        },
        Withdrawal {
            evidence: "a@3".into(),
            because: "z@3".into(),
            sequence: 2,
            event: "cycle".into(),
        },
    ];
    assert_withdrawals(&game, &remaining);
    let prefix = pending(&game);
    assert_handover(&prefix, &["a@2", "z@2", "a@4", "z@4"], &original);
    let mut resumed = restore(MUTUAL, &game);
    let mut uninterrupted = game.clone();
    for session in [&mut game, &mut resumed] {
        rejected_without_change(session, "refuse", "policy", "reject");
        // A final binding rejects after extraction and archive construction.
        // Repeated rejection must restore the vector, reason pins and queue.
        rejected_without_change(session, "fail", "evaluation", "expression");
        rejected_without_change(session, "fail", "evaluation", "expression");
        assert_withdrawals(session, &remaining);
        send(session, "release");
        assert_withdrawals(session, &[]);
        assert_depart_effects(session, &["a@3", "z@3"]);
    }
    send(&mut uninterrupted, "release");
    assert_eq!(game.save_json().unwrap(), resumed.save_json().unwrap());
    assert_eq!(
        game.save_json().unwrap(),
        uninterrupted.save_json().unwrap()
    );
    let archive = game.drain_archive();
    assert_eq!(archive, uninterrupted.drain_archive());
    assert_eq!(&archive[..prefix.len()], prefix);
    assert_eq!(&archive[prefix.len()..], resumed.drain_archive());
    assert_handover(
        &archive,
        &["a@2", "z@2", "a@4", "z@4", "a@3", "z@3"],
        &original,
    );
    for entry in records(&archive) {
        let withdrawal = entry.withdrawal.as_ref().unwrap();
        let counterpart = records(&archive)
            .into_iter()
            .find(|other| other.record == withdrawal.because)
            .unwrap();
        assert_eq!(
            counterpart.withdrawal.as_ref().unwrap().because,
            entry.record
        );
        assert!(entry.relations.contains(&[
            "withdrawn".into(),
            "qualifies".into(),
            entry.record.clone(),
        ]));
    }
    restore(MUTUAL, &game);
}
