//! D059: in a program without `journal window`, a windowed reading stream
//! or renewable evidence may be named `journal`, and its records are named
//! `journal@K` as the decision journal's entries are under a journal
//! window. Two name tests forgot to ask for the window. Restore refused
//! every save written by an event that retired such a record: "retire
//! effect is not one the last event can make". And the recount of pins left
//! out the `K`th journal entry's once the record `journal@K` had retired:
//! debug builds stopped on "incremental pin counts", and restore refused
//! saves with "retired journal@1 is pinned by nothing, so it would have
//! departed". Each program here is played beside its twin, the same
//! program with the history named `runs`. See spec/caveat-save-0.1.md
//! ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// The outcome of `event`, accepted or refused, as text.
fn send(session: &mut ReactiveSession, event: &str, payload: &str) -> String {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_string(&outcome).unwrap()
}

/// `text` with the hash of each archive reference blanked: an archive names
/// its history, so the twin's hashes differ.
fn hashless(text: &str) -> String {
    let key = "\"archive_ref\":\"";
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find(key) {
        let start = at + key.len();
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        rest = &rest[rest.find('"').expect("a closed string")..];
    }
    out.push_str(rest);
    out
}

fn source_ids(source: &str) -> (String, String) {
    let save = ReactiveSession::from_source(source)
        .unwrap()
        .save_json()
        .unwrap();
    let save: Value = serde_json::from_str(&save).unwrap();
    (
        save["source_id"].as_str().unwrap().to_string(),
        save["source_sha256"].as_str().unwrap().to_string(),
    )
}

/// Plays `events` on `template` with its history named `journal`, and on
/// its twin with the history named `runs`. After each event the outcome and
/// the save are the twin's, name for name; the save restores and saves the
/// same bytes again; and every session restored so far answers the events
/// still to come as the live one does. Returns the last save.
fn plays_as_its_twin(template: &str, events: &[(&str, &str)]) -> Value {
    let journal = template.replace("NAME", "journal");
    let runs = template.replace("NAME", "runs");
    let (journal_id, journal_sha) = source_ids(&journal);
    let (runs_id, runs_sha) = source_ids(&runs);
    let renamed = |text: &str| {
        hashless(
            &text
                .replace(&runs_id, &journal_id)
                .replace(&runs_sha, &journal_sha)
                .replace("runs", "journal"),
        )
    };
    let mut live = ReactiveSession::from_source(&journal).unwrap();
    let mut twin = ReactiveSession::from_source(&runs).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    for (index, (event, payload)) in events.iter().enumerate() {
        let answer = send(&mut live, event, payload);
        assert_eq!(
            hashless(&answer),
            renamed(&send(&mut twin, event, payload)),
            "{index}: {event}"
        );
        for run in &mut restored {
            assert_eq!(send(run, event, payload), answer, "{index}: {event}");
        }
        let save = live.save_json().unwrap();
        assert_eq!(
            hashless(&save),
            renamed(&twin.save_json().unwrap()),
            "{index}: {event}"
        );
        let again = ReactiveSession::restore_json(&journal, &save)
            .unwrap_or_else(|error| panic!("{index}: {event}: {error}"));
        assert_eq!(again.save_json().unwrap(), save, "{index}: {event}");
        restored.push(again);
    }
    let save = live.save_json().unwrap();
    for run in &restored {
        assert_eq!(run.save_json().unwrap(), save);
    }
    serde_json::from_str(&save).unwrap()
}

