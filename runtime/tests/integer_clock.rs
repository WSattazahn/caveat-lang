//! An integer clock counts whole units, so every schedule applies on the exact
//! event (spec/caveat-elapsed-0.1.md, "Integer clocks"; Version Lab F267).
use caveat_runtime::reactive::{BindingValue, ReactiveSession};
use caveat_runtime::web::WebReactiveSession;
use serde_json::Value;
use std::collections::BTreeMap;

const WORLD: &str = r#"
claim safe;
caveat faded consequence material;
evidence bite from "the slime bit it";
state delay = 60000;
event step dt min 0 max 100;
event eat;
on eat reveal bite supports safe;
bind hud.faded = carries(bite, faded);
bind hud.seconds = elapsed() / 1000;
"#;

const CLOCK: &str = "clock step every 16 integer;";
const LITERAL: &str = "on eat qualify bite with faded after 60000;";

fn load(extra: &str) -> ReactiveSession {
    ReactiveSession::from_source(&format!("{WORLD}\n{CLOCK}\n{extra}"))
        .unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn step(game: &mut ReactiveSession, dt: f64) {
    game.apply("step", &BTreeMap::from([("dt".into(), dt)]))
        .unwrap();
}

fn faded(game: &ReactiveSession) -> bool {
    game.snapshot().bindings["hud"]["faded"] == BindingValue::Bool(true)
}

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn refused(source: &str, setup: &[(&str, &str)], event: &str, payload: &str) -> Value {
    let mut session = WebReactiveSession::new(source).unwrap();
    for (event, payload) in setup {
        let result = json(&session.dispatch_outcome(event, payload).unwrap());
        assert_eq!(result["outcome"], "accepted", "{event} {payload}: {result}");
    }
    let before = (session.snapshot(), session.save().unwrap());
    let result = json(&session.dispatch_outcome(event, payload).unwrap());
    assert_eq!(result["outcome"], "rejected", "{result}");
    assert_eq!((session.snapshot(), session.save().unwrap()), before);
    result
}

// F267 on an integer clock: `after 60000` with steps of 62 and 63 applies on
// the exact event at which the sum of dt reaches 60000, never one later.
#[test]
fn f267_an_integer_clock_applies_a_schedule_on_the_exact_event() {
    for scheduled_after in [0, 7, 29] {
        let mut game = load(LITERAL);
        for _ in 0..scheduled_after {
            step(&mut game, 33.0);
        }
        game.apply("eat", &BTreeMap::new()).unwrap();
        let mut sum = 0.0;
        let mut index = 0u32;
        loop {
            let dt = if index.is_multiple_of(2) { 62.0 } else { 63.0 };
            let remaining = 60000.0 - sum;
            step(&mut game, if remaining < dt { remaining } else { dt });
            sum += if remaining < dt { remaining } else { dt };
            index += 1;
            if sum < 60000.0 {
                assert!(!faded(&game), "applied early at {sum}");
            } else {
                assert!(faded(&game), "not applied at {sum}");
                break;
            }
        }
    }
}

// The same schedule, saved while it waits and restored, applies on the same
// exact event; the save keeps whole readings.
#[test]
fn an_integer_schedule_survives_save_and_restore() {
    let source = format!("{WORLD}\n{CLOCK}\n{LITERAL}");
    let mut game = load(LITERAL);
    step(&mut game, 1.0);
    game.apply("eat", &BTreeMap::new()).unwrap();
    for _ in 0..500 {
        step(&mut game, 63.0);
    }
    let saved = game.save_json().unwrap();
    let mut game = ReactiveSession::restore_json(&source, &saved).unwrap();
    for _ in 0..452 {
        step(&mut game, 63.0);
    }
    // 952 × 63 = 59,976: 24 to go.
    step(&mut game, 23.0);
    assert!(!faded(&game));
    step(&mut game, 1.0);
    assert!(faded(&game));
    assert_eq!(game.snapshot().elapsed, 60001.0);
}

#[test]
fn a_fractional_dt_on_an_integer_clock_is_refused_with_nothing_changed() {
    let source = format!("{WORLD}\n{CLOCK}\n{LITERAL}");
    let result = refused(&source, &[("eat", "{}")], "step", r#"{"dt": 16.5}"#);
    assert_eq!(result["origin"], "input");
    assert_eq!(result["code"], "payload_invalid");
    // Out of range keeps its code.
    let result = refused(&source, &[], "step", r#"{"dt": 100.5}"#);
    assert_eq!(result["code"], "bound_exceeded");
    assert_eq!(result["origin"], "input");
}

#[test]
fn a_computed_fractional_delay_is_refused_as_an_expression_failure() {
    let source = format!(
        "{WORLD}\n{CLOCK}\nevent set_delay amount min 0 max 1e12;\non set_delay set delay = amount;\non eat qualify bite with faded after delay / 3;"
    );
    let result = refused(&source, &[("set_delay", r#"{"amount": 100}"#)], "eat", "{}");
    assert_eq!(result["origin"], "evaluation");
    assert_eq!(result["code"], "expression");
    // A whole one is accepted.
    let mut session = WebReactiveSession::new(&source).unwrap();
    session
        .dispatch_outcome("set_delay", r#"{"amount": 99}"#)
        .unwrap();
    let result = json(&session.dispatch_outcome("eat", "{}").unwrap());
    assert_eq!(result["outcome"], "accepted", "{result}");
}

// The reading stays within ±(2^53 − 1). With a dt of up to 1e12 the bound is
// reachable in about 9,000 events, and the event that would pass it is refused.
#[test]
fn the_integer_clock_bound_is_reachable_and_refused() {
    let source = format!(
        "{}\nevent big dt min 0 max 1e12;\nclock big every 1 integer;",
        WORLD
    );
    let mut session = WebReactiveSession::new(&source).unwrap();
    let mut accepted = 0u64;
    let refusal = loop {
        let result = json(&session.dispatch_outcome("big", r#"{"dt": 1e12}"#).unwrap());
        if result["outcome"] != "accepted" {
            break result;
        }
        accepted += 1;
    };
    assert_eq!(accepted, 9007);
    assert_eq!(refusal["origin"], "evaluation");
    assert_eq!(refusal["code"], "bound_exceeded");
    let elapsed = json(&session.snapshot())["elapsed"].as_f64().unwrap();
    assert_eq!(elapsed, 9_007_000_000_000_000.0);
    // A smaller step still fits.
    let result = json(
        &session
            .dispatch_outcome("big", r#"{"dt": 199254740991}"#)
            .unwrap(),
    );
    assert_eq!(result["outcome"], "accepted", "{result}");
    let result = json(&session.dispatch_outcome("big", r#"{"dt": 1}"#).unwrap());
    assert_eq!(result["code"], "bound_exceeded");
}

#[test]
fn integer_clock_declarations_are_checked_when_the_program_loads() {
    for (extra, message) in [
        ("on eat qualify bite with faded after 60.5;", "whole number"),
        ("on eat qualify bite with faded after -1;", "whole number"),
        (
            "on eat qualify bite with faded after 9007199254740992;",
            "whole number",
        ),
    ] {
        let error = ReactiveSession::from_source(&format!("{WORLD}\n{CLOCK}\n{extra}"))
            .err()
            .unwrap_or_else(|| panic!("{extra} loaded"));
        assert!(error.contains(message), "{extra}: {error}");
    }
    for (source, message) in [
        (
            format!("{WORLD}\nclock step every 16.5 integer;"),
            "step must be a whole number",
        ),
        (
            WORLD.replace(
                "event step dt min 0 max 100;",
                "event step dt min 0 max 100.5;",
            ) + "\nclock step every 16 integer;",
            "dt max must be a whole number",
        ),
        (
            WORLD.to_string() + "\nevent tick dt min 0 max 0.1;\nclock tick every 0.05 integer;",
            "cannot be tick",
        ),
    ] {
        let error = ReactiveSession::from_source(&source)
            .err()
            .unwrap_or_else(|| panic!("{source} loaded"));
        assert!(error.contains(message), "{error}");
    }
}

#[test]
fn only_an_integer_clock_publishes_integer_metadata() {
    let integer = load("");
    let snapshot = serde_json::to_value(integer.snapshot()).unwrap();
    assert_eq!(snapshot["clock"]["integer"], true);
    let plain = ReactiveSession::from_source(&format!("{WORLD}\nclock step every 16;")).unwrap();
    let snapshot = serde_json::to_value(plain.snapshot()).unwrap();
    assert_eq!(
        snapshot["clock"],
        serde_json::json!({"event": "step", "step": 16.0})
    );
}

type Edit = fn(&mut Value);

#[test]
fn restore_refuses_fractional_or_out_of_range_integer_clock_times() {
    let source = format!("{WORLD}\n{CLOCK}\n{LITERAL}");
    let mut game = load(LITERAL);
    step(&mut game, 16.0);
    game.apply("eat", &BTreeMap::new()).unwrap();
    step(&mut game, 16.0);
    let saved: Value = json(&game.save_json().unwrap());
    ReactiveSession::restore_json(&source, &saved.to_string()).unwrap();
    let edits: [(&str, Edit); 4] = [
        ("elapsed", |save| save["elapsed"] = 32.5.into()),
        ("elapsed", |save| {
            save["elapsed"] = 9007199254740992.0.into()
        }),
        ("scheduled", |save| {
            save["scheduled_qualifications"][0]["scheduled_at"] = 16.5.into()
        }),
        ("scheduled", |save| {
            save["scheduled_qualifications"][0]["after"] = 59999.5.into()
        }),
    ];
    for (what, edit) in edits {
        let mut altered = saved.clone();
        edit(&mut altered);
        let error = ReactiveSession::restore_json(&source, &altered.to_string())
            .err()
            .unwrap_or_else(|| panic!("{what}: {altered} restored"));
        assert!(error.contains("integer clock"), "{what}: {error}");
    }
    // The same fractions restore on the binary64 clock.
    let plain = format!("{WORLD}\nclock step every 16;\n{LITERAL}");
    let mut game = ReactiveSession::from_source(&plain).unwrap();
    step(&mut game, 16.0);
    game.apply("eat", &BTreeMap::new()).unwrap();
    step(&mut game, 16.5);
    ReactiveSession::restore_json(&plain, &game.save_json().unwrap()).unwrap();
}

// Glowcap round 7's four Caveat programs, converted to an integer clock in
// milliseconds (experiments/glowcap/round7/clock/): a taste fades on the exact
// step at which 60,000 ms have passed, with steps of 62 and 63.
#[test]
fn round_7_programs_on_an_integer_clock_fade_a_taste_at_exactly_sixty_seconds() {
    let folder = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../experiments/glowcap/round7/clock"
    );
    for author in ["C1", "C2", "C3", "C4"] {
        let source = std::fs::read_to_string(format!("{folder}/{author}.cav")).unwrap();
        let mut game =
            WebReactiveSession::new(&source).unwrap_or_else(|error| panic!("{author}: {error}"));
        let snapshot = json(&game.snapshot());
        assert_eq!(snapshot["clock"]["integer"], true, "{author}");
        let taste = snapshot["events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|event| event["name"] == "taste")
            .unwrap_or_else(|| panic!("{author}: no taste event"));
        let mushroom = taste["parameters"][0]["domain"]["entity"]["members"][0]
            .as_str()
            .unwrap_or_else(|| panic!("{author}: {taste}"))
            .to_string();
        let result = json(
            &game
                .dispatch_outcome(
                    "taste",
                    &format!(r#"{{"target": "{mushroom}", "sort": "glowcap"}}"#),
                )
                .unwrap(),
        );
        assert_eq!(result["outcome"], "accepted", "{author}: {result}");
        let waiting = |game: &WebReactiveSession| {
            json(&game.snapshot())["scheduled_qualifications"]
                .as_array()
                .map_or(0, Vec::len)
        };
        assert_eq!(waiting(&game), 1, "{author}");
        let mut sum = 0u32;
        let mut index = 0u32;
        while sum < 60000 {
            let dt = (if index.is_multiple_of(2) { 62 } else { 63 }).min(60000 - sum);
            let result = json(
                &game
                    .dispatch_outcome("step", &format!(r#"{{"dt": {dt}}}"#))
                    .unwrap(),
            );
            assert_eq!(result["outcome"], "accepted", "{author}: {result}");
            sum += dt;
            index += 1;
            assert_eq!(
                waiting(&game),
                usize::from(sum < 60000),
                "{author} at {sum} ms"
            );
        }
    }
}
