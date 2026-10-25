// Modules (draft 0.5, sections 2, 3, 4 and 8) under random events: a bundle
// links to the flat program its parts spell, with each imported name written
// `module__symbol` and host bindings left as they are; so the bundle and its
// hand-flattened twin, driven in lockstep, give the same snapshot but for
// source identity (Authorship 0.1's origin statements included), refusal for
// refusal: one graph, one budget, one
// transaction, importing observes and examines nothing, a module's state is
// one cell under a scoped name, a module's rule fires on its own event, and
// a trigger, a retaining clause, a reading stream and a procedure reach
// across the boundary. Saves and restores in place on both.
use caveat_runtime::link::{self, BundlePart};
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const MODULE: &str = r#"module glow;
claim lit;
evidence lamp from "the lamp";
evidence meter from "the light meter";
caveat flicker consequence material;
caveat dusty consequence low;
dusty qualifies lamp;
state level = 0 min 0 max 9;
state charges = 0;
readings lux from meter limit 20;
decisions keep limit 24 reopened by lux opposing lit;
event charge v min 0 max 9;
on charge set level = v;
on charge set charges = charges + 1;
proc light() {
    reveal lamp supports lit;
    set level = qualified(level, lamp);
};
proc dim(c caveat) {
    examine c cost 2;
};
bind ability.level = level;
bind ability.lit = if(observed(lamp), 1, 0);
"#;

const MAIN: &str = r#"use glow;
budget 30;
state seen = 0;
event look;
event read v min 0 max 9;
event bad v min 0 max 9;
event decide;
event doubt;
event undoubt;
event inspect;
event reconsider;
on look call glow::light();
on read sample glow::lux = v opposes glow::lit;
on bad sample glow::lux = v supports glow::lit;
on bad when v == 7 reject "Seven is refused after the sample.";
on decide when has_sample(glow::lux) and (not committed(glow::keep) or reopened(glow::keep)) commit glow::keep because enough using latest(glow::lux) + glow::level retaining glow::dusty;
on doubt when observed(glow::lamp) qualify glow::lamp with glow::flicker;
on undoubt unqualify glow::lamp with glow::flicker;
on inspect when not examined(glow::dusty) call glow::dim(glow::dusty);
on inspect set seen = seen + 1;
on reconsider when committed(glow::keep) and not reopened(glow::keep) and observed(glow::lamp) reopen glow::keep because glow::lamp;
bind view.seen = seen;
bind view.examined = if(examined(glow::dusty), 1, 0);
bind view.charges = glow::charges;
"#;

/// The twin, spelt flat by hand as section 3 and 5 say the linker spells it.
fn flat() -> String {
    let module = MODULE
        .replace("module glow;\n", "origin glow;\n")
        .replace("claim lit", "claim glow__lit")
        .replace("evidence lamp", "evidence glow__lamp")
        .replace("evidence meter", "evidence glow__meter")
        .replace("caveat flicker", "caveat glow__flicker")
        .replace("caveat dusty", "caveat glow__dusty")
        .replace("dusty qualifies lamp", "glow__dusty qualifies glow__lamp")
        .replace("state level", "state glow__level")
        .replace("state charges", "state glow__charges")
        .replace(
            "readings lux from meter",
            "readings glow__lux from glow__meter",
        )
        .replace(
            "decisions keep limit 24 reopened by lux opposing lit",
            "decisions glow__keep limit 24 reopened by glow__lux opposing glow__lit",
        )
        .replace("event charge", "event glow__charge")
        .replace(
            "on charge set level = v",
            "on glow__charge set glow__level = v",
        )
        .replace(
            "on charge set charges = charges + 1",
            "on glow__charge set glow__charges = glow__charges + 1",
        )
        .replace("proc light()", "proc glow__light()")
        .replace(
            "reveal lamp supports lit",
            "reveal glow__lamp supports glow__lit",
        )
        .replace(
            "set level = qualified(level, lamp)",
            "set glow__level = qualified(glow__level, glow__lamp)",
        )
        .replace("proc dim(c caveat)", "proc glow__dim(c caveat)")
        .replace(
            "bind ability.level = level",
            "bind ability.level = glow__level",
        )
        .replace(
            "bind ability.lit = if(observed(lamp), 1, 0)",
            "bind ability.lit = if(observed(glow__lamp), 1, 0)",
        );
    let main = MAIN
        .replace("use glow;\n", "origin main;\n")
        .replace("glow::", "glow__");
    format!("{module}{main}")
}

fn bundle() -> String {
    link::bundle(&[
        BundlePart {
            name: "glow".into(),
            source: MODULE.into(),
        },
        BundlePart {
            name: "main".into(),
            source: MAIN.into(),
        },
    ])
}

