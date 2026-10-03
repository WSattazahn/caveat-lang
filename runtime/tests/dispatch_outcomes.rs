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
    // The view path refuses the same event with the same text, and changes
    // nothing either, so the refusal below runs on the same session.
    let viewed = session.dispatch_view_outcome(event, payload).unwrap();
    assert_eq!(checkpoint(session), before, "view path: {event} {payload}");
    let wire = session.dispatch_outcome(event, payload).unwrap();
    assert_eq!(viewed, wire, "view path: {event} {payload}");
    let result = json(&wire);
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

const PULL_REQUESTS: &str = r#"
place repo kind repo;
entity pr_a kind pr at repo;
entity pr_b kind pr at repo;
claim changed;
event pushed target kind pr;
for pr as $p {
    evidence $p_push from "a new commit on $p";
    renewable $p_push limit 2;
    state $p_pushes = 0 min 0 max 9;
    on pushed when target == $index set $p_pushes = $p_pushes + 1;
    on pushed when target == $index and observed($p_push) renew $p_push;
    on pushed when target == $index reveal $p_push supports changed;
};
"#;

// A renewable evidence at its limit refuses the event that would renew it,
// classified, and the session goes on. The program has the agent ledger's
// shape: one renewable push per pull request, so one pull request running out
// of renewals must not end the session for the others.
#[test]
fn a_full_renewal_refuses_the_event_and_other_subjects_continue() {
    let mut game = session(PULL_REQUESTS);
    let a = r#"{"target":"pr_a"}"#;
    let b = r#"{"target":"pr_b"}"#;
    game.dispatch("pushed", a).unwrap();
    game.dispatch("pushed", a).unwrap();
    // The third push would need pr_a_push@3. The count set by the rule before
    // the renew is rolled back with the rest of the event.
    let refused = rejected(&mut game, "pushed", a, "limit", "renewal_limit");
    assert_eq!(
        refused["message"],
        "event pushed, rule 2: renewable pr_a_push reached its limit 2"
    );
    // Refused again, still classified: the limit is a state, not a one-off.
    rejected(&mut game, "pushed", a, "limit", "renewal_limit");

    // The other pull request is unaffected.
    let accepted = json(&game.dispatch_outcome("pushed", b).unwrap());
    assert_eq!(accepted["outcome"], "accepted");
    let snapshot = &accepted["snapshot"];
    assert_eq!(snapshot["values"]["pr_a_pushes"], 2.0);
    assert_eq!(snapshot["values"]["pr_b_pushes"], 1.0);
    assert_eq!(
        snapshot["renewals"]["pr_a_push"]["occurrences"],
        json!(["pr_a_push", "pr_a_push@2"])
    );
    assert_eq!(
        snapshot["renewals"]["pr_b_push"]["occurrences"],
        json!(["pr_b_push"])
    );

    // The same holds after a save and restore.
    let restored = WebReactiveSession::restore(PULL_REQUESTS, &game.save().unwrap());
    let mut restored = restored.unwrap_or_else(|error| panic!("restore failed: {error}"));
    rejected(&mut restored, "pushed", a, "limit", "renewal_limit");
    let accepted = json(&restored.dispatch_outcome("pushed", b).unwrap());
    assert_eq!(accepted["outcome"], "accepted");
    assert_eq!(accepted["snapshot"]["values"]["pr_b_pushes"], 2.0);
}

// Inside a procedure the refusal keeps its classification and gains the
// procedure's context, like every other diagnostic.
#[test]
fn a_full_renewal_inside_a_procedure_is_still_classified() {
    let mut game = session(
        r#"
        claim safe;
        evidence bite from "a bite";
        renewable bite limit 1;
        event regrow;
        proc again() { renew bite; };
        on regrow call again();
    "#,
    );
    let refused = rejected(&mut game, "regrow", "{}", "limit", "renewal_limit");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .ends_with("renewable bite reached its limit 1"),
        "{refused}"
    );
}

const DECISIONS: &str = r#"
claim safe;
evidence gauge from "a gauge";
readings depth from gauge limit 4;
decisions route limit 4;
state decides = 0 min 0 max 9;
event read value min 0 max 9;
event decide;
event doubt;
on read sample depth = value supports safe;
on decide set decides = decides + 1;
on decide commit route because enough using latest(depth);
on doubt when committed(route) and not reopened(route) reopen route because latest(depth);
"#;

