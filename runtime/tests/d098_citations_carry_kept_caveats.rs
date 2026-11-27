//! D098: restore refused a save the runtime wrote, in which a state's
//! grounds keep a take-back caveat that its lineage does not keep. A record
//! that holds a take-back caveat keeps it: a commitment's frozen basis, a
//! stored selection or reopening qualification, a record that has departed.
//! `set S = V because C` takes C's grounds as the state's, so a citation of
//! such a record gives the state's grounds the caveat kept, while its
//! lineage carries it only live, through the record its value read. The live
//! check compares names, so it accepts the citation. Restore required the
//! grounds to keep only what the lineage keeps, and refused that save; once
//! an `unqualify` took the caveat back, the lineage no longer carried it and
//! the grounds still kept it, and restore refused every later save as
//! "grounds include caveat ... outside its lineage". A commitment that cites
//! the state took the kept caveat into grounds its basis lacks, and since a
//! commitment's grounds never change, every later save of the session was
//! refused. Every release since take-backs (rc.18) did this. It is the kept
//! caveat sibling of D057's departed marker.
//!
//! Restore now accepts a take-back caveat grounds keep beyond their lineage
//! or basis where it can qualify records of a history a `because` citation
//! of the program can bring there, directly or through what reads the
//! grounds it gave (D057's bound), and refuses any other as before. See
//! spec/caveat-take-back-0.1.md ("Snapshot and save") and
//! spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn answer(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

/// Plays `events`, each with the outcome it must have (`accepted`, or the
/// code it is rejected with). After each, the save restores and saves the
/// same bytes again, and every session restored so far answers the events
/// still to come as the live one does. Returns the last save.
fn plays_and_restores(source: &str, events: &[(&str, &str, &str)]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    for (index, (event, payload, expected)) in events.iter().enumerate() {
        let outcome = answer(&mut live, event, payload);
        if *expected == "accepted" {
            assert_eq!(
                outcome["outcome"], "accepted",
                "{index}: {event}: {outcome}"
            );
        } else {
            assert_eq!(
                outcome["outcome"], "rejected",
                "{index}: {event}: {outcome}"
            );
            assert_eq!(outcome["code"], *expected, "{index}: {event}: {outcome}");
        }
        for run in &mut restored {
            assert_eq!(answer(run, event, payload), outcome, "{index}: {event}");
        }
        let save = live.save_json().unwrap();
        let again = ReactiveSession::restore_json(source, &save)
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

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted {save}"),
    }
}

/// One stream; `qual` and `uq` qualify and take back `shaky` on its latest
/// reading, so `shaky` is a take-back caveat. `t` cites `d`, whose basis is
/// frozen; `again`'s guard, when skipped, stores `d`'s selection.
const FROZEN: &str = "evidence ga from \"ga\";
readings a from ga window 1;
caveat shaky consequence low;
decisions d limit 24;
state t = 0;
event again v min 0 max 10;
event cd v min 0 max 10;
event qual v min 0 max 10;
event sa v min 0 max 10;
event st v min 0 max 10;
event uq v min 0 max 10;
on sa sample a = v;
on cd when latest(a) > 0 commit d because enough using v;
on st set t = latest(a) because latest(d);
on again when reopened(d) commit d because enough using latest(d);
on qual qualify latest(a) with shaky;
on uq unqualify latest(a) with shaky;
";

