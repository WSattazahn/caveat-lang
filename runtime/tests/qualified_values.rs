use caveat_runtime::reactive::{Provenance, ReactiveSession, ReactiveSnapshot};
use std::collections::BTreeSet;

const GRAPH: &str = r#"
budget 2;
claim safe;
evidence chart from "tidal archive";
evidence gauge from "live instrument";
evidence unseen from "unread instrument";
caveat age consequence high;
caveat drift consequence high;
caveat shadow consequence low;
caveat audit consequence low;
caveat manual consequence low;
chart supports safe;
age qualifies chart;
audit qualifies age;
shadow qualifies safe;
drift qualifies gauge;
"#;

fn session(body: &str) -> ReactiveSession {
    ReactiveSession::from_source(&format!("{GRAPH}\n{body}"))
        .unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).into()).collect()
}

fn trace(actual: &Provenance, evidence: &[&str], caveats: &[&str]) {
    assert_eq!(actual.evidence, names(evidence), "evidence dependencies");
    assert_eq!(actual.caveats, names(caveats), "retained uncertainty");
}

fn chart_trace(snapshot: &ReactiveSnapshot, state: &str) {
    trace(
        &snapshot.qualified_values[state].provenance,
        &["chart"],
        &["age", "audit", "shadow"],
    );
}

#[test]
fn qualified_samples_inherit_transitive_evidence_and_claim_caveats() {
    let game = session("state sample = qualified(8, chart); state plain = 8; event go;");
    let snapshot = game.snapshot();
    assert_eq!(snapshot.values["sample"], 8.0);
    assert_eq!(snapshot.qualified_values["sample"].value, 8.0);
    chart_trace(&snapshot, "sample");
    assert!(snapshot.qualified_values["plain"].provenance.is_empty());
    assert!(snapshot.commitments.is_empty());
    assert_eq!(snapshot.budget.unwrap().spent, 0);
    assert_eq!(
        snapshot
            .symbols
            .iter()
            .find(|symbol| symbol.name == "chart")
            .unwrap()
            .source
            .as_deref(),
        Some("tidal archive")
    );
}

#[test]
fn explicit_caveats_include_their_own_cyclic_qualification_closure() {
    let mut game = session(
        r#"
        gauge opposes safe;
        manual qualifies audit;
        age qualifies manual;
        state sample = qualified(1, gauge, age);
        event decide;
        on decide commit course because enough using sample;
    "#,
    );
    let snapshot = game.dispatch_json("decide", "{}").unwrap();
    trace(
        &snapshot.qualified_values["sample"].provenance,
        &["gauge"],
        &["age", "audit", "manual", "drift", "shadow"],
    );
    trace(
        &snapshot.commitment_bases["course"].provenance,
        &["gauge"],
        &["age", "audit", "manual", "drift", "shadow"],
    );
}

#[test]
fn arithmetic_and_eager_function_arguments_cannot_launder_dependencies() {
    let game = session(
        r#"
        gauge opposes safe;
        fn constant(ignored) = 7;
        fn nested(a, b) = constant(a) + constant(b);
        state a = qualified(4, chart);
        state b = qualified(-4, gauge, manual);
        state cancelled = a - a;
        state zero = a * 0;
        state bounded = clamp(a, 0, 1);
        state ignored = constant(a);
        state both = nested(a, b);
        state selected = min(a, b);
        state relabeled = qualified(a, gauge, manual);
        event go;
    "#,
    );
    let result = game.snapshot();
    for state in ["cancelled", "zero", "bounded", "ignored"] {
        chart_trace(&result, state);
    }
    assert_eq!(result.values["cancelled"], 0.0);
    assert_eq!(result.values["zero"], 0.0);
    assert_eq!(result.values["bounded"], 1.0);
    assert_eq!(result.values["ignored"], 7.0);
    assert_eq!(result.values["both"], 14.0);
    assert_eq!(result.values["selected"], -4.0);
    for state in ["both", "selected", "relabeled"] {
        trace(
            &result.qualified_values[state].provenance,
            &["chart", "gauge"],
            &["age", "audit", "shadow", "drift", "manual"],
        );
    }
}

