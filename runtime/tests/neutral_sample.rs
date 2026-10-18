//! `sample STREAM = EXPRESSION` without a stance: a reading that bears on no
//! claim. See spec/caveat-neutral-sample-0.1.md and issue #218.
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::web::WebReactiveSession;
use serde_json::{json, Value};

/// #218's inventory: a version number is a reading, not support.
const INVENTORY: &str = r#"
claim upgrade_safe;
evidence node_version from "node --version on the build machine";
evidence ci from "the CI run";
caveat local_only consequence low;
local_only qualifies node_version;
readings node from node_version limit 3;
readings checks from ci limit 4;
decisions upgrade limit 4 reopened by node;
decisions ship limit 4 reopened by checks supporting upgrade_safe;
state major = 0;
event inventory v min 0 max 99;
event check v min 0 max 1;
event decide;
event release;
on inventory sample node = v;
on check sample checks = v;
on decide when not committed(upgrade) or reopened(upgrade)
  commit upgrade because enough using latest(node);
on release when not committed(ship) or reopened(ship)
  commit ship because enough citing latest(checks);
on inventory set major = latest(node);
"#;

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn run(game: &mut ReactiveSession, events: &[(&str, &str)]) {
    for (event, payload) in events {
        game.dispatch_json(event, payload)
            .unwrap_or_else(|error| panic!("{event} {payload}: {error}"));
    }
}

