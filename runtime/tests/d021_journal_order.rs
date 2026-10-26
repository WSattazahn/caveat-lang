//! D021: a reopening with several causes listed them in the order they
//! reopened the decision, not the order they were first observed, so a save
//! made after it did not restore ("entry witnesses are unobserved, repeated,
//! or out of order"). Two ways reach it on 1.2.0 to 1.3.0: a withdrawal
//! breaking through a hold that deferred a later reading, and a release of
//! causes deferred out of observation order. See spec/caveat-holds-0.1.md
//! ("reopened because of every deferred cause, in the order they were
//! observed") and spec/caveat-decision-journal-0.1.md.
use caveat_runtime::reactive::ReactiveSession;

/// A hold defers a trigger reading; a withdrawal then breaks through it.
const THROUGH: &str = r#"
claim out_of_scope;
evidence review from "the finding that a rewrite is out of scope";
evidence recheck from "a finding that the review was wrong";
evidence brief from "the brief";
evidence scope_feed from "later scope reviews";
readings scope from scope_feed limit 8;
decisions no_rewrite limit 4 reopened by scope opposing out_of_scope;
event rule_out;
event hold_it;
event rescope;
event retract;
on rule_out reveal review supports out_of_scope;
on rule_out commit no_rewrite because enough citing qualified(1, review);
on hold_it reveal brief;
on hold_it hold no_rewrite because brief;
on rescope sample scope = 0 opposes out_of_scope;
on retract reveal recheck opposes out_of_scope;
on retract withdraw review because recheck;
on retract when committed(no_rewrite) and not reopened(no_rewrite) and rests_on_withdrawn(no_rewrite)
    reopen no_rewrite because recheck;
"#;

/// A hold defers a trigger reading, then a reopening by evidence observed
/// before it; the release applies both.
const RELEASED: &str = r#"
claim out_of_scope;
evidence review from "the finding that a rewrite is out of scope";
evidence old_note from "an earlier note against the plan";
evidence brief from "the brief";
evidence scope_feed from "later scope reviews";
readings scope from scope_feed limit 8;
decisions no_rewrite limit 4 reopened by scope opposing out_of_scope;
event rule_out;
event hold_it;
event rescope;
event retract;
on rule_out reveal review supports out_of_scope;
on rule_out reveal old_note opposes out_of_scope;
on rule_out commit no_rewrite because enough citing qualified(1, review);
on hold_it reveal brief;
on hold_it hold no_rewrite because brief;
on rescope sample scope = 0 opposes out_of_scope;
on retract reopen no_rewrite because old_note;
on retract release no_rewrite because brief;
"#;

fn reopened_because(source: &str) -> (Vec<String>, String) {
    let mut session = ReactiveSession::from_source(source).unwrap();
    for event in ["rule_out", "hold_it", "rescope", "retract"] {
        session
            .dispatch_json(event, "{}")
            .unwrap_or_else(|error| panic!("{event}: {error}"));
    }
    let snapshot = serde_json::to_value(session.snapshot()).unwrap();
    let entry = snapshot["decision_journal"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["change"] == "reopened")
        .expect("a reopening")
        .clone();
    let because = entry["because"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap().to_string())
        .collect();
    (because, session.save_json().unwrap())
}

fn restores(source: &str, saved: &str) {
    let restored = ReactiveSession::restore_json(source, saved)
        .unwrap_or_else(|error| panic!("restore: {error}"));
    assert_eq!(restored.save_json().unwrap(), saved);
}

#[test]
fn d021_a_withdrawal_through_a_hold_journals_its_causes_in_observation_order() {
    let (because, saved) = reopened_because(THROUGH);
    assert_eq!(because, ["scope@1", "recheck"]);
    restores(THROUGH, &saved);
}

#[test]
fn d021_a_release_journals_its_deferred_causes_in_observation_order() {
    let (because, saved) = reopened_because(RELEASED);
    assert_eq!(because, ["old_note", "scope@1"]);
    restores(RELEASED, &saved);
}

#[test]
fn d021_the_effects_report_the_causes_in_observation_order() {
    // The effects, the graph's relations and the journal agree.
    let mut session = ReactiveSession::from_source(THROUGH).unwrap();
    for event in ["rule_out", "hold_it", "rescope"] {
        session.dispatch_json(event, "{}").unwrap();
    }
    let outcome = serde_json::to_value(session.dispatch_json("retract", "{}").unwrap()).unwrap();
    let reopens = outcome["effects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|effect| effect["kind"] == "reopen")
        .map(|effect| effect["because"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(reopens, ["scope@1", "recheck"]);
}
