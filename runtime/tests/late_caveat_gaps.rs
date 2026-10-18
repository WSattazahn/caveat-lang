//! The save check for D015 (spec/caveat-check-0.1.md, C009): a value a
//! 1.0.x runtime saved without a late caveat that the evidence it lists now
//! carries, which 1.1 reads live (spec/caveat-caveat-sources-0.1.md).
use caveat_runtime::reactive::{check_save_with, CheckOptions, LateCaveatGap, ReactiveSession};
use serde_json::{json, Value};

/// The D015 case of `runtime/tests/take_back.rs`: `before` is set under
/// `committed(trust)` before the `qualify`, `after` after it, and a binding
/// shows `after`.
const PROGRAM: &str = r#"
caveat flaky consequence material;
evidence e from "tool";
renewable e limit 8;
event report;
event flag;
event unflag;
on report when observed(e) renew e;
on report reveal e;
on flag qualify e with flaky;
on unflag unqualify e with flaky;
state v = 0;
on report set v = qualified(1, e);
event decide;
on decide commit trust because enough using v;
state before = 0;
state after = 0;
event early;
event late;
on early when committed(trust) set before = 1;
on late when committed(trust) set after = 1;
bind view.after = after;
"#;

const RECORDS: CheckOptions = CheckOptions {
    unused: false,
    records: true,
};

fn played(events: &[&str]) -> ReactiveSession {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    for event in events {
        game.dispatch_json(event, "{}").unwrap();
    }
    game
}

/// A 1.1 save of `events`, with `flaky` taken out of `after`'s lineage and
/// grounds: what a 1.0.x runtime wrote for `after` set under
/// `committed(trust)` after the `qualify`.
fn as_one_point_zero_saved(events: &[&str]) -> String {
    let mut save: Value = serde_json::from_str(&played(events).save_json().unwrap()).unwrap();
    let state = &mut save["states"]["after"];
    let mut removed = false;
    for part in ["lineage", "grounds"] {
        let Some(part) = state.get_mut(part).and_then(Value::as_object_mut) else {
            continue;
        };
        if let Some(caveats) = part.get_mut("caveats").and_then(Value::as_array_mut) {
            let before = caveats.len();
            caveats.retain(|caveat| caveat != "flaky");
            removed |= caveats.len() < before;
            // A save leaves an empty list out.
            if caveats.is_empty() {
                part.remove("caveats");
            }
        }
    }
    assert!(removed, "the save held flaky on after: {}", save["states"]);
    save.to_string()
}

#[test]
fn a_session_1_1_plays_holds_no_gap() {
    let game = played(&["report", "decide", "early", "flag", "late"]);
    assert_eq!(game.late_caveat_gaps(), []);
    let restored = ReactiveSession::restore_json(PROGRAM, &game.save_json().unwrap()).unwrap();
    assert_eq!(restored.late_caveat_gaps(), []);
}

#[test]
fn restore_accepts_a_value_saved_without_the_late_caveat_and_the_gap_names_it() {
    let events = ["report", "decide", "early", "flag", "late"];
    let saved = as_one_point_zero_saved(&events);
    // Restore keeps the value as saved: it is a caveat restore allows a value
    // to lack, and no repair is done on load.
    let restored = ReactiveSession::restore_json(PROGRAM, &saved).unwrap();
    let shown = serde_json::to_value(restored.snapshot()).unwrap();
    assert_eq!(
        shown["qualified_values"]["after"]["provenance"]["caveats"],
        json!([])
    );
    let gap = |place: &'static str, name: &str| LateCaveatGap {
        place,
        name: name.into(),
        caveat: "flaky".into(),
        evidence: vec!["e".into()],
    };
    assert_eq!(
        restored.late_caveat_gaps(),
        [gap("state", "after"), gap("binding", "view.after")]
    );

    // Setting the value again on 1.1 closes the gap.
    let mut continued = restored.clone();
    continued.dispatch_json("late", "{}").unwrap();
    assert_eq!(continued.late_caveat_gaps(), []);
    // So does taking the caveat back: the relation is no longer live.
    let mut taken_back = restored;
    taken_back.dispatch_json("unflag", "{}").unwrap();
    assert_eq!(taken_back.late_caveat_gaps(), []);
}

