//! `unqualify EVIDENCE with CAVEAT`: taking back a late qualification.
//! See spec/caveat-take-back-0.1.md. Round 8 item 1 (CR18 to CR20): a doubt,
//! a regrowth and the doubt taken back.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

const PROGRAM: &str = r#"
claim edible;
caveat doubted consequence material;
evidence witness_pool from "the slime saw a creature eat it";
evidence taste from "a taste";
renewable witness_pool limit 64;
state memory = 0;
state tasted = 0;
state both = 0;
event see;
event sip;
event doubt;
event regrow;
event undoubt;
event decide;
event recall;
on see reveal witness_pool;
on see set memory = qualified(1, witness_pool);
on sip reveal taste supports edible;
on sip set tasted = qualified(1, taste);
on sip set both = memory + tasted;
on doubt qualify witness_pool with doubted;
on doubt qualify taste with doubted;
on regrow renew witness_pool;
on undoubt unqualify witness_pool with doubted;
on decide commit trust because enough using memory;
state recalled = 0;
on recall when committed(trust) set recalled = 1;
"#;

/// The round 8 program as the spec gives it, with a state that copies the
/// witness.
const ROUND8: &str = r#"
caveat doubted consequence material;
evidence witness_pool from "the slime saw a creature eat it";
renewable witness_pool limit 64;
state memory = 0;
event see;
event doubt;
event regrow;
event undoubt;
event decide;
on see reveal witness_pool;
on see set memory = qualified(1, witness_pool);
on doubt qualify witness_pool with doubted;
on regrow renew witness_pool;
on undoubt unqualify witness_pool with doubted;
on decide commit trust because enough using memory;
state doubts = 0;
event ask;
on ask when carries(witness_pool, doubted) set doubts = 1;
on ask when not carries(witness_pool, doubted) set doubts = 0;
"#;

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn caveats(game: &ReactiveSession, state: &str) -> Value {
    snapshot(game)["qualified_values"][state]["provenance"]["caveats"].clone()
}

#[test]
fn round8_item1_doubt_renew_take_back_reaches_the_earlier_occurrence() {
    let mut game = ReactiveSession::from_source(ROUND8).unwrap();
    game.dispatch_json("see", "{}").unwrap();
    game.dispatch_json("doubt", "{}").unwrap();
    assert_eq!(caveats(&game, "memory"), json!(["doubted"]));
    game.dispatch_json("regrow", "{}").unwrap();
    // The new occurrence does not carry the doubt; memory still does.
    assert_eq!(caveats(&game, "memory"), json!(["doubted"]));
    let shown = game.dispatch_json("undoubt", "{}").unwrap();
    let state = serde_json::to_value(&shown).unwrap();
    assert_eq!(state["qualified_values"]["memory"]["value"], json!(1.0));
    assert_eq!(
        state["qualified_values"]["memory"]["provenance"]["caveats"],
        json!([])
    );
    assert_eq!(state["value_grounds"]["memory"]["caveats"], json!([]));
    assert_eq!(
        state["taken_back"],
        json!([{"evidence": "witness_pool", "caveat": "doubted", "sequence": 4, "event": "undoubt"}])
    );
    assert_eq!(
        state["effects"],
        json!([{"kind": "unqualify", "evidence": "witness_pool", "caveat": "doubted",
                "occurrences": ["witness_pool"]}])
    );
    // The relation stays in the graph, marked by its record.
    assert!(shown
        .relations
        .iter()
        .any(|r| r.from == "doubted" && r.relation == "qualifies" && r.to == "witness_pool"));
}

