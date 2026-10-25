//! Agents: who sent each event. See spec/caveat-agents-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

const HANDOFF: &str = r#"
claim plan_ok; claim done; claim wrong;
evidence request from "the user's request";
evidence report from "the sub-agent's report";
evidence recheck from "the main agent's check of the report";
readings reports from report limit 8;
decisions plan limit 4;
agent main sends delegate, main_return;
agent sub sends sub_report;
agent reviewer;
event delegate;
event sub_report n min 0 max 10;
event main_return;
event note;
on delegate reveal request supports plan_ok;
on delegate commit plan because enough citing qualified(1, request);
on sub_report sample reports = n supports done;
on main_return when not observed(recheck) reveal recheck opposes done;
on main_return withdraw latest(reports) because recheck;
on main_return reopen plan because recheck;
"#;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn send(game: &mut ReactiveSession, event: &str, payload: Value, agent: Option<&str>) -> Value {
    serde_json::to_value(
        game.dispatch_outcome_as_json(event, &payload.to_string(), agent)
            .unwrap_or_else(|fatal| panic!("{event}: fatal {}", fatal.message)),
    )
    .unwrap()
}

fn refused(outcome: &Value) -> (&str, &str) {
    assert_eq!(outcome["outcome"], "rejected", "{outcome}");
    (
        outcome["origin"].as_str().unwrap(),
        outcome["code"].as_str().unwrap(),
    )
}

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

#[test]
fn an_event_some_agent_sends_takes_only_those_agents() {
    let mut game = session(HANDOFF);
    let before = snapshot(&game);
    for agent in [None, Some("sub"), Some("reviewer"), Some("mallory")] {
        let outcome = send(&mut game, "delegate", json!({}), agent);
        assert_eq!(refused(&outcome), ("input", "wrong_agent"), "{agent:?}");
        assert_eq!(snapshot(&game), before, "the refusal changes nothing");
    }
    assert_eq!(
        send(&mut game, "delegate", json!({}), Some("main"))["outcome"],
        "accepted"
    );
}

#[test]
fn an_event_no_agent_lists_is_open_to_any_agent_or_none() {
    let mut game = session(HANDOFF);
    for agent in [None, Some("main"), Some("sub"), Some("reviewer")] {
        assert_eq!(
            send(&mut game, "note", json!({}), agent)["outcome"],
            "accepted",
            "{agent:?}"
        );
    }
    let outcome = send(&mut game, "note", json!({}), Some("mallory"));
    assert_eq!(
        refused(&outcome),
        ("input", "wrong_agent"),
        "an agent must be declared"
    );
}

#[test]
fn an_unknown_event_and_a_malformed_payload_keep_their_codes() {
    let mut game = session(HANDOFF);
    let outcome = send(&mut game, "nothing", json!({}), Some("mallory"));
    assert_eq!(refused(&outcome), ("input", "unknown_event"));
    let outcome = send(&mut game, "sub_report", json!({"n": "x"}), Some("main"));
    assert_eq!(refused(&outcome), ("input", "payload_invalid"));
    // The agent is checked before the parameters.
    let outcome = send(&mut game, "sub_report", json!({"n": 11}), Some("main"));
    assert_eq!(refused(&outcome), ("input", "wrong_agent"));
    let outcome = send(&mut game, "sub_report", json!({"n": 11}), Some("sub"));
    assert_eq!(refused(&outcome), ("input", "bound_exceeded"));
}

#[test]
fn a_program_without_agents_refuses_a_named_agent_and_records_none() {
    let source =
        "claim c; evidence e from \"e\"; event go; on go when not observed(e) reveal e supports c;";
    let mut game = session(source);
    let outcome = send(&mut game, "go", json!({}), Some("main"));
    assert_eq!(refused(&outcome), ("input", "wrong_agent"));
    let accepted = send(&mut game, "go", json!({}), None);
    assert_eq!(accepted["outcome"], "accepted");
    assert!(accepted["snapshot"].get("last_agent").is_none());
    let save: Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
    assert!(save.get("last_agent").is_none());
}

