//! Saving a reactive session and resuming it without replaying events. See
//! spec/caveat-save-0.1.md.
use caveat_runtime::reactive::{ReactiveSave, ReactiveSession};
use std::collections::BTreeMap;

const PROGRAM: &str = r#"
budget 9;
claim safe;
evidence forecast from "forecast";
evidence sensor from "sensor";
evidence bite from "bite";
caveat stale consequence material;
caveat unmeasured consequence low;
caveat faded consequence low;
forecast supports safe;
unmeasured qualifies forecast;
renewable bite limit 8;
readings flow from sensor limit 16;
decisions route limit 16;
state support = qualified(1, forecast);
state level = 0 min 0 max 100;
state now = 0;
cue ping toast "Ping" 1;
event tick dt min 0 max 0.1;
event start;
event read x min -10 max 10;
event eat;
event regrow;
event check;
on tick set now = now + dt;
on start commit route because enough using support;
on read sample flow = x opposes safe;
on read when committed(route) and not reopened(route) reopen route because latest(flow);
on read when reopened(route) commit route because enough using latest(flow);
on eat reveal bite supports safe;
on eat set support = support + qualified(1, bite);
on eat set level = level + 10 because nothing;
on eat qualify bite with faded after 30;
on eat emit ping;
on regrow renew bite;
on check examine stale cost 1;
// A guard that reads qualified state: the lineage gains it, the grounds do not.
on check when support > 0 set level = level + 1;
bind hud.text = "ok" because nothing;
bind hud.text = "faded" when carries(bite, faded) because support;
bind hud.level = level;
bind hud.route = if(committed(route), latest(route), 0);
"#;

fn send(game: &mut ReactiveSession, event: &str, parameters: &[(&str, f64)]) {
    game.apply(
        event,
        &parameters
            .iter()
            .map(|(name, value)| (name.to_string(), *value))
            .collect(),
    )
    .unwrap_or_else(|error| panic!("{event}: {error}"));
}

fn wait(game: &mut ReactiveSession, seconds: usize) {
    for _ in 0..seconds * 16 {
        send(game, "tick", &[("dt", 0.0625)]);
    }
}

/// A session with everything a save must carry: states with lineage and
/// grounds, readings, a decision series revised by a reading, attention
/// spent, a cue, renewed evidence, a fired and a pending late caveat.
fn played() -> ReactiveSession {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    send(&mut game, "start", &[]);
    send(&mut game, "read", &[("x", 2.0)]);
    send(&mut game, "read", &[("x", 3.0)]);
    send(&mut game, "eat", &[]);
    wait(&mut game, 10);
    send(&mut game, "regrow", &[]);
    send(&mut game, "eat", &[]);
    wait(&mut game, 25);
    send(&mut game, "check", &[]);
    game
}

#[test]
fn a_resumed_session_is_the_session_that_was_saved() {
    let mut original = played();
    let snapshot = original.snapshot();
    assert_eq!(
        snapshot.scheduled_qualifications.len(),
        1,
        "one caveat still pending"
    );
    assert_eq!(snapshot.renewals["bite"].occurrences.len(), 2);
    assert_eq!(snapshot.budget.as_ref().unwrap().spent, 1);

    let save = original.save().unwrap();
    let mut resumed = ReactiveSession::restore(PROGRAM, &save).unwrap();
    assert_eq!(resumed.snapshot(), snapshot);

    // And it stays the same session through whatever comes next.
    let next: &[(&str, &[(&str, f64)])] = &[
        ("tick", &[("dt", 0.1)]),
        ("read", &[("x", -1.0)]),
        ("eat", &[]),
        ("regrow", &[]),
        ("check", &[]),
    ];
    for _ in 0..3 {
        for (event, parameters) in next {
            send(&mut original, event, parameters);
            send(&mut resumed, event, parameters);
            assert_eq!(resumed.snapshot(), original.snapshot(), "after {event}");
        }
        wait(&mut original, 10);
        wait(&mut resumed, 10);
        assert_eq!(resumed.snapshot(), original.snapshot());
    }
}

#[test]
fn a_save_round_trips_through_json_and_stays_small() {
    let original = played();
    let json = original.save_json().unwrap();
    let resumed = ReactiveSession::restore_json(PROGRAM, &json).unwrap();
    assert_eq!(resumed.snapshot(), original.snapshot());
    assert!(json.len() < 8 * 1024, "{} bytes", json.len());
    // Bindings are computed again, not stored.
    assert!(!json.contains("\"bindings\""));
}

#[test]
fn a_fresh_session_saves_almost_nothing() {
    let fresh = ReactiveSession::from_source(PROGRAM).unwrap();
    let json = fresh.save_json().unwrap();
    let resumed = ReactiveSession::restore_json(PROGRAM, &json).unwrap();
    assert_eq!(resumed.snapshot(), fresh.snapshot());
    // States, empty histories and the budget: 649 bytes for this program.
    assert!(json.len() < 1024, "{json}");
}

