//! D024: a reading sampled from renewable evidence inherits the caveats that
//! qualify the occurrence current when it is taken, not only the declared
//! evidence's. See spec/caveat-renewal-0.1.md ("Renewable evidence") and
//! spec/caveat-reactive-0.5.md ("A sample is a new observation").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeSet;

/// The graded-belief thread's guardrail: each assessment renews the guard,
/// flags it when the risk is high, and samples the risk from it. `again`
/// samples from the current occurrence without renewing it.
const GUARDRAIL: &str = r#"
claim action_is_safe;
evidence guard from "the guardrail model's estimated probability of harm";
renewable guard limit 64;
caveat guard_flags_risk consequence high;
caveat stale consequence material;
readings risks from guard limit 64;
state last = 0;
event assess p min 0 max 1;
event again;
event doubt;
event read;
on assess renew guard;
on assess reveal guard;
on assess when p >= 0.2 qualify guard with guard_flags_risk;
on assess sample risks = p;
on again sample risks = 0.5;
on doubt qualify guard with stale;
on read set last = latest(risks);
"#;

fn session() -> ReactiveSession {
    ReactiveSession::from_source(GUARDRAIL).unwrap()
}

fn accept(session: &mut ReactiveSession, event: &str, payload: &str) {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome: Value = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{outcome}");
}

fn assess(session: &mut ReactiveSession, p: f64) {
    accept(session, "assess", &format!(r#"{{"p":{p}}}"#));
}

fn send(session: &mut ReactiveSession, event: &str) {
    accept(session, event, "{}");
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

#[test]
fn a_reading_carries_the_flag_raised_on_its_occurrence() {
    let mut session = session();
    assess(&mut session, 0.1);
    assert_eq!(caveats(&mut session), names(&[]));
    // The flag is raised on guard@3 in the same event that samples from it.
    assess(&mut session, 0.9);
    assert_eq!(caveats(&mut session), names(&["guard_flags_risk"]));
    // A fresh occurrence starts clean, and so does what is read from it.
    assess(&mut session, 0.1);
    assert_eq!(caveats(&mut session), names(&[]));
}

#[test]
fn a_reading_carries_a_doubt_raised_earlier_on_the_current_occurrence() {
    let mut session = session();
    assess(&mut session, 0.1);
    send(&mut session, "doubt");
    send(&mut session, "again");
    assert_eq!(caveats(&mut session), names(&["stale"]));
    // The doubt was about guard@2. The next assessment renews to guard@3,
    // whose reading does not carry it.
    assess(&mut session, 0.1);
    assert_eq!(caveats(&mut session), names(&[]));
}

#[test]
fn the_reading_taken_before_a_doubt_keeps_its_own_caveats() {
    let mut session = session();
    assess(&mut session, 0.1);
    send(&mut session, "again");
    assert_eq!(caveats(&mut session), names(&[]));
    // A doubt raised after the sample is about the occurrence, not about the
    // reading already taken from it.
    send(&mut session, "doubt");
    assert_eq!(caveats(&mut session), names(&[]));
}

#[test]
fn the_declared_caveats_and_the_first_occurrence_are_unchanged() {
    // Caveats declared on the evidence still reach every reading, and a late
    // caveat on the first occurrence still reaches later readings as before:
    // nothing is taken away in 1.x.
    let source = r#"
evidence guard from "guardrail";
renewable guard limit 8;
caveat flag consequence high;
caveat secondhand consequence material;
secondhand qualifies guard;
readings risks from guard limit 8;
state last = 0;
event pre;
event assess p min 0 max 1;
event read;
on pre reveal guard;
on pre qualify guard with flag;
on assess renew guard;
on assess reveal guard;
on assess sample risks = p;
on read set last = latest(risks);
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    send(&mut session, "pre");
    assess(&mut session, 0.1);
    assert_eq!(caveats(&mut session), names(&["flag", "secondhand"]));
}

#[test]
fn the_save_round_trips_and_restores_the_inherited_caveats() {
    let mut session = session();
    assess(&mut session, 0.9);
    send(&mut session, "read");
    let save = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(GUARDRAIL, &save).unwrap();
    assert_eq!(restored.save_json().unwrap(), save);
    assert_eq!(restored.snapshot(), session.snapshot());
}