/// `a@1` is qualified before `d` commits on it, so `d`'s frozen basis keeps
/// `shaky`, and `t`'s grounds keep it through the citation, while `t`'s
/// lineage carries it through `a@1`'s live relation. After `uq` the lineage
/// no longer carries it and the grounds still keep it; setting `t` again
/// through the same citation is then refused live. No record departs.
#[test]
fn a_citation_of_a_frozen_basis_keeping_a_take_back_caveat_restores() {
    let save = plays_and_restores(
        FROZEN,
        &[
            ("sa", r#"{"v":4}"#, "accepted"),
            ("qual", r#"{"v":1}"#, "accepted"),
            ("cd", r#"{"v":9}"#, "accepted"),
            ("st", r#"{"v":7}"#, "accepted"),
            ("uq", r#"{"v":1}"#, "accepted"),
            ("st", r#"{"v":7}"#, "ungrounded_citation"),
            ("sa", r#"{"v":5}"#, "accepted"),
        ],
    );
    let t = &save["states"]["t"];
    assert_eq!(t["grounds"]["kept"], json!(["shaky"]));
    assert_eq!(t["grounds"]["caveats"], json!(["shaky"]));
    assert_eq!(t["lineage"].get("kept"), None);
}

/// `d` commits before `a@1` is qualified, and `again`'s skipped guard stores
/// a selection qualification that keeps `shaky`, which `latest(d)` carries.
#[test]
fn a_citation_of_a_skipped_guard_keeping_a_take_back_caveat_restores() {
    let save = plays_and_restores(
        FROZEN,
        &[
            ("sa", r#"{"v":4}"#, "accepted"),
            ("cd", r#"{"v":9}"#, "accepted"),
            ("qual", r#"{"v":10}"#, "accepted"),
            ("again", r#"{"v":6}"#, "accepted"),
            ("st", r#"{"v":7}"#, "accepted"),
            ("uq", r#"{"v":1}"#, "accepted"),
        ],
    );
    assert_eq!(save["states"]["t"]["grounds"]["kept"], json!(["shaky"]));
}

/// D057's program with a take-back caveat on the cited record: `b@1` is
/// qualified before `d` commits on it and then departs, so `d`'s basis holds
/// a `b` marker and keeps `shaky`. `b@2` is qualified too, so the citation's
/// names lie within `t`'s lineage and the live check accepts it.
const CITES_TAKEN_BACK: &str = "evidence gb from \"gb\";
readings b from gb window 1;
caveat shaky consequence low;
decisions d limit 4;
decisions r limit 4;
state t = 0;
event sb v min 0 max 10;
on sb sample b = v;
event qual v min 0 max 10;
on qual qualify latest(b) with shaky;
event uq v min 0 max 10;
on uq unqualify latest(b) with shaky;
event cd v min 0 max 10;
on cd when latest(b) > 0 commit d because enough using 1;
event st v min 0 max 10;
on st set t = latest(b) because latest(d);
event cr v min 0 max 10;
on cr commit r because enough citing t;
event reset v min 0 max 10;
on reset set t = 1;
";

#[test]
fn a_citation_of_a_departed_record_keeping_a_take_back_caveat_restores() {
    plays_and_restores(
        CITES_TAKEN_BACK,
        &[
            ("sb", r#"{"v":5}"#, "accepted"),
            ("qual", r#"{"v":1}"#, "accepted"),
            ("cd", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":6}"#, "accepted"),
            ("qual", r#"{"v":1}"#, "accepted"),
            ("st", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":7}"#, "accepted"),
        ],
    );
}

/// As above with the take-back of `b@2`'s live `shaky` before `b@2` departs:
/// the grounds keep `shaky` and the lineage no longer carries it, and
/// setting `t` again through `because latest(d)` is refused live.
#[test]
fn a_take_back_after_such_a_citation_leaves_later_saves_restorable() {
    plays_and_restores(
        CITES_TAKEN_BACK,
        &[
            ("sb", r#"{"v":5}"#, "accepted"),
            ("qual", r#"{"v":1}"#, "accepted"),
            ("cd", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":6}"#, "accepted"),
            ("qual", r#"{"v":1}"#, "accepted"),
            ("st", r#"{"v":1}"#, "accepted"),
            ("uq", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":7}"#, "accepted"),
            ("sb", r#"{"v":8}"#, "accepted"),
            ("st", r#"{"v":1}"#, "ungrounded_citation"),
        ],
    );
}

/// A commitment citing such a state keeps the caveat in its basis as in its
/// grounds while `t`'s lineage carries it; after `reset` its grounds keep a
/// `b 1..1` marker outside its basis's `b 2..2` (D057's commitment path).
#[test]
fn a_commitment_citing_such_a_state_restores() {
    let save = plays_and_restores(
        CITES_TAKEN_BACK,
        &[
            ("sb", r#"{"v":5}"#, "accepted"),
            ("qual", r#"{"v":1}"#, "accepted"),
            ("cd", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":6}"#, "accepted"),
            ("qual", r#"{"v":1}"#, "accepted"),
            ("st", r#"{"v":1}"#, "accepted"),
            ("cr", r#"{"v":1}"#, "accepted"),
            ("reset", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":7}"#, "accepted"),
        ],
    );
    assert_eq!(save["commitment_grounds"]["r@1"]["kept"], json!(["shaky"]));
    assert_eq!(
        save["commitment_bases"]["r@1"]["provenance"]["kept"],
        json!(["shaky"])
    );
}

/// The finding's program: `d` freezes `doubted` on `e` in its basis, `t`
/// cites it, `doubted` is taken back, and `r` cites `t`, so `r`'s grounds
/// keep `doubted` while its basis, made after the take-back, does not. A
/// commitment's grounds never change, so every later save held them.
const KEPT_COMMIT_ONLY: &str = "caveat doubted consequence material;
evidence e from \"e\";
decisions d limit 4;
decisions r limit 4;
state t = 0;
event see;
on see reveal e;
event doubt;
on doubt qualify e with doubted;
event cd;
on cd when qualified(1, e) > 0 commit d because enough using 1;
event st;
on st set t = qualified(1, e) because latest(d);
event reset;
on reset set t = 0;
event undoubt;
on undoubt unqualify e with doubted;
event cr;
on cr commit r because enough citing t;
event rr;
on rr when committed(r) reopen r because e;
";

#[test]
fn the_findings_commitment_restores_after_every_event() {
    let events = [
        "see", "doubt", "cd", "st", "undoubt", "cr", "reset", "see", "doubt", "undoubt", "rr",
        "cr", "reset", "see",
    ]
    .map(|event| (event, "{}", "accepted"));
    let save = plays_and_restores(KEPT_COMMIT_ONLY, &events);
    assert_eq!(
        save["commitment_grounds"]["r@1"]["kept"],
        json!(["doubted"])
    );
    assert_eq!(
        save["commitment_bases"]["r@1"]["provenance"].get("kept"),
        None
    );
}

/// A forged `kept` beyond the lineage is refused where no citation can bring
/// that caveat there: in a program with no citation, and in one whose only
/// citation reaches records the caveat cannot qualify.
#[test]
fn a_kept_caveat_no_citation_brings_is_refused() {
    // No citation: `t`'s grounds are its value's.
    let uncited = FROZEN.replace(" because latest(d)", "");
    let save = plays_and_restores(
        &uncited,
        &[
            ("sa", r#"{"v":4}"#, "accepted"),
            ("qual", r#"{"v":1}"#, "accepted"),
            ("cd", r#"{"v":9}"#, "accepted"),
            ("st", r#"{"v":7}"#, "accepted"),
        ],
    );
    let mut forged = save.clone();
    let t = forged["states"]["t"].as_object_mut().unwrap();
    assert_eq!(t["lineage"]["caveats"], json!(["shaky"]));
    let mut grounds = t["lineage"].clone();
    grounds["kept"] = json!(["shaky"]);
    t.insert("grounds".into(), grounds);
    refused(
        &uncited,
        &forged,
        "state t grounds keep a caveat its lineage does not keep",
    );

    // `t` cites `d`, whose basis reads only `a`, while `shaky` qualifies
    // only `c`: no citation can bring `shaky` into `t`'s grounds.
    let elsewhere = "evidence ga from \"ga\";
evidence gc from \"gc\";
readings a from ga window 1;
readings c from gc window 1;
caveat shaky consequence low;
decisions d limit 4;
state t = 0;
event sa v min 0 max 10;
event sc v min 0 max 10;
event qual v min 0 max 10;
event uq v min 0 max 10;
event cd v min 0 max 10;
event st v min 0 max 10;
on sa sample a = v;
on sc sample c = v;
on qual qualify latest(c) with shaky;
on uq unqualify latest(c) with shaky;
on cd when latest(a) > 0 commit d because enough using 1;
on st when latest(a) > 0 set t = latest(c) because latest(d);
";
    let save = plays_and_restores(
        elsewhere,
        &[
            ("sa", r#"{"v":4}"#, "accepted"),
            ("sc", r#"{"v":4}"#, "accepted"),
            ("qual", r#"{"v":1}"#, "accepted"),
            ("cd", r#"{"v":1}"#, "accepted"),
            ("st", r#"{"v":1}"#, "accepted"),
        ],
    );
    let mut forged = save.clone();
    let t = forged["states"]["t"].as_object_mut().unwrap();
    assert_eq!(t["lineage"]["caveats"], json!(["shaky"]));
    let grounds = t
        .get_mut("grounds")
        .expect("grounds narrower than the lineage");
    grounds["caveats"] = json!(["shaky"]);
    grounds["kept"] = json!(["shaky"]);
    refused(
        elsewhere,
        &forged,
        "state t grounds keep a caveat its lineage does not keep",
    );

    // The same in a commitment's grounds beyond its basis, with its journal
    // entry to match, in a program where nothing a commitment uses or cites
    // can bring it.
    let mut forged = save.clone();
    assert_eq!(forged["commitment_grounds"]["d@1"], json!({}));
    forged["commitment_grounds"]["d@1"] = json!({"caveats": ["shaky"], "kept": ["shaky"]});
    assert_eq!(forged["decision_journal"][0]["commitment"], "d@1");
    forged["decision_journal"][0]["caveats"] = json!(["shaky"]);
    refused(
        elsewhere,
        &forged,
        "commitment d@1 grounds include caveat shaky outside its lineage",
    );
}
