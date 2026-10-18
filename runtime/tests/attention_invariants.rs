// Attention under random events (Reactive 0.1 and 0.2, Dispatch 0.1): an
// `examine` spends its cost from the declared budget and marks the caveat
// examined, every time it runs, so a guard of `not examined(C)` is what pays
// once; an examination the remaining budget cannot pay refuses the event as
// limit/attention_limit and rolls back the whole event, including spending
// an earlier rule did, as any later refusal rolls back earlier spending; the
// budget is never replenished; examination discharges nothing; `examined`
// reads the live attention; the save records exactly the examined caveats;
// and a restored session is the one that saved. Checked after every event,
// with saves and restores in place.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeSet;

const BUDGET: u64 = 20;

const SOURCE: &str = r#"
budget 20;
claim safe;
evidence e from "sensor";
caveat a consequence low;
caveat b consequence low;
caveat c consequence material;
caveat d consequence low;
caveat never consequence low;
a qualifies e;
state done = 0;
event ex_a;
event ex_b;
event ex_c;
event ex_d_once;
event ex_both;
event ex_then_reject;
event ex_proc;
event mark;
event see;
event decide;
event reconsider;
proc inspect(x caveat) { examine x cost 2; };
on ex_a examine a cost 1;
on ex_b examine b cost 2;
on ex_c examine c cost 3;
on ex_d_once when not examined(d) examine d cost 4;
on ex_both examine a cost 1;
on ex_both examine b cost 2;
on ex_then_reject examine c cost 2;
on ex_then_reject reject "Looked, then refused.";
on ex_proc call inspect(d);
on mark when examined(a) set done = done + 1;
on see reveal e supports safe;
on decide when observed(e) and not committed(go) commit go because enough retaining a;
on reconsider when committed(go) and not reopened(go) reopen go because e;
bind hud.a = if(examined(a), 1, 0);
bind hud.b = if(examined(b), 1, 0);
bind hud.c = if(examined(c), 1, 0);
bind hud.d = if(examined(d), 1, 0);
bind hud.never = if(examined(never), 1, 0);
"#;

