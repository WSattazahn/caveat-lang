//! D042: an event whose `qualify … after` delay came out negative under the
//! binary64 clock was fatal `unclassified`, so a host lost the session,
//! although the runtime had already rolled the event back. The delay is
//! computed from what the session has seen, so the event is now refused as
//! `evaluation/expression`, as a computed delay an integer clock cannot wait
//! already was (Elapsed 0.1). The rollback and the diagnostic text are
//! unchanged, and an integer clock keeps its own diagnostic. See
//! spec/caveat-dispatch-0.1.md ("Changes"; Version Lab F262).
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// `show` reveals `sight`; `see` counts itself and then schedules `stale`
/// against `sight`, `d` units from now, either directly or through a
/// procedure.
fn program(schedule: &str) -> String {
    format!(
        "claim safe;\n\
         evidence sight from \"sight\";\n\
         caveat stale consequence material;\n\
         state seen = 0;\n\
         proc fade(e evidence, n) {{\n\
         \x20   qualify e with stale after n;\n\
         }};\n\
         event show;\n\
         event see d min -5 max 5;\n\
         event tick dt min 0 max 0.1;\n\
         on show reveal sight supports safe;\n\
         on see set seen = seen + 1;\n\
         {schedule}\n"
    )
}

fn dispatch(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} {payload} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

#[test]
fn a_negative_computed_delay_refuses_the_event() {
    for (form, schedule, at) in [
        (
            "a rule",
            "on see qualify sight with stale after d;",
            "event see, rule 3",
        ),
        (
            "a procedure",
            "on see call fade(sight, d);",
            "event see, rule 3: procedure fade[sight], step 1",
        ),
    ] {
        let source = program(schedule);
        let mut session = ReactiveSession::from_source(&source).unwrap();
        let outcome = dispatch(&mut session, "show", "{}");
        assert_eq!(outcome["outcome"], "accepted", "{form}: {outcome}");
        let before = session.save_json().unwrap();
        let outcome = dispatch(&mut session, "see", r#"{"d": -1}"#);
        assert_eq!(outcome["outcome"], "rejected", "{form}: {outcome}");
        assert_eq!(outcome["origin"], "evaluation", "{form}: {outcome}");
        assert_eq!(outcome["code"], "expression", "{form}: {outcome}");
        // The diagnostic is the text the fatal error carried.
        assert_eq!(
            outcome["message"],
            format!("{at}: qualify after requires a nonnegative number of seconds"),
            "{form}"
        );
        // The whole event is rolled back, the count before it included.
        assert_eq!(session.save_json().unwrap(), before, "{form}");
        assert_eq!(session.snapshot().values["seen"], 0.0, "{form}");
        // The session goes on: a delay of zero, or of minus zero, is not
        // negative, and the qualification applies on the next tick.
        for d in ["-0.0", "0", "2"] {
            let outcome = dispatch(&mut session, "see", &format!(r#"{{"d": {d}}}"#));
            assert_eq!(outcome["outcome"], "accepted", "{form}, d {d}: {outcome}");
        }
        assert_eq!(
            session.snapshot().scheduled_qualifications.len(),
            3,
            "{form}"
        );
        let outcome = dispatch(&mut session, "tick", r#"{"dt": 0.05}"#);
        assert_eq!(outcome["outcome"], "accepted", "{form}: {outcome}");
        assert_eq!(
            session.snapshot().scheduled_qualifications.len(),
            1,
            "{form}"
        );
        let saved = session.save_json().unwrap();
        let restored = ReactiveSession::restore_json(&source, &saved)
            .unwrap_or_else(|error| panic!("{form}: restore refused the save: {error}"));
        assert_eq!(restored.save_json().unwrap(), saved, "{form}");
    }
}

#[test]
fn a_negative_literal_delay_loads_and_refuses_the_event_it_runs_in() {
    // A literal delay is checked when the program loads only under an
    // integer clock; under the binary64 clock a negative one still loads,
    // as before, and the event that runs it is refused, not fatal.
    let source = program("on see qualify sight with stale after -1;");
    let mut session = ReactiveSession::from_source(&source).unwrap();
    let outcome = dispatch(&mut session, "show", "{}");
    assert_eq!(outcome["outcome"], "accepted", "{outcome}");
    let outcome = dispatch(&mut session, "see", r#"{"d": 0}"#);
    assert_eq!(outcome["outcome"], "rejected", "{outcome}");
    assert_eq!(outcome["code"], "expression", "{outcome}");
    let outcome = dispatch(&mut session, "tick", r#"{"dt": 0.05}"#);
    assert_eq!(outcome["outcome"], "accepted", "{outcome}");
}

#[test]
fn an_integer_clock_keeps_its_own_diagnostic_for_a_negative_delay() {
    // Under an integer clock a negative computed delay is one the clock cannot
    // wait. That was refused before D042, with the clock's diagnostic, and it
    // still is: the binary64 check does not run first.
    let source = "claim safe;\n\
                  caveat faded consequence material;\n\
                  evidence bite from \"bite\";\n\
                  event step dt min 0 max 100;\n\
                  event eat d min -5 max 5;\n\
                  clock step every 16 integer;\n\
                  on eat reveal bite supports safe;\n\
                  on eat qualify bite with faded after d;\n";
    let mut session = ReactiveSession::from_source(source).unwrap();
    let before = session.save_json().unwrap();
    let outcome = dispatch(&mut session, "eat", r#"{"d": -1}"#);
    assert_eq!(outcome["outcome"], "rejected", "{outcome}");
    assert_eq!(outcome["origin"], "evaluation", "{outcome}");
    assert_eq!(outcome["code"], "expression", "{outcome}");
    assert_eq!(
        outcome["message"],
        "event eat, rule 2: qualify bite with faded after: an integer clock waits \
         a whole number of units in 0..9007199254740991, not -1"
    );
    assert_eq!(session.save_json().unwrap(), before);
    let outcome = dispatch(&mut session, "eat", r#"{"d": 2}"#);
    assert_eq!(outcome["outcome"], "accepted", "{outcome}");
}
