//! Saving a reactive session and resuming it without replaying events. See
//! spec/caveat-save-0.1.md.
use caveat_runtime::reactive::{ReactiveSave, ReactiveSession};

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

/// `path` as a JSON pointer.
fn pointer(path: &[serde_json::Value]) -> String {
    path.iter()
        .map(|step| match step {
            serde_json::Value::String(key) => {
                format!("/{}", key.replace('~', "~0").replace('/', "~1"))
            }
            other => format!("/{other}"),
        })
        .collect()
}

/// The place at `path`, if an earlier alteration has not removed it.
fn at<'a>(
    value: &'a mut serde_json::Value,
    path: &[serde_json::Value],
) -> Option<&'a mut serde_json::Value> {
    value.pointer_mut(&pointer(path))
}

/// Take the place at `path` out of its object or array, if it is still there.
fn remove(value: &mut serde_json::Value, path: &[serde_json::Value]) {
    let Some((last, parent)) = path.split_last() else {
        return;
    };
    match (at(value, parent), last) {
        (Some(serde_json::Value::Object(fields)), serde_json::Value::String(key)) => {
            fields.remove(key);
        }
        (Some(serde_json::Value::Array(items)), serde_json::Value::Number(index)) => {
            if let Some(index) = index.as_u64().filter(|index| *index < items.len() as u64) {
                items.remove(index as usize);
            }
        }
        _ => {}
    }
}

