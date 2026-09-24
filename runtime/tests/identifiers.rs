//! Identifiers learned at run time. See spec/caveat-identifiers-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};
use std::collections::BTreeMap;

const LEDGER: &str = r#"
identifiers limit 8;
claim ready;
evidence ci from "the pull request's checks";
caveat flaky consequence low;
flaky qualifies ci;
readings checks from ci limit 8;
state head = 0;
state checked = 0;
event pushed commit id;
event checks commit id, result in failed passed;
event pair first id, second id;
on pushed set head = commit;
on checks when commit != head reject "Checks for a commit that is not the head.";
on checks when result == result.passed sample checks = commit supports ready;
on checks when result == result.failed sample checks = commit opposes ready;
on checks set checked = latest(checks);
bind pr.head = id_text(head);
bind pr.checked = id_text(checked);
"#;

// Two full SHAs that share their first eight hex digits.
const A: &str = "602bdbec0a047a5319f53e83f336b9f7aec0e5ed";
const B: &str = "602bdbec11111111111111111111111111111111";

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn send(game: &mut ReactiveSession, event: &str, payload: Value) -> Value {
    serde_json::to_value(
        game.dispatch_outcome_json(event, &payload.to_string())
            .unwrap_or_else(|fatal| panic!("{event} {payload}: fatal {}", fatal.message)),
    )
    .unwrap()
}

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn refused(
    game: &mut ReactiveSession,
    event: &str,
    payload: Value,
    origin: &str,
    code: &str,
) -> Value {
    let before = (snapshot(game), game.save_json().unwrap());
    let outcome = send(game, event, payload.clone());
    assert_eq!(
        outcome["outcome"], "rejected",
        "{event} {payload}: {outcome}"
    );
    assert_eq!(
        (outcome["origin"].as_str(), outcome["code"].as_str()),
        (Some(origin), Some(code)),
        "{outcome}"
    );
    assert_eq!(
        (snapshot(game), game.save_json().unwrap()),
        before,
        "a refused event changed the session"
    );
    outcome
}

