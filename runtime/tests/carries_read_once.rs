//! `carries(EVIDENCE, CAVEAT)` reads its evidence once (compiler step 3.2):
//! one check that the evidence is observed and one search for what qualifies
//! it give the value, the lineage and the grounds. Each case here reaches a
//! part of that read no other test did, where debug builds compare it with
//! the general path, which read `observed(EVIDENCE)` for what the answer
//! rests on. Each also checks, in any build, that `carries(e, C)` rests on
//! exactly what `observed(e)` rests on, for its lineage and its grounds.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("load: {error}"))
}

fn go(game: &mut ReactiveSession, event: &str) {
    game.dispatch_json(event, "{}")
        .unwrap_or_else(|error| panic!("{event}: {error}"));
}

fn send(game: &mut ReactiveSession, event: &str, v: f64) {
    game.apply(event, &BTreeMap::from([("v".to_string(), v)]))
        .unwrap_or_else(|error| panic!("{event} {v}: {error}"));
}

fn shot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn lineage(shot: &Value, state: &str) -> Value {
    shot["qualified_values"][state]["provenance"].clone()
}

fn grounds(shot: &Value, state: &str) -> Value {
    shot["value_grounds"][state].clone()
}

/// `e` is never observed, but a reveal of it was skipped under a
/// guard that read `g`, so `observed(e)` rests on `g`, and so must
/// `carries(e, flaky)`: the unconditional `predicate_dependency("observed")`
/// merge with `observed` false.
const SKIPPED_REVEAL: &str = r#"
claim c;
caveat flaky consequence material;
evidence g from "g";
evidence e from "e";
state t = 0;
state o = 0;
event see_g;
event try_e;
event ask;
on see_g reveal g supports c;
on try_e when not observed(g) reveal e;
on ask set t = if(carries(e, flaky), 1, 0);
on ask set o = if(observed(e), 1, 0);
"#;

#[test]
fn carries_of_unobserved_evidence_rests_on_a_skipped_reveals_guard() {
    let mut game = session(SKIPPED_REVEAL);
    for event in ["see_g", "try_e", "ask"] {
        go(&mut game, event);
    }
    let shot = shot(&game);
    assert_eq!(shot["qualified_values"]["t"]["value"], json!(0.0));
    assert_eq!(lineage(&shot, "t")["evidence"], json!(["g"]));
    assert_eq!(lineage(&shot, "t"), lineage(&shot, "o"));
    assert_eq!(grounds(&shot, "t"), grounds(&shot, "o"));
}

/// `e` was revealed under a guard that read `g`. Its lineage holds
/// `g` through the observation's record; its grounds do not.
const GUARDED_REVEAL: &str = r#"
claim c;
caveat flaky consequence material;
evidence g from "g";
evidence e from "e";
state t = 0;
state o = 0;
event see_g;
event see_e;
event doubt;
event ask;
on see_g reveal g supports c;
on see_e when observed(g) reveal e;
on doubt qualify e with flaky;
on ask set t = if(carries(e, flaky), 1, 0);
on ask set o = if(observed(e), 1, 0);
"#;

#[test]
fn carries_of_evidence_revealed_under_a_guard_rests_on_the_guard_but_is_not_grounded_on_it() {
    let mut game = session(GUARDED_REVEAL);
    for event in ["see_g", "see_e", "doubt", "ask"] {
        go(&mut game, event);
    }
    let shot = shot(&game);
    assert_eq!(shot["qualified_values"]["t"]["value"], json!(1.0));
    assert_eq!(lineage(&shot, "t")["evidence"], json!(["e", "g"]));
    assert_eq!(lineage(&shot, "t")["caveats"], json!(["flaky"]));
    assert_eq!(grounds(&shot, "t")["evidence"], json!(["e"]));
    assert_eq!(grounds(&shot, "t")["caveats"], json!(["flaky"]));
    assert_eq!(lineage(&shot, "t"), lineage(&shot, "o"));
    assert_eq!(grounds(&shot, "t"), grounds(&shot, "o"));
}

/// `e`'s observation record and its
/// `observed` predicate record each hold a different departed reading of
/// `r`, so each holds its own marker. The general path joins the
/// predicate's marker twice (through `qualify`, then again in
/// `predicate_tracked`), and the second join makes a new archive node, so
/// dropping either merge changes the marker's `archive_ref`.
const TWO_MARKERS: &str = r#"
claim c;
caveat flaky consequence material;
evidence src from "a source";
evidence e from "e";
readings r from src window 1;
state t = 0;
state o = 0;
event s v min 0 max 10;
event see;
event see_again;
event ask;
on s sample r = v supports c;
on see when latest(r) > 0 reveal e;
on see_again when latest(r) > 100 reveal e;
on ask set t = if(carries(e, flaky), 1, 0);
on ask set o = if(observed(e), 1, 0);
"#;

#[test]
fn carries_joins_departed_markers_as_observed_does() {
    let mut game = session(TWO_MARKERS);
    send(&mut game, "s", 5.0);
    go(&mut game, "see");
    send(&mut game, "s", 6.0);
    go(&mut game, "see_again");
    send(&mut game, "s", 7.0);
    go(&mut game, "ask");
    let shot = shot(&game);
    assert_eq!(shot["qualified_values"]["t"]["value"], json!(0.0));
    let departed = &lineage(&shot, "t")["departed"];
    assert_eq!(departed.as_array().map(Vec::len), Some(1), "{shot:#}");
    assert_eq!(departed[0]["history"], json!("r"));
    assert_eq!(lineage(&shot, "t"), lineage(&shot, "o"));
    assert_eq!(grounds(&shot, "t"), grounds(&shot, "o"));
}

/// No predicate dependency can be held under a `carries` kind, so the read
/// never finds one of its own: only what `observed` rests on reaches it. A
/// rule records dependencies only for observed, withdrawn, examined,
/// committed and reopened (`retain_skipped_effect`), and restore refuses any
/// other kind.
#[test]
fn a_save_cannot_hold_a_dependency_of_a_carries_predicate() {
    let mut game = session(SKIPPED_REVEAL);
    for event in ["see_g", "try_e"] {
        go(&mut game, event);
    }
    let mut save: Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
    let dependencies = save["predicate_qualifications"].as_object_mut().unwrap();
    let observed = dependencies.remove("observed").unwrap();
    assert_eq!(observed["e"]["evidence"], json!(["g"]));
    dependencies.insert("carries:flaky".into(), observed);
    let refused = ReactiveSession::restore_json(SKIPPED_REVEAL, &save.to_string())
        .expect_err("a carries dependency is refused");
    assert!(
        refused.contains("unknown predicate carries:flaky"),
        "{refused}"
    );
}