/// One way of altering a saved game.
type Alteration = Box<dyn Fn(&mut serde_json::Value)>;

fn tampered(change: impl Fn(&mut serde_json::Value)) -> String {
    let mut save = serde_json::to_value(played().save().unwrap()).unwrap();
    change(&mut save);
    ReactiveSession::restore_json(PROGRAM, &save.to_string())
        .map(|_| ())
        .unwrap_err()
}

#[test]
fn a_save_that_does_not_fit_the_program_is_refused() {
    let cases: Vec<(&str, Alteration)> = vec![
        (
            "different program",
            Box::new(|save| save["source_id"] = "fnv1a64:0:0".into()),
        ),
        (
            "schema",
            Box::new(|save| save["schema"] = "caveat-reactive-save/9".into()),
        ),
        (
            "its states are not",
            Box::new(|save| {
                save["states"]["intruder"] = serde_json::json!({"value": 1});
            }),
        ),
        (
            "level",
            Box::new(|save| save["states"]["level"]["value"] = 500.into()),
        ),
        (
            "must name a declared evidence",
            Box::new(|save| {
                save["states"]["support"]["lineage"]["evidence"] = serde_json::json!(["nowhere"]);
            }),
        ),
        (
            "relation names unknown",
            Box::new(|save| {
                save["graph"]["relations"][0][0] = "ghost".into();
            }),
        ),
        (
            "unknown relation",
            Box::new(|save| {
                save["graph"]["relations"][0][1] = "adores".into();
            }),
        ),
        (
            "not an occurrence",
            Box::new(|save| {
                save["graph"]["nodes"][0] =
                    serde_json::json!({"kind": "occurrence", "name": "forecast@2"});
            }),
        ),
        (
            "is not a commitment this program makes",
            Box::new(|save| {
                save["graph"]["nodes"][0] = serde_json::json!({"kind": "commitment", "name": "surrender", "reason": "enough"});
            }),
        ),
        (
            "has no basis",
            Box::new(|save| {
                save["commitment_bases"]
                    .as_object_mut()
                    .unwrap()
                    .remove("route@1");
            }),
        ),
        (
            "current is not its latest",
            Box::new(|save| {
                save["decision_series"]["route"]["current"] = "route@1".into();
            }),
        ),
        (
            "occurrences do not match",
            Box::new(|save| {
                save["renewals"]["bite"] = serde_json::json!([
                    "bite", "bite@2", "bite@3", "bite@4", "bite@5", "bite@6", "bite@7", "bite@8",
                    "bite@9"
                ]);
            }),
        ),
        (
            "must name a declared caveat",
            Box::new(|save| {
                save["scheduled_qualifications"][0]["caveat"] = "forecast".into();
            }),
        ),
        (
            "attention budget",
            Box::new(|save| save["resources"]["spent"] = 0.into()),
        ),
        (
            "unknown cue",
            Box::new(|save| {
                save["cues"] = serde_json::json!(["siren"]);
                save["cue_qualifications"] = serde_json::json!([{}]);
            }),
        ),
        (
            "unknown field",
            Box::new(|save| save["cheat"] = true.into()),
        ),
        (
            "unknown event",
            Box::new(|save| save["last_event"] = "win".into()),
        ),
    ];
    for (expected, change) in cases {
        let error = tampered(change);
        assert!(error.contains(expected), "{expected}: {error}");
    }
}

/// A small deterministic generator, so a failure names its seed.
struct Mulberry(u32);

impl Mulberry {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        t ^ (t >> 14)
    }

    fn below(&mut self, n: usize) -> usize {
        self.next() as usize % n.max(1)
    }
}

/// Every place in a JSON value, as a path of keys and indexes.
fn paths(
    value: &serde_json::Value,
    at: Vec<serde_json::Value>,
    out: &mut Vec<Vec<serde_json::Value>>,
) {
    out.push(at.clone());
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let mut next = at.clone();
                next.push(key.clone().into());
                paths(child, next, out);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let mut next = at.clone();
                next.push(index.into());
                paths(child, next, out);
            }
        }
        _ => {}
    }
}

/// The place at `path`, if an earlier alteration has not removed it.
fn at<'a>(
    value: &'a mut serde_json::Value,
    path: &[serde_json::Value],
) -> Option<&'a mut serde_json::Value> {
    let pointer: String = path
        .iter()
        .map(|step| match step {
            serde_json::Value::String(key) => {
                format!("/{}", key.replace('~', "~0").replace('/', "~1"))
            }
            other => format!("/{other}"),
        })
        .collect();
    value.pointer_mut(&pointer)
}

