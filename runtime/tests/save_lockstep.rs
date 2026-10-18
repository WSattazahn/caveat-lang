// Replay Save 0.1 and the save: a session restored from its state save, and
// one restored from its compact save, are the session that wrote them. Three
// sessions run in lockstep over random events, forked again from the
// original's saves at random points, and must agree on every outcome and
// every save, byte for byte. A compact save is never larger than the state
// save, and `restore` takes either form.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const SOURCE: &str = r#"
caveat flaky consequence material;
caveat late consequence low;
claim ok;
evidence e from "tool";
renewable e limit 4;
evidence f from "operator";
evidence g from "grant";
evidence h from "sensor";
readings s from f limit 6;
readings t from h limit 4;
decisions d limit 5 reopened by s opposing ok;
decisions p limit 3;
identifiers limit 3;
state a = 0;
state b = 0;
state held = 0;
state ticks = 0;
event good n min 0 max 50;
event push commit id;
event see_e;
event see_f;
event see_g;
event new_e;
event read_for;
event read_against;
event read_e;
event decide;
event decide_cite;
event approve;
event reconsider;
event reconsider_latest;
event doubt;
event undoubt;
event retract_s;
event retract_e;
event later;
event hold;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on good set a = a + n;
on push set held = commit;
on see_e reveal e supports ok;
on see_f reveal f;
on see_g reveal g;
on new_e renew e;
on read_for sample s = a supports ok;
on read_against sample s = a opposes ok;
on read_e when observed(e) sample t = b + 1 supports ok;
on decide when observed(e) commit d because enough using qualified(a, e);
on decide_cite when has_sample(s) commit d because budget citing latest(s), a;
on approve when has_sample(t) commit p because deadline using latest(t) permitted by g;
on reconsider when committed(d) reopen d because e;
on reconsider_latest when committed(d) and has_sample(s) reopen d because latest(s);
on doubt when observed(e) qualify e with flaky;
on undoubt unqualify e with flaky;
on retract_s when has_sample(s) withdraw latest(s) because f;
on retract_e when observed(e) withdraw e because f;
on later when observed(e) qualify e with late after 0.2;
on hold when observed(e) set b = qualified(a, e) + if(rests_on_withdrawn(d), 100, 0);
on tick set ticks = ticks + 1;
bind view.a = a;
bind view.b = b;
bind view.decided = if(committed(d), 1, 0);
bind view.reopened = if(reopened(d), 1, 0) when committed(d);
bind view.stale = if(rests_on_withdrawn(d), 1, 0) when committed(d);
bind view.flagged = if(has_caveat(b, flaky), 1, 0);
bind view.count = history_count(s);
"#;

