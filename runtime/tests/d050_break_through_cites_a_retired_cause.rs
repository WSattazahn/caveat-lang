//! D050: restore refused the save written after a reopening broke through a
//! hold, when a cause the hold had deferred retired while it waited: "d@1
//! reopened after runs@1 retired reads it". A hold never keeps in force a
//! revision that rests on withdrawn evidence: a `reopen` of it goes ahead,
//! because of its own causes and every deferred one, and the hold kept each
//! deferred cause from departing, not from retiring. Restore let a
//! reopening cite a record that had retired only when its event could
//! release the series. It now lets a reopening of a series the program can
//! hold do so when the program can withdraw, since only a withdrawal breaks
//! through a hold. See spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

fn send(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    outcome
}

/// Plays `events`. After each, the save restores and saves the same bytes
/// again, and every session restored so far answers the events still to
/// come as the live one does. Returns the last save.
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

/// `d` is committed on runs@1, then held; `reopen` defers runs@1, which
/// retires at the second reading after. `withdrawit` withdraws `e`, which
/// the basis holds, so the next `reopen` breaks through the hold.
const HELD: &str = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
evidence corr from \"corr\";
readings runs from feed window 2;
decisions d limit 2;
event go;
event reading v min 0 max 9;
event reopen;
event hold;
event withdrawit;
on go reveal e supports c;
on go commit d because enough using qualified(latest(runs), e);
on reading sample runs = v;
on reopen when committed(d) and not reopened(d) reopen d because latest(runs);
HOLD
on withdrawit reveal corr;
on withdrawit withdraw e because corr;
";

