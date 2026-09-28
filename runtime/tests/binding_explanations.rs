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

/// Evidence whose meaning changes during play: a caveat learned late, a
/// renewal, a withdrawal, readings and a decision made on them.
const PLAY: &str = r#"
claim safe;
evidence bite from "a bite";
evidence chart from "tidal archive";
evidence gauge from "live instrument";
evidence recheck from "a second look";
caveat faded consequence low;
renewable bite limit 4;
readings flow from gauge limit 8;
decisions route limit 4;
state from_bite = 0;
state from_chart = 0;
state derived = 0;
state marks = 0;
event tick dt min 0 max 0.1;
event eat;
event regrow;
event chart;
event note;
event read x min 0 max 9;
event decide;
event other;
on eat reveal bite supports safe;
on eat set from_bite = from_bite + qualified(1, bite);
on chart reveal chart supports safe;
on chart set from_chart = qualified(1, chart);
on other set marks = marks + 1;
"#;

fn play(body: &str) -> ReactiveSession {
    ReactiveSession::from_source(&format!("{PLAY}\n{body}"))
        .unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

/// Dispatch through the outcome API: "accepted", or the refusal's
/// ORIGIN/CODE. A fatal outcome fails the test.
fn outcome(game: &mut ReactiveSession, event: &str, payload: &str) -> String {
    let outcome = game
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} {payload} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    match outcome["outcome"].as_str() {
        Some("accepted") => "accepted".into(),
        _ => format!(
            "{}/{}",
            outcome["origin"].as_str().unwrap(),
            outcome["code"].as_str().unwrap()
        ),
    }
}

/// Every shown explanation within its binding's lineage.
fn assert_explanations_within_lineage(game: &ReactiveSession) {
    let snapshot = game.snapshot();
    for (target, properties) in &snapshot.binding_explanations {
        for (property, explanation) in properties {
            let lineage = &snapshot.binding_qualifications[target][property];
            assert!(
                explanation.evidence.is_subset(&lineage.evidence)
                    && explanation.caveats.is_subset(&lineage.caveats),
                "{target}.{property}: {explanation:?} ⊄ {lineage:?}"
            );
        }
    }
}

