use caveat_runtime::reactive::{Provenance, Tracked};
use caveat_runtime::source_library::{FunctionValue, SourceLibrary};
use caveat_runtime::web_source_library::WebSourceLibrary;
use serde_json::json;

fn tracked(value: f64, evidence: &str, caveat: &str) -> Tracked<f64> {
    Tracked::new(
        value,
        Provenance::from_names([evidence.into()], [caveat.into()]).unwrap(),
    )
    .unwrap()
}

#[test]
fn compiled_functions_share_the_prelude_and_infer_discoverable_result_types() {
    let library = SourceLibrary::from_source(
        r#"
        scene "Pure computation accompanies a program.";
        fn bounded(value) = clamp(value, 0, 10);
        fn positive(value) = bounded(value) > 0;
        fn label(value) = if(positive(value), "Ready; {广州}", "Wait");
    "#,
    )
    .unwrap();
    assert_eq!(
        library.call_numbers("bounded", &[12.0]).unwrap().value,
        FunctionValue::Number(10.0)
    );
    assert_eq!(
        library.call_numbers("positive", &[-1.0]).unwrap().value,
        FunctionValue::Bool(false)
    );
    assert_eq!(
        library.call_numbers("label", &[2.0]).unwrap().value,
        FunctionValue::Text("Ready; {广州}".into())
    );
    assert_eq!(
        library.call_numbers("time_text", &[60.1]).unwrap().value,
        FunctionValue::Text("1:01".into())
    );
    let signatures = library.signatures();
    for (name, kind) in [
        ("bounded", "number"),
        ("positive", "boolean"),
        ("label", "text"),
    ] {
        let signature = signatures
            .iter()
            .find(|function| function.name == name)
            .unwrap();
        assert_eq!(signature.result_type, kind);
        assert_eq!(signature.parameters, ["value"]);
    }
    assert!(signatures
        .windows(2)
        .all(|pair| pair[0].name < pair[1].name));
    assert_eq!(library, library.clone());
}

#[test]
fn ignored_arguments_and_all_return_types_preserve_public_provenance() {
    let library = SourceLibrary::from_module_source(
        r#"
        fn nested(ignored) = 0;
        fn number(first, second) = nested(first);
        fn boolean(first, second) = false;
        fn message(first, second) = "Provisional";
    "#,
    )
    .unwrap();
    let arguments = [
        tracked(5.0, "forecast", "age"),
        tracked(7.0, "reading", "calibration"),
    ];
    let expected = arguments[0]
        .provenance
        .union(&arguments[1].provenance)
        .unwrap();
    for name in ["number", "boolean", "message"] {
        assert_eq!(library.call(name, &arguments).unwrap().provenance, expected);
    }
}

#[test]
fn input_numbers_and_provenance_are_checked_even_for_ignored_parameters() {
    let library = SourceLibrary::from_source("fn ignored(value) = 0;").unwrap();
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        1_000_000_000_001.0,
        -1_000_000_000_001.0,
    ] {
        assert!(library.call_numbers("ignored", &[value]).is_err());
    }
    assert!(library
        .call_numbers("ignored", &[1_000_000_000_000.0])
        .is_ok());
    let malformed = Tracked {
        value: 1.0,
        provenance: Provenance {
            evidence: ["x".repeat(65_537)].into_iter().collect(),
            caveats: Default::default(),
        },
    };
    assert!(library.call("ignored", &[malformed]).is_err());
    assert_eq!(
        library.call_numbers("ignored", &[2.0]).unwrap().value,
        FunctionValue::Number(0.0)
    );
}

#[test]
fn combined_argument_provenance_never_truncates_to_fit_the_limit() {
    let library = SourceLibrary::from_source("fn ignored(first, second) = 0;").unwrap();
    let make = |start| {
        Tracked::new(
            0.0,
            Provenance::from_names(
                (start..start + 600).map(|index| format!("evidence_{index}")),
                [],
            )
            .unwrap(),
        )
        .unwrap()
    };
    assert!(library
        .call("ignored", &[make(0), make(600)])
        .unwrap_err()
        .contains("provenance"));
}

#[test]
fn invalid_unused_functions_and_non_numeric_parameters_fail_at_compile_time() {
    for source in [
        "fn unused(value) = undeclared;",
        "fn unused(value) = reef.x;",
        "fn unused(value) = observed(sensor);",
        "fn unused(value) = latest(readings);",
        "fn unused(value) = qualified(value, sensor);",
        "fn unused(value) = unused(value);",
        "fn first(value) = second(value); fn second(value) = first(value);",
        "fn unused(value, value) = value;",
        "fn unused(value) = missing(value);",
        "fn a(value) = value; fn unused(value) = a();",
        "fn a(value) = value; fn unused(value) = a(true);",
        "fn unused(value) = 1; fn unused(value) = 2;",
        "fn clamp(value) = value;",
        "fn unused(value) = if(value > 0, 1, \"text\");",
    ] {
        assert!(
            SourceLibrary::from_source(source).is_err(),
            "accepted {source}"
        );
    }
}

