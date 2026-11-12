//! D060: restore held a retirement's date only to the save's sequence and
//! to the retired record's own, so it accepted a retirement dated at an
//! event that added nothing to the record's history. The finding's save
//! dated the retirement of renewable evidence at a last event that renewed
//! nothing. A record retires as its history adds the record that takes its
//! place in the window, so restore now holds the date to what else the save
//! dates: the last event's retire effects, and the sequence of the reading
//! or journal entry that took the retired one's place. See
//! spec/caveat-windows-0.1.md ("Save and restore").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// The outcome of `event`, accepted or refused, as text.
fn send(session: &mut ReactiveSession, event: &str, payload: &str) -> String {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_string(&outcome).unwrap()
}

/// Plays `events`. After each one the save restores and saves the same
/// bytes again, and every session restored so far answers the events still
/// to come as the live one does. Returns the last save.
fn plays_and_restores(source: &str, events: &[(&str, &str)]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    for (index, (event, payload)) in events.iter().enumerate() {
        let answer = send(&mut live, event, payload);
        for run in &mut restored {
            assert_eq!(send(run, event, payload), answer, "{index}: {event}");
        }
        let save = live.save_json().unwrap();
        let again = ReactiveSession::restore_json(source, &save)
            .unwrap_or_else(|error| panic!("{index}: {event}: {error}\n{save}"));
        assert_eq!(again.save_json().unwrap(), save, "{index}: {event}");
        restored.push(again);
    }
    let save = live.save_json().unwrap();
    for run in &restored {
        assert_eq!(run.save_json().unwrap(), save);
    }
    serde_json::from_str(&save).unwrap()
}

/// What restore says of `save`: `None` when it restores.
fn refusal(source: &str, save: &Value) -> Option<String> {
    ReactiveSession::restore_json(source, &save.to_string()).err()
}

fn refused(source: &str, save: &Value, expected: &str) {
    assert_eq!(
        refusal(source, save),
        Some(format!("cannot restore save: {expected}")),
        "{save}"
    );
}

/// `save` with the retirement of `record` dated `at`.
fn dated(save: &Value, record: &str, at: u64) -> Value {
    let mut forged = save.clone();
    forged["retired"][record] = json!(at);
    forged
}

const NOT_AT_THE_LAST_EVENT: &str = "is dated at the last event, which did not retire it";

const READING: (&str, &str) = ("reading", r#"{"v":1}"#);
const REGROW: (&str, &str) = ("regrow", "{}");
const GO: (&str, &str) = ("go", "{}");
const IDLE: (&str, &str) = ("idle", "{}");

/// The finding's program: the first renewal retires the declared occurrence.
const RENEWABLE: &str = "claim safe;
evidence sensor from \"sensor\";
renewable sensor window 1;
event regrow;
event go;
on regrow renew sensor;
";

/// The finding's forgery. `regrow` retired `sensor` at event 1; `go`, the
/// last event, renewed nothing and reports no effect.
#[test]
fn a_retirement_dated_at_a_last_event_that_renewed_nothing_is_refused() {
    let save = plays_and_restores(RENEWABLE, &[REGROW, GO]);
    assert_eq!(save["retired"], json!({"sensor": 1}), "{save}");
    assert!(save["effects"].is_null(), "{save}");
    refused(
        RENEWABLE,
        &dated(&save, "sensor", 2),
        &format!("retired sensor {NOT_AT_THE_LAST_EVENT}"),
    );
}

/// A last event that did renew retired another record of the evidence, and
/// reports that one.
#[test]
fn a_retirement_dated_at_the_last_event_is_one_of_its_effects() {
    let save = plays_and_restores(RENEWABLE, &[REGROW, GO, REGROW]);
    assert_eq!(save["retired"], json!({"sensor": 1}), "{save}");
    assert_eq!(
        save["effects"][0],
        json!({"kind": "retire", "history": "sensor", "record": "sensor@2"}),
        "{save}"
    );
    refused(
        RENEWABLE,
        &dated(&save, "sensor", 3),
        &format!("retired sensor {NOT_AT_THE_LAST_EVENT}"),
    );
}

/// The save an event writes reports each retirement it dates, so the report
/// cannot be taken out and the date left.
#[test]
fn a_last_events_retirement_cannot_lose_its_effect() {
    let mut save = plays_and_restores(RENEWABLE, &[REGROW]);
    assert_eq!(save["retired"], json!({"sensor": 1}), "{save}");
    assert_eq!(
        save["effects"][0],
        json!({"kind": "retire", "history": "sensor", "record": "sensor"}),
        "{save}"
    );
    save["effects"].as_array_mut().unwrap().remove(0);
    refused(
        RENEWABLE,
        &save,
        &format!("retired sensor {NOT_AT_THE_LAST_EVENT}"),
    );
}

/// `s` rests on the first reading, which stays held once the second retires
/// it.
const STREAM: &str = "evidence feed from \"feed\";
readings a from feed window 1;
state s = 0;
event reading v min 0 max 9;
event keep;
event idle;
on reading sample a = v;
on keep set s = latest(a);
";

/// The readings a save holds, each with its sequence.
fn held(save: &Value, stream: &str) -> Vec<(String, u64)> {
    save["reading_streams"][stream]["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .map(|reading| {
            (
                reading["id"].as_str().unwrap().to_string(),
                reading["sequence"].as_u64().unwrap(),
            )
        })
        .collect()
}

