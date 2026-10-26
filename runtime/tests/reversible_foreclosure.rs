//! `decisions NAME limit N forecloses ACTION, …` without `irreversible`: a
//! decision that rules options out while it is in force, and lets them back
//! in when it is reopened. See spec/caveat-foreclosure-0.2.md.
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::web::WebReactiveSession;
use serde_json::Value;

/// A main agent rules out a rewrite and a plain `patch`, for a reason that
/// may turn out wrong; the plan's reviews arrive as readings.
const HANDOFF: &str = r#"
claim rewrite_out_of_scope;
evidence review from "the finding that a rewrite is out of scope";
evidence recheck from "a finding that the review was wrong";
evidence brief from "the brief to the sub-agent";
evidence scope_feed from "later scope reviews";
readings scope from scope_feed limit 8;
decisions no_rewrite limit 4 forecloses rewrite, patch reopened by scope opposing rewrite_out_of_scope;
decisions rewrite limit 4;
event rule_out;
event reconsider;
event rewrite_it;
event patch_it;
event hold_it;
event release_it;
event retract;
event rescope;
event rewrite_then_rule_out;
event rule_out_then_rewrite;
on rule_out when not observed(review) reveal review supports rewrite_out_of_scope;
on rule_out commit no_rewrite because enough citing qualified(1, review);
on reconsider when not observed(recheck) reveal recheck opposes rewrite_out_of_scope;
on reconsider reopen no_rewrite because recheck;
on rewrite_it commit rewrite because enough using 1;
on patch_it commit patch because enough;
on hold_it when not observed(brief) reveal brief;
on hold_it hold no_rewrite because brief;
on release_it release no_rewrite because brief;
on retract when not observed(recheck) reveal recheck opposes rewrite_out_of_scope;
on retract withdraw review because recheck;
on retract when committed(no_rewrite) and not reopened(no_rewrite) and rests_on_withdrawn(no_rewrite)
    reopen no_rewrite because recheck;