#[test]
fn no_altered_save_can_crash_the_runtime() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    let mut places = Vec::new();
    paths(&save, Vec::new(), &mut places);
    let names = [
        "bite",
        "bite@2",
        "bite@9",
        "route@1",
        "route@7",
        "flow@1",
        "stale",
        "safe",
        "",
        "@",
        "x@0",
        "unexamined",
    ];
    // SAVE_FUZZ_ROUNDS and SAVE_FUZZ_SEED run it longer or differently.
    let setting = |name: &str, default: u32| {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    };
    let rounds = setting("SAVE_FUZZ_ROUNDS", 3000);
    let mut random = Mulberry(setting("SAVE_FUZZ_SEED", 20_260_922));
    let mut accepted = 0;
    for round in 0..rounds {
        let mut altered = save.clone();
        for _ in 0..1 + random.below(3) {
            let path = &places[random.below(places.len())];
            let Some(target) = at(&mut altered, path) else {
                continue;
            };
            *target = match random.below(7) {
                0 => serde_json::Value::Null,
                1 => names[random.below(names.len())].into(),
                2 => (random.below(2000) as f64 - 1000.0).into(),
                3 => serde_json::json!([]),
                4 => serde_json::json!({}),
                5 => serde_json::json!([names[random.below(names.len())]]),
                _ => u64::MAX.into(),
            };
        }
        let text = altered.to_string();
        let outcome = std::panic::catch_unwind(|| ReactiveSession::restore_json(PROGRAM, &text));
        let restored = outcome.unwrap_or_else(|_| panic!("round {round} panicked on {text}"));
        if let Ok(mut game) = restored {
            accepted += 1;
            // Whatever was accepted must also keep running without panicking.
            let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                for event in ["tick", "read", "eat", "regrow", "check", "start"] {
                    let parameters: BTreeMap<String, f64> = match event {
                        "tick" => [("dt".to_string(), 0.1)].into(),
                        "read" => [("x".to_string(), 1.0)].into(),
                        _ => BTreeMap::new(),
                    };
                    let _ = game.apply(event, &parameters);
                    let _ = game.snapshot();
                    let _ = serde_json::to_string(&game.view());
                    let _ = game.save();
                }
            }));
            assert!(
                run.is_ok(),
                "round {round}: an accepted save crashed a later event: {text}"
            );
        }
    }
    // Some alterations are harmless (a changed number within range): make
    // sure the generator exercised both outcomes.
    assert!(accepted > 0 && accepted < rounds, "accepted {accepted}");
}

#[test]
fn a_save_is_a_plain_value_a_host_can_inspect() {
    let save: ReactiveSave = serde_json::from_str(&played().save_json().unwrap()).unwrap();
    assert_eq!(save.schema, "caveat-reactive-save/0.1");
    assert_eq!(save.renewals["bite"], ["bite", "bite@2"]);
    assert_eq!(save.states["level"].value, 21.0);
}

#[test]
fn grounds_that_differ_from_lineage_survive_a_save() {
    let original = played();
    let level = &original.snapshot().qualified_values["level"];
    let grounds = &original.snapshot().value_grounds["level"];
    assert_ne!(&level.provenance, grounds, "the fixture must exercise this");
    let save = original.save().unwrap();
    assert!(save.states["level"].grounds.is_some());
    let resumed = ReactiveSession::restore(PROGRAM, &save).unwrap();
    assert_eq!(resumed.snapshot().value_grounds["level"], *grounds);
    assert_eq!(resumed.snapshot().qualified_values["level"], *level);
}

#[test]
fn a_restored_number_is_the_same_number_to_the_last_bit() {
    // Time summed from 0.05 s ticks needs up to 17 significant digits, and a
    // parser that rounds the last one moves a `>= 45` test by a tick: the
    // round-6 fuzz found exactly that. Check every value on the way.
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    for tick in 0..200 {
        send(&mut game, "tick", &[("dt", 0.05)]);
        let snapshot = game.snapshot();
        let resumed = ReactiveSession::restore_json(PROGRAM, &game.save_json().unwrap()).unwrap();
        let restored = resumed.snapshot();
        assert_eq!(
            restored.values["now"].to_bits(),
            snapshot.values["now"].to_bits(),
            "tick {tick}: {} came back as {}",
            snapshot.values["now"],
            restored.values["now"]
        );
        assert_eq!(
            restored.elapsed.to_bits(),
            snapshot.elapsed.to_bits(),
            "tick {tick}"
        );
    }
}

