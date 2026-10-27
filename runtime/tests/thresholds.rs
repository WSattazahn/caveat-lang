//! `threshold NAME = NUMBER;`: a named number a commitment lists, with its
//! value, when its commit names it. See spec/caveat-thresholds-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// The graded-belief note's robot: go through, look again or replan, by two
/// bars the record now names.
const ROBOT: &str = r#"
claim door_open;
evidence estimator from "the robot's state estimator: probability the door ahead is open";
readings beliefs from estimator limit 32;
decisions plan limit 32 reopened by beliefs;
threshold go_bar = 0.9;
threshold look_bar = 0.3;
event belief p min 0 max 1;
on belief sample beliefs = p supports door_open;
on belief when not committed(plan) or reopened(plan)
    commit plan because enough
    using if(latest(beliefs) >= go_bar, 2, if(latest(beliefs) >= look_bar, 1, 0));
"#;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("{error}"))
}

fn accept(session: &mut ReactiveSession, event: &str, payload: &str) {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{outcome}");
}

fn belief(session: &mut ReactiveSession, p: f64) {
    accept(session, "belief", &format!(r#"{{"p":{p}}}"#));
}

fn bases(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.snapshot()).unwrap()["commitment_bases"].clone()
}

fn load_error(source: &str) -> String {
    match ReactiveSession::from_source(source) {
        Ok(_) => panic!("loaded: {source}"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn each_revision_lists_the_bars_its_commit_named() {
    let mut session = session(ROBOT);
    belief(&mut session, 0.55);
    belief(&mut session, 0.95);
    let bases = bases(&session);
    let bars = json!({"go_bar": 0.9, "look_bar": 0.3});
    assert_eq!(bases["plan@1"]["value"], 1.0);
    assert_eq!(bases["plan@1"]["thresholds"], bars);
    assert_eq!(bases["plan@2"]["value"], 2.0);
    assert_eq!(bases["plan@2"]["thresholds"], bars);
}

#[test]
fn a_threshold_reads_as_its_number_and_adds_no_lineage() {
    let literal = ROBOT
        .replace("threshold go_bar = 0.9;\n", "")
        .replace("threshold look_bar = 0.3;\n", "")
        .replace("go_bar", "0.9")
        .replace("look_bar", "0.3");
    let mut named = session(ROBOT);
    let mut written = session(&literal);
    for p in [0.55, 0.95, 0.2] {
        belief(&mut named, p);
        belief(&mut written, p);
    }
    let mut with_names = bases(&named);
    for basis in with_names.as_object_mut().unwrap().values_mut() {
        basis.as_object_mut().unwrap().remove("thresholds");
    }
    assert_eq!(with_names, bases(&written));
    assert!(!bases(&written).to_string().contains("thresholds"));
}

#[test]
fn a_guard_a_define_and_a_calling_rule_count() {
    let source = r#"
evidence probe from "probe";
readings r from probe limit 8;
threshold floor = 0.2;
threshold ceiling = 0.8;
threshold step = 1;
define high_enough = latest(r) >= floor;
decisions a limit 4;
decisions b limit 4;
decisions c limit 4;
event go p min 0 max 1;
proc decide_c() {
    commit c because enough using latest(r) + step;
};
on go sample r = p;
on go when high_enough commit a because enough using latest(r);
on go when latest(r) <= ceiling commit b because enough using latest(r);
on go when latest(r) <= ceiling call decide_c();
"#;
    let mut session = session(source);
    accept(&mut session, "go", r#"{"p":0.5}"#);
    let bases = bases(&session);
    assert_eq!(bases["a@1"]["thresholds"], json!({"floor": 0.2}));
    assert_eq!(bases["b@1"]["thresholds"], json!({"ceiling": 0.8}));
    assert_eq!(
        bases["c@1"]["thresholds"],
        json!({"ceiling": 0.8, "step": 1.0})
    );
}

#[test]
fn a_threshold_read_into_state_earlier_is_not_listed() {
    // The state is what the commit reads; its value is in the record.
    let source = r#"
threshold bar = 0.5;
state level = 0 min 0 max 1;
decisions d limit 2;
event set_it;
event go;
on set_it set level = bar;
on go commit d because enough using level;
"#;
    let mut session = session(source);
    accept(&mut session, "set_it", "{}");
    accept(&mut session, "go", "{}");
    let bases = bases(&session);
    assert_eq!(bases["d@1"]["value"], 0.5);
    assert!(bases["d@1"].get("thresholds").is_none(), "{bases}");
}

#[test]
fn the_save_round_trips_and_restore_refuses_a_threshold_the_source_lacks() {
    let mut session = session(ROBOT);
    belief(&mut session, 0.55);
    let save = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(ROBOT, &save).unwrap();
    assert_eq!(restored.save_json().unwrap(), save);
    assert_eq!(restored.snapshot(), session.snapshot());

    for (from, to) in [
        (r#""go_bar":0.9"#, r#""go_bar":0.8"#),
        (r#""go_bar":0.9"#, r#""stop_bar":0.9"#),
    ] {
        assert!(save.contains(from), "{save}");
        let tampered = save.replace(from, to);
        let error = ReactiveSession::restore_json(ROBOT, &tampered)
            .err()
            .unwrap_or_else(|| panic!("restored {to}"))
            .to_string();
        assert!(
            error.contains("which the source does not declare"),
            "{error}"
        );
    }
}

#[test]
fn what_does_not_load() {
    let base = "state level = 0;\nevent go p min 0 max 1;\ndefine near = level;\n";
    for (line, expected) in [
        ("threshold bar = level;", "threshold expects NAME = NUMBER"),
        (
            "threshold bar = 0.5 + 1;",
            "threshold expects NAME = NUMBER",
        ),
        ("threshold bar;", "threshold expects NAME = NUMBER"),
        (
            "threshold level = 0.5;",
            "threshold level collides with a state",
        ),
        (
            "threshold p = 0.5;",
            "threshold p collides with an event parameter",
        ),
        ("threshold near = 0.5;", "duplicate threshold near"),
        (
            "threshold bar = 0.5;\nthreshold bar = 0.6;",
            "duplicate threshold bar",
        ),
    ] {
        let error = load_error(&format!("{base}{line}\n"));
        assert!(error.contains(expected), "{line}: {error}");
    }
}

#[test]
fn a_negative_threshold_and_one_unused_load() {
    let source = r#"
threshold low = -2.5;
threshold unused = 7;
state level = 0 min -10 max 10;
decisions d limit 2;
event go;
on go commit d because enough using low;
"#;
    let mut session = session(source);
    accept(&mut session, "go", "{}");
    let bases = bases(&session);
    assert_eq!(bases["d@1"]["value"], -2.5);
    assert_eq!(bases["d@1"]["thresholds"], json!({"low": -2.5}));
}

#[test]
fn a_state_named_threshold_is_still_a_state() {
    let source = r#"
state threshold = 1 min 0 max 9;
decisions d limit 2;
event go;
on go set threshold = threshold + 1;
on go commit d because enough using threshold;
"#;
    let mut session = session(source);
    accept(&mut session, "go", "{}");
    let bases = bases(&session);
    assert_eq!(bases["d@1"]["value"], 2.0);
    assert!(bases["d@1"].get("thresholds").is_none(), "{bases}");
}