/// What a restored session must then get through, each event accepted or
/// rejected as a value.
const NEXT_EVENTS: [(&str, &str); 6] = [
    ("tick", r#"{"dt": 0.1}"#),
    ("read", r#"{"x": 1}"#),
    ("eat", "{}"),
    ("regrow", "{}"),
    ("check", "{}"),
    ("start", "{}"),
];

// The fix each known fatal outcome waits for, and the confirmed runtime bug
// behind it.
const LATE_CAVEAT_EXPLANATION: &str = "waits for fix/binding-explanation-late-caveat; remove it \
    when that fix is in the tested combination. When the late `faded` reaches bite@2, \
    hud.text's explanation is fatal, save or no save: tick played() by 0.1 to 40 seconds";
const RELATION_KINDS: &str = "waits for fix/restore-relation-kinds; remove it when that fix is \
    in the tested combination. Restore does not check the kinds a created relation connects: \
    with route@3 relies_on safe, or on evidence nothing observes, the next revision of route \
    is fatal";

/// The seed the save fuzz runs with unless SAVE_FUZZ_SEED names another.
const DEFAULT_SEED: u32 = 20_260_922;

/// A fatal outcome an accepted save still leads to: a confirmed runtime bug
/// with its own fix under way.
struct KnownFatal {
    /// The event that is fatal.
    event: &'static str,
    /// Its exact, full message.
    message: &'static str,
    /// The fix it waits for, and the bug.
    waits: &'static str,
    /// Its witness, the altered save that reproduces it: round `round`
    /// (counted from 0) of the save fuzz run with seed `seed`.
    seed: u32,
    round: usize,
}

/// Fatal outcomes an accepted save still leads to. A fatal outcome is skipped
/// only when an entry names its event and its whole message, so an entry
/// cannot hide any other fatal outcome; any other fails the test, like a
/// crash. These are every message the default seed and seeds 1 to 6 produce
/// in 30,000 rounds. Each witness is an earliest round found for its entry
/// running seeds 1 to 4,000 for 1,000 rounds each (seeds to 16,000 for the
/// last). Remove an entry when its fix is in the tested combination:
/// every_known_fatal_outcome_still_happens fails once its witness no longer
/// reproduces it.
const KNOWN_FATAL: [KnownFatal; 10] = [
    KnownFatal {
        event: "tick",
        message: "binding hud.text cites evidence bite, evidence forecast, caveat unmeasured \
                  that its value and conditions never read",
        waits: LATE_CAVEAT_EXPLANATION,
        seed: 1047,
        round: 0,
    },
    KnownFatal {
        event: "read",
        message: "event read, rule 5: route@1 must name a declared evidence",
        waits: RELATION_KINDS,
        seed: 3199,
        round: 19,
    },
    KnownFatal {
        event: "read",
        message: "event read, rule 5: safe must name a declared evidence",
        waits: RELATION_KINDS,
        seed: 3491,
        round: 11,
    },
    KnownFatal {
        event: "read",
        message: "event read, rule 5: stale must name a declared evidence",
        waits: RELATION_KINDS,
        seed: 336,
        round: 0,
    },
    KnownFatal {
        event: "read",
        message: "event read, rule 5: bite must name a declared caveat",
        waits: RELATION_KINDS,
        seed: 148,
        round: 9,
    },
    KnownFatal {
        event: "read",
        message: "event read, rule 5: bite@2 must name a declared caveat",
        waits: RELATION_KINDS,
        seed: 2948,
        round: 4,
    },
    KnownFatal {
        event: "read",
        message: "event read, rule 5: flow@1 must name a declared caveat",
        waits: RELATION_KINDS,
        seed: 59,
        round: 17,
    },
    KnownFatal {
        event: "read",
        message: "event read, rule 5: route@1 must name a declared caveat",
        waits: RELATION_KINDS,
        seed: 1247,
        round: 7,
    },
    KnownFatal {
        event: "read",
        message: "event read, rule 5: safe must name a declared caveat",
        waits: RELATION_KINDS,
        seed: 1520,
        round: 3,
    },
    KnownFatal {
        event: "read",
        message: "event read, rule 5: commitment basis includes unobserved evidence bite",
        waits: RELATION_KINDS,
        seed: 15_978,
        round: 216,
    },
];

/// The `KNOWN_FATAL` entry for a fatal `message` on `event`, if one names both.
fn known_fatal(event: &str, message: &str) -> Option<usize> {
    KNOWN_FATAL
        .iter()
        .position(|known| known.event == event && known.message == message)
}

// An entry is one whole fatal outcome, so it can never hide another: the same
// message on another event, or a message that contains it or that it
// contains, is not skipped.
#[test]
fn a_known_fatal_outcome_is_its_event_and_whole_message() {
    for (index, known) in KNOWN_FATAL.iter().enumerate() {
        let KnownFatal {
            event,
            message,
            waits,
            ..
        } = known;
        assert!(waits.starts_with("waits for "), "{message}: {waits}");
        assert_eq!(known_fatal(event, message), Some(index), "{message}");
        assert_eq!(known_fatal("eat", message), None, "{message}");
        assert_eq!(known_fatal(event, &format!("{message}.")), None);
        assert_eq!(known_fatal(event, &message[1..]), None);
    }
    for (event, message) in [
        (
            "read",
            "event read, rule 5: route@2 must name a declared evidence",
        ),
        (
            "read",
            "event read, rule 5: commitment basis includes unobserved evidence forecast",
        ),
        (
            "start",
            "event start, rule 2: current decision in go must be explicitly reopened before \
             revision",
        ),
        (
            "tick",
            "binding hud.level cites evidence bite that its value and conditions never read",
        ),
    ] {
        assert_eq!(known_fatal(event, message), None, "{message}");
    }
}

/// What a save came to when restored and played on.
#[derive(Debug, PartialEq)]
enum Resumed {
    /// Restore refused it, with this error.
    Refused(String),
    /// Restore accepted it, and each next event was accepted or rejected as a
    /// value.
    PlaysOn,
    /// Restore accepted it, and this event was fatal with this message.
    Fatal(&'static str, String),
}

/// Restore `text` and play `NEXT_EVENTS` on it, taking a snapshot, a view and
/// a save after each, up to the first fatal outcome, since a host discards a
/// session after one. `Err` if any of that panicked.
fn resume(text: &str) -> Result<Resumed, String> {
    let restored = std::panic::catch_unwind(|| ReactiveSession::restore_json(PROGRAM, text))
        .map_err(|_| "restoring it panicked".to_string())?;
    let mut game = match restored {
        Ok(game) => game,
        Err(error) => return Ok(Resumed::Refused(error)),
    };
    let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for (event, payload) in NEXT_EVENTS {
            if let Err(fatal) = game.dispatch_outcome_json(event, payload) {
                return Resumed::Fatal(event, fatal.message);
            }
            let _ = game.snapshot();
            let _ = serde_json::to_string(&game.view());
            let _ = game.save();
        }
        Resumed::PlaysOn
    }));
    run.map_err(|_| "a later event panicked".into())
}

/// Restore `text` and play `NEXT_EVENTS` on it: `Ok(false)` if it is refused,
/// `Ok(true)` if it is accepted and plays on. A known fatal outcome ends the
/// play and counts in `skipped`; any other fails, like a panic.
fn restore_and_play(text: &str, skipped: &mut [usize]) -> Result<bool, String> {
    match resume(text)? {
        Resumed::Refused(_) => Ok(false),
        Resumed::PlaysOn => Ok(true),
        Resumed::Fatal(event, message) => {
            let known = known_fatal(event, &message)
                .ok_or_else(|| format!("{event} was fatal: {message}"))?;
            skipped[known] += 1;
            Ok(true)
        }
    }
}