const EVENTS: [(&str, &str); 8] = [
    ("reading", r#"{"v":0}"#),
    ("go", "{}"),
    ("hold", "{}"),
    ("reopen", "{}"),
    ("reading", r#"{"v":0}"#),
    ("reading", r#"{"v":0}"#),
    ("withdrawit", "{}"),
    ("reopen", "{}"),
];

#[test]
fn a_reopening_that_breaks_through_a_hold_restores() {
    let source = HELD.replace("HOLD", "on hold hold d because e;");
    let save = plays_and_restores(&source, &EVENTS);
    let journal = save["decision_journal"].as_array().unwrap();
    let changes: Vec<&str> = journal
        .iter()
        .map(|entry| entry["change"].as_str().unwrap())
        .collect();
    assert_eq!(changes, ["committed", "held", "deferred", "reopened"]);
    let last = journal.last().unwrap();
    // Its own cause and the deferred one, in the order they were observed.
    assert_eq!(last["event"], "reopen");
    assert_eq!(last["because"], serde_json::json!(["runs@1", "runs@3"]));
    // runs@1 retired before the reopening that cites it.
    let retired = save["retired"]["runs@1"].as_u64().unwrap();
    assert!(retired < last["sequence"].as_u64().unwrap(), "{save}");
}

/// The same with a trigger: each reading reopens `d`, so while it is held
/// each one is deferred, and runs@2 retires while it waits. The reading
/// after `withdrawit` breaks through.
const TRIGGERED: &str = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
evidence corr from \"corr\";
readings runs from feed window 2;
decisions d limit 2 reopened by runs;
event go;
event reading v min 0 max 9;
event hold;
event withdrawit;
on go reveal e supports c;
on go commit d because enough using qualified(latest(runs), e);
on reading sample runs = v;
on hold hold d because e;
on withdrawit reveal corr;
on withdrawit withdraw e because corr;
";

#[test]
fn a_trigger_that_breaks_through_a_hold_restores() {
    let reading = ("reading", r#"{"v":0}"#);
    let save = plays_and_restores(
        TRIGGERED,
        &[
            reading,
            ("go", "{}"),
            ("hold", "{}"),
            reading,
            reading,
            reading,
            ("withdrawit", "{}"),
            reading,
        ],
    );
    let last = save["decision_journal"].as_array().unwrap().last().unwrap();
    assert_eq!(last["change"], "reopened", "{save}");
    let sequence = last["sequence"].as_u64().unwrap();
    let cited_retired = last["because"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|name| {
            save["retired"][name.as_str().unwrap()]
                .as_u64()
                .is_some_and(|at| at < sequence)
        })
        .count();
    assert!(cited_retired > 0, "{save}");
}

#[test]
fn a_reopening_of_a_series_no_rule_holds_still_may_not_cite_a_retired_record() {
    // Without the hold, `reopen` reopens `d` because of runs@1, which
    // retires later; `hold`, which does nothing here, comes last so that no
    // retirement is the last event's. A save that says runs@1 retired before
    // the reopening is refused.
    let source = HELD.replace("HOLD", "");
    let events = [&EVENTS[..6], &[("hold", "{}")]].concat();
    let save = plays_and_restores(&source, &events);
    refuses_runs_1_retired_before_its_reopening(&source, &save);
}

/// Moves runs@1's retirement before the reopening that cites it, which no
/// session of `source` writes, and checks restore refuses the save.
fn refuses_runs_1_retired_before_its_reopening(source: &str, save: &Value) {
    let reopened = save["decision_journal"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["change"] == "reopened")
        .unwrap();
    assert_eq!(reopened["because"], serde_json::json!(["runs@1"]));
    let mut forged = save.clone();
    let before = reopened["sequence"].as_u64().unwrap() - 1;
    forged["retired"]["runs@1"] = before.into();
    match ReactiveSession::restore_json(source, &forged.to_string()) {
        Err(error) => assert_eq!(
            error,
            "cannot restore save: d@1 reopened after runs@1 retired reads it"
        ),
        Ok(_) => panic!("accepted a reopening that cites a record retired before it"),
    }
}

#[test]
fn a_hold_of_another_series_does_not_excuse_this_one() {
    // `g` can be held and `d` cannot, so d's reopening still may not cite a
    // record that retired before it.
    let source = HELD.replace("HOLD", "").replace(
        "decisions d limit 2;",
        "decisions d limit 2;\ndecisions g limit 2;\nevent hg;\non hg hold g because e;",
    );
    let events = [&EVENTS[..6], &[("hold", "{}")]].concat();
    let save = plays_and_restores(&source, &events);
    refuses_runs_1_retired_before_its_reopening(&source, &save);
}

#[test]
fn a_hold_in_a_program_that_cannot_withdraw_does_not_excuse_it() {
    // `d` can be held, but nothing withdraws, so no reopening of it can
    // break through a hold; only a release could cite a deferred cause.
    // `hold` comes after the reopening, so it does nothing.
    let source = HELD
        .replace("HOLD", "on hold hold d because e;")
        .replace("on withdrawit withdraw e because corr;\n", "");
    let reading = ("reading", r#"{"v":0}"#);
    let events = [
        reading,
        ("go", "{}"),
        ("reopen", "{}"),
        reading,
        reading,
        ("hold", "{}"),
    ];
    let save = plays_and_restores(&source, &events);
    refuses_runs_1_retired_before_its_reopening(&source, &save);
}

/// A rule reopens `d` because of latest(runs) at a reading whose sample,
/// later in the same event, retires runs@1 (window 1).
const SAME_EVENT: &str = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
readings runs from feed window 1;
decisions d limit 2;
event go;
event reading v min 0 max 9;
event noop;
on go reveal e supports c;
on go commit d because enough using qualified(latest(runs), e);
on reading when committed(d) and not reopened(d) reopen d because latest(runs);
on reading sample runs = v;
on noop reveal e;
";

#[test]
fn a_reopening_at_the_event_its_cause_retires_restores() {
    // The reopening is at the event its cause retires, not after it, so the
    // save restores. `noop` keeps the retirement out of the last event.
    let reading = ("reading", r#"{"v":0}"#);
    let save = plays_and_restores(
        SAME_EVENT,
        &[reading, ("go", "{}"), reading, ("noop", "{}")],
    );
    let reopened = save["decision_journal"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["change"] == "reopened")
        .unwrap();
    assert_eq!(reopened["because"], serde_json::json!(["runs@1"]));
    assert_eq!(save["retired"]["runs@1"], reopened["sequence"], "{save}");
}
