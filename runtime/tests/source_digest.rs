//! Saves record their source's SHA-256, and decisions made before saves did
//! show it as not recorded (owner decision D6; spec/caveat-save-0.1.md, "Source
//! digest"). The digest identifies the source. It does not authenticate a
//! save, and restore under other source stays refused as before.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const SOURCE: &str = include_str!("fixtures/rc2-thermostat.cav");
const RC2_SAVE: &str = include_str!("fixtures/rc2-thermostat.save.json");

fn read(game: &mut ReactiveSession, value: f64) {
    game.apply("read", &BTreeMap::from([("value".to_string(), value)]))
        .unwrap();
}

fn save(game: &ReactiveSession) -> Value {
    serde_json::from_str(&game.save_json().unwrap()).unwrap()
}

fn refusal(source: &str, save: &Value) -> String {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Ok(_) => panic!("the save is refused"),
        Err(error) => error,
    }
}

#[test]
fn a_new_save_records_the_sha256_of_its_exact_source() {
    let mut game = ReactiveSession::from_source(SOURCE).unwrap();
    read(&mut game, 17.0);
    let expected = format!("{:x}", Sha256::digest(SOURCE.as_bytes()));
    assert_eq!(game.snapshot().source_sha256, expected);
    assert_eq!(game.snapshot().source_unrecorded_through, 0);
    let saved = save(&game);
    assert_eq!(saved["source_sha256"], json!(expected));
    assert!(saved.get("source_unrecorded_through").is_none());
    let restored = ReactiveSession::restore_json(SOURCE, &saved.to_string()).unwrap();
    assert_eq!(restored.snapshot(), game.snapshot());
    assert_eq!(save(&restored), saved);
}

#[test]
fn an_older_save_shows_its_decisions_source_as_not_recorded() {
    // Made by the rc.2 release, before saves recorded a digest.
    assert!(!RC2_SAVE.contains("source_sha256"));
    let mut game = ReactiveSession::restore_json(SOURCE, RC2_SAVE).unwrap();
    let snapshot = game.snapshot();
    // The source matched, so its digest is known for what happens next; the
    // two events already in the save stay not recorded, never a digest
    // computed now.
    assert_eq!(
        snapshot.source_sha256,
        format!("{:x}", Sha256::digest(SOURCE.as_bytes()))
    );
    assert_eq!(snapshot.source_unrecorded_through, 2);
    read(&mut game, 22.0);
    assert_eq!(game.snapshot().source_unrecorded_through, 2);
    // Saving and restoring keeps the boundary.
    let saved = save(&game);
    assert_eq!(saved["source_unrecorded_through"], json!(2));
    let restored = ReactiveSession::restore_json(SOURCE, &saved.to_string()).unwrap();
    assert_eq!(restored.snapshot().source_unrecorded_through, 2);
}

#[test]
fn a_save_under_edited_source_is_refused_as_before() {
    let mut game = ReactiveSession::from_source(SOURCE).unwrap();
    read(&mut game, 17.0);
    let saved = save(&game);
    let edited = format!("{SOURCE}\n// a comment\n");
    assert_eq!(
        refusal(&edited, &saved),
        "cannot restore save: it belongs to a different program"
    );
    // Without its digest too: the refusal does not rest on the new field.
    let mut older = saved.clone();
    older.as_object_mut().unwrap().remove("source_sha256");
    assert_eq!(
        refusal(&edited, &older),
        "cannot restore save: it belongs to a different program"
    );
}

#[test]
fn a_save_whose_digest_fields_disagree_with_it_is_refused() {
    let mut game = ReactiveSession::from_source(SOURCE).unwrap();
    read(&mut game, 17.0);
    read(&mut game, 25.0);
    let saved = save(&game);

    let mut other = saved.clone();
    other["source_sha256"] = json!(format!("{:x}", Sha256::digest(b"another source")));
    assert_eq!(
        refusal(SOURCE, &other),
        "cannot restore save: its source digest is not this source's"
    );

    let mut late = saved.clone();
    late["source_unrecorded_through"] = json!(3);
    assert_eq!(
        refusal(SOURCE, &late),
        "cannot restore save: source_unrecorded_through is past its sequence"
    );

    let mut bare = saved.clone();
    bare.as_object_mut().unwrap().remove("source_sha256");
    bare["source_unrecorded_through"] = json!(1);
    assert_eq!(
        refusal(SOURCE, &bare),
        "cannot restore save: source_unrecorded_through without source_sha256"
    );

    let mut within = saved;
    within["source_unrecorded_through"] = json!(2);
    let restored = ReactiveSession::restore_json(SOURCE, &within.to_string()).unwrap();
    assert_eq!(restored.snapshot().source_unrecorded_through, 2);
}
