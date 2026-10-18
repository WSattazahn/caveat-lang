// Dispatch outcomes 0.1, "Atomicity": every `rejected` outcome preserves the
// complete save, view and snapshot, including sequence, clock, pending
// qualifications, state, graph, effects and decision history, and successful
// work before the refusal does not survive. Random event sequences over a
// program whose events reach every classified refusal site the catalog names
// after doing some work first, with saves and restores interleaved.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// Each `work_*` event changes `a` and `c` before the effect that may refuse
/// it, so a refusal that kept the earlier effects would show in `a`.
const SOURCE: &str = r#"
caveat flaky consequence material;
caveat shaky consequence low;
claim ok;
evidence e from "tool";
renewable e limit 3;
evidence f from "other";
evidence g from "grant";
readings s from f limit 3;
decisions d limit 2;
identifiers limit 2;
state a = 0;
state b = 0 min 0 max 10;
state c = 0;
state head = 0;
state ticks = 0;
event good n min 0 max 100;
event tight n min 0 max 5;
event push commit id;
event see_e;
event see_f;
event see_g;
event new_e;
event read;
event decide;
event reconsider;
event doubt;
event undoubt;
event retract;
event later;
event tick dt min 0 max 0.1;
clock tick every 0.1;
event work_reject;
event work_unobserved;
event work_bound;
event work_divide;
event work_require;
event work_cite;
event work_permit;
event work_in_force;
on good set a = a + n;
on tight set a = a + n;
on push set head = commit;
on see_e reveal e supports ok;
on see_f reveal f;
on see_g reveal g;
on new_e renew e;
on read sample s = a supports ok;
on decide when observed(e) commit d because enough using qualified(a, e);
on reconsider reopen d because e;
on doubt qualify e with flaky;
on undoubt unqualify e with flaky;
on retract withdraw latest(s) because f;
on later qualify e with shaky after 0.2;
on tick set ticks = ticks + 1;
on work_reject set a = a + 1;
on work_reject set c = c + 1;
on work_reject when a > 0 reject "no";
on work_unobserved set a = a + 1;
on work_unobserved set c = qualified(1, e);
on work_bound set a = a + 1;
on work_bound set b = b + 7;
on work_divide set a = a + 1;
on work_divide set c = 1 / (a - a);
on work_require set a = a + 1;
on work_require set c = require(a < 0, 1);
on work_cite set a = a + 1;
on work_cite set c = c + 1 because latest(s);
on work_permit set a = a + 1;
on work_permit commit d because enough using a permitted by g;
on work_in_force set a = a + 1;
on work_in_force commit d because enough using a;
on work_in_force commit d because enough using a;
bind view.a = a;
bind view.c = c;
bind view.flagged = if(carries(e, flaky), 1, 0);
bind view.decided = if(committed(d), 1, 0);
"#;

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn view(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.view()).unwrap()
}

#[test]
fn the_refusal_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

/// (event, payload) pairs: most take the alphabet to a refusal site in some
/// states and to an accepted event in others, so both outcomes are frequent.
const EVENTS: [(&str, &str); 30] = [
    ("good", r#"{"n": 3}"#),
    ("good", r#"{"n": 101}"#),
    ("good", r#"{}"#),
    ("good", r#"{"n": 3, "m": 1}"#),
    ("tight", r#"{"n": 7}"#),
    ("push", r#"{"commit": "x"}"#),
    ("push", r#"{"commit": "y"}"#),
    ("push", r#"{"commit": "z"}"#),
    ("see_e", "{}"),
    ("see_f", "{}"),
    ("see_g", "{}"),
    ("new_e", "{}"),
    ("read", "{}"),
    ("decide", "{}"),
    ("reconsider", "{}"),
    ("doubt", "{}"),
    ("undoubt", "{}"),
    ("retract", "{}"),
    ("later", "{}"),
    ("tick", r#"{"dt": 0.1}"#),
    ("tick", r#"{"dt": 0.5}"#),
    ("nothing", "{}"),
    ("work_reject", "{}"),
    ("work_unobserved", "{}"),
    ("work_bound", "{}"),
    ("work_divide", "{}"),
    ("work_require", "{}"),
    ("work_cite", "{}"),
    ("work_permit", "{}"),
    ("work_in_force", "{}"),
];

#[test]
fn a_rejected_event_leaves_save_view_and_snapshot_byte_for_byte() {
    let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
    let mut rejected = 0;
    let mut accepted = 0;
    let mut codes = std::collections::BTreeMap::<String, usize>::new();
    for run in 0..300 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        for step in 0..40 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                // A save and restore in place: the restored session must be
                // the same session, save for save.
                trail.push(("save", ""));
                let before = game.save_json().unwrap();
                let restored =
                    ReactiveSession::restore_json(SOURCE, &before).unwrap_or_else(|error| {
                        panic!("run {run} step {step} restore after {trail:?}: {error}")
                    });
                assert_eq!(
                    restored.save_json().unwrap(),
                    before,
                    "run {run} step {step} save after restore, {trail:?}"
                );
                assert_eq!(
                    snapshot(&restored),
                    snapshot(&game),
                    "run {run} step {step} snapshot after restore, {trail:?}"
                );
                assert_eq!(
                    view(&restored),
                    view(&game),
                    "run {run} step {step} view after restore, {trail:?}"
                );
                game = restored;
                continue;
            }
            let (event, payload) = EVENTS[pick];
            trail.push((event, payload));
            let save_before = game.save_json().unwrap();
            let snapshot_before = snapshot(&game);
            let view_before = view(&game);
            let sequence_before = snapshot_before["sequence"].as_u64().unwrap();
            let outcome = game
                .dispatch_outcome_json(event, payload)
                .unwrap_or_else(|fatal| {
                    panic!("run {run} step {step} fatal {}: {trail:?}", fatal.message)
                });
            let outcome = serde_json::to_value(outcome).unwrap();
            if outcome["outcome"] == "accepted" {
                accepted += 1;
                assert_eq!(
                    outcome["snapshot"]["sequence"],
                    sequence_before + 1,
                    "run {run} step {step}: {trail:?}"
                );
            } else {
                rejected += 1;
                let site = format!(
                    "{}/{}",
                    outcome["origin"].as_str().unwrap(),
                    outcome["code"].as_str().unwrap()
                );
                *codes.entry(site.clone()).or_default() += 1;
                let detail = format!(
                    "run {run} step {step}, {site} {}: {trail:?}",
                    outcome["message"]
                );
                assert_eq!(
                    game.save_json().unwrap(),
                    save_before,
                    "save after refusal, {detail}"
                );
                assert_eq!(
                    snapshot(&game),
                    snapshot_before,
                    "snapshot after refusal, {detail}"
                );
                assert_eq!(view(&game), view_before, "view after refusal, {detail}");
            }
        }
    }
    eprintln!("accepted {accepted}, rejected {rejected}: {codes:?}");
    // The alphabet must reach the catalog's sites, or the check proves little.
    for site in [
        "policy/reject",
        "policy/not_permitted",
        "input/unknown_event",
        "input/payload_invalid",
        "input/bound_exceeded",
        "evaluation/bound_exceeded",
        "evaluation/decision_in_force",
        "evaluation/ungrounded_citation",
        "evaluation/unobserved_evidence",
        "evaluation/not_committed",
        "evaluation/expression",
        "evaluation/requirement_failed",
        "limit/history_limit",
        "limit/identifier_limit",
        "limit/renewal_limit",
    ] {
        assert!(
            codes.keys().any(|key| key == site),
            "the random check never reached {site}: {codes:?}"
        );
    }
}
