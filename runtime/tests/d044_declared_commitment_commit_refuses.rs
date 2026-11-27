//! D044: an event whose rule committed a commitment the program declared was
//! fatal `unclassified` ("commitment hold already exists"), so a host lost the
//! session, although the runtime had already rolled the event back. A
//! declaration such as `commit hold because enough;` makes a plain commitment
//! as the program loads, and the session holds it from then on, in force or,
//! once reopened, reopened. Committing it again is what
//! `evaluation/already_committed` classifies (D008), but only a commitment an
//! event had made was refused so. The rollback and the diagnostic text are
//! unchanged. See spec/caveat-dispatch-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// `hold` is declared committed, and reopened by `sensor` when `reopen` says
/// so. `go` counts and then commits `hold` again; `count` only counts.
fn program(reopen: &str) -> String {
    format!(
        "claim safe;\n\
         evidence sensor from \"sensor\";\n\
         sensor supports safe;\n\
         commit hold because enough;\n\
         {reopen}\
         state seen = 0;\n\
         event go;\n\
         event count;\n\
         on go set seen = seen + 1;\n\
         on go commit hold because enough;\n\
         on count set seen = seen + 1;\n"
    )
}

fn dispatch(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

#[test]
fn committing_a_declared_commitment_again_is_refused() {
    for (state, reopen) in [
        ("in force", ""),
        ("reopened", "reopen hold because sensor;\n"),
    ] {
        let source = program(reopen);
        let mut session = ReactiveSession::from_source(&source).unwrap();
        let before = session.save_json().unwrap();
        let outcome = dispatch(&mut session, "go");
        assert_eq!(outcome["outcome"], "rejected", "{state}: {outcome}");
        assert_eq!(outcome["origin"], "evaluation", "{state}");
        assert_eq!(outcome["code"], "already_committed", "{state}");
        assert_eq!(
            outcome["message"], "event go, rule 2: commitment hold already exists",
            "{state}"
        );
        // Nothing the event did stays, its first rule's count included, and
        // the session goes on.
        assert_eq!(session.save_json().unwrap(), before, "{state}");
        let outcome = dispatch(&mut session, "count");
        assert_eq!(outcome["outcome"], "accepted", "{state}: {outcome}");
        assert_eq!(session.snapshot().values["seen"], 1.0, "{state}");
        let save = session.save_json().unwrap();
        let mut restored = ReactiveSession::restore_json(&source, &save)
            .unwrap_or_else(|error| panic!("{state}: {error}"));
        assert_eq!(restored.snapshot(), session.snapshot(), "{state}");
        // The restored session refuses the commit alike.
        assert_eq!(dispatch(&mut restored, "go"), dispatch(&mut session, "go"));
    }
}
