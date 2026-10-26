//! `only_if(CONDITION, VALUE)`: VALUE when CONDITION holds, else 0, with the
//! condition's reads in the lineage and not in the grounds. See
//! spec/caveat-only-if-0.1.md and the model-votes experiment.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn snapshot(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.snapshot()).unwrap()
}

fn run(session: &mut ReactiveSession, events: &[(&str, &str)]) {
    for (event, payload) in events {
        session
            .dispatch_json(event, payload)
            .unwrap_or_else(|error| panic!("{event} {payload}: {error}"));
    }
}

/// Three voters, a decision on the votes that agreed, and a state set the
/// same way.
const SOURCE: &str = r#"
evidence a_out from "what a answered";
evidence b_out from "what b answered";
evidence c_out from "what c answered";
evidence a_bad from "a finding that a is unreliable";
evidence c_bad from "a finding that c is unreliable";
readings a from a_out limit 1;
readings b from b_out limit 1;
readings c from c_out limit 1;
decisions approve limit 2;
state tally = 0;
event vote_a v min 0 max 1;
event vote_b v min 0 max 1;
event vote_c v min 0 max 1;
event decide;
event count;
event distrust_a;
event distrust_c;
define a_yes = has_sample(a) and latest(a) == 1 and not withdrawn(latest(a));
define b_yes = has_sample(b) and latest(b) == 1 and not withdrawn(latest(b));
define c_yes = has_sample(c) and latest(c) == 1 and not withdrawn(latest(c));
define yes = if(a_yes, 1, 0) + if(b_yes, 1, 0) + if(c_yes, 1, 0);
on vote_a reveal a_out;
on vote_a sample a = v;
on vote_b reveal b_out;
on vote_b sample b = v;
on vote_c reveal c_out;
on vote_c sample c = v;
on decide when yes < 2 reject "Fewer than two agree.";
on decide commit approve because enough using
    only_if(a_yes, latest(a)) + only_if(b_yes, latest(b)) + only_if(c_yes, latest(c));
on count set tally = only_if(a_yes, latest(a)) + only_if(b_yes, latest(b)) + only_if(c_yes, latest(c));
on distrust_a reveal a_bad;
on distrust_a withdraw latest(a) because a_bad;
on distrust_c reveal c_bad;
on distrust_c withdraw latest(c) because c_bad;
"#;

const VOTES: [(&str, &str); 3] = [
    ("vote_a", r#"{"v":1}"#),
    ("vote_b", r#"{"v":1}"#),
    ("vote_c", r#"{"v":0}"#),
];

fn evidence(value: &Value) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_commit_rests_on_the_values_its_conditions_chose() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    run(&mut session, &VOTES);
    run(&mut session, &[("decide", "{}")]);
    let view = snapshot(&session);
    assert_eq!(
        evidence(&view["commitment_grounds"]["approve@1"]["evidence"]),
        ["a@1", "b@1"]
    );
    let lineage = evidence(&view["commitment_bases"]["approve@1"]["provenance"]["evidence"]);
    assert!(lineage.contains(&"c@1".to_string()), "{lineage:?}");
    assert_eq!(view["commitment_bases"]["approve@1"]["value"], json!(2.0));
}

#[test]
fn a_state_rests_on_the_values_its_conditions_chose() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    run(&mut session, &VOTES);
    run(&mut session, &[("count", "{}")]);
    let view = snapshot(&session);
    assert_eq!(view["values"]["tally"], json!(2.0));
    assert_eq!(
        evidence(&view["value_grounds"]["tally"]["evidence"]),
        ["a@1", "b@1"]
    );
    assert_eq!(
        evidence(&view["qualified_values"]["tally"]["provenance"]["evidence"]),
        ["a@1", "b@1", "c@1"]
    );
}

#[test]
fn a_false_condition_is_zero_and_does_not_evaluate_its_value() {
    // latest(b) before any sample would fail; the condition keeps it unread.
    let source = r#"
evidence b_out from "b";
readings b from b_out limit 1;
state picked = 5;
event pick;
on pick set picked = only_if(has_sample(b), latest(b)) + only_if(1 > 2, 1 / 0);
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(&mut session, &[("pick", "{}")]);
    let view = snapshot(&session);
    assert_eq!(view["values"]["picked"], json!(0.0));
    assert_eq!(
        evidence(&view["value_grounds"]["picked"]["evidence"]),
        Vec::<String>::new()
    );
}

#[test]
fn withdrawing_a_chosen_vote_reaches_the_decision_and_a_dissenting_one_does_not() {
    let source = format!(
        "{SOURCE}\nevent check;\nstate rests = 0;\non check set rests = if(rests_on_withdrawn(approve), 1, 0);"
    );
    let mut session = ReactiveSession::from_source(&source).unwrap();
    run(&mut session, &VOTES);
    run(
        &mut session,
        &[("decide", "{}"), ("distrust_c", "{}"), ("check", "{}")],
    );
    assert_eq!(snapshot(&session)["values"]["rests"], json!(0.0));
    run(&mut session, &[("distrust_a", "{}"), ("check", "{}")]);
    assert_eq!(snapshot(&session)["values"]["rests"], json!(1.0));
}

#[test]
fn a_nested_condition_stays_out_of_the_grounds() {
    let source = r#"
evidence a_out from "a";
evidence b_out from "b";
readings a from a_out limit 1;
readings b from b_out limit 1;
state picked = 0;
event go;
on go reveal a_out;
on go sample a = 1;
on go reveal b_out;
on go sample b = 1;
on go set picked = only_if(only_if(latest(b) == 1, 1) == 1, latest(a));
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(&mut session, &[("go", "{}")]);
    let view = snapshot(&session);
    assert_eq!(view["values"]["picked"], json!(1.0));
    assert_eq!(
        evidence(&view["value_grounds"]["picked"]["evidence"]),
        ["a@1"]
    );
}

#[test]
fn only_if_survives_save_and_restore() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    run(&mut session, &VOTES);
    run(
        &mut session,
        &[("decide", "{}"), ("count", "{}"), ("distrust_a", "{}")],
    );
    let saved = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(SOURCE, &saved).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&session));
    assert_eq!(restored.save_json().unwrap(), saved);
}

#[test]
fn a_program_that_defines_only_if_keeps_its_function() {
    let source = r#"
fn only_if(x, y) = x * 10 + y;
state picked = 0;
event go;
on go set picked = only_if(2, 3);
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(&mut session, &[("go", "{}")]);
    assert_eq!(snapshot(&session)["values"]["picked"], json!(23.0));
}

#[test]
fn only_if_takes_a_condition_and_a_number() {
    for (body, message) in [
        ("only_if(1, 2)", "bool"),
        ("only_if(true, \"x\")", "number"),
        ("only_if(true)", "only_if"),
    ] {
        let source = format!("state picked = 0;\nevent go;\non go set picked = {body};\n");
        let error = ReactiveSession::from_source(&source).expect_err(body);
        assert!(error.to_lowercase().contains(message), "{body}: {error}");
    }
}
