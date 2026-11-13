//! D062: restore accepted a save whose journal left out the entry of a
//! hold, of a deferred reopening or of a release, and one whose series left
//! out what its journal holds. A hold, a deferral and a release each journal
//! an entry in the event that makes them, and a reopening ends a hold. So
//! restore now reads each commitment's `held`, `deferred`, `released` and
//! `reopened` entries in order, and holds them, the series' `on_hold` and
//! `deferred`, and the reopening record to each other. See
//! spec/caveat-holds-0.1.md, section 4.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// The outcome of `event`, accepted or refused, as text.
fn send(session: &mut ReactiveSession, event: &str, payload: &str) -> String {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_string(&outcome).unwrap()
}

/// Plays `events`, each accepted. After each one the save restores and saves
/// the same bytes again, and every session restored so far answers the
/// events still to come as the live one does. Returns the last save.
fn plays_and_restores(source: &str, events: &[&str]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    for (index, event) in events.iter().enumerate() {
        let answer = send(&mut live, event, "{}");
        assert!(answer.contains("accepted"), "{index}: {event}: {answer}");
        for run in &mut restored {
            assert_eq!(send(run, event, "{}"), answer, "{index}: {event}");
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

fn refused(source: &str, save: &Value, expected: &str) {
    assert_eq!(
        ReactiveSession::restore_json(source, &save.to_string()).err(),
        Some(format!("cannot restore save: {expected}")),
        "{save}"
    );
}

/// The journal as text: `COMMITMENT CHANGE CAUSES` for each entry.
fn journal(save: &Value) -> Vec<String> {
    save["decision_journal"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|entry| {
            let mut line = format!(
                "{} {}",
                entry["commitment"].as_str().unwrap(),
                entry["change"].as_str().unwrap()
            );
            for cause in entry["because"].as_array().unwrap() {
                line.push(' ');
                line.push_str(cause.as_str().unwrap());
            }
            line
        })
        .collect()
}

/// `save` without its first journal entry whose change is `change`.
fn without(save: &Value, change: &str) -> Value {
    let mut forged = save.clone();
    let entries = forged["decision_journal"].as_array_mut().unwrap();
    let index = entries
        .iter()
        .position(|entry| entry["change"] == change)
        .unwrap();
    entries.remove(index);
    forged
}

/// `save` with one more journal entry at `index`: the entry before it, with
/// `change`, `event`, `sequence` and `because` in place of its own.
fn with_entry(
    save: &Value,
    index: usize,
    change: &str,
    (event, sequence): (&str, u64),
    because: &[&str],
) -> Value {
    let mut forged = save.clone();
    let entries = forged["decision_journal"].as_array_mut().unwrap();
    let mut entry = entries[index - 1].clone();
    entry["change"] = json!(change);
    entry["event"] = json!(event);
    entry["sequence"] = json!(sequence);
    entry["because"] = json!(because);
    entries.insert(index, entry);
    forged
}

/// `save` with `field` of series `d` set to `names`, or left out with none.
fn with_series(save: &Value, field: &str, names: &[&str]) -> Value {
    let mut forged = save.clone();
    let series = forged["decision_series"]["d"].as_object_mut().unwrap();
    if names.is_empty() {
        series.remove(field);
    } else {
        series.insert(field.into(), json!(names));
    }
    forged
}

/// A series that `pause` and `more` hold, `go` and `also` reopen and `rel`
/// releases; `both` releases it and then reopens it; and a one-off
/// commitment, which `apart` reopens after it releases the series.
const HOLD: &str = "claim safe;
evidence e from \"e\";
evidence f from \"f\";
evidence g from \"g\";
decisions d limit 4;
event seee;
event seef;
event seeg;
event dec;
event pause;
event more;
event go;
event also;
event rel;
event both;
event apart;
event plan;
event redo;
event idle;
on seee reveal e supports safe;
on seef reveal f supports safe;
on seeg reveal g supports safe;
on dec when not committed(d) or reopened(d) commit d because enough using 1;
on pause hold d because e;
on more hold d because g;
on go reopen d because f;
on also reopen d because g;
on rel release d because g;
on both release d because g;
on both reopen d because f;
on apart release d because g;
on apart reopen act because f;
on plan commit act because enough using 1;
on redo reopen act because f;
";

/// `HOLD` with a windowed journal.
fn windowed(window: usize) -> String {
    format!("{HOLD}journal window {window};\n")
}

/// A hold in force: the finding's first save.
fn held() -> Value {
    let save = plays_and_restores(HOLD, &["seee", "dec", "pause", "seef"]);
    assert_eq!(journal(&save), ["d@1 committed", "d@1 held e"]);
    assert_eq!(save["decision_series"]["d"]["on_hold"], json!(["e"]));
    save
}

/// A hold that defers a reopening: the finding's second save.
fn deferring() -> Value {
    let save = plays_and_restores(HOLD, &["seee", "seef", "dec", "pause", "go", "seeg"]);
    assert_eq!(
        journal(&save),
        ["d@1 committed", "d@1 held e", "d@1 deferred f"]
    );
    assert_eq!(save["decision_series"]["d"]["deferred"], json!(["f"]));
    save
}

/// A hold released with nothing deferred: the finding's third save.
fn released() -> Value {
    let save = plays_and_restores(HOLD, &["seee", "seeg", "dec", "pause", "rel"]);
    assert_eq!(
        journal(&save),
        ["d@1 committed", "d@1 held e", "d@1 released g"]
    );
    assert!(save["decision_series"]["d"]["on_hold"].is_null(), "{save}");
    save
}

/// A hold released with a reopening deferred: the release reopens.
fn released_and_reopened() -> Value {
    let save = plays_and_restores(HOLD, &["seee", "seef", "seeg", "dec", "pause", "go", "rel"]);
    assert_eq!(
        journal(&save),
        [
            "d@1 committed",
            "d@1 held e",
            "d@1 deferred f",
            "d@1 released g",
            "d@1 reopened f"
        ]
    );
    save
}

/// The finding's three forgeries: the series shows a hold, a deferred cause
/// or neither, and the journal's entry of it is gone.
#[test]
fn a_hold_a_deferral_and_a_release_each_have_their_entry() {
    refused(
        HOLD,
        &without(&held(), "held"),
        "decision series d holds e, which its journal does not",
    );
    refused(
        HOLD,
        &without(&deferring(), "deferred"),
        "decision series d defers f, which its journal does not",
    );
    refused(
        HOLD,
        &without(&released(), "released"),
        "decision series d does not hold e, which its journal does",
    );
    // And a reason added to a hold in force, which no entry names.
    refused(
        HOLD,
        &with_series(&held(), "on_hold", &["e", "f"]),
        "decision series d holds f, which its journal does not",
    );
}

/// And the other way: the journal holds or defers, and the series does not.
/// With the revision's `committed` entry departed the journal no longer says
/// all of the hold, but what its held entries hold, the series holds.
#[test]
fn a_series_holds_and_defers_what_its_journal_does() {
    refused(
        HOLD,
        &with_series(&held(), "on_hold", &[]),
        "decision series d does not hold e, which its journal does",
    );
    refused(
        HOLD,
        &with_series(&deferring(), "deferred", &[]),
        "decision series d does not defer f, which its journal does",
    );
    let script = ["seee", "seef", "seeg", "dec", "pause", "more", "go"];
    let source = windowed(3);
    let save = plays_and_restores(&source, &script);
    assert_eq!(
        journal(&save),
        ["d@1 held e", "d@1 held g", "d@1 deferred f"]
    );
    assert_eq!(save["journal_departed"], 1, "{save}");
    refused(
        &source,
        &with_series(&save, "on_hold", &["g"]),
        "decision series d does not hold e, which its journal does",
    );
    // Under a window of 1 the hold's own entries have departed, and the
    // series alone says what it holds.
    let source = windowed(1);
    let save = plays_and_restores(&source, &script);
    assert_eq!(journal(&save), ["d@1 deferred f"]);
    assert_eq!(save["decision_series"]["d"]["on_hold"], json!(["e", "g"]));
    // And with no entry of the revision left, the journal says nothing of it.
    let save = plays_and_restores(&source, &["seee", "dec", "pause", "plan"]);
    assert_eq!(journal(&save), ["act committed"]);
    assert_eq!(save["decision_series"]["d"]["on_hold"], json!(["e"]));
}

/// A windowed journal that still holds a revision's `committed` entry, or a
/// `released` one, says all of the hold from that entry on.
#[test]
fn a_windowed_journal_says_all_of_a_hold_from_a_commitment_or_a_release_on() {
    // `d@2` is committed in the window, and no entry holds it.
    let source = windowed(4);
    let save = plays_and_restores(
        &source,
        &["seee", "seef", "seeg", "dec", "pause", "both", "dec"],
    );
    assert_eq!(journal(&save)[3], "d@2 committed");
    assert_eq!(save["journal_departed"], 1, "{save}");
    refused(
        &source,
        &with_series(&save, "on_hold", &["e"]),
        "decision series d holds e, which its journal does not",
    );
    // `d@1` is released in the window, and the next entry defers a cause
    // where the runtime wrote the entry of a second hold.
    let source = windowed(3);
    let save = plays_and_restores(
        &source,
        &[
            "seee", "seef", "seeg", "dec", "pause", "rel", "pause", "idle",
        ],
    );
    assert_eq!(
        journal(&save),
        ["d@1 held e", "d@1 released g", "d@1 held e"]
    );
    assert_eq!(save["journal_departed"], 1, "{save}");
    let mut forged = with_series(&save, "deferred", &["f"]);
    let last = &mut forged["decision_journal"][2];
    last["change"] = json!("deferred");
    last["event"] = json!("go");
    last["because"] = json!(["f"]);
    refused(
        &source,
        &forged,
        "decision journal: d@1 has a deferred entry while not on hold",
    );
}

/// Only a revision on hold defers a reopening or is released, and a one-off
/// commitment is never on hold.
#[test]
fn a_deferral_and_a_release_are_of_a_hold() {
    refused(
        HOLD,
        &without(&deferring(), "held"),
        "decision journal: d@1 has a deferred entry while not on hold",
    );
    refused(
        HOLD,
        &without(&released(), "held"),
        "decision journal: d@1 has a released entry while not on hold",
    );
    let save = plays_and_restores(HOLD, &["seee", "seef", "plan", "idle", "redo"]);
    assert_eq!(journal(&save), ["act committed", "act reopened f"]);
    refused(
        HOLD,
        &with_entry(&save, 1, "deferred", ("redo", 4), &["e"]),
        "decision journal: act has a deferred entry while not on hold",
    );
    // Nor is it under a window that has let its `committed` entry depart.
    let source = windowed(1);
    let save = plays_and_restores(&source, &["seee", "seef", "plan", "idle", "redo", "idle"]);
    assert_eq!(journal(&save), ["act reopened f"]);
    let mut forged = save.clone();
    forged["decision_journal"][0]["change"] = json!("deferred");
    refused(
        &source,
        &forged,
        "decision journal: act has a deferred entry while not on hold",
    );
}

/// A reopening ends a hold, and a reopened revision takes none.
#[test]
fn a_reopened_revision_has_no_hold() {
    let save = plays_and_restores(HOLD, &["seee", "seef", "seeg", "dec", "go", "idle", "idle"]);
    assert_eq!(journal(&save), ["d@1 committed", "d@1 reopened f"]);
    for (change, event, cause) in [
        ("held", "pause", "e"),
        ("deferred", "go", "g"),
        ("released", "rel", "g"),
    ] {
        refused(
            HOLD,
            &with_entry(&save, 2, change, (event, 6), &[cause]),
            &format!("decision journal: d@1 has a {change} entry after its reopening"),
        );
    }
    refused(
        HOLD,
        &with_series(&save, "on_hold", &["e"]),
        "decision series d holds d@1, which is reopened",
    );
}

/// A `held` entry names the reasons its hold added, and a `deferred` entry
/// the causes its reopening deferred: none is named twice.
#[test]
fn a_hold_names_each_reason_and_each_deferred_cause_once() {
    refused(
        HOLD,
        &with_entry(&held(), 2, "held", ("pause", 3), &["e"]),
        "decision journal: d@1 is held because of e twice",
    );
    refused(
        HOLD,
        &with_entry(&deferring(), 3, "deferred", ("go", 5), &["f"]),
        "decision journal: d@1 defers f twice",
    );
}

/// A release reopens its revision because of what the hold deferred, at the
/// same event, and for nothing else.
#[test]
fn a_release_is_followed_by_the_reopening_it_deferred() {
    const UNFOLLOWED: &str =
        "decision journal: d@1 is released without the reopening its hold deferred";
    // The last entry is the release, of a hold that deferred a cause.
    let quiet = plays_and_restores(HOLD, &["seee", "seef", "dec", "pause", "seeg", "rel"]);
    assert_eq!(
        journal(&quiet),
        ["d@1 committed", "d@1 held e", "d@1 released g"]
    );
    refused(
        HOLD,
        &with_entry(&quiet, 2, "deferred", ("go", 5), &["f"]),
        UNFOLLOWED,
    );
    // The reopening after the release cites one deferred cause of two.
    let save = released_and_reopened();
    refused(
        HOLD,
        &with_entry(&save, 3, "deferred", ("go", 6), &["e"]),
        UNFOLLOWED,
    );
    // The reopening cites a cause the hold did not defer.
    let twice = plays_and_restores(
        HOLD,
        &["seee", "seef", "seeg", "dec", "pause", "go", "also", "rel"],
    );
    assert_eq!(
        journal(&twice),
        [
            "d@1 committed",
            "d@1 held e",
            "d@1 deferred f",
            "d@1 deferred g",
            "d@1 released g",
            "d@1 reopened f g"
        ]
    );
    refused(HOLD, &without(&twice, "deferred"), UNFOLLOWED);
    // The reopening cites another cause than the one the hold deferred.
    let mut forged = save.clone();
    forged["decision_journal"][2]["because"] = json!(["g"]);
    refused(HOLD, &forged, UNFOLLOWED);
    // The reopening is of a later event than the release.
    let late = plays_and_restores(
        HOLD,
        &[
            "seee", "seef", "seeg", "dec", "pause", "go", "rel", "idle", "idle",
        ],
    );
    assert_eq!(journal(&late), journal(&save));
    let mut forged = late.clone();
    forged["decision_journal"][4]["event"] = json!("go");
    forged["decision_journal"][4]["sequence"] = json!(8);
    refused(HOLD, &forged, UNFOLLOWED);
    // The entry after the release reopens another commitment, at the same
    // event and because of what the hold deferred.
    let apart = plays_and_restores(
        HOLD,
        &[
            "seee", "seef", "seeg", "plan", "dec", "pause", "idle", "apart",
        ],
    );
    assert_eq!(
        journal(&apart),
        [
            "act committed",
            "d@1 committed",
            "d@1 held e",
            "d@1 released g",
            "act reopened f"
        ]
    );
    refused(
        HOLD,
        &with_entry(&apart, 3, "deferred", ("go", 7), &["f"]),
        UNFOLLOWED,
    );
    // The hold deferred nothing, and `rel` reopens nothing of its own.
    const ONLY_RELEASES: &str = "decision journal: d@1 is reopened by rel, which only releases \
                                 it, and not because of what its hold deferred";
    refused(HOLD, &without(&save, "deferred"), ONLY_RELEASES);
    // `both` does: its release defers nothing, and its `reopen` reopens.
    let save = plays_and_restores(HOLD, &["seee", "seef", "seeg", "dec", "pause", "both"]);
    assert_eq!(
        journal(&save),
        [
            "d@1 committed",
            "d@1 held e",
            "d@1 released g",
            "d@1 reopened f"
        ]
    );
    // Under a window of 1 the `released` entry departs as its reopening is
    // journaled, and `rel`'s reopening is the journal's first entry.
    let save = plays_and_restores(
        &windowed(1),
        &["seee", "seef", "seeg", "dec", "pause", "go", "rel"],
    );
    assert_eq!(journal(&save), ["d@1 reopened f"]);
    // Only the first entry may have lost its release: under a window of 2
    // the entry before `rel`'s reopening is in the journal.
    let source = windowed(2);
    let save = plays_and_restores(
        &source,
        &[
            "seee", "seef", "seeg", "dec", "pause", "go", "idle", "rel", "idle",
        ],
    );
    assert_eq!(journal(&save), ["d@1 released g", "d@1 reopened f"]);
    let mut forged = save.clone();
    let first = &mut forged["decision_journal"][0];
    first["change"] = json!("held");
    first["event"] = json!("more");
    first["sequence"] = json!(7);
    refused(&source, &forged, ONLY_RELEASES);
}

/// A program whose `retract` withdraws what `d` rests on: `again` then
/// reopens `d` through a hold.
const WITHDRAW: &str = "claim safe;
evidence e from \"e\";
evidence f from \"f\";
evidence g from \"g\";
evidence bad from \"bad\";
decisions d limit 4;
event seee;
event seef;
event seeg;
event seebad;
event dec;
event pause;
event go;
event again;
event retract;
event idle;
on seee reveal e supports safe;
on seef reveal f supports safe;
on seeg reveal g supports safe;
on seebad reveal bad supports safe;
on dec when not committed(d) or reopened(d) commit d because enough using qualified(1, e);
on pause hold d because f;
on go reopen d because g;
on again reopen d because bad;
on retract withdraw e because bad;
";

/// A hold ends at a release, or when a `reopen` breaks through it for a
/// withdrawal, which cites every deferred cause.
#[test]
fn a_reopening_ends_a_hold_only_after_a_release_or_for_a_withdrawal() {
    // `rel` only releases: its reopening follows its `released` entry.
    refused(
        HOLD,
        &without(&released_and_reopened(), "released"),
        "decision journal: d@1 is reopened by rel, which only releases it, and not because of \
         what its hold deferred",
    );
    // `both` reopens too, but a `reopen` of a revision on hold is deferred
    // in a program that withdraws nothing.
    let save = plays_and_restores(HOLD, &["seee", "seef", "seeg", "dec", "pause", "both"]);
    refused(
        HOLD,
        &without(&save, "released"),
        "decision journal: d@1 is reopened while on hold, with no release, in a program that \
         withdraws nothing",
    );
    // A withdrawal breaks through, with nothing deferred and with a cause.
    let events = ["seee", "seef", "seeg", "seebad", "dec", "pause"];
    let save = plays_and_restores(WITHDRAW, &[&events[..], &["retract", "again"]].concat());
    assert_eq!(
        journal(&save),
        ["d@1 committed e", "d@1 held f", "d@1 reopened bad"]
    );
    let save = plays_and_restores(
        WITHDRAW,
        &[&events[..], &["go", "idle", "retract", "again"]].concat(),
    );
    assert_eq!(
        journal(&save),
        [
            "d@1 committed e",
            "d@1 held f",
            "d@1 deferred g",
            "d@1 reopened g bad"
        ]
    );
    refused(
        WITHDRAW,
        &with_entry(&save, 3, "deferred", ("go", 8), &["f"]),
        "decision journal: d@1 is reopened without f, which its hold deferred",
    );
}

/// A series takes its next revision once the current one is reopened, which
/// ends its hold. The journal's window holds the last four entries, so the
/// two put in place of the release and its reopening keep their count.
#[test]
fn no_revision_follows_one_on_hold() {
    let source = windowed(4);
    let save = plays_and_restores(
        &source,
        &["seee", "seef", "seeg", "dec", "pause", "both", "dec"],
    );
    assert_eq!(
        journal(&save),
        [
            "d@1 held e",
            "d@1 released g",
            "d@1 reopened f",
            "d@2 committed"
        ]
    );
    let mut forged = without(&without(&save, "released"), "reopened");
    for cause in ["g", "f"] {
        forged = with_entry(&forged, 1, "deferred", ("both", 6), &[cause]);
    }
    assert_eq!(
        journal(&forged),
        [
            "d@1 held e",
            "d@1 deferred f",
            "d@1 deferred g",
            "d@2 committed"
        ]
    );
    refused(
        &source,
        &forged,
        "decision journal: d@2 is committed while d@1 is on hold",
    );
}

/// Nor while it is in force. A release with nothing deferred leaves its
/// revision in force, and only a reopening, which journals its entry after
/// every other entry of the revision, lets the series take the next one.
/// With that entry and the graph's reopening taken out, the hold's entries
/// do not stand in for them.
#[test]
fn no_revision_follows_one_in_force() {
    let save = plays_and_restores(
        HOLD,
        &["seee", "seef", "seeg", "dec", "pause", "rel", "go", "dec"],
    );
    assert_eq!(
        journal(&save),
        [
            "d@1 committed",
            "d@1 held e",
            "d@1 released g",
            "d@1 reopened f",
            "d@2 committed"
        ]
    );
    assert_eq!(save["graph"]["open"], json!(["d@1"]), "{save}");
    let mut forged = without(&save, "reopened");
    forged["graph"]["open"] = json!([]);
    forged["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .retain(|relation| relation[1] != "reopens");
    refused(
        HOLD,
        &forged,
        "decision journal: d@2 is committed while d@1 is in force",
    );
}

/// A hold and its release each join the revision's reopening record. With
/// both entries gone the series and the journal agree, and the record is
/// left with nothing that made it.
#[test]
fn a_reopening_record_has_a_journal_entry() {
    let save = released();
    assert_eq!(
        save["reopening_qualifications"],
        json!({"d@1": {"evidence": ["e", "g"]}})
    );
    refused(
        HOLD,
        &without(&without(&save, "held"), "released"),
        "reopening of d@1: no journal entry reopens, holds, defers or releases it",
    );
    // A windowed journal that still holds a revision's `committed` entry
    // holds every entry of the revision.
    let source = windowed(4);
    let save = plays_and_restores(
        &source,
        &["seee", "seef", "seeg", "dec", "pause", "both", "dec"],
    );
    assert_eq!(journal(&save)[3], "d@2 committed");
    assert_eq!(save["journal_departed"], 1, "{save}");
    let mut forged = save.clone();
    forged["reopening_qualifications"]["d@2"] = save["reopening_qualifications"]["d@1"].clone();
    refused(
        &source,
        &forged,
        "reopening of d@2: no journal entry reopens, holds, defers or releases it",
    );
}

/// And each `held`, `deferred`, `released` or `reopened` entry joins its
/// revision's reopening record with the lineage of every cause it cites,
/// which it pins: the record holds each of them, whatever has departed.
/// What reads the hold reads its grounds from that record.
#[test]
fn a_journal_entry_has_its_reopening_record() {
    let save = deferring();
    assert_eq!(
        save["reopening_qualifications"],
        json!({"d@1": {"evidence": ["e", "f"]}})
    );
    let mut forged = save.clone();
    forged
        .as_object_mut()
        .unwrap()
        .remove("reopening_qualifications");
    refused(
        HOLD,
        &forged,
        "reopening of d@1: its held entry has no record",
    );
    forged["reopening_qualifications"] = json!({"d@1": {}});
    refused(
        HOLD,
        &forged,
        "reopening of d@1: its record leaves out e, which its held entry cites",
    );
    forged["reopening_qualifications"] = json!({"d@1": {"evidence": ["e"]}});
    refused(
        HOLD,
        &forged,
        "reopening of d@1: its record leaves out f, which its deferred entry cites",
    );
    // Under a window that has let the revision's `committed` entry depart.
    let source = windowed(2);
    let save = plays_and_restores(&source, &["seee", "seeg", "dec", "pause", "rel", "idle"]);
    assert_eq!(journal(&save), ["d@1 held e", "d@1 released g"]);
    assert_eq!(
        save["reopening_qualifications"],
        json!({"d@1": {"evidence": ["e", "g"]}})
    );
    let mut forged = save.clone();
    forged["reopening_qualifications"] = json!({"d@1": {"evidence": ["e"]}});
    refused(
        &source,
        &forged,
        "reopening of d@1: its record leaves out g, which its released entry cites",
    );
}

/// A series no rule or procedure holds is never on hold. A window that has
/// let its revision's `committed` entry depart leaves no part of a hold
/// unsaid, as for a one-off commitment, and its first `reopened` entry is
/// no release's.
#[test]
fn a_series_nothing_holds_has_no_hold_under_a_window() {
    let unheld = HOLD
        .replace("on pause hold d because e;\n", "")
        .replace("on more hold d because g;\n", "");
    assert!(!unheld.contains(" hold d "), "{unheld}");
    let source = format!("{unheld}journal window 1;\n");
    let save = plays_and_restores(&source, &["seee", "seef", "seeg", "dec", "go", "idle"]);
    assert_eq!(journal(&save), ["d@1 reopened f"]);
    let mut forged = save.clone();
    forged["decision_journal"][0]["event"] = json!("rel");
    refused(
        &source,
        &forged,
        "decision journal: d@1 is reopened by rel, which only releases it, and not because of \
         what its hold deferred",
    );
    // Where `pause` holds `d`, the same entry may be the reopening of a
    // release whose entry departed.
    let held = windowed(1);
    let save = plays_and_restores(&held, &["seee", "seef", "seeg", "dec", "go", "idle"]);
    let mut forged = save.clone();
    forged["decision_journal"][0]["event"] = json!("rel");
    assert!(ReactiveSession::restore_json(&held, &forged.to_string()).is_ok());
    // A deferral or a release of d@1, in force, as the window's only entry.
    let save = plays_and_restores(&source, &["seee", "seef", "seeg", "dec", "plan", "idle"]);
    assert_eq!(journal(&save), ["act committed"]);
    for (change, event, cause) in [("deferred", "go", "f"), ("released", "rel", "g")] {
        let mut forged = save.clone();
        let entry = &mut forged["decision_journal"][0];
        entry["decision"] = json!("d");
        entry["commitment"] = json!("d@1");
        entry["change"] = json!(change);
        entry["event"] = json!(event);
        entry["because"] = json!([cause]);
        refused(
            &source,
            &forged,
            &format!("decision journal: d@1 has a {change} entry while not on hold"),
        );
    }
}

/// Every reopening journals a `reopened` entry, and no entry of its
/// revision follows it. So the entries the journal holds of a revision the
/// graph shows reopened end with that one.
#[test]
fn a_reopened_revision_has_its_entries_end_with_its_reopening() {
    let source = windowed(1);
    let events = ["seee", "seef", "seeg", "dec", "pause", "go", "rel"];
    let save = plays_and_restores(&source, &[&events[..], &["idle"]].concat());
    assert_eq!(journal(&save), ["d@1 reopened f"]);
    let release = |save: &Value| {
        let mut forged = save.clone();
        forged["decision_journal"][0]["change"] = json!("released");
        forged["decision_journal"][0]["because"] = json!(["g"]);
        forged
    };
    const IN_FORCE: &str = "decision journal: d@1 is reopened, but its entries leave it in force";
    refused(&source, &release(&save), IN_FORCE);
    // And of the one before a revision committed in the window.
    let source = windowed(2);
    let save = plays_and_restores(&source, &[&events[..], &["dec"]].concat());
    assert_eq!(journal(&save), ["d@1 reopened f", "d@2 committed"]);
    refused(&source, &release(&save), IN_FORCE);
}

/// Two series that hold, defer, release and reopen, one of them by a
/// trigger, with a withdrawal that breaks through. A hold's reasons and the
/// causes it defers include a reading and renewable evidence, which retire
/// while the hold pins them. `redop` and `settle` reopen, release and hold
/// from procedure steps. `mixh`, `mixr` and `mixg` hold, release and reopen
/// `d` and are then refused, since `never` has no revision: the session
/// takes back what their first effect did.
const MIXED: &str = "claim safe;
evidence e from \"e\";
evidence f from \"f\";
evidence g from \"g\";
evidence bad from \"bad\";
evidence tool from \"tool\";
evidence probe from \"probe\";
renewable probe window 2;
decisions d limit 255 reopened by checks;
decisions other limit 255;
decisions never limit 4;
proc redo(cause evidence) {
    when true reopen d because cause;
};
proc settle(cause evidence) {
    when true release d because cause;
    when true hold other because cause;
};
event seee;
event seef;
event seeg;
event seebad;
event check v min 0 max 9;
event dec;
event dec2;
event pause;
event more;
event go;
event again;
event rel;
event both;
event retract;
event hold2;
event redo2;
event rel2;
event regrow;
event holdp;
event holdr;
event gop;
event redop;
event settle;
event mixh;
event mixr;
event mixg;
event idle;
on seee when not observed(e) reveal e supports safe;
on seef when not observed(f) reveal f supports safe;
on seeg when not observed(g) reveal g supports safe;
on seebad when not observed(bad) reveal bad supports safe;
on check sample checks = v supports safe;
on dec when (not committed(d) or reopened(d)) and has_sample(checks) commit d because enough using latest(checks);
on dec2 when not committed(other) or reopened(other) commit other because enough using 1;
on pause hold d because e;
on more hold d because g;
on go reopen d because f;
on again reopen d because bad;
on rel release d because g;
on both release d because g;
on both reopen d because f;
on retract when observed(bad) and has_sample(checks) and not withdrawn(latest(checks)) withdraw latest(checks) because bad;
on hold2 hold other because f;
on redo2 reopen other because g;
on rel2 release other because e;
on regrow renew probe;
on regrow reveal probe supports safe;
on holdp when observed(probe) hold d because probe;
on holdr when has_sample(checks) hold d because latest(checks);
on gop when observed(probe) reopen d because probe;
on redop when observed(f) call redo(f);
on settle when observed(g) call settle(g);
on mixh hold d because e;
on mixh hold never because e;
on mixr release d because g;
on mixr reopen never because g;
on mixg reopen d because f;
on mixg hold never because f;
";

/// Sessions of `MIXED` under each journal window, and with its readings
/// windowed too: every save restores and saves the same bytes again, and a
/// session restored on the way plays on as the live one does.
#[test]
fn sessions_that_hold_restore_under_every_journal_window() {
    let events: [(&str, &str); 36] = [
        ("seee", "{}"),
        ("seef", "{}"),
        ("seeg", "{}"),
        ("seebad", "{}"),
        ("check", r#"{"v":1}"#),
        ("check", r#"{"v":2}"#),
        ("dec", "{}"),
        ("dec", "{}"),
        ("dec", "{}"),
        ("dec", "{}"),
        ("dec2", "{}"),
        ("pause", "{}"),
        ("pause", "{}"),
        ("pause", "{}"),
        ("more", "{}"),
        ("more", "{}"),
        ("go", "{}"),
        ("go", "{}"),
        ("again", "{}"),
        ("rel", "{}"),
        ("rel", "{}"),
        ("both", "{}"),
        ("retract", "{}"),
        ("hold2", "{}"),
        ("redo2", "{}"),
        ("rel2", "{}"),
        ("regrow", "{}"),
        ("regrow", "{}"),
        ("holdp", "{}"),
        ("holdr", "{}"),
        ("gop", "{}"),
        ("redop", "{}"),
        ("settle", "{}"),
        ("mixh", "{}"),
        ("mixr", "{}"),
        ("mixg", "{}"),
    ];
    // By change, the entries journaled; then the saves that hold with their
    // revision's `committed` entry departed, the reopenings that ended a
    // hold with no release, and the events refused after an effect on `d`.
    let mut made = std::collections::BTreeMap::<String, usize>::new();
    let mut partial = 0;
    let mut through = 0;
    let mut undone = 0;
    let variants: [(&str, Option<usize>); 8] = [
        ("limit 255", None),
        ("limit 255", Some(1)),
        ("limit 255", Some(2)),
        ("limit 255", Some(3)),
        ("limit 255", Some(5)),
        ("limit 255", Some(9)),
        ("window 2", None),
        ("window 2", Some(4)),
    ];
    for (index, (readings, window)) in variants.iter().enumerate() {
        let mut source = format!("{MIXED}readings checks from tool {readings};\n");
        if let Some(window) = window {
            source.push_str(&format!("journal window {window};\n"));
        }
        for seed in 1..=2u64 {
            let mut state = (seed + 16 * index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut live = ReactiveSession::from_source(&source).unwrap();
            let mut twin = ReactiveSession::from_source(&source).unwrap();
            let mut sequence = 0;
            let mut on_hold = false;
            let mut revised = false;
            for step in 0..200 {
                // A 64-bit linear congruential step (Knuth's MMIX constants).
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let (event, payload) = events[(state >> 33) as usize % events.len()];
                let at = format!("program {index} seed {seed} step {step} {event}");
                let answer = send(&mut live, event, payload);
                assert_eq!(send(&mut twin, event, payload), answer, "{at}");
                let save = live.save_json().unwrap();
                assert_eq!(twin.save_json().unwrap(), save, "{at}");
                let again = ReactiveSession::restore_json(&source, &save)
                    .unwrap_or_else(|error| panic!("{at}: {error}\n{save}"));
                assert_eq!(again.save_json().unwrap(), save, "{at}");
                if step % 7 == 0 {
                    twin = again;
                }
                let save: Value = serde_json::from_str(&save).unwrap();
                // A refused event leaves the session, and its sequence, as
                // it was.
                let after = save["sequence"].as_u64().unwrap();
                if after == sequence {
                    if revised && event.starts_with("mix") {
                        undone += 1;
                    }
                    continue;
                }
                sequence = after;
                let entries = save["decision_journal"].as_array();
                let new = entries
                    .into_iter()
                    .flatten()
                    .filter(|entry| entry["sequence"] == after && entry["decision"] == "d")
                    .map(|entry| entry["change"].as_str().unwrap())
                    .collect::<Vec<_>>();
                for change in &new {
                    *made.entry(change.to_string()).or_default() += 1;
                }
                if on_hold && new.contains(&"reopened") && !new.contains(&"released") {
                    through += 1;
                }
                let series = &save["decision_series"]["d"];
                on_hold = !series["on_hold"].is_null();
                let current = series["current"].as_str().unwrap_or_default();
                revised = !current.is_empty();
                let committed = entries
                    .into_iter()
                    .flatten()
                    .any(|entry| entry["commitment"] == current && entry["change"] == "committed");
                if on_hold && !committed {
                    partial += 1;
                }
            }
        }
    }
    // The sessions did commit, hold, defer, release and reopen, and break
    // through a hold; many saves hold with less than all of the hold's
    // entries; and many events were refused after an effect on `d`.
    let count = |change: &str| made.get(change).copied().unwrap_or(0);
    let met = [
        count("committed"),
        count("held"),
        count("deferred"),
        count("released"),
        count("reopened"),
        through,
        partial,
        undone,
    ];
    let least = [150, 120, 60, 40, 220, 25, 200, 180];
    assert!(
        met.iter().zip(least).all(|(met, least)| *met >= least),
        "{met:?}"
    );
}
