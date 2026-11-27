//! D055: restore accepted a save in which the last event's reopening of a
//! series names a cause that event could not name, whenever the program can
//! hold the series. A reopening names causes deferred by earlier events only
//! when it ends a hold: at a release, or when it breaks through the hold for
//! a withdrawal (Holds 0.1). Restore let a reopening of any series the
//! program can hold, by a rule or a trigger, name any cause. So in a program
//! that holds a series but cannot withdraw, a save could have the last
//! event's reopening name evidence only another event reopens for, and it
//! restored though no session writes it. Restore now lets such a reopening
//! name a cause its own selector does not select only when the program can
//! withdraw; a release reports what it reopens for as its own effect. The
//! saves sessions write restore as before. See spec/caveat-save-0.1.md
//! ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// `go` commits `d` on the first reading and `e`, and `seef` observes `f`.
/// `reopen` reopens `d` because the latest reading, or with `trigger` each
/// reading does, and `other` reopens it because `f`, so while `d` is held
/// `other` defers `f`. `extra` adds what holds, releases or withdraws.
fn program(trigger: bool, extra: &str) -> String {
    let triggered = if trigger { " reopened by runs" } else { "" };
    format!(
        "claim c;
evidence e from \"e\";
evidence f from \"f\";
evidence corr from \"corr\";
evidence feed from \"feed\";
readings runs from feed limit 8;
decisions d limit 2{triggered};
event go;
event seef;
event reading v min 0 max 9;
event reopen;
event other;
event hold;
event release;
event withdrawit;
on go reveal e supports c;
on go commit d because enough using qualified(latest(runs), e);
on seef reveal f supports c;
on reading sample runs = v;
on reopen when committed(d) and not reopened(d) reopen d because latest(runs);
on other when committed(d) and not reopened(d) reopen d because f;
{extra}"
    )
}

const HOLD: &str = "on hold hold d because e;\n";
const RELEASE: &str = "on release release d because e;\n";
const WITHDRAW: &str = "on withdrawit reveal corr;\non withdrawit withdraw e because corr;\n";