#[test]
fn extraction_does_not_claim_to_validate_other_program_semantics() {
    let source = "state duplicate = 0; state duplicate = 1; event run; fn own(value) = value + 1;";
    let library = SourceLibrary::from_source(source).unwrap();
    assert_eq!(
        library.call_numbers("own", &[1.0]).unwrap().value,
        FunctionValue::Number(2.0)
    );
    assert!(caveat_runtime::reactive::ReactiveSession::from_source(source).is_err());
    assert!(SourceLibrary::from_module_source(source).is_err());
    assert!(SourceLibrary::from_source("fn broken(value) = ;").is_err());
}

#[test]
fn arity_output_bounds_and_runtime_requirements_return_errors() {
    let library = SourceLibrary::from_source(
        r#"
        fn twice(value) = value * 2;
        fn invalid(value) = 1 / value;
    "#,
    )
    .unwrap();
    assert!(library.call_numbers("missing", &[]).is_err());
    assert!(library.call_numbers("twice", &[]).is_err());
    assert!(library.call_numbers("twice", &[1.0, 2.0]).is_err());
    assert!(library
        .call_numbers("twice", &[1_000_000_000_000.0])
        .is_err());
    assert!(library.call_numbers("invalid", &[0.0]).is_err());
    assert!(library.call_numbers("clamp", &[1.0, 2.0, 0.0]).is_err());
    assert_eq!(
        library.call_numbers("twice", &[4.0]).unwrap().value,
        FunctionValue::Number(8.0)
    );
}

#[test]
fn compiler_bounds_total_retained_nodes_and_literal_bytes() {
    let mut expression = "value".to_owned();
    for _ in 0..8 {
        expression = format!("({expression} + {expression})");
    }
    let mut nodes = format!("fn large(value) = {expression};");
    for index in 0..70 {
        nodes.push_str(&format!("fn proxy{index}(value) = large(value);"));
    }
    assert!(SourceLibrary::from_source(&nodes)
        .unwrap_err()
        .contains("compiled node limit"));

    let mut strings = format!("fn message() = \"{}\";", "x".repeat(14_000));
    for index in 0..80 {
        strings.push_str(&format!("fn proxy{index}() = message();"));
    }
    assert!(SourceLibrary::from_source(&strings)
        .unwrap_err()
        .contains("compiled string limit"));

    let parameters = (0..33)
        .map(|i| format!("p{i}"))
        .collect::<Vec<_>>()
        .join(",");
    assert!(
        SourceLibrary::from_source(&format!("fn many({parameters}) = 0;"))
            .unwrap_err()
            .contains("parameter limit")
    );
    assert!(SourceLibrary::from_source(&" ".repeat(1_048_577))
        .unwrap_err()
        .contains("source limit"));
}

#[test]
fn source_only_policy_edits_change_native_and_web_results_identically() {
    let original = "fn response(value) = value * 2;";
    let edited = original.replace("value * 2", "value * 3");
    for (source, expected) in [(original, 8.0), (edited.as_str(), 12.0)] {
        let native = SourceLibrary::from_source(source)
            .unwrap()
            .call_numbers("response", &[4.0])
            .unwrap();
        let web = WebSourceLibrary::new(source).unwrap();
        assert_eq!(native.value, FunctionValue::Number(expected));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&web.call("response", "[4]").unwrap())
                .unwrap(),
            serde_json::to_value(native).unwrap()
        );
    }
}

#[test]
fn web_adapter_uses_numeric_arrays_and_serializes_typed_results_with_metadata() {
    let web = WebSourceLibrary::new(
        "fn selected(value) = value > 0; fn label(value) = \"广州; {ready}\";",
    )
    .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&web.call("selected", "[1]").unwrap()).unwrap(),
        json!({"value":true,"provenance":{"evidence":[],"caveats":[]}})
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&web.call("label", "[1]").unwrap()).unwrap()
            ["value"],
        "广州; {ready}"
    );
    for input in [
        "{}",
        "null",
        "[true]",
        "[null]",
        "[\"1\"]",
        "[1e999]",
        "[NaN]",
        "[1] trailing",
        "[1000000000001]",
    ] {
        assert!(web.call("label", input).is_err(), "accepted {input}");
    }
    assert!(web.call("label", &" ".repeat(4097)).is_err());
    let signatures: serde_json::Value = serde_json::from_str(&web.signatures().unwrap()).unwrap();
    assert!(signatures
        .as_array()
        .unwrap()
        .iter()
        .any(|signature| signature["name"] == "label" && signature["result_type"] == "text"));
}
