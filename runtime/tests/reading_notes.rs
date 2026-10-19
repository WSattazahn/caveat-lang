//! `sample STREAM = EXPRESSION … noting EXPRESSION`: a reading keeps the text
//! that was observed. See spec/caveat-reading-notes-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

const PROGRAM: &str = r#"
claim compatible;
evidence registry from "the npm registry";
readings node_min from registry limit 8;
identifiers limit 64;
state seen = 0;
event read minimum min 0 max 999999, raw id;
event plain minimum min 0 max 999999;
on read sample node_min = minimum supports compatible noting raw;
on read set seen = latest(node_min);
on plain sample node_min = minimum supports compatible;
"#;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("load: {error}"))
}

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn notes(game: &ReactiveSession) -> Vec<Value> {
    snapshot(game)["reading_streams"]["node_min"]["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .map(|occurrence| occurrence.get("note").cloned().unwrap_or(Value::Null))
        .collect()
}

fn load_error(source: &str) -> String {
    ReactiveSession::from_source(source)
        .err()
        .unwrap_or_default()
}

#[test]
fn a_reading_keeps_the_text_it_was_noted_with() {
    let mut game = session(PROGRAM);
    game.dispatch_json(
        "read",
        r#"{"minimum":222202,"raw":"^22.22.2 || ^24.15.0 || >=26.0.0"}"#,
    )
    .unwrap();
    game.dispatch_json("plain", r#"{"minimum":222200}"#)
        .unwrap();
    assert_eq!(
        notes(&game),
        [json!("^22.22.2 || ^24.15.0 || >=26.0.0"), Value::Null]
    );
    // The effect reports it, and the value is the number as before.
    let shown = snapshot(&game);
    assert_eq!(shown["qualified_values"]["seen"]["value"], 222202.0);
    let mut fresh = session(PROGRAM);
    fresh
        .dispatch_json("read", r#"{"minimum":1,"raw":"1.0.0"}"#)
        .unwrap();
    let effects = &snapshot(&fresh)["effects"];
    assert!(
        effects
            .as_array()
            .unwrap()
            .iter()
            .any(|effect| effect["kind"] == "sample"
                && effect["id"] == "node_min@1"
                && effect["note"] == "1.0.0"),
        "{effects}"
    );
}

#[test]
fn a_note_adds_nothing_to_lineage_or_grounds() {
    let mut noted = session(PROGRAM);
    noted
        .dispatch_json("read", r#"{"minimum":5,"raw":"five"}"#)
        .unwrap();
    let mut plain = session(PROGRAM);
    plain.dispatch_json("plain", r#"{"minimum":5}"#).unwrap();
    let reading = |game: &ReactiveSession| {
        snapshot(game)["reading_streams"]["node_min"]["occurrences"][0]["provenance"].clone()
    };
    assert_eq!(reading(&noted), reading(&plain));
}

#[test]
fn a_save_restores_its_notes_and_refuses_one_no_sample_could_take() {
    let mut game = session(PROGRAM);
    game.dispatch_json("read", r#"{"minimum":7,"raw":"7.0.0"}"#)
        .unwrap();
    game.dispatch_json("plain", r#"{"minimum":8}"#).unwrap();
    let save = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(PROGRAM, &save).unwrap();
    assert_eq!(restored.save_json().unwrap(), save);
    assert_eq!(notes(&restored), [json!("7.0.0"), Value::Null]);

    // A note that is not an identifier the save holds.
    let mut forged: Value = serde_json::from_str(&save).unwrap();
    forged["reading_streams"]["node_min"]["occurrences"][1]["note"] = json!("9.9.9");
    assert!(ReactiveSession::restore_json(PROGRAM, &forged.to_string())
        .err()
        .unwrap()
        .contains("has a note no sample of this program could take"));

    // A note on a stream no `noting` sample takes.
    let unnoted = PROGRAM.replace(" noting raw;", ";");
    let mut other = session(&unnoted);
    other
        .dispatch_json("read", r#"{"minimum":7,"raw":"7.0.0"}"#)
        .unwrap();
    let mut forged: Value = serde_json::from_str(&other.save_json().unwrap()).unwrap();
    forged["reading_streams"]["node_min"]["occurrences"][0]["note"] = json!("7.0.0");
    assert!(ReactiveSession::restore_json(&unnoted, &forged.to_string())
        .err()
        .unwrap()
        .contains("has a note no sample of this program could take"));
}

#[test]
fn a_note_that_is_not_an_identifier_refuses_the_event() {
    let source = PROGRAM.replace("noting raw;", "noting minimum;");
    let mut game = session(&source);
    let outcome = serde_json::to_value(
        game.dispatch_outcome_json("read", r#"{"minimum":5,"raw":"x"}"#)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(outcome["outcome"], "rejected", "{outcome}");
    assert_eq!(outcome["code"], "expression", "{outcome}");
    // Nothing the event did is kept.
    assert!(notes(&game).is_empty());
}

#[test]
fn noting_parses_only_where_it_names_a_note() {
    // A neutral sample noted in a procedure step, and one noting a state.
    let source = r#"
evidence registry from "the npm registry";
readings seen from registry limit 4;
identifiers limit 8;
event read v min 0 max 9, raw id;
proc take(v, h) { sample seen = v noting h; };
on read call take(v, raw);
"#;
    let mut game = session(source);
    game.dispatch_json("read", r#"{"v":1,"raw":"one"}"#)
        .unwrap();
    assert_eq!(
        snapshot(&game)["reading_streams"]["seen"]["occurrences"][0]["note"],
        "one"
    );
    let source = r#"
evidence registry from "the npm registry";
readings seen from registry limit 4;
identifiers limit 8;
state handle = 0;
event read v min 0 max 9, raw id;
on read set handle = raw;
on read sample seen = v noting handle;
"#;
    let mut game = session(source);
    game.dispatch_json("read", r#"{"v":1,"raw":"one"}"#)
        .unwrap();
    assert_eq!(
        snapshot(&game)["reading_streams"]["seen"]["occurrences"][0]["note"],
        "one"
    );
    // A 1.0 program may name a state `noting`; it still loads and means
    // what it meant.
    let source = r#"
claim c;
evidence registry from "the npm registry";
readings seen from registry limit 4;
state noting = 3;
event read;
on read sample seen = noting supports c;
on read sample seen = 2 * noting;
"#;
    let mut game = session(source);
    game.dispatch_json("read", "{}").unwrap();
    let shown = snapshot(&game);
    let occurrences = shown["reading_streams"]["seen"]["occurrences"]
        .as_array()
        .unwrap();
    assert_eq!(occurrences.len(), 2);
    assert!(occurrences
        .iter()
        .all(|occurrence| occurrence.get("note").is_none()));
    // A note must be text: a handle read through `id_text`.
    assert!(!load_error(
        "evidence r from \"r\";\nreadings s from r limit 2;\nevent e;\non e sample s = 1 noting 1 > 0;\n"
    )
    .is_empty());
}
