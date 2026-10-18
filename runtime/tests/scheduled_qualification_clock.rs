// Renewal 0.1 and Elapsed 0.1 on scheduled qualifications, under random
// events: `qualify E with C after S` is scheduled against the occurrence
// current then, and applies on the first clock event at which
// `elapsed() - scheduled_at >= S` in binary64, in scheduling order, exactly as
// an immediate qualify would, however often the evidence was renewed, taken
// back or withdrawn in between; until then it waits, and a non-clock event
// changes nothing about what waits except what it schedules itself. Checked
// against the snapshot after every accepted event, with saves and restores
// in place, on a fractional clock and on an integer clock.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const FRACTIONAL: &str = r#"
caveat slow consequence low;
caveat fast consequence material;
claim ok;
evidence e from "tool";
renewable e limit 6;
evidence f from "operator";
state a = 0;
event see_e;
event see_f;
event new_e;
event soon;
event later;
event much_later;
event flag_now;
event unflag_slow;
event unflag_fast;
event retract;
event hold;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on see_e reveal e supports ok;
on see_f reveal f;
on new_e renew e;
on soon qualify e with fast after 0.15;
on later qualify e with slow after 0.3;
on much_later qualify e with slow after 0.7;
on flag_now qualify e with fast;
on unflag_slow unqualify e with slow;
on unflag_fast unqualify e with fast;
on retract withdraw e because f;
on hold set a = qualified(1, e);
bind view.a = a;
bind view.slow = if(has_caveat(a, slow), 1, 0);
"#;

const INTEGER: &str = r#"
caveat slow consequence low;
caveat fast consequence material;
claim ok;
evidence e from "tool";
renewable e limit 6;
evidence f from "operator";
state a = 0;
event see_e;
event see_f;
event new_e;
event soon;
event later;
event much_later;
event flag_now;
event unflag_slow;
event unflag_fast;
event retract;
event hold;
event step dt min 0 max 3;
clock step every 1 integer;
on see_e reveal e supports ok;
on see_f reveal f;
on new_e renew e;
on soon qualify e with fast after 2;
on later qualify e with slow after 3;
on much_later qualify e with slow after 7;
on flag_now qualify e with fast;
on unflag_slow unqualify e with slow;
on unflag_fast unqualify e with fast;
on retract withdraw e because f;
on hold set a = qualified(1, e);
bind view.a = a;
bind view.slow = if(has_caveat(a, slow), 1, 0);
"#;

const EVENTS: [&str; 11] = [
    "see_e",
    "see_f",
    "new_e",
    "soon",
    "later",
    "much_later",
    "flag_now",
    "unflag_slow",
    "unflag_fast",
    "retract",
    "hold",
];

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

/// Whether `caveat qualifies occurrence` is live: more relations than take-backs.
fn live(shown: &Value, caveat: &str, occurrence: &str) -> bool {
    let relations = shown["relations"].as_array().map_or(0, |list| {
        list.iter()
            .filter(|edge| {
                edge["from"] == caveat
                    && edge["to"] == occurrence
                    && edge["relation"] == "qualifies"
            })
            .count()
    });
    let taken_back = shown["taken_back"].as_array().map_or(0, |list| {
        list.iter()
            .filter(|record| record["caveat"] == caveat && record["evidence"] == occurrence)
            .count()
    });
    relations > taken_back
}