#[test]
fn accepted_and_rejected_set_guards_record_the_conditional_decision() {
    let mut game = session(
        r#"
        state q = qualified(4, chart);
        state chosen = 0;
        state retained = 3;
        state unobserved_branch = 5;
        state short_circuit = 0;
        state attention_branch = 0;
        event go;
        on go when q > 0 set chosen = 9;
        on go when q < 0 set retained = 11;
        on go when observed(unseen) set unobserved_branch = 12;
        on go when true or qualified(1, unseen) > 0 set short_circuit = 2;
        on go when examined(manual) set attention_branch = 8;
    "#,
    );
    let result = game.dispatch_json("go", "{}").unwrap();
    assert_eq!(result.values["chosen"], 9.0);
    assert_eq!(result.values["retained"], 3.0);
    chart_trace(&result, "chosen");
    chart_trace(&result, "retained");
    assert_eq!(result.values["unobserved_branch"], 5.0);
    assert!(result.qualified_values["unobserved_branch"]
        .provenance
        .is_empty());
    assert_eq!(result.values["short_circuit"], 2.0);
    assert!(result.qualified_values["short_circuit"]
        .provenance
        .is_empty());
    trace(
        &result.qualified_values["attention_branch"].provenance,
        &[],
        &["manual"],
    );
    assert!(!result.relations.iter().any(|edge| edge.from == "unseen"));
}

#[test]
fn bindings_and_cues_keep_primitive_payloads_and_expose_selection_basis() {
    let mut game = session(
        r#"
        state q = qualified(4, chart);
        event show; event idle;
        bind reading.value = q * 2;
        bind reading.visible = q > 0;
        bind verdict.text = "Unconditional default";
        bind verdict.text = "Excluded qualified candidate" when q < 0;
        bind later.text = "Qualified candidate" when q > 0;
        bind later.text = "Final default";
        cue note toast "Proceed with uncertainty." 1;
        on show when q > 0 emit note;
    "#,
    );
    let shown = game.dispatch_json("show", "{}").unwrap();
    let json = serde_json::to_value(&shown).unwrap();
    assert_eq!(json["bindings"]["reading"]["value"], 8.0);
    assert_eq!(json["bindings"]["reading"]["visible"], true);
    assert_eq!(json["bindings"]["verdict"]["text"], "Unconditional default");
    assert_eq!(json["bindings"]["later"]["text"], "Final default");
    for (target, property) in [
        ("reading", "value"),
        ("reading", "visible"),
        ("verdict", "text"),
        ("later", "text"),
    ] {
        trace(
            &shown.binding_qualifications[target][property],
            &["chart"],
            &["age", "audit", "shadow"],
        );
    }
    assert_eq!(shown.cues.len(), 1);
    assert_eq!(shown.cue_qualifications.len(), shown.cues.len());
    trace(
        &shown.cue_qualifications[0],
        &["chart"],
        &["age", "audit", "shadow"],
    );
    let idle = game.dispatch_json("idle", "{}").unwrap();
    assert!(idle.cues.is_empty());
    assert!(idle.cue_qualifications.is_empty());
}

#[test]
fn using_freezes_a_sample_and_automatically_retains_its_entire_basis() {
    let mut game = session(
        r#"
        state estimate = qualified(4, chart);
        event decide; event replace;
        on decide commit course because enough using estimate retaining manual;
        on replace set estimate = 9;
    "#,
    );
    let decided = game.dispatch_json("decide", "{}").unwrap();
    let basis = &decided.commitment_bases["course"];
    assert_eq!(basis.value, Some(4.0));
    trace(
        &basis.provenance,
        &["chart"],
        &["age", "audit", "shadow", "manual"],
    );
    let commitment = decided
        .commitments
        .iter()
        .find(|entry| entry.action == "course")
        .unwrap();
    assert_eq!(
        commitment.retained.iter().cloned().collect::<BTreeSet<_>>(),
        names(&["age", "audit", "shadow", "manual"])
    );
    assert!(decided
        .relations
        .iter()
        .any(|edge| edge.from == "course" && edge.relation == "relies_on" && edge.to == "chart"));
    let replaced = game.dispatch_json("replace", "{}").unwrap();
    assert_eq!(replaced.values["estimate"], 9.0);
    assert!(
        replaced.qualified_values["estimate"].provenance.is_empty(),
        "fresh unconditional data may replace a current state slot"
    );
    assert_eq!(
        replaced.commitment_bases, decided.commitment_bases,
        "replacement cannot rewrite a historical decision"
    );
    assert_eq!(replaced.commitments, decided.commitments);
}