on rescope sample scope = 0 opposes rewrite_out_of_scope;
on rewrite_then_rule_out commit rewrite because enough using 2;
on rewrite_then_rule_out when not observed(review) reveal review supports rewrite_out_of_scope;
on rewrite_then_rule_out commit no_rewrite because enough citing qualified(1, review);
on rule_out_then_rewrite when not observed(review) reveal review supports rewrite_out_of_scope;
on rule_out_then_rewrite commit no_rewrite because enough citing qualified(1, review);
on rule_out_then_rewrite commit rewrite because enough using 3;
"#;

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn session(source: &str) -> WebReactiveSession {
    WebReactiveSession::new(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn accept(game: &mut WebReactiveSession, event: &str) -> Value {
    let result = json(&game.dispatch_outcome(event, "{}").unwrap());
    assert_eq!(result["outcome"], "accepted", "{event}: {result}");
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
    // The message after the rule it names.
    let message = result["message"].as_str().unwrap();
    message
        .split_once(": ")
        .map_or(message, |(_, rest)| rest)
        .to_string()
}

fn current(game: &WebReactiveSession, series: &str) -> Value {
    json(&game.snapshot())["decision_series"][series]["current"].clone()
}

#[test]
fn a_decision_in_force_forecloses_and_its_reopening_lifts_it() {
    let mut game = session(HANDOFF);
    // Open until the decision is made.
    accept(&mut game, "patch_it");
    let mut game = session(HANDOFF);
    accept(&mut game, "rule_out");
    let message = refuse(&mut game, "rewrite_it", "foreclosed");
    assert_eq!(message, "rewrite is foreclosed by no_rewrite@1");
    let message = refuse(&mut game, "patch_it", "foreclosed");
    assert_eq!(message, "patch is foreclosed by no_rewrite@1");
    // Reopening lifts it: the decision is in question, so the options it
    // ruled out are open again.
    accept(&mut game, "reconsider");
    accept(&mut game, "rewrite_it");
    accept(&mut game, "patch_it");
    assert_eq!(current(&game, "rewrite"), "rewrite@1");
}

#[test]
fn a_new_revision_forecloses_again_and_a_commit_already_made_stands() {
    let mut game = session(HANDOFF);
    for event in ["rule_out", "reconsider", "rewrite_it", "rule_out"] {
        accept(&mut game, event);
    }
    assert_eq!(current(&game, "no_rewrite"), "no_rewrite@2");
    // rewrite@1 keeps its status: a foreclosure closes only later commits.
    let snapshot = json(&game.snapshot());
    let open = snapshot["commitments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|commitment| commitment["action"] == "rewrite@1")
        .unwrap()["open"]
        .clone();
    assert_eq!(open, false);
    let message = refuse(&mut game, "rewrite_it", "foreclosed");
    assert_eq!(message, "rewrite is foreclosed by no_rewrite@2");
}

#[test]
fn a_trigger_and_a_withdrawal_lift_it() {
    let mut game = session(HANDOFF);
    accept(&mut game, "rule_out");
    refuse(&mut game, "rewrite_it", "foreclosed");
    accept(&mut game, "rescope");
    accept(&mut game, "rewrite_it");

    let mut game = session(HANDOFF);
    accept(&mut game, "rule_out");
    accept(&mut game, "retract");
    accept(&mut game, "rewrite_it");
}

#[test]
fn a_hold_keeps_it_until_the_release_but_never_on_withdrawn_evidence() {
    let mut game = session(HANDOFF);
    for event in ["rule_out", "hold_it", "reconsider"] {
        accept(&mut game, event);
    }
    // The reopening waits for the release, and so does the option.
    refuse(&mut game, "rewrite_it", "foreclosed");
    accept(&mut game, "release_it");
    accept(&mut game, "rewrite_it");

    let mut game = session(HANDOFF);
    for event in ["rule_out", "hold_it", "retract"] {
        accept(&mut game, event);
    }
    accept(&mut game, "rewrite_it");
}

#[test]
fn commits_run_in_rule_order_within_an_event() {
    let mut game = session(HANDOFF);
    accept(&mut game, "rewrite_then_rule_out");
    assert_eq!(current(&game, "rewrite"), "rewrite@1");
    assert_eq!(current(&game, "no_rewrite"), "no_rewrite@1");
    let mut game = session(HANDOFF);
    let message = refuse(&mut game, "rule_out_then_rewrite", "foreclosed");
    assert_eq!(message, "rewrite is foreclosed by no_rewrite@1");
    assert_eq!(current(&game, "no_rewrite"), Value::Null);
}

#[test]
fn two_decisions_may_exclude_each_other() {
    let source = r#"
claim ok;
decisions keep limit 4 forecloses replace;
decisions replace limit 4 forecloses keep;
event keep_it;
event replace_it;
event rethink_keep;
evidence rethought from "a second look";
on keep_it commit keep because enough;
on replace_it commit replace because enough;
on rethink_keep when not observed(rethought) reveal rethought;
on rethink_keep reopen keep because rethought;
"#;
    let mut game = session(source);
    accept(&mut game, "keep_it");
    let message = refuse(&mut game, "replace_it", "foreclosed");
    assert_eq!(message, "replace is foreclosed by keep@1");
    accept(&mut game, "rethink_keep");
    accept(&mut game, "replace_it");
    let message = refuse(&mut game, "keep_it", "foreclosed");
    assert_eq!(message, "keep is foreclosed by replace@1");
}

#[test]
fn the_earliest_foreclosure_is_named() {
    let source = r#"
claim ok;
evidence again from "a second look";
decisions publish limit 1 irreversible forecloses retract;
decisions freeze limit 4 forecloses retract;
decisions retract limit 4;
event freeze_it;
event publish_it;
event unfreeze;
event retract_it;
on freeze_it commit freeze because enough;
on publish_it commit publish because enough;
on unfreeze when not observed(again) reveal again;
on unfreeze reopen freeze because again;
on retract_it commit retract because enough;
"#;
    let mut game = session(source);
    accept(&mut game, "freeze_it");
    accept(&mut game, "publish_it");
    assert_eq!(
        refuse(&mut game, "retract_it", "foreclosed"),
        "retract is foreclosed by freeze@1"
    );
    // The irreversible foreclosure stands when the reversible one lifts.
    accept(&mut game, "unfreeze");
    assert_eq!(
        refuse(&mut game, "retract_it", "foreclosed"),
        "retract is foreclosed by publish@1"
    );
}

#[test]
fn a_procedure_commit_is_foreclosed() {
    let source = r#"
claim ok;
decisions no_rewrite limit 4 forecloses rewrite;
decisions rewrite limit 4;
event rule_out;
event go;
proc decide(d decisions) { commit d because enough using 1; };
on rule_out commit no_rewrite because enough;
on go call decide(rewrite);
"#;
    let mut game = session(source);
    accept(&mut game, "rule_out");
    refuse(&mut game, "go", "foreclosed");
}

#[test]
fn the_snapshot_and_explain_name_the_foreclosure() {
    let mut game = session(HANDOFF);
    let snapshot = json(&game.snapshot());
    assert_eq!(
        snapshot["forecloses"],
        serde_json::json!({"no_rewrite": ["rewrite", "patch"]})
    );
    assert!(snapshot.get("irreversible").is_none());
    accept(&mut game, "rule_out");
    // A program with no reversible foreclosure has no such field.
    let plain = session("claim ok;\ndecisions d limit 1 irreversible forecloses e;\ndecisions e limit 1;\nevent go;\n");
    let snapshot = json(&plain.snapshot());
    assert!(snapshot.get("forecloses").is_none());
    assert!(snapshot.get("irreversible").is_some());
    let none = session("claim ok;\ndecisions d limit 2;\nevent go;\n");
    assert!(json(&none.snapshot()).get("forecloses").is_none());
}

#[test]
fn saves_restore_and_restore_checks_the_order() {
    let mut game = ReactiveSession::from_source(HANDOFF).unwrap();
    for event in ["rule_out", "reconsider", "rewrite_it", "rule_out"] {
        game.dispatch_json(event, "{}")
            .unwrap_or_else(|error| panic!("{event}: {error}"));
    }
    let saved = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(HANDOFF, &saved).unwrap();
    assert_eq!(restored.save_json().unwrap(), saved);
    assert!(!saved.contains("forecloses"), "the save holds nothing new");
    // Move the reopening of no_rewrite@1 after the rewrite: a record this
    // program can't hold.
    let mut save = json(&saved);
    let rewrite = save["decision_series"]["rewrite"]["revisions"][0]["sequence"]
        .as_u64()
        .unwrap();
    for entry in save["decision_journal"].as_array_mut().unwrap() {
        if entry["commitment"] == "no_rewrite@1" && entry["change"] == "reopened" {
            entry["sequence"] = (rewrite + 1).into();
        }
    }
    let error = ReactiveSession::restore_json(HANDOFF, &save.to_string())
        .map(|_| ())
        .expect_err("a rewrite while the exclusion was in force restored");
    assert!(
        error.contains(
            "rewrite@1 is committed while no_rewrite@1, which forecloses it, is in force"
        ),
        "{error}"
    );

    // A current revision in force with a later commit of what it forecloses.
    let mut game = ReactiveSession::from_source(HANDOFF).unwrap();
    for event in ["rewrite_it", "rule_out"] {
        game.dispatch_json(event, "{}").unwrap();
    }
    let mut save = json(&game.save_json().unwrap());
    let first = save["decision_series"]["rewrite"]["revisions"][0]["sequence"].clone();
    let second = save["decision_series"]["no_rewrite"]["revisions"][0]["sequence"].clone();
    save["decision_series"]["rewrite"]["revisions"][0]["sequence"] = second.clone();
    save["decision_series"]["no_rewrite"]["revisions"][0]["sequence"] = first.clone();
    for entry in save["decision_journal"].as_array_mut().unwrap() {
        if entry["commitment"] == "rewrite@1" {
            entry["sequence"] = second.clone();
        } else if entry["commitment"] == "no_rewrite@1" {
            entry["sequence"] = first.clone();
        }
    }
    let error = ReactiveSession::restore_json(HANDOFF, &save.to_string())
        .map(|_| ())
        .expect_err("a rewrite after the exclusion restored");
    assert!(
        error.contains(
            "rewrite@1 is committed while no_rewrite@1, which forecloses it, is in force"
        ),
        "{error}"
    );
}

#[test]
fn load_rules() {
    let base = "claim ok;\nevidence e from \"e\";\nreadings r from e limit 2;\ndecisions other limit 2;\nevent go;\non go commit plain because enough;\n";
    for (line, expected) in [
        (
            "decisions d limit 2 forecloses nowhere;",
            "d forecloses nowhere, which is not a decision",
        ),
        ("decisions d limit 2 forecloses d;", "d cannot foreclose itself"),
        (
            "decisions d limit 2 forecloses other, other;",
            "duplicate foreclosed decision other",
        ),
        (
            "decisions d limit 2 forecloses;",
            "decisions expects NAME limit CAPACITY [irreversible] [forecloses ACTION, …] [reopened by STREAM, …]",
        ),
        (
            "decisions d limit 2 forecloses other extra;",
            "forecloses takes decision names separated by commas, not other extra",
        ),
    ] {
        let error = ReactiveSession::from_source(&format!("{base}{line}"))
            .err()
            .unwrap_or_else(|| panic!("{line} should not load"));
        assert!(error.contains(expected), "{line}: {error}");
    }
    for line in [
        "decisions d limit 2 forecloses other, plain;",
        "decisions d limit 1 forecloses other;",
        "decisions d limit 4 forecloses other reopened by r;",
        "decisions forecloses limit 2 forecloses other;",
    ] {
        ReactiveSession::from_source(&format!("{base}{line}"))
            .unwrap_or_else(|error| panic!("{line}: {error}"));
    }
}

#[test]
fn check_still_asks_a_reversible_series_for_a_reopening_path() {
    let source = r#"
claim ok;
evidence e from "e";
readings r from e limit 2;
decisions fixed limit 2 forecloses other;
decisions other limit 2;
event go;
on go sample r = 1 supports ok;
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
    assert!(
        flagged.iter().any(|message| message.starts_with("`fixed`")),
        "{flagged:?}"
    );
}

/// HANDOFF with the foreclosure written as guards, the way a program had to
/// say it before this profile.
fn guarded() -> String {
    let guard = |event: &str| {
        format!(
            "on {event} when committed(no_rewrite) and not reopened(no_rewrite) reject \"ruled out\";\n"
        )
    };
    let mut source = String::new();
    for line in HANDOFF.lines() {
        let line = line.replace(" forecloses rewrite, patch", "");
        if line.starts_with("on rewrite_it commit") {
            source.push_str(&guard("rewrite_it"));
        } else if line.starts_with("on patch_it commit") {
            source.push_str(&guard("patch_it"));
        } else if line.starts_with("on rule_out_then_rewrite commit rewrite") {
            source.push_str(&guard("rule_out_then_rewrite"));
        } else if line.starts_with("on rewrite_then_rule_out commit rewrite") {
            source.push_str(&guard("rewrite_then_rule_out"));
        }
        source.push_str(&line);
        source.push('\n');
    }
    source
}

/// Random events over HANDOFF and its guarded twin, with saves and restores
/// in place. After every step: both accept or both refuse, a foreclosed
/// refusal where the twin's guard refused; a refusal changed nothing; a
/// restored session is the same session; and no foreclosed action was
/// committed while `no_rewrite` was in force, by the journal.
#[test]
fn random_events_keep_the_exclusion_and_match_the_guards() {
    const EVENTS: &[&str] = &[
        "rule_out",
        "reconsider",
        "rewrite_it",
        "patch_it",
        "hold_it",
        "release_it",
        "retract",
        "rescope",
        "rewrite_then_rule_out",
        "rule_out_then_rewrite",
        "rule_out",
        "rewrite_it",
    ];
    let twin_source = guarded();
    let snapshot = |game: &ReactiveSession| serde_json::to_value(game.snapshot()).unwrap();
    let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
    let mut codes = std::collections::BTreeMap::<String, usize>::new();
    for run in 0..300 {
        let mut game = ReactiveSession::from_source(HANDOFF).unwrap();
        let mut twin = ReactiveSession::from_source(&twin_source).unwrap();
        let mut trail = Vec::new();
        for step in 0..40 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 1);
            if pick == EVENTS.len() {
                trail.push("save");
                let saved = game.save_json().unwrap();
                let restored =
                    ReactiveSession::restore_json(HANDOFF, &saved).unwrap_or_else(|error| {
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
            let event = EVENTS[pick];
            trail.push(event);
            let before = (game.save_json().unwrap(), snapshot(&game));
            let outcome =
                serde_json::to_value(game.dispatch_outcome_json(event, "{}").unwrap()).unwrap();
            let theirs =
                serde_json::to_value(twin.dispatch_outcome_json(event, "{}").unwrap()).unwrap();
            assert_eq!(
                outcome["outcome"], theirs["outcome"],
                "run {run} step {step} {trail:?}"
            );
            if outcome["outcome"] != "accepted" {
                let code = format!(
                    "{}/{}",
                    outcome["origin"].as_str().unwrap(),
                    outcome["code"].as_str().unwrap()
                );
                if code == "evaluation/foreclosed" {
                    assert!(
                        theirs["message"].as_str().unwrap().ends_with("ruled out"),
                        "run {run} step {step} {theirs} {trail:?}"
                    );
                } else {
                    assert_eq!(
                        outcome["code"], theirs["code"],
                        "run {run} step {step} {trail:?}"
                    );
                }
                *codes.entry(code).or_default() += 1;
                assert_eq!(
                    (game.save_json().unwrap(), snapshot(&game)),
                    before,
                    "run {run} step {step} {trail:?}"
                );
                continue;
            }
            let mut in_force = false;
            for entry in snapshot(&game)["decision_journal"].as_array().unwrap() {
                match (
                    entry["decision"].as_str().unwrap(),
                    entry["change"].as_str().unwrap(),
                ) {
                    ("no_rewrite", "committed") => in_force = true,
                    ("no_rewrite", "reopened") => in_force = false,
                    ("rewrite" | "patch", "committed") => assert!(
                        !in_force,
                        "run {run} step {step}: {entry} while ruled out {trail:?}"
                    ),
                    _ => {}
                }
            }
        }
    }
    eprintln!("{codes:?}");
    assert!(
        codes.contains_key("evaluation/foreclosed"),
        "the random check never reached a foreclosure: {codes:?}"
    );
}