#[test]
fn texts_become_handles_in_the_order_the_session_first_receives_them() {
    let mut game = session(LEDGER);
    send(&mut game, "pushed", json!({ "commit": A }));
    send(
        &mut game,
        "checks",
        json!({ "commit": A, "result": "passed" }),
    );
    send(&mut game, "pushed", json!({ "commit": B }));
    let shot = snapshot(&game);
    assert_eq!(shot["identifiers"], json!([A, B]));
    assert_eq!(shot["values"]["head"], 2.0);
    assert_eq!(shot["bindings"]["pr"]["head"], B);
    assert_eq!(shot["bindings"]["pr"]["checked"], A);
    // Same eight-digit prefix, still two identifiers.
    refused(
        &mut game,
        "checks",
        json!({ "commit": A, "result": "passed" }),
        "policy",
        "reject",
    );
    let signature = shot["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["name"] == "pushed")
        .unwrap();
    assert_eq!(
        signature["parameters"][0]["domain"],
        json!({ "identifier": { "limit": 8 } })
    );
}

#[test]
fn only_identical_text_is_the_same_identifier() {
    let mut game = session(LEDGER);
    for text in ["abc", "ABC", "abc ", " abc", "a/b", "a//b", "abc"] {
        send(&mut game, "pushed", json!({ "commit": text }));
    }
    assert_eq!(
        snapshot(&game)["identifiers"],
        json!(["abc", "ABC", "abc ", " abc", "a/b", "a//b"])
    );
    assert_eq!(snapshot(&game)["values"]["head"], 1.0);
}

#[test]
fn new_identifiers_in_one_event_are_numbered_in_declaration_order() {
    let mut game = session(LEDGER);
    // The payload lists `second` first; the event declares `first` first.
    send(&mut game, "pair", json!({ "second": "y", "first": "x" }));
    send(&mut game, "pair", json!({ "first": "z", "second": "z" }));
    assert_eq!(snapshot(&game)["identifiers"], json!(["x", "y", "z"]));
}

#[test]
fn a_host_sends_text_only_and_the_text_is_bounded() {
    let mut game = session(LEDGER);
    send(&mut game, "pushed", json!({ "commit": A }));
    for payload in [
        json!({ "commit": 1 }),
        json!({ "commit": "" }),
        json!({ "commit": "x".repeat(1025) }),
        json!({ "commit": true }),
        json!({ "commit": null }),
    ] {
        refused(&mut game, "pushed", payload, "input", "payload_invalid");
    }
    send(&mut game, "pushed", json!({ "commit": "é".repeat(512) }));
    assert_eq!(snapshot(&game)["identifiers"].as_array().unwrap().len(), 2);
}

#[test]
fn the_native_api_takes_only_the_handle_of_an_identifier_it_holds() {
    let mut game = session(LEDGER);
    send(&mut game, "pushed", json!({ "commit": A }));
    let payload = |commit: f64| BTreeMap::from([("commit".to_string(), commit)]);
    game.apply("pushed", &payload(1.0)).unwrap();
    for handle in [0.0, 2.0, 1.5, -1.0] {
        let error = game.apply("pushed", &payload(handle)).unwrap_err();
        assert!(
            error.contains("not the handle of an identifier"),
            "{handle}: {error}"
        );
    }
    assert_eq!(snapshot(&game)["identifiers"], json!([A]));
}

#[test]
fn a_refused_or_failed_event_adds_no_identifier() {
    let refusing = format!("{LEDGER}\nevent late one id, two id;\non late reject \"not now\";");
    let mut game = session(&refusing);
    send(&mut game, "pushed", json!({ "commit": A }));
    // Two new identifiers, then a rule refuses the event.
    refused(
        &mut game,
        "late",
        json!({ "one": "p", "two": "q" }),
        "policy",
        "reject",
    );
    // A history limit partway through the event.
    let full = format!("{LEDGER}\nreadings few from ci limit 1;\nevent note commit id;\non note sample few = commit supports ready;");
    let mut limited = session(&full);
    send(&mut limited, "note", json!({ "commit": "first" }));
    refused(
        &mut limited,
        "note",
        json!({ "commit": "second" }),
        "limit",
        "history_limit",
    );
    // Handles after a refusal continue from what the session holds.
    send(&mut game, "pushed", json!({ "commit": B }));
    assert_eq!(snapshot(&game)["identifiers"], json!([A, B]));

    // A fatal error rolls back too, as every fatal event does.
    let fatal = format!("{LEDGER}\nstate shown = 0;\nevent broken commit id;\non broken set head = commit;\nbind pr.wrong = id_text(head + 5) when head > 0;");
    let mut broken = session(&fatal);
    let before = (snapshot(&broken), broken.save_json().unwrap());
    let error = broken
        .dispatch_outcome_json("broken", &json!({ "commit": A }).to_string())
        .unwrap_err();
    assert_eq!(error.code, "unclassified");
    assert!(
        error
            .message
            .contains("id_text: 6 is not the handle of an identifier"),
        "{}",
        error.message
    );
    assert_eq!((snapshot(&broken), broken.save_json().unwrap()), before);
}

#[test]
fn id_text_of_zero_is_empty_and_of_any_other_non_handle_is_an_error() {
    let game = session(LEDGER);
    assert_eq!(snapshot(&game)["bindings"]["pr"]["head"], "");
    let error = ReactiveSession::from_source(&format!("{LEDGER}\nbind pr.bad = id_text(1);"))
        .err()
        .unwrap();
    assert!(
        error.contains("id_text: 1 is not the handle of an identifier"),
        "{error}"
    );
}

#[test]
fn the_limit_refuses_a_new_identifier_and_still_takes_known_ones() {
    let mut game = session(&LEDGER.replace("identifiers limit 8;", "identifiers limit 2;"));
    send(&mut game, "pushed", json!({ "commit": A }));
    // One slot left: an event bringing two new identifiers is refused whole.
    let outcome = refused(
        &mut game,
        "pair",
        json!({ "first": "x", "second": "y" }),
        "limit",
        "identifier_limit",
    );
    assert!(outcome["message"].as_str().unwrap().contains("limit 2"));
    send(&mut game, "pushed", json!({ "commit": B }));
    refused(
        &mut game,
        "pushed",
        json!({ "commit": "third" }),
        "limit",
        "identifier_limit",
    );
    send(&mut game, "pushed", json!({ "commit": A }));
    assert_eq!(snapshot(&game)["values"]["head"], 1.0);
}

#[test]
fn a_session_holds_at_most_one_mebibyte_of_identifier_text() {
    let names = (1..=8)
        .map(|index| format!("p{index} id"))
        .collect::<Vec<_>>()
        .join(", ");
    let source =
        format!("identifiers limit 65536;\nstate x = 0;\nevent many {names};\non many set x = 1;");
    let mut game = session(&source);
    let mut next = 0_usize;
    let mut text = || {
        next += 1;
        format!("{next:0>1024}")
    };
    for _ in 0..128 {
        let payload = (1..=8)
            .map(|index| (format!("p{index}"), json!(text())))
            .collect::<serde_json::Map<_, _>>();
        send(&mut game, "many", Value::Object(payload));
    }
    assert_eq!(
        snapshot(&game)["identifiers"].as_array().unwrap().len(),
        1024
    );
    let payload = (1..=8)
        .map(|index| (format!("p{index}"), json!(text())))
        .collect::<serde_json::Map<_, _>>();
    refused(
        &mut game,
        "many",
        Value::Object(payload),
        "limit",
        "identifier_limit",
    );
}

#[test]
fn id_text_keeps_the_evidence_and_caveats_of_its_argument() {
    let source = format!("{LEDGER}\nstate cited = 0;\non checks set cited = qualified(commit, ci);\nbind pr.cited = id_text(cited);\non checks reveal ci supports ready;");
    let source = source.replace(
        "on checks set cited = qualified(commit, ci);",
        "on checks when not observed(ci) reveal ci supports ready;\non checks set cited = qualified(commit, ci);",
    );
    let source = source.replace("\non checks reveal ci supports ready;", "");
    let mut game = session(&source);
    send(&mut game, "pushed", json!({ "commit": A }));
    send(
        &mut game,
        "checks",
        json!({ "commit": A, "result": "passed" }),
    );
    let shot = snapshot(&game);
    assert_eq!(shot["bindings"]["pr"]["cited"], A);
    let lineage = &shot["binding_qualifications"]["pr"]["cited"];
    assert!(
        lineage["evidence"]
            .as_array()
            .unwrap()
            .contains(&json!("ci")),
        "{lineage}"
    );
    assert!(
        lineage["caveats"]
            .as_array()
            .unwrap()
            .contains(&json!("flaky")),
        "{lineage}"
    );
    // A received identifier on its own carries nothing.
    assert_eq!(
        shot["qualified_values"]["head"]["provenance"],
        json!({ "evidence": [], "caveats": [] })
    );
}

#[test]
fn two_observations_of_one_commit_share_its_handle_and_stay_two_observations() {
    let mut game = session(LEDGER);
    send(&mut game, "pushed", json!({ "commit": A }));
    send(
        &mut game,
        "checks",
        json!({ "commit": A, "result": "passed" }),
    );
    send(
        &mut game,
        "checks",
        json!({ "commit": A, "result": "failed" }),
    );
    let stream = &snapshot(&game)["reading_streams"]["checks"];
    let occurrences = stream["occurrences"].as_array().unwrap();
    assert_eq!(occurrences.len(), 2);
    assert_eq!(
        (
            occurrences[0]["value"].clone(),
            occurrences[1]["value"].clone()
        ),
        (json!(1.0), json!(1.0))
    );
    assert_eq!(
        (occurrences[0]["id"].as_str(), occurrences[1]["id"].as_str()),
        (Some("checks@1"), Some("checks@2"))
    );
    assert_ne!(occurrences[0]["relation"], occurrences[1]["relation"]);
}

#[test]
fn save_and_restore_keep_every_handle_and_the_same_history_gives_the_same_list() {
    let history = [
        ("pushed", json!({ "commit": A })),
        ("checks", json!({ "commit": A, "result": "passed" })),
        ("pair", json!({ "first": "x", "second": B })),
        ("pushed", json!({ "commit": B })),
    ];
    let mut original = session(LEDGER);
    let mut again = session(LEDGER);
    for (event, payload) in &history {
        send(&mut original, event, payload.clone());
        send(&mut again, event, payload.clone());
    }
    let saved = original.save_json().unwrap();
    assert_eq!(saved, again.save_json().unwrap());
    let parsed: Value = serde_json::from_str(&saved).unwrap();
    assert_eq!(parsed["identifiers"], json!([A, "x", B]));

    let mut restored = ReactiveSession::restore_json(LEDGER, &saved).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&original));
    for game in [&mut original, &mut restored] {
        send(game, "pushed", json!({ "commit": "new" }));
        send(
            game,
            "checks",
            json!({ "commit": "new", "result": "failed" }),
        );
    }
    assert_eq!(snapshot(&restored), snapshot(&original));
    assert_eq!(
        snapshot(&restored)["identifiers"],
        json!([A, "x", B, "new"])
    );
}

