//! Grounded explanations: `bind ... because CITATIONS`. See
//! spec/caveat-explanations-0.1.md.
use caveat_runtime::reactive::{BindingValue, Provenance, ReactiveSession};
use std::collections::BTreeSet;

const GRAPH: &str = r#"
claim safe;
evidence chart from "tidal archive";
evidence gauge from "live instrument";
caveat age consequence high;
caveat manual consequence low;
age qualifies chart;
state from_chart = 0;
state from_gauge = 0;
event read;
on read reveal chart supports safe;
on read reveal gauge opposes safe;
on read set from_chart = qualified(1, chart);
on read set from_gauge = qualified(1, gauge);
"#;

fn session(body: &str) -> ReactiveSession {
    ReactiveSession::from_source(&format!("{GRAPH}\n{body}"))
        .unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).into()).collect()
}

fn trace(actual: &Provenance, evidence: &[&str], caveats: &[&str]) {
    assert_eq!(actual.evidence, names(evidence), "cited evidence");
    assert_eq!(actual.caveats, names(caveats), "cited caveats");
}

#[test]
fn without_because_a_binding_cites_its_whole_lineage() {
    let mut game = session(r#"bind hud.text = "both" when from_chart > 0 and from_gauge > 0;"#);
    let shown = game.dispatch_json("read", "{}").unwrap();
    trace(
        &shown.binding_qualifications["hud"]["text"],
        &["chart", "gauge"],
        &["age"],
    );
    assert_eq!(
        shown.binding_explanations["hud"]["text"],
        shown.binding_qualifications["hud"]["text"]
    );
}

#[test]
fn because_cites_a_chosen_part_of_the_lineage() {
    let mut game = session(
        r#"bind hud.text = "contradicted" when from_chart > 0 and from_gauge > 0 because from_gauge;"#,
    );
    let shown = game.dispatch_json("read", "{}").unwrap();
    // The full lineage is kept for audit; only the explanation is selective.
    trace(
        &shown.binding_qualifications["hud"]["text"],
        &["chart", "gauge"],
        &["age"],
    );
    trace(&shown.binding_explanations["hud"]["text"], &["gauge"], &[]);
}

#[test]
fn a_cited_value_brings_its_caveats() {
    let mut game = session(
        r#"bind hud.text = "charted" when from_chart > 0 and from_gauge > 0 because from_chart;"#,
    );
    let shown = game.dispatch_json("read", "{}").unwrap();
    trace(
        &shown.binding_explanations["hud"]["text"],
        &["chart"],
        &["age"],
    );
}

#[test]
fn several_citations_are_united() {
    let mut game = session(
        r#"bind hud.value = from_chart + from_gauge because from_chart, if(from_gauge > 0, from_gauge, 0);"#,
    );
    let shown = game.dispatch_json("read", "{}").unwrap();
    trace(
        &shown.binding_explanations["hud"]["value"],
        &["chart", "gauge"],
        &["age"],
    );
}

#[test]
fn because_nothing_cites_nothing() {
    let mut game = session(r#"bind hud.text = "" when from_chart > 0 because nothing;"#);
    let shown = game.dispatch_json("read", "{}").unwrap();
    trace(
        &shown.binding_qualifications["hud"]["text"],
        &["chart"],
        &["age"],
    );
    trace(&shown.binding_explanations["hud"]["text"], &[], &[]);
}

#[test]
fn citing_evidence_the_binding_never_read_rejects_the_event() {
    let mut game = session(r#"bind hud.text = "charted" when from_chart > 0 because from_gauge;"#);
    let before = game.snapshot();
    let error = game.dispatch_json("read", "{}").unwrap_err();
    assert!(error.contains("hud.text cites evidence gauge"), "{error}");
    assert!(error.contains("never read"), "{error}");
    let after = game.snapshot();
    assert_eq!(after.sequence, before.sequence);
    assert_eq!(after.values, before.values);
    assert_eq!(after.relations, before.relations);
    assert_eq!(after.binding_explanations, before.binding_explanations);
}

#[test]
fn a_citation_cannot_add_a_caveat_the_value_never_carried() {
    let mut game = session(
        r#"bind hud.text = "charted" when from_chart > 0 because qualified(1, chart, manual);"#,
    );
    let error = game.dispatch_json("read", "{}").unwrap_err();
    assert!(error.contains("cites caveat manual"), "{error}");
}

#[test]
fn only_the_winning_declaration_cites() {
    let mut game = session(
        r#"
        bind hud.text = "chart" when from_chart > 0 because from_chart;
        bind hud.text = "gauge" when from_gauge > 0;
        bind hud.text = "never" when false because 1 / 0;
        "#,
    );
    let shown = game.dispatch_json("read", "{}").unwrap();
    assert_eq!(
        shown.bindings["hud"]["text"],
        BindingValue::Text("gauge".into())
    );
    // The winner has no `because`, so it cites its whole lineage, including
    // the guard of the losing candidate before it.
    assert_eq!(
        shown.binding_explanations["hud"]["text"],
        shown.binding_qualifications["hud"]["text"]
    );
}

#[test]
fn an_omitted_binding_has_no_explanation() {
    let mut game = session(r#"bind hud.text = "charted" when from_chart > 1 because from_chart;"#);
    let shown = game.dispatch_json("read", "{}").unwrap();
    assert!(!shown
        .bindings
        .get("hud")
        .is_some_and(|hud| hud.contains_key("text")));
    assert!(!shown
        .binding_explanations
        .get("hud")
        .is_some_and(|hud| hud.contains_key("text")));
}

#[test]
fn because_is_found_outside_text_and_parentheses() {
    let mut game = session(
        r#"bind hud.text = "because of the gauge" when if(from_gauge > 0, true, false) because from_gauge;"#,
    );
    let shown = game.dispatch_json("read", "{}").unwrap();
    assert_eq!(
        shown.bindings["hud"]["text"],
        BindingValue::Text("because of the gauge".into())
    );
    trace(&shown.binding_explanations["hud"]["text"], &["gauge"], &[]);
}

#[test]
fn malformed_citations_are_rejected_before_execution() {
    for (body, expected) in [
        ("bind hud.value = 1 because;", "because"),
        (
            "bind hud.value = 1 because from_chart,, from_gauge;",
            "separated by commas",
        ),
        ("bind hud.value = 1 because nowhere;", "nowhere"),
        ("bind hud.value = 1 because from_chart when true;", ""),
    ] {
        let error = ReactiveSession::from_source(&format!("{GRAPH}\n{body}"))
            .err()
            .unwrap_or_else(|| panic!("{body} should be rejected"));
        assert!(error.contains(expected), "{body}: {error}");
    }
}

#[test]
fn explanations_are_published_to_the_browser_snapshot() {
    let mut game = session(
        r#"bind hud.text = "x" when from_chart > 0 and from_gauge > 0 because from_gauge;"#,
    );
    game.dispatch_json("read", "{}").unwrap();
    let json = serde_json::to_value(game.snapshot()).unwrap();
    assert_eq!(
        json["binding_explanations"]["hud"]["text"]["evidence"],
        serde_json::json!(["gauge"])
    );
}
