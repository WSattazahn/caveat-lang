use caveat_runtime::reactive::{ReactiveSession, REACTIVE_SCHEMA};
use caveat_runtime::web::WebReactiveSession;
use std::collections::BTreeMap;

const SOURCE: &str = r#"
scene "An uncertain approach.";
display course "Follow the measured route";
budget 2;
claim clear;
evidence chart from "old chart";
evidence stone from "lookout";
caveat fog consequence high;
chart supports clear;
fog qualifies chart;
place sea kind sea;
entity buoy kind buoy at sea;
position sea at 0 0 0;
position buoy at 3 0 4;
state x = buoy.x min -10 max 10;
state velocity = 1 min -10 max 10;
state distance = 0;
event start;
event aim target min -10 max 10;
event scan strength min 0 max 1;
event tick dt min 0 max 0.1;
on start when not committed(course) commit course because enough retaining fog;
on aim set velocity = target;
on tick set x = x + velocity * dt;
on tick when observed(stone) set distance = distance + dt;
on scan when strength >= 0.5 reveal stone opposes clear;
on scan when observed(stone) and not examined(fog) examine fog cost 1;
on scan when observed(stone) and committed(course) reopen course because stone;
"#;

#[test]
fn source_world_coordinates_initialize_numeric_state_and_bound_events() {
    let game = ReactiveSession::from_source(SOURCE).unwrap();
    let snapshot = game.snapshot();
    assert_eq!(snapshot.schema, REACTIVE_SCHEMA);
    assert_eq!(snapshot.values["x"], 3.0);
    assert_eq!(snapshot.sequence, 0);
    assert!(snapshot.last_event.is_none());
    assert_eq!(snapshot.labels["course"], "Follow the measured route");
    assert_eq!(
        snapshot
            .events
            .iter()
            .find(|event| event.name == "tick")
            .unwrap()
            .parameters[0]
            .max
            .value(),
        0.1
    );
    let moved =
        ReactiveSession::from_source(&SOURCE.replace("position buoy at 3", "position buoy at 8"))
            .unwrap()
            .snapshot();
    assert_eq!(moved.values["x"], 8.0);
    assert_eq!(
        moved.world.presentation.positions[1].position[0].value(),
        8.0
    );
    assert_ne!(moved.source_id, snapshot.source_id);
}

#[test]
fn rules_execute_in_source_order_against_evolving_numeric_state() {
    let source = "state a = 0; state b = a + 2; event go; on go set a = 4; on go when a == 4 set b = a * 3; on go when b == 12 set a = sqrt(b + 4);";
    let mut game = ReactiveSession::from_source(source).unwrap();
    let result = game.dispatch_json("go", "{}").unwrap();
    assert_eq!(result.values["a"], 4.0);
    assert_eq!(result.values["b"], 12.0);
    assert_eq!(result.sequence, 1);
}