/// What the save fuzz writes in place of a value: names the save holds, names
/// past a limit, and names nothing declares.
const NAMES: [&str; 12] = [
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

/// The save fuzz's altered saves, one a round: one to three alterations of a
/// fresh copy of `save`, each a place replaced or removed. The generator runs
/// on from round to round, so a seed and a round name one altered save, the
/// same on every run, which `.nth(round)` rebuilds.
struct Alterations {
    save: serde_json::Value,
    places: Vec<Vec<serde_json::Value>>,
    random: Mulberry,
}

impl Alterations {
    fn new(save: &serde_json::Value, seed: u32) -> Self {
        let mut places = Vec::new();
        paths(save, Vec::new(), &mut places);
        Alterations {
            save: save.clone(),
            places,
            random: Mulberry(seed),
        }
    }
}

impl Iterator for Alterations {
    type Item = serde_json::Value;

    fn next(&mut self) -> Option<serde_json::Value> {
        let random = &mut self.random;
        let mut altered = self.save.clone();
        for _ in 0..1 + random.below(3) {
            let path = &self.places[random.below(self.places.len())];
            let alteration = random.below(8);
            if alteration == 7 {
                remove(&mut altered, path);
                continue;
            }
            let Some(target) = at(&mut altered, path) else {
                continue;
            };
            *target = match alteration {
                0 => serde_json::Value::Null,
                1 => NAMES[random.below(NAMES.len())].into(),
                2 => (random.below(2000) as f64 - 1000.0).into(),
                3 => serde_json::json!([]),
                4 => serde_json::json!({}),
                5 => serde_json::json!([NAMES[random.below(NAMES.len())]]),
                _ => u64::MAX.into(),
            };
        }
        Some(altered)
    }
}

#[test]
fn no_altered_save_can_crash_the_runtime() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    // SAVE_FUZZ_ROUNDS and SAVE_FUZZ_SEED run it longer or differently.
    let setting = |name: &str, default: u32| {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    };
    let rounds = setting("SAVE_FUZZ_ROUNDS", 3000);
    let seed = setting("SAVE_FUZZ_SEED", DEFAULT_SEED);
    let mut accepted = 0;
    let mut skipped = [0; KNOWN_FATAL.len()];
    let alterations = Alterations::new(&save, seed).take(rounds as usize);
    for (round, altered) in alterations.enumerate() {
        let text = altered.to_string();
        // Whatever was accepted must also keep running: no panic, and no
        // fatal outcome on the next events.
        match restore_and_play(&text, &mut skipped) {
            Ok(played) => accepted += u32::from(played),
            Err(failure) => panic!("seed {seed}, round {round}: {failure}: {text}"),
        }
    }
    eprintln!(
        "seed {seed}, {rounds} rounds: accepted {accepted}, refused {}; known fatal outcomes \
         skipped: {skipped:?}",
        rounds - accepted
    );
    // Some alterations are harmless (a changed number within range): make
    // sure the generator exercised both outcomes.
    assert!(accepted > 0 && accepted < rounds, "accepted {accepted}");
}

// An entry of KNOWN_FATAL waits for a fix and goes once that fix is in the
// tested combination. That is this test's to require, not anyone's memory:
// each entry's witness, rebuilt from its seed and round, must still be
// accepted and then fatal on exactly its event with exactly its message. A
// fix makes it no longer fatal, and this fails naming the entry to remove.
#[test]
fn every_known_fatal_outcome_still_happens() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    let mut fixed = Vec::new();
    for KnownFatal {
        event,
        message,
        waits,
        seed,
        round,
    } in &KNOWN_FATAL
    {
        let altered = Alterations::new(&save, *seed).nth(*round).unwrap();
        let now = match resume(&altered.to_string()) {
            Ok(Resumed::Fatal(on, fatal)) if on == *event && fatal == *message => continue,
            Ok(Resumed::Fatal(on, fatal)) => format!("is fatal on {on} instead: {fatal}"),
            Ok(Resumed::PlaysOn) => "plays on through every next event".to_string(),
            Ok(Resumed::Refused(error)) => format!("is refused: {error}"),
            Err(failure) => format!("panics: {failure}"),
        };
        fixed.push(format!(
            "- {event}: \"{message}\". Its witness, round {round} of seed {seed}, now {now}. \
             It {waits}"
        ));
    }
    assert!(
        fixed.is_empty(),
        "these known fatal outcomes no longer happen. Remove each from KNOWN_FATAL now that \
         the fix it waits for is in (if that fix is not in, find the entry a new witness):\n{}",
        fixed.join("\n")
    );
}

