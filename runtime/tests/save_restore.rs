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

/// Fatal outcomes an accepted save still leads to: none now. The last ones
/// ended with fix/restore-sequence-bound and fix/restore-relation-kinds, and
/// the default seed and seeds 1 to 7 produce none in 30,000 rounds. A fatal
/// outcome is skipped only when an entry names its event and its whole
/// message, so an entry cannot hide any other fatal outcome; any other fails
/// the test, like a crash. An entry is a confirmed runtime bug with its own
/// fix under way, and its witness an earliest round found for it. Remove an
/// entry when its fix is in the tested combination:
/// every_known_fatal_outcome_still_happens fails once its witness no longer
/// reproduces it.
const KNOWN_FATAL: [KnownFatal; 0] = [];

/// The entry of `known` for a fatal `message` on `event`, if one names both.
fn known_fatal(known: &[KnownFatal], event: &str, message: &str) -> Option<usize> {
    known
        .iter()
        .position(|entry| entry.event == event && entry.message == message)
}

// An entry is one whole fatal outcome, so it can never hide another: the same
// message on another event, or a message that contains it or that it
// contains, is not skipped. KNOWN_FATAL holds no entry now, so an example,
// the entry fix/restore-sequence-bound ended, keeps that checked for the next.
#[test]
fn a_known_fatal_outcome_is_its_event_and_whole_message() {
    let example = [KnownFatal {
        event: "tick",
        message: "reactive event sequence exhausted",
        waits: "waits for fix/restore-sequence-bound",
        seed: 2464,
        round: 1,
    }];
    for list in [&KNOWN_FATAL[..], &example[..]] {
        for (index, known) in list.iter().enumerate() {
            let KnownFatal {
                event,
                message,
                waits,
                ..
            } = known;
            assert!(waits.starts_with("waits for "), "{message}: {waits}");
            assert_eq!(known_fatal(list, event, message), Some(index), "{message}");
            assert_eq!(known_fatal(list, "eat", message), None, "{message}");
            assert_eq!(known_fatal(list, event, &format!("{message}.")), None);
            assert_eq!(known_fatal(list, event, &message[1..]), None);
        }
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
        assert_eq!(known_fatal(&KNOWN_FATAL, event, message), None, "{message}");
        assert_eq!(known_fatal(&example, event, message), None, "{message}");
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
            let known = known_fatal(&KNOWN_FATAL, event, &message)
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
// made it a rejection, and its entry left KNOWN_FATAL. The save its witness,
// round 0 of seed 1032, was altered from must now be refused as that
// rejection, with nothing kept, not merely play on. The witness itself drops
// forecast from route@1's basis while route@1 still relies on it, and since
// fix/restore-relation-kinds restore refuses it for that.
#[test]
fn a_commit_in_force_after_a_restore_is_refused_as_decision_in_force() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    let witness = Alterations::new(&save, 1032).next().unwrap();
    assert_eq!(
        resume(&witness.to_string()),
        Ok(Resumed::Refused(
            "cannot restore save: relation route@1 relies_on forecast: forecast is not in \
             route@1's basis"
                .into()
        ))
    );
    let mut game =
        ReactiveSession::restore_json(PROGRAM, &save.to_string()).expect("the save restores");
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

// A relation between kinds of node no event relates, and a relation to
// evidence nothing observes, were known fatal outcomes on the next revision
// of route until fix/restore-relation-kinds made restore refuse such a save,
// and their entries left KNOWN_FATAL. Each witness, by its seed and round,
// must now be refused at restore with the relation and why, not merely play
// on.
#[test]
fn a_relation_fatal_after_a_restore_is_refused_at_restore() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    // Each witness, the fatal outcome it led to, and the refusal it now meets.
    for (seed, round, fatal, refusal) in [
        (
            3199,
            19,
            "route@1 must name a declared evidence",
            "route@3 relies_on route@1: route@1 is a commitment, not evidence",
        ),
        (
            3491,
            11,
            "safe must name a declared evidence",
            "route@3 relies_on safe: safe is a claim, not evidence",
        ),
        (
            336,
            0,
            "stale must name a declared evidence",
            "route@3 relies_on stale: stale is a caveat, not evidence",
        ),
        (
            148,
            9,
            "bite must name a declared caveat",
            "route@3 retains bite: bite is evidence, not a caveat",
        ),
        (
            2948,
            4,
            "bite@2 must name a declared caveat",
            "route@3 retains bite@2: bite@2 is evidence, not a caveat",
        ),
        (
            59,
            17,
            "flow@1 must name a declared caveat",
            "route@3 retains flow@1: flow@1 is evidence, not a caveat",
        ),
        (
            1247,
            7,
            "route@1 must name a declared caveat",
            "route@3 retains route@1: route@1 is a commitment, not a caveat",
        ),
        (
            1520,
            3,
            "safe must name a declared caveat",
            "route@3 retains safe: safe is a claim, not a caveat",
        ),
        (
            15_978,
            216,
            "commitment basis includes unobserved evidence bite",
            "route@3 relies_on bite: bite is not observed",
        ),
    ] {
        let witness = Alterations::new(&save, seed).nth(round).unwrap();
        assert_eq!(
            resume(&witness.to_string()),
            Ok(Resumed::Refused(format!(
                "cannot restore save: relation {refusal}"
            ))),
            "round {round} of seed {seed}, once fatal on read: \"event read, rule 5: {fatal}\""
        );
    }
}

// A late caveat that made hud.text cite what its value and conditions never
// read was a known fatal outcome until fix/binding-explanation-late-caveat
// made it a rejection, and its entry left KNOWN_FATAL. Its witness, round 0
// of seed 1047, must now be refused on the tick as that rejection, with
// nothing kept, not merely play on.
#[test]
fn a_late_caveat_after_a_restore_is_refused_as_an_ungrounded_citation() {
    let save = serde_json::to_value(played().save().unwrap()).unwrap();
    let witness = Alterations::new(&save, 1047).next().unwrap();
    let mut game =
        ReactiveSession::restore_json(PROGRAM, &witness.to_string()).expect("the save restores");
    for (event, payload) in NEXT_EVENTS {
        let before = serde_json::to_string(&game.save().unwrap()).unwrap();
        let outcome = game
            .dispatch_outcome_json(event, payload)
            .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
        if event != "tick" {
            continue;
        }
        let outcome = serde_json::to_value(&outcome).unwrap();
        assert_eq!(outcome["outcome"], "rejected");
        assert_eq!(outcome["origin"], "evaluation");
        assert_eq!(outcome["code"], "ungrounded_citation");
        assert_eq!(
            outcome["message"],
            "binding hud.text cites evidence bite, evidence forecast, caveat unmeasured that its \
             value and conditions never read"
        );
        assert_eq!(
            serde_json::to_string(&game.save().unwrap()).unwrap(),
            before
        );
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

/// A state set by an event, and two bindings that show it: atan2 shows the
/// sign of a zero, -pi for -0.0 and pi for 0.0, though `-0.0 == 0.0`.
const SIGNED: &str = "state x = 0 min -1 max 1; event e v min -1 max 1; on e set x = v; \
                      bind hud.x = x; bind hud.angle = atan2(x, -1);";

fn is_negative_zero(value: f64) -> bool {
    value.to_bits() == (-0.0_f64).to_bits()
}

fn angle(game: &ReactiveSession) -> Option<f64> {
    game.snapshot().bindings["hud"]["angle"].as_number()
}

/// What a session shows, as JSON text, which keeps a zero's sign.
fn shown(game: &ReactiveSession) -> (String, String) {
    (
        serde_json::to_string(&game.snapshot()).unwrap(),
        serde_json::to_string(&game.view()).unwrap(),
    )
}

/// `game` resumed from its own save must show what it shows to the bit, and
/// save the same text again.
fn resumes_exactly(source: &str, game: &ReactiveSession) {
    let save = game.save_json().unwrap();
    let resumed = ReactiveSession::restore_json(source, &save).unwrap();
    assert_eq!(shown(&resumed), shown(game), "{save}");
    assert_eq!(resumed.save_json().unwrap(), save);
}

// A save leaves out a state still equal to what the program loaded, and it
// compared with `==`: a state holding -0.0 where the program loads 0 was left
// out, and came back as 0.0. So did a -0.0 the host never sent as "-0.0".
#[test]
fn a_negative_zero_survives_a_save() {
    for payload in [r#"{"v": -0.0}"#, r#"{"v": -0}"#, r#"{"v": -1e-400}"#] {
        let mut game = ReactiveSession::from_source(SIGNED).unwrap();
        game.dispatch_json("e", payload).unwrap();
        assert!(is_negative_zero(game.snapshot().values["x"]), "{payload}");
        assert!(is_negative_zero(game.save().unwrap().states["x"].value));
        resumes_exactly(SIGNED, &game);
        // 0.0 is the loaded value again, and left out again.
        game.dispatch_json("e", r#"{"v": 0}"#).unwrap();
        assert!(game.save().unwrap().states.is_empty());
        resumes_exactly(SIGNED, &game);
    }
}

// A program makes -0.0 itself from payloads without one: negation, a product
// with zero, a quotient and rounding.
#[test]
fn a_negative_zero_the_program_computes_survives_a_save() {
    let source = "state negated = 0 min -1 max 1; state product = 0 min -9 max 9; \
                  state quotient = 0 min -1 max 1; state rounded = 0 min -1 max 1; \
                  event e v min -1 max 1, w min -9 max 9; \
                  on e set negated = -v; on e set product = w * 0; \
                  on e set quotient = v / -2; on e set rounded = round(v - 0.4);";
    let mut game = ReactiveSession::from_source(source).unwrap();
    game.dispatch_json("e", r#"{"v": 0, "w": -2}"#).unwrap();
    let values = game.snapshot().values;
    assert!(
        values.values().all(|value| is_negative_zero(*value)),
        "{values:?}"
    );
    assert_eq!(game.save().unwrap().states.len(), 4);
    resumes_exactly(source, &game);
}

// The other way: a state the program loads as -0.0 and an event sets to 0.0
// was left out too, and came back as -0.0.
#[test]
fn a_zero_over_a_loaded_negative_zero_survives_a_save() {
    let source = "state x = -0 min -1 max 1; event e v min -1 max 1; on e set x = v; \
                  bind hud.angle = atan2(x, -1);";
    let mut game = ReactiveSession::from_source(source).unwrap();
    assert!(is_negative_zero(game.snapshot().values["x"]));
    resumes_exactly(source, &game);
    game.dispatch_json("e", r#"{"v": 0}"#).unwrap();
    assert_eq!(angle(&game), Some(std::f64::consts::PI));
    assert_eq!(game.save().unwrap().states["x"].value.to_bits(), 0);
    resumes_exactly(source, &game);
}

// A binding is evaluated again only when something it reads changed, and a
// state that changed only its zero's sign compared unchanged: hud.angle went on
// showing pi for -0.0, which only a full evaluation, as a restore makes,
// corrected. Debug builds check every event against a full evaluation, and
// panicked here; they compare text now, so hud.x's sign counts as well.
#[test]
fn a_state_that_changes_only_its_zeros_sign_is_shown_again() {
    use std::f64::consts::PI;
    let mut game = ReactiveSession::from_source(SIGNED).unwrap();
    assert_eq!(angle(&game), Some(PI));
    game.dispatch_json("e", r#"{"v": -0.0}"#).unwrap();
    assert_eq!(angle(&game), Some(-PI));
    let x = game.snapshot().bindings["hud"]["x"].as_number().unwrap();
    assert!(is_negative_zero(x));
    game.dispatch_json("e", r#"{"v": 0}"#).unwrap();
    assert_eq!(angle(&game), Some(PI));
}

// Every other number a save holds was already written whole, sign and all: a
// reading, a commitment's frozen `using` value and its journal entry, a
// scheduled qualification's delay and the last event's effects. (Elapsed time
// is elapsed_clock.rs's.) This keeps them so.
#[test]
fn every_other_saved_number_keeps_its_zeros_sign() {
    let source = r#"
        claim safe;
        evidence sensor from "sensor";
        evidence sight from "sight";
        caveat stale consequence material;
        readings flow from sensor limit 4;
        event read v min -1 max 1;
        event see d min 0 max 5;
        event decide;
        event tick dt min 0 max 0.1;
        on read sample flow = v supports safe;
        on see reveal sight supports safe;
        on see qualify sight with stale after -d;
        on decide commit hold because enough using latest(flow);
        bind hud.latest = if(has_sample(flow), atan2(latest(flow), -1), 0);
    "#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    for (event, payload) in [
        ("read", r#"{"v": -0.0}"#),
        ("decide", "{}"),
        ("see", r#"{"d": 0}"#),
    ] {
        game.dispatch_json(event, payload).unwrap();
        resumes_exactly(source, &game);
    }
    let save = game.save().unwrap();
    assert!(is_negative_zero(
        save.reading_streams["flow"].occurrences[0].value
    ));
    assert!(is_negative_zero(
        save.commitment_bases["hold"].value.unwrap()
    ));
    assert!(is_negative_zero(save.decision_journal[0].value.unwrap()));
    assert!(is_negative_zero(save.scheduled_qualifications[0].after));
}

// A save written before -0.0 was kept left such a state out. It restores as it
// did then, to the 0.0 the program loads, and saves the same text again.
#[test]
fn a_save_that_left_a_negative_zero_out_restores_as_before() {
    let source_id = ReactiveSession::from_source(SIGNED)
        .unwrap()
        .snapshot()
        .source_id;
    let old = format!(
        r#"{{"schema":"caveat-reactive-save/0.1","source_id":"{source_id}","sequence":1,"last_event":"e","elapsed":0.0,"states":{{}},"graph":{{}}}}"#
    );
    let restored = ReactiveSession::restore_json(SIGNED, &old).unwrap();
    assert_eq!(restored.snapshot().values["x"].to_bits(), 0);
    assert_eq!(angle(&restored), Some(std::f64::consts::PI));
    assert_eq!(restored.save_json().unwrap(), old);
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

/// Every kind of effect and a cue, each made by one event: directly, through
/// a procedure or a procedure specialized for the names it is passed, by a reading's reopening trigger, and by a scheduled caveat
/// a tick applies. The rc.11 restore checks (cues, effect kinds) are made
/// against this program's rules.
const EFFECTS: &str = r#"
budget 4;
claim safe;
claim misreading;
evidence chart from "archive";
evidence bite from "bite";
evidence recheck from "recheck";
evidence ci from "ci";
evidence memo from "memo";
caveat stale consequence low;
caveat faded consequence low;
renewable bite limit 4;
readings checks from ci limit 8;
decisions go limit 4 reopened by checks;
cue ping toast "Ping" 1;
cue bell toast "Bell" 1;
event tick dt min 0 max 0.1;
event look;
event eat;
event check result min 0 max 1;
event decide;
event regrow;
event misread;
event note;
event audit;
event mark;
proc stamp(e evidence, c caveat) { qualify e with c; };
proc chime() {
    emit bell;
    examine stale cost 1;
};
on look reveal chart supports safe;
on look qualify chart with stale;
on eat reveal bite supports safe;
on eat qualify bite with faded after 0.05;
on eat emit ping;
on check sample checks = result supports safe;
on decide commit go because enough using latest(checks);
on regrow renew bite;
on misread when not observed(recheck) reveal recheck supports misreading;
on misread withdraw latest(checks) because recheck;
on note reveal memo;
on audit call chime();
on mark call stamp(chart, faded);
"#;

/// `EFFECTS` played through every event, with the save after each.
fn effect_saves() -> Vec<(&'static str, serde_json::Value)> {
    let mut game = ReactiveSession::from_source(EFFECTS).unwrap();
    let steps: [(&str, &[(&str, f64)]); 11] = [
        ("look", &[]),
        ("eat", &[]),
        ("check", &[("result", 1.0)]),
        ("decide", &[]),
        ("check", &[("result", 0.0)]),
        ("tick", &[("dt", 0.1)]),
        ("regrow", &[]),
        ("misread", &[]),
        ("note", &[]),
        ("audit", &[]),
        ("mark", &[]),
    ];
    steps
        .into_iter()
        .map(|(event, parameters)| {
            send(&mut game, event, parameters);
            let save = serde_json::to_value(game.save().unwrap()).unwrap();
            (event, save)
        })
        .collect()
}

/// The save `EFFECTS` writes after `event`.
fn effect_save(event: &str) -> serde_json::Value {
    effect_saves()
        .into_iter()
        .rev()
        .find(|(after, _)| *after == event)
        .unwrap()
        .1
}

fn refused_effects(save: &serde_json::Value, expected: &str) {
    match ReactiveSession::restore_json(EFFECTS, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(game) => panic!(
            "accepted with effects {:?} and cues {:?}",
            game.snapshot().effects,
            save["cues"]
        ),
    }
}

// The saves the runtime writes after each event restore, whatever effects and
// cues that event made, and the restored session goes on as the original.
#[test]
fn every_save_of_each_effect_and_cue_restores() {
    let saves = effect_saves();
    let made = |event: &str| -> serde_json::Value {
        let save = &saves
            .iter()
            .rev()
            .find(|(after, _)| *after == event)
            .unwrap()
            .1;
        serde_json::json!([save["effects"], save["cues"]])
    };
    assert_eq!(
        made("eat"),
        serde_json::json!([
            [
                {"kind": "reveal", "evidence": "bite", "relation": "supports", "target": "safe"}
            ],
            ["ping"]
        ])
    );
    assert_eq!(made("audit")[1], serde_json::json!(["bell"]));
    assert_eq!(made("tick")[0][0]["kind"], "qualify");
    assert_eq!(made("check")[0][1]["kind"], "reopen");
    assert_eq!(
        made("mark")[0],
        serde_json::json!([{"kind": "qualify", "evidence": "chart", "caveat": "faded"}])
    );
    for (event, save) in &saves {
        let mut resumed = ReactiveSession::restore_json(EFFECTS, &save.to_string())
            .unwrap_or_else(|error| panic!("after {event}: {error}"));
        assert_eq!(
            serde_json::to_value(resumed.save().unwrap()).unwrap(),
            *save,
            "after {event}"
        );
        send(&mut resumed, "tick", &[("dt", 0.1)]);
    }
}

// F115: a cue the program declares, but the last event cannot emit, was
// restored and shown.
#[test]
fn a_cue_the_last_event_cannot_emit_is_refused() {
    let mut look = effect_save("look");
    look["cues"] = serde_json::json!(["ping"]);
    look["cue_qualifications"] = serde_json::json!([{}]);
    refused_effects(&look, "cue ping is not one the last event can emit");
    // A cue its procedure emits is not one another event's rule emits.
    let mut audit = effect_save("audit");
    audit["cues"] = serde_json::json!(["ping"]);
    refused_effects(&audit, "cue ping is not one the last event can emit");
    let mut eat = effect_save("eat");
    eat["cues"] = serde_json::json!(["ping", "bell"]);
    eat["cue_qualifications"] = serde_json::json!([{}, {}]);
    refused_effects(&eat, "cue bell is not one the last event can emit");
    // A fresh session has no last event, so it shows no cue.
    let mut fresh = serde_json::to_value(
        ReactiveSession::from_source(EFFECTS)
            .unwrap()
            .save()
            .unwrap(),
    )
    .unwrap();
    fresh["cues"] = serde_json::json!(["ping"]);
    fresh["cue_qualifications"] = serde_json::json!([{}]);
    refused_effects(&fresh, "cue ping is not one the last event can emit");
}

// F248: restore checked an effect's names, and a reveal's relation, but not
// that the last event could make an effect of that kind on those names. Each
// of these was a declared or created name, so each was restored and shown.
#[test]
fn an_effect_the_last_event_cannot_make_is_refused() {
    let cases = [
        (
            "look",
            serde_json::json!({"kind": "examine", "caveat": "stale", "cost": 1}),
        ),
        (
            "audit",
            serde_json::json!({"kind": "examine", "caveat": "stale", "cost": 2}),
        ),
        (
            "look",
            serde_json::json!({"kind": "qualify", "evidence": "chart", "caveat": "faded"}),
        ),
        // Only the clock applies the scheduled caveat, and only to its evidence.
        (
            "audit",
            serde_json::json!({"kind": "qualify", "evidence": "bite", "caveat": "faded"}),
        ),
        (
            "tick",
            serde_json::json!({"kind": "qualify", "evidence": "chart", "caveat": "faded"}),
        ),
        (
            "audit",
            serde_json::json!({"kind": "reveal", "evidence": "chart", "relation": "supports", "target": "safe"}),
        ),
        (
            "audit",
            serde_json::json!({"kind": "reveal", "evidence": "memo"}),
        ),
        (
            "audit",
            serde_json::json!({"kind": "sample", "stream": "checks", "id": "checks@1", "value": 1.0, "relation": "supports", "target": "safe"}),
        ),
        (
            "audit",
            serde_json::json!({"kind": "commit", "action": "go@1", "retained": []}),
        ),
        (
            "audit",
            serde_json::json!({"kind": "reopen", "action": "go@1", "because": "checks@2"}),
        ),
        (
            "audit",
            serde_json::json!({"kind": "renew", "evidence": "bite", "occurrence": "bite@2"}),
        ),
        (
            "audit",
            serde_json::json!({"kind": "withdraw", "evidence": "checks@2", "because": "recheck"}),
        ),
    ];
    for (event, effect) in cases {
        let mut save = effect_save(event);
        let kind = effect["kind"].as_str().unwrap().to_string();
        save["effects"]
            .as_array_mut()
            .unwrap_or_else(|| panic!("{event} saved no effects"))
            .push(effect);
        refused_effects(
            &save,
            &format!("{kind} effect is not one the last event can make"),
        );
    }
    // An effect made on a different evidence than its rule names.
    let mut look = effect_save("look");
    look["effects"][1]["evidence"] = "bite".into();
    refused_effects(&look, "qualify effect is not one the last event can make");
}