#[test]
fn round8_item1_a_decision_made_while_doubted_keeps_the_doubt() {
    let mut game = ReactiveSession::from_source(ROUND8).unwrap();
    for event in ["see", "doubt", "decide", "regrow", "undoubt"] {
        game.dispatch_json(event, "{}").unwrap();
    }
    let state = snapshot(&game);
    assert_eq!(
        state["commitment_bases"]["trust"]["provenance"]["caveats"],
        json!(["doubted"])
    );
    assert_eq!(
        state["commitment_bases"]["trust"]["provenance"]["kept"],
        json!(["doubted"])
    );
    assert_eq!(
        state["commitment_grounds"]["trust"]["caveats"],
        json!(["doubted"])
    );
    assert_eq!(state["decision_journal"][0]["caveats"], json!(["doubted"]));
    assert_eq!(caveats(&game, "memory"), json!([]));
}

#[test]
fn round8_item1_carries_answers_the_live_relations() {
    let mut game = ReactiveSession::from_source(ROUND8).unwrap();
    game.dispatch_json("see", "{}").unwrap();
    game.dispatch_json("doubt", "{}").unwrap();
    let doubts = |game: &mut ReactiveSession| {
        game.dispatch_json("ask", "{}").unwrap();
        snapshot(game)["values"]["doubts"].clone()
    };
    assert_eq!(doubts(&mut game), json!(1.0));
    game.dispatch_json("undoubt", "{}").unwrap();
    assert_eq!(doubts(&mut game), json!(0.0));
    // Raised again: a new relation, live until taken back again.
    game.dispatch_json("doubt", "{}").unwrap();
    assert_eq!(doubts(&mut game), json!(1.0));
    assert_eq!(caveats(&game, "memory"), json!(["doubted"]));
    game.dispatch_json("undoubt", "{}").unwrap();
    assert_eq!(doubts(&mut game), json!(0.0));
    assert_eq!(caveats(&game, "memory"), json!([]));
    let state = snapshot(&game);
    assert_eq!(state["taken_back"].as_array().unwrap().len(), 2);
    assert_eq!(
        state["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["from"] == "doubted" && r["to"] == "witness_pool")
            .count(),
        2
    );
}

#[test]
fn round8_item1_nothing_to_take_back_does_nothing() {
    let mut game = ReactiveSession::from_source(ROUND8).unwrap();
    game.dispatch_json("see", "{}").unwrap();
    let shown = game.dispatch_json("undoubt", "{}").unwrap();
    assert!(shown.effects.is_empty());
    assert!(snapshot(&game).get("taken_back").is_none());
    // Unobserved evidence is not a failure either.
    let mut fresh = ReactiveSession::from_source(ROUND8).unwrap();
    assert!(fresh.dispatch_json("undoubt", "{}").is_ok());
    // A second take-back finds nothing live.
    game.dispatch_json("doubt", "{}").unwrap();
    game.dispatch_json("undoubt", "{}").unwrap();
    let again = game.dispatch_json("undoubt", "{}").unwrap();
    assert!(again.effects.is_empty());
    assert_eq!(snapshot(&game)["taken_back"].as_array().unwrap().len(), 1);
}

#[test]
fn round8_item1_a_value_with_other_live_doubted_evidence_keeps_it() {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    for event in ["see", "sip", "doubt", "undoubt"] {
        game.dispatch_json(event, "{}").unwrap();
    }
    // memory rests only on the witness; both also on the doubted taste.
    assert_eq!(caveats(&game, "memory"), json!([]));
    assert_eq!(caveats(&game, "tasted"), json!(["doubted"]));
    assert_eq!(caveats(&game, "both"), json!(["doubted"]));
    let state = snapshot(&game);
    assert_eq!(state["qualified_values"]["both"]["value"], json!(2.0));
}

#[test]
fn round8_item1_a_value_read_from_a_record_keeps_the_caveat() {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    for event in ["see", "sip", "doubt", "decide", "undoubt"] {
        game.dispatch_json(event, "{}").unwrap();
    }
    assert_eq!(caveats(&game, "memory"), json!([]));
    game.dispatch_json("recall", "{}").unwrap();
    let state = snapshot(&game);
    assert_eq!(
        state["qualified_values"]["recalled"]["provenance"]["caveats"],
        json!(["doubted"])
    );
    assert_eq!(
        state["qualified_values"]["recalled"]["provenance"]["kept"],
        json!(["doubted"])
    );
    // A further take-back after doubting again leaves the kept caveat.
    game.dispatch_json("doubt", "{}").unwrap();
    game.dispatch_json("undoubt", "{}").unwrap();
    assert_eq!(caveats(&game, "recalled"), json!(["doubted"]));
}

#[test]
fn round8_item1_a_failed_event_takes_nothing_back() {
    let mut game = ReactiveSession::from_source(&format!(
        "{ROUND8}\nevent broken;\non broken unqualify witness_pool with doubted;\non broken set memory = 1 / 0;"
    ))
    .unwrap();
    game.dispatch_json("see", "{}").unwrap();
    game.dispatch_json("doubt", "{}").unwrap();
    let before = game.snapshot();
    assert!(game.dispatch_json("broken", "{}").is_err());
    assert_eq!(game.snapshot(), before);
}

#[test]
fn round8_item1_a_scheduled_qualification_still_applies() {
    let mut game = ReactiveSession::from_source(&format!(
        "{ROUND8}\nevent tick dt min 0 max 0.1;\nevent later;\non later qualify witness_pool with doubted after 0.2;"
    ))
    .unwrap();
    game.dispatch_json("see", "{}").unwrap();
    game.dispatch_json("later", "{}").unwrap();
    game.dispatch_json("doubt", "{}").unwrap();
    game.dispatch_json("undoubt", "{}").unwrap();
    assert_eq!(caveats(&game, "memory"), json!([]));
    for _ in 0..3 {
        game.dispatch_json("tick", r#"{"dt": 0.1}"#).unwrap();
    }
    assert_eq!(caveats(&game, "memory"), json!(["doubted"]));
}

#[test]
fn round8_item1_the_load_rules_keep_a_take_back_exact() {
    let refused = [
        ("caveat doubted consequence material;\ndoubted qualifies witness_pool;", "declaration"),
        ("caveat doubted consequence material;\ncaveat other consequence low;\nother qualifies doubted;", "declared caveat"),
        ("caveat doubted consequence material;\nstate x = 0;\nevent e;\non e set x = qualified(1, witness_pool, doubted);", "qualified"),
        ("caveat doubted consequence material;\nbudget 3;\nevent e;\non e examine doubted cost 1;", "examine"),
        ("caveat doubted consequence material;\nevent e;\non e commit trust because enough retaining doubted;", "retaining"),
        ("caveat doubted consequence material;\nreadings checks from witness_pool limit 4;", "template"),
    ];
    for (declarations, why) in refused {
        let source = format!(
            "evidence witness_pool from \"a witness\";\n{declarations}\nevent u;\non u unqualify witness_pool with doubted;\n"
        );
        let error = ReactiveSession::from_source(&source)
            .err()
            .unwrap_or_default();
        assert!(error.contains(why), "{why}: {error}");
    }
    let unknown = "evidence witness_pool from \"a witness\";\nevent u;\non u unqualify witness_pool with doubted;\n";
    assert!(ReactiveSession::from_source(unknown).is_err());
    assert!(ReactiveSession::from_source("evidence unqualify from \"x\";").is_err());
}

#[test]
fn round8_item1_save_and_restore_keep_the_records() {
    let mut game = ReactiveSession::from_source(ROUND8).unwrap();
    for event in ["see", "doubt", "decide", "regrow", "undoubt"] {
        game.dispatch_json(event, "{}").unwrap();
    }
    let saved = game.save_json().unwrap();
    let save: Value = serde_json::from_str(&saved).unwrap();
    assert_eq!(save["taken_back"][0]["evidence"], json!("witness_pool"));
    assert_eq!(
        save["commitment_bases"]["trust"]["provenance"]["kept"],
        json!(["doubted"])
    );
    let restored = ReactiveSession::restore_json(ROUND8, &saved).unwrap();
    assert_eq!(restored.snapshot(), game.snapshot());
    // The new occurrence doubted after restoring, then taken back with it.
    let mut restored = restored;
    for event in ["see", "doubt", "undoubt"] {
        restored.dispatch_json(event, "{}").unwrap();
    }
    let effects = serde_json::to_value(&restored.snapshot().effects).unwrap();
    assert_eq!(effects[0]["occurrences"], json!(["witness_pool@2"]));
    assert_eq!(
        snapshot(&restored)["taken_back"].as_array().unwrap().len(),
        2
    );
}

/// Found by the rc.19 stabilization trial: a reopening recorded because of a
/// caveat, then that caveat taken back, left a save that restore refused
/// ("reopening cites caveats unrelated to its witnesses"). The journal entry
/// is left alone (Take-back 0.1), so the save must restore.
const REOPENED: &str = r#"
claim build_green;
evidence ci_run from "a CI run";
caveat flaky_runner consequence material;
renewable ci_run window 4;
state ci_passed = 0;
decisions deploy limit 8;
event ci_reported passed min 0 max 1;
event runner_flaky;
event runner_cleared;
event deploy;
on ci_reported when observed(ci_run) renew ci_run;
on ci_reported reveal ci_run supports build_green;
on ci_reported set ci_passed = qualified(passed, ci_run);
on runner_flaky qualify ci_run with flaky_runner;
on runner_flaky when committed(deploy) and not reopened(deploy)
    reopen deploy because caveated(ci_passed, flaky_runner);
on runner_cleared unqualify ci_run with flaky_runner;
on deploy when has_caveat(ci_passed, flaky_runner) reject "in doubt";
on deploy commit deploy because enough using ci_passed;
"#;

#[test]
fn stabilization_trial_a_take_back_after_a_caveated_reopening_restores() {
    let mut game = ReactiveSession::from_source(REOPENED).unwrap();
    for (event, payload) in [
        ("ci_reported", r#"{"passed":1}"#),
        ("deploy", "{}"),
        ("runner_flaky", "{}"),
        ("runner_cleared", "{}"),
        ("deploy", "{}"),
        ("ci_reported", r#"{"passed":1}"#),
    ] {
        game.dispatch_json(event, payload).unwrap();
    }
    let journal = &snapshot(&game)["decision_journal"];
    let reopening = journal
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["change"] == json!("reopened"))
        .unwrap();
    assert_eq!(reopening["caveats"], json!(["flaky_runner"]));
    let saved = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(REOPENED, &saved).unwrap();
    assert_eq!(restored.snapshot(), game.snapshot());
}

fn refused(save: &Value) -> String {
    ReactiveSession::restore_json(ROUND8, &save.to_string())
        .err()
        .unwrap_or_default()
}

#[test]
fn round8_item1_restore_refuses_records_the_program_cannot_hold() {
    let mut game = ReactiveSession::from_source(ROUND8).unwrap();
    for event in ["see", "doubt", "undoubt"] {
        game.dispatch_json(event, "{}").unwrap();
    }
    let save: Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
    // A record dated after the save.
    let mut late = save.clone();
    late["taken_back"][0]["sequence"] = json!(9);
    assert!(!refused(&late).is_empty());
    // A record no event's unqualify reaches.
    let mut stray = save.clone();
    stray["taken_back"][0]["event"] = json!("doubt");
    assert!(!refused(&stray).is_empty());
    // Two records for one relation.
    let mut twice = save.clone();
    let record = twice["taken_back"][0].clone();
    twice["taken_back"].as_array_mut().unwrap().push(record);
    assert!(!refused(&twice).is_empty());
    // Kept names a caveat the provenance does not carry.
    let mut kept = save.clone();
    kept["states"]["memory"]["lineage"]["kept"] = json!(["doubted"]);
    assert!(!refused(&kept).is_empty());
    // A program that does not take back refuses the field.
    let other = ROUND8.replace("on undoubt unqualify witness_pool with doubted;", "");
    assert!(ReactiveSession::restore_json(&other, &save.to_string()).is_err());
    // Two live relations: the record gone, the relation it marked live.
    let mut live = save.clone();
    live["taken_back"] = json!([]);
    live["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .push(json!(["doubted", "qualifies", "witness_pool"]));
    assert!(!refused(&live).is_empty());
}

#[test]
fn round8_item1_view_v2_shows_the_records_only_where_a_program_takes_back() {
    let mut game = ReactiveSession::from_source(ROUND8).unwrap();
    game.dispatch_json("see", "{}").unwrap();
    game.dispatch_json("doubt", "{}").unwrap();
    let view = serde_json::to_value(game.view_v2()).unwrap();
    assert_eq!(view["taken_back"], json!([]));
    let outcome = game
        .dispatch_view_delta_outcome_json("undoubt", "{}")
        .unwrap();
    let delta = serde_json::to_value(&outcome).unwrap();
    assert_eq!(delta["delta"]["taken_back"]["removed"], json!([]));
    assert_eq!(
        delta["delta"]["taken_back"]["appended"][0]["evidence"],
        json!("witness_pool")
    );
    let plain = ReactiveSession::from_source(
        &ROUND8.replace("on undoubt unqualify witness_pool with doubted;", ""),
    )
    .unwrap();
    assert!(serde_json::to_value(plain.view_v2())
        .unwrap()
        .get("taken_back")
        .is_none());
}

/// D012: a renewal guarded on the doubted evidence. The flagged first
/// occurrence's doubt reaches the second through the renewal's guard, and the
/// second is then doubted in its own right. Taking the doubt back must reach a
/// value read from the second occurrence, as it reaches the occurrences.
const D012: &str = r#"
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
bind view.v_flaky = if(has_caveat(v, flaky), 1, 0);
bind view.e_flaky = if(carries(e, flaky), 1, 0);
"#;

fn bound(game: &ReactiveSession, name: &str) -> f64 {
    snapshot(game)["bindings"]["view"][name].as_f64().unwrap()
}

#[test]
fn d012_unflag_reaches_a_value_read_after_a_guarded_renewal() {
    let mut game = ReactiveSession::from_source(D012).unwrap();
    for event in ["report", "flag", "report", "flag"] {
        game.dispatch_json(event, "{}").unwrap();
    }
    assert_eq!(bound(&game, "v_flaky"), 1.0);
    game.dispatch_json("unflag", "{}").unwrap();
    // Both occurrences' flags are taken back.
    assert_eq!(bound(&game, "e_flaky"), 0.0);
    let taken = snapshot(&game)["taken_back"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["evidence"].clone())
        .collect::<Vec<_>>();
    assert_eq!(taken, [json!("e"), json!("e@2")]);
    assert_eq!(bound(&game, "v_flaky"), 0.0);
    assert_eq!(caveats(&game, "v"), json!([]));
    // A value read again after the take-back agrees with the one taken back.
    game.dispatch_json("report", "{}").unwrap();
    assert_eq!(bound(&game, "v_flaky"), 0.0);
}

#[test]
fn d012_a_decision_made_while_flagged_keeps_the_flag() {
    let source = format!(
        "{D012}event decide;\non decide commit trust because enough using v;\n\
         state held = 0;\nevent recall;\non recall when committed(trust) set held = 1;\n"
    );
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in [
        "report", "flag", "report", "flag", "decide", "unflag", "recall",
    ] {
        game.dispatch_json(event, "{}").unwrap();
    }
    assert_eq!(bound(&game, "v_flaky"), 0.0);
    // The basis is a record: a value read from it keeps the flag.
    assert_eq!(caveats(&game, "held"), json!(["flaky"]));
}

/// D019: `examine` stores its guard as the attention decision's record. A
/// guard that read flagged evidence stores the flag through the live
/// relation, so a take-back reaches it there as it does in any other guard
/// record, and a value read through `examined(...)` after the unflag carries
/// no flag. A late qualify reaches the record the same way (D014).
#[test]
fn d019_unflag_reaches_a_flag_an_examine_guard_read() {
    let source = format!(
        "{D012}budget 10;\ncaveat stale consequence low;\nevent scan;\nevent look;\n\
         state seen = 0;\non scan when observed(e) examine stale cost 1;\n\
         on look when examined(stale) set seen = 1;\n"
    );
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["report", "flag", "scan", "unflag", "look"] {
        game.dispatch_json(event, "{}").unwrap();
    }
    assert_eq!(bound(&game, "e_flaky"), 0.0);
    assert_eq!(caveats(&game, "seen"), json!(["stale"]));
    game.dispatch_json("flag", "{}").unwrap();
    game.dispatch_json("look", "{}").unwrap();
    assert_eq!(caveats(&game, "seen"), json!(["flaky", "stale"]));
}

/// `e` is observed under a guard that reads `f`, so its observation's record
/// lists `f`. Shared by D013 and D014.
const D013_THROUGH_F: &str = r#"
caveat flaky consequence material;
evidence e from "tool";
evidence f from "operator";
event see_f;
event see_e;
event flag_e;
event unflag_e;
event flag_f;
event early;
event late;
state before = 0;
state after = 0;
on see_f reveal f;
on see_e when observed(f) reveal e;
on flag_e qualify e with flaky;
on unflag_e unqualify e with flaky;
on flag_f qualify f with flaky;
on early set before = qualified(1, e);
on late set after = qualified(1, e);
"#;

#[test]
fn d014_a_later_qualify_reaches_a_read_through_a_guard() {
    let mut game = ReactiveSession::from_source(D013_THROUGH_F).unwrap();
    for event in ["see_f", "see_e", "early", "flag_f", "late"] {
        game.dispatch_json(event, "{}").unwrap();
    }
    // Computed before or after the qualify, from the same records.
    assert_eq!(caveats(&game, "before"), json!(["flaky"]));
    assert_eq!(caveats(&game, "after"), json!(["flaky"]));
}

#[test]
fn d013_a_take_back_tests_grounds_by_their_own_evidence() {
    let mut game = ReactiveSession::from_source(D013_THROUGH_F).unwrap();
    for event in ["see_f", "flag_f", "see_e", "flag_e", "late", "unflag_e"] {
        game.dispatch_json(event, "{}").unwrap();
    }
    let shown = snapshot(&game);
    // The lineage still holds `f`, which carries the flag; the grounds hold
    // only `e`, which no longer does.
    assert_eq!(
        shown["qualified_values"]["after"]["provenance"]["caveats"],
        json!(["flaky"])
    );
    assert_eq!(shown["value_grounds"]["after"]["evidence"], json!(["e"]));
    assert_eq!(shown["value_grounds"]["after"]["caveats"], json!([]));
}

/// A caveat reaches `t` through a guard read of `r`, and `t` is flagged with
/// a caveat of the same name.
const D013: &str = r#"
caveat flaky consequence material;
evidence t from "tool";
evidence r from "tool";
event setup; event report; event flag; event unflag;
state fz = 0;
state p = 0;
on setup reveal r;
on setup qualify r with flaky;
on setup when observed(r) set fz = 0;
on report when fz == 0 reveal t;
on report set p = qualified(1, t);
on flag qualify t with flaky;
on unflag unqualify t with flaky;
bind x.t_flaky = if(carries(t, flaky), 1, 0);
bind x.p_flaky = if(has_caveat(p, flaky), 1, 0);
"#;

#[test]
fn d013_unflag_reaches_a_flag_named_like_one_a_guard_carried() {
    let mut game = ReactiveSession::from_source(D013).unwrap();
    let mut shown = Vec::new();
    for event in ["setup", "report", "flag", "unflag"] {
        game.dispatch_json(event, "{}").unwrap();
        shown.push((
            snapshot(&game)["bindings"]["x"]["t_flaky"]
                .as_f64()
                .unwrap(),
            snapshot(&game)["bindings"]["x"]["p_flaky"]
                .as_f64()
                .unwrap(),
        ));
    }
    assert_eq!(
        shown,
        [(0.0, 0.0), (0.0, 0.0), (1.0, 1.0), (0.0, 0.0)],
        "{}",
        snapshot(&game)
    );
}

/// Invariant: a value carries a caveat exactly when evidence it rests on still
/// carries it, with no caveat stale and none missing. For a state that reads
/// no record (a decision's basis, or a reading archived with a caveat it
/// keeps) and has no caveat placed on it directly, its lineage carries
/// `flaky` exactly when some occurrence in its lineage carries it now, and
/// `has_caveat(state, flaky)` holds exactly when some occurrence in its
/// grounds does. Random event sequences, checked after every event and after
/// saving and restoring at random points, so a take-back or qualify that
/// misses a record cannot hide.
#[test]
fn d012_random_sequences_keep_caveats_matched_to_their_evidence() {
    // Every guarded effect a source rule can write that stores an observation
    // qualification (renew, reveal, sample), and a second evidence `f` whose
    // `flaky` reaches `e` only through a guard, directly or through a state,
    // while `e` has its own `flaky` (D013).
    // `save` is no event: it saves and restores the session in its place.
    const EVENTS: [&str; 18] = [
        "report", "plain", "via_f", "via_fz", "see_f", "read", "flag", "unflag", "flag_f",
        "unflag_f", "check", "show", "decide", "recall", "doubt", "withdraw", "see_op", "save",
    ];
    let source = r#"
caveat flaky consequence material;
claim c;
evidence e from "tool";
renewable e limit 64;
evidence f from "operator";
renewable f limit 64;
evidence probe from "probe";
readings s from probe limit 64;
event report;
event plain;
event via_f;
event via_fz;
event see_f;
event read;
event flag;
event unflag;
event flag_f;
event unflag_f;
event check;
event show;
event decide;
event recall;
event doubt;
event withdraw;
event see_op;
evidence op from "operator check";
decisions trust limit 64;
state recalled = 0;
on decide when observed(e) commit trust because enough using v;
on recall when committed(trust) set recalled = qualified(1, e);
on doubt when committed(trust) and observed(e) reopen trust because e;
on see_op reveal op;
on withdraw when observed(op) and observed(e) withdraw e because op;
state held = 0;
state v = 0;
state w = 0;
state fz = 0;
on report when observed(e) renew e;
on report reveal e;
on plain when not observed(e) renew e;
on plain reveal e;
on via_f when observed(f) renew e;
on via_f when observed(f) reveal e;
on see_f when observed(f) renew f;
on see_f reveal f;
on see_f when observed(f) set fz = 1;
on via_fz when fz == 1 renew e;
on via_fz when fz == 1 reveal e;
on via_fz when observed(e) set v = qualified(1, e);
on read when observed(e) sample s = 1 supports c;
on read when has_sample(s) set w = latest(s);
on flag when observed(e) qualify e with flaky;
on unflag unqualify e with flaky;
on flag_f when observed(f) qualify f with flaky;
on unflag_f when observed(f) unqualify f with flaky;
on show when observed(f) and observed(e) reveal e supports c;
on report set v = qualified(1, e);
on plain set v = qualified(1, e);
on via_f when observed(e) set v = qualified(1, e);
on show when observed(e) set v = qualified(1, e);
on check when observed(e) set held = v + qualified(0, e);
"#;
    let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut d015 = 0;
    for run in 0..400 {
        let mut game = ReactiveSession::from_source(source).unwrap();
        let mut trail = Vec::new();
        for _ in 0..24 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let event = EVENTS[(seed >> 33) as usize % EVENTS.len()];
            trail.push(event);
            if event == "save" {
                let before = snapshot(&game);
                let saved = game.save_json().unwrap();
                game = ReactiveSession::restore_json(source, &saved)
                    .unwrap_or_else(|error| panic!("run {run} after {trail:?}: {error}"));
                for part in [
                    "qualified_values",
                    "value_grounds",
                    "observation_qualifications",
                    "predicate_qualifications",
                ] {
                    assert_eq!(
                        snapshot(&game)[part],
                        before[part],
                        "run {run}, {part} after restoring {trail:?}"
                    );
                }
            } else if game.dispatch_json(event, "{}").is_err() {
                continue;
            }
            let shown = snapshot(&game);
            let basis = shown["commitment_bases"]
                .as_object()
                .into_iter()
                .flat_map(|bases| bases.values())
                .flat_map(|basis| basis["provenance"]["evidence"].as_array().unwrap().clone())
                .collect::<Vec<_>>();
            for state in ["v", "held", "w", "fz", "recalled"] {
                let provenance = &shown["qualified_values"][state]["provenance"];
                let lineage = provenance["evidence"].as_array().unwrap();
                // Live: more `flaky qualifies` relations than take-backs.
                let count = |list: &str, key: &str, occurrence: &Value| {
                    shown[list].as_array().map_or(0, |entries| {
                        entries
                            .iter()
                            .filter(|entry| {
                                entry[key] == *occurrence
                                    && (entry["caveat"] == "flaky" || entry["from"] == "flaky")
                            })
                            .count()
                    })
                };
                let live = lineage.iter().any(|occurrence| {
                    count("relations", "to", occurrence)
                        > count("taken_back", "evidence", occurrence)
                });
                let holds = |list: &Value| {
                    list.as_array()
                        .is_some_and(|caveats| caveats.iter().any(|caveat| caveat == "flaky"))
                };
                // Only a record keeps the flag: a reading archived while `e`
                // was flagged, or a decision's basis.
                let kept = holds(&provenance["kept"]);
                assert!(
                    !kept || state == "w" || state == "recalled",
                    "run {run}, {state} kept: {provenance}"
                );
                let carried = holds(&provenance["caveats"]);
                // D015 (open): `recalled` reaches evidence through the basis
                // `committed(trust)` reads. Whether a later `qualify` of that
                // evidence reaches it depends today on when it was set, so it
                // may match either answer; each such reading is counted.
                if state == "recalled" && carried != (live || kept) {
                    let live_outside_basis = lineage.iter().any(|occurrence| {
                        !basis.contains(occurrence)
                            && count("relations", "to", occurrence)
                                > count("taken_back", "evidence", occurrence)
                    });
                    if carried == (live_outside_basis || kept) {
                        d015 += 1;
                        continue;
                    }
                }
                assert_eq!(
                    carried,
                    live || kept,
                    "run {run}, {state} after {trail:?}: {provenance}\n{}\n{}",
                    shown["observation_qualifications"],
                    shown["predicate_qualifications"]
                );
                // `has_caveat` reads the grounds.
                let grounds = &shown["value_grounds"][state];
                let rests_on_flagged =
                    grounds["evidence"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|occurrence| {
                            count("relations", "to", occurrence)
                                > count("taken_back", "evidence", occurrence)
                        });
                let has_caveat = holds(&grounds["caveats"]);
                assert_eq!(
                    has_caveat,
                    rests_on_flagged || holds(&grounds["kept"]),
                    "run {run}, {state} grounds after {trail:?}: {grounds}"
                );
            }
        }
    }
    // Every reading the open D015 question leaves either way, so a run shows
    // how often the shape occurs; this count goes when the question is settled.
    eprintln!("D015 (open): {d015} readings of `recalled` matched the other answer");
}