#[test]
fn the_save_check_reports_each_value_at_its_declaration() {
    let saved = as_one_point_zero_saved(&["report", "decide", "early", "flag", "late"]);
    let report = check_save_with(PROGRAM, &saved, RECORDS).unwrap();
    let line = |prefix: &str| {
        PROGRAM
            .lines()
            .position(|line| line.starts_with(prefix))
            .unwrap()
            + 1
    };
    let found = report
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.line))
        .collect::<Vec<_>>();
    assert_eq!(
        found,
        [
            ("C009", line("state after")),
            ("C008", line("on early")),
            ("C008", line("on late")),
            ("C009", line("bind view.after")),
        ]
    );
    let after = &report.diagnostics[0];
    assert_eq!(after.name, "save-lacks-late-caveat");
    assert!(
        after
            .message
            .contains("the state `after` without `flaky`, which qualifies `e` (in its value"),
        "{}",
        after.message
    );
    assert_eq!(
        after
            .related
            .iter()
            .map(|related| (related.line, related.note.as_str()))
            .collect::<Vec<_>>(),
        [(line("on flag"), "`qualify e with flaky`")]
    );

    // A save 1.1 wrote: C008 for the program, no C009.
    let clean = played(&["report", "decide", "early", "flag", "late"])
        .save_json()
        .unwrap();
    let report = check_save_with(PROGRAM, &clean, RECORDS).unwrap();
    assert!(report
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code == "C008"));
}

#[test]
fn the_save_check_is_part_of_the_records_check_and_needs_a_save_that_restores() {
    let saved = played(&["report"]).save_json().unwrap();
    assert!(check_save_with(PROGRAM, &saved, CheckOptions::default()).is_err());
    let error = check_save_with(PROGRAM, "{}", RECORDS).unwrap_err();
    assert!(error.contains("cannot restore save"), "{error}");
    let other = PROGRAM.replace("state before = 0;", "state before = 1;");
    let error = check_save_with(&other, &saved, RECORDS).unwrap_err();
    assert!(error.contains("different program"), "{error}");
}

/// After every event, and after a save and restore at random points, a
/// session that 1.1 played holds no value without a late caveat its
/// evidence carries: the save check is empty for every save 1.1 writes.
/// Values are read through a commitment's basis, a decision series' latest
/// revision and its history, a reading stream, and a binding, while the
/// evidence they list is qualified, taken back, renewed and withdrawn.
#[test]
fn random_sequences_on_1_1_leave_no_gap() {
    const SOURCE: &str = r#"
caveat flaky consequence material;
claim c;
evidence e from "tool";
renewable e limit 64;
evidence probe from "probe";
readings s from probe limit 64;
evidence op from "operator check";
decisions trust limit 64;
budget 100000;
state v = 0;
state recalled = 0;
state latest_trust = 0;
state counted = 0;
state first = 0;
state sampled = 0;
event report;
event flag;
event unflag;
event decide;
event doubt;
event recall;
event read;
event see_op;
event withdraw;
on report when observed(e) renew e;
on report reveal e;
on report set v = qualified(1, e);
on flag when observed(e) qualify e with flaky;
on unflag unqualify e with flaky;
on decide when observed(e) commit trust because enough using v;
on doubt when committed(trust) and observed(e) reopen trust because e;
on recall when committed(trust) set recalled = 1;
on recall when committed(trust) set latest_trust = latest(trust);
on recall set counted = history_count(trust);
on recall when history_count(trust) > 0 set first = history_at(trust, 0);
on read when observed(e) sample s = 1 supports c;
on read when has_sample(s) set sampled = latest(s);
on see_op reveal op;
on withdraw when observed(op) and observed(e) withdraw e because op;
bind view.recalled = recalled;
bind view.reopened = if(reopened(trust), 1, 0);
bind view.first = first + latest_trust;
"#;
    const EVENTS: [&str; 10] = [
        "report", "flag", "unflag", "decide", "doubt", "recall", "read", "see_op", "withdraw",
        "save",
    ];
    let mut seed: u64 = 0x5851_f42d_4c95_7f2d;
    let mut gained = 0;
    for run in 0..300 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        for _ in 0..30 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let event = EVENTS[(seed >> 33) as usize % EVENTS.len()];
            trail.push(event);
            if event == "save" {
                let saved = game.save_json().unwrap();
                game = ReactiveSession::restore_json(SOURCE, &saved)
                    .unwrap_or_else(|error| panic!("run {run} after {trail:?}: {error}"));
            } else if game.dispatch_json(event, "{}").is_err() {
                continue;
            }
            assert_eq!(game.late_caveat_gaps(), [], "run {run} after {trail:?}");
            let shown = serde_json::to_value(game.snapshot()).unwrap();
            if shown["qualified_values"]["recalled"]["provenance"]["caveats"]
                .as_array()
                .is_some_and(|caveats| caveats.iter().any(|caveat| caveat == "flaky"))
            {
                gained += 1;
            }
        }
    }
    // The sequences reach the D015 case: a value read through a basis
    // carries the late caveat.
    assert!(gained > 0);
}
