//! `decisions NAME limit 1 irreversible [forecloses ACTION, …]`: a decision
//! that can't be undone, and the options it closes off. See
//! spec/caveat-foreclosure-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::web::WebReactiveSession;
use serde_json::Value;

/// The design note's migration: dropping the old table can't be undone, and
/// it closes the rollback and a plain `undo`.
const MIGRATION: &str = r#"
claim migration_safe;
evidence full_check from "full migration dry run";
evidence sample_check from "sampled migration dry run";
evidence approvals from "owner";
evidence recheck from "a second look at the dry run";
caveat sampled_only consequence high;
caveat timing_noise consequence low;
sampled_only qualifies sample_check;
timing_noise qualifies full_check;
readings full_runs from full_check limit 8;
readings sampled_runs from sample_check limit 8;
readings grants from approvals limit 4;
decisions drop_old_table limit 1 irreversible forecloses rollback, undo reopened by full_runs opposing migration_safe;
decisions rollback limit 4;
event dry_run rows_ok min 0 max 1, full in sampled complete;
event approve;
event drop;
event drop_accepting;
event roll_back;
event undo_it;
event roll_back_then_drop;
event drop_then_roll_back;
event misread;
on dry_run when full == full.complete and rows_ok == 1 sample full_runs = 1 supports migration_safe;
on dry_run when full == full.complete and rows_ok == 0 sample full_runs = 0 opposes migration_safe;
on dry_run when full == full.sampled sample sampled_runs = rows_ok supports migration_safe;
on approve sample grants = 1 supports migration_safe;
on drop commit drop_old_table because enough
  using if(has_sample(full_runs), latest(full_runs), latest(sampled_runs))
  permitted by latest(grants);
on drop_accepting commit drop_old_table because enough
  using latest(sampled_runs) permitted by latest(grants) retaining sampled_only;
on roll_back when not committed(rollback) or reopened(rollback) commit rollback because enough using 1;
on undo_it commit undo because enough;
on roll_back_then_drop commit rollback because enough using 2;
on roll_back_then_drop commit drop_old_table because enough
  using latest(full_runs) permitted by latest(grants);
on drop_then_roll_back commit drop_old_table because enough
  using latest(full_runs) permitted by latest(grants);