fn names(held: &[(&str, u64)]) -> Vec<(String, u64)> {
    held.iter()
        .map(|(name, sequence)| (name.to_string(), *sequence))
        .collect()
}

/// `a@2`, sampled by event 4, took the place of `a@1`: no other event
/// retired it.
#[test]
fn a_readings_retirement_is_dated_at_the_reading_that_took_its_place() {
    let keep = ("keep", "{}");
    let save = plays_and_restores(STREAM, &[READING, keep, IDLE, READING, IDLE, IDLE]);
    assert_eq!(save["retired"], json!({"a@1": 4}), "{save}");
    assert_eq!(held(&save, "a"), names(&[("a@1", 1), ("a@2", 4)]), "{save}");
    for at in [1, 2, 3, 5] {
        refused(
            STREAM,
            &dated(&save, "a@1", at),
            "retired a@1 is not dated at the event that added a@2",
        );
    }
    refused(
        STREAM,
        &dated(&save, "a@1", 6),
        &format!("retired a@1 {NOT_AT_THE_LAST_EVENT}"),
    );
    // Outside the session, as before.
    for at in [0, 7] {
        refused(
            STREAM,
            &dated(&save, "a@1", at),
            "retired a@1 is out of sequence",
        );
    }
}

/// Two states rest on the first two readings of a window of two, and the
/// readings that took their places depart.
const TWO_KEPT: &str = "evidence feed from \"feed\";
readings a from feed window 2;
state s1 = 0;
state s2 = 0;
event reading v min 0 max 9;
event keep1;
event keep2;
event idle;
on reading sample a = v;
on keep1 set s1 = latest(a);
on keep2 set s2 = latest(a);
";

/// When the reading that took a retired one's place has departed, the save
/// no longer dates it: the retirement lies between the readings held before
/// and after it, and restore refuses a date outside them.
#[test]
fn with_its_successor_departed_a_retirement_lies_between_the_readings_around_it() {
    let (keep1, keep2) = (("keep1", "{}"), ("keep2", "{}"));
    let save = plays_and_restores(
        TWO_KEPT,
        &[
            READING, keep1, READING, keep2, READING, READING, READING, IDLE, READING, IDLE,
        ],
    );
    assert_eq!(save["retired"], json!({"a@1": 5, "a@2": 6}), "{save}");
    // a@3 took the place of a@1 and a@4 that of a@2; both have departed.
    assert_eq!(
        held(&save, "a"),
        names(&[("a@1", 1), ("a@2", 3), ("a@5", 7), ("a@6", 9)]),
        "{save}"
    );
    // Before a@2, which a@3 followed.
    refused(
        TWO_KEPT,
        &dated(&save, "a@1", 2),
        "retired a@1 is not dated at the event that added a@3",
    );
    // After a@5, which followed a@4.
    refused(
        TWO_KEPT,
        &dated(&save, "a@2", 8),
        "retired a@2 is not dated at the event that added a@4",
    );
    // Between them the save tells no event from another.
    assert_eq!(refusal(TWO_KEPT, &dated(&save, "a@1", 4)), None, "{save}");
}

/// A save written by 0.1.0-rc.15, before records departed: it holds 145
/// retired journal entries under `journal window 6`. The departure gate
/// restores the same file.
const RC15_SOURCE: &str = include_str!("../../experiments/departure-gate/c3-windows.cav");
const RC15_SAVE: &str = include_str!("../../experiments/departure-gate/fixtures/rc15-c3-save.json");

#[test]
fn a_journal_entrys_retirement_is_dated_at_the_entry_that_took_its_place() {
    ReactiveSession::restore_json(RC15_SOURCE, RC15_SAVE).expect("the rc.15 save restores");
    let save: Value = serde_json::from_str(RC15_SAVE).unwrap();
    let journal = save["decision_journal"].as_array().unwrap();
    let retired = save["retired"].as_object().unwrap();
    let entries = retired
        .keys()
        .filter(|record| record.starts_with("journal@"))
        .count();
    assert_eq!((journal.len(), entries), (151, 145));
    // Entry 7 took the place of entry 1, and entry 8 that of entry 2.
    let seventh = journal[6]["sequence"].as_u64().unwrap();
    let eighth = journal[7]["sequence"].as_u64().unwrap();
    assert_eq!(retired["journal@1"], json!(seventh));
    assert_eq!(retired["journal@2"], json!(eighth));
    assert!(seventh + 1 < eighth, "{seventh} {eighth}");
    for at in [seventh - 1, seventh + 1] {
        refused(
            RC15_SOURCE,
            &dated(&save, "journal@1", at),
            "retired journal@1 is not dated at the event that added journal@7",
        );
    }
}

