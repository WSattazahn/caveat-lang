//! Grounds: what a state or commitment is based on, as distinct from its
//! lineage. See spec/caveat-explanations-0.2.md.
use caveat_runtime::reactive::{Provenance, ReactiveSession, ReactiveSnapshot};
use std::collections::BTreeSet;

const GRAPH: &str = r#"
claim safe;
evidence chart from "tidal archive";
evidence gauge from "live instrument";
evidence buoy from "harbour buoy";
caveat age consequence high;
caveat manual consequence low;
age qualifies chart;
state from_chart = 0;
state from_gauge = 0;
state x = 0;
event read;
event other;
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
    assert_eq!(actual.evidence, names(evidence), "evidence");
    assert_eq!(actual.caveats, names(caveats), "caveats");
}

fn lineage<'a>(snapshot: &'a ReactiveSnapshot, state: &str) -> &'a Provenance {
    &snapshot.qualified_values[state].provenance
}

/// The contract: grounds may leave dependencies out, never add one.
fn assert_grounds_within_lineage(snapshot: &ReactiveSnapshot) {
    for (state, grounds) in &snapshot.value_grounds {
        let lineage = lineage(snapshot, state);
        assert!(
            grounds.evidence.is_subset(&lineage.evidence),
            "{state}: {grounds:?} ⊄ {lineage:?}"
        );
        assert!(
            grounds.caveats.is_subset(&lineage.caveats),
            "{state}: {grounds:?} ⊄ {lineage:?}"
        );
    }
    for (commitment, grounds) in &snapshot.commitment_grounds {
        let basis = &snapshot.commitment_bases[commitment].provenance;
        assert!(grounds.evidence.is_subset(&basis.evidence), "{commitment}");
        assert!(grounds.caveats.is_subset(&basis.caveats), "{commitment}");
    }
}

#[test]
fn a_guard_enters_lineage_but_not_grounds() {
    let mut game = session("on read when from_gauge > 0 set x = from_chart;");
    let shown = game.dispatch_json("read", "{}").unwrap();
    trace(lineage(&shown, "x"), &["chart", "gauge"], &["age"]);
    trace(&shown.value_grounds["x"], &["chart"], &["age"]);
    assert_grounds_within_lineage(&shown);
}

#[test]
fn a_skipped_rule_changes_lineage_but_not_grounds() {
    let mut game = session(
        "on read set x = from_chart;\n\
         on other when from_gauge > 5 set x = 0;",
    );
    game.dispatch_json("read", "{}").unwrap();
    let shown = game.dispatch_json("other", "{}").unwrap();
    // Leaving x alone depended on the gauge; x is still about the chart.
    trace(lineage(&shown, "x"), &["chart", "gauge"], &["age"]);
    trace(&shown.value_grounds["x"], &["chart"], &["age"]);
    assert_grounds_within_lineage(&shown);
}

#[test]
fn a_read_carries_grounds_not_the_guards_behind_them() {
    let mut game = session(
        "state y = 0;\n\
         on read when from_gauge > 0 set x = from_chart;\n\
         on read set y = x + 1;",
    );
    let shown = game.dispatch_json("read", "{}").unwrap();
    trace(lineage(&shown, "y"), &["chart", "gauge"], &["age"]);
    trace(&shown.value_grounds["y"], &["chart"], &["age"]);
}

#[test]
fn qualified_grounds_leave_out_the_guard_that_revealed_the_evidence() {
    let mut game = session(
        "state z = 0;\n\
         on other when from_gauge > 0 reveal buoy supports safe;\n\
         on other set z = qualified(1, buoy);",
    );
    game.dispatch_json("read", "{}").unwrap();
    let shown = game.dispatch_json("other", "{}").unwrap();
    trace(lineage(&shown, "z"), &["buoy", "gauge"], &[]);
    trace(&shown.value_grounds["z"], &["buoy"], &[]);
}

