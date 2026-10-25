// Reactive 0.7 and Procedure Symbols 0.1 under random events. Three
// promises, each its own check:
//
// 1. A call shares its event's transaction: a failure anywhere in a body,
//    nested calls included, rolls the whole event back, save, snapshot and
//    view byte for byte.
// 2. A call's basis, the calling guard's qualifications, the enclosing
//    calls' bases and every argument's qualifications, carries through all
//    descendant effects, even an unused parameter's; a plain rule's effect
//    carries only what it read.
// 3. A procedure with symbol parameters runs exactly as its hand-written
//    specialization would: a program calling it and its twin with the body
//    written out per set of names run in lockstep, snapshots equal but for
//    the source's identity.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeSet;

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn view(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.view()).unwrap()
}

fn outcome(game: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    serde_json::to_value(
        game.dispatch_outcome_json(event, payload)
            .unwrap_or_else(|fatal| panic!("{event} {payload}: fatal {}", fatal.message)),
    )
    .unwrap()
}

fn strings(list: &Value) -> BTreeSet<String> {
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

// 1. Transactional calls.

const ROLLBACK: &str = r#"
caveat flaky consequence material;
evidence e from "tool";
state a = 0;
state b = 0 min 0 max 10;
state c = 0;
state depth = 0;
event see_e;
event grow;
event shrink;
event deep n min 0 max 3;
event inner_reject;
event inner_bound;
event inner_divide;
event inner_unobserved;
event inner_require;
event fine;
proc work(n) {
    set a = a + n;
    set c = c + 1;
};
proc then_reject(n) {
    call work(n);
    when a > 2 reject "too much";
    set depth = depth + 1;
};
proc then_bound(n) {
    call work(n);
    set b = b + 7;
};
proc then_divide(n) {
    call work(n);
    set c = n / (a - a);
};
proc then_unobserved(n) {
    call work(n);
    set c = qualified(n, e);
};
proc then_require(n) {
    call work(n);
    set c = require(a < 0, n);
};
proc nest(n) {
    when n > 0 call work(n);
    when n > 1 call then_bound(n);
    when n > 2 call then_reject(n);
};
on see_e reveal e;
on grow call work(1);
on shrink set a = 0;
on shrink set b = 0;
on deep call nest(n);
on inner_reject call then_reject(1);
on inner_bound call then_bound(1);
on inner_divide call then_divide(1);
on inner_unobserved call then_unobserved(1);
on inner_require call then_require(1);
on fine call work(0);
bind view.a = a;
bind view.c = c;
"#;

const ROLLBACK_EVENTS: [(&str, &str); 13] = [
    ("see_e", "{}"),
    ("grow", "{}"),
    ("grow", "{}"),
    ("shrink", "{}"),
    ("deep", r#"{"n": 1}"#),
    ("deep", r#"{"n": 2}"#),
    ("deep", r#"{"n": 3}"#),
    ("inner_reject", "{}"),
    ("inner_bound", "{}"),
    ("inner_divide", "{}"),
    ("inner_unobserved", "{}"),
    ("inner_require", "{}"),
    ("fine", "{}"),
];

#[test]
fn a_failure_anywhere_in_a_call_rolls_the_event_back() {
    let mut seed: u64 = 0x1234_abcd_5678_ef01;
    let mut codes = std::collections::BTreeMap::<String, usize>::new();
    let mut accepted = 0;
    for run in 0..200 {
        let mut game = ReactiveSession::from_source(ROLLBACK).unwrap();
        let mut trail = Vec::new();
        for step in 0..40 {
            let pick = next(&mut seed) as usize % (ROLLBACK_EVENTS.len() + 1);
            if pick == ROLLBACK_EVENTS.len() {
                trail.push(("save", ""));
                let before = snapshot(&game);
                let saved = game.save_json().unwrap();
                game = ReactiveSession::restore_json(ROLLBACK, &saved).unwrap();
                assert_eq!(snapshot(&game), before, "run {run} step {step}: {trail:?}");
                continue;
            }
            let (event, payload) = ROLLBACK_EVENTS[pick];
            trail.push((event, payload));
            let save_before = game.save_json().unwrap();
            let snapshot_before = snapshot(&game);
            let view_before = view(&game);
            let result = outcome(&mut game, event, payload);
            // Every event's save restores, to the same snapshot (D021).
            let saved_game = game.save_json().unwrap();
            let restored_game = ReactiveSession::restore_json(ROLLBACK, &saved_game)
                .unwrap_or_else(|error| {
                    panic!("run {run} step {step}: {trail:?}: restore after the event: {error}")
                });
            assert_eq!(
                restored_game.save_json().unwrap(),
                saved_game,
                "run {run} step {step}: {trail:?}: save round trip"
            );
            assert_eq!(
                snapshot(&restored_game),
                snapshot(&game),
                "run {run} step {step}: {trail:?}: snapshot after restore"
            );
            if result["outcome"] == "accepted" {
                accepted += 1;
                // An accepted call did all of its work.
                let shown = snapshot(&game);
                let a = |s: &Value| s["bindings"]["view"]["a"].as_f64().unwrap();
                let c = |s: &Value| s["bindings"]["view"]["c"].as_f64().unwrap();
                match event {
                    "grow" => assert_eq!(
                        (a(&shown), c(&shown)),
                        (a(&snapshot_before) + 1.0, c(&snapshot_before) + 1.0),
                        "run {run} step {step}: {trail:?}"
                    ),
                    "fine" => assert_eq!(
                        (a(&shown), c(&shown)),
                        (a(&snapshot_before), c(&snapshot_before) + 1.0),
                        "run {run} step {step}: {trail:?}"
                    ),
                    "deep" => {
                        let n: f64 = serde_json::from_str::<Value>(payload).unwrap()["n"]
                            .as_f64()
                            .unwrap();
                        // work once for n >= 1 and again inside then_bound for n >= 2; n = 3 reaches then_reject.
                        let calls = if n >= 2.0 {
                            2.0
                        } else if n >= 1.0 {
                            1.0
                        } else {
                            0.0
                        };
                        assert_eq!(
                            c(&shown),
                            c(&snapshot_before) + calls,
                            "run {run} step {step}: {trail:?}"
                        );
                        assert_eq!(
                            a(&shown),
                            a(&snapshot_before) + n * calls,
                            "run {run} step {step}: {trail:?}"
                        );
                    }
                    _ => {}
                }
            } else {
                let site = format!(
                    "{}/{}",
                    result["origin"].as_str().unwrap(),
                    result["code"].as_str().unwrap()
                );
                *codes.entry(site.clone()).or_default() += 1;
                let detail = format!(
                    "run {run} step {step}, {site} {}: {trail:?}",
                    result["message"]
                );
                assert_eq!(
                    game.save_json().unwrap(),
                    save_before,
                    "save after a refused call, {detail}"
                );
                assert_eq!(
                    snapshot(&game),
                    snapshot_before,
                    "snapshot after a refused call, {detail}"
                );
                assert_eq!(
                    view(&game),
                    view_before,
                    "view after a refused call, {detail}"
                );
            }
        }
    }
    eprintln!("{accepted} accepted; refusals inside calls: {codes:?}");
    for site in [
        "policy/reject",
        "evaluation/bound_exceeded",
        "evaluation/expression",
        "evaluation/unobserved_evidence",
        "evaluation/requirement_failed",
    ] {
        assert!(
            codes.contains_key(site),
            "no refusal at {site} inside a call: {codes:?}"
        );
    }
}

// 2. The call basis.

const BASIS: &str = r#"
caveat flaky consequence material;
caveat dim consequence low;
evidence e from "tool";
evidence f from "operator";
evidence g from "third";
dim qualifies g;
state a = 0;
state via_call = 0;
state via_nested = 0;
state via_rule = 0;
state unused_arg = 0;
event see_e;
event see_f;
event see_g;
event doubt_e;
event doubt_f;
event bump;
event run_call;
event run_nested;
event run_rule;
event run_unused;
proc inner(x) {
    set via_nested = a + 1;
};
proc outer(x) {
    set via_call = a + 1;
    when observed(g) call inner(qualified(x, g));
};
proc ignores(x) {
    set unused_arg = a + 1;
};
on see_e reveal e;
on see_f reveal f;
on see_g reveal g;
on doubt_e when observed(e) qualify e with flaky;
on doubt_f when observed(f) qualify f with flaky;
on bump set a = a + 1;
on run_call when observed(e) call outer(qualified(a, f));
on run_nested when observed(e) call outer(qualified(a, f));
on run_rule when observed(e) set via_rule = a + 1;
on run_unused when observed(e) call ignores(qualified(a, f));
"#;

const BASIS_EVENTS: [&str; 10] = [
    "see_e",
    "see_f",
    "see_g",
    "doubt_e",
    "doubt_f",
    "bump",
    "run_call",
    "run_nested",
    "run_rule",
    "run_unused",
];

#[test]
fn a_call_basis_reaches_every_descendant_effect_and_a_rule_carries_only_its_reads() {
    let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
    let (mut calls, mut nested, mut unused) = (0, 0, 0);
    for run in 0..200 {
        let mut game = ReactiveSession::from_source(BASIS).unwrap();
        let mut trail = Vec::new();
        for step in 0..30 {
            let event = BASIS_EVENTS[next(&mut seed) as usize % BASIS_EVENTS.len()];
            trail.push(event);
            let before = snapshot(&game);
            let result = outcome(&mut game, event, "{}");
            // Every event's save restores, to the same snapshot (D021).
            let saved_game = game.save_json().unwrap();
            let restored_game =
                ReactiveSession::restore_json(BASIS, &saved_game).unwrap_or_else(|error| {
                    panic!("run {run} step {step}: {trail:?}: restore after the event: {error}")
                });
            assert_eq!(
                restored_game.save_json().unwrap(),
                saved_game,
                "run {run} step {step}: {trail:?}: save round trip"
            );
            assert_eq!(
                snapshot(&restored_game),
                snapshot(&game),
                "run {run} step {step}: {trail:?}: snapshot after restore"
            );
            if result["outcome"] != "accepted" {
                // Only the unobserved f in an argument can refuse here.
                assert_eq!(
                    result["code"], "unobserved_evidence",
                    "run {run} step {step}: {result}"
                );
                continue;
            }
            let shown = snapshot(&game);
            let at = format!("run {run} step {step} after {event}: {trail:?}");
            let lineage =
                |state: &str| strings(&shown["qualified_values"][state]["provenance"]["evidence"]);
            let caveats =
                |state: &str| strings(&shown["qualified_values"][state]["provenance"]["caveats"]);
            let observed = |name: &str| {
                before["observations"]
                    .as_array()
                    .is_some_and(|list| list.iter().any(|item| item == name))
            };
            let flagged = |name: &str| {
                before["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|edge| edge["from"] == "flaky" && edge["to"] == name)
            };
            // Every checked rule is guarded on observed(e); before that, an
            // accepted event sets nothing.
            if !observed("e") {
                continue;
            }
            match event {
                "run_call" | "run_nested" => {
                    calls += 1;
                    // The guard read e and the argument read f: both reach via_call.
                    let evidence = lineage("via_call");
                    assert!(
                        evidence.contains("e") && evidence.contains("f"),
                        "{at}: via_call lineage {evidence:?}"
                    );
                    let expected = [flagged("e"), flagged("f")].iter().any(|&yes| yes);
                    assert_eq!(
                        caveats("via_call").contains("flaky"),
                        expected,
                        "{at}: via_call caveats {:?}",
                        caveats("via_call")
                    );
                    if observed("g") {
                        nested += 1;
                        // Inherited: e and f from the enclosing call, g from the step's guard and argument, dim with it.
                        let evidence = lineage("via_nested");
                        assert!(
                            evidence.is_superset(
                                &["e", "f", "g"].iter().map(|s| s.to_string()).collect()
                            ),
                            "{at}: via_nested lineage {evidence:?}"
                        );
                        assert!(
                            caveats("via_nested").contains("dim"),
                            "{at}: via_nested caveats {:?}",
                            caveats("via_nested")
                        );
                        assert_eq!(caveats("via_nested").contains("flaky"), expected, "{at}");
                    }
                }
                "run_rule" => {
                    // A plain rule carries its guard's e and nothing of f.
                    let evidence = lineage("via_rule");
                    assert!(
                        evidence.contains("e") && !evidence.contains("f"),
                        "{at}: via_rule lineage {evidence:?}"
                    );
                    assert_eq!(caveats("via_rule").contains("flaky"), flagged("e"), "{at}");
                }
                "run_unused" => {
                    unused += 1;
                    // An unused argument still carries: f reaches unused_arg.
                    let evidence = lineage("unused_arg");
                    assert!(
                        evidence.contains("e") && evidence.contains("f"),
                        "{at}: unused_arg lineage {evidence:?}"
                    );
                }
                _ => {}
            }
        }
    }
    eprintln!("{calls} calls, {nested} nested, {unused} with an unused argument");
    assert!(calls > 500 && nested > 200 && unused > 200);
}

// 3. Symbol specialization.

const SYMBOLIC: &str = r#"
claim ok;
caveat flaky consequence material;
evidence e from "tool";
evidence f from "operator";
evidence h from "sensor";
readings s from h limit 6;
readings t from h limit 6;
decisions d limit 4;
decisions p limit 4;
state a = 0;
state last_s = 0;
state last_t = 0;
event see_e;
event see_f;
event bump;
event read_s v min 0 max 20;
event read_t v min 0 max 20;
event decide_d;
event decide_p;
event doubt_e;
event doubt_f;
proc learn(x evidence, c caveat) {
    when not observed(x) reveal x supports ok;
    when observed(x) and a > 3 qualify x with c;
};
proc take(st readings, ds decisions, v) {
    when history_count(st) >= 6 reject "full";
    when v >= 10 sample st = v supports ok;
    when v < 10 sample st = v opposes ok;
    when v < 10 and committed(ds) and not reopened(ds) reopen ds because latest(st);
};
proc decide(ds decisions, st readings) {
    when has_sample(st) and not committed(ds) commit ds because enough using latest(st);
    when has_sample(st) and committed(ds) and reopened(ds) commit ds because enough using latest(st);
};
on see_e call learn(e, flaky);
on see_f call learn(f, flaky);
on bump set a = a + 1;
on read_s call take(s, d, v);
on read_s when has_sample(s) set last_s = latest(s);
on read_t call take(t, p, v);
on read_t when has_sample(t) set last_t = latest(t);
on decide_d call decide(d, s);
on decide_p call decide(p, t);
on doubt_e call learn(e, flaky);
on doubt_f call learn(f, flaky);
"#;

const BY_HAND: &str = r#"
claim ok;
caveat flaky consequence material;
evidence e from "tool";
evidence f from "operator";
evidence h from "sensor";
readings s from h limit 6;
readings t from h limit 6;
decisions d limit 4;
decisions p limit 4;
state a = 0;
state last_s = 0;
state last_t = 0;
event see_e;
event see_f;
event bump;
event read_s v min 0 max 20;
event read_t v min 0 max 20;
event decide_d;
event decide_p;
event doubt_e;
event doubt_f;
proc learn_e() {
    when not observed(e) reveal e supports ok;
    when observed(e) and a > 3 qualify e with flaky;
};
proc learn_f() {
    when not observed(f) reveal f supports ok;
    when observed(f) and a > 3 qualify f with flaky;
};
proc take_s(v) {
    when history_count(s) >= 6 reject "full";
    when v >= 10 sample s = v supports ok;
    when v < 10 sample s = v opposes ok;
    when v < 10 and committed(d) and not reopened(d) reopen d because latest(s);
};
proc take_t(v) {
    when history_count(t) >= 6 reject "full";
    when v >= 10 sample t = v supports ok;
    when v < 10 sample t = v opposes ok;
    when v < 10 and committed(p) and not reopened(p) reopen p because latest(t);
};
proc decide_ds() {
    when has_sample(s) and not committed(d) commit d because enough using latest(s);
    when has_sample(s) and committed(d) and reopened(d) commit d because enough using latest(s);
};
proc decide_pt() {
    when has_sample(t) and not committed(p) commit p because enough using latest(t);
    when has_sample(t) and committed(p) and reopened(p) commit p because enough using latest(t);
};
on see_e call learn_e();
on see_f call learn_f();
on bump set a = a + 1;
on read_s call take_s(v);
on read_s when has_sample(s) set last_s = latest(s);
on read_t call take_t(v);
on read_t when has_sample(t) set last_t = latest(t);
on decide_d call decide_ds();
on decide_p call decide_pt();
on doubt_e call learn_e();
on doubt_f call learn_f();
"#;

const SYMBOLIC_EVENTS: [(&str, &str); 11] = [
    ("see_e", "{}"),
    ("see_f", "{}"),
    ("bump", "{}"),
    ("read_s", r#"{"v": 12}"#),
    ("read_s", r#"{"v": 4}"#),
    ("read_t", r#"{"v": 15}"#),
    ("read_t", r#"{"v": 3}"#),
    ("decide_d", "{}"),
    ("decide_p", "{}"),
    ("doubt_e", "{}"),
    ("doubt_f", "{}"),
];

fn comparable(shown: &mut Value) {
    if let Some(record) = shown.as_object_mut() {
        record.remove("source_id");
        record.remove("source_sha256");
        // Specializations are named after the procedure; the hand-written
        // twin names its own. Procedure declarations are static.
        record.remove("procedures");
    }
}

#[test]
fn a_symbol_procedure_runs_as_its_hand_written_specialization() {
    let mut seed: u64 = 0x5851_f42d_4c95_7f2d;
    let mut accepted = 0;
    for run in 0..200 {
        let mut one = ReactiveSession::from_source(SYMBOLIC).unwrap();
        let mut two = ReactiveSession::from_source(BY_HAND).unwrap();
        let mut trail = Vec::new();
        for step in 0..40 {
            let pick = next(&mut seed) as usize % (SYMBOLIC_EVENTS.len() + 1);
            if pick == SYMBOLIC_EVENTS.len() {
                trail.push(("save", ""));
                one = ReactiveSession::restore_json(SYMBOLIC, &one.save_json().unwrap()).unwrap();
                two = ReactiveSession::restore_json(BY_HAND, &two.save_json().unwrap()).unwrap();
            } else {
                let (event, payload) = SYMBOLIC_EVENTS[pick];
                trail.push((event, payload));
                let mut expected = outcome(&mut one, event, payload);
                let mut actual = outcome(&mut two, event, payload);
                for result in [&mut expected, &mut actual] {
                    if let Some(shown) = result.get_mut("snapshot") {
                        comparable(shown);
                    }
                    // A refusal's message names the procedure, `decide[d, s]`
                    // against the twin's `decide_ds`; its origin and code are
                    // compared, the text is not.
                    result.as_object_mut().unwrap().remove("message");
                }
                assert_eq!(actual, expected, "run {run} step {step}: {trail:?}");
                // Every event's save restores, to the same snapshot (D021).
                let saved_one = one.save_json().unwrap();
                let restored_one = ReactiveSession::restore_json(SYMBOLIC, &saved_one)
                    .unwrap_or_else(|error| {
                        panic!("run {run} step {step}: {trail:?}: restore after the event: {error}")
                    });
                assert_eq!(
                    restored_one.save_json().unwrap(),
                    saved_one,
                    "run {run} step {step}: {trail:?}: save round trip"
                );
                assert_eq!(
                    snapshot(&restored_one),
                    snapshot(&one),
                    "run {run} step {step}: {trail:?}: snapshot after restore"
                );
                // Every event's save restores, to the same snapshot (D021).
                let saved_two = two.save_json().unwrap();
                let restored_two = ReactiveSession::restore_json(BY_HAND, &saved_two)
                    .unwrap_or_else(|error| {
                        panic!("run {run} step {step}: {trail:?}: restore after the event: {error}")
                    });
                assert_eq!(
                    restored_two.save_json().unwrap(),
                    saved_two,
                    "run {run} step {step}: {trail:?}: save round trip"
                );
                assert_eq!(
                    snapshot(&restored_two),
                    snapshot(&two),
                    "run {run} step {step}: {trail:?}: snapshot after restore"
                );
                if expected["outcome"] == "accepted" {
                    accepted += 1;
                }
            }
            let (mut a, mut b) = (snapshot(&one), snapshot(&two));
            comparable(&mut a);
            comparable(&mut b);
            assert_eq!(b, a, "run {run} step {step}: {trail:?}");
        }
    }
    eprintln!("{accepted} accepted events in lockstep");
    assert!(accepted > 3000);
}
