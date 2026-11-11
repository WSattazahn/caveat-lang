//! D071: restore checked a windowed history's retired records in time that
//! grew with the square of the records the save holds. For each retired
//! record it scanned the records its history holds, and for each reopened
//! journal entry it visited every retired record. It now indexes the records
//! each windowed history holds once per restore, looks each retired record
//! up there, and looks up in the retired map only the records a reopening
//! cites. Every save restores, or is refused with the same message, as
//! before. See spec/caveat-save-0.1.md ("Changes").
//!
//! The timing test here covers the reopened-entry loop only: its retired
//! records are journal records, which restore finds by their number, not in
//! the index. The index is timed by d071_retired_index_tests in
//! src/reactive_save.rs, on restore_retired alone, since a whole restore
//! also runs restore_records' check of each occurrence, which is quadratic
//! in the same records.
use caveat_runtime::reactive::{ReactiveSession, ViewFields};
use serde_json::{json, Value};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

/// The tests in this file run one at a time, so that nothing in this binary
/// runs beside the timing test.
static ALONE: Mutex<()> = Mutex::new(());

fn alone() -> MutexGuard<'static, ()> {
    ALONE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn dispatch(session: &mut ReactiveSession, event: &str, payload: &str) {
    let none = ViewFields::from_names([]).unwrap();
    let outcome = session
        .dispatch_view_fields_outcome_json(event, payload, none)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
}

/// Plays `events` and returns the save, which restores to a session that
/// writes the same save.
fn played(source: &str, events: &[(&str, &str)]) -> Value {
    let mut session = ReactiveSession::from_source(source).unwrap();
    for (event, payload) in events {
        dispatch(&mut session, event, payload);
    }
    let save = session.save_json().unwrap();
    let again = ReactiveSession::restore_json(source, &save)
        .unwrap_or_else(|error| panic!("{error}: {save}"));
    assert_eq!(again.save_json().unwrap(), save);
    serde_json::from_str(&save).unwrap()
}

/// The refusal of `save` changed by `forge`.
fn refusal(source: &str, save: &Value, forge: impl FnOnce(&mut Value)) -> String {
    let mut forged = save.clone();
    forge(&mut forged);
    match ReactiveSession::restore_json(source, &forged.to_string()) {
        Err(error) => error,
        Ok(_) => panic!("accepted {forged}"),
    }
}

// The timing test.

/// Causes of a reopening, each its own evidence and event.
const CAUSES: usize = 64;
/// The program's journal window.
const WINDOW: usize = 2;
/// Rounds in the smaller and the larger save: the larger holds 64 times the
/// journal entries of the smaller.
const SMALL: usize = 4;
const LARGE: usize = 256;
/// A restore that takes time linear in the save takes about 64 times as
/// long for the larger save; this allows four times that.
const BOUND: f64 = 4.0 * (LARGE / SMALL) as f64;

/// Each round, `decide` commits d1 and each `rI` reopens it because of its
/// own evidence; `bump` ends the play with an event that changes no
/// decision.
fn program(journal_window: bool) -> String {
    let mut source = String::from(
        "claim ok;\nevent decide;\nstate t = 0;\nevent bump;\non bump set t = t + 1;\n\
         decisions d1 limit 256;\non decide commit d1 because enough using 1;\n",
    );
    if journal_window {
        source.push_str(&format!("journal window {WINDOW};\n"));
    }
    for i in 1..=CAUSES {
        source.push_str(&format!(
            "evidence e{i:03} from \"tool\";\nevent r{i};\non r{i} reveal e{i:03} supports ok;\n\
             on r{i} reopen d1 because e{i:03};\n"
        ));
    }
    source
}