const EVENTS: [(&str, &str); 16] = [
    ("glow__charge", r#"{"v": 3}"#),
    ("glow__charge", r#"{"v": 8}"#),
    ("look", "{}"),
    ("read", r#"{"v": 2}"#),
    ("read", r#"{"v": 5}"#),
    ("bad", r#"{"v": 7}"#),
    ("bad", r#"{"v": 1}"#),
    ("decide", "{}"),
    ("doubt", "{}"),
    ("undoubt", "{}"),
    ("inspect", "{}"),
    ("reconsider", "{}"),
    ("decide", "{}"),
    ("read", r#"{"v": 9}"#),
    ("inspect", "{}"),
    ("look", "{}"),
];

/// Takes the source's identity out of a snapshot, wherever it stands.
fn anonymous(mut shown: Value) -> Value {
    if let Some(object) = shown.as_object_mut() {
        object.remove("source_id");
        object.remove("source_sha256");
        if let Some(inner) = object.get_mut("snapshot") {
            *inner = anonymous(inner.take());
        }
    }
    shown
}

fn snapshot(game: &ReactiveSession) -> Value {
    anonymous(serde_json::to_value(game.snapshot()).unwrap())
}

#[test]
fn the_bundle_links_to_its_flat_twin() {
    let linked = link::link(&bundle()).unwrap();
    ReactiveSession::from_source(&linked).unwrap();
    ReactiveSession::from_source(&flat()).unwrap();
    // The twin is the program the linker writes, statement for statement.
    let statements = |text: &str| {
        text.split(';')
            .map(|part| part.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
    };
    assert_eq!(statements(&linked), statements(&flat()), "{linked}");
}

#[test]
fn a_bundle_runs_as_its_flat_twin() {
    let bundled = bundle();
    let twin = flat();
    let mut seed: u64 = 0x8d4b_26f1_e0c9_3a57;
    let (mut accepted, mut refused, mut commits, mut crossings) = (0, 0, 0, 0);
    for run in 0..150 {
        let mut left = ReactiveSession::from_source(&bundled).unwrap();
        let mut right = ReactiveSession::from_source(&twin).unwrap();
        assert_eq!(snapshot(&left), snapshot(&right), "run {run}: at load");
        let mut trail = Vec::new();
        for step in 0..50 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                trail.push("save");
                let before = snapshot(&left);
                left = ReactiveSession::restore_json(&bundled, &left.save_json().unwrap())
                    .unwrap_or_else(|error| {
                        panic!("run {run} step {step} restore: {error}: {trail:?}")
                    });
                right = ReactiveSession::restore_json(&twin, &right.save_json().unwrap())
                    .unwrap_or_else(|error| {
                        panic!("run {run} step {step} restore: {error}: {trail:?}")
                    });
                assert_eq!(snapshot(&left), before, "run {run} step {step}: {trail:?}");
                assert_eq!(snapshot(&right), before, "run {run} step {step}: {trail:?}");
                continue;
            }
            let (event, payload) = EVENTS[pick];
            trail.push(event);
            let at = format!("run {run} step {step} after {event} {payload}: {trail:?}");
            let bundled_outcome = anonymous(
                serde_json::to_value(
                    left.dispatch_outcome_json(event, payload)
                        .unwrap_or_else(|fatal| panic!("{at}: bundle fatal {}", fatal.message)),
                )
                .unwrap(),
            );
            let flat_outcome = anonymous(
                serde_json::to_value(
                    right
                        .dispatch_outcome_json(event, payload)
                        .unwrap_or_else(|fatal| panic!("{at}: twin fatal {}", fatal.message)),
                )
                .unwrap(),
            );
            assert_eq!(bundled_outcome, flat_outcome, "{at}: the outcomes differ");
            assert_eq!(
                snapshot(&left),
                snapshot(&right),
                "{at}: the snapshots differ"
            );
            // Every event's save restores, to the same snapshot (D021).
            let saved_left = left.save_json().unwrap();
            let restored_left = ReactiveSession::restore_json(&bundled, &saved_left)
                .unwrap_or_else(|error| panic!("{at}: restore after the event: {error}"));
            assert_eq!(
                restored_left.save_json().unwrap(),
                saved_left,
                "{at}: save round trip"
            );
            assert_eq!(
                snapshot(&restored_left),
                snapshot(&left),
                "{at}: snapshot after restore"
            );
            // Every event's save restores, to the same snapshot (D021).
            let saved_right = right.save_json().unwrap();
            let restored_right = ReactiveSession::restore_json(&twin, &saved_right)
                .unwrap_or_else(|error| panic!("{at}: restore after the event: {error}"));
            assert_eq!(
                restored_right.save_json().unwrap(),
                saved_right,
                "{at}: save round trip"
            );
            assert_eq!(
                snapshot(&restored_right),
                snapshot(&right),
                "{at}: snapshot after restore"
            );
            if bundled_outcome["outcome"] == "accepted" {
                accepted += 1;
                let effects = bundled_outcome["snapshot"]["effects"].as_array().unwrap();
                if effects.iter().any(|effect| effect["kind"] == "commit") {
                    commits += 1;
                }
                if matches!(event, "look" | "inspect") && !effects.is_empty() {
                    crossings += 1;
                }
            } else {
                refused += 1;
            }
        }
    }
    eprintln!("{accepted} accepted events, {refused} refused, {commits} commits, {crossings} calls across the boundary with an effect");
    assert!(
        accepted > 4000 && refused > 300 && commits > 300 && crossings >= 250,
        "{accepted} / {refused} / {commits} / {crossings}"
    );
}
