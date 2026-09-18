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

#[test]
fn bindings_are_typed_ordered_and_recomputed_from_reached_state() {
    let source = r#"
        state amount = 1; state phase = 0;
        event advance;
        on advance set amount = 4;
        on advance set phase = 1;
        bind asset.body.position_x = amount * 2;
        bind panel.visible = phase == 1;
        bind panel.text = "Wait when uncertain; keep the caveat.";
        bind panel.text = "Ready" when phase == 1;
        bind panel.text = "Last matching rule" when amount > 2;
        bind transient.visible = true when phase == 0;
    "#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    let initial = game.snapshot();
    assert_eq!(
        initial.bindings["asset"]["body.position_x"].as_number(),
        Some(2.0)
    );
    assert_eq!(initial.bindings["panel"]["visible"].as_bool(), Some(false));
    assert_eq!(
        initial.bindings["panel"]["text"].as_str(),
        Some("Wait when uncertain; keep the caveat.")
    );
    let next = game.dispatch_json("advance", "{}").unwrap();
    assert_eq!(
        next.bindings["asset"]["body.position_x"].as_number(),
        Some(8.0)
    );
    assert_eq!(
        next.bindings["panel"]["text"].as_str(),
        Some("Last matching rule")
    );
    assert!(!next.bindings.contains_key("transient"));
    let json = serde_json::to_value(next).unwrap();
    assert_eq!(json["bindings"]["panel"]["visible"], true);
}

#[test]
fn declared_cues_emit_only_for_the_current_successful_event() {
    let source = r##"
        place sea kind sea; entity marker kind buoy at sea;
        event show; event idle;
        cue note toast "Observed; not settled." 2;
        cue chime sound 440 0.2 0.05;
        cue impact flash overlay 0.1;
        cue circle ring marker "#aBcDef" 0.8;
        cue numeric ring marker 16711680 0.3;
        on show emit note; on show emit chime; on show emit impact;
        on show emit circle; on show emit numeric;
    "##;
    let mut game = ReactiveSession::from_source(source).unwrap();
    assert!(game.snapshot().cues.is_empty());
    let shown = game.dispatch_json("show", "{}").unwrap();
    assert_eq!(
        shown.cues.iter().map(|cue| cue.id()).collect::<Vec<_>>(),
        ["note", "chime", "impact", "circle", "numeric"]
    );
    let json = serde_json::to_value(&shown).unwrap();
    assert_eq!(json["cues"][0]["kind"], "toast");
    assert_eq!(json["cues"][1]["frequency"], 440.0);
    assert_eq!(json["cues"][3]["color"], "#aBcDef");
    assert_eq!(json["cues"][4]["color"], 16711680);
    assert!(game.dispatch_json("idle", "{}").unwrap().cues.is_empty());
}

#[test]
fn a_failed_post_event_binding_rolls_back_numeric_graph_budget_commitment_and_cues() {
    let source = r#"
        budget 1; claim safe; evidence reading from "sensor";
        caveat fault consequence high; fault qualifies reading;
        state divisor = 1; event show; event fail;
        cue note toast "Measured." 1;
        bind panel.value = 1 / divisor;
        on show emit note;
        on fail commit course because enough retaining fault;
        on fail reveal reading opposes safe;
        on fail examine fault cost 1;
        on fail reopen course because reading;
        on fail emit note;
        on fail set divisor = 0;
    "#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    let before = game.dispatch_json("show", "{}").unwrap();
    assert!(game
        .dispatch_json("fail", "{}")
        .unwrap_err()
        .contains("binding panel.value"));
    assert_eq!(game.snapshot(), before);
    assert_eq!(
        game.snapshot().cues.len(),
        1,
        "rollback preserves the previous accepted snapshot too"
    );
    assert!(game.snapshot().commitments.is_empty());
    assert_eq!(game.snapshot().budget.unwrap().spent, 0);

    assert!(
        ReactiveSession::from_source("state x = 0; event go; bind panel.value = 1 / x;")
            .unwrap_err()
            .contains("binding panel.value")
    );
}

#[test]
fn controls_and_clock_are_validated_source_metadata() {
    let source = "event begin; event frame dt min 0 max 0.2; control start = begin; control retry = begin reset; clock frame every 0.05;";
    let game = ReactiveSession::from_source(source).unwrap();
    let snapshot = game.snapshot();
    assert_eq!(snapshot.controls["start"].event, "begin");
    assert!(!snapshot.controls["start"].reset);
    assert!(snapshot.controls["retry"].reset);
    assert_eq!(snapshot.clock.as_ref().unwrap().event, "frame");
    assert_eq!(snapshot.clock.as_ref().unwrap().step.value(), 0.05);
    assert_eq!(
        snapshot.sequence, 0,
        "declarations do not dispatch or reset by themselves"
    );
    let legacy = ReactiveSession::from_source("event go;")
        .unwrap()
        .snapshot();
    assert!(legacy.clock.is_none());
    assert!(legacy.controls.is_empty());
    assert!(legacy.bindings.is_empty());
    assert!(legacy.cues.is_empty());
}