#[test]
fn because_on_set_narrows_grounds_and_is_checked() {
    let mut game = session("on read set x = from_chart + from_gauge because from_gauge;");
    let shown = game.dispatch_json("read", "{}").unwrap();
    trace(lineage(&shown, "x"), &["chart", "gauge"], &["age"]);
    trace(&shown.value_grounds["x"], &["gauge"], &[]);

    let mut game = session("on read set x = from_chart because nothing;");
    let shown = game.dispatch_json("read", "{}").unwrap();
    trace(&shown.value_grounds["x"], &[], &[]);

    let mut game = session("on read set x = from_chart because from_gauge;");
    let before = game.snapshot();
    let error = game.dispatch_json("read", "{}").unwrap_err();
    assert!(error.contains("state x cites evidence gauge"), "{error}");
    assert_eq!(game.snapshot().values, before.values);
    assert_eq!(game.snapshot().value_grounds, before.value_grounds);
}

#[test]
fn a_binding_citing_a_state_cites_its_grounds() {
    let mut game = session(
        "on read when from_gauge > 0 set x = from_chart;\n\
         bind hud.text = \"charted\" when x > 0 because x;",
    );
    let shown = game.dispatch_json("read", "{}").unwrap();
    trace(
        &shown.binding_qualifications["hud"]["text"],
        &["chart", "gauge"],
        &["age"],
    );
    trace(
        &shown.binding_explanations["hud"]["text"],
        &["chart"],
        &["age"],
    );
}

#[test]
fn commitment_grounds_leave_out_the_guard_and_the_predecessor() {
    let mut game = session(
        "decisions trust limit 4;\n\
         evidence second from \"second sighting\";\n\
         event recover;\n\
         on read when from_gauge > 0 commit trust because enough using from_chart;\n\
         on other reveal buoy opposes safe;\n\
         on other reopen trust because buoy;\n\
         on recover reveal second supports safe;\n\
         on recover when reopened(trust) commit trust because enough using qualified(1, second);",
    );
    game.dispatch_json("read", "{}").unwrap();
    game.dispatch_json("other", "{}").unwrap();
    let shown = game.dispatch_json("recover", "{}").unwrap();
    // Lineage: the guard's gauge, the predecessor's chart and its reopening buoy.
    trace(
        &shown.commitment_bases["trust@1"].provenance,
        &["chart", "gauge"],
        &["age"],
    );
    trace(
        &shown.commitment_bases["trust@2"].provenance,
        &["buoy", "chart", "gauge", "second"],
        &["age"],
    );
    // Grounds: what each revision was actually made on.
    trace(&shown.commitment_grounds["trust@1"], &["chart"], &["age"]);
    trace(&shown.commitment_grounds["trust@2"], &["second"], &[]);
    assert_grounds_within_lineage(&shown);
}

#[test]
fn committed_and_reopened_read_grounds_in_citations() {
    let mut game = session(
        "decisions trust limit 4;\n\
         on read when from_gauge > 0 commit trust because enough using from_chart;\n\
         on other reveal buoy opposes safe;\n\
         on other reopen trust because buoy;\n\
         bind hud.text = \"trusting\" when committed(trust) because committed(trust);\n\
         bind hud.why = \"doubting\" when reopened(trust) because reopened(trust);",
    );
    let shown = game.dispatch_json("read", "{}").unwrap();
    trace(
        &shown.binding_explanations["hud"]["text"],
        &["chart"],
        &["age"],
    );
    let shown = game.dispatch_json("other", "{}").unwrap();
    trace(
        &shown.binding_explanations["hud"]["why"],
        &["buoy", "chart"],
        &["age"],
    );
}

#[test]
fn grounds_are_published_and_rolled_back_with_the_event() {
    let mut game = session(
        "on read set x = from_chart;\n\
         on other set x = from_gauge;\n\
         on other set from_chart = 1 / 0;",
    );
    game.dispatch_json("read", "{}").unwrap();
    let before = game.snapshot();
    assert!(game.dispatch_json("other", "{}").is_err());
    assert_eq!(game.snapshot().value_grounds, before.value_grounds);
    let json = serde_json::to_value(game.snapshot()).unwrap();
    assert_eq!(
        json["value_grounds"]["x"]["evidence"],
        serde_json::json!(["chart"])
    );
    assert!(json["commitment_grounds"].is_object());
}
