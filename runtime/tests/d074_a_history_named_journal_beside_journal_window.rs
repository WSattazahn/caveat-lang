//! D074: under `journal window`, the journal's entries are named
//! `journal@K`, and so are the readings of a reading stream and the
//! occurrences of renewable evidence named `journal`, either of which may
//! have a limit there though not a window, and the revisions of a decision
//! series named `journal`. The session kept an entry's retirement and
//! departure by name alone: in `retired`, the departure candidates, the pin
//! counts and the withdrawal collector. Once entry `journal@K` retired while
//! a record `journal@K` was held, a pin on the record kept the entry
//! retired past its window, reads of the record took it for retired, and
//! the entry's departure took the record's graph node, observation and
//! withdrawal with it and compacted what cited it. Restore then refused the
//! session's own saves ("observations omits reading journal@1"), debug
//! builds stopped on the collector's or the history reach audit, and a
//! renewable named `journal` committed less than the same program with the
//! history named otherwise. Restore also took a reading stream or evidence
//! named `journal` for the journal where it asks which take-back caveats
//! grounds can keep, and refused the session's saves there.
//!
//! An entry is never pinned and has no node, so it now departs in the event
//! that retires it and takes nothing a record of its spelling holds. Each
//! program here is played beside its twin, the same program with the history
//! named `jrnl`, whose records no journal entry shares a name with. See
//! spec/caveat-departure-0.1.md and spec/caveat-save-0.1.md ("Changes").
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

/// Plays `events` on `template` with `NAME` spelled `journal`, and on its
/// twin with `NAME` spelled `jrnl`. After each event the outcome and the
/// save are the twin's, name for name; the save restores and saves the same
/// bytes again; and every session restored so far answers the events still
/// to come as the live one does. Returns the save after each event.
fn plays_as_its_twin(template: &str, events: &[(&str, &str)]) -> Vec<Value> {
    let journal = template.replace("NAME", "journal");
    let twin_source = template.replace("NAME", "jrnl");
    let (journal_id, journal_sha) = source_ids(&journal);
    let (twin_id, twin_sha) = source_ids(&twin_source);
    let renamed = |text: &str| {
        hashless(
            &text
                .replace(&twin_id, &journal_id)
                .replace(&twin_sha, &journal_sha)
                .replace("jrnl", "journal"),
        )
    };
    let mut live = ReactiveSession::from_source(&journal).unwrap();
    let mut twin = ReactiveSession::from_source(&twin_source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    let mut saves = Vec::new();
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
        saves.push(serde_json::from_str(&save).unwrap());
    }
    let save = live.save_json().unwrap();
    for run in &restored {
        assert_eq!(run.save_json().unwrap(), save);
    }
    saves
}

