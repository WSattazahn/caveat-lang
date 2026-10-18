// Identifiers 0.1 under random events: an identifier is a text, equal only
// byte for byte; the session hands out handles 1, 2, 3, … in the order it
// first receives each text, numbering a single event's new identifiers in
// the order its parameters are declared; a refused event, whatever refuses
// it, adds no identifier; past the limit a new identifier refuses the event
// as limit/identifier_limit while a known one is still accepted; `id_text`
// gives the text of a handle, the empty text for 0, and refuses the event as
// evaluation/expression for any other number; a payload must be a string of
// 1 to 1,024 bytes; the snapshot lists the texts in handle order and the save
// holds the same list only when it is not empty; and the same history gives
// the same list. Checked after every event, with saves and restores in place.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

const SOURCE: &str = r#"
identifiers limit 6;
claim ready;
evidence ci from "the pull request's checks";
readings checks from ci limit 40;
state head = 0;
state other = 0;
state n = 0;
event pushed commit id;
event checks commit id, result in failed passed;
event pair first id, second id;
event pair_reject first id, second id, bad min 0 max 1;
event probe x min -1 max 9;
on pushed set head = commit;
on checks when commit != head reject "Checks for a commit that is not the head.";
on checks when result == result.passed sample checks = commit supports ready;
on checks when result == result.failed sample checks = commit opposes ready;
on pair set head = first;
on pair set other = second;
on pair_reject set head = first;
on pair_reject set other = second;
on pair_reject when bad == 1 reject "A pair the policy refuses.";
on probe set n = x;
bind view.head = id_text(head);
bind view.other = id_text(other);
bind view.probe = id_text(n);
"#;

