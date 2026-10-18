// Repetition 0.1 and Define 0.1 promise source-level equivalences: a `for`
// block expands, member-major, to statements the author could have written by
// hand, and converting one does not change behaviour; a `define` is inlined
// where it is read, with values, lineage and grounds exactly those of the
// expression written there by hand, and appears nowhere in the snapshot.
// Random events drive each program and its hand-written twin in lockstep;
// every outcome and every snapshot, apart from the source's own identity, must
// agree.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const REPEATED: &str = r#"
claim healthy;
caveat noisy consequence low;
evidence calibration from "bench";
place rack kind probe;
entity alpha kind probe at rack;
entity beta kind probe at rack;
entity gamma kind probe at rack;
state total = 0;
event poke target kind probe;
event sweep;
event calibrate;
event doubt;
event report;
for probe as $p {
    evidence $p_reading from "$p";
    state $p_level = 0;
    state $p_seen = 0;
    on poke when target == target.$p set $p_level = $p_level + $index;
    on poke when target == target.$p and observed($p_reading) set $p_seen = $p_seen + 1;
    on sweep when $p_level > 2 reveal $p_reading supports healthy;
    on calibrate when observed($p_reading) set $p_level = qualified($p_level, $p_reading) + qualified(0, calibration);
    on doubt when observed($p_reading) qualify $p_reading with noisy;
    on report set total = total + $p_level;
    bind $p.level = $p_level;
    bind $p.noisy = if(has_caveat($p_level, noisy), 1, 0) when observed($p_reading);
};
on calibrate reveal calibration;
bind view.total = total;
"#;

const UNROLLED: &str = r#"
claim healthy;
caveat noisy consequence low;
evidence calibration from "bench";
place rack kind probe;
entity alpha kind probe at rack;
entity beta kind probe at rack;
entity gamma kind probe at rack;
state total = 0;
event poke target kind probe;
event sweep;
event calibrate;
event doubt;
event report;
    evidence alpha_reading from "alpha";
    state alpha_level = 0;
    state alpha_seen = 0;
    on poke when target == target.alpha set alpha_level = alpha_level + 1;
    on poke when target == target.alpha and observed(alpha_reading) set alpha_seen = alpha_seen + 1;
    on sweep when alpha_level > 2 reveal alpha_reading supports healthy;
    on calibrate when observed(alpha_reading) set alpha_level = qualified(alpha_level, alpha_reading) + qualified(0, calibration);
    on doubt when observed(alpha_reading) qualify alpha_reading with noisy;
    on report set total = total + alpha_level;
    bind alpha.level = alpha_level;
    bind alpha.noisy = if(has_caveat(alpha_level, noisy), 1, 0) when observed(alpha_reading);
    evidence beta_reading from "beta";
    state beta_level = 0;
    state beta_seen = 0;
    on poke when target == target.beta set beta_level = beta_level + 2;
    on poke when target == target.beta and observed(beta_reading) set beta_seen = beta_seen + 1;
    on sweep when beta_level > 2 reveal beta_reading supports healthy;
    on calibrate when observed(beta_reading) set beta_level = qualified(beta_level, beta_reading) + qualified(0, calibration);
    on doubt when observed(beta_reading) qualify beta_reading with noisy;
    on report set total = total + beta_level;
    bind beta.level = beta_level;
    bind beta.noisy = if(has_caveat(beta_level, noisy), 1, 0) when observed(beta_reading);
    evidence gamma_reading from "gamma";
    state gamma_level = 0;
    state gamma_seen = 0;
    on poke when target == target.gamma set gamma_level = gamma_level + 3;
    on poke when target == target.gamma and observed(gamma_reading) set gamma_seen = gamma_seen + 1;
    on sweep when gamma_level > 2 reveal gamma_reading supports healthy;
    on calibrate when observed(gamma_reading) set gamma_level = qualified(gamma_level, gamma_reading) + qualified(0, calibration);
    on doubt when observed(gamma_reading) qualify gamma_reading with noisy;
    on report set total = total + gamma_level;
    bind gamma.level = gamma_level;
    bind gamma.noisy = if(has_caveat(gamma_level, noisy), 1, 0) when observed(gamma_reading);
on calibrate reveal calibration;
bind view.total = total;
"#;

