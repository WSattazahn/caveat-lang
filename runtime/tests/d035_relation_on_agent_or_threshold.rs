//! D035: from 1.4.0 a program that names evidence `agent` and states a
//! relation about it (`agent supports c;`) did not load: the agents statement
//! took every line that starts with `agent` and refused it with "agent expects
//! NAME [sends EVENT, …]". The program loaded on 1.3.0. The Thresholds draft
//! did the same to evidence named `threshold`. A three-word relation now stays
//! a relation; neither statement has such a form. Found by the compiler thread,
//! 2026-10-31. See `parse_directive_at` in `reactive.rs`.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

fn program(name: &str) -> String {
    format!(
        "claim c;\nclaim d;\ncaveat shaky consequence low;\n\
         evidence {name} from \"a log\";\n\
         {name} supports c;\n{name} opposes d;\nshaky qualifies {name};\n\
         state seen = 0;\nevent look;\n\
         on look reveal {name};\non look set seen = qualified(1, {name});\n"
    )
}

#[test]
fn d035_relations_about_evidence_named_agent_or_threshold_load() {
    for name in ["agent", "threshold"] {
        let source = program(name);
        let mut session =
            ReactiveSession::from_source(&source).unwrap_or_else(|error| panic!("{name}: {error}"));
        let outcome = session
            .dispatch_outcome_json("look", "{}")
            .unwrap_or_else(|fatal| panic!("{name}: {}", fatal.message));
        let outcome = serde_json::to_value(outcome).unwrap();
        assert_eq!(outcome["outcome"], "accepted", "{name}: {outcome}");
        let snapshot: Value = serde_json::to_value(session.snapshot()).unwrap();
        assert_eq!(snapshot["values"]["seen"], 1.0, "{name}");
        assert_eq!(
            snapshot["qualified_values"]["seen"]["provenance"]["caveats"],
            serde_json::json!(["shaky"]),
            "{name}: the declared qualification holds"
        );
    }
}

#[test]
fn d035_the_agent_and_threshold_statements_keep_their_meaning() {
    // `agent` still declares an agent: an unsigned `go` is refused.
    let source = "state n = 0;\nevent go;\nagent planner sends go;\n";
    let mut session = ReactiveSession::from_source(source).unwrap();
    let outcome = serde_json::to_value(session.dispatch_outcome_json("go", "{}").unwrap()).unwrap();
    assert_eq!(outcome["code"], "wrong_agent", "{outcome}");
    // `threshold` still declares a threshold.
    let source = "state n = 0;\nevent go;\nthreshold enough = 3;\non go set n = enough;\n";
    let mut session = ReactiveSession::from_source(source).unwrap();
    let outcome = serde_json::to_value(session.dispatch_outcome_json("go", "{}").unwrap()).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{outcome}");
    let snapshot: Value = serde_json::to_value(session.snapshot()).unwrap();
    assert_eq!(snapshot["values"]["n"], 3.0);
    for (source, message) in [
        (
            "agent planner sends;\n",
            "agent expects NAME [sends EVENT, …]",
        ),
        ("threshold enough 3;\n", "threshold expects NAME = NUMBER"),
    ] {
        let Err(error) = ReactiveSession::from_source(source) else {
            panic!("{source} loaded");
        };
        assert!(error.contains(message), "{error}");
    }
}
