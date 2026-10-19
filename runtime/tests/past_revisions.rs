//! `rests_on_withdrawn(revision(SERIES, INDEX))`: whether a revision of a
//! decision series, current or past, rests on withdrawn evidence. See
//! spec/caveat-past-revisions-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// The spec's example: three ship decisions, each on the report current when
/// it was made, then an operator says which report was wrong.
fn ledger(renewal: &str) -> String {
    format!(
        r#"
claim passing;
evidence tests from "test runner";
renewable tests {renewal};
evidence operator from "the operator's correction";
decisions ship limit 8;
journal window 2;
state answer = 0;
state current = 0;
event first_report;
event report;
event decide;
event correct n min 0 max 9;
event ask k min -1 max 9;
on first_report reveal tests supports passing;
on report renew tests;
on report reveal tests supports passing;
on report when committed(ship) and not reopened(ship) reopen ship because tests;
on decide commit ship because enough using qualified(1, tests);
on correct when not observed(occurrence(tests, n)) reject "no such report";
on correct reveal operator;
on correct withdraw occurrence(tests, n) because operator;
on ask set answer = if(rests_on_withdrawn(revision(ship, k)), 1, 0);
on ask set current = if(rests_on_withdrawn(ship), 1, 0);
"#
    )
}

fn run(game: &mut ReactiveSession, events: &[(&str, &str)]) {
    for (event, payload) in events {
        game.dispatch_json(event, payload)
            .unwrap_or_else(|error| panic!("{event} {payload}: {error}"));
    }
}

fn value(game: &ReactiveSession, name: &str) -> f64 {
    let shown = serde_json::to_value(game.snapshot()).unwrap();
    shown["qualified_values"][name]["value"].as_f64().unwrap()
}

