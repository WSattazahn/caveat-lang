//! Dispatch outcomes distinguish expected source/input refusal from fatal faults.
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::web::WebReactiveSession;
use serde_json::{json, Value};
use std::collections::BTreeMap;

const SCHEMA: &str = "caveat-dispatch/0.1";

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn session(source: &str) -> WebReactiveSession {
    WebReactiveSession::new(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn checkpoint(session: &WebReactiveSession) -> (Value, Value, Value) {
    (
        json(&session.snapshot()),
        json(&session.view()),
        json(&session.save().unwrap()),
    )
}

fn rejected(
    session: &mut WebReactiveSession,
    event: &str,
    payload: &str,
    origin: &str,
    code: &str,
) -> Value {
    let before = checkpoint(session);
    let result = json(&session.dispatch_outcome(event, payload).unwrap());
    assert_eq!(result["schema"], SCHEMA);
    assert_eq!(result["outcome"], "rejected");
    assert_eq!(result["origin"], origin);
    assert_eq!(result["code"], code);
    assert!(result["message"].as_str().is_some());
    assert!(result.get("snapshot").is_none());
    assert_eq!(checkpoint(session), before, "{event} {payload}");
    result
}

const TIMED: &str = r#"
budget 4;
claim safe;
evidence sight from "a sighting";
evidence sensor from "the instrument";
evidence unseen from "the next observation";
caveat stale consequence material;
caveat calibration consequence high;
calibration qualifies sensor;
readings readings_log from sensor limit 4;
decisions route limit 4;
state output = 0;
state blocked = 1;
event seed;
event unblock;
event advance dt min 0 max 1;
clock advance every 0.125;
cue note toast "Observed" 1;
on seed reveal sight supports safe;
on seed set output = qualified(1, sight);
on seed qualify sight with stale after 0.25;
on seed commit route because enough using output;
on seed emit note;
on unblock set blocked = 0;
proc mutate() {
    set output = qualified(2, sight);
    sample readings_log = 4 supports safe;
    reveal unseen opposes safe;
    examine calibration cost 1;
    reopen route because unseen;
    commit route because enough using latest(readings_log);
    emit note;
};
on advance when elapsed() >= 0.25 call mutate();
bind hud.value = output;
bind hud.stale = carries(sight, stale);
bind hud.text = "Observed" when output > 0 because output;
"#;

#[test]
fn accepted_outcomes_contain_the_full_legacy_snapshot() {
    let mut outcome = session(TIMED);
    let mut legacy = session(TIMED);
    let mut viewed = session(TIMED);
    let mut core = ReactiveSession::from_source(TIMED).unwrap();
    for (event, payload) in [
        ("seed", "{}"),
        ("unblock", "{}"),
        ("advance", r#"{"dt":0.25}"#),
    ] {
        let result = json(&outcome.dispatch_outcome(event, payload).unwrap());
        let snapshot = json(&legacy.dispatch(event, payload).unwrap());
        assert_eq!(
            result,
            json!({
                "schema": SCHEMA,
                "outcome": "accepted",
                "snapshot": snapshot,
            })
        );
        assert_eq!(checkpoint(&outcome), checkpoint(&legacy));
        assert_eq!(
            json(&viewed.dispatch_view(event, payload).unwrap()),
            json(&outcome.view())
        );
        assert_eq!(
            serde_json::to_value(core.dispatch_outcome_json(event, payload).unwrap()).unwrap(),
            result,
        );
    }
    let snapshot = json(&outcome.snapshot());
    assert!(snapshot.get("world").is_some());
    assert!(snapshot.get("events").is_some());
    assert!(snapshot.get("qualified_values").is_some());
    assert_eq!(
        snapshot["reading_streams"]["readings_log"]["occurrences"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn direct_and_nested_policy_rejections_preserve_authored_text_and_roll_back_every_surface() {
    const MESSAGE: &str = "rejected: Keep the route; 条件 {pending}.";
    for refusal in [
        format!(r#"on advance when blocked == 1 reject "{MESSAGE}";"#),
        format!(
            r#"
            proc inner() {{ reject "{MESSAGE}"; }};
            proc outer() {{ call inner(); }};
            on advance when blocked == 1 call outer();
        "#
        ),
    ] {
        let source = format!("{TIMED}\n{refusal}");
        let mut game = session(&source);
        let mut control = session(&source);
        game.dispatch("seed", "{}").unwrap();
        control.dispatch("seed", "{}").unwrap();
        let before = json(&game.snapshot());
        assert_eq!(
            before["scheduled_qualifications"].as_array().unwrap().len(),
            1
        );
        assert_eq!(before["cues"].as_array().unwrap().len(), 1);
        let result = rejected(&mut game, "advance", r#"{"dt":0.25}"#, "policy", "reject");
        assert_eq!(result["message"], MESSAGE);
        assert_eq!(checkpoint(&game), checkpoint(&control));

        // The failed event crossed a due qualification and changed graph,
        // attention, history, decisions, cues and state. Retrying must allocate
        // the same occurrence identities and apply the timer exactly once.
        for program in [&mut game, &mut control] {
            program.dispatch("unblock", "{}").unwrap();
            program.dispatch("advance", r#"{"dt":0.25}"#).unwrap();
        }
        assert_eq!(checkpoint(&game), checkpoint(&control));
        let after = json(&game.snapshot());
        assert_eq!(after["elapsed"], 0.25);
        assert_eq!(after["bindings"]["hud"]["stale"], true);
        assert!(after["scheduled_qualifications"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(
            after["reading_streams"]["readings_log"]["current"],
            "readings_log@1"
        );
        assert_eq!(after["decision_series"]["route"]["current"], "route@2");
        assert_eq!(after["budget"]["spent"], 1);
    }
}

#[test]
fn malformed_missing_extra_duplicate_null_and_wrongly_typed_payloads_are_input_rejections() {
    let mut game = session(
        r#"
        place garden kind garden;
        entity cave kind mushroom at garden;
        entity pool kind mushroom at garden;
        state selected = 0;
        event number value min 0 max 1;
        event choose target kind mushroom, sort in glowcap duskcap;
        on number set selected = value;
        on choose set selected = target + sort;
    "#,
    );
    game.dispatch("number", r#"{"value":0.5}"#).unwrap();
    for payload in [
        "{",
        "null",
        "[]",
        "{}",
        r#"{"value":0.2,"extra":1}"#,
        r#"{"value":0.1,"value":0.2}"#,
        r#"{"value":"0.5"}"#,
        r#"{"value":true}"#,
        r#"{"value":null}"#,
        r#"{"value":{}}"#,
        r#"{"value":[]}"#,
        r#"{"value":1e999}"#,
    ] {
        rejected(&mut game, "number", payload, "input", "payload_invalid");
    }
    for payload in [
        r#"{"target":"missing","sort":"glowcap"}"#,
        r#"{"target":"pool","sort":"missing"}"#,
        r#"{"target":null,"sort":"glowcap"}"#,
        r#"{"target":"pool","target":"cave","sort":"glowcap"}"#,
    ] {
        rejected(&mut game, "choose", payload, "input", "payload_invalid");
    }
    rejected(&mut game, "missing", "{}", "input", "unknown_event");
    rejected(
        &mut game,
        "choose",
        r#"{"target":3,"sort":1}"#,
        "input",
        "bound_exceeded",
    );
}

#[test]
fn input_bounds_and_state_bounds_have_different_origins() {
    let mut game = session(
        r#"
        state output = 0 min 0 max 1;
        event set_value value min 0 max 2;
        on set_value set output = value;
    "#,
    );
    game.dispatch("set_value", r#"{"value":0.5}"#).unwrap();
    rejected(
        &mut game,
        "set_value",
        r#"{"value":3}"#,
        "input",
        "bound_exceeded",
    );
    rejected(
        &mut game,
        "set_value",
        r#"{"value":2}"#,
        "evaluation",
        "bound_exceeded",
    );
}

#[test]
fn live_and_skipped_procedure_work_share_the_classified_event_budget() {
    let body = "set output = output + 1;".repeat(2048);
    for guard in ["true", "false"] {
        let source = format!(
            "state output = 0; event run; proc many() {{ {body} }}; \
             on run when {guard} call many(); on run when {guard} call many();"
        );
        let mut game = session(&source);
        rejected(&mut game, "run", "{}", "limit", "work_limit");
    }
}

// A full reading stream or decision series refuses the event that would add
// to it, classified, and the session goes on; it is not a fatal fault.
#[test]
fn full_histories_refuse_the_event_and_the_session_continues() {
    let mut game = session(
        r#"
        claim safe;
        evidence gauge from "a gauge";
        readings depth from gauge limit 2;
        decisions route limit 1;
        state doubts = 0 min 0 max 9;
        event read value min 0 max 9;
        event decide;
        event doubt;
        on read sample depth = value supports safe;
        on decide commit route because enough using latest(depth);
        on doubt when committed(route) and not reopened(route) reopen route because latest(depth);
        on doubt set doubts = doubts + 1;
    "#,
    );
    game.dispatch("read", r#"{"value":1}"#).unwrap();
    game.dispatch("read", r#"{"value":2}"#).unwrap();
    let refused = rejected(
        &mut game,
        "read",
        r#"{"value":3}"#,
        "limit",
        "history_limit",
    );
    // Like other diagnostics, the message keeps its rule prefix.
    assert_eq!(
        refused["message"],
        "event read, rule 1: reading stream depth reached its history limit 2"
    );

    game.dispatch("decide", "{}").unwrap();
    game.dispatch("doubt", "{}").unwrap();
    let refused = rejected(&mut game, "decide", "{}", "limit", "history_limit");
    assert!(refused["message"]
        .as_str()
        .unwrap()
        .ends_with("decision series route reached its history limit 1"));

    // Still usable: the next event is accepted and nothing was published by the
    // refused ones.
    let accepted = json(&game.dispatch_outcome("doubt", "{}").unwrap());
    assert_eq!(accepted["outcome"], "accepted");
    assert_eq!(accepted["snapshot"]["values"]["doubts"], 2.0);
    assert_eq!(
        accepted["snapshot"]["reading_streams"]["depth"]["occurrences"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        accepted["snapshot"]["decision_series"]["route"]["revisions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn the_maximum_valid_procedure_depth_still_dispatches() {
    let mut source = String::from("state output = 0; event run; proc p0() { set output = 1; };");
    for index in 1..64 {
        source.push_str(&format!("proc p{index}() {{ call p{}(); }};", index - 1));
    }
    source.push_str("on run call p63();");
    let mut game = session(&source);
    let result = json(&game.dispatch_outcome("run", "{}").unwrap());
    assert_eq!(result["outcome"], "accepted");
    assert_eq!(result["snapshot"]["values"]["output"], 1.0);
    // Deeper call graphs cannot load, so the defensive runtime depth limit is
    // exercised by the core's internal test rather than invalid public source.
    source.push_str("proc p64() { call p63(); };");
    assert!(WebReactiveSession::new(&source)
        .err()
        .unwrap()
        .contains("depth"));
}

#[test]
fn unclassified_expression_errors_are_fatal_and_atomic() {
    for expression in ["require(false, 1)", "1 / 0"] {
        let source = format!(
            r#"
            state output = 0;
            event run;
            on run set output = 1;
            on run set output = {expression};
        "#
        );
        let mut game = session(&source);
        let mut legacy = session(&source);
        let mut core = ReactiveSession::from_source(&source).unwrap();
        let before = checkpoint(&game);
        let error = json(&game.dispatch_outcome("run", "{}").unwrap_err());
        assert_eq!(error["schema"], SCHEMA);
        assert_eq!(error["outcome"], "fatal");
        assert_eq!(error["code"], "unclassified");
        assert_eq!(error["message"], legacy.dispatch("run", "{}").unwrap_err());
        assert!(error.get("origin").is_none());
        assert!(error.get("snapshot").is_none());
        assert_eq!(checkpoint(&game), before);
        assert_eq!(checkpoint(&legacy), before);
        assert_eq!(
            serde_json::to_value(core.dispatch_outcome_json("run", "{}").unwrap_err()).unwrap(),
            error,
        );
    }
}

#[test]
fn legacy_dispatch_view_and_apply_keep_their_error_text_and_behavior() {
    const SOURCE: &str = r#"
        state output = 0;
        event seed;
        event run;
        proc inner() { reject "authored refusal"; };
        on seed set output = 2;
        on run set output = 7;
        on run call inner();
        bind hud.value = output;
    "#;
    let mut outcome = session(SOURCE);
    let mut legacy = session(SOURCE);
    let mut viewed = session(SOURCE);
    let mut applied = ReactiveSession::from_source(SOURCE).unwrap();
    for game in [&mut outcome, &mut legacy, &mut viewed] {
        game.dispatch("seed", "{}").unwrap();
    }
    applied.apply("seed", &BTreeMap::new()).unwrap();
    let before = checkpoint(&outcome);
    let result = rejected(&mut outcome, "run", "{}", "policy", "reject");
    assert_eq!(result["message"], "authored refusal");
    let error = "event run, rule 3: procedure inner, step 1: rejected: authored refusal";
    assert_eq!(legacy.dispatch("run", "{}").unwrap_err(), error);
    assert_eq!(viewed.dispatch_view("run", "{}").unwrap_err(), error);
    assert_eq!(applied.apply("run", &BTreeMap::new()).unwrap_err(), error);
    assert_eq!(checkpoint(&legacy), before);
    assert_eq!(checkpoint(&viewed), before);
    assert_eq!(serde_json::to_value(applied.snapshot()).unwrap(), before.0);
    assert_eq!(json(&applied.save_json().unwrap()), before.2);
}
