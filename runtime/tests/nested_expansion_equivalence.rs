// Nested Repetition 0.1 under random events: a routed block nested in a
// routed block is the hand-written program with `P == i and P2 == j` at the
// front of each routed rule's guard, outermost route first, an `or` guard
// parenthesized, the copies in outer-member-major order, and nothing else;
// so the nested program and its hand-written twin, driven in lockstep, give
// the same snapshot but for source identity, refusal for refusal, with saves
// and restores in place.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const COMPONENTS: [&str; 3] = ["api", "web", "worker"];
const SLOTS: [&str; 3] = ["d1", "d2", "d3"];

const HEAD: &str = r#"
budget 40;
claim ok;
place repo kind repository;
entity api kind component at repo;
entity web kind component at repo;
entity worker kind component at repo;
entity d1 kind slot at repo;
entity d2 kind slot at repo;
entity d3 kind slot at repo;
evidence tool from "a tool";
state ticks = 0;
event report component kind component, slot kind slot, n min 0 max 9;
event ack component kind component, slot kind slot, n min 0 max 9;
event pick component kind component, slot kind slot, n min 0 max 9;
event pulse;
event reset_api;
event reset_web;
event reset_worker;
on pulse set ticks = ticks + 1;
"#;

const NESTED: &str = r#"
for component as $c routed by component {
    state $c_total = 0;
    on reset_$c set $c_total = 0;
    for slot as $k routed by slot {
        readings $c_$k from tool limit 12;
        state $c_ack_$k = 0;
        on report sample $c_$k = n supports ok;
        on ack when n == 9 reject "Nine is not an ack for $c $k.";
        on ack set $c_ack_$k = $c_ack_$k + 1;
        on ack when n > 2 or n == 0 set $c_total = $c_total + n + 0 * $c_ack_$k;
        on pick when n < 1 or n > history_count($c_$k) reject "no such report";
        on pick set $c_total = $c_total + history_at($c_$k, n - 1);
        on pulse set $c_ack_$k = $c_ack_$k + 0;
        bind $c.$k = $c_ack_$k;
    };
    bind $c.total = $c_total;
};
bind view.ticks = ticks;
"#;

/// The program an author writes by hand without a nested block: one copy
/// per component and slot, outer member first, each routed rule selecting
/// its component and then its slot.
fn hand_written() -> String {
    let mut out = String::new();
    for (i, c) in COMPONENTS.iter().enumerate() {
        let i = i + 1;
        out.push_str(&format!(
            "state {c}_total = 0;\non reset_{c} set {c}_total = 0;\n"
        ));
        for (j, k) in SLOTS.iter().enumerate() {
            let j = j + 1;
            let route = format!("component == {i} and slot == {j}");
            out.push_str(&format!(
                "readings {c}_{k} from tool limit 12;\n\
                 state {c}_ack_{k} = 0;\n\
                 on report when {route} sample {c}_{k} = n supports ok;\n\
                 on ack when {route} and n == 9 reject \"Nine is not an ack for {c} {k}.\";\n\
                 on ack when {route} set {c}_ack_{k} = {c}_ack_{k} + 1;\n\
                 on ack when {route} and (n > 2 or n == 0) set {c}_total = {c}_total + n + 0 * {c}_ack_{k};\n\
                 on pick when {route} and (n < 1 or n > history_count({c}_{k})) reject \"no such report\";\n\
                 on pick when {route} set {c}_total = {c}_total + history_at({c}_{k}, n - 1);\n\
                 on pulse set {c}_ack_{k} = {c}_ack_{k} + 0;\n\
                 bind {c}.{k} = {c}_ack_{k};\n"
            ));
        }
        out.push_str(&format!("bind {c}.total = {c}_total;\n"));
    }
    out.push_str("bind view.ticks = ticks;\n");
    out
}