#[test]
fn guarded_commits_and_commitment_predicates_inherit_the_frozen_basis() {
    let mut game = session(
        r#"
        state q = qualified(4, chart);
        state action = 0;
        event decide; event act; event disclose;
        on decide when q > 0 commit course because enough;
        on act when committed(course) set action = 1;
        on disclose reveal gauge opposes safe;
        on disclose reopen course because gauge;
        on disclose when reopened(course) set action = 2;
    "#,
    );
    let decided = game.dispatch_json("decide", "{}").unwrap();
    assert_eq!(decided.commitment_bases["course"].value, None);
    trace(
        &decided.commitment_bases["course"].provenance,
        &["chart"],
        &["age", "audit", "shadow"],
    );
    chart_trace(&game.dispatch_json("act", "{}").unwrap(), "action");
    let reopened = game.dispatch_json("disclose", "{}").unwrap();
    assert_eq!(reopened.values["action"], 2.0);
    assert!(reopened.qualified_values["action"]
        .provenance
        .evidence
        .contains("chart"));
    assert_eq!(reopened.commitment_bases, decided.commitment_bases);
}

#[test]
fn continuing_an_unreopened_commitment_keeps_its_existing_qualified_basis() {
    let mut game = session(
        r#"
        state q = qualified(4, chart);
        state decision = 0;
        event decide; event continue_course;
        on decide commit course because enough using q;
        on continue_course when not reopened(course) set decision = 1;
    "#,
    );
    let committed = game.dispatch_json("decide", "{}").unwrap();
    assert!(!committed.commitments[0].open);
    let continued = game.dispatch_json("continue_course", "{}").unwrap();
    assert_eq!(continued.values["decision"], 1.0);
    chart_trace(&continued, "decision");
    assert_eq!(continued.commitment_bases, committed.commitment_bases);
}

#[test]
fn contradictory_later_samples_preserve_old_values_and_old_decisions() {
    let mut game = session(
        r#"
        state estimate = qualified(4, chart);
        state old_copy = estimate * 2;
        event decide; event revise;
        on decide commit initial_course because enough using estimate;
        on revise reveal gauge opposes safe;
        on revise set estimate = qualified(-3, gauge);
        on revise reopen initial_course because gauge;
        on revise commit revised_course because enough using estimate;
    "#,
    );
    let original = game.dispatch_json("decide", "{}").unwrap();
    let revised = game.dispatch_json("revise", "{}").unwrap();
    assert_eq!(
        revised.qualified_values["old_copy"],
        original.qualified_values["old_copy"]
    );
    assert_eq!(
        revised.commitment_bases["initial_course"],
        original.commitment_bases["initial_course"]
    );
    assert_eq!(revised.commitment_bases["revised_course"].value, Some(-3.0));
    trace(
        &revised.commitment_bases["revised_course"].provenance,
        &["gauge"],
        &["drift", "shadow"],
    );
    assert!(revised
        .relations
        .iter()
        .any(|edge| edge.from == "chart" && edge.relation == "supports" && edge.to == "safe"));
    assert!(revised
        .relations
        .iter()
        .any(|edge| edge.from == "gauge" && edge.relation == "opposes" && edge.to == "safe"));
    assert!(
        revised
            .commitments
            .iter()
            .find(|entry| entry.action == "initial_course")
            .unwrap()
            .open
    );
}

