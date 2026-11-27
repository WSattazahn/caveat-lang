//! Before 1.4, `only_if` was an ordinary source-function name. Recover
//! numeric wrappers without refusing unused bodies 1.4 infers as the form.
use caveat_runtime::reactive::{Provenance, ReactiveSession, Tracked};
use caveat_runtime::source_library::{FunctionValue, SourceLibrary};
use serde_json::{json, Value};

#[test]
fn numeric_calls_infer_the_authored_result_and_preserve_arguments() {
    for (body, expected) in [
        ("x + y", FunctionValue::Number(6.0)),
        ("x > y", FunctionValue::Bool(true)),
        (
            "text(x) + \":\" + text(y)",
            FunctionValue::Text("4:2".into()),
        ),
        ("0", FunctionValue::Number(0.0)),
    ] {
        let source = format!("fn a_call(x) = only_if(x, 2);\nfn only_if(x, y) = {body};\n");
        let library = SourceLibrary::from_module_source(&source).unwrap();
        let argument = Tracked::new(
            4.0,
            Provenance::from_names(["reading".into()], ["uncertain".into()]).unwrap(),
        )
        .unwrap();
        let result = library
            .call("a_call", std::slice::from_ref(&argument))
            .unwrap();
        assert_eq!(result.value, expected, "{source}");
        assert_eq!(result.provenance, argument.provenance, "{source}");
    }
}

#[test]
fn forward_calls_and_nested_authored_only_if_calls_are_inferred() {
    let source = "\
fn a_call(x) = b_call(x) + 1;
fn b_call(x) = only_if(only_if(x, 1), 2);
fn only_if(x, y) = sum(x, y);
fn sum(x, y) = x + y;
";
    let library = SourceLibrary::from_module_source(source).unwrap();
    assert_eq!(
        library.call_numbers("a_call", &[4.0]).unwrap().value,
        FunctionValue::Number(8.0)
    );
}

#[test]
fn mixed_registry_preserves_legacy_success_and_recovers_numeric_wrappers() {
    let source = "\
fn only_if(x, y) = x + y;
fn old_unused(x) = only_if(x > 0, 1);
fn old_chain(x) = old_unused(x) + 1;
fn recovered(x) = only_if(x, 2);
state n = 0;
event go;
on go set n = recovered(n);
";
    let mut session = ReactiveSession::from_source(source).unwrap();
    session.dispatch_json("go", "{}").unwrap();
    assert_eq!(session.snapshot().values["n"], 2.0);
    let mut restored = ReactiveSession::restore(source, &session.save().unwrap()).unwrap();
    session.dispatch_json("go", "{}").unwrap();
    restored.dispatch_json("go", "{}").unwrap();
    assert_eq!(
        serde_json::to_value(session.snapshot()).unwrap(),
        serde_json::to_value(restored.snapshot()).unwrap()
    );
    // A source library compiles every body, including the unused boolean
    // argument that a reactive session has historically left uncalled.
    assert!(SourceLibrary::from_source(source).is_err());
}

#[test]
fn fallback_keeps_refusals_for_types_captures_arity_cycles_and_limits() {
    for source in [
        "fn only_if(x) = x;\nfn wrapper(x) = only_if(x, 2);\n",
        "fn only_if(x, y) = wrapper(x);\nfn wrapper(x) = only_if(x, 2);\n",
        "fn only_if(x, y) = x + y;\nfn wrapper(x) = only_if(missing, 2);\n",
        "fn only_if(x, y) = x + y;\nfn wrapper(x) = only_if(x, elapsed());\n",
        "fn only_if(x, y) = x + y;\nfn wrapper(x) = only_if(x, qualified(1, e));\n",
        "fn only_if(x, y) = x + y;\nfn wrapper(x) = only_if(x, latest(history));\n",
        "fn only_if(x, y) = x + y;\nfn wrapper(x) = only_if(x > 0, 1);\n",
        "fn only_if(x, y) = x + y;\nfn wrapper(x) = if(false, only_if(x > 0, 1), 0);\n",
    ] {
        assert!(
            SourceLibrary::from_module_source(source).is_err(),
            "{source}"
        );
    }
    let mut deep = "fn only_if(x, y) = x + y;\nfn f0(x) = only_if(x, 2);\n".to_owned();
    for index in 1..70 {
        deep.push_str(&format!("fn f{index}(x) = f{}(x);\n", index - 1));
    }
    assert!(SourceLibrary::from_module_source(&deep).is_err());

    // The fallback must keep the original inference error if the actual
    // authored call also fails (here because its arity is wrong).
    let error =
        SourceLibrary::from_module_source("fn only_if(x) = x;\nfn wrapper(x) = only_if(x, 2);\n")
            .unwrap_err();
    assert!(
        error.contains("expression requires boolean, found number"),
        "{error}"
    );

    // No authored function: the intrinsic's number/bool rules are unchanged.
    assert!(SourceLibrary::from_module_source("fn wrapper(x) = only_if(x, 2);\n").is_err());
    let library =
        SourceLibrary::from_module_source("fn wrapper(x) = only_if(x > 0, 2);\n").unwrap();
    assert_eq!(
        library.call_numbers("wrapper", &[4.0]).unwrap().value,
        FunctionValue::Number(2.0)
    );
}

