//! `sample STREAM = VALUE ... with CAVEAT[, CAVEAT...] [when CONDITION]`: a
//! caveat on the one reading a sample takes, under a condition the reading
//! then rests on. See spec/caveat-reading-caveats-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeSet;

/// The graded-belief note's robot: a belief in the middle band is taken with
/// `uncertain`, and an irreversible plan may not rest on it unretained.
const ROBOT: &str = r#"
claim door_open;
evidence estimator from "the robot's state estimator: probability the door is open";
evidence lidar from "the lidar's clearance estimate";
caveat uncertain consequence material;
caveat dusty consequence low;
dusty qualifies lidar;
readings beliefs from estimator limit 32;
decisions plan limit 1 irreversible;
decisions careful limit 1 irreversible;
state clearance = 0 min 0 max 10;
state last = 0;
event scan;
event belief p min 0 max 1;
event near p min 0 max 1;
event go;
event go_careful;
event read;
on scan reveal lidar;
on scan set clearance = qualified(5, lidar);
on belief sample beliefs = p supports door_open with uncertain when p > 0.3 and p < 0.9;
on near sample beliefs = p supports door_open with uncertain when clearance < 3;
on read set last = latest(beliefs);
on go commit plan because enough using latest(beliefs);
on go_careful commit careful because enough using latest(beliefs) retaining uncertain;
"#;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("{error}"))
}