// Committing a decision already in force was a known fatal outcome until #53
// made it a rejection, and its entry left KNOWN_FATAL. Its witness, round 0
// of seed 1032, and the save it was altered from must now be refused as that
// rejection, with nothing kept, not merely play on.
#[test]
fn a_commit_in_force_after_a_restore_is_refused_as_decision_in_force() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    let witness = Alterations::new(&save, 1032).next().unwrap();
    for text in [save.to_string(), witness.to_string()] {
        let mut game = ReactiveSession::restore_json(PROGRAM, &text).expect("the save restores");
        for (event, payload) in NEXT_EVENTS {
            let before = serde_json::to_string(&game.save().unwrap()).unwrap();
            let outcome = game
                .dispatch_outcome_json(event, payload)
                .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
            if event != "start" {
                continue;
            }
            let outcome = serde_json::to_value(&outcome).unwrap();
            assert_eq!(outcome["outcome"], "rejected");
            assert_eq!(outcome["origin"], "evaluation");
            assert_eq!(outcome["code"], "decision_in_force");
            assert_eq!(
                outcome["message"],
                "event start, rule 2: current decision in route must be explicitly reopened \
                 before revision"
            );
            assert_eq!(
                serde_json::to_string(&game.save().unwrap()).unwrap(),
                before
            );
        }
    }
}

// A save whose sequence no session reaches was a known fatal outcome until
// fix/restore-sequence-bound bounded the sequence restore accepts, and its
// entry left KNOWN_FATAL. Its witness, round 1 of seed 2464, sets the sequence
// to u64::MAX, and its next tick could not be numbered. It must now be refused
// at restore with that bound, before any event, and for its sequence alone.
#[test]
fn a_save_past_the_sequence_bound_is_refused_before_any_event() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    let mut witness = Alterations::new(&save, 2464).nth(1).unwrap();
    assert_eq!(
        witness["sequence"],
        u64::MAX,
        "the witness must hold the sequence"
    );
    assert_eq!(
        resume(&witness.to_string()),
        Ok(Resumed::Refused(
            "cannot restore save: sequence must be at most 9007199254740991".to_string()
        ))
    );
    witness["sequence"] = save["sequence"].clone();
    assert_eq!(resume(&witness.to_string()), Ok(Resumed::PlaysOn));
}

// Every save missing one record, a list entry or a field, must be refused or
// play on. The fuzz above removes records too, but reaches any given one
// about once in 2,500 alterations. A stream missing its first reading, which
// restore used to accept and the next sample then failed on, is one of them.
#[test]
fn no_save_missing_a_record_can_crash_the_runtime() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    let mut places = Vec::new();
    paths(&save, Vec::new(), &mut places);
    let mut accepted = 0;
    let mut skipped = [0; KNOWN_FATAL.len()];
    // The first place is the whole save.
    for path in &places[1..] {
        let mut altered = save.clone();
        remove(&mut altered, path);
        let text = altered.to_string();
        match restore_and_play(&text, &mut skipped) {
            Ok(played) => accepted += usize::from(played),
            Err(failure) => panic!("without {}: {failure}: {text}", pointer(path)),
        }
    }
    eprintln!("known fatal outcomes skipped: {skipped:?}");
    assert!(
        accepted > 0 && accepted < places.len() - 1,
        "accepted {accepted}"
    );
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

/// One reading stream, sampled by one event.
const READING: &str = r#"
claim safe;
evidence plain from "a plain reading";
readings flow from plain limit 4;
event rd x min -10 max 10;
on rd sample flow = x supports safe;
"#;

/// A save of `READING` after `samples` samples.
fn sampled(samples: usize) -> serde_json::Value {
    let mut game = ReactiveSession::from_source(READING).unwrap();
    for sample in 1..=samples {
        send(&mut game, "rd", &[("x", sample as f64)]);
    }
    serde_json::to_value(game.save().unwrap()).unwrap()
}

/// The edited `save` must be refused with `expected`. An accepted one fails
/// with what its next sample does.
fn unsampled(save: &serde_json::Value, expected: &str) {
    match ReactiveSession::restore_json(READING, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(mut game) => panic!(
            "accepted with readings {:?}; the next sample gives {}",
            game.snapshot().reading_streams["flow"]
                .occurrences
                .iter()
                .map(|occurrence| occurrence.id.clone())
                .collect::<Vec<_>>(),
            match game.dispatch_outcome_json("rd", r#"{"x": 5}"#) {
                Ok(outcome) => serde_json::to_value(outcome).unwrap()["outcome"].to_string(),
                Err(fatal) => serde_json::to_string(&fatal).unwrap(),
            }
        ),
    }
}