/// The save a session of `program(false)` writes after `rounds` rounds of
/// `decide`, `r1` ... `r64` and then `bump`, built directly, since playing
/// a large one takes long.
fn generated(rounds: usize) -> Value {
    let all: Vec<String> = (1..=CAUSES).map(|i| format!("e{i:03}")).collect();
    let revision = |k: usize| format!("d1@{k}");
    let start = |k: usize| ((k - 1) * (CAUSES + 1) + 1) as u64;
    let mut nodes = Vec::new();
    let mut relations = Vec::new();
    let mut open = Vec::new();
    let mut bases = serde_json::Map::new();
    let mut grounds = serde_json::Map::new();
    let mut revisions = Vec::new();
    let mut reopenings = serde_json::Map::new();
    let mut journal = Vec::new();
    for k in 1..=rounds {
        nodes.push(json!({"kind": "commitment", "name": revision(k), "reason": "enough"}));
        open.push(json!(revision(k)));
        if k > 1 {
            for name in &all {
                relations.push(json!([revision(k), "relies_on", name]));
            }
        }
        for name in &all {
            if k == 1 {
                relations.push(json!([name, "supports", "ok"]));
            }
            relations.push(json!([name, "reopens", revision(k)]));
        }
        let evidence = if k == 1 { vec![] } else { all.clone() };
        bases.insert(
            revision(k),
            json!({"value": 1.0, "provenance": {"evidence": evidence, "caveats": []}}),
        );
        grounds.insert(revision(k), json!({}));
        revisions.push(json!({
            "id": revision(k),
            "previous": if k == 1 { Value::Null } else { json!(revision(k - 1)) },
            "ordinal": k,
            "sequence": start(k),
            "event": "decide",
        }));
        reopenings.insert(revision(k), json!({"evidence": all}));
        journal.push(json!({
            "decision": "d1", "commitment": revision(k), "change": "committed",
            "sequence": start(k), "event": "decide", "elapsed": 0.0, "value": 1.0,
            "because": [], "caveats": [],
        }));
        for (i, name) in all.iter().enumerate() {
            journal.push(json!({
                "decision": "d1", "commitment": revision(k), "change": "reopened",
                "sequence": start(k) + i as u64 + 1, "event": format!("r{}", i + 1),
                "elapsed": 0.0, "value": 1.0, "because": [name], "caveats": [],
            }));
        }
    }
    let fresh = fresh_save(&program(false));
    json!({
        "schema": fresh["schema"],
        "source_id": fresh["source_id"],
        "source_sha256": fresh["source_sha256"],
        "sequence": (rounds * (CAUSES + 1) + 1) as u64,
        "last_event": "bump",
        "elapsed": 0.0,
        "states": {"t": {"value": 1.0}},
        "graph": {"nodes": nodes, "relations": relations, "open": open},
        "commitment_bases": bases,
        "commitment_grounds": grounds,
        "decision_series": {"d1": {
            "limit": 256, "current": revision(rounds), "revisions": revisions,
            "selection_qualifications": {"evidence": [], "caveats": []},
        }},
        "observation_qualifications": all
            .iter()
            .map(|name| (name.clone(), json!({})))
            .collect::<serde_json::Map<_, _>>(),
        "reopening_qualifications": reopenings,
        "decision_journal": journal,
    })
}

fn fresh_save(source: &str) -> Value {
    let session = ReactiveSession::from_source(source).unwrap();
    serde_json::from_str(&session.save_json().unwrap()).unwrap()
}