// Texts that differ only in case, spacing or a doubled separator are
// different identifiers; the pool outgrows the limit of 6.
const TEXTS: [&str; 9] = ["a", "A", "a ", " a", "a/b", "a//b", "b", "c", "d"];

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn strings(list: &Value) -> Vec<String> {
    list.as_array()
        .map(|items| {
            items
                .iter()
                .map(|item| item.as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn next(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *seed >> 33
}

/// What the spec says the event does: the refusal it meets, or the texts it
/// adds, in parameter order.
enum Expected {
    Refused(&'static str, &'static str),
    Accepted,
}

struct Model {
    ids: Vec<String>,
    head: usize,
    other: usize,
    n: f64,
}

impl Model {
    /// Numbers the event's new texts in parameter order and says what happens.
    fn apply(&mut self, event: &str, payload: &Value) -> Expected {
        let texts: Vec<&str> = match event {
            "pushed" | "checks" => vec![payload["commit"].as_str().unwrap()],
            "pair" | "pair_reject" => vec![
                payload["first"].as_str().unwrap(),
                payload["second"].as_str().unwrap(),
            ],
            _ => vec![],
        };
        let mut ids = self.ids.clone();
        for text in &texts {
            if !ids.iter().any(|known| known == text) {
                ids.push(text.to_string());
            }
        }
        if ids.len() > 6 {
            return Expected::Refused("limit", "identifier_limit");
        }
        let handle = |text: &str| ids.iter().position(|known| known == text).unwrap() + 1;
        match event {
            "pushed" => {
                self.head = handle(texts[0]);
            }
            "checks" => {
                if handle(texts[0]) != self.head {
                    return Expected::Refused("policy", "reject");
                }
            }
            "pair" => {
                self.head = handle(texts[0]);
                self.other = handle(texts[1]);
            }
            "pair_reject" => {
                if payload["bad"] == 1 {
                    return Expected::Refused("policy", "reject");
                }
                self.head = handle(texts[0]);
                self.other = handle(texts[1]);
            }
            "probe" => {
                let x = payload["x"].as_f64().unwrap();
                if x != 0.0 && !(x >= 1.0 && x <= ids.len() as f64 && x.fract() == 0.0) {
                    return Expected::Refused("evaluation", "expression");
                }
                self.n = x;
            }
            _ => unreachable!(),
        }
        self.ids = ids;
        Expected::Accepted
    }

    fn text(&self, handle: usize) -> &str {
        if handle == 0 {
            ""
        } else {
            &self.ids[handle - 1]
        }
    }
}

#[test]
fn the_identifier_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn handles_number_texts_in_receipt_order_and_refusals_add_nothing() {
    let mut seed: u64 = 0x1d3f_8a27_c6b4_9e05;
    let (mut accepted, mut limit_refusals, mut rejects, mut payload_refusals, mut id_text_refusals) =
        (0, 0, 0, 0, 0);
    let mut known_at_limit = 0;
    for run in 0..200 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail: Vec<(String, Value)> = Vec::new();
        let mut before = snapshot(&game);
        let mut model = Model {
            ids: Vec::new(),
            head: 0,
            other: 0,
            n: 0.0,
        };
        for step in 0..60 {
            let pick = next(&mut seed) % 12;
            let text = |seed: &mut u64| TEXTS[next(seed) as usize % TEXTS.len()];
            let (event, payload) = match pick {
                0 | 1 => ("pushed", json!({ "commit": text(&mut seed) })),
                2 | 3 => (
                    "checks",
                    json!({ "commit": text(&mut seed), "result": if next(&mut seed).is_multiple_of(2) { "passed" } else { "failed" } }),
                ),
                4 => (
                    "pair",
                    json!({ "second": text(&mut seed), "first": text(&mut seed) }),
                ),
                5 => (
                    "pair_reject",
                    json!({ "first": text(&mut seed), "second": text(&mut seed), "bad": next(&mut seed) % 2 }),
                ),
                6 | 7 => ("probe", json!({ "x": (next(&mut seed) % 11) as i64 - 1 })),
                8 => {
                    // Not a text, empty, or past the bound: refused before anything.
                    let bad = match next(&mut seed) % 4 {
                        0 => json!(7),
                        1 => json!(""),
                        2 => json!("x".repeat(1025)),
                        _ => json!(null),
                    };
                    ("pushed", json!({ "commit": bad }))
                }
                9 => ("pushed", json!({ "commit": "y".repeat(1024) })),
                _ => {
                    trail.push(("save".to_string(), Value::Null));
                    let saved: Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
                    assert_eq!(
                        saved.get("identifiers").map(strings),
                        (!model.ids.is_empty()).then(|| model.ids.clone()),
                        "run {run} step {step}: the save's list: {trail:?}"
                    );
                    game = ReactiveSession::restore_json(SOURCE, &saved.to_string())
                        .unwrap_or_else(|error| {
                            panic!("run {run} step {step} restore: {error}: {trail:?}")
                        });
                    assert_eq!(
                        snapshot(&game),
                        before,
                        "run {run} step {step} after restore: {trail:?}"
                    );
                    continue;
                }
            };
            trail.push((event.to_string(), payload.clone()));
            let at = format!("run {run} step {step} after {event} {payload}: {trail:?}");
            let save_before = game.save_json().unwrap();
            let outcome = serde_json::to_value(
                game.dispatch_outcome_json(event, &payload.to_string())
                    .unwrap_or_else(|fatal| panic!("{at}: fatal {}", fatal.message)),
            )
            .unwrap();
            let expected = if pick == 8 {
                Expected::Refused("input", "payload_invalid")
            } else {
                model.apply(event, &payload)
            };
            match expected {
                Expected::Refused(origin, code) => {
                    assert_eq!(
                        (outcome["origin"].as_str(), outcome["code"].as_str()),
                        (Some(origin), Some(code)),
                        "{at}: {outcome}"
                    );
                    assert_eq!(
                        game.save_json().unwrap(),
                        save_before,
                        "{at}: a refused event changed the save"
                    );
                    assert_eq!(
                        snapshot(&game),
                        before,
                        "{at}: a refused event changed the snapshot"
                    );
                    match code {
                        "identifier_limit" => limit_refusals += 1,
                        "reject" => rejects += 1,
                        "payload_invalid" => payload_refusals += 1,
                        _ => id_text_refusals += 1,
                    }
                    continue;
                }
                Expected::Accepted => {
                    assert_eq!(outcome["outcome"], "accepted", "{at}: {outcome}");
                }
            }
            accepted += 1;
            if model.ids.len() == 6 && matches!(event, "pushed" | "checks" | "pair" | "pair_reject")
            {
                known_at_limit += 1;
            }
            let shown = snapshot(&game);
            assert_eq!(
                strings(&shown["identifiers"]),
                model.ids,
                "{at}: the handle order"
            );
            assert_eq!(
                shown["values"]["head"].as_f64(),
                Some(model.head as f64),
                "{at}: head"
            );
            assert_eq!(
                shown["values"]["other"].as_f64(),
                Some(model.other as f64),
                "{at}: other"
            );
            assert_eq!(
                shown["bindings"]["view"]["head"],
                model.text(model.head),
                "{at}: id_text(head)"
            );
            assert_eq!(
                shown["bindings"]["view"]["other"],
                model.text(model.other),
                "{at}: id_text(other)"
            );
            assert_eq!(
                shown["bindings"]["view"]["probe"],
                model.text(model.n as usize),
                "{at}: id_text(n)"
            );
            // A reading of an identifier is its handle.
            if let Some(reading) = shown["reading_streams"]["checks"]["occurrences"]
                .as_array()
                .and_then(|list| list.last())
            {
                if event == "checks" {
                    assert_eq!(
                        reading["value"].as_f64(),
                        Some(model.head as f64),
                        "{at}: the reading's value is not the head's handle"
                    );
                }
            }
            before = shown;
        }
        // The same history gives the same list, and the same session.
        let mut replay = ReactiveSession::from_source(SOURCE).unwrap();
        for (event, payload) in &trail {
            if event == "save" {
                continue;
            }
            replay
                .dispatch_outcome_json(event, &payload.to_string())
                .unwrap_or_else(|fatal| panic!("run {run} replay fatal {}", fatal.message));
        }
        assert_eq!(
            snapshot(&replay),
            before,
            "run {run}: replaying the history: {trail:?}"
        );
    }
    eprintln!("{accepted} accepted events, {limit_refusals} refused at the identifier limit, {known_at_limit} known identifiers accepted at the limit, {rejects} policy rejects, {payload_refusals} payloads refused, {id_text_refusals} id_text refusals");
    assert!(
        limit_refusals > 500
            && known_at_limit > 500
            && rejects > 300
            && payload_refusals > 300
            && id_text_refusals > 300,
        "{limit_refusals} / {known_at_limit} / {rejects} / {payload_refusals} / {id_text_refusals}"
    );
}
