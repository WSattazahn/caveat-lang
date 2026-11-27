//! Across forms must not take an older call whose first argument is named `for`.

use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::repeat;

fn round_trip(source: &str, expected: f64) {
    assert_eq!(repeat::expand(source).unwrap(), source);
    let mut session = ReactiveSession::from_source(source).unwrap();
    assert_eq!(session.snapshot().values["result"], expected);
    let saved = session.save_json().unwrap();
    let mut restored = ReactiveSession::restore_json(source, &saved).unwrap();
    session.dispatch_json("go", "{}").unwrap();
    restored.dispatch_json("go", "{}").unwrap();
    assert_eq!(session.snapshot().values["result"], expected + 1.0);
    assert_eq!(session.save_json().unwrap(), restored.save_json().unwrap());
}

#[test]
fn ordinary_calls_read_a_variable_named_for_in_all_three_functions() {
    for function in ["sum", "any", "all"] {
        for (argument, expected) in [
            ("for + 1", 4.0),
            ("for - 1", 2.0),
            ("for * 2", 6.0),
            ("for / 2", 1.5),
            ("for ", 3.0),
            ("for\n", 3.0),
            ("for # the old numeric argument\n + 1", 4.0),
            ("for // the old numeric argument\n + 1", 4.0),
        ] {
            let source = format!(
                "fn {function}(x) = x;\nstate for = 3;\n\
                 state result = {function}({argument});\nevent go;\n\
                 on go set result = result + 1;\n"
            );
            round_trip(&source, expected);
        }
    }
}

#[test]
fn for_can_be_a_function_or_the_first_of_multiple_arguments() {
    for function in ["sum", "any", "all"] {
        round_trip(
            &format!(
                "fn for(x) = x + 10;\nfn {function}(x) = x;\n\
                 state result = {function}(for (2));\nevent go;\n\
                 on go set result = result + 1;\n"
            ),
            12.0,
        );
        round_trip(
            &format!(
                "fn {function}(x, y) = x + y;\nstate for = 3;\n\
                 state result = {function}(for , 2);\nevent go;\n\
                 on go set result = result + 1;\n"
            ),
            5.0,
        );
        round_trip(
            &format!(
                "fn {function}(x) = x;\nfn apply(for) = {function}(for + 1);\n\
                 state result = apply(3);\nevent go;\n\
                 on go set result = result + 1;\n"
            ),
            4.0,
        );
    }
}

#[test]
fn an_event_parameter_named_for_stays_an_ordinary_argument() {
    for function in ["sum", "any", "all"] {
        let source = format!(
            "fn {function}(x) = x;\nstate result = 0;\n\
             event go for min 0 max 10;\non go set result = {function}(for + 1);\n"
        );
        assert_eq!(repeat::expand(&source).unwrap(), source);
        let mut session = ReactiveSession::from_source(&source).unwrap();
        session.dispatch_json("go", r#"{"for":2}"#).unwrap();
        assert_eq!(session.snapshot().values["result"], 3.0);
    }
}

#[test]
fn an_ordinary_outer_call_can_hold_a_real_across_form() {
    for function in ["sum", "any", "all"] {
        let source = format!(
            "place field kind ground;\nentity north kind member at field;\n\
             entity south kind member at field;\nfn {function}(x) = x;\n\
             state for = 3;\nstate result = {function}(for + sum(for member as $m: $index));\nevent go;\n"
        );
        let expanded = repeat::expand(&source).unwrap();
        assert!(expanded.contains(&format!("{function}(for + ((1) + (2)))")));
        assert_eq!(
            ReactiveSession::from_source(&source)
                .unwrap()
                .snapshot()
                .values["result"],
            6.0
        );
    }
}

#[test]
fn boolean_operator_words_can_still_name_real_across_kinds() {
    for kind in ["and", "or"] {
        let source = format!(
            "place field kind ground;\nentity north kind {kind} at field;\n\
             entity south kind {kind} at field;\n\
             state result = sum(for {kind} as $m: $index);\nevent go;\n"
        );
        assert_eq!(
            ReactiveSession::from_source(&source)
                .unwrap()
                .snapshot()
                .values["result"],
            3.0
        );
    }
    for operator in ["and", "or"] {
        let source = format!("fn sum(x) = x;\ndefine result = sum(for {operator} other);\n");
        assert_eq!(repeat::expand(&source).unwrap(), source);
    }
}

#[test]
fn malformed_headers_for_operator_named_kinds_are_still_refused() {
    for kind in ["and", "or"] {
        for tail in [
            "as member: 1)",
            "$m: 1)",
            "as $m 1)",
            "as $m: )",
            "as $m: (1)",
        ] {
            let source = format!(
                "place field kind ground;\nentity north kind {kind} at field;\n\
                 state result = sum(for {kind} {tail};\nevent go;\n"
            );
            assert!(ReactiveSession::from_source(&source).is_err(), "{source}");
        }
    }
}