#[test]
fn live_graph_reasoning_changes_simulation_and_preserves_contradiction() {
    let mut game = ReactiveSession::from_source(SOURCE).unwrap();
    game.dispatch_json("start", "{}").unwrap();
    let before = game.dispatch_json("tick", r#"{"dt":0.1}"#).unwrap();
    assert_eq!(
        before.values["distance"], 0.0,
        "a declaration is not an observation"
    );
    assert!(!before.commitments[0].open);
    let revealed = game.dispatch_json("scan", r#"{"strength":1}"#).unwrap();
    assert!(revealed
        .relations
        .iter()
        .any(|edge| edge.from == "chart" && edge.relation == "supports"));
    assert!(revealed
        .relations
        .iter()
        .any(|edge| edge.from == "stone" && edge.relation == "opposes"));
    assert_eq!(
        revealed
            .symbols
            .iter()
            .find(|symbol| symbol.name == "fog")
            .unwrap()
            .attention
            .as_deref(),
        Some("examined")
    );
    assert_eq!(revealed.budget.as_ref().unwrap().spent, 1);
    assert!(revealed.commitments[0].open);
    assert_eq!(revealed.commitments[0].retained, ["fog"]);
    assert_eq!(revealed.commitments[0].reopened_by, ["stone"]);
    assert_eq!(revealed.effects.len(), 3);
    let after = game.dispatch_json("tick", r#"{"dt":0.1}"#).unwrap();
    assert_eq!(after.values["distance"], 0.1);
}

#[test]
fn repeated_observations_and_reopenings_do_not_duplicate_graph_edges() {
    let mut game = ReactiveSession::from_source(SOURCE).unwrap();
    game.dispatch_json("start", "{}").unwrap();
    let first = game.dispatch_json("scan", r#"{"strength":1}"#).unwrap();
    let second = game.dispatch_json("scan", r#"{"strength":1}"#).unwrap();
    assert_eq!(first.relations, second.relations);
    assert_eq!(first.budget, second.budget);
    assert_eq!(first.commitments, second.commitments);
    assert!(second.effects.is_empty());
}

#[test]
fn an_error_after_graph_and_numeric_effects_rolls_back_the_entire_event() {
    let source = format!("{SOURCE} on scan set velocity = 2; on scan set x = 100;");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    game.dispatch_json("start", "{}").unwrap();
    let before = game.snapshot();
    assert!(game
        .dispatch_json("scan", r#"{"strength":1}"#)
        .unwrap_err()
        .contains("finite and in"));
    assert_eq!(game.snapshot(), before);

    let source = format!("{SOURCE} on start set x = 1 / 0;");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    let before = game.snapshot();
    assert!(game.dispatch_json("start", "{}").is_err());
    assert_eq!(
        game.snapshot(),
        before,
        "failed arithmetic must also discard the new commitment"
    );
}

#[test]
fn insufficient_attention_and_unobserved_reopening_are_atomic_errors() {
    let mut game = ReactiveSession::from_source(&SOURCE.replace("budget 2", "budget 0")).unwrap();
    let before = game.snapshot();
    assert!(game
        .dispatch_json("scan", r#"{"strength":1}"#)
        .unwrap_err()
        .contains("budget"));
    assert_eq!(game.snapshot(), before);

    let source = format!("{SOURCE} on start reopen course because stone;");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    let before = game.snapshot();
    assert!(game
        .dispatch_json("start", "{}")
        .unwrap_err()
        .contains("unobserved"));
    assert_eq!(game.snapshot(), before);
}

#[test]
fn payloads_reject_unknown_missing_extra_duplicate_and_non_numeric_values() {
    let mut game = ReactiveSession::from_source(SOURCE).unwrap();
    let before = game.snapshot();
    for payload in [
        "{}",
        r#"{"dt":0.2}"#,
        r#"{"dt":-0.1}"#,
        r#"{"dt":"0.1"}"#,
        r#"{"dt":true}"#,
        r#"{"dt":null}"#,
        r#"{"dt":0.1,"extra":1}"#,
        r#"{"dt":0.01,"dt":0.02}"#,
        r#"{"dt":1e999}"#,
        "null",
        "[]",
    ] {
        assert!(
            game.dispatch_json("tick", payload).is_err(),
            "accepted {payload}"
        );
        assert_eq!(game.snapshot(), before);
    }
    assert!(game
        .dispatch_json("undeclared", "{}")
        .unwrap_err()
        .contains("undeclared"));
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(game
            .dispatch("tick", &BTreeMap::from([("dt".into(), invalid)]))
            .is_err());
        assert_eq!(game.snapshot(), before);
    }
}

#[test]
fn source_validation_rejects_wrong_types_unknown_names_and_ambiguous_inputs() {
    let invalid = [
        "state x = 0; state x = 1; event go;",
        "state x = later; state later = 1; event go;",
        "state x = missing.x; event go;",
        "state x = true; event go;",
        "state x = 2 min 0 max 1; event go;",
        "state x = 0 min 2 max 1; event go;",
        "state x = 0; event go x min 0 max 1;",
        "event go x min 0 max 1, x min 0 max 2;",
        "event go; event go;",
        "event tick dt min 0 max 1;",
        "event tick;",
        "event tick dt min -1 max 0.1;",
        "event tick dt min 0 max 0.1, extra min 0 max 1;",
        "event go; on absent set x = 1;",
        "event go; on go set x = 1;",
        "state x = 0; event go; on go when x set x = 1;",
        "state x = 0; event go; on go set x = true;",
        "claim c; event go; on go reveal c supports c;",
        "evidence e from a; event go; on go reveal e supports e;",
        "claim c; event go; on go examine c cost 1;",
        "caveat c consequence low; event go; on go examine c cost 1;",
        "claim c; event go; on go commit act because enough retaining c;",
        "claim c; event go; on go commit c because enough;",
        "evidence e from a; event go; on go reopen unknown because e;",
        "state x = 0; event go; on go when committed(unknown) set x = 1;",
        "state x = 0; event go; on go when observed(x) set x = 1;",
    ];
    for source in invalid {
        assert!(
            ReactiveSession::from_source(source).is_err(),
            "accepted {source}"
        );
    }
}

#[test]
fn repeated_same_event_stream_has_identical_snapshots() {
    let mut first = ReactiveSession::from_source(SOURCE).unwrap();
    let mut second = ReactiveSession::from_source(SOURCE).unwrap();
    for (event, payload) in [
        ("start", "{}"),
        ("aim", r#"{"target":-2}"#),
        ("tick", r#"{"dt":0.1}"#),
        ("scan", r#"{"strength":1}"#),
        ("tick", r#"{"dt":0.07}"#),
    ] {
        assert_eq!(
            first.dispatch_json(event, payload).unwrap(),
            second.dispatch_json(event, payload).unwrap()
        );
    }
    assert_eq!(
        serde_json::to_string(&first.snapshot()).unwrap(),
        serde_json::to_string(&second.snapshot()).unwrap()
    );
}

#[test]
fn effect_words_are_legal_identifiers_and_spaced_predicates_are_unambiguous() {
    let source = "budget 1; caveat examine consequence low; examine examine cost 1; state set = 1; event go; on go when set > 0 and examined( examine ) set set = set + 1;";
    let mut game = ReactiveSession::from_source(source).unwrap();
    assert_eq!(game.dispatch_json("go", "{}").unwrap().values["set"], 2.0);
}

#[test]
fn web_bridge_exposes_events_source_world_and_only_executed_graph_effects() {
    let mut web = WebReactiveSession::new(SOURCE).unwrap();
    let initial: serde_json::Value = serde_json::from_str(&web.snapshot()).unwrap();
    assert_eq!(initial["schema"], REACTIVE_SCHEMA);
    assert_eq!(initial["values"]["x"], 3.0);
    assert!(initial["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["name"] == "tick" && event["parameters"][0]["max"] == 0.1));
    assert!(initial["commitments"].as_array().unwrap().is_empty());
    let snapshot: serde_json::Value =
        serde_json::from_str(&web.dispatch("start", "{}").unwrap()).unwrap();
    assert_eq!(snapshot["commitments"][0]["retained"][0], "fog");
    let before = web.snapshot();
    assert!(web.dispatch("tick", r#"{"dt":10}"#).is_err());
    assert_eq!(web.snapshot(), before);
}