const EVENTS: [(&str, &str); 24] = [
    ("good", r#"{"n": 2}"#),
    ("good", r#"{"n": 7}"#),
    ("push", r#"{"commit": "one"}"#),
    ("push", r#"{"commit": "two"}"#),
    ("see_e", "{}"),
    ("see_f", "{}"),
    ("see_g", "{}"),
    ("new_e", "{}"),
    ("read_for", "{}"),
    ("read_against", "{}"),
    ("read_e", "{}"),
    ("decide", "{}"),
    ("decide_cite", "{}"),
    ("approve", "{}"),
    ("reconsider", "{}"),
    ("reconsider_latest", "{}"),
    ("doubt", "{}"),
    ("undoubt", "{}"),
    ("retract_s", "{}"),
    ("retract_e", "{}"),
    ("later", "{}"),
    ("hold", "{}"),
    ("tick", r#"{"dt": 0.1}"#),
    ("tick", r#"{"dt": 0.05}"#),
];

fn outcome(game: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    serde_json::to_value(
        game.dispatch_outcome_json(event, payload)
            .unwrap_or_else(|fatal| panic!("{event} {payload}: fatal {}", fatal.message)),
    )
    .unwrap()
}

#[test]
fn the_lockstep_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn sessions_restored_from_state_and_compact_saves_run_in_lockstep() {
    let mut seed: u64 = 0x7f4a_7c15_9e37_79b9;
    let mut forks = 0;
    let mut replay_saves = 0;
    let mut accepted = 0;
    for run in 0..200 {
        let mut original = ReactiveSession::from_source(SOURCE).unwrap();
        let mut from_state: Option<ReactiveSession> = None;
        let mut from_compact: Option<ReactiveSession> = None;
        let mut trail = Vec::new();
        for step in 0..48 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 4);
            if pick >= EVENTS.len() {
                // Fork again from the original's saves, as a host that
                // stopped here and started again from either would.
                trail.push(("fork", ""));
                forks += 1;
                let state = original.save_json().unwrap();
                let compact = original.save_compact_json().unwrap();
                assert!(
                    compact.len() <= state.len(),
                    "run {run} step {step}: compact {} > state {}: {trail:?}",
                    compact.len(),
                    state.len()
                );
                if compact.starts_with("{\"schema\":\"caveat-replay-save/0.1\"") {
                    replay_saves += 1;
                }
                let restored =
                    ReactiveSession::restore_json(SOURCE, &state).unwrap_or_else(|error| {
                        panic!("run {run} step {step} state restore: {error}: {trail:?}")
                    });
                assert_eq!(
                    restored.save_json().unwrap(),
                    state,
                    "run {run} step {step} state save after restore: {trail:?}"
                );
                from_state = Some(restored);
                let restored =
                    ReactiveSession::restore_json(SOURCE, &compact).unwrap_or_else(|error| {
                        panic!("run {run} step {step} compact restore: {error}: {trail:?}")
                    });
                assert_eq!(
                    restored.save_json().unwrap(),
                    state,
                    "run {run} step {step} state save after compact restore: {trail:?}"
                );
                // The same session gives the same compact save every time.
                assert_eq!(
                    original.save_compact_json().unwrap(),
                    compact,
                    "run {run} step {step} compact save twice: {trail:?}"
                );
                from_compact = Some(restored);
                continue;
            }
            let (event, payload) = EVENTS[pick];
            trail.push((event, payload));
            let expected = outcome(&mut original, event, payload);
            if expected["outcome"] == "accepted" {
                accepted += 1;
            }
            let save = original.save_json().unwrap();
            for (name, fork) in [("state", &mut from_state), ("compact", &mut from_compact)] {
                let Some(fork) = fork.as_mut() else { continue };
                let actual = outcome(fork, event, payload);
                assert_eq!(actual, expected, "run {run} step {step}: a session restored from the {name} save answered differently: {trail:?}");
                assert_eq!(fork.save_json().unwrap(), save, "run {run} step {step}: a session restored from the {name} save saves differently: {trail:?}");
            }
        }
    }
    eprintln!(
        "{accepted} accepted events, {forks} forks, {replay_saves} of them with a replay save"
    );
    assert!(
        replay_saves > 0,
        "no fork ever had a replay save; the alphabet keeps sessions too large"
    );
}

/// Past the replay save's 4,096 bytes of accepted events, the compact save is
/// the state save, and a session restored from it still runs in lockstep.
#[test]
fn a_session_past_the_replay_bound_saves_state_and_runs_in_lockstep() {
    let mut seed: u64 = 0x1234_5678_9abc_def1;
    for run in 0..3 {
        let mut original = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        let mut was_replay = false;
        let mut crossed_at = None;
        for step in 0..700 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let (event, payload) = EVENTS[(seed >> 33) as usize % EVENTS.len()];
            trail.push((event, payload));
            outcome(&mut original, event, payload);
            if step % 25 == 0 {
                let compact = original.save_compact_json().unwrap();
                let replay = compact.starts_with("{\"schema\":\"caveat-replay-save/0.1\"");
                if replay {
                    was_replay = true;
                } else if was_replay && crossed_at.is_none() {
                    crossed_at = Some(step);
                }
                // Once outgrown, it stays a state save.
                assert!(
                    replay || crossed_at.is_some() || !was_replay,
                    "run {run} step {step}"
                );
            }
        }
        let state = original.save_json().unwrap();
        let compact = original.save_compact_json().unwrap();
        assert_eq!(
            compact, state,
            "run {run}: past the bound the compact save is the state save"
        );
        let crossed = crossed_at.unwrap_or_else(|| {
            panic!("run {run}: the session never outgrew its replay save in 700 events")
        });
        let mut restored = ReactiveSession::restore_json(SOURCE, &compact).unwrap();
        for step in 0..40 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let (event, payload) = EVENTS[(seed >> 33) as usize % EVENTS.len()];
            let expected = outcome(&mut original, event, payload);
            assert_eq!(
                outcome(&mut restored, event, payload),
                expected,
                "run {run} after step {step}, crossed at {crossed}"
            );
            assert_eq!(
                restored.save_json().unwrap(),
                original.save_json().unwrap(),
                "run {run} after step {step}"
            );
        }
        eprintln!("run {run}: outgrew the replay save by event {crossed}");
    }
}
