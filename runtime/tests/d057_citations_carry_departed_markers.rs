//! D057: restore refused a save the runtime wrote, in which a state's grounds
//! hold a departed marker that its lineage does not. `set S = V because C`
//! takes C's grounds as the state's. While a record C rests on is held, a
//! citation of it that the value never read is refused as
//! `ungrounded_citation`. Once the record departs, C holds only its history's
//! marker, and the live check, which compares names, accepts the citation.
//! The state's grounds then hold a marker of records its value never read,
//! and restore, which required each grounds marker to lie within the
//! lineage's, refused that save and every later one until the state was set
//! again. A state or a commitment that reads those grounds through `using`,
//! `citing` or its value takes the same marker into grounds that its lineage
//! or basis lacks, and was refused the same way. Every release since rc.16,
//! when records began to depart, did this.
//!
//! Restore now accepts a grounds marker beyond the lineage where a `because`
//! citation of the program can bring that history there, directly or through
//! what reads the grounds it gave, and refuses any other as before. See
//! spec/caveat-lineage-compaction-0.1.md ("Meaning") and
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

/// The histories a provenance holds markers of, with each marker's range.
fn markers(provenance: &Value) -> Vec<(String, u64, u64)> {
    provenance["departed"]
        .as_array()
        .map(|markers| {
            markers
                .iter()
                .map(|marker| {
                    (
                        marker["history"].as_str().unwrap().to_string(),
                        marker["from"].as_u64().unwrap(),
                        marker["through"].as_u64().unwrap(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn marker(history: &str, from: u64, through: u64) -> (String, u64, u64) {
    (history.to_string(), from, through)
}

/// The finding's program: `t` cites `d`, whose guard read `b@1`, which
/// departs as `b@2` arrives.
const CITES: &str = "evidence gb from \"gb\";
readings b from gb window 1;
decisions d limit 4;
state t = 0;
event sb v min 0 max 10;
on sb sample b = v;
event cd v min 0 max 10;
on cd when latest(b) > 0 commit d because enough using 1;
event st v min 0 max 10;
on st set t = latest(b) because latest(d);
";

#[test]
fn a_state_citing_a_departed_record_restores_after_every_event() {
    let save = plays_and_restores(
        CITES,
        &[
            ("sb", r#"{"v":5}"#, "accepted"),
            ("cd", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":6}"#, "accepted"),
            ("st", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":7}"#, "accepted"),
            ("sb", r#"{"v":8}"#, "accepted"),
        ],
    );
    let t = &save["states"]["t"];
    assert_eq!(t["value"], json!(6.0), "{save}");
    assert_eq!(markers(&t["grounds"]), [marker("b", 1, 1)], "{save}");
    assert_eq!(markers(&t["lineage"]), [marker("b", 2, 2)], "{save}");
}

/// The finding's second program: `t`'s value reads `a`, so citing `d` is
/// refused while `b@1` is held and accepted once it has departed.
const CITES_UNREAD: &str = "evidence ga from \"ga\";
evidence gb from \"gb\";
readings a from ga window 1;
readings b from gb window 1;
decisions d limit 4;
state t = 0;
event sa v min 0 max 10;
on sa sample a = v;
event sb v min 0 max 10;
on sb sample b = v;
event cd v min 0 max 10;
on cd when latest(b) > 0 commit d because enough using 1;
event st v min 0 max 10;
on st set t = latest(a) because latest(d);
";

#[test]
fn a_citation_refused_while_its_record_is_held_restores_once_it_departs() {
    let save = plays_and_restores(
        CITES_UNREAD,
        &[
            ("sa", r#"{"v":5}"#, "accepted"),
            ("sb", r#"{"v":5}"#, "accepted"),
            ("cd", r#"{"v":1}"#, "accepted"),
            ("st", r#"{"v":1}"#, "ungrounded_citation"),
            ("sb", r#"{"v":6}"#, "accepted"),
            ("st", r#"{"v":1}"#, "accepted"),
            ("sa", r#"{"v":7}"#, "accepted"),
            ("sb", r#"{"v":7}"#, "accepted"),
        ],
    );
    let t = &save["states"]["t"];
    assert_eq!(t["value"], json!(5.0), "{save}");
    assert_eq!(markers(&t["grounds"]), [marker("b", 1, 1)], "{save}");
    assert_eq!(markers(&t["lineage"]), [marker("a", 1, 1)], "{save}");
}

/// What reads `t`'s grounds takes the marker too: `u` through its value, `r`
/// through `citing` and `q` through `using`. Their lineage and bases take
/// `t`'s lineage.
const READS_CITED: &str = "evidence gb from \"gb\";
readings b from gb window 1;
decisions d limit 4;
decisions r limit 4;
decisions q limit 4;
state t = 0;
state u = 0;
event sb v min 0 max 10;
on sb sample b = v;
event cd v min 0 max 10;
on cd when latest(b) > 0 commit d because enough using 1;
event st v min 0 max 10;
on st set t = latest(b) because latest(d);
event cu v min 0 max 10;
on cu set u = t;
event cr v min 0 max 10;
on cr commit r because enough citing t;
event cq v min 0 max 10;
on cq commit q because enough using t;
";

#[test]
fn what_reads_those_grounds_restores_after_every_event() {
    let save = plays_and_restores(
        READS_CITED,
        &[
            ("sb", r#"{"v":5}"#, "accepted"),
            ("cd", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":6}"#, "accepted"),
            ("st", r#"{"v":1}"#, "accepted"),
            ("cu", r#"{"v":1}"#, "accepted"),
            ("cr", r#"{"v":1}"#, "accepted"),
            ("cq", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":7}"#, "accepted"),
            ("sb", r#"{"v":8}"#, "accepted"),
        ],
    );
    for state in ["t", "u"] {
        let cell = &save["states"][state];
        assert_eq!(
            markers(&cell["grounds"]),
            [marker("b", 1, 1)],
            "{state}: {save}"
        );
        assert_eq!(
            markers(&cell["lineage"]),
            [marker("b", 2, 2)],
            "{state}: {save}"
        );
    }
    for commitment in ["r@1", "q@1"] {
        assert_eq!(
            markers(&save["commitment_grounds"][commitment]),
            [marker("b", 1, 1)],
            "{commitment}: {save}"
        );
        assert_eq!(
            markers(&save["commitment_bases"][commitment]["provenance"]),
            [marker("b", 2, 2)],
            "{commitment}: {save}"
        );
    }
}

/// `t`'s guard reads `c`, which its lineage keeps and its grounds do not.
const GUARDED: &str = "evidence gb from \"gb\";
evidence gc from \"gc\";
readings b from gb window 1;
readings c from gc window 1;
decisions d limit 4;
decisions r limit 4;
state t = 0;
event sb v min 0 max 10;
on sb sample b = v;
event sc v min 0 max 10;
on sc sample c = v;
event cd v min 0 max 10;
on cd when latest(b) > 0 commit d because enough using 1;
event st v min 0 max 10;
on st when latest(c) > 0 set t = latest(b) because latest(d);
event cr v min 0 max 10;
on cr commit r because enough citing t;
";

#[test]
fn a_grounds_marker_outside_the_lineage_is_bounded_by_what_the_grounds_can_carry() {
    let save = plays_and_restores(
        GUARDED,
        &[
            ("sb", r#"{"v":5}"#, "accepted"),
            ("sc", r#"{"v":5}"#, "accepted"),
            ("cd", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":6}"#, "accepted"),
            ("sc", r#"{"v":6}"#, "accepted"),
            ("st", r#"{"v":1}"#, "accepted"),
            ("cr", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":7}"#, "accepted"),
            ("sc", r#"{"v":7}"#, "accepted"),
        ],
    );
    let t = &save["states"]["t"];
    assert_eq!(markers(&t["grounds"]), [marker("b", 1, 1)], "{save}");
    assert_eq!(
        markers(&t["lineage"]),
        [marker("b", 2, 2), marker("c", 2, 2)],
        "{save}"
    );
    // `c` reaches `t`'s lineage through the guard, never its grounds, nor
    // through them `r`'s. A `c` marker in those grounds that the lineage or
    // basis does not hold is refused as before; one it does hold is refused
    // as a history no rule carries into the grounds (D054).
    let lineage_c = t["lineage"]["departed"][1].clone();
    assert_eq!(lineage_c["history"], "c", "{save}");
    let mut earlier_c = lineage_c.clone();
    earlier_c["from"] = json!(1);
    earlier_c["through"] = json!(1);
    earlier_c["departed_at"] = save["states"]["t"]["grounds"]["departed"][0]["departed_at"].clone();
    for (grounds, place) in [
        ("/states/t/grounds", "state t"),
        ("/commitment_grounds/r@1", "commitment r@1"),
    ] {
        let mut forged = save.clone();
        let departed = forged.pointer_mut(grounds).unwrap()["departed"]
            .as_array_mut()
            .unwrap();
        departed.push(earlier_c.clone());
        refused(
            GUARDED,
            &forged,
            &format!("{place} grounds hold departed c records outside its lineage"),
        );
    }
    let mut forged = save.clone();
    forged["states"]["t"]["grounds"]["departed"]
        .as_array_mut()
        .unwrap()
        .push(lineage_c);
    refused(
        GUARDED,
        &forged,
        "state t grounds: no rule or declaration of this program carries records of c into it",
    );
}

/// The third program without its citation: no rule brings a marker beyond
/// `t`'s lineage, nor through `t` into what reads its grounds, so one forged
/// there is refused as before, as it is in every program without a citation.
#[test]
fn a_marker_beyond_the_lineage_is_refused_where_no_citation_brings_it() {
    let source = READS_CITED.replace(" because latest(d)", "");
    assert_ne!(source, READS_CITED);
    let save = plays_and_restores(
        &source,
        &[
            ("sb", r#"{"v":5}"#, "accepted"),
            ("cd", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":6}"#, "accepted"),
            ("st", r#"{"v":1}"#, "accepted"),
            ("cu", r#"{"v":1}"#, "accepted"),
            ("cr", r#"{"v":1}"#, "accepted"),
            ("cq", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":7}"#, "accepted"),
        ],
    );
    // `d`'s basis keeps the guard's departed `b@1`, which `t` never read.
    let departed = save["commitment_bases"]["d@1"]["provenance"]["departed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|marker| marker["history"] == "b")
        .unwrap_or_else(|| panic!("{save}"))
        .clone();
    assert_eq!(
        (&departed["from"], &departed["through"]),
        (&json!(1), &json!(1))
    );
    for (path, place) in [
        ("/states/t", "state t"),
        ("/states/u", "state u"),
        ("/commitment_grounds/r@1", "commitment r@1"),
        ("/commitment_grounds/q@1", "commitment q@1"),
    ] {
        let mut forged = save.clone();
        let mut held = forged.pointer_mut(path).unwrap_or_else(|| panic!("{save}"));
        // A state's grounds that equal its lineage are saved as the lineage.
        if path.starts_with("/states/") {
            if held.get("grounds").is_none() {
                held["grounds"] = held["lineage"].clone();
            }
            held = &mut held["grounds"];
        }
        assert!(markers(held).is_empty(), "{place}: {save}");
        held["departed"] = json!([departed]);
        refused(
            &source,
            &forged,
            &format!("{place} grounds hold departed b records outside its lineage"),
        );
    }
}