#[test]
fn repeated_samples_share_declared_evidence_identity_but_not_frozen_numbers() {
    let mut game = session(
        r#"
        state estimate = 0;
        event first sample min -10 max 10;
        event second sample min -10 max 10;
        on first set estimate = qualified(sample, chart);
        on first commit earlier because enough using estimate;
        on second set estimate = qualified(sample, chart);
        on second commit later because enough using estimate;
    "#,
    );
    game.dispatch_json("first", r#"{"sample":2}"#).unwrap();
    let result = game.dispatch_json("second", r#"{"sample":-6}"#).unwrap();
    assert_eq!(result.commitment_bases["earlier"].value, Some(2.0));
    assert_eq!(result.commitment_bases["later"].value, Some(-6.0));
    assert_eq!(
        result.commitment_bases["earlier"].provenance, result.commitment_bases["later"].provenance,
        "this version identifies declared evidence sources, not individual observation occurrences"
    );
}

#[test]
fn unseen_or_wrong_kind_evidence_cannot_be_claimed_by_a_qualified_value() {
    for expression in [
        "qualified(1, unseen)",
        "qualified(1, safe)",
        "qualified(1, chart, gauge)",
        "qualified(true, chart)",
    ] {
        assert!(
            ReactiveSession::from_source(&format!("{GRAPH} state q = {expression}; event go;"))
                .is_err(),
            "accepted {expression}"
        );
    }
    // A rule that can never succeed, because nothing ever observes its
    // evidence, is rejected when the program loads.
    let error = ReactiveSession::from_source(&format!(
        "{GRAPH} state q = 0; event attempt; on attempt set q = 3; on attempt set q = qualified(1, unseen);"
    ))
    .unwrap_err();
    assert!(error.contains("nothing observes it"), "{error}");
    // One that fails only until another event reveals it rolls back whole.
    let mut game = session(
        r#"
        state q = 0; event attempt; event notice;
        on notice reveal unseen supports safe;
        on attempt set q = 3;
        on attempt set q = qualified(1, unseen);
    "#,
    );
    let before = game.snapshot();
    assert!(game.dispatch_json("attempt", "{}").is_err());
    assert_eq!(game.snapshot(), before);
    game.dispatch_json("notice", "{}").unwrap();
    assert_eq!(
        game.dispatch_json("attempt", "{}").unwrap().values["q"],
        1.0
    );
}

#[test]
fn late_binding_failure_rolls_back_new_qualifications_graph_basis_and_cues() {
    let mut game = session(
        r#"
        state q = qualified(4, chart);
        state divisor = 1;
        event show; event fail;
        cue note toast "Provisional." 1;
        bind panel.value = q / divisor;
        on show commit initial_course because enough using q;
        on show when q > 0 emit note;
        on fail when q > 0 reveal gauge opposes safe;
        on fail when q > 0 examine drift cost 1;
        on fail when q > 0 reopen initial_course because gauge;
        on fail set q = qualified(-3, gauge);
        on fail commit course because enough using q;
        on fail when q < 0 emit note;
        on fail set divisor = 0;
    "#,
    );
    let before = game.dispatch_json("show", "{}").unwrap();
    assert!(game
        .dispatch_json("fail", "{}")
        .unwrap_err()
        .contains("binding panel.value"));
    assert_eq!(
        game.snapshot(),
        before,
        "no rejected observation, cue, attention spend, trace, or historical basis may escape"
    );
    assert_eq!(before.cues.len(), 1);
    assert_eq!(before.cue_qualifications.len(), 1);
}

#[test]
fn graph_effects_cannot_launder_the_qualified_guard_that_caused_them() {
    let mut game = session(
        r#"
        state q = qualified(4, chart);
        state derived = 0;
        state inspection = 0;
        state reconsidered = 0;
        event decide; event disclose; event repeat;
        on decide commit course because enough;
        on disclose when q > 0 reveal unseen supports safe;
        on disclose set derived = qualified(1, unseen);
        on disclose when q > 0 examine manual cost 1;
        on disclose when examined(manual) set inspection = 1;
        on disclose reveal gauge opposes safe;
        on disclose when q > 0 reopen course because gauge;
        on disclose when reopened(course) set reconsidered = 1;
        on repeat set q = qualified(9, gauge);
        on repeat when q > 0 reveal unseen supports safe;
        on repeat when q > 0 reopen course because gauge;
    "#,
    );
    let before = game.dispatch_json("decide", "{}").unwrap();
    let after = game.dispatch_json("disclose", "{}").unwrap();
    trace(
        &after.qualified_values["derived"].provenance,
        &["chart", "unseen"],
        &["age", "audit", "shadow"],
    );
    trace(
        &after.qualified_values["inspection"].provenance,
        &["chart"],
        &["age", "audit", "shadow", "manual"],
    );
    trace(
        &after.qualified_values["reconsidered"].provenance,
        &["chart", "gauge"],
        &["age", "audit", "shadow", "drift"],
    );
    assert_eq!(
        after.commitment_bases, before.commitment_bases,
        "later reasons for reopening must not rewrite the historical basis"
    );
    assert_eq!(after.budget.as_ref().unwrap().spent, 1);
    let repeated = game.dispatch_json("repeat", "{}").unwrap();
    assert_eq!(
        repeated.observation_qualifications,
        after.observation_qualifications
    );
    assert_eq!(
        repeated.reopening_qualifications,
        after.reopening_qualifications
    );
    assert_eq!(
        repeated.relations, after.relations,
        "an idempotent relation is not a new observation occurrence"
    );
}

#[test]
fn negative_graph_decisions_preserve_the_guard_without_fabricating_an_observation() {
    let mut game = session(
        r#"
        state q = qualified(0, chart);
        state absent_observation = 0;
        state absent_examination = 0;
        state absent_commitment = 0;
        state absent_commitment_reopening = 0;
        state absent_reopening = 0;
        state later_observation = 0;
        event begin; event decline; event independent;
        on begin commit existing because enough;
        on decline when q > 0 reveal unseen supports safe;
        on decline when q > 0 examine manual cost 1;
        on decline when q > 0 commit withheld because enough;
        on decline when q > 0 reopen existing because chart;
        on decline when not observed(unseen) set absent_observation = 1;
        on decline when not examined(manual) set absent_examination = 1;
        on decline when not committed(withheld) set absent_commitment = 1;
        on decline when not reopened(withheld) set absent_commitment_reopening = 1;
        on decline when not reopened(existing) set absent_reopening = 1;
        on decline commit deferred because enough using absent_observation;
        on independent reveal unseen supports safe;
        on independent when observed(unseen) set later_observation = 1;
    "#,
    );
    game.dispatch_json("begin", "{}").unwrap();
    let declined = game.dispatch_json("decline", "{}").unwrap();
    for state in [
        "absent_observation",
        "absent_commitment",
        "absent_commitment_reopening",
        "absent_reopening",
    ] {
        assert_eq!(declined.values[state], 1.0);
        chart_trace(&declined, state);
    }
    trace(
        &declined.qualified_values["absent_examination"].provenance,
        &["chart"],
        &["age", "audit", "shadow", "manual"],
    );
    assert_eq!(declined.budget.as_ref().unwrap().spent, 0);
    assert!(declined
        .commitments
        .iter()
        .all(|entry| entry.action != "withheld"));
    assert!(!declined.relations.iter().any(|edge| edge.from == "unseen"));
    assert!(!declined.qualified_values["absent_observation"]
        .provenance
        .evidence
        .contains("unseen"));
    let independent = game.dispatch_json("independent", "{}").unwrap();
    trace(
        &independent.qualified_values["later_observation"].provenance,
        &["unseen"],
        &["shadow"],
    );
    assert_eq!(
        independent.commitment_bases["deferred"], declined.commitment_bases["deferred"],
        "an independent later observation may replace current absence, never historical reasoning"
    );
}

#[test]
fn examining_a_qualification_does_not_discharge_it_or_veto_deliberate_action() {
    let mut game = session(
        r#"
        state q = qualified(4, chart);
        state steps = 0;
        event inspect; event act;
        on inspect examine age cost 1;
        on inspect commit course because enough using q;
        on act when committed(course) set steps = steps + 1;
    "#,
    );
    let initial = game.snapshot();
    let examined = game.dispatch_json("inspect", "{}").unwrap();
    assert_eq!(examined.budget.as_ref().unwrap().spent, 1);
    assert_eq!(
        examined.qualified_values["q"],
        initial.qualified_values["q"]
    );
    assert!(examined
        .relations
        .iter()
        .any(|edge| edge.from == "age" && edge.relation == "qualifies" && edge.to == "chart"));
    assert!(examined.commitments[0].retained.contains(&"age".into()));
    let acted = game.dispatch_json("act", "{}").unwrap();
    assert_eq!(acted.values["steps"], 1.0);
    chart_trace(&acted, "steps");
}

#[test]
fn repeated_dataflow_deduplicates_provenance_without_changing_numeric_projection() {
    let mut game = session(
        r#"
        state q = qualified(4, chart);
        event update;
        on update set q = q + qualified(0, chart, age);
    "#,
    );
    let initial = game.snapshot();
    for _ in 0..100 {
        let result = game.dispatch_json("update", "{}").unwrap();
        assert_eq!(result.qualified_values, initial.qualified_values);
        assert_eq!(result.values, initial.values);
    }
    let json = serde_json::to_value(game.snapshot()).unwrap();
    assert_eq!(json["values"]["q"], 4.0);
    assert_eq!(
        json["qualified_values"]["q"]["provenance"]["evidence"],
        serde_json::json!(["chart"])
    );
}
