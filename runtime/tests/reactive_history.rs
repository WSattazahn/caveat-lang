use caveat_runtime::reactive::{Provenance, ReactiveSession};
use std::collections::BTreeSet;

const GRAPH: &str = r#"
budget 3;
claim mild;
claim eastward;
evidence forecast from "published forecast";
evidence sensor from "beam instrument";
evidence warning from "weather report";
caveat calibration consequence high;
caveat review consequence low;
caveat local consequence material;
forecast supports mild;
warning opposes mild;
calibration qualifies sensor;
review qualifies calibration;
local qualifies eastward;
"#;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(&format!("{GRAPH}\n{source}"))
        .unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn evidence(trace: &Provenance, expected: &[&str]) {
    assert_eq!(
        trace.evidence,
        expected.iter().map(|name| (*name).into()).collect()
    );
}

#[test]
fn equal_readings_are_distinct_observations_and_prior_values_keep_their_basis() {
    let mut game = session(
        r#"
        readings flow from sensor limit 4;
        state saved = 0;
        event first value min -8 max 8;
        event next value min -8 max 8;
        on first sample flow = value supports eastward;
        on first set saved = latest(flow) * 2;
        on next sample flow = value opposes eastward;
        bind label.text = if(has_sample(flow), number_text(latest(flow)), "unread");
    "#,
    );
    assert_eq!(
        game.snapshot().bindings["label"]["text"].as_str(),
        Some("unread")
    );
    let first = game.dispatch_json("first", r#"{"value":2}"#).unwrap();
    let saved = first.qualified_values["saved"].clone();
    let record = first.reading_streams["flow"].occurrences[0].clone();
    let second = game.dispatch_json("next", r#"{"value":2}"#).unwrap();
    let stream = &second.reading_streams["flow"];
    assert_eq!(stream.current.as_deref(), Some("flow@2"));
    assert_eq!(stream.occurrences.len(), 2);
    assert_eq!(stream.occurrences[0], record);
    assert_eq!(stream.occurrences[1].value, record.value);
    assert_eq!(stream.occurrences[1].ordinal, 2);
    assert_eq!(stream.occurrences[1].sequence, 2);
    assert_eq!(stream.occurrences[1].event, "next");
    assert_eq!(second.qualified_values["saved"], saved);
    evidence(&saved.provenance, &["flow@1"]);
    evidence(&stream.occurrences[1].provenance, &["flow@2"]);
    assert_eq!(
        record.provenance.caveats,
        BTreeSet::from(["calibration".into(), "review".into(), "local".into(),])
    );
    assert!(second
        .relations
        .iter()
        .any(|edge| edge.from == "flow@1" && edge.to == "eastward" && edge.relation == "supports"));
    assert!(second
        .relations
        .iter()
        .any(|edge| edge.from == "flow@2" && edge.to == "eastward" && edge.relation == "opposes"));
    assert!(!second
        .relations
        .iter()
        .any(|edge| edge.from == "sensor"
            && matches!(edge.relation.as_str(), "supports" | "opposes")));
    assert_eq!(
        second
            .symbols
            .iter()
            .find(|symbol| symbol.name == "flow@2")
            .unwrap()
            .source
            .as_deref(),
        Some("beam instrument")
    );
    assert_eq!(second.budget.unwrap().spent, 0);
}

#[test]
fn repeated_decisions_require_reopening_and_preserve_every_frozen_revision() {
    let mut game = session(
        r#"
        readings flow from sensor limit 4;
        decisions navigation limit 4;
        fn plan(reading) = reading * -0.5;
        event start;
        event inspect value min -8 max 8;
        event bypass;
        event weather;
        on start commit navigation because enough using qualified(0, forecast);
        on inspect sample flow = value opposes mild;
        on inspect reopen navigation because latest(flow);
        on inspect commit navigation because enough using plan(latest(flow));
        on bypass commit navigation because enough using 99;
        on weather reopen navigation because warning;
        bind plan.text = if(committed(navigation), number_text(latest(navigation)), "none");
    "#,
    );
    let initial = game.dispatch_json("start", "{}").unwrap();
    let before = game.snapshot();
    assert!(game
        .dispatch_json("bypass", "{}")
        .unwrap_err()
        .contains("explicitly reopened"));
    assert_eq!(game.snapshot(), before);
    let first = game.dispatch_json("inspect", r#"{"value":4}"#).unwrap();
    let first_basis = first.commitment_bases["navigation@2"].clone();
    let second = game.dispatch_json("inspect", r#"{"value":-6}"#).unwrap();
    assert_eq!(
        second.decision_series["navigation"].current.as_deref(),
        Some("navigation@3")
    );
    assert_eq!(
        second.decision_series["navigation"].revisions[2]
            .previous
            .as_deref(),
        Some("navigation@2")
    );
    assert_eq!(
        second.commitment_bases["navigation@1"],
        initial.commitment_bases["navigation@1"]
    );
    assert_eq!(second.commitment_bases["navigation@2"], first_basis);
    assert_eq!(second.commitment_bases["navigation@3"].value, Some(3.0));
    // The new measurement itself is independent; the revision also depends
    // on the prior decision and its mandatory explicit reopening.
    evidence(
        &second.reading_streams["flow"].occurrences[1].provenance,
        &["flow@2"],
    );
    evidence(
        &second.commitment_bases["navigation@3"].provenance,
        &["forecast", "flow@1", "flow@2"],
    );
    assert_eq!(second.bindings["plan"]["text"].as_str(), Some("3"));
    let reopened = game.dispatch_json("weather", "{}").unwrap();
    assert_eq!(reopened.commitment_bases["navigation@2"], first_basis);
    assert_eq!(
        reopened.commitment_bases["navigation@3"],
        second.commitment_bases["navigation@3"]
    );
    for (action, cause) in [
        ("navigation@1", "flow@1"),
        ("navigation@2", "flow@2"),
        ("navigation@3", "warning"),
    ] {
        let commitment = reopened
            .commitments
            .iter()
            .find(|item| item.action == action)
            .unwrap();
        assert!(commitment.open);
        assert!(commitment.reopened_by.iter().any(|item| item == cause));
    }
    assert!(reopened
        .commitments
        .iter()
        .filter(|item| item.action != "navigation@1")
        .all(|item| item.retained.contains(&"calibration".into())));
}

#[test]
fn skipped_samples_and_revisions_preserve_head_dependencies_without_changing_archives() {
    let mut game = session(
        r#"
        readings flow from sensor limit 4;
        decisions navigation limit 4;
        state saved = 0;
        state absent = 0;
        state q = qualified(0, forecast);
        event skipped;
        event actual;
        event inspect;
        on skipped when q > 0 sample flow = 7 supports mild;
        on skipped when q > 0 commit navigation because enough using 7;
        on skipped when not has_sample(flow) and not reopened(navigation) set absent = 1;
        on actual sample flow = 9 supports mild;
        on actual commit navigation because enough using 9;
        on inspect set saved = latest(flow) + latest(navigation);
    "#,
    );
    let skipped = game.dispatch_json("skipped", "{}").unwrap();
    assert!(skipped.reading_streams["flow"].occurrences.is_empty());
    assert!(skipped.decision_series["navigation"].revisions.is_empty());
    evidence(
        &skipped.qualified_values["absent"].provenance,
        &["forecast"],
    );
    let actual = game.dispatch_json("actual", "{}").unwrap();
    evidence(
        &actual.reading_streams["flow"].occurrences[0].provenance,
        &["flow@1"],
    );
    assert!(actual.decision_series["navigation"]
        .selection_qualifications
        .is_empty());
    let archive = actual.reading_streams["flow"].occurrences[0].clone();
    let basis = actual.commitment_bases["navigation@1"].clone();
    game.dispatch_json("skipped", "{}").unwrap();
    let inspected = game.dispatch_json("inspect", "{}").unwrap();
    evidence(
        &inspected.qualified_values["saved"].provenance,
        &["flow@1", "forecast"],
    );
    assert_eq!(inspected.reading_streams["flow"].occurrences[0], archive);
    assert_eq!(inspected.commitment_bases["navigation@1"], basis);
}

#[test]
fn reopening_a_selected_reading_carries_its_head_selection_dependency() {
    let mut game = session(
        r#"
        readings flow from sensor limit 3;
        decisions navigation limit 3;
        state q = qualified(0, forecast);
        event start;
        event revise;
        on start sample flow = 4 supports mild;
        on start commit navigation because enough using 0;
        on revise when q > 0 sample flow = 99 opposes mild;
        on revise reopen navigation because latest ( flow );
        on revise commit navigation because enough using latest(flow);
    "#,
    );
    let first = game.dispatch_json("start", "{}").unwrap();
    let revised = game.dispatch_json("revise", "{}").unwrap();
    assert_eq!(
        revised.reading_streams["flow"].occurrences,
        first.reading_streams["flow"].occurrences
    );
    evidence(
        &revised.commitment_bases["navigation@2"].provenance,
        &["flow@1", "forecast"],
    );
}

#[test]
fn failed_events_roll_back_occurrences_ids_attention_and_cues() {
    let mut game = session(
        r#"
        readings flow from sensor limit 3;
        decisions navigation limit 3;
        state denominator = 1;
        cue bell toast "read" 1;
        event fail;
        event succeed;
        on fail sample flow = 4 opposes mild;
        on fail examine calibration cost 1;
        on fail commit navigation because enough using latest(flow);
        on fail emit bell;
        on fail set denominator = 0;
        on succeed sample flow = 4 opposes mild;
        on succeed commit navigation because enough using latest(flow);
        bind result.text = number_text(1 / denominator);
    "#,
    );
    let before = game.snapshot();
    assert!(game.dispatch_json("fail", "{}").is_err());
    assert_eq!(game.snapshot(), before);
    let next = game.dispatch_json("succeed", "{}").unwrap();
    assert_eq!(
        next.reading_streams["flow"].current.as_deref(),
        Some("flow@1")
    );
    assert_eq!(
        next.decision_series["navigation"].current.as_deref(),
        Some("navigation@1")
    );
    assert_eq!(next.sequence, 1);
    assert_eq!(next.budget.unwrap().spent, 0);
}

#[test]
fn history_capacity_never_evicts_old_values_or_partially_publishes() {
    let mut game = session(
        r#"
        readings flow from sensor limit 2;
        decisions navigation limit 2;
        event inspect value min -8 max 8;
        on inspect sample flow = value supports mild;
        on inspect when committed(navigation) reopen navigation because latest(flow);
        on inspect commit navigation because enough using latest(flow);
    "#,
    );
    game.dispatch_json("inspect", r#"{"value":1}"#).unwrap();
    game.dispatch_json("inspect", r#"{"value":2}"#).unwrap();
    let full = game.snapshot();
    assert!(game
        .dispatch_json("inspect", r#"{"value":3}"#)
        .unwrap_err()
        .contains("history limit"));
    assert_eq!(game.snapshot(), full);

    let mut revisions = session(
        r#"
        readings flow from sensor limit 2;
        decisions navigation limit 1;
        event inspect;
        on inspect sample flow = 1 supports mild;
        on inspect when committed(navigation) reopen navigation because latest(flow);
        on inspect commit navigation because enough using latest(flow);
    "#,
    );
    revisions.dispatch_json("inspect", "{}").unwrap();
    let before = revisions.snapshot();
    assert!(revisions.dispatch_json("inspect", "{}").is_err());
    assert_eq!(revisions.snapshot(), before);
}

#[test]
fn latest_requires_a_reached_numeric_occurrence_and_types_are_validated() {
    for body in [
        "readings flow from sensor limit 2; state x = latest(flow); event go;",
        "decisions nav limit 2; state x = latest(nav); event go;",
        "readings flow from mild limit 2; event go;",
        "decisions nav limit 2; event go; bind x.visible = has_sample(nav);",
        "readings flow from sensor limit 2; event go; bind x.visible = observed(flow);",
        "readings flow from sensor limit 2; event go; on go commit flow because enough;",
        "decisions nav limit 2; event go; on go sample nav = 1 supports mild;",
        "decisions nav limit 2; event go; on go reopen nav because latest(nav);",
        "readings flow from sensor limit 2; state flow = 0; event go;",
        "claim flow@1; readings flow from sensor limit 2; event go;",
        "readings flow from sensor limit 0; event go;",
        "decisions nav limit 257; event go;",
    ] {
        assert!(
            ReactiveSession::from_source(&format!("{GRAPH}\n{body}")).is_err(),
            "{body}"
        );
    }
    let mut game = session("decisions nav limit 2; event begin; on begin commit nav because enough; bind x.text = if(committed(nav), number_text(latest(nav)), \"none\");");
    let before = game.snapshot();
    assert!(game
        .dispatch_json("begin", "{}")
        .unwrap_err()
        .contains("no numeric using value"));
    assert_eq!(game.snapshot(), before);
}

#[test]
fn total_capacity_is_bounded_and_replaying_inputs_reproduces_occurrence_identity() {
    let legal = (0..4)
        .map(|index| format!("decisions decisions_{index} limit 256;\n"))
        .collect::<String>();
    assert!(ReactiveSession::from_source(&format!("{legal}event go;")).is_ok());
    assert!(
        ReactiveSession::from_source(&format!("{legal}decisions excess limit 1; event go;"))
            .is_err()
    );
    let source = r#"
        readings flow from sensor limit 3;
        decisions nav limit 3;
        event inspect x min -2 max 2;
        on inspect sample flow = x supports mild;
        on inspect when committed(nav) reopen nav because latest(flow);
        on inspect commit nav because enough using latest(flow);
    "#;
    let mut one = session(source);
    let mut two = session(source);
    for payload in [r#"{"x":1}"#, r#"{"x":-1}"#, r#"{"x":1}"#] {
        assert_eq!(
            one.dispatch_json("inspect", payload).unwrap(),
            two.dispatch_json("inspect", payload).unwrap()
        );
    }
    let before = one.snapshot();
    assert_eq!(one.snapshot(), before);
    assert_eq!(one.snapshot(), before);
}

#[test]
fn late_provenance_overflow_rolls_back_the_new_reading_and_reopening() {
    let qualifications = (0..1023)
        .map(|index| format!("caveat c{index} consequence low; c{index} qualifies sensor;\n"))
        .collect::<String>();
    let source = format!(
        r#"
        claim measured;
        evidence sensor from "instrument";
        {qualifications}
        readings flow from sensor limit 2;
        decisions nav limit 2;
        event inspect;
        on inspect sample flow = 1 supports measured;
        on inspect when committed(nav) reopen nav because latest(flow);
        on inspect commit nav because enough using latest(flow);
    "#
    );
    let mut game = ReactiveSession::from_source(&source).unwrap();
    game.dispatch_json("inspect", "{}").unwrap();
    let before = game.snapshot();
    assert_eq!(
        before.reading_streams["flow"].occurrences[0]
            .provenance
            .caveats
            .len(),
        1023
    );
    let error = game.dispatch_json("inspect", "{}").unwrap_err();
    assert!(error.contains("provenance exceeds"), "{error}");
    assert_eq!(game.snapshot(), before);
}