#[test]
fn the_agent_is_recorded_on_what_its_event_did() {
    let mut game = session(HANDOFF);
    send(&mut game, "delegate", json!({}), Some("main"));
    send(&mut game, "sub_report", json!({"n": 3}), Some("sub"));
    let state = snapshot(&game);
    assert_eq!(state["last_agent"], "sub");
    let journal = state["decision_journal"].as_array().unwrap();
    assert_eq!(journal[0]["change"], "committed");
    assert_eq!(journal[0]["agent"], "main");
    let reading = &state["reading_streams"]["reports"]["occurrences"][0];
    assert_eq!(reading["agent"], "sub");
    send(&mut game, "main_return", json!({}), Some("main"));
    let state = snapshot(&game);
    assert_eq!(state["withdrawals"][0]["agent"], "main");
    let journal = state["decision_journal"].as_array().unwrap();
    assert_eq!(journal.last().unwrap()["change"], "reopened");
    assert_eq!(journal.last().unwrap()["agent"], "main");
    // An event sent without an agent records none.
    send(&mut game, "note", json!({}), None);
    assert!(snapshot(&game).get("last_agent").is_none());
}

#[test]
fn a_save_keeps_the_agents_and_restore_checks_them() {
    let mut game = session(HANDOFF);
    send(&mut game, "delegate", json!({}), Some("main"));
    send(&mut game, "sub_report", json!({"n": 3}), Some("sub"));
    let saved = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(HANDOFF, &saved).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&game));
    // A compact save holds no agents, so it is the state save.
    let compact: Value = serde_json::from_str(&game.save_compact_json().unwrap()).unwrap();
    assert_eq!(compact["schema"], "caveat-reactive-save/0.1");

    let edit = |change: &dyn Fn(&mut Value)| {
        let mut save: Value = serde_json::from_str(&saved).unwrap();
        change(&mut save);
        ReactiveSession::restore_json(HANDOFF, &save.to_string()).map(|_| ())
    };
    let undeclared = edit(&|save| save["decision_journal"][0]["agent"] = json!("mallory"));
    assert!(undeclared.unwrap_err().contains("undeclared agent mallory"));
    let not_a_sender = edit(&|save| save["decision_journal"][0]["agent"] = json!("sub"));
    assert!(not_a_sender
        .unwrap_err()
        .contains("not recorded with an agent that sends it"));
    let missing = edit(&|save| {
        save["decision_journal"][0]
            .as_object_mut()
            .unwrap()
            .remove("agent");
    });
    assert!(missing
        .unwrap_err()
        .contains("not recorded with an agent that sends it"));
    let last = edit(&|save| save["last_agent"] = json!("main"));
    assert!(last.unwrap_err().contains("agent"));
}

#[test]
fn loading_checks_the_declarations() {
    let base = "event go; event stop;";
    for (declarations, message) in [
        ("agent a; agent a;", "duplicate agent a"),
        (
            "agent a sends nowhere;",
            "agent a sends undeclared event nowhere",
        ),
        ("agent a sends go, go;", "agent a lists event go twice"),
        ("agent a sends;", "agent expects NAME [sends EVENT, …]"),
        (
            "agent a sends go stop;",
            "sends takes event names separated by commas",
        ),
    ] {
        let error = ReactiveSession::from_source(&format!("{base} {declarations}"))
            .err()
            .unwrap_or_else(|| panic!("{declarations} loaded"));
        assert!(error.contains(message), "{declarations}: {error}");
    }
    // Two agents may send one event.
    let mut game = session("event go; agent a sends go; agent b sends go;");
    assert_eq!(
        send(&mut game, "go", json!({}), Some("b"))["outcome"],
        "accepted"
    );
}

#[test]
fn the_interface_lists_the_agents() {
    let interface: Value =
        serde_json::to_value(caveat_runtime::reactive::interface_source(HANDOFF).unwrap()).unwrap();
    assert_eq!(
        interface["agents"],
        json!([
            {"name": "main", "sends": ["delegate", "main_return"]},
            {"name": "reviewer", "sends": []},
            {"name": "sub", "sends": ["sub_report"]},
        ])
    );
    let plain: Value =
        serde_json::to_value(caveat_runtime::reactive::interface_source("event go;").unwrap())
            .unwrap();
    assert!(plain.get("agents").is_none());
}