#[test]
fn functions_are_reusable_in_initializers_rules_and_bindings() {
    let source = r#"
        fn distance(ax, az, bx, bz) = sqrt((ax-bx)*(ax-bx)+(az-bz)*(az-bz));
        fn twice(value) = value * 2;
        state value = distance(0, 0, 3, 4);
        event increase delta min 0 max 10;
        on increase set value = twice(value + delta);
        bind boat.heading = atan2(value, 0);
        bind panel.value = twice(value);
    "#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    assert_eq!(game.snapshot().values["value"], 5.0);
    let next = game.dispatch_json("increase", r#"{"delta":1}"#).unwrap();
    assert_eq!(next.values["value"], 12.0);
    assert_eq!(next.bindings["panel"]["value"].as_number(), Some(24.0));
    assert_eq!(
        next.bindings["boat"]["heading"].as_number(),
        Some(std::f64::consts::FRAC_PI_2)
    );
}

#[test]
fn invalid_presentation_controls_clocks_and_functions_fail_before_dispatch() {
    let invalid = [
        "event go; on go emit missing;",
        "event go; cue c toast \"a\" 1; cue c toast \"b\" 1;",
        "event go; cue c sound 0 1 0.2;",
        "event go; cue c sound 440 0 0.2;",
        "event go; cue c sound 440 1 2;",
        "event go; cue c toast \"a\" 31;",
        "event go; cue c ring missing \"#aabbcc\" 1;",
        "place room kind room; entity marker kind buoy at room; event go; cue c ring marker \"red\" 1;",
        "place room kind room; entity marker kind buoy at room; event go; cue c ring marker 16777216 1;",
        "event go; control start = absent;",
        "event go; control start = go; control start = go;",
        "event go; clock go every 0.1;",
        "event go dt min 0 max 0.1; clock go every 0;",
        "event go dt min 0 max 0.1; clock go every 0.2;",
        "event go dt min 0 max 0.1; clock absent every 0.1;",
        "event go dt min 0 max 0.1; clock go every 0.1; clock go every 0.1;",
        "event go; bind ui..value = 1;",
        "event go; bind ui.value = absent;",
        "event go n min 0 max 1; bind ui.value = n;",
        "event go; bind ui.value = 1 when 1;",
        "event go; bind ui.value = 1; bind ui.value = \"mixed\";",
        "event go; fn f(x) = x; fn f(x) = x;",
        "event go; fn f(x) = f(x);",
        "state x = 1; event go; fn f(y) = x + y;",
        "evidence e from sensor; event go; fn f(x) = observed(e);",
        "event go; fn f(x) = x; bind ui.value = f(1, 2);",
    ];
    for source in invalid {
        assert!(
            ReactiveSession::from_source(source).is_err(),
            "accepted {source}"
        );
    }
}

#[test]
fn examination_does_not_discharge_uncertainty_or_make_objections_automatic_vetoes() {
    let source = r#"
        budget 1; claim safe;
        evidence chart from "old chart"; evidence warning from "new measurement";
        caveat uncertainty consequence high;
        chart supports safe; uncertainty qualifies chart;
        state steps = 0;
        event start; event inspect; event disclose; event reconsider; event act;
        on start commit course because enough retaining uncertainty;
        on inspect examine uncertainty cost 1;
        on disclose reveal warning opposes safe;
        on reconsider reopen course because warning;
        on act when committed(course) and not reopened(course) set steps = steps + 1;
        bind ui.examined = examined(uncertainty);
        bind ui.warning = observed(warning);
    "#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    let committed = game.dispatch_json("start", "{}").unwrap();
    let examined = game.dispatch_json("inspect", "{}").unwrap();
    assert_eq!(
        examined.relations, committed.relations,
        "examination changes attention and cost, not graph claims"
    );
    assert_eq!(examined.commitments[0].retained, ["uncertainty"]);
    assert_eq!(examined.budget.unwrap().spent, 1);
    assert_eq!(examined.bindings["ui"]["examined"].as_bool(), Some(true));
    let disclosed = game.dispatch_json("disclose", "{}").unwrap();
    assert!(disclosed
        .relations
        .iter()
        .any(|edge| edge.relation == "qualifies"));
    assert!(disclosed
        .relations
        .iter()
        .any(|edge| edge.relation == "retains"));
    assert!(disclosed
        .relations
        .iter()
        .any(|edge| edge.relation == "supports"));
    assert!(disclosed
        .relations
        .iter()
        .any(|edge| edge.relation == "opposes"));
    assert_eq!(
        game.dispatch_json("act", "{}").unwrap().values["steps"],
        1.0,
        "a recorded objection is not itself authority to halt"
    );
    game.dispatch_json("reconsider", "{}").unwrap();
    assert_eq!(
        game.dispatch_json("act", "{}").unwrap().values["steps"],
        1.0
    );
}