fn projection(snapshot: Value) -> Value {
    let mut result = serde_json::Map::new();
    for field in [
        "values",
        "qualified_values",
        "value_grounds",
        "bindings",
        "binding_qualifications",
        "commitment_bases",
        "commitment_grounds",
        "decision_journal",
    ] {
        result.insert(field.into(), snapshot[field].clone());
    }
    Value::Object(result)
}

fn historical_save(source: &str, saved: &str, expected: &str) {
    let expected: Value = serde_json::from_str(expected).unwrap();
    let saved = serde_json::from_str(saved).unwrap();
    let mut restored = ReactiveSession::restore(source, &saved).unwrap();
    let mut live = ReactiveSession::from_source(source).unwrap();
    live.dispatch_json("go", r#"{"v":4}"#).unwrap();
    assert_eq!(
        serde_json::to_value(restored.snapshot()).unwrap(),
        serde_json::to_value(live.snapshot()).unwrap()
    );
    assert_eq!(
        projection(serde_json::to_value(restored.snapshot()).unwrap()),
        expected["initial"]
    );
    for step in expected["continuation"].as_array().unwrap() {
        let payload = serde_json::to_string(&step["payload"]).unwrap();
        let outcome = restored.dispatch_outcome_json("go", &payload).unwrap();
        let fresh = live.dispatch_outcome_json("go", &payload).unwrap();
        assert_eq!(
            serde_json::to_value(&outcome).unwrap(),
            serde_json::to_value(fresh).unwrap()
        );
        assert_eq!(
            serde_json::to_value(outcome).unwrap()["outcome"],
            step["outcome"]
        );
        assert_eq!(
            projection(serde_json::to_value(restored.snapshot()).unwrap()),
            step["snapshot"]
        );
    }
    assert_eq!(
        serde_json::to_value(restored.snapshot()).unwrap(),
        serde_json::to_value(live.snapshot()).unwrap()
    );
    let snapshot = serde_json::to_value(restored.snapshot()).unwrap();
    assert_eq!(
        snapshot["commitment_grounds"]["d"]["evidence"],
        json!(["e"])
    );
    assert_eq!(
        snapshot["commitment_grounds"]["d"]["caveats"],
        json!(["uncertainty"])
    );
}

macro_rules! historical_fixture {
    ($name:literal) => {
        historical_save(
            include_str!(concat!("fixtures/only-if-functions/", $name, ".cav")),
            include_str!(concat!("fixtures/only-if-functions/", $name, ".save.json")),
            include_str!(concat!(
                "fixtures/only-if-functions/",
                $name,
                ".expected.json"
            )),
        )
    };
}

#[test]
fn genuine_v100_numeric_boolean_and_text_saves_restore_and_continue() {
    historical_fixture!("v1.0.0-numeric");
    historical_fixture!("v1.0.0-boolean");
    historical_fixture!("v1.0.0-text");
}

#[test]
fn genuine_v141_unused_boolean_wrapper_save_still_restores_and_continues() {
    historical_fixture!("v1.4.1-unused");
}
