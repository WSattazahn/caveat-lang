//! D034: a source function copied each argument wherever its body read the
//! parameter, so `min(a, b) = if(a < b, a, b)` doubled its arguments at every
//! level. `min` nested to the right over eight values passed the 4,096-node
//! expansion limit and the program did not load, although the same `min`
//! written balanced loaded (found by the third agent test, 2026-10-31). Each
//! argument is now expanded and evaluated once and the body reads it by
//! position, so nesting grows linearly. Values, lineage and grounds are those
//! of the balanced form. See `Node::Argument` in `reactive_expr.rs`.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

fn right_nested(names: &[String]) -> String {
    let mut expression = names[names.len() - 1].clone();
    for name in names[..names.len() - 1].iter().rev() {
        expression = format!("min({name}, {expression})");
    }
    expression
}

fn balanced(names: &[String]) -> String {
    if names.len() == 1 {
        return names[0].clone();
    }
    let (left, right) = names.split_at(names.len() / 2);
    format!("min({}, {})", balanced(left), balanced(right))
}

/// `count` sampled readings, each from its own evidence, and `low` set to
/// their minimum by `expression`.
fn program(count: usize, expression: impl Fn(&[String]) -> String) -> String {
    let names: Vec<String> = (0..count).map(|i| format!("part{i}")).collect();
    let mut source = String::from("claim sound;\nstate low = 0;\nevent rate i min 0 max 64, v min 0 max 100;\nevent settle;\n");
    for (i, name) in names.iter().enumerate() {
        source.push_str(&format!(
            "evidence {name}_report from \"{name}\";\nreadings {name}_r from {name}_report limit 8;\nstate {name} = 100;\n\
             on rate when i == {i} sample {name}_r = v supports sound;\n\
             on rate when i == {i} set {name} = latest({name}_r);\n"
        ));
    }
    source.push_str(&format!("on settle set low = {};\n", expression(&names)));
    source
}

fn dispatch(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

/// `low` as the snapshot gives it, with everything the snapshot says about it.
fn low(count: usize, expression: impl Fn(&[String]) -> String) -> Value {
    let source = program(count, expression);
    let mut session = ReactiveSession::from_source(&source)
        .unwrap_or_else(|error| panic!("{count} values did not load: {error}"));
    for i in 0..count {
        let payload = format!("{{\"i\": {i}, \"v\": {}}}", 90 - (i * 37) % 80);
        assert_eq!(
            dispatch(&mut session, "rate", &payload)["outcome"],
            "accepted"
        );
    }
    assert_eq!(
        dispatch(&mut session, "settle", "{}")["outcome"],
        "accepted"
    );
    let snapshot = serde_json::to_value(session.snapshot()).unwrap();
    serde_json::json!({
        "value": snapshot["values"]["low"],
        "qualified": snapshot["qualified_values"]["low"],
        "grounds": snapshot["value_grounds"]["low"],
    })
}

#[test]
fn d034_min_nested_to_the_right_loads_and_agrees_with_the_balanced_form() {
    // 7 loaded before the fix; 8 and up did not.
    for count in [2, 7, 8, 16, 32] {
        let right = low(count, right_nested);
        let even = low(count, balanced);
        assert!(
            !right["qualified"].is_null(),
            "low carries its lineage: {right}"
        );
        assert_eq!(right, even, "{count} values");
        let expected = (0..count).map(|i| 90 - (i * 37) % 80).min().unwrap() as f64;
        assert_eq!(right["value"].as_f64(), Some(expected), "{right}");
    }
}

#[test]
fn d034_a_parameter_read_many_times_is_expanded_once() {
    // Each level reads its parameter four times. Copying would need 4^8
    // nodes; reading by position needs a few per level.
    let mut source =
        String::from("fn spread(x) = x + x + x + x;\nstate a = 1;\nstate out = 0;\nevent go;\n");
    let mut expression = String::from("a");
    for _ in 0..8 {
        expression = format!("spread({expression})");
    }
    source.push_str(&format!("on go set out = {expression} / 65536;\n"));
    let mut session = ReactiveSession::from_source(&source).unwrap();
    assert_eq!(dispatch(&mut session, "go", "{}")["outcome"], "accepted");
    let snapshot = serde_json::to_value(session.snapshot()).unwrap();
    assert_eq!(snapshot["values"]["out"].as_f64(), Some(1.0));
}

#[test]
fn d034_an_unused_argument_is_still_evaluated() {
    // Arguments stay eager: `constant(1 / 0)` still fails although the body
    // never reads its parameter.
    let source = "fn constant(x) = 7;\nstate out = 0;\nevent go;\nevent bad;\n\
                  on go set out = constant(out + 1);\non bad set out = constant(1 / 0);\n";
    let mut session = ReactiveSession::from_source(source).unwrap();
    assert_eq!(dispatch(&mut session, "go", "{}")["outcome"], "accepted");
    let refused = dispatch(&mut session, "bad", "{}");
    assert_eq!(refused["outcome"], "rejected", "{refused}");
}