const GO: (&str, &str) = ("go", "{}");
const REOPEN: (&str, &str) = ("reopen", "{}");
const IDLE: (&str, &str) = ("idle", "{}");
const READING: (&str, &str) = ("reading", r#"{"v":3}"#);
const READING_AGAIN: (&str, &str) = ("reading", r#"{"v":5}"#);

/// The names of the records the save's observation ledger holds.
fn observed(save: &Value) -> Vec<String> {
    save["observations"]
        .as_array()
        .map(|records| {
            records
                .iter()
                .map(|record| record.as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// D074's case B1: a reading stream named `journal` with a limit, which a
/// commitment's basis and a reopening cite.
const STREAM: &str = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
decisions d limit 4;
event go;
event reading v min 0 max 9;
event reopen;
event idle;
on go reveal e supports c;
readings NAME from feed limit 6;
journal window 1;
on go commit d because enough using qualified(latest(NAME), e);
on reading sample NAME = v;
on reopen when committed(d) and not reopened(d) reopen d because latest(NAME);
";

/// A pin on reading `journal@1` holds the reading alone. Before, its
/// citation by `d@1`'s basis held journal entry `journal@1` retired after
/// `reopen` retired it, and the entry's departure at the next `go` took
/// readings `journal@1` and `journal@2` out of the ledger and the graph.
/// Restore refused the save an optimized build wrote then, and every later
/// one, with "observations omits reading journal@1", and a debug build
/// stopped there on the history reach audit (D054).
#[test]
fn a_stream_named_journal_keeps_its_readings() {
    let saves = plays_as_its_twin(
        STREAM,
        &[
            READING,
            GO,
            READING_AGAIN,
            READING,
            REOPEN,
            GO,
            READING_AGAIN,
            IDLE,
            REOPEN,
            GO,
            IDLE,
        ],
    );
    // The reopening retired the commit's entry, and it departed at once.
    assert_eq!(saves[4]["journal_departed"], json!(1), "{}", saves[4]);
    assert_eq!(saves[4].get("retired"), None, "{}", saves[4]);
    for name in ["journal@1", "journal@2", "journal@3", "journal@4"] {
        assert!(
            observed(&saves[10]).iter().any(|record| record == name),
            "{name}: {}",
            saves[10]
        );
    }
}

/// D074's case B2: a decision series named `journal`, whose revisions are
/// `journal@K`. Debug builds stopped at the first `reopen` on the collector's
/// audit ("collector owner references"), since the entry's departure dropped
/// what the revision `journal@1` owns; restore refused the save an optimized
/// build wrote then ("journal@1 is not a commitment").
#[test]
fn a_series_named_journal_keeps_its_revisions() {
    let source = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
event go;
event reading v min 0 max 9;
event reopen;
event idle;
on go reveal e supports c;
readings r from feed limit 6;
decisions NAME limit 4;
journal window 1;
on go commit NAME because enough using qualified(latest(r), e);
on reading sample r = v;
on reopen when committed(NAME) and not reopened(NAME) reopen NAME because latest(r);
";
    let saves = plays_as_its_twin(
        source,
        &[
            READING,
            GO,
            READING_AGAIN,
            READING,
            REOPEN,
            GO,
            READING_AGAIN,
            IDLE,
            REOPEN,
            GO,
            IDLE,
        ],
    );
    assert_eq!(saves[4]["journal_departed"], json!(1), "{}", saves[4]);
    assert_eq!(saves[4].get("retired"), None, "{}", saves[4]);
}

/// D074's case B3: renewable evidence named `journal` with a limit, whose
/// renewals are `journal@2` and on. An entry held past its window by a pin
/// on the occurrence of its name left the session short of the twin's last
/// commitment, `d@3`.
#[test]
fn a_renewable_named_journal_commits_as_its_twin() {
    let source = "claim seen;
evidence NAME from \"a record\";
renewable NAME limit 4;
journal window 1;
decisions d limit 4;
event see;
event again;
event go;
event reopen;
on see reveal NAME supports seen;
on again renew NAME;
on again reveal NAME supports seen;
on go when observed(NAME) commit d because enough using qualified(1, NAME);
on reopen when committed(d) and not reopened(d) reopen d because NAME;
";
    let saves = plays_as_its_twin(
        source,
        &[
            ("see", "{}"),
            GO,
            ("again", "{}"),
            REOPEN,
            GO,
            ("again", "{}"),
            REOPEN,
            GO,
        ],
    );
    assert!(
        saves[7]["commitment_bases"].get("d@3").is_some(),
        "{}",
        saves[7]
    );
}

/// `history_count` and `history_at` read a history's live records. Once a
/// commit retired journal entry `journal@1`, a read later in that event took
/// reading `journal@1` for retired and left it out.
#[test]
fn a_read_of_a_stream_named_journal_counts_every_reading() {
    let source = "evidence feed from \"feed\";
readings NAME from feed limit 8;
decisions a limit 1;
decisions b limit 1;
decisions d limit 1;
journal window 1;
state n = 0;
state first = 0;
event reading v min 0 max 9;
event ga;
event gb;
event gd;
on reading sample NAME = v;
on ga commit a because enough using 1;
on ga set n = history_count(NAME);
on ga set first = history_at(NAME, 0);
on gb commit b because enough using 1;
on gb set n = history_count(NAME);
on gb set first = history_at(NAME, 0);
on gd commit d because enough using 1;
on gd set n = history_count(NAME);
on gd set first = history_at(NAME, 0);
";
    let saves = plays_as_its_twin(
        source,
        &[
            READING,
            ("ga", "{}"),
            READING_AGAIN,
            ("gb", "{}"),
            READING,
            ("gd", "{}"),
        ],
    );
    for (index, count) in [(1, 1.0), (3, 2.0), (5, 3.0)] {
        assert_eq!(
            saves[index]["states"]["n"]["value"],
            json!(count),
            "{index}: {}",
            saves[index]
        );
        assert_eq!(
            saves[index]["states"]["first"]["value"],
            json!(3.0),
            "{index}: {}",
            saves[index]
        );
    }
}

/// A declared reopening trigger reopens `d` from inside the sample that adds
/// reading `journal@K`, and that reopening retires journal entry
/// `journal@K` in the same event.
#[test]
fn a_reopening_trigger_on_a_stream_named_journal() {
    let source = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
readings NAME from feed limit 8;
decisions d limit 4 reopened by NAME;
journal window 1;
event go;
event reading v min 0 max 9;
on go reveal e supports c;
on go commit d because enough using qualified(1, e);
on reading sample NAME = v;
";
    plays_as_its_twin(
        source,
        &[READING, GO, READING_AGAIN, GO, READING, GO, READING_AGAIN],
    );
}

/// A hold defers a reopening for reading `journal@2`, and its release adds
/// two entries in one event, each retiring the one before. In an optimized
/// build that release was fatal ("journal entry journal@2 departed out of
/// order"); from `freeze` on restore refused each save ("observations omits
/// reading journal@1"), and a debug build stopped at the next reading on
/// the collector's audit.
#[test]
fn a_release_beside_a_stream_named_journal() {
    let source = "claim c;
evidence e from \"e\";
evidence team from \"team\";
evidence feed from \"feed\";
readings NAME from feed limit 8;
decisions d limit 4;
journal window 1;
event see;
event go;
event reading v min 0 max 9;
event freeze;
event reopen;
event unfreeze;
on see reveal e supports c;
on see reveal team supports c;
on go commit d because enough using qualified(1, e);
on reading sample NAME = v;
on freeze hold d because team;
on reopen reopen d because latest(NAME);
on unfreeze release d because team;
";
    plays_as_its_twin(
        source,
        &[
            ("see", "{}"),
            GO,
            READING,
            ("freeze", "{}"),
            READING_AGAIN,
            REOPEN,
            READING,
            ("unfreeze", "{}"),
            GO,
            READING_AGAIN,
        ],
    );
}

/// A withdrawal of reading `journal@1`: the departure of journal entry
/// `journal@1` took the withdrawal for its own and released its reason.
#[test]
fn a_withdrawn_reading_of_a_stream_named_journal() {
    let source = "claim c;
evidence e from \"e\";
evidence team from \"team\";
evidence feed from \"feed\";
readings NAME from feed limit 8;
decisions a limit 1;
decisions b limit 1;
journal window 1;
event see;
event ga;
event gb;
event reading v min 0 max 9;
event retract;
on see reveal e supports c;
on see reveal team supports c;
on ga commit a because enough using qualified(1, e);
on gb commit b because enough using qualified(1, e);
on reading sample NAME = v;
on retract withdraw latest(NAME) because team;
";
    let saves = plays_as_its_twin(
        source,
        &[
            ("see", "{}"),
            READING,
            ("retract", "{}"),
            ("ga", "{}"),
            ("gb", "{}"),
            READING_AGAIN,
        ],
    );
    assert_eq!(saves[4]["journal_departed"], json!(1), "{}", saves[4]);
    assert_eq!(
        saves[5]["withdrawals"][0]["evidence"],
        json!("journal@1"),
        "{}",
        saves[5]
    );
}

/// A take-back caveat on a stream named `journal`: `d`'s frozen basis keeps
/// `shaky`, and from `st` `t`'s grounds keep it through their citation of
/// `d`, while `t`'s lineage carries it live until `uq` takes it back (D098's
/// case, with a limit in place of the window). Restore took the stream for
/// the decision journal and refused the save written at `st` and every
/// later one, where a debug build stopped.
#[test]
fn a_take_back_on_a_stream_named_journal() {
    let source = "evidence ga from \"ga\";
readings NAME from ga limit 8;
caveat shaky consequence low;
decisions d limit 24;
journal window 1;
state t = 0;
event cd v min 0 max 10;
event qual v min 0 max 10;
event sa v min 0 max 10;
event st v min 0 max 10;
event uq v min 0 max 10;
on sa sample NAME = v;
on cd when latest(NAME) > 0 commit d because enough using v;
on st set t = latest(NAME) because latest(d);
on qual qualify latest(NAME) with shaky;
on uq unqualify latest(NAME) with shaky;
";
    let saves = plays_as_its_twin(
        source,
        &[
            ("sa", r#"{"v":4}"#),
            ("qual", r#"{"v":1}"#),
            ("cd", r#"{"v":9}"#),
            ("st", r#"{"v":7}"#),
            ("uq", r#"{"v":1}"#),
            ("cd", r#"{"v":8}"#),
            ("sa", r#"{"v":5}"#),
        ],
    );
    assert_eq!(
        saves[6]["states"]["t"]["grounds"]["kept"],
        json!(["shaky"]),
        "{}",
        saves[6]
    );
}

/// The same take-back on evidence named `journal`, which has no records:
/// `cd`'s guard reads `journal` while `shaky` qualifies it, so `d`'s frozen
/// basis keeps `shaky`, and from `st` `t`'s grounds keep it through their
/// citation of `d`, while `t`'s lineage carries it live until `uq` takes it
/// back. Restore took the evidence for the decision journal too and refused
/// the save written at `st` and every later one ("state t grounds keep a
/// caveat its lineage does not keep"), where a debug build stopped on the
/// history reach audit (D098).
#[test]
fn a_take_back_on_evidence_named_journal() {
    let source = "evidence NAME from \"ga\";
caveat shaky consequence low;
decisions d limit 24;
journal window 1;
state t = 0;
event cd v min 0 max 10;
event qual v min 0 max 10;
event sa v min 0 max 10;
event st v min 0 max 10;
event uq v min 0 max 10;
on sa reveal NAME;
on cd when qualified(1, NAME) > 0 commit d because enough using v;
on st set t = qualified(v, NAME) because latest(d);
on qual qualify NAME with shaky;
on uq unqualify NAME with shaky;
";
    let saves = plays_as_its_twin(
        source,
        &[
            ("sa", r#"{"v":4}"#),
            ("qual", r#"{"v":1}"#),
            ("cd", r#"{"v":9}"#),
            ("st", r#"{"v":7}"#),
            ("uq", r#"{"v":1}"#),
            ("cd", r#"{"v":8}"#),
            ("sa", r#"{"v":5}"#),
        ],
    );
    assert_eq!(
        saves[6]["states"]["t"]["grounds"]["kept"],
        json!(["shaky"]),
        "{}",
        saves[6]
    );
}

/// No history shares the journal's name. Departures are reported after
/// every other effect, by history and then number, journal entries among
/// them, and the archive holds them in that order.
#[test]
fn journal_entries_depart_in_history_order() {
    let source = "evidence feed from \"feed\";
readings alpha from feed window 1;
readings zeta from feed window 1;
decisions a limit 1;
decisions b limit 1;
journal window 1;
event first v min 0 max 9;
event second v min 0 max 9;
on first sample alpha = v;
on first sample zeta = v;
on first commit a because enough using v;
on second sample alpha = v;
on second sample zeta = v;
on second commit b because enough using v;
";
    let mut session = ReactiveSession::from_source(source).unwrap();
    send(&mut session, "first", r#"{"v":1}"#);
    let outcome: Value = serde_json::from_str(&send(&mut session, "second", r#"{"v":2}"#)).unwrap();
    let departures = outcome["snapshot"]["effects"]
        .as_array()
        .unwrap_or_else(|| panic!("{outcome}"))
        .iter()
        .filter(|effect| effect["kind"] == "depart")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        departures,
        vec![
            json!({"kind": "depart", "history": "alpha", "record": "alpha@1"}),
            json!({"kind": "depart", "history": "journal", "record": "journal@1"}),
            json!({"kind": "depart", "history": "zeta", "record": "zeta@1"}),
        ],
        "{outcome}"
    );
    // Each departed record, then the anchors its holders point to.
    let archive = serde_json::to_value(session.drain_archive()).unwrap();
    let records = archive
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item.get("holders").is_some())
        .map(|item| item["record"].clone())
        .collect::<Vec<_>>();
    assert_eq!(
        records,
        vec![json!("alpha@1"), json!("journal@1"), json!("zeta@1")],
        "{archive}"
    );
    assert_eq!(
        archive[1],
        json!({
            "record": "journal@1",
            "history": "journal",
            "number": 1,
            "retired_at": 2,
            "departed_at": 2,
            "journal_entry": archive[1]["journal_entry"].clone(),
            "holders": [],
        }),
        "{archive}"
    );
    assert_eq!(
        archive[1]["journal_entry"]["commitment"],
        json!("a@1"),
        "{archive}"
    );
}