/// An edit that gives a save a fault restore already refused.
type Fault = fn(&mut Value);

/// A save refused before for something else is refused for that still:
/// the date is checked after everything else.
#[test]
fn an_older_fault_keeps_its_message() {
    let keep = ("keep", "{}");
    // The third reading retires the second, which departs: a save that
    // shows a departure holds no retired record that nothing pins.
    let save = plays_and_restores(STREAM, &[READING, keep, READING, READING, IDLE]);
    assert_eq!(save["retired"], json!({"a@1": 3}), "{save}");
    assert_eq!(held(&save, "a"), names(&[("a@1", 1), ("a@3", 4)]), "{save}");
    let wrong_date = dated(&save, "a@1", 5);
    refused(
        STREAM,
        &wrong_date,
        &format!("retired a@1 {NOT_AT_THE_LAST_EVENT}"),
    );
    let faults: [(&str, Fault); 3] = [
        // Checked before the retired records are.
        ("an unknown last event", |save| {
            save["last_event"] = json!("nothing")
        }),
        // Checked with them.
        ("a record the stream does not hold", |save| {
            save["retired"]["a@9"] = json!(4)
        }),
        // Checked after them: nothing pins the retired reading.
        ("no pin", |save| save["states"]["s"] = json!({"value": 1.0})),
    ];
    for (name, fault) in faults {
        let mut alone = save.clone();
        fault(&mut alone);
        let mut both = wrong_date.clone();
        fault(&mut both);
        let message = refusal(STREAM, &alone);
        assert!(message.is_some(), "{name} is refused");
        assert_ne!(message, refusal(STREAM, &wrong_date), "{name}");
        assert_eq!(refusal(STREAM, &both), message, "{name}");
    }
}

/// Two windowed streams, renewable evidence and a windowed journal, with
/// states that keep retired records held and an event that samples twice.
const MIXED: &str = "claim ok;
evidence feed from \"feed\";
evidence probe from \"probe\";
renewable probe window 2;
readings a from feed window 1;
readings b from feed window 3;
decisions d limit 64;
journal window 2;
state s1 = 0;
state s2 = 0;
state s3 = 0;
event reading v min 0 max 9;
event other v min 0 max 9;
event twice v min 0 max 9;
event keep1;
event keep2;
event keep3;
event again;
event decide;
event rethink;
event idle;
on reading sample a = v;
on other sample b = v;
on twice sample a = v;
on twice sample b = v;
on twice sample b = v + 1;
on keep1 set s1 = latest(a);
on keep2 set s2 = latest(b);
on keep3 when observed(probe) set s3 = qualified(1, probe) because nothing;
on again renew probe;
on again reveal probe supports ok;
on decide commit d because enough using latest(b);
on rethink when committed(d) and not reopened(d) reopen d because latest(a);
";

/// Every save a session writes keeps restoring, after accepted and refused
/// events alike: a retirement is dated as restore now requires. The steps
/// come from a fixed generator, so each run plays the same sessions.
#[test]
fn the_saves_of_sessions_with_every_kind_of_window_restore() {
    let events: [(&str, &str); 12] = [
        READING,
        ("other", r#"{"v":2}"#),
        ("twice", r#"{"v":3}"#),
        ("keep1", "{}"),
        ("keep2", "{}"),
        ("keep3", "{}"),
        ("again", "{}"),
        ("decide", "{}"),
        ("rethink", "{}"),
        IDLE,
        READING,
        ("again", "{}"),
    ];
    let mut retirements = 0;
    let mut kept = 0;
    let mut refusals = 0;
    for seed in 1..=6u64 {
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut live = ReactiveSession::from_source(MIXED).unwrap();
        let mut sequence = 0;
        for step in 0..160 {
            // A 64-bit linear congruential step (Knuth's MMIX constants).
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let (event, payload) = events[(state >> 33) as usize % events.len()];
            send(&mut live, event, payload);
            let save = live.save_json().unwrap();
            let again = ReactiveSession::restore_json(MIXED, &save)
                .unwrap_or_else(|error| panic!("seed {seed} step {step} {event}: {error}\n{save}"));
            assert_eq!(again.save_json().unwrap(), save, "seed {seed} step {step}");
            let save: Value = serde_json::from_str(&save).unwrap();
            // A refused event leaves the session, and its sequence, as it was.
            let after = save["sequence"].as_u64().unwrap();
            if after == sequence {
                refusals += 1;
                continue;
            }
            sequence = after;
            let retired = save["retired"].as_object().map_or(0, |map| map.len());
            kept = kept.max(retired);
            retirements += save["effects"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|effect| effect["kind"] == "retire")
                .count();
        }
    }
    // The sessions did retire records, held several retired at once, and
    // met refusals.
    assert!(retirements > 500, "{retirements}");
    assert!(kept >= 5, "{kept}");
    assert!(refusals > 40, "{refusals}");
}
