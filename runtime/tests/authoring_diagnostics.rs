//! Authoring mistakes: distinguish invalid citations from missing names and
//! explain why a state/history expression is a `define`, not a capturing `fn`.
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::web::WebReactiveSession;
use serde_json::{json, Value};

const DECLARATIONS: &str = r#"
claim intent_clear;
evidence memory from "memory source";
caveat stale consequence material;
readings freshness from memory limit 4;
decisions strategy limit 4;
state confidence = 0;
event observe value min 0 max 100;
on observe reveal memory supports intent_clear;
on observe sample freshness = value supports intent_clear;
on observe set confidence = latest(freshness);
"#;

#[test]
fn because_distinguishes_known_wrong_kinds_from_unknown_names_in_bind_and_set() {
    for (citation, kind) in [
        ("intent_clear", "claim"),
        ("memory", "evidence"),
        ("stale", "caveat"),
        ("freshness", "reading stream"),
        ("strategy", "decision series"),
        ("observe", "event"),
    ] {
        for statement in [
            format!("bind hud.text = \"ok\" because {citation};"),
            format!("on observe set confidence = value because {citation};"),
            format!("bind hud.value = 1 because if(true, 1, {citation});"),
        ] {
            let error =
                ReactiveSession::from_source(&format!("{DECLARATIONS} {statement}")).unwrap_err();
            assert!(
                error.contains(&format!(
                    "{citation} is a declared {kind}, not a value citation"
                )),
                "{statement}: {error}"
            );
            assert!(!error.contains("unknown"), "{statement}: {error}");
        }
    }
    for statement in [
        "bind hud.text = \"ok\" because missing;",
        "on observe set confidence = value because missing;",
    ] {
        let error =
            ReactiveSession::from_source(&format!("{DECLARATIONS} {statement}")).unwrap_err();
        assert!(
            error.contains("unknown citation identifier missing"),
            "{error}"
        );
        assert!(!error.contains("declared"), "{error}");
    }
}

#[test]
fn valid_citations_keep_exact_grounds_and_qualifications() {
    let source = format!(
        r#"{DECLARATIONS}
        stale qualifies memory;
        define current_freshness = latest(freshness);
        on observe set confidence = current_freshness because latest(freshness);
        bind hud.value = confidence when has_sample(freshness) because confidence;
        bind hud.seen = observed(memory) because observed(memory);
    "#
    );
    let mut session = ReactiveSession::from_source(&source).unwrap();
    let snapshot =
        serde_json::to_value(session.dispatch_json("observe", r#"{"value":68}"#).unwrap()).unwrap();
    assert_eq!(snapshot["values"]["confidence"], 68.0);
    assert_eq!(
        snapshot["value_grounds"]["confidence"],
        json!({"evidence":["freshness@1"], "caveats":["stale"]})
    );
    assert_eq!(
        snapshot["binding_explanations"]["hud"]["value"],
        snapshot["value_grounds"]["confidence"]
    );
    assert_eq!(
        snapshot["binding_explanations"]["hud"]["seen"],
        json!({"evidence":["memory"], "caveats":["stale"]})
    );
}

#[test]
fn ungrounded_citations_still_reject_atomically_and_leave_the_session_usable() {
    for statement in [
        "bind hud.text = \"ok\" because observed(memory);",
        "on observe set confidence = 1 because latest(freshness);",
    ] {
        let source = format!("{DECLARATIONS} event retry; {statement}");
        let mut session = WebReactiveSession::new(&source).unwrap();
        let before = session.save().unwrap();
        let rejected: Value = serde_json::from_str(
            &session
                .dispatch_outcome("observe", r#"{"value":68}"#)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(rejected["outcome"], "rejected");
        assert_eq!(rejected["origin"], "evaluation");
        assert_eq!(rejected["code"], "ungrounded_citation");
        assert!(rejected["message"].as_str().unwrap().contains("never read"));
        assert_eq!(session.save().unwrap(), before);
        let retry: Value =
            serde_json::from_str(&session.dispatch_outcome("retry", "{}").unwrap()).unwrap();
        assert_eq!(retry["outcome"], "accepted");
    }
}

#[test]
fn function_shaped_defines_explain_named_expressions_and_pure_function_limits() {
    for declaration in [
        "define freshness_last() = latest(freshness);",
        "define freshness_last () = confidence;",
        "define freshness_last(x) = x;",
    ] {
        let error =
            ReactiveSession::from_source(&format!("{DECLARATIONS} {declaration}")).unwrap_err();
        assert!(
            error.contains("define declares a named expression and does not take parameters"),
            "{error}"
        );
        assert!(
            error.contains("use define freshness_last = EXPRESSION"),
            "{error}"
        );
        assert!(
            error.contains("only for a pure function with explicit parameters"),
            "{error}"
        );
        assert!(
            error.contains("cannot capture state, history or evidence"),
            "{error}"
        );
    }
    ReactiveSession::from_source(&format!(
        "{DECLARATIONS} define freshness_last = latest(freshness); fn doubled(x) = x * 2;"
    ))
    .unwrap();
    for body in ["confidence", "latest(freshness)", "observed(memory)"] {
        assert!(
            ReactiveSession::from_source(&format!("{DECLARATIONS} fn captured() = {body};"))
                .is_err()
        );
    }
}