fn outcome(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

fn accept(session: &mut ReactiveSession, event: &str, payload: &str) {
    let outcome = outcome(session, event, payload);
    assert_eq!(outcome["outcome"], "accepted", "{outcome}");
}

fn send(session: &mut ReactiveSession, event: &str) {
    accept(session, event, "{}");
}

fn belief(session: &mut ReactiveSession, p: f64) {
    accept(session, "belief", &format!(r#"{{"p":{p}}}"#));
}

/// The caveats of the latest reading, read into state.
fn caveats(session: &mut ReactiveSession) -> BTreeSet<String> {
    send(session, "read");
    session.snapshot().qualified_values["last"]
        .provenance
        .caveats
        .iter()
        .cloned()
        .collect()
}

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn load_error(source: &str) -> String {
    match ReactiveSession::from_source(source) {
        Ok(_) => panic!("loaded: {source}"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn the_caveat_is_on_the_reading_its_condition_holds_for() {
    let mut session = session(ROBOT);
    belief(&mut session, 0.95);
    assert_eq!(caveats(&mut session), names(&[]));
    belief(&mut session, 0.5);
    assert_eq!(caveats(&mut session), names(&["uncertain"]));
    belief(&mut session, 0.1);
    assert_eq!(caveats(&mut session), names(&[]));
}

#[test]
fn the_caveat_is_on_that_reading_alone() {
    let mut session = session(ROBOT);
    belief(&mut session, 0.5);
    belief(&mut session, 0.95);
    let snapshot = session.snapshot();
    let occurrences = &snapshot.reading_streams["beliefs"].occurrences;
    let carried = occurrences
        .iter()
        .map(|occurrence| occurrence.provenance.caveats.iter().cloned().collect())
        .collect::<Vec<BTreeSet<String>>>();
    assert_eq!(carried, vec![names(&["uncertain"]), names(&[])]);
    // Neither the template nor the claim is qualified.
    let snapshot = serde_json::to_value(&snapshot).unwrap();
    let relations = snapshot["relations"].as_array().unwrap();
    assert!(
        relations
            .iter()
            .all(|relation| relation["from"] != "uncertain" || relation["to"] == "beliefs@1"),
        "{relations:?}"
    );
}

#[test]
fn an_irreversible_commit_on_an_uncertain_reading_is_refused_unless_retained() {
    let mut session = session(ROBOT);
    belief(&mut session, 0.5);
    let refused = outcome(&mut session, "go", "{}");
    assert_eq!(refused["outcome"], "rejected", "{refused}");
    assert_eq!(refused["code"], "caveated_irreversible", "{refused}");
    assert!(
        refused["message"].as_str().unwrap().ends_with(
            "irreversible plan rests on uncertain (material), which its commit does not retain"
        ),
        "{refused}"
    );
    send(&mut session, "go_careful");
    // A sure reading commits without retaining anything.
    let mut session = self::session(ROBOT);
    belief(&mut session, 0.95);
    send(&mut session, "go");
}

#[test]
fn the_condition_is_read_whichever_way_it_falls() {
    // As a guard's lineage is: spec/caveat-reactive-0.5.md, "A sample is a
    // new observation".
    let mut session = session(ROBOT);
    send(&mut session, "scan");
    accept(&mut session, "near", r#"{"p":0.5}"#);
    let snapshot = session.snapshot();
    let observation = &snapshot.observation_qualifications["beliefs@1"];
    assert_eq!(observation.evidence, names(&["lidar"]));
    assert_eq!(caveats(&mut session), names(&["dusty"]));

    let mut session = self::session(ROBOT);
    accept(&mut session, "near", r#"{"p":0.5}"#);
    assert_eq!(caveats(&mut session), names(&["uncertain"]));
}

#[test]
fn without_a_condition_every_reading_carries_the_caveats() {
    let source = ROBOT.replace(
        "with uncertain when p > 0.3 and p < 0.9",
        "with uncertain, dusty",
    );
    let mut session = session(&source);
    belief(&mut session, 0.95);
    assert_eq!(caveats(&mut session), names(&["dusty", "uncertain"]));
}

#[test]
fn a_procedure_may_take_a_reading_with_caveats() {
    let source = r#"
evidence estimator from "estimator";
caveat uncertain consequence material;
readings beliefs from estimator limit 8;
state last = 0;
event belief p min 0 max 1;
event read;
proc take(p) {
    sample beliefs = p with uncertain when p < 0.9;
};
on belief call take(p);
on read set last = latest(beliefs);
"#;
    let mut session = session(source);
    belief(&mut session, 0.5);
    assert_eq!(caveats(&mut session), names(&["uncertain"]));
}

#[test]
fn the_save_round_trips_and_restores_the_reading_caveats() {
    let mut session = session(ROBOT);
    send(&mut session, "scan");
    belief(&mut session, 0.5);
    belief(&mut session, 0.95);
    send(&mut session, "read");
    let save = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(ROBOT, &save).unwrap();
    assert_eq!(restored.save_json().unwrap(), save);
    assert_eq!(restored.snapshot(), session.snapshot());
}

#[test]
fn what_does_not_load() {
    let base = |sample: &str| {
        format!(
            r#"
evidence estimator from "estimator";
claim open;
caveat uncertain consequence material;
caveat later consequence low;
readings beliefs from estimator limit 8;
state level = 0;
event belief p min 0 max 1;
event doubt;
on belief {sample};
on doubt unqualify latest(beliefs) with later;
on doubt qualify latest(beliefs) with later;
"#
        )
    };
    let undeclared = load_error(&base("sample beliefs = p with unknown"));
    assert!(undeclared.contains("unknown"), "{undeclared}");
    let not_a_caveat = load_error(&base("sample beliefs = p with open"));
    assert!(not_a_caveat.contains("open"), "{not_a_caveat}");
    let take_back = load_error(&base("sample beliefs = p with later"));
    assert!(
        take_back.contains("sample with cannot name later: it is taken back by unqualify"),
        "{take_back}"
    );
    let numeric = load_error(&base("sample beliefs = p with uncertain when p"));
    assert!(
        numeric.contains("sample beliefs with ... when requires a boolean condition"),
        "{numeric}"
    );
    let twice = load_error(&base("sample beliefs = p with uncertain, uncertain"));
    assert!(twice.contains("sample expects"), "{twice}");
    let empty = load_error(&base("sample beliefs = p with uncertain when"));
    assert!(empty.contains("sample expects"), "{empty}");
    let withdrawn = load_error(
        &base("sample beliefs = p with withdrawn").replace(
            "on doubt unqualify",
            "evidence recheck from \"a recheck\";\non doubt withdraw latest(beliefs) because recheck;\non doubt unqualify",
        ),
    );
    assert!(
        withdrawn.contains("sample with cannot apply withdrawn"),
        "{withdrawn}"
    );
}

#[test]
fn with_after_the_stance_and_the_note() {
    let source = r#"
claim open;
evidence estimator from "estimator";
caveat uncertain consequence material;
readings beliefs from estimator limit 8;
identifiers limit 8;
state last = 0;
event belief p min 0 max 1, why id;
event read;
on belief sample beliefs = p opposes open noting why with uncertain when p < 0.9;
on read set last = latest(beliefs);
"#;
    let mut session = session(source);
    accept(&mut session, "belief", r#"{"p":0.5,"why":"check"}"#);
    let snapshot = session.snapshot();
    let occurrence = &snapshot.reading_streams["beliefs"].occurrences[0];
    assert_eq!(occurrence.relation.as_deref(), Some("opposes"));
    assert_eq!(occurrence.note.as_deref(), Some("check"));
    assert_eq!(caveats(&mut session), names(&["uncertain"]));
}

#[test]
fn a_state_named_with_is_still_a_value() {
    // `with` is a word only where caveats follow it.
    let source = r#"
evidence estimator from "estimator";
readings beliefs from estimator limit 8;
state with = 2;
state last = 0;
event belief;
event read;
on belief sample beliefs = with;
on belief sample beliefs = 1 + with;
on read set last = latest(beliefs);
"#;
    let mut session = session(source);
    send(&mut session, "belief");
    send(&mut session, "read");
    assert_eq!(session.snapshot().values["last"], 3.0);
}

#[test]
fn a_procedure_may_name_the_caveat_and_the_stream_as_parameters() {
    let source = r#"
evidence estimator from "estimator";
caveat uncertain consequence material;
caveat rough consequence low;
readings beliefs from estimator limit 8;
state last = 0;
event belief p min 0 max 1;
event read;
proc take(s readings, p, c caveat) {
    sample s = p with c when p < 0.9;
};
on belief call take(beliefs, p, uncertain);
on belief call take(beliefs, p, rough);
on read set last = latest(beliefs);
"#;
    let mut session = session(source);
    belief(&mut session, 0.5);
    assert_eq!(caveats(&mut session), names(&["rough"]));
    let occurrences = &session.snapshot().reading_streams["beliefs"].occurrences;
    assert_eq!(
        occurrences[0].provenance.caveats,
        names(&["uncertain"]),
        "{occurrences:?}"
    );
    let save = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(source, &save).unwrap();
    assert_eq!(restored.snapshot(), session.snapshot());
}