const REPEATED_EVENTS: [(&str, &str); 8] = [
    ("poke", r#"{"target": "alpha"}"#),
    ("poke", r#"{"target": "beta"}"#),
    ("poke", r#"{"target": "gamma"}"#),
    ("poke", r#"{"target": "delta"}"#),
    ("sweep", "{}"),
    ("calibrate", "{}"),
    ("doubt", "{}"),
    ("report", "{}"),
];

const DEFINED: &str = r#"
claim warm;
caveat stale consequence material;
evidence sensor from "probe";
evidence log from "operator";
readings s from sensor limit 8;
decisions heat limit 4;
state a = 0;
state b = 0;
event read v min -10 max 40;
event note;
event decide;
event reconsider;
event doubt;
event hold;
define hot = latest(s) > 20;
define grounded = has_sample(s) and observed(log);
define margin = latest(s) - 20 + a;
on read sample s = v supports warm;
on note reveal log;
on decide when grounded and hot commit heat because enough using margin;
on reconsider when committed(heat) and not hot reopen heat because latest(s);
on doubt when observed(log) qualify log with stale;
on hold when grounded set b = margin + qualified(0, log) because latest(s), a, qualified(0, log);
on hold when not grounded set a = a + 1;
bind view.hot = if(hot, 1, 0) when has_sample(s);
bind view.margin = margin when has_sample(s) because latest(s), a;
bind view.b = b;
"#;

const INLINED: &str = r#"
claim warm;
caveat stale consequence material;
evidence sensor from "probe";
evidence log from "operator";
readings s from sensor limit 8;
decisions heat limit 4;
state a = 0;
state b = 0;
event read v min -10 max 40;
event note;
event decide;
event reconsider;
event doubt;
event hold;
on read sample s = v supports warm;
on note reveal log;
on decide when (has_sample(s) and observed(log)) and (latest(s) > 20) commit heat because enough using (latest(s) - 20 + a);
on reconsider when committed(heat) and not (latest(s) > 20) reopen heat because latest(s);
on doubt when observed(log) qualify log with stale;
on hold when (has_sample(s) and observed(log)) set b = (latest(s) - 20 + a) + qualified(0, log) because latest(s), a, qualified(0, log);
on hold when not (has_sample(s) and observed(log)) set a = a + 1;
bind view.hot = if((latest(s) > 20), 1, 0) when has_sample(s);
bind view.margin = (latest(s) - 20 + a) when has_sample(s) because latest(s), a;
bind view.b = b;
"#;

const DEFINED_EVENTS: [(&str, &str); 9] = [
    ("read", r#"{"v": 25}"#),
    ("read", r#"{"v": 15}"#),
    ("read", r#"{"v": 21}"#),
    ("note", "{}"),
    ("decide", "{}"),
    ("reconsider", "{}"),
    ("doubt", "{}"),
    ("hold", "{}"),
    ("hold", "{}"),
];

/// The snapshot without the source's own identity, which is the one thing
/// two differently written programs may differ in.
fn comparable(game: &ReactiveSession) -> Value {
    let mut shown = serde_json::to_value(game.snapshot()).unwrap();
    let record = shown.as_object_mut().unwrap();
    record.remove("source_id");
    record.remove("source_sha256");
    shown
}

fn outcome(game: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let mut outcome = serde_json::to_value(
        game.dispatch_outcome_json(event, payload)
            .unwrap_or_else(|fatal| panic!("{event} {payload}: fatal {}", fatal.message)),
    )
    .unwrap();
    if let Some(snapshot) = outcome.get_mut("snapshot").and_then(Value::as_object_mut) {
        snapshot.remove("source_id");
        snapshot.remove("source_sha256");
    }
    outcome
}

fn lockstep(
    original: &str,
    twin: &str,
    events: &[(&str, &str)],
    seed_start: u64,
    name: &str,
) -> usize {
    let mut seed = seed_start;
    let mut accepted = 0;
    for run in 0..150 {
        let mut one = ReactiveSession::from_source(original).unwrap();
        let mut two = ReactiveSession::from_source(twin).unwrap();
        assert_eq!(
            comparable(&one),
            comparable(&two),
            "{name}: the fresh sessions differ"
        );
        let mut trail = Vec::new();
        for step in 0..40 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (events.len() + 2);
            if pick >= events.len() {
                trail.push(("save", ""));
                let saved = one.save_json().unwrap();
                one = ReactiveSession::restore_json(original, &saved).unwrap();
                let saved = two.save_json().unwrap();
                two = ReactiveSession::restore_json(twin, &saved).unwrap();
                assert_eq!(
                    comparable(&one),
                    comparable(&two),
                    "{name} run {run} step {step} after restores: {trail:?}"
                );
                continue;
            }
            let (event, payload) = events[pick];
            trail.push((event, payload));
            let expected = outcome(&mut one, event, payload);
            let actual = outcome(&mut two, event, payload);
            assert_eq!(
                actual, expected,
                "{name} run {run} step {step}: outcomes differ: {trail:?}"
            );
            if expected["outcome"] == "accepted" {
                accepted += 1;
            }
            assert_eq!(
                comparable(&two),
                comparable(&one),
                "{name} run {run} step {step}: snapshots differ: {trail:?}"
            );
        }
    }
    accepted
}

#[test]
fn the_twins_load() {
    for source in [REPEATED, UNROLLED, DEFINED, INLINED] {
        ReactiveSession::from_source(source).unwrap();
    }
}

#[test]
fn a_for_block_behaves_as_its_hand_unrolled_twin() {
    let accepted = lockstep(
        REPEATED,
        UNROLLED,
        &REPEATED_EVENTS,
        0x6a09_e667_f3bc_c908,
        "repetition",
    );
    eprintln!("repetition: {accepted} accepted events in lockstep");
    assert!(accepted > 2000);
}

#[test]
fn a_define_behaves_as_its_inlined_twin() {
    let accepted = lockstep(
        DEFINED,
        INLINED,
        &DEFINED_EVENTS,
        0xbb67_ae85_84ca_a73b,
        "define",
    );
    eprintln!("define: {accepted} accepted events in lockstep");
    assert!(accepted > 2000);
}
