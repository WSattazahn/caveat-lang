// Routed Repetition 0.1 under random events: a routed block is the plain
// block with `P == $index` written at the front of each routed rule's guard,
// parenthesising a guard with an `or`, and nothing else; a rule on an event
// that names no member, or on an event written with the binding, is not
// routed; so a routed program and its hand-selected twin, driven in
// lockstep, give the same snapshot but for source identity, refusal for
// refusal, with saves and restores in place. The twin is the program the
// agent ledger wrote by hand before this profile.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const HEAD: &str = r#"
budget 40;
claim wet;
place field kind field;
entity north kind plot at field;
entity south kind plot at field;
entity east kind plot at field;
entity handheld kind probe at field;
evidence gauge from "the rain gauge";
state total = 0;
state swaps = 0;
decisions irrigate limit 24;
event read target kind plot, value min 0 max 9;
event ping target kind plot, loud min 0 max 1;
event decide target kind plot;
event swap device kind probe;
event tick dt min 0 max 0.1;
event reset_north;
event reset_south;
event reset_east;
proc note(c caveat) { examine c cost 1; };
on tick set total = total + 1;
on swap set swaps = swaps + 1;
"#;

const ROUTED: &str = r#"
for plot as $p routed by target {
    caveat $p_fog consequence low;
    readings $p_readings from gauge limit 12;
    state $p_n = 0;
    state $p_pings = 0;
    on read when value == 9 reject "Nine is not a reading for $p.";
    on read set $p_n = value;
    on read when value > 2 or value == 0 sample $p_readings = value supports wet;
    on read when value > 4 call note($p_fog);
    on ping when loud == 1 or $p_pings > 2 set $p_pings = $p_pings + 1;
    on ping when loud == 0 examine $p_fog cost 1;
    on ping when loud == 1 and has_sample($p_readings) and committed(irrigate) and not reopened(irrigate) reopen irrigate because latest($p_readings);
    on decide when has_sample($p_readings) and (not committed(irrigate) or reopened(irrigate)) commit irrigate because enough using latest($p_readings) + $index;
    on tick set $p_n = $p_n + 0;
    on reset_$p set $p_n = 0;
    on reset_$p set $p_pings = 0;
};
bind view.total = total;
"#;

const PLAIN: &str = r#"
for plot as $p {
    caveat $p_fog consequence low;
    readings $p_readings from gauge limit 12;
    state $p_n = 0;
    state $p_pings = 0;
    on read when target == $index and value == 9 reject "Nine is not a reading for $p.";
    on read when target == $index set $p_n = value;
    on read when target == $index and (value > 2 or value == 0) sample $p_readings = value supports wet;
    on read when target == $index and value > 4 call note($p_fog);
    on ping when target == $index and (loud == 1 or $p_pings > 2) set $p_pings = $p_pings + 1;
    on ping when target == $index and loud == 0 examine $p_fog cost 1;
    on ping when target == $index and loud == 1 and has_sample($p_readings) and committed(irrigate) and not reopened(irrigate) reopen irrigate because latest($p_readings);
    on decide when target == $index and has_sample($p_readings) and (not committed(irrigate) or reopened(irrigate)) commit irrigate because enough using latest($p_readings) + $index;
    on tick set $p_n = $p_n + 0;
    on reset_$p set $p_n = 0;
    on reset_$p set $p_pings = 0;
};
bind view.total = total;
"#;

