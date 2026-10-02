//! Restore must preserve the same grounds-within-lineage contract as execution.
use caveat_runtime::reactive::{ReactiveSession, ReactiveSnapshot};
use serde_json::{json, Value};

const SOURCE: &str = r#"
claim ready;
evidence chart from "chart";
evidence other from "independent sensor";
evidence neutral from "unassessed observation";
caveat age consequence low;
caveat unrelated consequence material;
age qualifies chart;
state basis = 0;
state narrow = 0;
state neutral_value = 0;
state continued = 0;
event consult;
event decide;
event qualify_later;
event continue_run;
on consult reveal chart supports ready;
on consult reveal other opposes ready;
on consult reveal neutral;
on consult set basis = qualified(1, chart);
on consult set neutral_value = qualified(2, neutral);
on consult set narrow = neutral_value because nothing;
on decide commit act because enough using basis;
on qualify_later qualify chart with unrelated;
on continue_run set continued = basis + narrow + neutral_value;
"#;

fn played() -> ReactiveSession {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    session.dispatch_json("consult", "{}").unwrap();
    session.dispatch_json("decide", "{}").unwrap();
    session
}

fn saved() -> Value {
    serde_json::from_str(&played().save_json().unwrap()).unwrap()
}

fn assert_grounds_within_lineage(snapshot: &ReactiveSnapshot) {
    for (name, grounds) in &snapshot.value_grounds {
        let lineage = &snapshot.qualified_values[name].provenance;
        assert!(grounds.evidence.is_subset(&lineage.evidence), "{name}");
        assert!(grounds.caveats.is_subset(&lineage.caveats), "{name}");
    }
    for (name, grounds) in &snapshot.commitment_grounds {
        let lineage = &snapshot.commitment_bases[name].provenance;
        assert!(grounds.evidence.is_subset(&lineage.evidence), "{name}");
        assert!(grounds.caveats.is_subset(&lineage.caveats), "{name}");
    }
}

fn refused(save: &Value, expected: &str) {
    let error = match ReactiveSession::restore_json(SOURCE, &save.to_string()) {
        Err(error) => error,
        Ok(_) => panic!("restore accepted grounds outside lineage: {expected}"),
    };
    assert!(error.contains(expected), "{error}");
}

#[test]
fn state_grounds_cannot_add_observed_evidence_absent_from_lineage() {
    let mut save = saved();
    save["states"]["basis"]["grounds"] =
        json!({"evidence": ["chart", "other"], "caveats": ["age"]});
    refused(
        &save,
        "state basis grounds include evidence other outside its lineage",
    );
}

#[test]
fn state_grounds_cannot_add_declared_caveats_absent_from_lineage() {
    let mut save = saved();
    save["states"]["basis"]["grounds"] =
        json!({"evidence": ["chart"], "caveats": ["age", "unrelated"]});
    refused(
        &save,
        "state basis grounds include caveat unrelated outside its lineage",
    );
}

#[test]
fn commitment_grounds_cannot_add_evidence_even_when_the_journal_agrees() {
    let mut save = saved();
    save["commitment_grounds"]["act"]["evidence"] = json!(["chart", "other"]);
    // Keep the journal consistent with the edited grounds. The frozen basis
    // and its graph relations still record only the chart.
    save["decision_journal"][0]["because"] = json!(["chart", "other"]);
    refused(
        &save,
        "commitment act grounds include evidence other outside its lineage",
    );
}

#[test]
fn commitment_grounds_cannot_add_caveats_even_when_the_journal_agrees() {
    let mut save = saved();
    save["commitment_grounds"]["act"]["caveats"] = json!(["age", "unrelated"]);
    save["decision_journal"][0]["caveats"] = json!(["age", "unrelated"]);
    refused(
        &save,
        "commitment act grounds include caveat unrelated outside its lineage",
    );
}

#[test]
fn neutral_observations_and_narrowed_grounds_restore_and_continue() {
    let mut original = played();
    let before = original.snapshot();
    assert!(before.value_grounds["neutral_value"]
        .evidence
        .contains("neutral"));
    assert!(before.value_grounds["narrow"].is_empty());
    assert!(!before.qualified_values["narrow"].provenance.is_empty());
    let mut restored =
        ReactiveSession::restore_json(SOURCE, &original.save_json().unwrap()).unwrap();
    assert_eq!(restored.snapshot(), before);
    assert_grounds_within_lineage(&restored.snapshot());
    for event in ["continue_run", "qualify_later", "continue_run"] {
        original.dispatch_json(event, "{}").unwrap();
        restored.dispatch_json(event, "{}").unwrap();
        assert_eq!(restored.snapshot(), original.snapshot(), "after {event}");
        assert_grounds_within_lineage(&restored.snapshot());
    }
}

#[test]
fn omitted_state_grounds_still_default_to_saved_lineage() {
    let original = played();
    let mut save: Value = serde_json::from_str(&original.save_json().unwrap()).unwrap();
    // An absent grounds field has always meant the saved lineage. Exercise
    // both that compact encoding and an explicit equal grounds record.
    save["states"]["basis"]
        .as_object_mut()
        .unwrap()
        .remove("grounds");
    let implicit = ReactiveSession::restore_json(SOURCE, &save.to_string()).unwrap();
    assert_eq!(implicit.snapshot(), original.snapshot());
    save["states"]["basis"]["grounds"] = save["states"]["basis"]["lineage"].clone();
    let explicit = ReactiveSession::restore_json(SOURCE, &save.to_string()).unwrap();
    assert_eq!(explicit.snapshot(), implicit.snapshot());
}

#[test]
fn late_caveats_do_not_rewrite_frozen_commitment_grounds_on_restore() {
    let mut original = played();
    let committed = original.snapshot();
    original.dispatch_json("qualify_later", "{}").unwrap();
    let later = original.snapshot();
    assert!(later.value_grounds["basis"].caveats.contains("unrelated"));
    assert_eq!(later.commitment_grounds, committed.commitment_grounds);
    assert_eq!(later.commitment_bases, committed.commitment_bases);
    let restored = ReactiveSession::restore_json(SOURCE, &original.save_json().unwrap()).unwrap();
    assert_eq!(restored.snapshot(), later);
    assert_grounds_within_lineage(&restored.snapshot());
}