// The graph holds flow@2 but the stream lists only flow@1, so the next
// sample would generate flow@2 again: a fatal collision on a later event
// instead of a refused save. The same for a stream that lists none of its
// readings, only its later one, or both in the wrong order.
#[test]
fn a_reading_its_stream_does_not_list_is_refused() {
    let save = sampled(2);
    let listed = |save: &serde_json::Value| {
        save["reading_streams"]["flow"]["occurrences"]
            .as_array()
            .unwrap()
            .iter()
            .map(|occurrence| occurrence["id"].clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(listed(&save), ["flow@1", "flow@2"]);
    let mut first = save.clone();
    first["reading_streams"]["flow"]["occurrences"]
        .as_array_mut()
        .unwrap()
        .truncate(1);
    first["reading_streams"]["flow"]["current"] = "flow@1".into();
    unsampled(
        &first,
        "reading stream flow occurrence flow@2 is not in its occurrences",
    );
    let mut none = save.clone();
    none["reading_streams"]["flow"]["occurrences"] = serde_json::json!([]);
    none["reading_streams"]["flow"]["current"] = serde_json::Value::Null;
    unsampled(
        &none,
        "reading stream flow occurrence flow@1 is not in its occurrences",
    );
    let mut second = save.clone();
    second["reading_streams"]["flow"]["occurrences"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    assert_eq!(listed(&second), ["flow@2"]);
    unsampled(
        &second,
        "reading stream flow occurrence flow@2 is out of order",
    );
    let mut swapped = save.clone();
    swapped["reading_streams"]["flow"]["occurrences"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    swapped["reading_streams"]["flow"]["current"] = "flow@1".into();
    unsampled(
        &swapped,
        "reading stream flow occurrence flow@2 is out of order",
    );
}

// A reading past the declared limit of 4, which no sample could make. Renamed
// everywhere the save names it, it is out of order; added to the graph alone,
// it is not listed; listed in order, the stream holds more than its limit.
#[test]
fn a_reading_past_its_limit_is_refused() {
    let renamed: serde_json::Value =
        serde_json::from_str(&sampled(2).to_string().replace("flow@2", "flow@9")).unwrap();
    assert_eq!(renamed["reading_streams"]["flow"]["current"], "flow@9");
    unsampled(
        &renamed,
        "reading stream flow occurrence flow@9 is out of order",
    );
    let mut extra = sampled(2);
    extra["graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"kind": "occurrence", "name": "flow@5"}));
    unsampled(
        &extra,
        "reading stream flow occurrence flow@5 is not in its occurrences",
    );
    let mut listed: serde_json::Value =
        serde_json::from_str(&sampled(4).to_string().replace("flow@4", "flow@5")).unwrap();
    let mut fourth = listed["reading_streams"]["flow"]["occurrences"][3].clone();
    fourth["id"] = "flow@4".into();
    listed["reading_streams"]["flow"]["occurrences"]
        .as_array_mut()
        .unwrap()
        .insert(3, fourth);
    unsampled(&listed, "reading stream flow does not match the program");
}

// The stream was already checked against the graph the other way; that stays
// as it was.
#[test]
fn readings_the_graph_does_not_hold_are_refused() {
    let mut missing = sampled(1);
    let mut second = missing["reading_streams"]["flow"]["occurrences"][0].clone();
    second["id"] = "flow@2".into();
    missing["reading_streams"]["flow"]["occurrences"]
        .as_array_mut()
        .unwrap()
        .push(second);
    missing["reading_streams"]["flow"]["current"] = "flow@2".into();
    unsampled(&missing, "flow@2 must name a declared evidence");
    let mut stale = sampled(2);
    stale["reading_streams"]["flow"]["current"] = "flow@1".into();
    unsampled(
        &stale,
        "reading stream flow current is not its latest reading",
    );
}

// Decision series have no such gap. Every commitment the graph holds needs a
// journal entry, the entry of a series needs the revision it names, and each
// revision must be ACTION@N at position N (check_saved_journal). So a series
// that leaves out a revision the graph holds, one renamed past the limit, or
// one only the graph holds was already refused, and still is.
#[test]
fn a_revision_its_series_does_not_list_is_refused() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    assert_eq!(save["decision_series"]["route"]["current"], "route@3");
    assert_eq!(save["decision_series"]["route"]["limit"], 16);
    let refused = |save: &serde_json::Value, expected: &str| {
        let error = ReactiveSession::restore_json(PROGRAM, &save.to_string())
            .map(|_| ())
            .unwrap_err();
        assert_eq!(error, format!("cannot restore save: {expected}"));
    };
    let mut unlisted = save.clone();
    unlisted["decision_series"]["route"]["revisions"]
        .as_array_mut()
        .unwrap()
        .pop();
    unlisted["decision_series"]["route"]["current"] = "route@2".into();
    refused(
        &unlisted,
        "decision journal: commitment does not belong to its decision series",
    );
    let mut unjournaled = unlisted.clone();
    let journal = unjournaled["decision_journal"].as_array_mut().unwrap();
    assert_eq!(journal.pop().unwrap()["commitment"], "route@3");
    refused(
        &unjournaled,
        "decision journal: created commitment has no journal entry",
    );
    // An entry whose decision is its own commitment is not looked up in a
    // series, but then no rule of its event commits it.
    let mut undecided = unlisted.clone();
    let journal = undecided["decision_journal"].as_array_mut().unwrap();
    journal.last_mut().unwrap()["decision"] = "route@3".into();
    refused(
        &undecided,
        "decision journal: event cannot make this decision change",
    );
    let renamed: serde_json::Value =
        serde_json::from_str(&save.to_string().replace("route@3", "route@17")).unwrap();
    refused(
        &renamed,
        "decision journal: revision identity/order has no matching commitment",
    );
    let mut extra = save.clone();
    extra["graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"kind": "commitment", "name": "route@9", "reason": "enough"}));
    refused(
        &extra,
        "decision journal: created commitment has no journal entry",
    );
}

/// Readings with what else can happen to them: identifiers, a withdrawal, a
/// decision reopened and revised on them, a late caveat, and their limit.
const SAMPLING: &str = r#"
identifiers limit 8;
claim safe;
claim misreading;
evidence plain from "a plain reading";
evidence recheck from "a second look";
caveat faded consequence low;
readings flow from plain limit 4;
decisions go limit 3;
event tick dt min 0 max 0.1;
event rd x min -10 max 10;
event tagged who id;
event decide;
event doubt;
event revise;
event misread;
on rd sample flow = x supports safe;
on tagged sample flow = who supports safe;
on decide when not committed(go) commit go because enough using latest(flow);
on doubt when committed(go) and not reopened(go) reopen go because latest(flow);
on revise when reopened(go) commit go because enough using latest(flow);
on misread when not observed(recheck) reveal recheck supports misreading;
on misread withdraw latest(flow) because recheck;
on misread qualify recheck with faded after 0.2;
"#;

/// A session of `SAMPLING` resumed from `game`'s save must be `game`, and
/// stay it through the next events: samples, a new identifier, a revision, a
/// withdrawal and the time a late caveat takes.
fn resumes_sampling(game: &ReactiveSession) {
    let mut original = game.clone();
    let mut resumed = ReactiveSession::restore_json(SAMPLING, &game.save_json().unwrap())
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(resumed.snapshot(), original.snapshot());
    for (event, payload) in [
        ("rd", r#"{"x": -1}"#),
        ("tagged", r#"{"who": "b"}"#),
        ("decide", "{}"),
        ("doubt", "{}"),
        ("revise", "{}"),
        ("misread", "{}"),
        ("tick", r#"{"dt": 0.1}"#),
        ("tick", r#"{"dt": 0.1}"#),
        ("tick", r#"{"dt": 0.1}"#),
        ("rd", r#"{"x": 1}"#),
    ] {
        let expected = serde_json::to_value(original.dispatch_outcome_json(event, payload));
        let actual = serde_json::to_value(resumed.dispatch_outcome_json(event, payload));
        assert_eq!(actual.unwrap(), expected.unwrap(), "after {event}");
    }
    assert_eq!(resumed.snapshot(), original.snapshot());
}

// Saves the runtime writes itself still restore: with no reading, one,
// several and as many as the limit allows, a reading of an identifier, a
// withdrawn reading, a decision reopened and revised on readings, a late
// caveat pending and fired, and after the limits refused a sample and a
// revision.
#[test]
fn every_save_of_a_sampling_session_restores() {
    let mut game = ReactiveSession::from_source(SAMPLING).unwrap();
    resumes_sampling(&game);
    for (event, payload, expected) in [
        ("rd", r#"{"x": 1}"#, "accepted"),
        ("decide", "{}", "accepted"),
        ("tagged", r#"{"who": "a"}"#, "accepted"),
        ("doubt", "{}", "accepted"),
        ("revise", "{}", "accepted"),
        ("misread", "{}", "accepted"),
        ("tick", r#"{"dt": 0.1}"#, "accepted"),
        ("tick", r#"{"dt": 0.1}"#, "accepted"),
        ("tick", r#"{"dt": 0.1}"#, "accepted"),
        ("rd", r#"{"x": 2}"#, "accepted"),
        ("doubt", "{}", "accepted"),
        ("revise", "{}", "accepted"),
        ("rd", r#"{"x": 3}"#, "accepted"),
        ("rd", r#"{"x": 4}"#, "history_limit"),
        ("doubt", "{}", "accepted"),
        ("revise", "{}", "history_limit"),
    ] {
        let outcome = game
            .dispatch_outcome_json(event, payload)
            .unwrap_or_else(|fatal| panic!("{event}: fatal {}", fatal.message));
        let outcome = serde_json::to_value(outcome).unwrap();
        let code = outcome.get("code").unwrap_or(&outcome["outcome"]);
        assert_eq!(code, expected, "{event}: {outcome}");
        resumes_sampling(&game);
    }
    let snapshot = game.snapshot();
    let readings = &snapshot.reading_streams["flow"];
    assert_eq!(readings.occurrences.len(), readings.limit);
    assert_eq!(snapshot.decision_series["go"].revisions.len(), 3);
    let save = game.save().unwrap();
    assert_eq!(save.identifiers, ["a"]);
    assert_eq!(save.withdrawals.len(), 1);
    assert!(save.scheduled_qualifications.is_empty(), "the caveat fired");
}

/// The edited `save` of `source` must be refused with `expected`. An accepted
/// one fails with what each of the `next` events does to it.
fn misordered(source: &str, save: &serde_json::Value, expected: &str, next: &[(&str, &str)]) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(game) => panic!(
            "accepted with readings {:?}; then {:?}",
            game.snapshot().reading_streams["flow"]
                .occurrences
                .iter()
                .map(|occurrence| occurrence.id.clone())
                .collect::<Vec<_>>(),
            next.iter()
                .map(|(event, payload)| {
                    let mut game = game.clone();
                    match game.dispatch_outcome_json(event, payload) {
                        Ok(outcome) => {
                            let outcome = serde_json::to_value(outcome).unwrap();
                            format!(
                                "{event}: {}",
                                outcome.get("code").unwrap_or(&outcome["outcome"])
                            )
                        }
                        Err(fatal) => format!("{event}: fatal {}", fatal.message),
                    }
                })
                .collect::<Vec<_>>()
        ),
    }
}

// Each entry of a stream's occurrences had only to name evidence, not to be
// STREAM@N at its position N. A stream that listed its template, other
// evidence, a reading twice or another stream's reading was accepted, and the
// next event on latest(flow) acted on that entry: qualifying or withdrawing
// evidence nothing had observed was fatal, and a reading listed twice counted
// twice toward the limit.
#[test]
fn a_stream_listing_other_than_its_readings_in_order_is_refused() {
    let mut game = ReactiveSession::from_source(SAMPLING).unwrap();
    send(&mut game, "rd", &[("x", 1.0)]);
    send(&mut game, "decide", &[]);
    let save = serde_json::to_value(game.save().unwrap()).unwrap();
    let next = [("doubt", "{}"), ("misread", "{}"), ("rd", r#"{"x": 2}"#)];
    for listed in ["plain", "recheck", "flow@1"] {
        let mut edited = save.clone();
        let flow = &mut edited["reading_streams"]["flow"];
        let mut entry = flow["occurrences"][0].clone();
        entry["id"] = listed.into();
        flow["occurrences"].as_array_mut().unwrap().push(entry);
        flow["current"] = listed.into();
        misordered(
            SAMPLING,
            &edited,
            &format!("reading stream flow occurrence {listed} is out of order"),
            &next,
        );
    }
    let mut game = ReactiveSession::from_source(SAMPLING).unwrap();
    send(&mut game, "rd", &[("x", 1.0)]);
    let mut repeated = serde_json::to_value(game.save().unwrap()).unwrap();
    let flow = &mut repeated["reading_streams"]["flow"];
    let first = flow["occurrences"][0].clone();
    flow["occurrences"] = serde_json::json!([first.clone(), first.clone(), first.clone(), first]);
    misordered(
        SAMPLING,
        &repeated,
        "reading stream flow occurrence flow@1 is out of order",
        &next,
    );
    let streams = format!(
        "{READING}readings other from plain limit 4;\non rd sample other = x supports safe;\n"
    );
    let mut game = ReactiveSession::from_source(&streams).unwrap();
    send(&mut game, "rd", &[("x", 1.0)]);
    let mut other = serde_json::to_value(game.save().unwrap()).unwrap();
    let entry = other["reading_streams"]["other"]["occurrences"][0].clone();
    assert_eq!(entry["id"], "other@1");
    let flow = &mut other["reading_streams"]["flow"];
    flow["occurrences"].as_array_mut().unwrap().push(entry);
    flow["current"] = "other@1".into();
    misordered(
        &streams,
        &other,
        "reading stream flow occurrence other@1 is out of order",
        &[("rd", r#"{"x": 2}"#)],
    );
}

/// The largest sequence a save can hold: 2^53 - 1.
const MOST_EVENTS: u64 = (1 << 53) - 1;

/// `played()`'s save with its sequence set to `sequence`, as JSON text.
fn at_sequence(sequence: u64) -> String {
    let mut save = serde_json::to_value(played().save().unwrap()).unwrap();
    save["sequence"] = sequence.into();
    save.to_string()
}

// A save at u64::MAX was accepted, and its next event was fatal: "reactive
// event sequence exhausted". No session accepts 2^53 events, let alone 2^64,
// so restore refuses a sequence past 2^53 - 1, and an accepted save numbers
// its next events.
#[test]
fn a_sequence_no_session_reaches_is_refused() {
    for sequence in [u64::MAX, u64::MAX - 1, 1 << 63, MOST_EVENTS + 1] {
        match ReactiveSession::restore_json(PROGRAM, &at_sequence(sequence)) {
            Err(error) => assert_eq!(
                error,
                format!("cannot restore save: sequence must be at most {MOST_EVENTS}")
            ),
            Ok(mut game) => panic!(
                "accepted a sequence of {sequence}; its next tick gives {}",
                match game.dispatch_outcome_json("tick", r#"{"dt": 0.1}"#) {
                    Ok(outcome) => serde_json::to_value(outcome).unwrap()["outcome"].to_string(),
                    Err(fatal) => serde_json::to_string(&fatal).unwrap(),
                }
            ),
        }
    }
}

// The largest sequence restores, and its next events are numbered past it
// like any others.
#[test]
fn a_save_at_the_largest_sequence_numbers_its_next_events() {
    let mut game = ReactiveSession::restore_json(PROGRAM, &at_sequence(MOST_EVENTS))
        .unwrap_or_else(|error| panic!("{error}"));
    let mut sequence = MOST_EVENTS;
    for (event, payload) in [
        ("tick", r#"{"dt": 0.1}"#),
        ("read", r#"{"x": 1}"#),
        ("eat", "{}"),
        ("regrow", "{}"),
        ("check", "{}"),
        ("tick", r#"{"dt": 0.1}"#),
    ] {
        let outcome = game
            .dispatch_outcome_json(event, payload)
            .unwrap_or_else(|fatal| panic!("{event}: {}", fatal.message));
        let outcome = serde_json::to_value(outcome).unwrap();
        if outcome["outcome"] == "accepted" {
            sequence += 1;
        }
        assert_eq!(game.snapshot().sequence, sequence, "after {event}");
    }
    assert!(sequence > MOST_EVENTS + 1, "the events must be accepted");
}

// Only an edited save resumes a session at the bound. Its save there
// restores; once its events pass the bound, its saves are refused like any
// other past it, and it plays on.
#[test]
fn a_session_resumed_at_the_bound_saves_past_it_only_to_be_refused() {
    let mut game = ReactiveSession::restore_json(PROGRAM, &at_sequence(MOST_EVENTS))
        .unwrap_or_else(|error| panic!("{error}"));
    ReactiveSession::restore_json(PROGRAM, &game.save_json().unwrap())
        .unwrap_or_else(|error| panic!("its save at the bound: {error}"));
    for past in 1..=2 {
        game.dispatch_outcome_json("tick", r#"{"dt": 0.1}"#)
            .unwrap_or_else(|fatal| panic!("tick: {}", fatal.message));
        assert_eq!(game.snapshot().sequence, MOST_EVENTS + past);
        assert_eq!(
            ReactiveSession::restore_json(PROGRAM, &game.save_json().unwrap()).map(|_| ()),
            Err(format!(
                "cannot restore save: sequence must be at most {MOST_EVENTS}"
            )),
            "its save {past} past the bound"
        );
    }
}

// A reading records the sequence of the event that took it, from 1 to the
// save's own; the journal's entries and withdrawals were already held to that.
#[test]
fn a_reading_dated_outside_the_session_is_refused() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    let sequence = save["sequence"].as_u64().unwrap();
    let readings = save["reading_streams"]["flow"]["occurrences"]
        .as_array()
        .unwrap();
    assert_eq!(readings.len(), 2, "the fixture must hold readings");
    for dated in [0, sequence + 1, MOST_EVENTS, u64::MAX] {
        let mut edited = save.clone();
        edited["reading_streams"]["flow"]["occurrences"][1]["sequence"] = dated.into();
        assert_eq!(
            ReactiveSession::restore_json(PROGRAM, &edited.to_string()).map(|_| ()),
            Err("cannot restore save: reading flow@2 is out of sequence".to_string()),
            "a reading dated {dated} in a save of {sequence}"
        );
    }
    // The bound is all this checks: a reading dated by any event in the
    // session is accepted, as a reading of the last event is.
    for dated in [1, sequence] {
        let mut edited = save.clone();
        edited["reading_streams"]["flow"]["occurrences"][1]["sequence"] = dated.into();
        ReactiveSession::restore_json(PROGRAM, &edited.to_string())
            .unwrap_or_else(|error| panic!("a reading dated {dated}: {error}"));
    }
}