/// `answer` for revision `k`.
fn ask(game: &mut ReactiveSession, k: f64) -> f64 {
    run(game, &[("ask", &format!(r#"{{"k":{k}}}"#))]);
    value(game, "answer")
}

/// Three revisions: ship@1 on `tests`, ship@2 on `tests@2`, ship@3 on
/// `tests@3`.
fn three_ships(renewal: &str) -> ReactiveSession {
    let mut game = ReactiveSession::from_source(&ledger(renewal)).unwrap();
    run(
        &mut game,
        &[
            ("first_report", "{}"),
            ("decide", "{}"),
            ("report", "{}"),
            ("decide", "{}"),
            ("report", "{}"),
            ("decide", "{}"),
        ],
    );
    game
}

#[test]
fn a_past_revision_answers_for_its_own_grounds() {
    let mut game = three_ships("limit 8");
    for k in [1.0, 2.0, 3.0] {
        assert_eq!(ask(&mut game, k), 0.0, "revision {k} before any withdrawal");
    }
    run(&mut game, &[("correct", r#"{"n":2}"#)]);
    assert_eq!(ask(&mut game, 1.0), 0.0);
    assert_eq!(ask(&mut game, 2.0), 1.0);
    assert_eq!(ask(&mut game, 3.0), 0.0);
    // The current revision, ship@3, rests on tests@3, so the series' own
    // question still answers false.
    assert_eq!(value(&game, "current"), 0.0);
    run(&mut game, &[("correct", r#"{"n":1}"#)]);
    assert_eq!(ask(&mut game, 1.0), 1.0);
    assert_eq!(ask(&mut game, 2.0), 1.0);
}

#[test]
fn the_current_revision_answers_as_the_series_does() {
    let mut game = three_ships("limit 8");
    run(&mut game, &[("correct", r#"{"n":3}"#)]);
    assert_eq!(ask(&mut game, 3.0), 1.0);
    assert_eq!(value(&game, "current"), 1.0);
}

#[test]
fn an_index_naming_no_revision_answers_false() {
    let mut game = three_ships("limit 8");
    for n in [1, 2, 3] {
        run(&mut game, &[("correct", &format!(r#"{{"n":{n}}}"#))]);
    }
    for k in [-1.0, 0.0, 1.5, 4.0, 9.0] {
        assert_eq!(ask(&mut game, k), 0.0, "index {k}");
    }
    // Before any decision, the series has no revision to name.
    let mut fresh = ReactiveSession::from_source(&ledger("limit 8")).unwrap();
    run(
        &mut fresh,
        &[("first_report", "{}"), ("correct", r#"{"n":1}"#)],
    );
    assert_eq!(ask(&mut fresh, 1.0), 0.0);
}

/// With windows on the report and the journal, a withdrawn report retires as
/// later ones arrive, and no journal entry still cites it. A series
/// read through `revision(…)` keeps every revision's grounds pinned, so the
/// report stays held, its withdrawal stands, and the answer does not change
/// with no event about the decision.
#[test]
fn a_window_does_not_change_a_past_revisions_answer() {
    let mut game = three_ships("window 2");
    run(&mut game, &[("correct", r#"{"n":2}"#)]);
    assert_eq!(ask(&mut game, 2.0), 1.0);
    for _ in 0..4 {
        run(&mut game, &[("report", "{}"), ("decide", "{}")]);
    }
    assert_eq!(ask(&mut game, 2.0), 1.0);
    assert_eq!(ask(&mut game, 1.0), 0.0);
    let shown = serde_json::to_value(game.snapshot()).unwrap();
    assert!(
        shown["withdrawals"]
            .as_array()
            .unwrap()
            .iter()
            .any(|withdrawal| withdrawal["evidence"] == "tests@2"),
        "the withdrawal of tests@2 stands: {}",
        shown["withdrawals"]
    );
    // Restored, the session answers the same and keeps going.
    let save = game.save_json().unwrap();
    let mut restored = ReactiveSession::restore_json(&ledger("window 2"), &save).unwrap();
    assert_eq!(ask(&mut restored, 2.0), 1.0);
    run(&mut restored, &[("report", "{}"), ("decide", "{}")]);
    assert_eq!(ask(&mut restored, 2.0), 1.0);
}

/// A program that does not read past revisions keeps today's pins: only the
/// current revision's grounds and the journal's window stay, so a superseded
/// revision's report departs with its withdrawal, as it did before this
/// profile.
#[test]
fn a_series_not_read_through_revision_keeps_todays_pins() {
    let plain = ledger("window 2").replace(
        "on ask set answer = if(rests_on_withdrawn(revision(ship, k)), 1, 0);\n",
        "",
    );
    let mut game = ReactiveSession::from_source(&plain).unwrap();
    run(
        &mut game,
        &[
            ("first_report", "{}"),
            ("decide", "{}"),
            ("report", "{}"),
            ("decide", "{}"),
            ("correct", r#"{"n":2}"#),
        ],
    );
    for _ in 0..4 {
        run(&mut game, &[("report", "{}"), ("decide", "{}")]);
    }
    let shown = serde_json::to_value(game.snapshot()).unwrap();
    assert!(
        !shown["withdrawals"]
            .as_array()
            .is_some_and(|withdrawals| withdrawals
                .iter()
                .any(|withdrawal| withdrawal["evidence"] == "tests@2")),
        "{}",
        shown["withdrawals"]
    );
}

#[test]
fn revision_names_a_decision_series_and_only_in_rests_on_withdrawn() {
    let base = ledger("limit 8");
    let load = |line: &str| {
        ReactiveSession::from_source(&format!(
            "{base}event other;\non other commit once because enough using 1;\n{line}\n"
        ))
        .err()
        .unwrap_or_default()
    };
    assert!(
        load("on other set answer = if(rests_on_withdrawn(revision(once, 1)), 1, 0);")
            .contains("names no decision series")
    );
    assert!(
        load("on other set answer = if(rests_on_withdrawn(revision(nothing, 1)), 1, 0);")
            .contains("names no decision series")
    );
    assert!(!load("on other set answer = if(committed(revision(ship, 1)), 1, 0);").is_empty());
    assert!(!load("on other set answer = if(withdrawn(revision(ship, 1)), 1, 0);").is_empty());
    assert!(
        !load("on other set answer = if(rests_on_withdrawn(revision(ship)), 1, 0);").is_empty()
    );
}

/// The answer carries the withdrawal's reason into the value's lineage, as
/// `rests_on_withdrawn(SERIES)` does.
#[test]
fn the_answer_carries_the_withdrawals_reason() {
    let mut game = three_ships("limit 8");
    run(&mut game, &[("correct", r#"{"n":1}"#)]);
    ask(&mut game, 1.0);
    let shown: Value = serde_json::to_value(game.snapshot()).unwrap();
    let evidence = &shown["qualified_values"]["answer"]["provenance"]["evidence"];
    assert!(
        evidence
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "operator"),
        "{evidence}"
    );
}
