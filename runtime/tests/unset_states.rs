//! spec/caveat-unset-states-0.1.md: a state declared `unset` has no value
//! until a rule sets it, and reading it before then refuses the event as
//! `evaluation/unset_state`. Found by the third agent test (gap 6): an
//! inherited program rated a bridge from its rated parts while a part had no
//! report, because UNRATED was written as 1000 and `min` hid it.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

const BRIDGE: &str = "\
state deck unset min 0 max 200;
state pier unset min 0 max 200;
state bridge unset;
state verdict = 0;
event report_deck capacity min 0 max 200;
event report_pier capacity min 0 max 200;
event withdraw_pier;
event rate;
on report_deck when is_set(deck) set deck = min(deck, capacity);
on report_deck when not is_set(deck) set deck = capacity;
on report_pier when is_set(pier) set pier = min(pier, capacity);
on report_pier when not is_set(pier) set pier = capacity;
on withdraw_pier unset pier;
on rate when is_set(deck) and is_set(pier) set bridge = min(deck, pier);
on rate when not (is_set(deck) and is_set(pier)) unset bridge;
on rate when is_set(bridge) set verdict = 1;
on rate when not is_set(bridge) set verdict = 2;
";

fn send(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event}: fatal {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

fn snapshot(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.snapshot()).unwrap()
}

#[test]
fn an_unset_state_starts_unset_and_is_left_out_of_values() {
    let session = ReactiveSession::from_source(BRIDGE).unwrap();
    let snapshot = snapshot(&session);
    assert_eq!(snapshot["unset"], json!(["bridge", "deck", "pier"]));
    assert_eq!(snapshot["values"], json!({"verdict": 0.0}));
    assert!(snapshot["qualified_values"].get("deck").is_none());
    assert!(snapshot["value_grounds"].get("deck").is_none());
}