fn send(session: &mut ReactiveSession, event: &str) -> Value {
    let payload = if event == "reading" {
        r#"{"v":0}"#
    } else {
        "{}"
    };
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
fn plays_and_restores(source: &str, events: &[&str]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    for (index, event) in events.iter().enumerate() {
        let answer = send(&mut live, event);
        for run in &mut restored {
            assert_eq!(send(run, event), answer, "{index}: {event}");
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

/// The last journal entry, checked to be d@1's reopening at `event`
/// because of `causes`.
fn reopened_at(save: &Value, event: &str, causes: &[&str]) {
    let last = save["decision_journal"].as_array().unwrap().last().unwrap();
    assert_eq!(last["change"], "reopened", "{save}");
    assert_eq!(last["commitment"], "d@1", "{save}");
    assert_eq!(last["event"], event, "{save}");
    assert_eq!(last["sequence"], save["sequence"], "{save}");
    assert_eq!(last["because"], json!(causes), "{save}");
}

/// `save` with `f` added to the causes of the last event's reopening of
/// d@1, as a hold that deferred `f` would leave it when the reopening ends
/// the hold: in its journal entry, its `reopens` relations and its effects,
/// each in observation order, and in its reopening record.
fn naming_f_too(save: &Value) -> Value {
    let mut forged = save.clone();
    let rank = |name: &str| {
        save["observations"]
            .as_array()
            .unwrap()
            .iter()
            .position(|observed| observed == name)
            .unwrap()
    };
    let entry = forged["decision_journal"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap();
    let mut causes: Vec<String> = entry["because"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap().to_string())
        .chain(["f".to_string()])
        .collect();
    causes.sort_by_key(|name| rank(name));
    entry["because"] = json!(causes);
    let reopens = |item: &Value| item[1] == "reopens" && item[2] == "d@1";
    let relations = forged["graph"]["relations"].as_array_mut().unwrap();
    let at = relations.iter().position(reopens).unwrap();
    relations.retain(|relation| !reopens(relation));
    for (offset, cause) in causes.iter().enumerate() {
        relations.insert(at + offset, json!([cause, "reopens", "d@1"]));
    }
    let reported = |item: &Value| item["kind"] == "reopen" && item["action"] == "d@1";
    let effects = forged["effects"].as_array_mut().unwrap();
    let at = effects.iter().position(reported).unwrap();
    effects.retain(|effect| !reported(effect));
    for (offset, cause) in causes.iter().enumerate() {
        effects.insert(
            at + offset,
            json!({"action": "d@1", "because": cause, "kind": "reopen"}),
        );
    }
    let record = forged["reopening_qualifications"]["d@1"]["evidence"]
        .as_array_mut()
        .unwrap();
    record.push("f".into());
    record.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
    forged
}

fn refused(source: &str, save: &Value) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(
            error,
            "cannot restore save: reopen effect is not one the last event can make"
        ),
        Ok(_) => panic!("accepted a reopening that names a cause its event cannot: {save}"),
    }
}

/// Each program, with the save after its last event reopens d@1 because
/// of runs@2, `f` observed before: by the rule at `reopen`, and by the
/// trigger at a reading.
fn last_event_reopenings(extra: &str) -> [(String, Value); 2] {
    let rule = program(false, extra);
    let save = plays_and_restores(&rule, &["reading", "go", "seef", "reading", "reopen"]);
    reopened_at(&save, "reopen", &["runs@2"]);
    let trigger = program(true, extra);
    let triggered = plays_and_restores(&trigger, &["reading", "go", "seef", "reading"]);
    reopened_at(&triggered, "reading", &["runs@2"]);
    [(rule, save), (trigger, triggered)]
}

#[test]
fn a_hold_alone_lets_no_reopening_name_a_cause_its_event_cannot() {
    // Nothing releases `d` or withdraws, so no reopening ends a hold: each
    // names only what its own selector selects, here the latest reading.
    for (source, save) in last_event_reopenings(HOLD) {
        refused(&source, &naming_f_too(&save));
    }
}

#[test]
fn a_release_at_another_event_does_not_excuse_it() {
    // `release` can end the hold, but the last event cannot release.
    for (source, save) in last_event_reopenings(&format!("{HOLD}{RELEASE}")) {
        refused(&source, &naming_f_too(&save));
    }
}

#[test]
fn a_program_that_holds_nothing_refuses_it_as_before() {
    // The same refusal, with the same message, as before D055, whether or
    // not the program withdraws: no hold, so nothing to break through.
    for extra in ["", WITHDRAW] {
        for (source, save) in last_event_reopenings(extra) {
            refused(&source, &naming_f_too(&save));
        }
    }
}

#[test]
fn a_release_reopens_for_what_its_hold_deferred() {
    // `other` defers `f` while `d` is held, and the release reopens `d`
    // because of it, though `release` selects only `e`.
    let source = program(false, &format!("{HOLD}{RELEASE}"));
    let save = plays_and_restores(
        &source,
        &["reading", "go", "seef", "hold", "other", "release"],
    );
    reopened_at(&save, "release", &["f"]);
}

#[test]
fn a_break_through_reopens_for_what_its_hold_deferred() {
    // `other` defers `f` while `d` is held, and `withdrawit` withdraws `e`,
    // which d@1 rests on, so the next reopening breaks through the hold
    // because of its own cause and `f`, which its selector does not select.
    let extra = format!("{HOLD}{WITHDRAW}");
    let rule = program(false, &extra);
    let save = plays_and_restores(
        &rule,
        &[
            "reading",
            "go",
            "seef",
            "hold",
            "other",
            "withdrawit",
            "reading",
            "reopen",
        ],
    );
    reopened_at(&save, "reopen", &["f", "runs@2"]);
    let trigger = program(true, &extra);
    let save = plays_and_restores(
        &trigger,
        &[
            "reading",
            "go",
            "seef",
            "hold",
            "other",
            "withdrawit",
            "reading",
        ],
    );
    reopened_at(&save, "reading", &["f", "runs@2"]);
}