#[test]
fn a_number_a_host_sends_arrives_exactly() {
    // A frame time computed by a host needs every digit too. Both numbers
    // come back one unit off from a parser that is not exact.
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    for dt in ["0.0013580246789999999", "0.0018518518349999998"] {
        game.dispatch_json("tick", &format!(r#"{{"dt": {dt}}}"#))
            .unwrap();
    }
    let expected = 0.0013580246789999999_f64 + 0.0018518518349999998_f64;
    assert_eq!(game.snapshot().values["now"].to_bits(), expected.to_bits());
}

/// The program of issue #43: one renewable evidence, renewed by one event.
const RENEWING: &str = r#"
claim safe;
evidence bite from "a bite";
renewable bite limit 3;
event re;
on re renew bite;
"#;

/// A save of `RENEWING` after `renewals` renewals.
fn renewed(renewals: usize) -> serde_json::Value {
    let mut game = ReactiveSession::from_source(RENEWING).unwrap();
    for _ in 0..renewals {
        send(&mut game, "re", &[]);
    }
    serde_json::to_value(game.save().unwrap()).unwrap()
}

/// The edited `save` must be refused with `expected`. An accepted one fails
/// with what its next renewal does.
fn refused(save: &serde_json::Value, expected: &str) {
    match ReactiveSession::restore_json(RENEWING, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(mut game) => panic!(
            "accepted with renewals {:?}; the next renewal gives {}",
            game.snapshot().renewals["bite"].occurrences,
            match game.dispatch_outcome_json("re", "{}") {
                Ok(outcome) => serde_json::to_value(outcome).unwrap()["outcome"].to_string(),
                Err(fatal) => serde_json::to_string(&fatal).unwrap(),
            }
        ),
    }
}

// Issue #43. The graph holds bite@2 but the renewals do not list it, so the
// program's `[bite]` would stay current and the next renewal would generate
// bite@2 again: a fatal collision on a later event instead of a refused save.
#[test]
fn a_renewal_occurrence_its_renewals_do_not_list_is_refused() {
    let save = renewed(1);
    assert_eq!(
        save["renewals"],
        serde_json::json!({"bite": ["bite", "bite@2"]})
    );
    let mut dropped = save.clone();
    dropped["renewals"] = serde_json::json!({});
    // The renewal's effect names bite@2 too; drop it so only the graph does.
    dropped["effects"] = serde_json::json!([]);
    refused(
        &dropped,
        "renewable bite occurrence bite@2 is not in its renewals",
    );
    let mut first = save.clone();
    first["renewals"]["bite"] = serde_json::json!(["bite"]);
    refused(
        &first,
        "renewable bite occurrence bite@2 is not in its renewals",
    );
    let mut shortened = renewed(2);
    shortened["renewals"]["bite"] = serde_json::json!(["bite", "bite@2"]);
    refused(
        &shortened,
        "renewable bite occurrence bite@3 is not in its renewals",
    );
}

// Issue #43, the variant: an occurrence past the declared limit of 3, which
// no renewal could make. Listed or not, the limit bounds it.
#[test]
fn a_renewal_occurrence_past_its_limit_is_refused() {
    let mut renamed = renewed(1);
    renamed["graph"]["nodes"][0]["name"] = "bite@9".into();
    renamed["renewals"] = serde_json::json!({});
    renamed["effects"] = serde_json::json!([]);
    refused(
        &renamed,
        "renewable bite occurrence bite@9 is not in its renewals",
    );
    let mut extra = renewed(1);
    extra["graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"kind": "occurrence", "name": "bite@4"}));
    refused(
        &extra,
        "renewable bite occurrence bite@4 is not in its renewals",
    );
    let mut listed = renewed(1);
    listed["graph"]["nodes"][0]["name"] = "bite@4".into();
    listed["renewals"]["bite"] = serde_json::json!(["bite", "bite@2", "bite@3", "bite@4"]);
    refused(
        &listed,
        "renewable bite occurrences do not match the program",
    );
}

// The renewals were already checked against the graph the other way; that
// stays as it was.
#[test]
fn renewals_the_graph_does_not_hold_in_order_are_refused() {
    let mut missing = renewed(1);
    missing["renewals"]["bite"] = serde_json::json!(["bite", "bite@2", "bite@3"]);
    refused(&missing, "bite@3 must name a declared evidence");
    let mut swapped = renewed(2);
    swapped["renewals"]["bite"] = serde_json::json!(["bite", "bite@3", "bite@2"]);
    refused(&swapped, "renewable bite occurrence bite@3 is out of order");
    let mut repeated = renewed(1);
    repeated["renewals"]["bite"] = serde_json::json!(["bite", "bite@2", "bite@2"]);
    refused(
        &repeated,
        "renewable bite occurrence bite@2 is out of order",
    );
    let mut long = renewed(2);
    long["renewals"]["bite"] = serde_json::json!(["bite", "bite@2", "bite@3", "bite@4"]);
    refused(&long, "renewable bite occurrences do not match the program");
}

/// A session resumed from `game`'s save must be `game`, and stay it through
/// the next events, a renewal among them.
fn resumes(game: &ReactiveSession) {
    let mut original = game.clone();
    let mut resumed = ReactiveSession::restore_json(PROGRAM, &game.save_json().unwrap())
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(resumed.snapshot(), original.snapshot());
    for (event, payload) in [
        ("regrow", "{}"),
        ("eat", "{}"),
        ("read", r#"{"x": -1}"#),
        ("regrow", "{}"),
    ] {
        let expected = serde_json::to_value(original.dispatch_outcome_json(event, payload));
        let actual = serde_json::to_value(resumed.dispatch_outcome_json(event, payload));
        assert_eq!(actual.unwrap(), expected.unwrap(), "after {event}");
    }
}

// Saves the runtime writes itself still restore: with no renewal, one,
// several and as many as the limit allows, between readings, revisions,
// reveals and scheduled caveats, and after the limit refused a renewal.
#[test]
fn every_save_of_a_renewing_session_restores() {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    resumes(&game);
    send(&mut game, "start", &[]);
    send(&mut game, "eat", &[]);
    resumes(&game);
    for renewal in 1..8 {
        send(&mut game, "regrow", &[]);
        resumes(&game);
        send(&mut game, "read", &[("x", renewal as f64)]);
        send(&mut game, "eat", &[]);
        wait(&mut game, 5);
        resumes(&game);
    }
    send(&mut game, "check", &[]);
    let snapshot = game.snapshot();
    assert_eq!(snapshot.renewals["bite"].occurrences.len(), 8);
    assert!(
        snapshot.scheduled_qualifications.len() < 8,
        "some have fired"
    );
    let outcome = serde_json::to_value(game.dispatch_outcome_json("regrow", "{}")).unwrap();
    assert_eq!(outcome["Ok"]["code"], "renewal_limit", "{outcome}");
    resumes(&game);
}

/// What `next` does to a restored session, event by event, up to the first
/// fatal outcome.
fn outcomes(game: &mut ReactiveSession, next: &[(&str, &str)]) -> String {
    let mut outcomes = Vec::new();
    for (event, payload) in next {
        match game.dispatch_outcome_json(event, payload) {
            Ok(outcome) => {
                let outcome = serde_json::to_value(outcome).unwrap();
                outcomes.push(format!("{event} {}", outcome["outcome"]));
            }
            Err(fatal) => {
                outcomes.push(format!("{event} fatal: {}", fatal.message));
                break;
            }
        }
    }
    outcomes.join("; ")
}

/// `save` of `source` must be refused with `expected`. An accepted one fails
/// with what `next` then does.
fn not_restored(source: &str, save: &serde_json::Value, expected: &str, next: &[(&str, &str)]) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(mut game) => panic!("{expected}: accepted; {}", outcomes(&mut game, next)),
    }
}

/// The events after a `PROGRAM` save: a read reopens and revises route,
/// which reads what route@3 retains and relies on.
const PLAY_ON: [(&str, &str); 5] = [
    ("tick", r#"{"dt": 0.1}"#),
    ("read", r#"{"x": 1}"#),
    ("eat", "{}"),
    ("regrow", "{}"),
    ("check", "{}"),
];

/// `played()`'s save with relation `index` replaced by `relation`, written
/// `FROM RELATION TO`.
fn related(index: usize, relation: &str) -> serde_json::Value {
    let mut save = serde_json::to_value(played().save().unwrap()).unwrap();
    save["graph"]["relations"][index] = relation.split(' ').collect();
    save
}

// An event adds a relation only between the kinds of node its effect names:
// a reading or revealed evidence supports or opposes a claim; a caveat
// qualifies evidence; a commitment retains caveats and relies on evidence;
// evidence reopens a commitment; nothing adds `in_context`. Restore used to
// relate any two nodes the save named. Read back as what they were written
// as, a retained evidence or a relied-on claim then made route's next
// revision fatal ("safe must name a declared evidence").
#[test]
fn a_relation_between_kinds_no_event_relates_is_refused() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    for (index, relation) in [
        (3, "flow@1 reopens route@1"),
        (9, "route@3 retains unmeasured"),
        (12, "route@3 relies_on forecast"),
        (13, "bite supports safe"),
        (15, "faded qualifies bite"),
    ] {
        let written: serde_json::Value = relation.split(' ').collect();
        assert_eq!(save["graph"]["relations"][index], written);
    }
    // Which relation is replaced, by what, and why that is refused.
    for case in [
        "12 route@3 relies_on safe: safe is a claim, not evidence",
        "12 route@3 relies_on route@1: route@1 is a commitment, not evidence",
        "12 route@3 relies_on stale: stale is a caveat, not evidence",
        "12 flow@1 relies_on forecast: flow@1 is evidence, not a commitment",
        "9 route@3 retains bite: bite is evidence, not a caveat",
        "9 route@3 retains route@1: route@1 is a commitment, not a caveat",
        "9 safe retains unmeasured: safe is a claim, not a commitment",
        "3 route@2 reopens route@1: route@2 is a commitment, not evidence",
        "3 flow@1 reopens safe: safe is a claim, not a commitment",
        "13 safe supports safe: safe is a claim, not evidence",
        "13 route@1 supports safe: route@1 is a commitment, not evidence",
        "13 bite supports route@1: route@1 is a commitment, not a claim",
        "13 bite opposes stale: stale is a caveat, not a claim",
        "15 bite qualifies bite@2: bite is evidence, not a caveat",
        "15 faded qualifies safe: safe is a claim, not evidence",
        "15 faded qualifies stale: stale is a caveat, not evidence",
        "15 faded qualifies route@1: route@1 is a commitment, not evidence",
    ] {
        let (index, refusal) = case.split_once(' ').unwrap();
        let (relation, _) = refusal.split_once(':').unwrap();
        not_restored(
            PROGRAM,
            &related(index.parse().unwrap(), relation),
            &format!("relation {refusal}"),
            &PLAY_ON,
        );
    }
    not_restored(
        PROGRAM,
        &related(13, "bite in_context safe"),
        "relation bite in_context safe is not one an event adds",
        &PLAY_ON,
    );
}

// A commitment relies on evidence, and evidence reopens one, only once it is
// observed: something it supports or opposes. The next revision relies on
// route@3's evidence again and requires that, so evidence nothing observes
// made it fatal ("commitment basis includes unobserved evidence sensor").
#[test]
fn a_relation_to_evidence_nothing_observes_is_refused() {
    not_restored(
        PROGRAM,
        &related(12, "route@3 relies_on sensor"),
        "relation route@3 relies_on sensor: sensor is not observed",
        &PLAY_ON,
    );
    // bite is observed only by the relation that reveals it.
    let mut unrevealed = related(12, "route@3 relies_on bite");
    unrevealed["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .remove(13);
    not_restored(
        PROGRAM,
        &unrevealed,
        "relation route@3 relies_on bite: bite is not observed",
        &PLAY_ON,
    );
    // The fuzz's form of it: the reveal made to start at a claim.
    let mut reversed = related(10, "route@3 relies_on bite");
    reversed["graph"]["relations"][13][0] = "safe".into();
    not_restored(
        PROGRAM,
        &reversed,
        "relation safe supports safe: safe is a claim, not evidence",
        &PLAY_ON,
    );
    // The journal already required a reopening's cause to be observed; the
    // relation is now refused before it is compared with the journal.
    not_restored(
        PROGRAM,
        &related(3, "sensor reopens route@1"),
        "relation sensor reopens route@1: sensor is not observed",
        &PLAY_ON,
    );
}

// The same fact for the records a commitment reads again: any lineage, basis
// or guard that cites evidence nothing observes. route@3's basis and route's
// selection guards enter its next revision.
#[test]
fn a_record_citing_evidence_nothing_observes_is_refused() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    let mut basis = save.clone();
    basis["commitment_bases"]["route@3"]["provenance"]["evidence"] =
        serde_json::json!(["flow@1", "flow@2", "forecast", "sensor"]);
    not_restored(
        PROGRAM,
        &basis,
        "commitment route@3: sensor is not observed",
        &PLAY_ON,
    );
    let mut selection = save.clone();
    selection["decision_series"]["route"]["selection_qualifications"]["evidence"] =
        serde_json::json!(["sensor"]);
    not_restored(
        PROGRAM,
        &selection,
        "decision series route: sensor is not observed",
        &PLAY_ON,
    );
    let mut lineage = save.clone();
    lineage["states"]["support"]["lineage"]["evidence"] = serde_json::json!(["forecast", "sensor"]);
    not_restored(
        PROGRAM,
        &lineage,
        "state support: sensor is not observed",
        &PLAY_ON,
    );
}

/// A decision made on a state, and a withdrawal a rule reads.
const CITING: &str = r#"
claim safe;
claim misreading;
evidence seen from "seen";
evidence unseen from "never seen";
evidence plain from "a plain reading";
evidence recheck from "a second look";
readings flow from plain limit 4;
state level = 0;
state flagged = 0;
event see;
event rd x min -10 max 10;
event misread;
event decide;
event ask;
on see reveal seen supports safe;
on see set level = qualified(1, seen);
on rd sample flow = x supports safe;
on misread reveal recheck supports misreading;
on misread withdraw latest(flow) because recheck;
on decide commit go because enough using level;
on ask when withdrawn(latest(flow)) set flagged = 1;
"#;

// A state's lineage and a withdrawal's reason, citing evidence nothing
// observes: the commitment made on the state, and the rule that reads the
// withdrawal, were fatal.
#[test]
fn a_lineage_or_withdrawal_citing_evidence_nothing_observes_is_refused() {
    let mut game = ReactiveSession::from_source(CITING).unwrap();
    for (event, payload) in [("see", "{}"), ("rd", r#"{"x": 1}"#), ("misread", "{}")] {
        game.dispatch_json(event, payload).unwrap();
    }
    let save = serde_json::to_value(game.save().unwrap()).unwrap();
    assert_eq!(
        save["graph"]["relations"][0],
        serde_json::json!(["seen", "supports", "safe"])
    );
    assert_eq!(save["withdrawals"][0]["because"], "recheck");
    let next = [("decide", "{}"), ("ask", "{}")];
    let mut unseen = save.clone();
    unseen["states"]["level"]["lineage"]["evidence"] = serde_json::json!(["seen", "unseen"]);
    not_restored(
        CITING,
        &unseen,
        "state level: unseen is not observed",
        &next,
    );
    let mut unrevealed = save.clone();
    unrevealed["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    not_restored(
        CITING,
        &unrevealed,
        "state level: seen is not observed",
        &next,
    );
    let mut reason = save.clone();
    reason["withdrawals"][0]["because"] = "unseen".into();
    not_restored(
        CITING,
        &reason,
        "the withdrawal of flow@1: unseen is not observed",
        &[("ask", "{}")],
    );
}

/// A withdrawal a later event makes again.
const REWITHDRAWING: &str = r#"
claim safe;
evidence seen from "seen";
evidence reason from "reason";
event see;
event wd;
event again;
on see reveal seen supports safe;
on see reveal reason supports safe;
on wd withdraw seen because reason;
on again when withdrawn(seen) withdraw seen because reason;
"#;

// Withdrawn evidence was observed when it was withdrawn, and withdrawing it
// again requires that: with nothing observing it, the next withdrawal of it
// was fatal ("cannot withdraw unobserved evidence seen").
#[test]
fn a_withdrawal_of_evidence_nothing_observes_is_refused() {
    let mut game = ReactiveSession::from_source(REWITHDRAWING).unwrap();
    for event in ["see", "wd", "again"] {
        game.dispatch_json(event, "{}").unwrap();
    }
    let save = serde_json::to_value(game.save().unwrap()).unwrap();
    assert_eq!(
        save["graph"]["relations"][0],
        serde_json::json!(["seen", "supports", "safe"])
    );
    let mut resumed = ReactiveSession::restore_json(REWITHDRAWING, &save.to_string()).unwrap();
    assert_eq!(
        outcomes(&mut resumed, &[("again", "{}")]),
        r#"again "accepted""#
    );
    let mut unseen = save.clone();
    unseen["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    not_restored(
        REWITHDRAWING,
        &unseen,
        "the withdrawal of seen: seen is not observed",
        &[("again", "{}")],
    );
}

// A commitment retains exactly its basis's caveats and relies on exactly its
// evidence. Reading `committed(...)` or `reopened(...)` adds what it retains
// and relies on to that basis: anything more changed what the next revision
// was made on.
#[test]
fn a_relation_beyond_a_commitments_basis_is_refused() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    let basis = &save["commitment_bases"]["route@3"]["provenance"];
    assert_eq!(basis["caveats"], serde_json::json!(["unmeasured"]));
    assert_eq!(
        basis["evidence"],
        serde_json::json!(["flow@1", "flow@2", "forecast"])
    );
    for case in [
        "9 route@3 retains faded: faded is not in route@3's basis",
        "12 route@3 relies_on bite: bite is not in route@3's basis",
        "9 route@1 retains stale: stale is not in route@1's basis",
        "12 route@1 relies_on flow@2: flow@2 is not in route@1's basis",
    ] {
        let (index, refusal) = case.split_once(' ').unwrap();
        let (relation, _) = refusal.split_once(':').unwrap();
        not_restored(
            PROGRAM,
            &related(index.parse().unwrap(), relation),
            &format!("relation {refusal}"),
            &PLAY_ON,
        );
    }
}

// The same with long names: relying on more than its basis took the read of
// `committed(go)` past the provenance limit of 65536 name bytes, which was
// fatal.
#[test]
fn a_commitment_relying_on_more_than_its_basis_is_refused() {
    let names = (0..4)
        .map(|index| format!("e{index}{}", "x".repeat(20_000)))
        .collect::<Vec<_>>();
    let mut source =
        String::from("claim safe;\nstate s = 0;\nevent see;\nevent decide;\nevent poke;\n");
    for name in &names {
        source.push_str(&format!(
            "evidence {name} from \"long\";\non see reveal {name} supports safe;\n"
        ));
    }
    source.push_str("on decide commit go because enough;\n");
    source.push_str("on poke when committed(go) set s = s + 1;\n");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["see", "decide", "poke"] {
        game.dispatch_json(event, "{}").unwrap();
    }
    let mut save = serde_json::to_value(game.save().unwrap()).unwrap();
    let relations = save["graph"]["relations"].as_array_mut().unwrap();
    for name in &names {
        relations.push(serde_json::json!(["go", "relies_on", name]));
    }
    let first = &names[0];
    not_restored(
        &source,
        &save,
        &format!("relation go relies_on {first}: {first} is not in go's basis"),
        &[("poke", "{}")],
    );
}

/// Every relation an event adds: readings that support and oppose, a reveal,
/// caveats a reading inherits, a renewal carries, a late qualification adds
/// and a withdrawal adds, retained caveats, evidence relied on, and
/// reopenings by name and by reading.
const RELATING: &str = r#"
claim safe;
claim misreading;
evidence plain from "a plain reading";
evidence bite from "a bite";
evidence recheck from "a second look";
caveat noisy consequence low;
caveat faded consequence low;
caveat stale consequence material;
noisy qualifies plain;
faded qualifies bite;
readings flow from plain limit 8;
renewable bite limit 4;
decisions go limit 6;
state level = 0;
event rd x min -10 max 10;
event low x min -10 max 10;
event eat;
event regrow;
event decide;
event doubt;
event recall;
event revise;
event age;
event misread;
on rd sample flow = x supports safe;
on low sample flow = x opposes safe;
on eat reveal bite supports safe;
on eat set level = level + qualified(1, bite);
on regrow renew bite;
on decide when not committed(go) commit go because enough using latest(flow) retaining stale;
on doubt when committed(go) and not reopened(go) reopen go because latest(flow);
on recall when committed(go) and observed(bite) reopen go because bite;
on revise when reopened(go) commit go because enough using latest(flow) + level;
on age qualify bite with stale;
on misread when not observed(recheck) reveal recheck supports misreading;
on misread withdraw latest(flow) because recheck;
"#;

// Saves the runtime writes restore, whatever relations their events added,
// and the restored session plays on identically.
#[test]
fn every_relation_an_event_adds_restores() {
    let mut game = ReactiveSession::from_source(RELATING).unwrap();
    let steps = [
        ("rd", r#"{"x": 1}"#),
        ("decide", "{}"),
        ("eat", "{}"),
        ("doubt", "{}"),
        ("revise", "{}"),
        ("recall", "{}"),
        ("regrow", "{}"),
        ("low", r#"{"x": -2}"#),
        ("eat", "{}"),
        ("revise", "{}"),
        ("age", "{}"),
        ("misread", "{}"),
        ("doubt", "{}"),
        ("revise", "{}"),
    ];
    for (step, (event, payload)) in steps.iter().enumerate() {
        let outcome = game
            .dispatch_outcome_json(event, payload)
            .unwrap_or_else(|fatal| panic!("{event}: fatal {}", fatal.message));
        let outcome = serde_json::to_value(outcome).unwrap();
        assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
        let mut original = game.clone();
        let mut resumed = ReactiveSession::restore_json(RELATING, &game.save_json().unwrap())
            .unwrap_or_else(|error| panic!("after {event}: {error}"));
        assert_eq!(resumed.snapshot(), original.snapshot(), "after {event}");
        for (event, payload) in &steps[step + 1..] {
            let expected = serde_json::to_value(original.dispatch_outcome_json(event, payload));
            let actual = serde_json::to_value(resumed.dispatch_outcome_json(event, payload));
            assert_eq!(actual.unwrap(), expected.unwrap(), "after {event}");
        }
        assert_eq!(resumed.snapshot(), original.snapshot());
    }
    let save = game.save().unwrap();
    let mut added = save
        .graph
        .relations
        .iter()
        .map(|[from, relation, to]| {
            let base = |name: &str| name.split('@').next().unwrap().to_string();
            format!("{} {relation} {}", base(from), base(to))
        })
        .collect::<Vec<_>>();
    added.sort();
    added.dedup();
    assert_eq!(
        added,
        [
            "bite reopens go",
            "bite supports safe",
            "faded qualifies bite",
            "flow opposes safe",
            "flow reopens go",
            "flow supports safe",
            "go relies_on bite",
            "go relies_on flow",
            "go retains faded",
            "go retains noisy",
            "go retains stale",
            "go retains withdrawn",
            "noisy qualifies flow",
            "recheck supports misreading",
            "stale qualifies bite",
            "withdrawn qualifies flow",
        ]
    );
}