const EVENTS: [&str; 14] = [
    "ex_a",
    "ex_b",
    "ex_c",
    "ex_d_once",
    "ex_both",
    "ex_then_reject",
    "ex_proc",
    "mark",
    "see",
    "decide",
    "reconsider",
    "ex_a",
    "ex_both",
    "mark",
];

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn examined_in(shown: &Value) -> BTreeSet<String> {
    shown["symbols"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|symbol| symbol["attention"] == "examined")
        .map(|symbol| symbol["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn the_attention_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn examination_spends_the_budget_atomically_and_marks_what_it_examined() {
    let mut seed: u64 = 0x62e1_7b0c_3f9a_d415;
    let (mut accepted, mut spends, mut exhausted, mut rolled_back_spending, mut paid_once) =
        (0, 0, 0, 0, 0);
    for run in 0..300 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        let mut spent: u64 = 0;
        let mut examined: BTreeSet<String> = BTreeSet::new();
        let mut done = 0.0;
        for step in 0..40 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                trail.push("save");
                let saved = game.save_json().unwrap();
                let save: Value = serde_json::from_str(&saved).unwrap();
                let recorded: BTreeSet<String> = save["graph"]["attention"]
                    .as_object()
                    .map(|map| {
                        map.iter()
                            .map(|(caveat, state)| {
                                assert_eq!(state, "examined", "run {run} step {step}: {trail:?}");
                                caveat.clone()
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                assert_eq!(
                    recorded, examined,
                    "run {run} step {step}: the save's attention: {trail:?}"
                );
                game = ReactiveSession::restore_json(SOURCE, &saved).unwrap_or_else(|error| {
                    panic!("run {run} step {step} restore: {error}: {trail:?}")
                });
                assert_eq!(
                    snapshot(&game),
                    before,
                    "run {run} step {step} after restore: {trail:?}"
                );
                continue;
            }
            let event = EVENTS[pick];
            trail.push(event);
            let at = format!("run {run} step {step} after {event}: {trail:?}");
            let save_before = game.save_json().unwrap();
            let outcome = serde_json::to_value(
                game.dispatch_outcome_json(event, "{}")
                    .unwrap_or_else(|fatal| panic!("{at}: fatal {}", fatal.message)),
            )
            .unwrap();
            // What the event examines, from the source.
            let (cost, caveats): (u64, &[&str]) = match event {
                "ex_a" => (1, &["a"]),
                "ex_b" => (2, &["b"]),
                "ex_c" => (3, &["c"]),
                "ex_d_once" if !examined.contains("d") => (4, &["d"]),
                "ex_both" => (3, &["a", "b"]),
                "ex_then_reject" => (2, &["c"]),
                "ex_proc" => (2, &["d"]),
                _ => (0, &[]),
            };
            if event == "ex_d_once" && examined.contains("d") {
                paid_once += 1;
            }
            let expected = if spent + cost > BUDGET {
                Some(("limit", "attention_limit"))
            } else if event == "ex_then_reject" {
                Some(("policy", "reject"))
            } else {
                None
            };
            if let Some((origin, code)) = expected {
                assert_eq!(
                    (outcome["origin"].as_str(), outcome["code"].as_str()),
                    (Some(origin), Some(code)),
                    "{at}: {outcome}"
                );
                assert_eq!(
                    game.save_json().unwrap(),
                    save_before,
                    "{at}: a refusal changed the save"
                );
                assert_eq!(
                    snapshot(&game),
                    before,
                    "{at}: a refusal changed the snapshot"
                );
                if code == "attention_limit" {
                    exhausted += 1;
                } else {
                    rolled_back_spending += 1;
                }
                continue;
            }
            if outcome["outcome"] != "accepted" {
                // Only the budget and the policy refuse here; anything else is
                // a guard not met, which accepts the event and does nothing.
                panic!("{at}: {outcome}");
            }
            accepted += 1;
            spent += cost;
            if cost > 0 {
                spends += 1;
            }
            for caveat in caveats {
                examined.insert(caveat.to_string());
            }
            if event == "mark" && examined.contains("a") {
                done += 1.0;
            }
            let shown = snapshot(&game);
            assert_eq!(
                shown["budget"]["spent"].as_u64(),
                Some(spent),
                "{at}: spent"
            );
            assert_eq!(examined_in(&shown), examined, "{at}: examined");
            for caveat in ["a", "b", "c", "d", "never"] {
                assert_eq!(
                    shown["bindings"]["hud"][caveat].as_f64(),
                    Some(if examined.contains(caveat) { 1.0 } else { 0.0 }),
                    "{at}: examined({caveat})"
                );
            }
            assert_eq!(shown["values"]["done"].as_f64(), Some(done), "{at}: done");
            // Examination discharges nothing: the qualifies edge stays, and a
            // retained caveat stays retained.
            assert!(
                shown["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|edge| edge["from"] == "a"
                        && edge["relation"] == "qualifies"
                        && edge["to"] == "e"),
                "{at}: examination removed a qualifies relation"
            );
            if let Some(commitment) = shown["commitments"].as_array().unwrap().first() {
                assert_eq!(
                    commitment["retained"],
                    serde_json::json!(["a"]),
                    "{at}: {commitment}"
                );
            }
            before = shown;
        }
    }
    eprintln!("{accepted} accepted events, {spends} examinations, {exhausted} refused by the budget, {rolled_back_spending} spends rolled back by a later refusal, {paid_once} guarded examinations skipped");
    assert!(
        spends > 2000 && exhausted > 1000 && rolled_back_spending > 200 && paid_once > 300,
        "{spends} / {exhausted} / {rolled_back_spending} / {paid_once}"
    );
}