#[test]
fn a_saved_identifier_list_that_a_session_could_not_have_made_is_refused() {
    let mut game = session(LEDGER);
    send(&mut game, "pushed", json!({ "commit": A }));
    let saved: Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
    let with = |identifiers: Value| {
        let mut save = saved.clone();
        save["identifiers"] = identifiers;
        save.to_string()
    };
    for (identifiers, expected) in [
        (json!("abc"), "invalid type"),
        (json!([1]), "invalid type"),
        (json!([A, A]), "repeat"),
        (json!([""]), "1 to 1024 bytes"),
        (json!(["x".repeat(1025)]), "1 to 1024 bytes"),
        (
            json!(["a", "b", "c", "d", "e", "f", "g", "h", "i"]),
            "exceed the program's limit",
        ),
    ] {
        let error = ReactiveSession::restore_json(LEDGER, &with(identifiers.clone()))
            .err()
            .unwrap();
        assert!(error.contains(expected), "{identifiers}: {error}");
    }
    let plain = "state x = 0;\nevent go;\non go set x = 1;";
    let mut other = session(plain);
    other.apply("go", &BTreeMap::new()).unwrap();
    let mut save: Value = serde_json::from_str(&other.save_json().unwrap()).unwrap();
    assert!(
        save.get("identifiers").is_none(),
        "an empty list is left out"
    );
    save["identifiers"] = json!(["a"]);
    let error = ReactiveSession::restore_json(plain, &save.to_string())
        .err()
        .unwrap();
    assert!(error.contains("declares none"), "{error}");
}