fn pending(shown: &Value) -> Vec<(String, String, f64, f64)> {
    shown["scheduled_qualifications"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|item| {
                    (
                        item["evidence"].as_str().unwrap().to_string(),
                        item["caveat"].as_str().unwrap().to_string(),
                        item["scheduled_at"].as_f64().unwrap(),
                        item["after"].as_f64().unwrap(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn run_program(
    source: &str,
    clock: &str,
    ticks: &[&str],
    seed_start: u64,
) -> (usize, usize, usize) {
    let mut seed = seed_start;
    let (mut applied, mut waited, mut scheduled) = (0, 0, 0);
    for run in 0..200 {
        let mut game = ReactiveSession::from_source(source).unwrap();
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        for step in 0..60 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + ticks.len() + 2);
            let (event, payload) = if pick < EVENTS.len() {
                (EVENTS[pick], "{}")
            } else if pick < EVENTS.len() + ticks.len() {
                (clock, ticks[pick - EVENTS.len()])
            } else {
                trail.push(("save", ""));
                let saved = game.save_json().unwrap();
                game = ReactiveSession::restore_json(source, &saved).unwrap_or_else(|error| {
                    panic!("run {run} step {step} restore: {error}: {trail:?}")
                });
                assert_eq!(
                    snapshot(&game),
                    before,
                    "run {run} step {step} after restore: {trail:?}"
                );
                continue;
            };
            trail.push((event, payload));
            let outcome =
                serde_json::to_value(game.dispatch_outcome_json(event, payload).unwrap_or_else(
                    |fatal| panic!("run {run} step {step} fatal {}: {trail:?}", fatal.message),
                ))
                .unwrap();
            if outcome["outcome"] != "accepted" {
                continue;
            }
            let shown = snapshot(&game);
            let at = format!("run {run} step {step} after {event} {payload}: {trail:?}");
            let was = pending(&before);
            let now = pending(&shown);
            let elapsed = shown["elapsed"].as_f64().unwrap();
            if event == clock {
                // Time advanced by dt, then every due qualification applied in
                // its scheduling order, and the rest still wait, in order.
                let dt: f64 = serde_json::from_str::<Value>(payload).unwrap()["dt"]
                    .as_f64()
                    .unwrap();
                assert_eq!(
                    elapsed,
                    before["elapsed"].as_f64().unwrap() + dt,
                    "{at}: elapsed"
                );
                let (due, left): (Vec<_>, Vec<_>) = was
                    .iter()
                    .partition(|(_, _, scheduled_at, after)| elapsed - scheduled_at >= *after);
                assert_eq!(
                    now,
                    left.into_iter().cloned().collect::<Vec<_>>(),
                    "{at}: what still waits after the tick"
                );
                let effects = shown["effects"].as_array().unwrap();
                for (occurrence, caveat, _, _) in &due {
                    applied += 1;
                    assert!(
                        live(&shown, caveat, occurrence),
                        "{at}: {caveat} due on {occurrence} is not live after the tick: {}",
                        shown["relations"]
                    );
                    assert!(
                        effects.iter().any(|effect| effect["kind"] == "qualify"
                            && effect["evidence"] == occurrence.as_str()
                            && effect["caveat"] == caveat.as_str()),
                        "{at}: no qualify effect for {caveat} on {occurrence}: {effects:?}"
                    );
                }
                waited += now.len();
            } else {
                // A non-clock event leaves the queue as it was, plus what it
                // scheduled, each against the occurrence current now.
                assert!(
                    now.len() >= was.len() && now[..was.len()] == was[..],
                    "{at}: the queue changed under a non-clock event\n{was:?}\n{now:?}"
                );
                assert_eq!(
                    elapsed,
                    before["elapsed"].as_f64().unwrap(),
                    "{at}: elapsed moved on a non-clock event"
                );
                let current = shown["renewals"]["e"]["occurrences"]
                    .as_array()
                    .unwrap()
                    .last()
                    .unwrap()
                    .as_str()
                    .unwrap();
                for (occurrence, _, scheduled_at, after) in &now[was.len()..] {
                    scheduled += 1;
                    assert_eq!(
                        occurrence, current,
                        "{at}: scheduled against an occurrence that is not current"
                    );
                    assert_eq!(*scheduled_at, elapsed, "{at}: scheduled_at");
                    assert!(*after > 0.0, "{at}: after");
                }
                let added = now.len() - was.len();
                let expected = match event {
                    "soon" | "later" | "much_later" => 1,
                    _ => 0,
                };
                assert_eq!(added, expected, "{at}: scheduled {added}");
            }
            before = shown;
        }
    }
    (applied, waited, scheduled)
}

#[test]
fn the_clock_programs_load() {
    ReactiveSession::from_source(FRACTIONAL).unwrap();
    ReactiveSession::from_source(INTEGER).unwrap();
}

#[test]
fn scheduled_qualifications_apply_on_the_first_due_clock_event_and_not_before() {
    let (applied, waited, scheduled) = run_program(
        FRACTIONAL,
        "tick",
        &[
            r#"{"dt": 0.1}"#,
            r#"{"dt": 0.1}"#,
            r#"{"dt": 0.05}"#,
            r#"{"dt": 0.025}"#,
            r#"{"dt": 0}"#,
        ],
        0x3c6e_f372_fe94_f82b,
    );
    eprintln!(
        "fractional clock: {scheduled} scheduled, {applied} applied, {waited} waits observed"
    );
    assert!(
        applied > 300 && waited > 500,
        "the alphabet barely exercised the queue: {applied} / {waited}"
    );
    let (applied, waited, scheduled) = run_program(
        INTEGER,
        "step",
        &[
            r#"{"dt": 1}"#,
            r#"{"dt": 1}"#,
            r#"{"dt": 2}"#,
            r#"{"dt": 3}"#,
            r#"{"dt": 0}"#,
        ],
        0x2d35_8dcc_aa6c_78a5,
    );
    eprintln!("integer clock: {scheduled} scheduled, {applied} applied, {waited} waits observed");
    assert!(
        applied > 300 && waited > 500,
        "the alphabet barely exercised the queue: {applied} / {waited}"
    );
}
