//! Declared windows retire the oldest live record of a history, which
//! departs once nothing pins it (spec/caveat-windows-0.1.md,
//! spec/caveat-departure-0.1.md).
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::web::WebReactiveSession;
use serde_json::{json, Value};
use std::collections::BTreeMap;

const WORLD: &str = r#"
claim seen; claim safe;
evidence glimpse from "a glimpse";
evidence taste from "the slime tasted it";
readings sighting from glimpse window 3;
renewable taste window 2;
decisions trust limit 64;
journal window 3;
fn add(total, reading) = total + reading;
event look v min 0 max 10;
event eat; event regrow; event decide; event stop;
on look sample sighting = v supports seen;
on eat reveal taste supports safe;
on regrow renew taste;
on decide when committed(trust) and not reopened(trust) reopen trust because latest(sighting);
on decide commit trust because enough using history_count(sighting);
on stop reveal glimpse supports seen;
on stop reject "not now";
bind hud.count = history_count(sighting);
bind hud.first = 0;
bind hud.first = history_at(sighting, 0) when history_count(sighting) > 0;
bind hud.sum = fold_history(sighting, 0, add);
bind hud.decided = history_count(trust);
bind hud.tasted = observed(taste);
"#;

fn session() -> ReactiveSession {
    ReactiveSession::from_source(WORLD).unwrap_or_else(|error| panic!("fixture: {error}"))
}

fn look(game: &mut ReactiveSession, v: f64) {
    game.apply("look", &BTreeMap::from([("v".into(), v)]))
        .unwrap();
}

fn go(game: &mut ReactiveSession, event: &str) {
    game.apply(event, &BTreeMap::new()).unwrap();
}

fn shot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