#[test]
fn a_save_made_by_the_rc2_release_still_restores() {
    let source = include_str!("fixtures/rc2-thermostat.cav");
    let saved = include_str!("fixtures/rc2-thermostat.save.json");
    let restored = ReactiveSession::restore_json(source, saved).unwrap();
    let mut replayed = session(source);
    for value in [17.0, 25.0] {
        replayed
            .apply("read", &BTreeMap::from([("value".to_string(), value)]))
            .unwrap();
    }
    assert_eq!(snapshot(&restored), snapshot(&replayed));
    assert_eq!(restored.save_json().unwrap().trim_end(), saved.trim_end());
}

#[test]
fn declarations_are_checked_when_the_program_loads() {
    for (source, expected) in [
        (
            "state x = 0;\nevent go commit id;\non go set x = commit;",
            "declare identifiers limit N",
        ),
        (
            "identifiers limit 2;\nidentifiers limit 3;\nstate x = 0;\nevent go;\non go set x = 1;",
            "declared more than once",
        ),
        (
            "identifiers limit 0;\nstate x = 0;\nevent go;\non go set x = 1;",
            "limit must be in 1..65536",
        ),
        (
            "identifiers limit 65537;\nstate x = 0;\nevent go;\non go set x = 1;",
            "limit must be in 1..65536",
        ),
        (
            "identifiers 4;\nstate x = 0;\nevent go;\non go set x = 1;",
            "identifiers expects limit CAPACITY",
        ),
    ] {
        let error = ReactiveSession::from_source(source).err().unwrap();
        assert!(error.contains(expected), "{source}: {error}");
    }
}

#[test]
fn a_module_can_declare_an_identifier_parameter() {
    use caveat_runtime::link::{bundle, BundlePart};
    let part = |name: &str, source: &str| BundlePart {
        name: name.into(),
        source: source.into(),
    };
    let source = bundle(&[
        part(
            "git",
            "module git;\nidentifiers limit 4;\nevent pushed commit id;\n",
        ),
        part(
            "main",
            "use git;\nstate head = 0;\non git::pushed set head = commit;\nbind repo.head = id_text(head);\n",
        ),
    ]);
    let mut game = session(&source);
    let events = snapshot(&game)["events"].clone();
    let name = events[0]["name"].as_str().unwrap().to_string();
    send(&mut game, &name, json!({ "commit": A }));
    assert_eq!(snapshot(&game)["bindings"]["repo"]["head"], A);
}
