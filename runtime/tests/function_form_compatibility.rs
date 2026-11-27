//! Names introduced as forms must keep an older program's own function,
//! including calls inside another function where result types are inferred.
use caveat_runtime::reactive::{Provenance, ReactiveSession, Tracked};
use caveat_runtime::source_library::{FunctionValue, SourceLibrary};

const UNARY: &[&str] = &["is_set", "grounds_of", "on_hold", "resolved", "outcome"];

#[test]
fn authored_form_names_work_inside_functions_and_keep_result_types() {
    for name in UNARY {
        for (body, expected) in [
            ("x + 1", FunctionValue::Number(5.0)),
            ("x > 0", FunctionValue::Bool(true)),
            ("text(x)", FunctionValue::Text("4".into())),
        ] {
            // The caller sorts before the callee: inference must discover
            // the call rather than relying on definition or name order.
            let source = format!("fn a_call(x) = {name}(x);\nfn {name}(x) = {body};\n");
            let library = SourceLibrary::from_module_source(&source)
                .unwrap_or_else(|error| panic!("{source}: {error}"));
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
    let source = "fn a_call(x) = in_basis(x, x);\nfn in_basis(x, y) = x + y;\n";
    let library = SourceLibrary::from_module_source(source).unwrap();
    assert_eq!(
        library.call_numbers("a_call", &[4.0]).unwrap().value,
        FunctionValue::Number(8.0)
    );
}

#[test]
fn an_unused_only_if_wrapper_keeps_its_old_load_behavior() {
    let source = "\
fn only_if(x, y) = x + y;
fn unused(x) = only_if(x > 0, 1);
state n = 0;
event go;
on go set n = n + 1;
";
    // Older releases infer this unused body as the intrinsic, then expand
    // it as the authored function without checking its boolean argument.
    // Resolving only_if during inference would newly refuse this program.
    let mut session = ReactiveSession::from_source(source).unwrap();
    session.dispatch_json("go", "{}").unwrap();
    assert_eq!(session.snapshot().values["n"], 1.0);
}

#[test]
fn authored_selectors_inside_form_calls_are_inferred_too() {
    for name in ["resolved", "outcome"] {
        let source = format!(
            "fn a_call(x) = {name}(revision(x, 2));\n\
             fn {name}(x) = x + 1;\nfn revision(x, y) = x + y;\n"
        );
        let library = SourceLibrary::from_module_source(&source)
            .unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(
            library.call_numbers("a_call", &[4.0]).unwrap().value,
            FunctionValue::Number(7.0),
            "{source}"
        );
    }
    let source = "\
fn a_call(x) = in_basis(revision(x, 2), occurrence(x, 3));
fn in_basis(x, y) = x + y;
fn revision(x, y) = x + y;
fn occurrence(x, y) = x + y;
";
    let library = SourceLibrary::from_module_source(source).unwrap();
    assert_eq!(
        library.call_numbers("a_call", &[4.0]).unwrap().value,
        FunctionValue::Number(13.0)
    );
}

#[test]
fn a_session_using_an_authored_is_set_function_saves_and_continues() {
    let source = "\
fn is_set(x) = x + 1;
fn wrapped(x) = is_set(x);
state n = 0;
event go;
on go set n = wrapped(n);
";
    let mut session = ReactiveSession::from_source(source).unwrap();
    session.dispatch_json("go", "{}").unwrap();
    let mut restored = ReactiveSession::restore(source, &session.save().unwrap()).unwrap();
    for _ in 0..2 {
        session.dispatch_json("go", "{}").unwrap();
        restored.dispatch_json("go", "{}").unwrap();
    }
    assert_eq!(session.snapshot().values["n"], 3.0);
    assert_eq!(
        serde_json::to_value(session.snapshot()).unwrap(),
        serde_json::to_value(restored.snapshot()).unwrap()
    );
}

#[test]
fn authored_form_calls_still_check_arity_cycles_types_and_captures() {
    for name in UNARY {
        for (source, message) in [
            (
                format!("fn {name}(x, y) = x;\nfn a_call(x) = {name}(x);\n"),
                "requires 2 arguments",
            ),
            (
                format!("fn {name}(x) = a_call(x);\nfn a_call(x) = {name}(x);\n"),
                "recursive source function cycle",
            ),
            (
                format!("fn {name}(x) = x;\nfn a_call() = {name}(missing);\n"),
                "unknown numeric identifier missing",
            ),
            (
                format!("fn {name}(x) = x > 0;\nfn a_call(x) = {name}(x) + 1;\n"),
                "addition requires two numbers or two texts",
            ),
        ] {
            let error = SourceLibrary::from_module_source(&source).unwrap_err();
            assert!(error.contains(message), "{source}: {error}");
        }
    }
    for form in [
        "is_set(x)",
        "on_hold(x)",
        "grounds_of(x)",
        "resolved(x)",
        "outcome(x)",
        "in_basis(x, x)",
    ] {
        let source = format!("fn a_call(x) = {form};\n");
        let error = SourceLibrary::from_module_source(&source).unwrap_err();
        assert!(
            error.contains("cannot query graph predicates"),
            "{source}: {error}"
        );
    }
}
