//! D040: restore refused a save the runtime itself wrote, with "cannot
//! restore save: retire effect is not one the last event can make", when the
//! session's last event resolved a decision while its `journal window` was
//! full. A resolve adds an entry to the decision journal (Outcomes 0.1), and
//! a full window retires its oldest entry first (Windows 0.1), but restore
//! accepted the retirement of a journal entry only from an event that reaches
//! a commit, a reopening or a sample. So the event was accepted, and its save
//! refused until the next accepted event. Restore now accepts it from an
//! event that reaches a resolve too. See spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::{check_save_with, CheckOptions, ReactiveSession};
use serde_json::Value;

const RECORDS: CheckOptions = CheckOptions {
    unused: false,
    records: true,
    names: false,
    withdrawals: false,
};

/// A sensor's readings and a series committed from them, whose journal keeps
/// its latest entry. `settle` resolves the series; `rules` adds declarations
/// and rules.
fn program(settle: &str, rules: &str) -> String {
    format!(
        "evidence h from \"sensor\";\n\
         readings s from h limit 4;\n\
         decisions d limit 2;\n\
         journal window 1;\n\
         event read x min 0 max 9;\n\
         event decide;\n\
         event settle;\n\
         {rules}\
         on read sample s = x;\n\
         on decide when has_sample(s) commit d because enough using latest(s);\n\
         {settle}"
    )
}

/// A rule that resolves the series' current revision.
const RESOLVE: &str = "on settle when committed(d) resolve d = latest(s) because latest(s);\n";

fn outcome(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

/// A reading, the commit, and the resolve, whose journal entry retires the
/// commit's.
const SETTLED: [(&str, &str); 3] = [("read", r#"{"x":5}"#), ("decide", "{}"), ("settle", "{}")];

fn settled(source: &str) -> ReactiveSession {
    let mut session = ReactiveSession::from_source(source).unwrap();
    for (event, payload) in SETTLED {
        let outcome = outcome(&mut session, event, payload);
        assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    }
    session
}

/// The save's effects of its last event, as `kind history record`.
fn effects(save: &Value) -> Vec<String> {
    save["effects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|effect| {
            format!(
                "{} {} {}",
                effect["kind"].as_str().unwrap(),
                effect["history"].as_str().unwrap_or(""),
                effect["record"].as_str().unwrap_or("")
            )
        })
        .collect()
}

#[test]
fn a_resolve_that_retires_a_journal_entry_leaves_a_save_that_restores() {
    let procedure = "proc settle_it(dd decisions, ss readings) {\n\
                     \x20   when committed(dd) resolve dd = latest(ss) because latest(ss);\n\
                     };\n";
    for (form, source) in [
        ("a rule", program(RESOLVE, "")),
        (
            "a revision",
            program(
                &RESOLVE.replace("resolve d =", "resolve revision(d, 1) ="),
                "",
            ),
        ),
        (
            "a procedure",
            program("on settle call settle_it(d, s);\n", procedure),
        ),
    ] {
        let session = settled(&source);
        let saved = session.save_json().unwrap();
        let save: Value = serde_json::from_str(&saved).unwrap();
        assert_eq!(save["last_event"], "settle", "{form}");
        assert!(
            effects(&save).contains(&String::from("retire journal journal@1")),
            "{form}: {:?}",
            effects(&save)
        );
        let restored = ReactiveSession::restore_json(&source, &saved).unwrap_or_else(|error| {
            panic!("{form}: restore_json refused the runtime's own save: {error}")
        });
        assert_eq!(restored.save_json().unwrap(), saved, "{form}");
        assert_eq!(restored.snapshot(), session.snapshot(), "{form}");
        let structured = ReactiveSession::restore(&source, &session.save().unwrap())
            .unwrap_or_else(|error| {
                panic!("{form}: restore refused the runtime's own save: {error}")
            });
        assert_eq!(structured.save_json().unwrap(), saved, "{form}");
        check_save_with(&source, &saved, RECORDS)
            .unwrap_or_else(|error| panic!("{form}: the records check refused the save: {error}"));
    }
}

#[test]
fn a_session_restored_after_a_resolve_retires_a_journal_entry_plays_on_alike() {
    let source = program(
        RESOLVE,
        "event recheck;\non recheck when committed(d) reopen d because latest(s);\n",
    );
    let mut live = settled(&source);
    let mut restored = ReactiveSession::restore_json(&source, &live.save_json().unwrap())
        .unwrap_or_else(|error| panic!("restore_json refused the runtime's own save: {error}"));
    // A reopening, a commit and a resolve each retire the entry before
    // theirs; the second resolve of the same revision is refused.
    let mut answers = Vec::new();
    for (event, payload) in [
        ("read", r#"{"x":6}"#),
        ("recheck", "{}"),
        ("decide", "{}"),
        ("settle", "{}"),
        ("settle", "{}"),
    ] {
        let answer = outcome(&mut live, event, payload);
        assert_eq!(
            outcome(&mut restored, event, payload),
            answer,
            "{event} after restore"
        );
        answers.push(
            answer["code"]
                .as_str()
                .or_else(|| answer["outcome"].as_str())
                .unwrap()
                .to_string(),
        );
        let saved = live.save_json().unwrap();
        assert_eq!(restored.save_json().unwrap(), saved, "save after {event}");
        ReactiveSession::restore_json(&source, &saved)
            .unwrap_or_else(|error| panic!("the save after {event} was refused: {error}"));
    }
    assert_eq!(
        answers,
        [
            "accepted",
            "accepted",
            "accepted",
            "accepted",
            "already_resolved"
        ]
    );
}

#[test]
fn a_journal_retirement_by_an_event_that_adds_no_entry_is_still_refused() {
    // The same save, but naming as its last event one whose only rule
    // reveals evidence: no rule it reaches adds a journal entry, so none
    // retires one there.
    let source = program(
        RESOLVE,
        "claim ok;\nevidence memo from \"desk\";\nevent note;\non note reveal memo supports ok;\n",
    );
    let mut save: Value = serde_json::from_str(&settled(&source).save_json().unwrap()).unwrap();
    save["last_event"] = "note".into();
    let error = ReactiveSession::restore_json(&source, &save.to_string())
        .expect_err("a retirement no rule of the last event can make is refused");
    assert!(
        error.contains("retire effect is not one the last event can make"),
        "{error}"
    );
}