#[test]
fn issue_218_a_reading_needs_no_stance() {
    let mut game = ReactiveSession::from_source(INVENTORY).unwrap();
    run(&mut game, &[("inventory", r#"{"v": 22}"#)]);
    let shown = snapshot(&game);
    let reading = &shown["reading_streams"]["node"]["occurrences"][0];
    assert_eq!(reading["id"], "node@1");
    assert_eq!(reading["value"], 22.0);
    assert!(reading.get("relation").is_none(), "{reading}");
    assert!(reading.get("claim").is_none(), "{reading}");
    assert_eq!(shown["observations"], json!(["node@1"]));
    // No stance relation: the reading's only relation is its inherited caveat.
    let relations = shown["relations"].as_array().unwrap();
    assert!(
        relations
            .iter()
            .filter(|relation| relation["from"] == "node@1" || relation["to"] == "node@1")
            .all(|relation| relation["relation"] == "qualifies"),
        "{relations:?}"
    );
    // A value read from it carries it and its caveat, as a stance reading's would.
    assert_eq!(
        shown["value_grounds"]["major"],
        json!({"evidence": ["node@1"], "caveats": ["local_only"]})
    );
}

#[test]
fn a_neutral_sample_reports_no_relation_or_target() {
    let mut session = WebReactiveSession::new(INVENTORY).unwrap();
    let outcome = json(
        &session
            .dispatch_outcome("inventory", r#"{"v": 22}"#)
            .unwrap(),
    );
    assert_eq!(outcome["outcome"], "accepted", "{outcome}");
    let mut game = ReactiveSession::from_source(INVENTORY).unwrap();
    run(&mut game, &[("inventory", r#"{"v": 22}"#)]);
    assert_eq!(
        snapshot(&game)["effects"],
        json!([{"kind": "sample", "stream": "node", "id": "node@1", "value": 22.0}])
    );
}

#[test]
fn only_an_unfiltered_trigger_fires_on_a_neutral_reading() {
    let mut game = ReactiveSession::from_source(INVENTORY).unwrap();
    run(
        &mut game,
        &[
            ("inventory", r#"{"v": 22}"#),
            ("decide", "{}"),
            ("check", r#"{"v": 1}"#),
            ("release", "{}"),
            ("inventory", r#"{"v": 24}"#),
            ("check", r#"{"v": 1}"#),
        ],
    );
    let shown = snapshot(&game);
    let changes = shown["decision_journal"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| (entry["decision"].clone(), entry["change"].clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        changes,
        [
            (json!("upgrade"), json!("committed")),
            (json!("ship"), json!("committed")),
            (json!("upgrade"), json!("reopened")),
        ],
        "{shown}"
    );
}

#[test]
fn neutral_readings_save_and_restore_and_restore_checks_them() {
    let mut game = ReactiveSession::from_source(INVENTORY).unwrap();
    run(
        &mut game,
        &[
            ("inventory", r#"{"v": 22}"#),
            ("decide", "{}"),
            ("inventory", r#"{"v": 24}"#),
        ],
    );
    let saved = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(INVENTORY, &saved).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&game));
    assert_eq!(restored.save_json().unwrap(), saved);

    let refused = |edit: &dyn Fn(&mut Value)| {
        let mut save = json(&saved);
        edit(&mut save);
        match ReactiveSession::restore_json(INVENTORY, &save.to_string()) {
            Ok(_) => panic!("a tampered save restored"),
            Err(error) => error,
        }
    };
    let only_relation = refused(&|save| {
        save["reading_streams"]["node"]["occurrences"][0]["relation"] = json!("supports");
    });
    assert!(only_relation.contains("together"), "{only_relation}");
    let unobserved = refused(&|save| {
        let observations = save["observations"].as_array_mut().unwrap();
        observations.retain(|name| name != "node@1");
    });
    assert!(unobserved.contains("node@1"), "{unobserved}");
    let stanced = refused(&|save| {
        save["graph"]["relations"]
            .as_array_mut()
            .unwrap()
            .push(json!(["node@1", "supports", "upgrade_safe"]));
    });
    assert!(stanced.contains("node@1"), "{stanced}");
}

#[test]
fn a_neutral_reading_of_a_stream_only_stance_samples_write_is_refused() {
    let source = INVENTORY.replace(
        "on check sample checks = v;",
        "on check sample checks = v supports upgrade_safe;",
    );
    let mut game = ReactiveSession::from_source(&source).unwrap();
    run(
        &mut game,
        &[("check", r#"{"v": 1}"#), ("inventory", r#"{"v": 22}"#)],
    );
    let saved = json(&game.save_json().unwrap());
    assert_eq!(saved["observations"], json!(["checks@1", "node@1"]));
    // Made to look neutral: no stance fields and no stance relation. No rule
    // of this program takes a checks reading without a stance.
    let mut save = saved.clone();
    let reading = save["reading_streams"]["checks"]["occurrences"][0]
        .as_object_mut()
        .unwrap();
    reading.remove("relation");
    reading.remove("claim");
    save["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .retain(|relation| relation[0] != "checks@1");
    let error = ReactiveSession::restore_json(&source, &save.to_string())
        .err()
        .unwrap();
    assert!(
        error.contains("checks@1") && error.contains("neutral"),
        "{error}"
    );
    // A neutral reading given a claim alone is refused too.
    let mut save = saved;
    save["reading_streams"]["node"]["occurrences"][0]["claim"] = json!("upgrade_safe");
    let error = ReactiveSession::restore_json(&source, &save.to_string())
        .err()
        .unwrap();
    assert!(error.contains("together"), "{error}");
}

#[test]
fn a_stance_sample_keeps_its_record_byte_for_byte() {
    let source = INVENTORY.replace(
        "on inventory sample node = v;",
        "on inventory sample node = v supports upgrade_safe;",
    );
    let mut game = ReactiveSession::from_source(&source).unwrap();
    run(&mut game, &[("inventory", r#"{"v": 22}"#)]);
    let shown = snapshot(&game);
    let reading = &shown["reading_streams"]["node"]["occurrences"][0];
    assert_eq!(reading["relation"], "supports");
    assert_eq!(reading["claim"], "upgrade_safe");
    assert!(shown.get("observations").is_none(), "{shown}");
}

#[test]
fn load_rules() {
    for (line, expected) in [
        ("on decide sample node = 1 supports;", "sample expects"),
        (
            "on decide sample node = supports upgrade_safe;",
            "sample expects",
        ),
        (
            "on decide sample node = 1 supports upgrade_safe + 1;",
            "sample expects",
        ),
        ("on decide sample node =;", "sample expects"),
        ("on decide sample node;", "sample expects"),
        ("on decide sample node = 1 opposes nowhere;", "nowhere"),
        ("on decide sample node = \"text\";", "numeric"),
    ] {
        let error = ReactiveSession::from_source(&format!("{INVENTORY}\n{line}"))
            .err()
            .unwrap_or_else(|| panic!("{line} should not load"));
        assert!(error.contains(expected), "{line}: {error}");
    }
}

#[test]
fn a_procedure_takes_a_neutral_reading() {
    let source = r#"
evidence probe from "a probe";
readings r from probe limit 2;
event go v min 0 max 9;
proc take(s readings, x) {
    sample s = x;
};
on go call take(r, v);
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    run(&mut game, &[("go", r#"{"v": 4}"#)]);
    let shown = snapshot(&game);
    assert_eq!(shown["observations"], json!(["r@1"]));
    let saved = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(source, &saved).unwrap();
    assert_eq!(restored.save_json().unwrap(), saved);
}