on drop_then_roll_back commit rollback because enough using 3;
on misread when not observed(recheck) reveal recheck opposes migration_safe;
on misread withdraw latest(full_runs) because recheck;
"#;

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn session(source: &str) -> WebReactiveSession {
    WebReactiveSession::new(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn accept(game: &mut WebReactiveSession, event: &str, payload: &str) -> Value {
    let result = json(&game.dispatch_outcome(event, payload).unwrap());
    assert_eq!(result["outcome"], "accepted", "{event} {payload}: {result}");
    result
}

/// Refused with `code`, and nothing changed: snapshot and save are as before.
fn refuse(game: &mut WebReactiveSession, event: &str, code: &str) -> String {
    let before = (game.snapshot(), game.save().unwrap());
    let result = json(&game.dispatch_outcome(event, "{}").unwrap());
    assert_eq!(result["outcome"], "rejected", "{event}: {result}");
    assert_eq!(result["origin"], "evaluation", "{event}: {result}");
    assert_eq!(result["code"], code, "{event}: {result}");
    assert_eq!((game.snapshot(), game.save().unwrap()), before, "{event}");
    result["message"].as_str().unwrap().to_string()
}

const COMPLETE: &str = r#"{"rows_ok":1,"full":"complete"}"#;
const SAMPLED: &str = r#"{"rows_ok":1,"full":"sampled"}"#;

#[test]
fn the_migration_drop_forecloses_rollback() {
    let mut game = session(MIGRATION);
    accept(&mut game, "dry_run", COMPLETE);
    accept(&mut game, "approve", "{}");
    // A low caveat on the grounds is not read.
    accept(&mut game, "drop", "{}");
    let message = refuse(&mut game, "roll_back", "foreclosed");
    assert_eq!(
        message,
        "event roll_back, rule 7: rollback is foreclosed by drop_old_table@1"
    );
    // A plain commitment is foreclosed the same way.
    let message = refuse(&mut game, "undo_it", "foreclosed");
    assert!(
        message.ends_with("undo is foreclosed by drop_old_table@1"),
        "{message}"
    );
    let snapshot = json(&game.snapshot());
    assert_eq!(
        snapshot["irreversible"],
        serde_json::json!({"drop_old_table": ["rollback", "undo"]})
    );
    assert_eq!(
        snapshot["commitment_grounds"]["drop_old_table@1"]["caveats"],
        serde_json::json!(["timing_noise"])
    );
}

#[test]
fn an_irreversible_commit_names_the_caveats_it_accepts() {
    let mut game = session(MIGRATION);
    accept(&mut game, "dry_run", SAMPLED);
    accept(&mut game, "approve", "{}");
    let message = refuse(&mut game, "drop", "caveated_irreversible");
    assert!(
        message.ends_with(
            "irreversible drop_old_table rests on sampled_only (high), which its commit does not retain"
        ),
        "{message}"
    );
    // Nothing was committed, so rollback stays open.
    accept(&mut game, "roll_back", "{}");
    // Retaining the caveat decides with it attached, on purpose.
    accept(&mut game, "drop_accepting", "{}");
    let snapshot = json(&game.snapshot());
    assert_eq!(
        snapshot["commitment_grounds"]["drop_old_table@1"]["caveats"],
        serde_json::json!(["sampled_only"])
    );
}

#[test]
fn permission_is_checked_before_the_caveats() {
    let mut game = session(MIGRATION);
    accept(&mut game, "dry_run", SAMPLED);
    let result = json(&game.dispatch_outcome("drop", "{}").unwrap());
    assert_eq!(result["outcome"], "rejected");
    assert_ne!(result["code"], "caveated_irreversible", "{result}");
}

#[test]
fn withdrawn_evidence_never_grounds_an_irreversible_commit() {
    let mut game = session(MIGRATION);
    accept(&mut game, "dry_run", COMPLETE);
    accept(&mut game, "approve", "{}");
    accept(&mut game, "misread", "{}");
    let message = refuse(&mut game, "drop", "caveated_irreversible");
    assert!(message.contains("withdrawn (material)"), "{message}");
}

#[test]
fn reopening_an_irreversible_decision_keeps_its_foreclosure() {
    let mut game = session(MIGRATION);
    accept(&mut game, "dry_run", COMPLETE);
    accept(&mut game, "approve", "{}");
    accept(&mut game, "drop", "{}");
    // An opposing complete run triggers the declared reopening.
    accept(&mut game, "dry_run", r#"{"rows_ok":0,"full":"complete"}"#);
    let snapshot = json(&game.snapshot());
    let journal = snapshot["decision_journal"].as_array().unwrap();
    assert_eq!(journal.last().unwrap()["change"], "reopened");
    assert_eq!(journal.last().unwrap()["commitment"], "drop_old_table@1");
    refuse(&mut game, "roll_back", "foreclosed");
    // The record can be questioned; the act can't be repeated.
    let result = json(&game.dispatch_outcome("drop", "{}").unwrap());
    assert_eq!(result["outcome"], "rejected");
    assert_eq!(result["code"], "history_limit");
}

#[test]
fn commits_run_in_rule_order_within_an_event() {
    let mut game = session(MIGRATION);
    accept(&mut game, "dry_run", COMPLETE);
    accept(&mut game, "approve", "{}");
    let message = refuse(&mut game, "drop_then_roll_back", "foreclosed");
    assert!(
        message.ends_with("rollback is foreclosed by drop_old_table@1"),
        "{message}"
    );
    // A rollback before the drop, in the same event, is accepted and kept.
    accept(&mut game, "roll_back_then_drop", "{}");
    let snapshot = json(&game.snapshot());
    assert_eq!(
        snapshot["decision_series"]["rollback"]["current"],
        "rollback@1"
    );
    assert_eq!(
        snapshot["decision_series"]["drop_old_table"]["current"],
        "drop_old_table@1"
    );
}

#[test]
fn saves_restore_and_restore_checks_the_order() {
    let mut game = ReactiveSession::from_source(MIGRATION).unwrap();
    for (event, payload) in [
        ("dry_run", COMPLETE),
        ("approve", "{}"),
        ("roll_back", "{}"),
        ("drop", "{}"),
    ] {
        game.dispatch_json(event, payload)
            .unwrap_or_else(|error| panic!("{event}: {error}"));
    }
    let saved = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(MIGRATION, &saved).unwrap();
    assert_eq!(restored.save_json().unwrap(), saved);
    assert!(
        !saved.contains("irreversible"),
        "the save holds nothing new"
    );
    // Move the rollback after the drop: a record this program can't hold.
    let mut save = json(&saved);
    let drop_sequence =
        save["decision_series"]["drop_old_table"]["revisions"][0]["sequence"].clone();
    let rollback_sequence = save["decision_series"]["rollback"]["revisions"][0]["sequence"].clone();
    save["decision_series"]["rollback"]["revisions"][0]["sequence"] = drop_sequence.clone();
    save["decision_series"]["drop_old_table"]["revisions"][0]["sequence"] =
        rollback_sequence.clone();
    for entry in save["decision_journal"].as_array_mut().unwrap() {
        if entry["commitment"] == "rollback@1" {
            entry["sequence"] = drop_sequence.clone();
        } else if entry["commitment"] == "drop_old_table@1" {
            entry["sequence"] = rollback_sequence.clone();
        }
    }
    let error = ReactiveSession::restore_json(MIGRATION, &save.to_string())
        .map(|_| ())
        .expect_err("a rollback after the drop restored");
    assert!(
        error.contains("rollback@1 is committed after drop_old_table@1, which forecloses it"),
        "{error}"
    );
}

#[test]
fn check_does_not_ask_an_irreversible_series_for_a_reopening_path() {
    let source = r#"
claim ok;
evidence e from "e";
readings r from e limit 2;
decisions once limit 1 irreversible;
decisions fixed limit 1;
event go;
on go sample r = 1 supports ok;
on go commit once because enough using latest(r);
on go commit fixed because enough using latest(r);
"#;
    let report = json(&WebReactiveSession::check(source).unwrap());
    let flagged = report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|diagnostic| diagnostic["code"] == "C002")
        .map(|diagnostic| diagnostic["message"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(flagged.len(), 1, "{flagged:?}");
    assert!(flagged[0].starts_with("`fixed`"), "{flagged:?}");
}

#[test]
fn a_procedure_commits_an_irreversible_series() {
    let source = r#"
claim ok;
evidence e from "e";
caveat shaky consequence material;
shaky qualifies e;
readings r from e limit 2;
decisions publish limit 1 irreversible forecloses retract;
decisions retract limit 2;
event go;
event back;
proc decide(d decisions) { commit d because enough using latest(r) retaining shaky; };
on go sample r = 1 supports ok;
on go call decide(publish);
on back commit retract because enough using 1;
"#;
    let mut game = session(source);
    accept(&mut game, "go", "{}");
    refuse(&mut game, "back", "foreclosed");
}

#[test]
fn the_words_reserve_nothing() {
    let source = r#"
claim irreversible;
evidence e from "e";
readings forecloses from e limit 2;
decisions irreversible_x limit 1;
event go;
on go sample forecloses = 1 supports irreversible;
"#;
    ReactiveSession::from_source(source).unwrap();
    let named = r#"
claim ok;
evidence e from "e";
decisions irreversible limit 1 irreversible;
event go;
on go commit irreversible because enough;
"#;
    ReactiveSession::from_source(named).unwrap();
}

#[test]
fn load_rules() {
    let base = "claim ok;\nevidence e from \"e\";\nreadings r from e limit 2;\ndecisions other limit 2;\nevent go;\non go commit plain because enough;\n";
    for (line, expected) in [
        (
            "decisions d limit 2 irreversible;",
            "an irreversible series holds one revision: decisions d limit 1 irreversible",
        ),
        (
            "decisions d limit 2 forecloses other;",
            "forecloses follows irreversible: decisions d limit 1 irreversible forecloses ACTION, …",
        ),
        (
            "decisions d limit 1 irreversible forecloses nowhere;",
            "d forecloses nowhere, which is not a decision",
        ),
        (
            "decisions d limit 1 irreversible forecloses d;",
            "d cannot foreclose itself",
        ),
        (
            "decisions d limit 1 irreversible forecloses other, other;",
            "duplicate foreclosed decision other",
        ),
        (
            "decisions d limit 1 irreversible forecloses;",
            "decisions expects NAME limit 1 irreversible",
        ),
        (
            "decisions d limit 1 extra;",
            "decisions expects NAME limit CAPACITY [reopened by STREAM, …]",
        ),
    ] {
        let error = ReactiveSession::from_source(&format!("{base}{line}"))
            .err()
            .unwrap_or_else(|| panic!("{line} should load not"));
        assert!(error.contains(expected), "{line}: {error}");
    }
    for line in [
        "decisions d limit 1 irreversible forecloses other, plain;",
        "decisions d limit 1 irreversible forecloses other reopened by r;",
        "decisions d limit 1 irreversible reopened by r;",
    ] {
        ReactiveSession::from_source(&format!("{base}{line}"))
            .unwrap_or_else(|error| panic!("{line}: {error}"));
    }
}

#[test]
fn a_program_without_the_words_has_no_irreversible_field() {
    let game = session("claim ok;\ndecisions d limit 1;\nevent go;\n");
    assert!(json(&game.snapshot()).get("irreversible").is_none());
}

/// Random events over the migration, with saves and restores in place. After
/// every step: a refusal changed nothing; a restored session is the same
/// session; once the drop is committed no foreclosed action gains a commit;
/// and the drop's grounds never hold an unretained `material` or worse
/// caveat.
#[test]
fn random_events_keep_the_foreclosure_rules() {
    const EVENTS: [(&str, &str); 13] = [
        ("dry_run", r#"{"rows_ok":1,"full":"complete"}"#),
        ("dry_run", r#"{"rows_ok":0,"full":"complete"}"#),
        ("dry_run", r#"{"rows_ok":1,"full":"sampled"}"#),
        ("approve", "{}"),
        ("drop", "{}"),
        ("drop_accepting", "{}"),
        ("roll_back", "{}"),
        ("undo_it", "{}"),
        ("roll_back_then_drop", "{}"),
        ("drop_then_roll_back", "{}"),
        ("misread", "{}"),
        ("approve", "{}"),
        ("roll_back", "{}"),
    ];
    let snapshot = |game: &ReactiveSession| serde_json::to_value(game.snapshot()).unwrap();
    let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut codes = std::collections::BTreeMap::<String, usize>::new();
    for run in 0..200 {
        let mut game = ReactiveSession::from_source(MIGRATION).unwrap();
        let mut trail = Vec::new();
        for step in 0..30 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 1);
            if pick == EVENTS.len() {
                trail.push(("save", ""));
                let saved = game.save_json().unwrap();
                let restored =
                    ReactiveSession::restore_json(MIGRATION, &saved).unwrap_or_else(|error| {
                        panic!("run {run} step {step} restore: {error} {trail:?}")
                    });
                assert_eq!(
                    restored.save_json().unwrap(),
                    saved,
                    "run {run} step {step} {trail:?}"
                );
                assert_eq!(
                    snapshot(&restored),
                    snapshot(&game),
                    "run {run} step {step} {trail:?}"
                );
                game = restored;
                continue;
            }
            let (event, payload) = EVENTS[pick];
            trail.push((event, payload));
            let before = (game.save_json().unwrap(), snapshot(&game));
            let outcome = game
                .dispatch_outcome_json(event, payload)
                .unwrap_or_else(|fatal| {
                    panic!("run {run} step {step} fatal {}: {trail:?}", fatal.message)
                });
            let outcome = serde_json::to_value(outcome).unwrap();
            if outcome["outcome"] != "accepted" {
                *codes
                    .entry(format!(
                        "{}/{}",
                        outcome["origin"].as_str().unwrap(),
                        outcome["code"].as_str().unwrap()
                    ))
                    .or_default() += 1;
                assert_eq!(
                    (game.save_json().unwrap(), snapshot(&game)),
                    before,
                    "run {run} step {step} {trail:?}"
                );
                continue;
            }
            let now = snapshot(&game);
            let drop = &now["decision_series"]["drop_old_table"]["revisions"];
            if let Some(drop) = drop.as_array().unwrap().first() {
                let at = drop["sequence"].as_u64().unwrap();
                let journal = now["decision_journal"].as_array().unwrap();
                let dropped = journal
                    .iter()
                    .position(|entry| {
                        entry["commitment"] == "drop_old_table@1" && entry["change"] == "committed"
                    })
                    .unwrap();
                for (index, entry) in journal.iter().enumerate() {
                    if (entry["decision"] == "rollback" || entry["decision"] == "undo")
                        && entry["change"] == "committed"
                    {
                        assert!(
                            index < dropped && entry["sequence"].as_u64().unwrap() <= at,
                            "run {run} step {step}: {entry} after the drop {trail:?}"
                        );
                    }
                }
                let caveats = now["commitment_grounds"]["drop_old_table@1"]["caveats"].clone();
                for caveat in caveats.as_array().unwrap() {
                    assert!(
                        caveat == "timing_noise" || caveat == "sampled_only",
                        "run {run} step {step}: drop rests on {caveat} {trail:?}"
                    );
                }
            }
        }
    }
    eprintln!("{codes:?}");
    for site in [
        "evaluation/foreclosed",
        "evaluation/caveated_irreversible",
        "limit/history_limit",
    ] {
        assert!(
            codes.contains_key(site),
            "the random check never reached {site}: {codes:?}"
        );
    }
}