const EVENTS: [(&str, &str); 20] = [
    ("read", r#"{"target": "north", "value": 0}"#),
    ("read", r#"{"target": "south", "value": 3}"#),
    ("read", r#"{"target": "east", "value": 5}"#),
    ("read", r#"{"target": 1, "value": 9}"#),
    ("read", r#"{"target": 2, "value": 1}"#),
    ("read", r#"{"target": "east", "value": 7}"#),
    ("read", r#"{"target": 3, "value": 2}"#),
    ("ping", r#"{"target": "north", "loud": 1}"#),
    ("ping", r#"{"target": "south", "loud": 0}"#),
    ("ping", r#"{"target": "east", "loud": 1}"#),
    ("ping", r#"{"target": 2, "loud": 1}"#),
    ("decide", r#"{"target": "north"}"#),
    ("decide", r#"{"target": "south"}"#),
    ("decide", r#"{"target": "east"}"#),
    ("swap", r#"{"device": "handheld"}"#),
    ("tick", r#"{"dt": 0.1}"#),
    ("reset_north", "{}"),
    ("reset_south", "{}"),
    ("reset_east", "{}"),
    ("read", r#"{"target": "west", "value": 1}"#),
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
fn both_programs_load() {
    ReactiveSession::from_source(&format!("{HEAD}{ROUTED}")).unwrap();
    ReactiveSession::from_source(&format!("{HEAD}{PLAIN}")).unwrap();
}

#[test]
fn a_routed_block_runs_as_its_hand_selected_twin() {
    let routed_source = format!("{HEAD}{ROUTED}");
    let plain_source = format!("{HEAD}{PLAIN}");
    let mut seed: u64 = 0x5a0c_e3d7_91b4_f26e;
    let (mut accepted, mut refused, mut commits, mut rejects) = (0, 0, 0, 0);
    for run in 0..150 {
        let mut routed = ReactiveSession::from_source(&routed_source).unwrap();
        let mut plain = ReactiveSession::from_source(&plain_source).unwrap();
        let mut trail = Vec::new();
        for step in 0..50 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                trail.push("save");
                let before = snapshot(&routed);
                routed =
                    ReactiveSession::restore_json(&routed_source, &routed.save_json().unwrap())
                        .unwrap_or_else(|error| {
                            panic!("run {run} step {step} restore: {error}: {trail:?}")
                        });
                plain = ReactiveSession::restore_json(&plain_source, &plain.save_json().unwrap())
                    .unwrap_or_else(|error| {
                        panic!("run {run} step {step} restore: {error}: {trail:?}")
                    });
                assert_eq!(
                    snapshot(&routed),
                    before,
                    "run {run} step {step}: {trail:?}"
                );
                assert_eq!(snapshot(&plain), before, "run {run} step {step}: {trail:?}");
                continue;
            }
            let (event, payload) = EVENTS[pick];
            trail.push(payload);
            let at = format!("run {run} step {step} after {event} {payload}: {trail:?}");
            let left = anonymous(
                serde_json::to_value(
                    routed
                        .dispatch_outcome_json(event, payload)
                        .unwrap_or_else(|fatal| panic!("{at}: routed fatal {}", fatal.message)),
                )
                .unwrap(),
            );
            let right = anonymous(
                serde_json::to_value(
                    plain
                        .dispatch_outcome_json(event, payload)
                        .unwrap_or_else(|fatal| panic!("{at}: plain fatal {}", fatal.message)),
                )
                .unwrap(),
            );
            assert_eq!(left, right, "{at}: the outcomes differ");
            assert_eq!(
                snapshot(&routed),
                snapshot(&plain),
                "{at}: the snapshots differ"
            );
            // Every event's save restores, to the same snapshot (D021).
            let saved_routed = routed.save_json().unwrap();
            let restored_routed = ReactiveSession::restore_json(&routed_source, &saved_routed)
                .unwrap_or_else(|error| panic!("{at}: restore after the event: {error}"));
            assert_eq!(
                restored_routed.save_json().unwrap(),
                saved_routed,
                "{at}: save round trip"
            );
            assert_eq!(
                snapshot(&restored_routed),
                snapshot(&routed),
                "{at}: snapshot after restore"
            );
            // Every event's save restores, to the same snapshot (D021).
            let saved_plain = plain.save_json().unwrap();
            let restored_plain = ReactiveSession::restore_json(&plain_source, &saved_plain)
                .unwrap_or_else(|error| panic!("{at}: restore after the event: {error}"));
            assert_eq!(
                restored_plain.save_json().unwrap(),
                saved_plain,
                "{at}: save round trip"
            );
            assert_eq!(
                snapshot(&restored_plain),
                snapshot(&plain),
                "{at}: snapshot after restore"
            );
            if left["outcome"] == "accepted" {
                accepted += 1;
                if left["snapshot"]["effects"]
                    .as_array()
                    .is_some_and(|list| list.iter().any(|effect| effect["kind"] == "commit"))
                {
                    commits += 1;
                }
            } else {
                refused += 1;
                if left["code"] == "reject" {
                    rejects += 1;
                }
            }
        }
    }
    eprintln!(
        "{accepted} accepted events, {refused} refused, {commits} commits, {rejects} rejects"
    );
    assert!(
        accepted > 4000 && refused > 500 && commits > 300 && rejects > 200,
        "{accepted} / {refused} / {commits} / {rejects}"
    );
}