// A citation of what the value and its conditions read stays grounded through
// everything play does to that: a caveat learned late, on the clock and on an
// event, one learned for an occurrence a renewal has since replaced, a
// renewal, a withdrawal, a reading that reopens a decision and its revision,
// and a state cited beside the one it was derived into. The cited grounds
// change with the lineage, so the check never fails for these citations.
#[test]
fn a_citation_of_what_was_read_stays_grounded_through_play() {
    let mut game = play(
        r#"
        on eat qualify bite with faded after 0.15;
        on regrow renew bite;
        on note qualify chart with faded;
        on note reveal recheck supports safe;
        on note withdraw bite because recheck;
        on chart set derived = from_chart * 2;
        on read sample flow = x supports safe;
        on read when committed(route) and not reopened(route) reopen route because latest(flow);
        on decide when not committed(route) commit route because enough using latest(flow);
        on decide when reopened(route) commit route because enough using latest(flow);
        bind hud.bite = from_bite when from_bite > 0 because from_bite;
        bind hud.text = "fresh" because nothing;
        bind hud.text = "faded" when carries(bite, faded) because carries(bite, faded);
        bind hud.value = derived when derived > 0 because from_chart;
        bind hud.route = "decided" when committed(route) because committed(route);
        bind hud.open = "open" when reopened(route) because reopened(route), latest(flow);
        bind hud.reading = latest(flow) when has_sample(flow) because latest(flow);
        bind hud.withdrawn = "withdrawn" when withdrawn(bite) because withdrawn(bite);
        "#,
    );
    for (event, payload) in [
        ("eat", "{}"),
        ("chart", "{}"),
        ("read", r#"{"x": 1}"#),
        ("decide", "{}"),
        ("tick", r#"{"dt": 0.1}"#),
        ("regrow", "{}"),
        ("eat", "{}"),
        // bite's caveat arrives after bite@2 has replaced it,
        ("tick", r#"{"dt": 0.1}"#),
        // then bite@2's, so "faded" is shown.
        ("tick", r#"{"dt": 0.1}"#),
        ("read", r#"{"x": 2}"#),
        ("decide", "{}"),
        ("read", r#"{"x": 3}"#),
        ("note", "{}"),
        ("chart", "{}"),
        ("other", "{}"),
    ] {
        assert_eq!(outcome(&mut game, event, payload), "accepted", "{event}");
        assert_explanations_within_lineage(&game);
    }
    let shown = game.snapshot();
    assert_eq!(
        shown.bindings["hud"]["text"],
        BindingValue::Text("faded".into())
    );
    assert_eq!(
        shown.bindings["hud"]["withdrawn"],
        BindingValue::Text("withdrawn".into())
    );
    assert_eq!(
        shown.decision_series["route"].current.as_deref(),
        Some("route@2")
    );
    trace(
        &shown.binding_explanations["hud"]["bite"],
        &["bite", "bite@2"],
        &["faded", "withdrawn"],
    );
    trace(
        &shown.binding_explanations["hud"]["value"],
        &["chart"],
        &["faded"],
    );
}

/// A citation its value and conditions never read, reached during play: the
/// event is refused as evaluation/ungrounded_citation with the diagnostic the
/// legacy API returns, nothing it did is kept, and the session goes on, also
/// after a save and restore.
fn refuses_ungrounded(
    body: &str,
    setup: &[(&str, &str)],
    (event, payload): (&str, &str),
    diagnostic: &str,
) {
    let source = format!("{PLAY}\n{body}");
    let mut game = play(body);
    for (event, payload) in setup {
        assert_eq!(outcome(&mut game, event, payload), "accepted", "{event}");
    }
    let resumed = ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    for mut game in [game, resumed] {
        let before = (game.snapshot(), game.save_json().unwrap());
        for _ in 0..2 {
            let result = game
                .dispatch_outcome_json(event, payload)
                .unwrap_or_else(|fatal| panic!("{event} {payload} was fatal: {}", fatal.message));
            let result = serde_json::to_value(result).unwrap();
            assert_eq!(result["outcome"], "rejected", "{result}");
            assert_eq!(result["origin"], "evaluation");
            assert_eq!(result["code"], "ungrounded_citation");
            assert!(
                result["message"].as_str().unwrap().ends_with(diagnostic),
                "{result}"
            );
            let legacy = game.clone().dispatch_json(event, payload).unwrap_err();
            assert_eq!(result["message"], legacy);
            assert_eq!((game.snapshot(), game.save_json().unwrap()), before);
        }
        assert_eq!(outcome(&mut game, "other", "{}"), "accepted");
    }
}

// The save fuzz's case: a caveat scheduled on eating fires on a tick, the
// declaration that shows it wins for the first time, and its citation was
// never read. Nothing about the citation changed; the clock made it count.
#[test]
fn a_late_caveat_on_the_clock_refuses_the_tick() {
    refuses_ungrounded(
        r#"
        on eat qualify bite with faded after 0.15;
        bind hud.text = "fresh" because nothing;
        bind hud.text = "faded" when carries(bite, faded) because from_chart;
        "#,
        &[("chart", "{}"), ("eat", "{}"), ("tick", r#"{"dt": 0.1}"#)],
        ("tick", r#"{"dt": 0.1}"#),
        "binding hud.text cites evidence chart that its value and conditions never read",
    );
}

#[test]
fn a_late_caveat_on_an_event_refuses_the_event() {
    refuses_ungrounded(
        r#"
        on note qualify bite with faded;
        bind hud.text = "faded" when carries(bite, faded) because from_chart;
        "#,
        &[("chart", "{}"), ("eat", "{}")],
        ("note", "{}"),
        "binding hud.text cites evidence chart that its value and conditions never read",
    );
}

// As in the save fuzz's program: the caveat for the first bite arrives when a
// renewal has replaced it and changes nothing shown; the one for the current
// occurrence makes the declaration win.
#[test]
fn a_stale_caveat_is_accepted_and_the_current_one_refused() {
    refuses_ungrounded(
        r#"
        on eat qualify bite with faded after 0.15;
        on regrow renew bite;
        bind hud.text = "faded" when carries(bite, faded) because from_chart;
        "#,
        &[
            ("chart", "{}"),
            ("eat", "{}"),
            ("tick", r#"{"dt": 0.1}"#),
            ("regrow", "{}"),
            ("eat", "{}"),
            ("tick", r#"{"dt": 0.1}"#),
        ],
        ("tick", r#"{"dt": 0.1}"#),
        "binding hud.text cites evidence chart that its value and conditions never read",
    );
}

#[test]
fn a_withdrawal_that_makes_a_declaration_win_is_refused() {
    refuses_ungrounded(
        r#"
        on note reveal recheck supports safe;
        on note withdraw bite because recheck;
        bind hud.text = "withdrawn" when withdrawn(bite) because from_chart;
        "#,
        &[("chart", "{}"), ("eat", "{}")],
        ("note", "{}"),
        "binding hud.text cites evidence chart that its value and conditions never read",
    );
}

// `qualified(1, bite)` names the current occurrence. Once a renewal replaces
// it, the citation names bite@2, which the value was never computed from.
#[test]
fn a_renewal_moves_what_a_citation_names() {
    refuses_ungrounded(
        r#"
        on regrow renew bite;
        on regrow reveal bite supports safe;
        bind hud.bite = from_bite when from_bite > 0 because qualified(1, bite);
        "#,
        &[("eat", "{}")],
        ("regrow", "{}"),
        "binding hud.bite cites evidence bite@2 that its value and conditions never read",
    );
}

#[test]
fn a_reading_the_decision_was_not_made_on_is_refused() {
    refuses_ungrounded(
        r#"
        on read sample flow = x supports safe;
        on decide when not committed(route) commit route because enough using latest(flow);
        bind hud.route = "decided" when committed(route) because latest(flow);
        "#,
        &[("read", r#"{"x": 1}"#), ("decide", "{}")],
        ("read", r#"{"x": 2}"#),
        "binding hud.route cites evidence flow@2 that its value and conditions never read",
    );
}

// The reading that reopens the decision is read by `reopened(route)`; the
// next one, which does not reopen it again, is not.
#[test]
fn a_reading_after_the_reopening_is_refused() {
    refuses_ungrounded(
        r#"
        on read sample flow = x supports safe;
        on read when committed(route) and not reopened(route) reopen route because latest(flow);
        on decide when not committed(route) commit route because enough using latest(flow);
        bind hud.route = "open" when reopened(route) because latest(flow);
        "#,
        &[
            ("read", r#"{"x": 1}"#),
            ("decide", "{}"),
            ("read", r#"{"x": 2}"#),
        ],
        ("read", r#"{"x": 3}"#),
        "binding hud.route cites evidence flow@3 that its value and conditions never read",
    );
}

// A value derived from a state may cite that state while it is still what the
// value was derived from, but not once it changes on its own.
#[test]
fn a_cited_state_that_changes_apart_from_the_value_is_refused() {
    refuses_ungrounded(
        r#"
        on chart set derived = from_chart * 2;
        on note set from_chart = from_bite;
        bind hud.value = derived when derived > 0 because from_chart;
        "#,
        &[("eat", "{}"), ("chart", "{}")],
        ("note", "{}"),
        "binding hud.value cites evidence bite that its value and conditions never read",
    );
}

// `or` reads its second operand only when the first is false.
#[test]
fn an_operand_left_unread_is_refused() {
    refuses_ungrounded(
        r#"
        bind hud.text = "either" when from_chart > 0 or from_bite > 0 because from_bite;
        "#,
        &[("eat", "{}")],
        ("chart", "{}"),
        "binding hud.text cites evidence bite that its value and conditions never read",
    );
}

// The same check on `set ... because`, in a rule and in a procedure.
#[test]
fn a_state_citation_is_refused_the_same_way() {
    refuses_ungrounded(
        r#"
        on eat qualify bite with faded after 0.15;
        on tick when carries(bite, faded) set derived = 1 because from_chart;
        "#,
        &[("chart", "{}"), ("eat", "{}"), ("tick", r#"{"dt": 0.1}"#)],
        ("tick", r#"{"dt": 0.1}"#),
        "state derived cites evidence chart that its value and conditions never read",
    );
    // Before the chart is read, `from_chart` is grounded on nothing, so
    // citing it cites nothing.
    refuses_ungrounded(
        r#"
        proc mark() { set derived = 1 because from_chart; };
        on note call mark();
        "#,
        &[("note", "{}"), ("chart", "{}")],
        ("note", "{}"),
        "procedure mark, step 1: state derived cites evidence chart that its value and conditions never read",
    );
}

// A declaration that already wins when the program loads is checked then,
// and the program does not load.
#[test]
fn a_citation_ungrounded_at_load_refuses_the_program() {
    let error = ReactiveSession::from_source(&format!(
        "{PLAY}
        evidence forecast from \"forecast\";
        forecast supports safe;
        state support = qualified(1, forecast);
        bind hud.level = marks because support;"
    ))
    .err()
    .unwrap();
    assert_eq!(
        error,
        "binding hud.level cites evidence forecast that its value and conditions never read"
    );
}
