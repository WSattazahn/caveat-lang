//! `reopened by changed STREAM`: reopen only on a reading whose value
//! differs from the stream's previous one. See
//! spec/caveat-changed-triggers-0.1.md and issue #223.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// #223's re-check: every reading is taken again, most unchanged.
const RECHECK: &str = r#"
claim present;
evidence probe from "the presence probe";
readings n20 from probe limit 8;
readings n22 from probe limit 8;
decisions install limit 4 reopened by changed n20, n22;
decisions watch limit 4 reopened by changed n20 supporting present;
event observe_n20 value min 0 max 1;
event observe_n22 value min 0 max 1;
event observe_supporting value min 0 max 1;
event assess;
on observe_n20 sample n20 = value;
on observe_n22 sample n22 = value;
on observe_supporting sample n20 = value supports present;
on assess when not committed(install) or reopened(install)
  commit install because enough citing latest(n20), latest(n22);
on assess when not committed(watch) or reopened(watch)
  commit watch because enough citing latest(n20);
"#;

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn run(game: &mut ReactiveSession, events: &[(&str, &str)]) {
    for (event, payload) in events {
        game.dispatch_json(event, payload)
            .unwrap_or_else(|error| panic!("{event} {payload}: {error}"));
    }
}

fn journal(game: &ReactiveSession) -> Vec<(String, String, Vec<String>)> {
    snapshot(game)["decision_journal"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry["commitment"].as_str().unwrap().to_string(),
                entry["change"].as_str().unwrap().to_string(),
                entry["because"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|name| name.as_str().unwrap().to_string())
                    .collect(),
            )
        })
        .collect()
}

fn assessed() -> ReactiveSession {
    let mut game = ReactiveSession::from_source(RECHECK).unwrap();
    run(
        &mut game,
        &[
            ("observe_n20", r#"{"value": 1}"#),
            ("observe_n22", r#"{"value": 0}"#),
            ("assess", "{}"),
        ],
    );
    game
}

#[test]
fn issue_223_an_unchanged_reading_does_not_reopen() {
    let mut game = assessed();
    let before = journal(&game);
    run(&mut game, &[("observe_n20", r#"{"value": 1}"#)]);
    assert_eq!(journal(&game), before);
    // The plain trigger on n22 still fires on a repeated value.
    run(&mut game, &[("observe_n22", r#"{"value": 0}"#)]);
    let after = journal(&game);
    assert_eq!(
        after.last().unwrap(),
        &(
            "install@1".to_string(),
            "reopened".to_string(),
            vec!["n22@2".to_string()]
        )
    );
}

#[test]
fn a_changed_reading_reopens_and_the_journal_names_it() {
    let mut game = assessed();
    run(
        &mut game,
        &[
            ("observe_n20", r#"{"value": 1}"#),
            ("observe_n20", r#"{"value": 0}"#),
        ],
    );
    let entries = journal(&game);
    let reopened = entries
        .iter()
        .filter(|(_, change, _)| change == "reopened")
        .collect::<Vec<_>>();
    assert_eq!(
        reopened,
        [&(
            "install@1".to_string(),
            "reopened".to_string(),
            vec!["n20@3".to_string()]
        )]
    );
}

#[test]
fn a_first_reading_is_a_change() {
    let source = RECHECK.replace(
        "on assess when not committed(install) or reopened(install)\n  commit install because enough citing latest(n20), latest(n22);",
        "on assess when not committed(install) or reopened(install)\n  commit install because enough citing latest(n22);",
    );
    assert_ne!(source, RECHECK);
    let source = source.replace(
        "on assess when not committed(watch) or reopened(watch)\n  commit watch because enough citing latest(n20);\n",
        "",
    );
    let mut game = ReactiveSession::from_source(&source).unwrap();
    run(
        &mut game,
        &[
            ("observe_n22", r#"{"value": 0}"#),
            ("assess", "{}"),
            ("observe_n20", r#"{"value": 1}"#),
        ],
    );
    assert_eq!(
        journal(&game).last().unwrap(),
        &(
            "install@1".to_string(),
            "reopened".to_string(),
            vec!["n20@1".to_string()]
        )
    );
}

#[test]
fn the_claim_filter_applies_to_the_new_reading_and_the_previous_value_is_the_streams() {
    let mut game = assessed();
    // Same value as n20@1, now with a stance: not a change.
    run(&mut game, &[("observe_supporting", r#"{"value": 1}"#)]);
    assert!(journal(&game)
        .iter()
        .all(|(_, change, _)| change == "committed"));
    // A changed neutral reading reopens install but not watch, whose
    // trigger wants a supporting reading.
    run(&mut game, &[("observe_n20", r#"{"value": 0}"#)]);
    let reopened = journal(&game)
        .into_iter()
        .filter(|(_, change, _)| change == "reopened")
        .map(|(commitment, _, _)| commitment)
        .collect::<Vec<_>>();
    assert_eq!(reopened, ["install@1"]);
    // A changed supporting reading reopens watch.
    run(&mut game, &[("observe_supporting", r#"{"value": 1}"#)]);
    assert!(journal(&game)
        .iter()
        .any(|(commitment, change, _)| commitment == "watch@1" && change == "reopened"));
}

#[test]
fn a_full_window_compares_with_the_reading_it_retires() {
    let source = r#"
evidence probe from "a probe";
readings r from probe window 1;
decisions d limit 4 reopened by changed r;
event look value min 0 max 9;
event decide;
on look sample r = value;
on decide when not committed(d) or reopened(d) commit d because enough citing latest(r);
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    run(
        &mut game,
        &[
            ("look", r#"{"value": 4}"#),
            ("decide", "{}"),
            ("look", r#"{"value": 4}"#),
        ],
    );
    assert!(journal(&game)
        .iter()
        .all(|(_, change, _)| change == "committed"));
    run(&mut game, &[("look", r#"{"value": 5}"#)]);
    assert_eq!(journal(&game).last().unwrap().1, "reopened");
}

#[test]
fn reopenings_save_and_restore() {
    let mut game = assessed();
    run(
        &mut game,
        &[
            ("observe_n20", r#"{"value": 1}"#),
            ("observe_n20", r#"{"value": 0}"#),
            ("assess", "{}"),
        ],
    );
    let saved = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(RECHECK, &saved).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&game));
    assert_eq!(restored.save_json().unwrap(), saved);
}

#[test]
fn load_rules() {
    // A stream named `changed` keeps both of its forms.
    let named = r#"
claim c;
evidence e from "e";
readings changed from e limit 2;
decisions a limit 2 reopened by changed;
decisions b limit 2 reopened by changed supporting c;
decisions d limit 2 reopened by changed changed supporting c;
event go;
on go sample changed = 1 supports c;
"#;
    assert!(ReactiveSession::from_source(named).is_ok());
    for (line, expected) in [
        (
            "decisions x limit 2 reopened by changed n20 n22;",
            "reopened by takes",
        ),
        (
            "decisions x limit 2 reopened by n20 changed;",
            "reopened by takes",
        ),
        (
            "decisions x limit 2 reopened by changed nowhere;",
            "nowhere",
        ),
    ] {
        let error = ReactiveSession::from_source(&format!("{RECHECK}\n{line}"))
            .err()
            .unwrap_or_else(|| panic!("{line} should not load"));
        assert!(error.contains(expected), "{line}: {error}");
    }
}