const READING: (&str, &str) = ("reading", r#"{"v":1}"#);
const GO: (&str, &str) = ("go", "{}");
const REOPEN: (&str, &str) = ("reopen", "{}");
const IDLE: (&str, &str) = ("idle", "{}");

/// Each reading after the first retires the one before it.
const STREAM: &str = "evidence feed from \"feed\";
readings NAME from feed window 1;
event reading v min 0 max 9;
event idle;
on reading sample NAME = v;
";

#[test]
fn a_windowed_stream_named_journal_restores_after_each_retirement() {
    let save = plays_as_its_twin(STREAM, &[READING, READING, READING, IDLE, READING]);
    assert_eq!(
        save["effects"][0],
        json!({"kind": "retire", "history": "journal", "record": "journal@3"}),
        "{save}"
    );
}

/// `take`'s guard reads the latest reading, so the lineage of `s` names it
/// and nothing pins it: the next reading retires it, it departs, and the
/// lineage keeps a marker of the history `journal` in its place.
#[test]
fn a_departed_record_of_a_stream_named_journal_leaves_its_marker() {
    let save = plays_as_its_twin(
        "evidence feed from \"feed\";
readings NAME from feed window 1;
state s = 0;
event reading v min 0 max 9;
event take;
event idle;
on reading sample NAME = v;
on take when latest(NAME) >= 0 set s = s + 1;
",
        &[
            READING,
            ("take", "{}"),
            READING,
            IDLE,
            ("take", "{}"),
            READING,
            IDLE,
            ("take", "{}"),
        ],
    );
    let lineage = &save["states"]["s"]["lineage"];
    assert_eq!(lineage["evidence"], json!(["journal@3"]), "{save}");
    let marker = &lineage["departed"][0];
    assert_eq!(marker["history"], "journal", "{save}");
    assert_eq!(marker["from"], json!(1), "{save}");
    assert_eq!(marker["through"], json!(2), "{save}");
}

/// Each renewal retires the occurrence before it, and `s` rests on one that
/// departs.
#[test]
fn renewable_evidence_named_journal_restores_after_each_retirement() {
    let save = plays_as_its_twin(
        "claim seen;
evidence NAME from \"a record\";
renewable NAME window 1;
state s = 0;
event see;
event again;
event go;
on see reveal NAME supports seen;
on again renew NAME;
on again reveal NAME supports seen;
on go when observed(NAME) set s = qualified(1, NAME) because nothing;
",
        &[
            ("see", "{}"),
            ("again", "{}"),
            GO,
            ("again", "{}"),
            ("again", "{}"),
            GO,
            ("again", "{}"),
        ],
    );
    assert_eq!(
        save["effects"][0],
        json!({"kind": "retire", "history": "journal", "record": "journal@4"}),
        "{save}"
    );
}

/// The journal has no window, so its entries never retire: the entry of
/// `d@1`'s commit, the first, cites `e` and the first reading, and keeps
/// both pinned after the record `journal@1` has retired.
const DECISIONS: &str = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
readings NAME from feed window 2;
decisions d limit 4;
event go;
event reading v min 0 max 9;
event reopen;
event idle;
on go reveal e supports c;
on go commit d because enough using qualified(latest(NAME), e);
on reading sample NAME = v;
on reopen when committed(d) and not reopened(d) reopen d because latest(NAME);
";

#[test]
fn a_journal_entry_keeps_its_pins_when_a_record_of_its_name_retires() {
    let save = plays_as_its_twin(
        DECISIONS,
        &[
            READING, GO, READING, READING, REOPEN, GO, READING, IDLE, READING, GO, IDLE,
        ],
    );
    // The first entry cites the record of its own name, which stays held,
    // retired and pinned, while later records depart.
    assert_eq!(
        save["decision_journal"][0]["because"],
        json!(["journal@1", "e"])
    );
    assert_eq!(save["retired"]["journal@1"], json!(4), "{save}");
    let held: Vec<&str> = save["reading_streams"]["journal"]["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .map(|reading| reading["id"].as_str().unwrap())
        .collect();
    assert_eq!(held[0], "journal@1", "{save}");
    assert!(!held.contains(&"journal@2"), "{save}");
}

#[test]
fn journal_entries_and_records_of_one_name_play_as_the_twin_does() {
    plays_as_its_twin(
        DECISIONS,
        &[
            READING, GO, REOPEN, GO, GO, IDLE, READING, READING, READING, GO, READING, IDLE,
        ],
    );
    plays_as_its_twin(
        DECISIONS,
        &[READING, GO, REOPEN, GO, READING, READING, READING, IDLE, GO],
    );
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted: {expected}"),
    }
}

/// The save after `events`, none of it restored.
fn played(source: &str, events: &[(&str, &str)]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    for (event, payload) in events {
        send(&mut live, event, payload);
    }
    serde_json::from_str(&live.save_json().unwrap()).unwrap()
}

/// A retirement of `journal` is still held to the last event's rules: no
/// rule of `idle` samples the stream.
#[test]
fn a_retirement_the_last_event_cannot_make_is_still_refused() {
    let source = STREAM.replace("NAME", "journal");
    let mut save = played(&source, &[READING, READING]);
    assert_eq!(
        save["effects"][0],
        json!({"kind": "retire", "history": "journal", "record": "journal@1"}),
        "{save}"
    );
    save["last_event"] = json!("idle");
    refused(
        &source,
        &save,
        "retire effect is not one the last event can make",
    );
}

/// And a retirement of `journal` in a program that has no history of that
/// name, and no journal window, is refused as before.
#[test]
fn a_retirement_of_a_journal_the_program_lacks_is_still_refused() {
    let source = STREAM.replace("NAME", "runs");
    let mut save = played(&source, &[READING, READING]);
    assert_eq!(
        save["effects"][0],
        json!({"kind": "retire", "history": "runs", "record": "runs@1"}),
        "{save}"
    );
    save["effects"][0]["history"] = json!("journal");
    refused(
        &source,
        &save,
        "retire effect is not one the last event can make",
    );
}
