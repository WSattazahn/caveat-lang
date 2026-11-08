//! D045: an event that read `reopened(hold)`, or cited it with `because
//! reopened(hold)`, while the evidence a declaration reopened `hold` with was
//! not yet observed was fatal `unclassified` ("cannot qualify a value with
//! unobserved evidence sensor"), so a host lost the session, although the
//! runtime had already rolled the event back. A `reopen` in a rule refuses
//! unobserved evidence, but a declaration such as `reopen hold because
//! sensor;` loads before anything is observed, and both the value
//! `reopened(hold)` reads and the grounds a citation of it cites carry that
//! evidence. The event is now refused as `evaluation/unobserved_evidence`, as
//! reading `qualified(…)` of that evidence is; once an event observes it, the
//! read is accepted, as it was. The rollback and the diagnostic text are unchanged, and a binding
//! that reads it as the program loads is still rejected. See
//! spec/caveat-dispatch-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// `hold` is declared committed and reopened by `sensor`, which `see`
/// observes. `go` counts, then reads `reopened(hold)` as `read` says.
fn program(read: &str) -> String {
    format!(
        "claim safe;\n\
         evidence sensor from \"sensor\";\n\
         commit hold because enough;\n\
         reopen hold because sensor;\n\
         state seen = 0;\n\
         event go;\n\
         event see;\n\
         on go set seen = seen + 1;\n\
         {read}\n\
         on see reveal sensor supports safe;\n"
    )
}

fn dispatch(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

#[test]
fn reading_a_reopening_by_unobserved_evidence_refuses_the_event() {
    for (form, read) in [
        ("a guard", "on go when reopened(hold) set seen = seen + 10;"),
        (
            "an expression",
            "on go set seen = seen + if(reopened(hold), 10, 20);",
        ),
    ] {
        let source = program(read);
        let mut session = ReactiveSession::from_source(&source).unwrap();
        let before = session.save_json().unwrap();
        let outcome = dispatch(&mut session, "go");
        assert_eq!(outcome["outcome"], "rejected", "{form}: {outcome}");
        assert_eq!(outcome["origin"], "evaluation", "{form}");
        assert_eq!(outcome["code"], "unobserved_evidence", "{form}");
        assert_eq!(
            outcome["message"],
            "event go, rule 2: cannot qualify a value with unobserved evidence sensor",
            "{form}"
        );
        // Nothing the event did stays, its first rule's count included.
        assert_eq!(session.save_json().unwrap(), before, "{form}");
        let save = session.save_json().unwrap();
        let mut restored = ReactiveSession::restore_json(&source, &save)
            .unwrap_or_else(|error| panic!("{form}: {error}"));
        assert_eq!(dispatch(&mut restored, "go"), outcome, "{form}");
        // Once `see` observes `sensor`, the read is accepted and carries it.
        for run in [&mut session, &mut restored] {
            assert_eq!(dispatch(run, "see")["outcome"], "accepted", "{form}");
            assert_eq!(dispatch(run, "go")["outcome"], "accepted", "{form}");
            let snapshot = run.snapshot();
            assert_eq!(snapshot.values["seen"], 11.0, "{form}");
            let carried = serde_json::to_value(&snapshot.qualified_values["seen"]).unwrap();
            assert_eq!(
                carried["provenance"]["evidence"],
                serde_json::json!(["sensor"]),
                "{form}: {carried}"
            );
        }
        assert_eq!(restored.snapshot(), session.snapshot(), "{form}");
    }
}

#[test]
fn a_citation_of_the_reopening_refuses_the_event() {
    // `because reopened(hold)` cites the reopening's grounds, which carry
    // `sensor` as its value does. Each value reads the reopening only once
    // `sensor` is observed, so until then only the citation reads it, and
    // once it is observed the citation is grounded.
    let value = "if(observed(sensor), if(reopened(hold), 1, 2), 3)";
    for (form, read, message) in [
        (
            "a set's citation",
            format!("on go set seen = {value} because reopened(hold);"),
            "event go, rule 2: state seen because: \
             cannot qualify a value with unobserved evidence sensor",
        ),
        (
            "a guarded binding's citation",
            format!("bind view.x = {value} when seen > 0 because reopened(hold);"),
            "binding view.x because: cannot qualify a value with unobserved evidence sensor",
        ),
    ] {
        let source = program(&read);
        let mut session = ReactiveSession::from_source(&source).unwrap();
        let before = session.save_json().unwrap();
        let outcome = dispatch(&mut session, "go");
        assert_eq!(outcome["outcome"], "rejected", "{form}: {outcome}");
        assert_eq!(outcome["origin"], "evaluation", "{form}");
        assert_eq!(outcome["code"], "unobserved_evidence", "{form}");
        assert_eq!(outcome["message"], message, "{form}");
        assert_eq!(session.save_json().unwrap(), before, "{form}");
        let mut restored = ReactiveSession::restore_json(&source, &before)
            .unwrap_or_else(|error| panic!("{form}: {error}"));
        assert_eq!(dispatch(&mut restored, "go"), outcome, "{form}");
        // Once `see` observes `sensor`, the citation is accepted.
        for run in [&mut session, &mut restored] {
            assert_eq!(dispatch(run, "see")["outcome"], "accepted", "{form}");
            let accepted = dispatch(run, "go");
            assert_eq!(accepted["outcome"], "accepted", "{form}: {accepted}");
        }
        assert_eq!(restored.snapshot(), session.snapshot(), "{form}");
    }
}

#[test]
fn a_binding_that_reads_it_as_the_program_loads_is_still_rejected() {
    let source = format!(
        "{}bind view.reopened = if(reopened(hold), 1, 0);\n",
        program("")
    );
    let error = ReactiveSession::from_source(&source).err().unwrap();
    assert_eq!(
        error.to_string(),
        "binding view.reopened: cannot qualify a value with unobserved evidence sensor"
    );
}