const EVENTS: [(&str, &str); 24] = [
    ("report", r#"{"component": "api", "slot": "d1", "n": 0}"#),
    ("report", r#"{"component": "api", "slot": 2, "n": 3}"#),
    ("report", r#"{"component": 2, "slot": "d3", "n": 5}"#),
    ("report", r#"{"component": "worker", "slot": "d1", "n": 7}"#),
    ("report", r#"{"component": 3, "slot": 3, "n": 9}"#),
    ("report", r#"{"component": "web", "slot": "d2", "n": 1}"#),
    ("ack", r#"{"component": "api", "slot": "d1", "n": 0}"#),
    ("ack", r#"{"component": "api", "slot": "d2", "n": 4}"#),
    ("ack", r#"{"component": "web", "slot": 3, "n": 9}"#),
    ("ack", r#"{"component": 2, "slot": "d2", "n": 2}"#),
    ("ack", r#"{"component": "worker", "slot": "d1", "n": 6}"#),
    ("ack", r#"{"component": "worker", "slot": "d3", "n": 1}"#),
    ("pick", r#"{"component": "api", "slot": "d1", "n": 1}"#),
    ("pick", r#"{"component": "api", "slot": "d2", "n": 2}"#),
    ("pick", r#"{"component": "web", "slot": "d3", "n": 0}"#),
    ("pick", r#"{"component": 2, "slot": 2, "n": 1}"#),
    ("pick", r#"{"component": "worker", "slot": "d1", "n": 3}"#),
    ("pick", r#"{"component": "worker", "slot": "d3", "n": 9}"#),
    ("pulse", "{}"),
    ("reset_api", "{}"),
    ("reset_web", "{}"),
    ("reset_worker", "{}"),
    ("report", r#"{"component": "db", "slot": "d1", "n": 1}"#),
    ("ack", r#"{"component": "api", "slot": 4, "n": 1}"#),
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
    ReactiveSession::from_source(&format!("{HEAD}{NESTED}")).unwrap();
    ReactiveSession::from_source(&format!("{HEAD}{}", hand_written())).unwrap();
}

#[test]
fn a_nested_routed_block_runs_as_its_hand_written_twin() {
    let nested_source = format!("{HEAD}{NESTED}");
    let plain_source = format!("{HEAD}{}", hand_written());
    let mut seed: u64 = 0x2c9d_71e4_b3a8_5f06;
    let (mut accepted, mut refused, mut samples, mut rejects, mut picks) = (0, 0, 0, 0, 0);
    for run in 0..150 {
        let mut nested = ReactiveSession::from_source(&nested_source).unwrap();
        let mut plain = ReactiveSession::from_source(&plain_source).unwrap();
        let mut trail = Vec::new();
        for step in 0..50 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                trail.push("save");
                let before = snapshot(&nested);
                nested =
                    ReactiveSession::restore_json(&nested_source, &nested.save_json().unwrap())
                        .unwrap_or_else(|error| {
                            panic!("run {run} step {step} restore: {error}: {trail:?}")
                        });
                plain = ReactiveSession::restore_json(&plain_source, &plain.save_json().unwrap())
                    .unwrap_or_else(|error| {
                        panic!("run {run} step {step} restore: {error}: {trail:?}")
                    });
                assert_eq!(
                    snapshot(&nested),
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
                    nested
                        .dispatch_outcome_json(event, payload)
                        .unwrap_or_else(|fatal| panic!("{at}: nested fatal {}", fatal.message)),
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
                snapshot(&nested),
                snapshot(&plain),
                "{at}: the snapshots differ"
            );
            // Every event's save restores, to the same snapshot (D021).
            let saved_nested = nested.save_json().unwrap();
            let restored_nested = ReactiveSession::restore_json(&nested_source, &saved_nested)
                .unwrap_or_else(|error| panic!("{at}: restore after the event: {error}"));
            assert_eq!(
                restored_nested.save_json().unwrap(),
                saved_nested,
                "{at}: save round trip"
            );
            assert_eq!(
                snapshot(&restored_nested),
                snapshot(&nested),
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
                if event == "report" {
                    samples += 1;
                }
                if event == "pick" {
                    picks += 1;
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
        "{accepted} accepted events, {refused} refused, {samples} reports, {picks} picks, {rejects} rejects"
    );
    assert!(
        accepted > 4000 && refused > 500 && samples > 1000 && picks > 300 && rejects > 300,
        "{accepted} / {refused} / {samples} / {picks} / {rejects}"
    );
}