/// `generated(rounds)` for `program(true)`, in the form of rc.15's windowed
/// saves: every journal entry but the last two retired, when the entry two
/// after it was added, and none departed (spec/caveat-departure-0.1.md).
fn older(rounds: usize) -> String {
    let mut save = generated(rounds);
    let fresh = fresh_save(&program(true));
    save["source_id"] = fresh["source_id"].clone();
    save["source_sha256"] = fresh["source_sha256"].clone();
    let journal = save["decision_journal"].as_array().unwrap();
    let retired = (0..journal.len() - WINDOW)
        .map(|j| {
            (
                format!("journal@{}", j + 1),
                journal[j + WINDOW]["sequence"].clone(),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    save["retired"] = Value::Object(retired);
    save.to_string()
}

/// This thread's CPU time, which other processes and threads do not
/// advance.
#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
fn cpu_time() -> Duration {
    #[repr(C)]
    struct Timespec {
        seconds: std::ffi::c_long,
        nanoseconds: std::ffi::c_long,
    }
    extern "C" {
        fn clock_gettime(clock: i32, time: *mut Timespec) -> i32;
    }
    const CLOCK_THREAD_CPUTIME_ID: i32 = 3;
    let mut time = Timespec {
        seconds: 0,
        nanoseconds: 0,
    };
    // SAFETY: clock_gettime writes one timespec through a valid pointer.
    let status = unsafe { clock_gettime(CLOCK_THREAD_CPUTIME_ID, &mut time) };
    assert_eq!(status, 0, "clock_gettime");
    Duration::new(time.seconds as u64, time.nanoseconds as u32)
}

/// Elsewhere, the time since the first call.
#[cfg(not(all(target_os = "linux", target_pointer_width = "64")))]
fn cpu_time() -> Duration {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed()
}

/// Seconds `ReactiveSession::restore_json` takes to restore `save`, which
/// it must accept.
fn restore_time(source: &str, save: &str) -> f64 {
    let start = cpu_time();
    let restored = ReactiveSession::restore_json(source, save);
    let time = (cpu_time() - start).as_secs_f64();
    restored.unwrap_or_else(|error| panic!("{error}"));
    time
}

/// Each round commits d1 and reopens it once for each of 64 causes, so a
/// save of K rounds holds 65K journal entries, 64K of them reopenings, and
/// all but the last two retired. Before D071 each reopening visited every
/// retired record. The time each save takes is the least of several
/// restores on this thread's CPU clock.
#[test]
fn retired_records_are_checked_in_linear_time() {
    let _alone = alone();
    // The generated save is the one a session writes, checked at a size
    // that is quick to play.
    let mut session = ReactiveSession::from_source(&program(false)).unwrap();
    for _ in 0..2 {
        dispatch(&mut session, "decide", "{}");
        for i in 1..=CAUSES {
            dispatch(&mut session, &format!("r{i}"), "{}");
        }
    }
    dispatch(&mut session, "bump", "{}");
    let written: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    assert_eq!(generated(2), written);
    let source = program(true);
    let small = older(SMALL);
    let large = older(LARGE);
    // Restore accepts the older form and writes it back as it was.
    let restored = ReactiveSession::restore_json(&source, &small).unwrap();
    let again: Value = serde_json::from_str(&restored.save_json().unwrap()).unwrap();
    assert_eq!(again, serde_json::from_str::<Value>(&small).unwrap());
    let small_time = (0..5)
        .map(|_| restore_time(&source, &small))
        .fold(f64::INFINITY, f64::min);
    let bound = BOUND * small_time;
    let mut large_time = f64::INFINITY;
    for _ in 0..3 {
        large_time = large_time.min(restore_time(&source, &large));
        if large_time < bound {
            break;
        }
    }
    let ratio = large_time / small_time;
    eprintln!(
        "restore: {SMALL} rounds {:.1} ms ({} bytes), {LARGE} rounds {:.1} ms ({} bytes), \
         ratio {ratio:.1}, bound {BOUND}",
        small_time * 1e3,
        small.len(),
        large_time * 1e3,
        large.len(),
    );
    assert!(
        large_time < bound,
        "restoring {LARGE} rounds took {ratio:.1} times as long as {SMALL} rounds \
         ({:.1} ms against {:.1} ms); linear time takes about {}",
        large_time * 1e3,
        small_time * 1e3,
        LARGE / SMALL,
    );
}

// What restore refuses, and with which message, is as before.

/// D050's program without its hold or withdrawal: `go` commits `d` on the
/// latest reading, `reopen` reopens it because of the latest reading, and
/// `idle` does nothing.
const STREAM: &str = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
readings runs from feed window 2;
decisions d limit 2;
event go;
event reading v min 0 max 9;
event reopen;
event idle;
on go reveal e supports c;
on go commit d because enough using qualified(latest(runs), e);
on reading sample runs = v;
on reopen when committed(d) and not reopened(d) reopen d because latest(runs);
";

/// runs@1 is read at 2 and cited by the commitment at 3 and the reopening
/// at 5; it retires at 7, when runs@3 is read.
fn stream_save() -> Value {
    let reading = ("reading", r#"{"v":0}"#);
    let idle = ("idle", "{}");
    let save = played(
        STREAM,
        &[
            idle,
            reading,
            ("go", "{}"),
            idle,
            ("reopen", "{}"),
            reading,
            reading,
            idle,
        ],
    );
    assert_eq!(save["retired"], json!({"runs@1": 7}), "{save}");
    let occurrences = &save["reading_streams"]["runs"]["occurrences"];
    assert_eq!(occurrences[0]["id"], "runs@1");
    assert_eq!(occurrences[0]["sequence"], 2);
    let reopened = &save["decision_journal"][1];
    assert_eq!(reopened["change"], "reopened");
    assert_eq!(reopened["sequence"], 5);
    assert_eq!(reopened["because"], json!(["runs@1"]));
    save
}

fn retire(save: &mut Value, record: &str, at: u64) {
    save["retired"][record] = json!(at);
}

fn refused(message: &str) -> String {
    format!("cannot restore save: {message}")
}

#[test]
fn a_retired_record_no_history_holds_is_refused_as_before() {
    let _alone = alone();
    let save = stream_save();
    for record in ["runs@9", "runs", "runs@01", "e", "journal@1"] {
        assert_eq!(
            refusal(STREAM, &save, |save| retire(save, record, 7)),
            refused(&format!(
                "retired {record} is no record of a windowed history it holds"
            )),
        );
    }
}

#[test]
fn a_retired_record_out_of_sequence_is_refused_as_before() {
    let _alone = alone();
    let save = stream_save();
    // Before it was read at 2, at 0, or after the save's sequence 8.
    for at in [1, 0, 9] {
        assert_eq!(
            refusal(STREAM, &save, |save| retire(save, "runs@1", at)),
            refused("retired runs@1 is out of sequence"),
            "{at}",
        );
    }
}

#[test]
fn a_reopening_that_cites_a_retired_record_is_refused_as_before() {
    let _alone = alone();
    let save = stream_save();
    // After it was read and cited by the commitment, before the reopening.
    assert_eq!(
        refusal(STREAM, &save, |save| retire(save, "runs@1", 4)),
        refused("d@1 reopened after runs@1 retired reads it"),
    );
}

/// A stream occurrence or a renewal listed twice is refused before
/// restore_retired runs, by restore_graph and restore_records, with the
/// messages they gave before D071. These pin those refusals; the index's
/// rule that the first of two entries stands is not reached by any save.
#[test]
fn a_record_held_twice_is_refused_as_before() {
    let _alone = alone();
    let save = stream_save();
    assert_eq!(
        refusal(STREAM, &save, |save| {
            let occurrences = save["reading_streams"]["runs"]["occurrences"]
                .as_array_mut()
                .unwrap();
            occurrences.insert(1, occurrences[0].clone());
        }),
        refused("observations changes reading chronology"),
    );
    let (source, save) = two_cited_save();
    assert_eq!(
        refusal(source, &save, |save| {
            let renewals = save["renewals"]["alpha"].as_array_mut().unwrap();
            renewals.insert(1, renewals[0].clone());
        }),
        refused("renewable alpha occurrence alpha is out of order"),
    );
}

/// `reconsider` reopens `plan` because of each record in the grounds of
/// `s` that `old` qualifies, zeta and alpha, in the order they were
/// observed; `renew` retires both.
const TWO_CITED: &str = "caveat old consequence material;
claim fresh;
evidence zeta from \"z\";
renewable zeta window 1;
evidence alpha from \"a\";
renewable alpha window 1;
state s = 0;
decisions plan limit 4;
event see;
event read;
event decide;
event flag;
event reconsider;
event renew;
event idle;
on see reveal zeta supports fresh;
on see reveal alpha supports fresh;
on read set s = qualified(1, zeta) + qualified(1, alpha);
on decide commit plan because enough using s;
on flag qualify zeta with old;
on flag qualify alpha with old;
on reconsider when committed(plan) reopen plan because caveated(s, old);
on renew renew zeta;
on renew renew alpha;
";

fn two_cited_save() -> (&'static str, Value) {
    let events = [
        "see",
        "read",
        "decide",
        "flag",
        "reconsider",
        "renew",
        "idle",
    ]
    .map(|event| (event, "{}"));
    let save = played(TWO_CITED, &events);
    assert_eq!(save["retired"], json!({"alpha": 6, "zeta": 6}), "{save}");
    let reopened = &save["decision_journal"][1];
    assert_eq!(reopened["change"], "reopened", "{save}");
    assert_eq!(reopened["sequence"], 5);
    // Not in the retired map's order.
    assert_eq!(reopened["because"], json!(["zeta", "alpha"]));
    (TWO_CITED, save)
}

#[test]
fn a_renewable_evidence_never_renewed_is_its_one_record() {
    let _alone = alone();
    // The save lists no renewals of alpha, whose one record is alpha
    // itself: a record of its history, so not refused as none, and its
    // newest, so it must stay live.
    let events = ["see", "read", "decide", "idle"].map(|event| (event, "{}"));
    let save = played(TWO_CITED, &events);
    assert!(save["renewals"]["alpha"].is_null(), "{save}");
    assert_eq!(
        refusal(TWO_CITED, &save, |save| retire(save, "alpha", 3)),
        refused("alpha does not keep exactly its newest 1 records live"),
    );
}

#[test]
fn of_several_faults_the_same_is_reported_first() {
    let _alone = alone();
    let save = stream_save();
    // The retired map is visited in its order, by name.
    assert_eq!(
        refusal(STREAM, &save, |save| {
            retire(save, "runs@10", 7);
            retire(save, "runs@1", 1);
        }),
        refused("retired runs@1 is out of sequence"),
    );
    assert_eq!(
        refusal(STREAM, &save, |save| {
            retire(save, "e", 7);
            retire(save, "runs@1", 1);
        }),
        refused("retired e is no record of a windowed history it holds"),
    );
    // A reopening that cites two records retired before it is refused for
    // the first in the retired map's order, not in the order it cites them.
    let (source, save) = two_cited_save();
    assert_eq!(
        refusal(source, &save, |save| retire(save, "zeta", 4)),
        refused("plan@1 reopened after zeta retired reads it"),
    );
    assert_eq!(
        refusal(source, &save, |save| {
            retire(save, "zeta", 4);
            retire(save, "alpha", 4);
        }),
        refused("plan@1 reopened after alpha retired reads it"),
    );
}

/// A reading retired before it was read is refused for its own read, not
/// for the reading before it or the stream's first: runs@2 is read at 6 and
/// runs@3 at 7, each after the reading before it.
#[test]
fn a_later_reading_retired_before_it_was_read_is_refused_as_before() {
    let _alone = alone();
    let save = stream_save();
    let occurrences = &save["reading_streams"]["runs"]["occurrences"];
    assert_eq!(occurrences[1]["id"], "runs@2");
    assert_eq!(occurrences[1]["sequence"], 6);
    assert_eq!(occurrences[2]["id"], "runs@3");
    assert_eq!(occurrences[2]["sequence"], 7);
    for (record, at) in [("runs@2", 3), ("runs@2", 5), ("runs@3", 6)] {
        assert_eq!(
            refusal(STREAM, &save, |save| retire(save, record, at)),
            refused(&format!("retired {record} is out of sequence")),
            "{record} at {at}",
        );
    }
}