// A commit to a decision series whose current revision is still in force,
// committed and not reopened, refuses the event, classified, and the session
// goes on. Reactive 0.5 requires the explicit reopening; the event is rolled
// back, so it is a refusal, not a fault.
#[test]
fn a_commit_while_the_decision_is_in_force_refuses_the_event_and_the_session_continues() {
    let mut game = session(DECISIONS);
    let mut legacy = session(DECISIONS);
    let mut core = ReactiveSession::from_source(DECISIONS).unwrap();
    for (event, payload) in [("read", r#"{"value":1}"#), ("decide", "{}")] {
        game.dispatch(event, payload).unwrap();
        legacy.dispatch(event, payload).unwrap();
        core.dispatch_json(event, payload).unwrap();
    }
    // The count set by the rule before the commit is rolled back with the rest
    // of the event.
    let refused = rejected(&mut game, "decide", "{}", "evaluation", "decision_in_force");
    let error = "event decide, rule 3: current decision in route must be explicitly reopened before revision";
    assert_eq!(refused["message"], error);
    // The legacy APIs keep their error text.
    assert_eq!(legacy.dispatch("decide", "{}").unwrap_err(), error);
    assert_eq!(core.dispatch_json("decide", "{}").unwrap_err(), error);
    // Refused again, still classified: a decision in force is a state, not a
    // one-off.
    rejected(&mut game, "decide", "{}", "evaluation", "decision_in_force");

    // Another event is accepted, and after the reopening the commit makes the
    // next revision.
    let accepted = json(&game.dispatch_outcome("read", r#"{"value":2}"#).unwrap());
    assert_eq!(accepted["outcome"], "accepted");
    game.dispatch("doubt", "{}").unwrap();
    let accepted = json(&game.dispatch_outcome("decide", "{}").unwrap());
    assert_eq!(accepted["outcome"], "accepted");
    let snapshot = &accepted["snapshot"];
    assert_eq!(snapshot["values"]["decides"], 2.0);
    assert_eq!(snapshot["decision_series"]["route"]["current"], "route@2");
    assert_eq!(snapshot["commitment_bases"]["route@2"]["value"], 2.0);

    // The same holds after a save and restore, with route@2 in force.
    let restored = WebReactiveSession::restore(DECISIONS, &game.save().unwrap());
    let mut restored = restored.unwrap_or_else(|error| panic!("restore failed: {error}"));
    rejected(
        &mut restored,
        "decide",
        "{}",
        "evaluation",
        "decision_in_force",
    );
    restored.dispatch("doubt", "{}").unwrap();
    let accepted = json(&restored.dispatch_outcome("decide", "{}").unwrap());
    assert_eq!(accepted["outcome"], "accepted");
    assert_eq!(
        accepted["snapshot"]["decision_series"]["route"]["current"],
        "route@3"
    );
}

// The authoring-trial program that found it (v1, run A2, final.cav), as it
// was frozen. Its `decide` rules set a number before the commit.
const COLD_STORAGE: &str = r#"
claim dispatch_safe;
evidence sensor_reading from "cold-storage temperature sensor";
renewable sensor_reading limit 4;
caveat calibration_uncertain consequence material;
caveat stale consequence material;
calibration_uncertain qualifies sensor_reading;
decisions dispatch limit 3;

state reading_count = 0 min 0 max 4;
state current_temperature = 0 min -10 max 20;
state plan_basis = 0;
state revision_count = 0 min 0 max 3;
state elapsed_time = 0 min 0 max 1000000000000;

event read temperature min -10 max 20;
event decide;
event advance dt min 0 max 2;
clock advance every 1;

on read when reading_count >= 4 reject "reading capacity exhausted";
on read when reading_count > 0 renew sensor_reading;
on read when temperature <= 4 reveal sensor_reading supports dispatch_safe;
on read when temperature > 4 reveal sensor_reading opposes dispatch_safe;
on read set current_temperature = temperature;
on read set reading_count = reading_count + 1;
on read qualify sensor_reading with stale after 2;

on decide when not observed(sensor_reading) reject "a reading is required";
on decide when carries(sensor_reading, stale) reject "the latest reading is stale";
on decide set plan_basis = qualified(if(current_temperature <= 4, 1, 0), sensor_reading);
on decide commit dispatch because enough using plan_basis;
on decide set revision_count = revision_count + 1;

on advance set elapsed_time = elapsed_time + dt;
on advance when committed(dispatch) and not reopened(dispatch) and has_caveat(plan_basis, stale)
    reopen dispatch because caveated(plan_basis, stale);

bind hud.readings = reading_count;
bind hud.temperature = current_temperature;
bind hud.recommendation = if(reading_count == 0, "waiting", if(carries(sensor_reading, stale), "stale", if(current_temperature <= 4, "release", "block")));
bind hud.decision = if(not committed(dispatch), "none", if(reopened(dispatch), "review", if(latest(dispatch) == 1, "release", "block")));
bind hud.frozen = if(committed(dispatch), latest(dispatch), -1);
bind hud.revision = revision_count;
bind hud.elapsed = elapsed_time;
"#;

// The trial's sequence: read, decide, decide. The second decide ended the
// session; now it is refused and the trial goes on to its next revision.
#[test]
fn a_second_decide_in_the_trial_program_is_refused_and_the_trial_goes_on() {
    let mut game = session(COLD_STORAGE);
    game.dispatch("read", r#"{"temperature":1}"#).unwrap();
    game.dispatch("decide", "{}").unwrap();
    let refused = rejected(&mut game, "decide", "{}", "evaluation", "decision_in_force");
    assert_eq!(
        refused["message"],
        "event decide, rule 11: current decision in dispatch must be explicitly reopened before revision"
    );
    let shot = json(&game.snapshot());
    assert_eq!(shot["values"]["revision_count"], 1.0);
    assert_eq!(shot["bindings"]["hud"]["decision"], "release");

    // The program's own way on: the reading goes stale, which reopens the
    // decision, and a fresh reading decides again.
    for (event, payload) in [("advance", r#"{"dt":2}"#), ("read", r#"{"temperature":5}"#)] {
        let accepted = json(&game.dispatch_outcome(event, payload).unwrap());
        assert_eq!(accepted["outcome"], "accepted", "{event}");
    }
    let restored = WebReactiveSession::restore(COLD_STORAGE, &game.save().unwrap());
    let mut restored = restored.unwrap_or_else(|error| panic!("restore failed: {error}"));
    for program in [&mut game, &mut restored] {
        let accepted = json(&program.dispatch_outcome("decide", "{}").unwrap());
        assert_eq!(accepted["outcome"], "accepted");
        let snapshot = &accepted["snapshot"];
        assert_eq!(
            snapshot["decision_series"]["dispatch"]["current"],
            "dispatch@2"
        );
        assert_eq!(snapshot["bindings"]["hud"]["decision"], "block");
        rejected(program, "decide", "{}", "evaluation", "decision_in_force");
    }
}

// Inside a procedure, with the series passed as a parameter, the refusal keeps
// its classification and gains the procedure's context.
#[test]
fn a_commit_in_force_inside_a_procedure_is_still_classified() {
    let mut game = session(
        r#"
        claim safe;
        evidence gauge from "a gauge";
        readings depth from gauge limit 4;
        decisions route limit 4;
        event read value min 0 max 9;
        event decide;
        proc settle(d decisions, s readings) { commit d because enough using latest(s); };
        on read sample depth = value supports safe;
        on decide call settle(route, depth);
    "#,
    );
    game.dispatch("read", r#"{"value":1}"#).unwrap();
    game.dispatch("decide", "{}").unwrap();
    let refused = rejected(&mut game, "decide", "{}", "evaluation", "decision_in_force");
    assert_eq!(
        refused["message"],
        "event decide, rule 2: procedure settle[route, depth], step 1: current decision in route must be explicitly reopened before revision"
    );
    let accepted = json(&game.dispatch_outcome("read", r#"{"value":2}"#).unwrap());
    assert_eq!(accepted["outcome"], "accepted");
}

// A permitted commit on a decision in force is refused for that, not as a
// denial, whether its grant is present or withdrawn: the decision is checked
// before the grant. Once the decision is reopened, the withdrawn grant is what
// refuses the commit.
#[test]
fn a_permitted_commit_while_the_decision_is_in_force_is_not_a_denial() {
    let mut game = session(
        r#"
        claim ready;
        claim may_merge;
        claim revoked;
        evidence ci from "the checks";
        evidence go from "a go-ahead";
        evidence revocation from "the go-ahead was taken back";
        readings checks from ci limit 4;
        readings approvals from go limit 4;
        decisions merge limit 4;
        event check;
        event approved;
        event merge;
        event revoke;
        event doubt;
        on check sample checks = 1 supports ready;
        on approved sample approvals = 1 supports may_merge;
        on merge commit merge because enough using latest(checks) permitted by latest(approvals);
        on revoke when not observed(revocation) reveal revocation supports revoked;
        on revoke withdraw latest(approvals) because revocation;
        on doubt when committed(merge) and not reopened(merge) reopen merge because latest(checks);
    "#,
    );
    for event in ["check", "approved", "merge"] {
        game.dispatch(event, "{}").unwrap();
    }
    // The grant is present and matches.
    rejected(&mut game, "merge", "{}", "evaluation", "decision_in_force");
    // The grant is withdrawn, and the decision is still in force.
    game.dispatch("revoke", "{}").unwrap();
    rejected(&mut game, "merge", "{}", "evaluation", "decision_in_force");
    // Reopened, the same commit is denied for its withdrawn grant.
    game.dispatch("doubt", "{}").unwrap();
    let refused = rejected(&mut game, "merge", "{}", "policy", "not_permitted");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("commit merge is not permitted: approvals@1 was withdrawn"),
        "{refused}"
    );
    // A fresh grant permits the next revision.
    game.dispatch("approved", "{}").unwrap();
    let accepted = json(&game.dispatch_outcome("merge", "{}").unwrap());
    assert_eq!(accepted["outcome"], "accepted");
    assert_eq!(
        accepted["snapshot"]["decision_series"]["merge"]["current"],
        "merge@2"
    );
}

// A full series whose decision is also in force is refused as full: the
// history limit is checked first.
#[test]
fn a_full_series_in_force_is_a_history_limit() {
    let mut game = session(
        r#"
        claim safe;
        evidence gauge from "a gauge";
        readings depth from gauge limit 4;
        decisions route limit 1;
        event read value min 0 max 9;
        event decide;
        on read sample depth = value supports safe;
        on decide commit route because enough using latest(depth);
    "#,
    );
    game.dispatch("read", r#"{"value":1}"#).unwrap();
    game.dispatch("decide", "{}").unwrap();
    let refused = rejected(&mut game, "decide", "{}", "limit", "history_limit");
    assert_eq!(
        refused["message"],
        "event decide, rule 2: decision series route reached its history limit 1"
    );
}

const LATE_FADE: &str = r#"
claim safe;
evidence bite from "a bite";
evidence chart from "tidal archive";
caveat faded consequence low;
renewable bite limit 4;
state from_chart = 0;
state eaten = 0 min 0 max 9;
event tick dt min 0 max 0.1;
event eat;
event regrow;
event chart;
on chart reveal chart supports safe;
on chart set from_chart = qualified(1, chart);
on eat reveal bite supports safe;
on eat set eaten = eaten + 1;
on eat qualify bite with faded after 0.15;
on regrow renew bite;
bind hud.text = "ok" because nothing;
bind hud.text = "faded" when carries(bite, faded) because from_chart;
"#;

// A citation that its value and conditions never read refuses the event,
// classified, and the session goes on. The save fuzz found it without any
// save: a caveat scheduled on eating fires on a tick, the declaration that
// cites what it never read wins for the first time, and the host lost the
// session to a clock tick.
#[test]
fn an_ungrounded_citation_refuses_the_event_and_the_session_continues() {
    let mut game = session(LATE_FADE);
    let mut legacy = session(LATE_FADE);
    for (event, payload) in [
        ("chart", "{}"),
        ("eat", "{}"),
        ("tick", r#"{"dt":0.1}"#),
        ("regrow", "{}"),
        ("eat", "{}"),
        // The caveat for the first bite, which bite@2 has replaced.
        ("tick", r#"{"dt":0.1}"#),
    ] {
        game.dispatch(event, payload).unwrap();
        legacy.dispatch(event, payload).unwrap();
    }
    // The caveat for bite@2 makes "faded" win, citing the chart.
    let tick = r#"{"dt":0.1}"#;
    let error = "binding hud.text cites evidence chart that its value and conditions never read";
    let refused = rejected(&mut game, "tick", tick, "evaluation", "ungrounded_citation");
    assert_eq!(refused["message"], error);
    assert_eq!(legacy.dispatch("tick", tick).unwrap_err(), error);
    assert_eq!(checkpoint(&legacy), checkpoint(&game));
    // Refused again: the caveat is still due, and the citation still ungrounded.
    rejected(&mut game, "tick", tick, "evaluation", "ungrounded_citation");

    let accepted = json(&game.dispatch_outcome("eat", "{}").unwrap());
    assert_eq!(accepted["outcome"], "accepted");
    assert_eq!(accepted["snapshot"]["values"]["eaten"], 3.0);
    assert_eq!(accepted["snapshot"]["bindings"]["hud"]["text"], "ok");

    let restored = WebReactiveSession::restore(LATE_FADE, &game.save().unwrap());
    let mut restored = restored.unwrap_or_else(|error| panic!("restore failed: {error}"));
    rejected(
        &mut restored,
        "tick",
        tick,
        "evaluation",
        "ungrounded_citation",
    );
    let accepted = json(&restored.dispatch_outcome("chart", "{}").unwrap());
    assert_eq!(accepted["outcome"], "accepted");
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
    for expression in [
        "qualified(1, never_seen)",
        "if(output == 1, qualified(1, never_seen), 0)",
    ] {
        let source = format!(
            r#"
            claim fatal_claim;
            evidence never_seen from "never";
            event see_never;
            on see_never reveal never_seen supports fatal_claim;
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

// dispatch_view_outcome, the view path, is dispatch_outcome with the view in
// place of the snapshot. Every refusal above also goes through it, in
// `rejected`. Accepted, it returns the view the session then shows, byte for
// byte, and the two paths leave the same session behind.
fn accepted_view_wire(view: &str) -> String {
    format!(r#"{{"schema":"{SCHEMA}","outcome":"accepted","view":{view}}}"#)
}

#[test]
fn an_accepted_view_outcome_is_the_view_after_the_same_transaction() {
    let mut outcome = session(TIMED);
    let mut viewed = session(TIMED);
    let mut core = ReactiveSession::from_source(TIMED).unwrap();
    for (event, payload) in [
        ("seed", "{}"),
        ("unblock", "{}"),
        ("advance", r#"{"dt":0.125}"#),
        ("advance", r#"{"dt":0.125}"#),
        ("advance", r#"{"dt":0.25}"#),
    ] {
        let wire = viewed.dispatch_view_outcome(event, payload).unwrap();
        assert_eq!(wire, accepted_view_wire(&viewed.view()), "{event}");
        let result = json(&outcome.dispatch_outcome(event, payload).unwrap());
        assert_eq!(result["outcome"], "accepted");
        // The same session: the same save and snapshot, to the byte.
        assert_eq!(viewed.save().unwrap(), outcome.save().unwrap(), "{event}");
        assert_eq!(viewed.snapshot(), outcome.snapshot(), "{event}");
        assert_eq!(viewed.view(), outcome.view(), "{event}");
        let native = core.dispatch_view_outcome_json(event, payload).unwrap();
        assert_eq!(serde_json::to_string(&native).unwrap(), wire);
        let value = json(&wire);
        assert!(value.get("snapshot").is_none());
        assert_eq!(value["view"]["schema"], "caveat-reactive-view/0.1");
        assert_eq!(value["view"]["sequence"], result["snapshot"]["sequence"]);
    }
    let shown = json(&viewed.view());
    assert_eq!(shown["bindings"]["hud"]["value"], 2.0);
    // mutate ran on the second and third advance.
    assert_eq!(shown["decision_series"]["route"]["current"], "route@3");
}

#[test]
fn a_fatal_view_outcome_is_the_same_report_and_changes_nothing() {
    for expression in [
        "qualified(1, never_seen)",
        "if(output == 1, qualified(1, never_seen), 0)",
    ] {
        let source = format!(
            r#"
            claim fatal_claim;
            evidence never_seen from "never";
            event see_never;
            on see_never reveal never_seen supports fatal_claim;
            state output = 0;
            event run;
            on run set output = 1;
            on run set output = {expression};
        "#
        );
        let mut game = session(&source);
        let mut viewed = session(&source);
        let mut core = ReactiveSession::from_source(&source).unwrap();
        let before = checkpoint(&viewed);
        let fatal = viewed.dispatch_view_outcome("run", "{}").unwrap_err();
        assert_eq!(fatal, game.dispatch_outcome("run", "{}").unwrap_err());
        assert_eq!(json(&fatal)["outcome"], "fatal");
        assert_eq!(json(&fatal)["code"], "unclassified");
        assert_eq!(checkpoint(&viewed), before);
        assert_eq!(
            serde_json::to_string(&core.dispatch_view_outcome_json("run", "{}").unwrap_err())
                .unwrap(),
            fatal
        );
    }
}

// The one refusal code no test above reaches through `rejected`.
#[test]
fn an_identifier_past_the_limit_is_the_same_refusal_on_the_view_path() {
    let mut game = session(
        r#"
        identifiers limit 1;
        state head = 0;
        event pushed commit id;
        on pushed set head = commit;
        bind pr.head = id_text(head);
    "#,
    );
    let accepted = game.dispatch_view_outcome("pushed", r#"{"commit":"a"}"#);
    assert_eq!(accepted.unwrap(), accepted_view_wire(&game.view()));
    rejected(
        &mut game,
        "pushed",
        r#"{"commit":"b"}"#,
        "limit",
        "identifier_limit",
    );
    rejected(
        &mut game,
        "pushed",
        r#"{"commit":1}"#,
        "input",
        "payload_invalid",
    );
    assert_eq!(json(&game.view())["bindings"]["pr"]["head"], "a");
}

#[test]
fn attention_exhaustion_is_recoverable_before_and_after_partial_work() {
    let minimal = "budget 1; caveat stale consequence low; event run; event steady; \
        on run examine stale cost 2; on steady examine stale cost 1;";
    let mut single = session(minimal);
    let result = rejected(&mut single, "run", "{}", "limit", "attention_limit");
    assert_eq!(
        result["message"],
        "event run, rule 1: insufficient examination budget"
    );
    assert_eq!(
        json(&single.dispatch_outcome("steady", "{}").unwrap())["outcome"],
        "accepted"
    );

    for refusal in [
        "on advance when blocked == 1 examine stale cost 4;",
        "proc exhaust() { examine stale cost 4; }; on advance when blocked == 1 call exhaust();",
    ] {
        let source = format!("{TIMED}\n{refusal}");
        recover_after_refusal(
            &source,
            "limit",
            "attention_limit",
            "insufficient examination budget",
            "unblock",
        );
    }
}

#[test]
fn empty_caveated_selection_is_recoverable_even_when_has_caveat_is_true() {
    for selector in [
        "on advance when has_caveat(plan_basis, phantom) reopen route because caveated(plan_basis, phantom);",
        "proc select_witness() { reopen route because caveated(plan_basis, phantom); }; on advance when has_caveat(plan_basis, phantom) call select_witness();",
    ] {
        let source = format!(r#"{TIMED}
            caveat phantom consequence low;
            state plan_basis = 0;
            event attach;
            on seed set plan_basis = qualified(1, sight, phantom);
            on attach qualify sight with phantom;
            bind hud.phantom = has_caveat(plan_basis, phantom);
            {selector}
        "#);
        recover_after_refusal(
            &source,
            "evaluation",
            "empty_caveated_selection",
            "caveated(plan_basis, phantom) selects no observed evidence carrying phantom from the state's grounds",
            "attach",
        );
    }
}

// The refused clock event has already applied a scheduled qualification, spent
// attention, sampled, revealed, revised a decision, set state and emitted a cue.
// Compare the complete public surfaces and future execution with an untouched
// control, including a session restored from the save immediately after refusal.
fn recover_after_refusal(source: &str, origin: &str, code: &str, message: &str, recovery: &str) {
    let mut game = session(source);
    let mut control = session(source);
    for run in [&mut game, &mut control] {
        run.dispatch("seed", "{}").unwrap();
    }
    let before = checkpoint(&game);
    assert_eq!(before.0["decision_journal"].as_array().unwrap().len(), 1);
    assert_eq!(before.0["cues"].as_array().unwrap().len(), 1);
    assert_eq!(
        before.0["scheduled_qualifications"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    if code == "empty_caveated_selection" {
        assert_eq!(before.0["bindings"]["hud"]["phantom"], true);
    }
    let refusal = rejected(&mut game, "advance", r#"{"dt":0.25}"#, origin, code);
    assert!(refusal["message"].as_str().unwrap().ends_with(message));
    assert_eq!(checkpoint(&game), checkpoint(&control));
    let mut restored = WebReactiveSession::restore(source, &game.save().unwrap()).unwrap();
    assert_eq!(checkpoint(&restored), before);
    rejected(&mut restored, "advance", r#"{"dt":0.25}"#, origin, code);
    for run in [&mut game, &mut control, &mut restored] {
        assert_eq!(
            json(&run.dispatch_outcome(recovery, "{}").unwrap())["outcome"],
            "accepted"
        );
        assert_eq!(
            json(&run.dispatch_outcome("advance", r#"{"dt":0.25}"#).unwrap())["outcome"],
            "accepted"
        );
    }
    assert_eq!(checkpoint(&game), checkpoint(&control));
    assert_eq!(checkpoint(&restored), checkpoint(&control));
    let after = json(&game.snapshot());
    assert_eq!(after["elapsed"], 0.25);
    assert_eq!(
        after["reading_streams"]["readings_log"]["current"],
        "readings_log@1"
    );
    assert_eq!(after["decision_series"]["route"]["current"], "route@2");
    assert_eq!(after["values"]["output"].as_f64(), Some(2.0));
    assert_eq!(after["bindings"]["hud"]["stale"], true);
    let roundtrip = WebReactiveSession::restore(source, &game.save().unwrap()).unwrap();
    assert_eq!(checkpoint(&roundtrip), checkpoint(&game));
}

#[test]
fn recovery_classification_does_not_swallow_qualifying_with_unobserved_evidence() {
    for source in [
        "claim safe; evidence never from \"never\"; event see; on see reveal never supports safe; \
         state value = 0; event run; on run set value = qualified(1, never);",
        "claim safe; evidence never from \"never\"; event see; on see reveal never supports safe; \
         state value = 0; event run; on run when qualified(1, never) > 0 set value = 1;",
    ] {
        let mut game = session(source);
        let fatal = json(&game.dispatch_outcome("run", "{}").unwrap_err());
        assert_eq!(fatal["outcome"], "fatal");
        assert_eq!(fatal["code"], "unclassified");
    }
}

// Evidence some rule could reveal, but which no dispatched event has observed,
// and a reading stream that never receives a reading.
const UNOBSERVED: &str = r#"
evidence never from "an unreported observation";
readings empty_log from sensor limit 2;
event see;
on see reveal never supports safe;
"#;

// A decision series that a later event could commit, but none has.
const EMPTY_SERIES: &str = r#"
decisions unrouted limit 2;
event settle;
on settle commit unrouted because enough using output;
"#;

// Version Lab F154, F78, F158: acting on evidence no event has observed.
#[test]
fn acting_on_unobserved_evidence_is_recoverable() {
    for (refusal, message) in [
        (
            "on advance when blocked == 1 qualify never with stale;",
            "cannot qualify unobserved evidence never",
        ),
        (
            "on advance when blocked == 1 qualify never with stale after 1;",
            "cannot qualify unobserved evidence never",
        ),
        (
            "on advance when blocked == 1 withdraw never because sight;",
            "cannot withdraw unobserved evidence never",
        ),
        (
            "on advance when blocked == 1 withdraw sight because never;",
            "cannot withdraw because unobserved evidence never",
        ),
        (
            "on advance when blocked == 1 withdraw latest(empty_log) because sight;",
            "cannot withdraw Latest(\"empty_log\"): its stream has no reading",
        ),
        (
            "on advance when blocked == 1 reopen route because never;",
            "cannot reopen route because unobserved evidence never",
        ),
        (
            "proc hide() { reopen route because never; }; on advance when blocked == 1 call hide();",
            "cannot reopen route because unobserved evidence never",
        ),
        (
            "on advance when blocked == 1 reopen route because latest(empty_log);",
            "cannot reopen route because latest(empty_log): its stream has no reading",
        ),
    ] {
        let source = format!("{TIMED}\n{UNOBSERVED}\n{refusal}");
        recover_after_refusal(
            &source,
            "evaluation",
            "unobserved_evidence",
            message,
            "unblock",
        );
    }
}

// Version Lab F158: reopening a decision series that has no commitment.
#[test]
fn reopening_an_uncommitted_decision_is_recoverable() {
    for refusal in [
        "on advance when blocked == 1 reopen spare because sight;",
        "proc retry() { reopen spare because sight; }; on advance when blocked == 1 call retry();",
    ] {
        let source = format!("{TIMED}\ndecisions spare limit 2;\n{refusal}");
        recover_after_refusal(
            &source,
            "evaluation",
            "not_committed",
            "cannot reopen uncommitted action spare",
            "unblock",
        );
    }
}

// Version Lab F104, F212: division by zero, nonfinite results, history indexes.
#[test]
fn expression_failures_in_rules_and_bindings_are_recoverable() {
    for (refusal, message) in [
        (
            "on advance when blocked == 1 set output = 1 / (blocked - 1);",
            "division by zero in expression",
        ),
        (
            "on advance when 1 / (blocked - 1) > 0 set output = 3;",
            "division by zero in expression",
        ),
        (
            "on advance when blocked == 1 set output = 1e300 * 1e300 * blocked;",
            "expression produced a non-finite number",
        ),
        (
            "on advance when blocked == 1 set output = history_at(readings_log, 3);",
            "history index 3 is out of range for readings_log",
        ),
        (
            "on advance when blocked == 1 set output = history_at(readings_log, blocked / 2);",
            "history index must be an integer in 0..256",
        ),
        (
            "on advance when blocked == 1 set output = history_at(route, 4);",
            "history index 4 is out of range for route",
        ),
        (
            "proc divide() { set output = 1 / (blocked - 1); }; on advance when blocked == 1 call divide();",
            "division by zero in expression",
        ),
        (
            "bind hud.ratio = if(elapsed() >= 0.25 and blocked == 1, 1 / (blocked - 1), 0);",
            "division by zero in expression",
        ),
        (
            "bind hud.huge = if(elapsed() >= 0.25 and blocked == 1, 1e300 * 1e300, 0);",
            "expression produced a non-finite number",
        ),
        (
            "bind hud.indexed = if(elapsed() >= 0.25 and blocked == 1, history_at(readings_log, 3), 0);",
            "history index 3 is out of range for readings_log",
        ),
        (
            "on advance when blocked == 1 set output = sqrt(0 - blocked);",
            "sqrt requires a nonnegative number",
        ),
        (
            "bind hud.root = if(elapsed() >= 0.25 and blocked == 1, sqrt(0 - blocked), 0);",
            "sqrt requires a nonnegative number",
        ),
        (
            "on advance when blocked == 1 set output = latest(empty_log);",
            "reading stream empty_log has no reached sample",
        ),
        (
            "on advance when blocked == 1 set output = latest(unrouted);",
            "decision series unrouted has no commitment",
        ),
    ] {
        let source = format!("{TIMED}\n{UNOBSERVED}\n{EMPTY_SERIES}\n{refusal}");
        recover_after_refusal(&source, "evaluation", "expression", message, "unblock");
    }
}

#[test]
fn a_false_requirement_is_recoverable() {
    for refusal in [
        "on advance when blocked == 1 set output = require(blocked == 0, 1);",
        "on advance when require(blocked == 0, false) set output = 3;",
        "on advance when blocked == 1 set output = clamp(1, blocked, 0);",
        "bind hud.required = if(elapsed() >= 0.25, require(blocked == 0, 1), 0);",
    ] {
        let source = format!("{TIMED}\n{refusal}");
        recover_after_refusal(
            &source,
            "evaluation",
            "requirement_failed",
            "source expression requirement failed",
            "unblock",
        );
    }
}

// Version Lab F184: the 4,097th pending `qualify … after`.
#[test]
fn a_full_scheduled_qualification_table_refuses_the_event_and_the_session_continues() {
    // Each `load` schedules 64, so 64 of them fill the table of 4,096.
    let source = format!(
        r#"
        claim safe;
        evidence sight from "a sighting";
        caveat stale consequence low;
        state kept = 0;
        event seed;
        event load;
        event fail;
        event resume;
        event advance dt min 0 max 1;
        clock advance every 0.125;
        on seed reveal sight supports safe;
        on fail set kept = 99;
        on fail qualify sight with stale after 100;
        on resume set kept = 2;
        bind hud.kept = kept;
        {}"#,
        "on load qualify sight with stale after 100;\n".repeat(64)
    );
    let mut game = session(&source);
    game.dispatch("seed", "{}").unwrap();
    for _ in 0..64 {
        game.dispatch("load", "{}").unwrap();
    }
    assert_eq!(
        json(&game.snapshot())["scheduled_qualifications"]
            .as_array()
            .unwrap()
            .len(),
        4096
    );
    let before = checkpoint(&game);
    let refusal = rejected(&mut game, "fail", "{}", "limit", "scheduled_limit");
    assert_eq!(
        refusal["message"],
        "event fail, rule 3: scheduled qualifications exceed limit 4096"
    );
    let mut restored = WebReactiveSession::restore(&source, &game.save().unwrap()).unwrap();
    assert_eq!(checkpoint(&restored), before);
    rejected(&mut restored, "fail", "{}", "limit", "scheduled_limit");
    for run in [&mut game, &mut restored] {
        assert_eq!(
            json(&run.dispatch_outcome("resume", "{}").unwrap())["outcome"],
            "accepted"
        );
        assert_eq!(
            json(&run.snapshot())["bindings"]["hud"]["kept"].as_f64(),
            Some(2.0)
        );
    }
    assert_eq!(checkpoint(&restored), checkpoint(&game));
}