#[test]
fn a_window_retires_the_oldest_reading_and_reads_only_live_ones() {
    let mut game = session();
    for v in [1.0, 2.0, 3.0] {
        look(&mut game, v);
    }
    assert!(
        shot(&game).get("retired").is_none(),
        "nothing has retired yet"
    );
    look(&mut game, 4.0);
    let after = shot(&game);
    let kinds = |shot: &Value| {
        shot["effects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|effect| effect["kind"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        after["effects"][0],
        json!({"kind": "retire", "history": "sighting", "record": "sighting@1"}),
        "retirement is reported before the sample that caused it"
    );
    assert_eq!(kinds(&after), ["retire", "sample", "depart"]);
    // Nothing cites sighting@1, so it departs in the same event
    // (spec/caveat-departure-0.1.md).
    assert_eq!(
        after["effects"][2],
        json!({"kind": "depart", "history": "sighting", "record": "sighting@1"}),
        "departure is reported after every other effect"
    );
    assert!(after.get("retired").is_none(), "{after}");
    assert_eq!(after["bindings"]["hud"]["count"], 3.0);
    assert_eq!(after["bindings"]["hud"]["first"], 2.0);
    assert_eq!(after["bindings"]["hud"]["sum"], 9.0);
    assert_eq!(
        after["reading_streams"]["sighting"]["occurrences"][0]["id"],
        "sighting@2"
    );
    look(&mut game, 5.0);
    let later = shot(&game);
    assert_eq!(
        later["reading_streams"]["sighting"]["current"], "sighting@5",
        "a name is never reused"
    );
    assert_eq!(later["effects"][2]["record"], "sighting@2");
}

#[test]
fn a_retired_occurrence_is_not_observed_and_the_current_one_is() {
    let mut game = session();
    go(&mut game, "eat");
    go(&mut game, "regrow");
    go(&mut game, "regrow");
    let after = shot(&game);
    assert_eq!(after["retired"], json!({"taste": 3}));
    assert_eq!(
        after["bindings"]["hud"]["tasted"], false,
        "taste@3 is not revealed yet"
    );
    go(&mut game, "eat");
    assert_eq!(shot(&game)["bindings"]["hud"]["tasted"], true);
}

#[test]
fn the_journal_keeps_its_newest_entries() {
    let mut game = session();
    look(&mut game, 1.0);
    for _ in 0..4 {
        go(&mut game, "decide");
    }
    let after = shot(&game);
    assert!(
        after.get("retired").is_none(),
        "a retired journal entry departs at once: {after}"
    );
    assert_eq!(after["decision_series"]["trust"]["current"], "trust@4");
    let kinds = after["effects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|effect| effect["kind"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        ["retire", "reopen", "retire", "commit", "depart", "depart"],
        "each retirement comes before the effect that adds a record"
    );
    assert_eq!(after["effects"][4]["record"], "journal@3");
    assert_eq!(after["effects"][5]["record"], "journal@4");
    assert_eq!(after["bindings"]["hud"]["decided"], 4.0);
    assert_eq!(after["decision_journal"].as_array().unwrap().len(), 3);
    let save: Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
    assert_eq!(save["journal_departed"], 4);
}

#[test]
fn a_refused_event_retires_nothing() {
    let source = format!("{WORLD}\non look when history_count(sighting) == 3 reject \"full\";");
    let mut refusing = WebReactiveSession::new(&source).unwrap();
    for v in 1..=2 {
        refusing
            .dispatch_outcome("look", &format!("{{\"v\": {v}}}"))
            .unwrap();
    }
    refusing.dispatch_outcome("look", "{\"v\": 3}").unwrap();
    let before = (refusing.snapshot(), refusing.save().unwrap());
    let result: Value =
        serde_json::from_str(&refusing.dispatch_outcome("look", "{\"v\": 4}").unwrap()).unwrap();
    assert_eq!(result["outcome"], "rejected", "{result}");
    assert_eq!((refusing.snapshot(), refusing.save().unwrap()), before);
    assert!(!before.0.contains("\"retired\""));
}

#[test]
fn a_save_with_retirements_restores_and_a_forged_one_is_refused() {
    let mut game = session();
    for v in [1.0, 2.0, 3.0, 4.0] {
        look(&mut game, v);
    }
    go(&mut game, "eat");
    for _ in 0..3 {
        go(&mut game, "regrow");
        go(&mut game, "decide");
    }
    let save = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(WORLD, &save).unwrap();
    assert_eq!(shot(&restored), shot(&game));
    assert_eq!(restored.save_json().unwrap(), save);
    let held: Value = serde_json::from_str(&save).unwrap();
    // A renewable's declared first occurrence never departs.
    assert!(held["retired"].get("taste").is_some(), "{held}");

    let edit = |change: &dyn Fn(&mut Value)| {
        let mut forged: Value = serde_json::from_str(&save).unwrap();
        change(&mut forged);
        ReactiveSession::restore_json(WORLD, &forged.to_string())
    };
    // A live record retired: fewer live records than the window.
    assert!(edit(&|save| {
        save["retired"]["sighting@2"] = json!(5);
    })
    .is_err());
    // More live records than the window.
    assert!(edit(&|save| {
        save["retired"].as_object_mut().unwrap().remove("taste");
    })
    .is_err());
    // Dated in the future.
    assert!(edit(&|save| {
        save["retired"]["taste"] = json!(10_000);
    })
    .is_err());
    // A history without a window: the program has none, so any name it does
    // not hold is refused.
    assert!(edit(&|save| {
        save["retired"]["nothing@1"] = json!(1);
    })
    .is_err());
}

#[test]
fn a_program_without_a_window_saves_no_retired_field() {
    let source = WORLD
        .replace("window 3;\nrenewable", "limit 8;\nrenewable")
        .replace("taste window 2", "taste limit 8")
        .replace("journal window 3;\n", "");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for v in [1.0, 2.0, 3.0, 4.0] {
        look(&mut game, v);
    }
    go(&mut game, "decide");
    assert!(!game.save_json().unwrap().contains("retired"));
    assert!(shot(&game).get("retired").is_none());
}

#[test]
fn a_window_must_be_in_range_and_a_journal_window_declared_once() {
    for bad in [
        "claim c; evidence e from \"e\"; readings r from e window 0;",
        "claim c; evidence e from \"e\"; readings r from e window 257;",
        "decisions d window 3;",
        "journal window 0;",
        "journal window 2;\njournal window 3;",
        "claim c; evidence e from \"e\"; readings journal from e window 2; journal window 2;",
    ] {
        assert!(ReactiveSession::from_source(bad).is_err(), "{bad} loaded");
    }
}

#[test]
fn a_reopen_whose_evidence_retired_still_reads_and_restores() {
    let source = format!(
        "{WORLD}\nevent doubt;\non doubt reopen trust because latest(sighting);\n\
         bind hud.open = reopened(trust);"
    );
    let mut game = ReactiveSession::from_source(&source).unwrap();
    look(&mut game, 1.0);
    go(&mut game, "decide");
    go(&mut game, "doubt");
    for v in [2.0, 3.0, 4.0] {
        look(&mut game, v);
    }
    let after = shot(&game);
    assert_eq!(after["retired"]["sighting@1"], 6, "{after}");
    assert_eq!(after["bindings"]["hud"]["open"], true);
    assert!(
        after["binding_qualifications"]["hud"]["open"]["evidence"]
            .as_array()
            .unwrap()
            .contains(&json!("sighting@1")),
        "the reopen still carries the retired reading it cited"
    );
    let save = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(&source, &save).unwrap();
    assert_eq!(shot(&restored), after);
}

// Since Reactive 0.5 a revision's basis holds its predecessor's, through the
// check that the predecessor was reopened, whatever `using` reads. A departed
// reading reaches a new basis that way as its history's marker, and the save
// still restores (spec/caveat-lineage-compaction-0.1.md).
#[test]
fn a_revision_basis_may_hold_a_departed_reading_through_its_predecessor() {
    let mut game = session();
    for v in 0..6 {
        look(&mut game, f64::from(v));
        go(&mut game, "decide");
    }
    let after = shot(&game);
    let basis = &after["commitment_bases"]["trust@6"]["provenance"];
    let mut marker = basis["departed"][0].clone();
    let root = marker
        .as_object_mut()
        .unwrap()
        .remove("archive_ref")
        .unwrap();
    assert_eq!(root.as_str().unwrap().len(), 64);
    assert_eq!(
        marker,
        json!({"history": "sighting", "read": 2, "from": 1, "through": 2, "departed_at": 12}),
        "{basis}"
    );
    assert!(basis["evidence"]
        .as_array()
        .unwrap()
        .contains(&json!("sighting@3")));
    // The journal's commit of trust@5 still cites sighting@3, so it stays.
    assert_eq!(after["retired"], json!({"sighting@3": 11}));
    let save = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(WORLD, &save).unwrap();
    assert_eq!(shot(&restored), after);
}