#[test]
fn set_gives_a_value_and_unset_takes_it_back() {
    let mut session = ReactiveSession::from_source(BRIDGE).unwrap();
    assert_eq!(
        send(&mut session, "report_deck", r#"{"capacity":50}"#)["outcome"],
        "accepted"
    );
    assert_eq!(
        send(&mut session, "report_deck", r#"{"capacity":42}"#)["outcome"],
        "accepted"
    );
    assert_eq!(send(&mut session, "rate", "{}")["outcome"], "accepted");
    let after = snapshot(&session);
    assert_eq!(after["values"]["deck"], 42.0);
    assert_eq!(
        after["values"]["verdict"], 2.0,
        "a part has no report: UNRATED"
    );
    assert_eq!(after["unset"], json!(["bridge", "pier"]));
    send(&mut session, "report_pier", r#"{"capacity":46}"#);
    send(&mut session, "rate", "{}");
    let after = snapshot(&session);
    assert_eq!(after["values"]["bridge"], 42.0);
    assert_eq!(after["values"]["verdict"], 1.0);
    assert!(after.get("unset").is_none(), "left out when empty: {after}");
    send(&mut session, "withdraw_pier", "{}");
    send(&mut session, "rate", "{}");
    let after = snapshot(&session);
    assert_eq!(after["unset"], json!(["bridge", "pier"]));
    assert_eq!(after["values"]["verdict"], 2.0);
}

#[test]
fn bounds_hold_for_a_set() {
    let mut session = ReactiveSession::from_source(
        "state x unset min 0 max 10;\nevent go v min -100 max 100;\non go set x = v;\n",
    )
    .unwrap();
    let outcome = send(&mut session, "go", r#"{"v":11}"#);
    assert_eq!(outcome["code"], "bound_exceeded", "{outcome}");
    assert_eq!(snapshot(&session)["unset"], json!(["x"]));
}

#[test]
fn the_inherited_wrong_rule_refuses_its_first_rate() {
    let source = "\
state deck unset;
state pier unset;
state bridge = 0;
event report_deck capacity min 0 max 200;
event rate;
on report_deck set deck = capacity;
on rate set bridge = min(deck, pier);
";
    let mut session = ReactiveSession::from_source(source).unwrap();
    send(&mut session, "report_deck", r#"{"capacity":42}"#);
    let before = snapshot(&session);
    let outcome = send(&mut session, "rate", "{}");
    assert_eq!(outcome["outcome"], "rejected", "{outcome}");
    assert_eq!(outcome["origin"], "evaluation");
    assert_eq!(outcome["code"], "unset_state");
    let message = outcome["message"].as_str().unwrap();
    assert!(message.contains("pier"), "{message}");
    assert!(message.contains("rule 2"), "names the rule: {message}");
    assert_eq!(snapshot(&session), before, "a refusal changes nothing");
}

#[test]
fn every_kind_of_read_refuses_and_is_set_never_does() {
    for (rule, refused) in [
        ("on go when x > 1 set n = 1;", true),
        ("on go set n = x;", true),
        ("on go set n = 1 because x;", true),
        ("on go set n = f(x);", true),
        ("on go commit d because enough using x;", true),
        ("on go sample s = x;", true),
        ("on go when is_set(x) and x > 1 set n = 1;", false),
        ("on go when not is_set(x) or x > 1 set n = 2;", false),
        ("on go set n = if(is_set(x), x, 3);", false),
    ] {
        let source = format!(
            "fn f(a) = 7;\nevidence log from \"a log\";\nreadings s from log limit 3;\nstate x unset;\nstate n = 0;\nevent go;\n{rule}\n"
        );
        let mut session =
            ReactiveSession::from_source(&source).unwrap_or_else(|error| panic!("{rule}: {error}"));
        let outcome = send(&mut session, "go", "{}");
        if refused {
            assert_eq!(outcome["code"], "unset_state", "{rule}: {outcome}");
        } else {
            assert_eq!(outcome["outcome"], "accepted", "{rule}: {outcome}");
        }
    }
}

#[test]
fn unset_and_is_set_name_only_a_state_declared_unset() {
    for (source, message) in [
        (
            "state x = 0;\nevent go;\non go unset x;\n",
            "x always has a value; declare it `state x unset;`",
        ),
        (
            "state x = 0;\nstate n = 0;\nevent go;\non go when is_set(x) set n = 1;\n",
            "x always has a value",
        ),
        ("event go;\non go unset y;\n", "undeclared state y"),
        (
            "state x unset;\nstate y = x;\n",
            "initializer runs before any rule",
        ),
        (
            "state x unset min 1;\n",
            "state expects NAME = EXPRESSION or NAME unset",
        ),
    ] {
        let Err(error) = ReactiveSession::from_source(source) else {
            panic!("{source} loaded");
        };
        assert!(error.contains(message), "{source}: {error}");
    }
}

#[test]
fn a_program_that_defines_fn_is_set_keeps_its_function() {
    let source = "fn is_set(a) = a + 1;\nstate n = 0;\nevent go;\non go set n = is_set(4);\n";
    let mut session = ReactiveSession::from_source(source).unwrap();
    send(&mut session, "go", "{}");
    assert_eq!(snapshot(&session)["values"]["n"], 5.0);
}

#[test]
fn grounds_and_lineage_of_is_set_and_unset() {
    let source = "\
evidence log from \"a log\";
claim c;
log supports c;
state x unset;
state n = 0;
event see;
event set_x;
event clear_x;
event check;
on see reveal log;
on set_x set x = qualified(4, log);
on clear_x unset x;
on check when is_set(x) set n = 1;
";
    let mut session = ReactiveSession::from_source(source).unwrap();
    send(&mut session, "see", "{}");
    send(&mut session, "set_x", "{}");
    let after = snapshot(&session);
    assert_eq!(after["value_grounds"]["x"]["evidence"], json!(["log"]));
    send(&mut session, "check", "{}");
    let after = snapshot(&session);
    assert_eq!(
        after["qualified_values"]["n"]["provenance"]["evidence"],
        json!(["log"]),
        "is_set reads the state's lineage: {after}"
    );
    send(&mut session, "clear_x", "{}");
    let after = snapshot(&session);
    assert_eq!(after["unset"], json!(["x"]));
    assert!(after["value_grounds"].get("x").is_none());
}

#[test]
fn save_and_restore_round_trip() {
    let mut session = ReactiveSession::from_source(BRIDGE).unwrap();
    let fresh = serde_json::to_value(session.save().unwrap()).unwrap();
    assert!(fresh["states"].as_object().unwrap().is_empty(), "{fresh}");
    send(&mut session, "report_deck", r#"{"capacity":42}"#);
    send(&mut session, "report_pier", r#"{"capacity":46}"#);
    send(&mut session, "withdraw_pier", "{}");
    let save = session.save().unwrap();
    let saved = serde_json::to_value(&save).unwrap();
    assert!(saved["states"].get("deck").is_some());
    assert!(
        saved["states"].get("pier").is_none(),
        "unset again is as loaded: {saved}"
    );
    let mut restored = ReactiveSession::restore(BRIDGE, &save).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&session));
    for (event, payload) in [
        ("rate", "{}"),
        ("report_pier", r#"{"capacity":9}"#),
        ("rate", "{}"),
    ] {
        assert_eq!(
            send(&mut restored, event, payload),
            send(&mut session, event, payload)
        );
    }
}

#[test]
fn programs_without_unset_states_write_no_unset_field() {
    let mut session =
        ReactiveSession::from_source("state n = 0;\nevent go;\non go set n = n + 1;\n").unwrap();
    send(&mut session, "go", "{}");
    let text = serde_json::to_string(&session.snapshot()).unwrap();
    assert!(!text.contains("\"unset\""), "{text}");
}

#[test]
fn check_c013_names_unguarded_reads_only() {
    use caveat_runtime::reactive::check_source;
    let source = "\
state deck unset;
state pier unset;
state bridge = 0;
event rate;
on rate set bridge = min(deck, pier);
on rate when is_set(deck) and is_set(pier) set bridge = min(deck, pier);
on rate when is_set(deck) set bridge = if(is_set(pier), min(deck, pier), deck);
on rate when not is_set(deck) or deck > 3 set bridge = 1;
";
    let report = serde_json::to_value(check_source(source).unwrap()).unwrap();
    let found = report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|diagnostic| diagnostic["code"] == "C013")
        .map(|diagnostic| {
            (
                diagnostic["line"].as_u64().unwrap(),
                diagnostic["message"].as_str().unwrap().to_string(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found.iter().all(|(line, _)| *line == 5), "{found:?}");
    assert!(
        found[0].1.contains("`deck`") && found[1].1.contains("`pier`"),
        "{found:?}"
    );
    let plain = serde_json::to_value(
        check_source("state n = 0;\nevent go;\non go set n = n + 1;\n").unwrap(),
    )
    .unwrap();
    assert!(!plain.to_string().contains("C013"));
}
