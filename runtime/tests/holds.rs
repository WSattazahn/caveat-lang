//! Holds 0.1: `hold D because E`, `release D because E` and `on_hold(D)`.

use caveat_runtime::reactive::ReactiveSession;

/// A release gate: a ship rests on the latest test report, a new report
/// reopens it, a freeze holds it, and a withdrawn report breaks through.
const GATE: &str = "claim passing;
evidence tool from \"a tool result\";
evidence correction from \"a tool result found wrong\";
evidence team from \"the team's instruction\";
readings tests from tool limit 8;
decisions gate limit 8 reopened by tests;
event report outcome in pass fail;
event decide;
event freeze;
event unfreeze;
event retract;
on report when outcome == outcome.pass sample tests = 1 supports passing;
on report when outcome == outcome.fail sample tests = 0 opposes passing;
on decide commit gate because enough using latest(tests);
on freeze when on_hold(gate) reject \"already frozen\";
on freeze reveal team;
on freeze hold gate because team;
on unfreeze when not on_hold(gate) reject \"not frozen\";
on unfreeze release gate because team;
on retract reveal correction;
on retract withdraw latest(tests) because correction;
on retract when committed(gate) and not reopened(gate) and in_basis(gate, latest(tests)) reopen gate because latest(tests);
bind gate.frozen = on_hold(gate);
bind gate.open = reopened(gate);
";

fn run(session: &mut ReactiveSession, events: &[(&str, &str)]) {
    for (event, payload) in events {
        session
            .dispatch_json(event, payload)
            .unwrap_or_else(|error| panic!("{event} {payload}: {error}"));
    }
}

fn binding(session: &ReactiveSession, property: &str) -> String {
    serde_json::to_string(&session.snapshot().bindings["gate"][property]).unwrap()
}

fn journal(session: &ReactiveSession) -> Vec<String> {
    session
        .snapshot()
        .decision_journal
        .iter()
        .map(|entry| {
            format!(
                "{} {} {}",
                entry.commitment,
                entry.change,
                entry.because.join(",")
            )
        })
        .collect()
}

#[test]
fn a_held_reopening_waits_for_the_release() {
    let mut session = ReactiveSession::from_source(GATE).expect("loads");
    run(
        &mut session,
        &[
            ("report", r#"{"outcome":"pass"}"#),
            ("decide", "{}"),
            ("freeze", "{}"),
            ("report", r#"{"outcome":"pass"}"#),
            ("report", r#"{"outcome":"fail"}"#),
        ],
    );
    assert_eq!(binding(&session, "frozen"), "true");
    assert_eq!(binding(&session, "open"), "false");
    // A commit is still refused: the decision is in force.
    assert!(session.dispatch_json("decide", "{}").is_err());
    run(&mut session, &[("unfreeze", "{}")]);
    assert_eq!(binding(&session, "frozen"), "false");
    assert_eq!(binding(&session, "open"), "true");
    assert_eq!(
        journal(&session),
        [
            "gate@1 committed tests@1",
            "gate@1 held team",
            "gate@1 deferred tests@2",
            "gate@1 deferred tests@3",
            "gate@1 released team",
            "gate@1 reopened tests@2,tests@3",
        ]
    );
}

#[test]
fn a_release_with_nothing_deferred_leaves_the_decision_in_force() {
    let mut session = ReactiveSession::from_source(GATE).expect("loads");
    run(
        &mut session,
        &[
            ("report", r#"{"outcome":"pass"}"#),
            ("decide", "{}"),
            ("freeze", "{}"),
            ("unfreeze", "{}"),
            ("report", r#"{"outcome":"pass"}"#),
        ],
    );
    assert_eq!(
        journal(&session),
        [
            "gate@1 committed tests@1",
            "gate@1 held team",
            "gate@1 released team",
            "gate@1 reopened tests@2",
        ]
    );
}

#[test]
fn a_withdrawal_reaches_a_revision_on_hold() {
    let mut session = ReactiveSession::from_source(GATE).expect("loads");
    run(
        &mut session,
        &[
            ("report", r#"{"outcome":"pass"}"#),
            ("decide", "{}"),
            ("freeze", "{}"),
            ("retract", "{}"),
        ],
    );
    assert_eq!(binding(&session, "frozen"), "false");
    assert_eq!(binding(&session, "open"), "true");
    assert_eq!(
        journal(&session),
        [
            "gate@1 committed tests@1",
            "gate@1 held team",
            "gate@1 reopened tests@1",
        ]
    );
}

#[test]
fn a_save_keeps_the_hold_and_what_it_deferred() {
    let mut session = ReactiveSession::from_source(GATE).expect("loads");
    run(
        &mut session,
        &[
            ("report", r#"{"outcome":"pass"}"#),
            ("decide", "{}"),
            ("freeze", "{}"),
            ("report", r#"{"outcome":"fail"}"#),
        ],
    );
    let save = session.save_json().expect("saves");
    assert!(save.contains(r#""on_hold":["team"]"#), "{save}");
    assert!(save.contains(r#""deferred":["tests@2"]"#), "{save}");
    let mut restored = ReactiveSession::restore_json(GATE, &save).expect("restores");
    assert_eq!(restored.snapshot(), session.snapshot());
    run(&mut restored, &[("unfreeze", "{}")]);
    run(&mut session, &[("unfreeze", "{}")]);
    assert_eq!(restored.snapshot(), session.snapshot());
    let again = ReactiveSession::restore_json(GATE, &restored.save_json().unwrap())
        .expect("restores after the release");
    assert_eq!(again.snapshot(), session.snapshot());
}

#[test]
fn a_program_without_holds_saves_as_before() {
    let source = GATE
        .lines()
        .filter(|line| !line.contains("hold") && !line.contains("release"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut session = ReactiveSession::from_source(&source).expect("loads");
    run(
        &mut session,
        &[("report", r#"{"outcome":"pass"}"#), ("decide", "{}")],
    );
    let save = session.save_json().expect("saves");
    assert!(
        !save.contains("on_hold") && !save.contains("deferred"),
        "{save}"
    );
}

#[test]
fn a_save_cannot_hold_a_series_no_rule_holds() {
    let source = GATE.replace("on freeze hold gate because team;", "");
    let mut session = ReactiveSession::from_source(&source).expect("loads");
    run(
        &mut session,
        &[
            ("report", r#"{"outcome":"pass"}"#),
            ("decide", "{}"),
            ("freeze", "{}"),
        ],
    );
    let save = session.save_json().expect("saves");
    let forged = save.replace(
        r#""current":"gate@1""#,
        r#""current":"gate@1","on_hold":["team"]"#,
    );
    assert_ne!(forged, save);
    let error = ReactiveSession::restore_json(&source, &forged).expect_err("refused");
    assert!(error.contains("hold no source can make"), "{error}");
}

#[test]
fn hold_and_release_name_a_decision_series() {
    for (line, verb) in [
        ("on freeze hold gate because team;", "hold"),
        ("on unfreeze release gate because team;", "release"),
    ] {
        let source = GATE
            .replace("decisions gate limit 8 reopened by tests;", "")
            .replace("on retract when", "on retract when 1 == 2 and")
            .replace(
                "on decide commit gate because enough using latest(tests);",
                "on decide commit gate because enough;",
            )
            .replace("bind gate.frozen = on_hold(gate);\n", "")
            .replace(
                "on freeze when on_hold(gate) reject \"already frozen\";\n",
                "",
            )
            .replace(
                "on unfreeze when not on_hold(gate) reject \"not frozen\";\n",
                "",
            );
        let other = if verb == "hold" {
            "on unfreeze release gate because team;"
        } else {
            "on freeze hold gate because team;"
        };
        let source = source.replace(other, "");
        assert!(source.contains(line), "{source}");
        let error = ReactiveSession::from_source(&source).expect_err("refused");
        assert!(
            error.contains(&format!("{verb} names a decision series; gate is not one")),
            "{error}"
        );
    }
}

#[test]
fn a_program_may_keep_its_own_on_hold() {
    let source = "claim ok;
evidence note from \"a note\";
fn on_hold(x) = x + 1;
state level = 0 min 0 max 9;
event bump;
on bump set level = on_hold(level);
place yard kind ground;
entity meter kind gauge at yard;
bind meter.level = level;
";
    let mut session = ReactiveSession::from_source(source).expect("loads");
    run(&mut session, &[("bump", "{}"), ("bump", "{}")]);
    assert_eq!(
        serde_json::to_string(&session.snapshot().bindings["meter"]["level"]).unwrap(),
        "2.0"
    );
}

#[test]
fn a_hold_keeps_what_it_defers_through_a_window() {
    let source = GATE.replace(
        "readings tests from tool limit 8;",
        "readings tests from tool window 1;\njournal window 2;",
    );
    let mut session = ReactiveSession::from_source(&source).expect("loads");
    run(
        &mut session,
        &[
            ("report", r#"{"outcome":"pass"}"#),
            ("decide", "{}"),
            ("freeze", "{}"),
            ("report", r#"{"outcome":"fail"}"#),
            ("report", r#"{"outcome":"pass"}"#),
            ("report", r#"{"outcome":"pass"}"#),
        ],
    );
    let save = session.save_json().expect("saves");
    assert!(
        save.contains(r#""deferred":["tests@2","tests@3","tests@4"]"#),
        "{save}"
    );
    let restored = ReactiveSession::restore_json(&source, &save).expect("restores");
    assert_eq!(restored.snapshot(), session.snapshot());
    run(&mut session, &[("unfreeze", "{}")]);
    let reopened = session.snapshot().decision_journal.last().unwrap().clone();
    assert_eq!(reopened.change, "reopened");
    assert_eq!(reopened.because, ["tests@2", "tests@3", "tests@4"]);
    let save = session.save_json().expect("saves after the release");
    ReactiveSession::restore_json(&source, &save).expect("restores after the release");
}

#[test]
fn a_procedure_step_can_hold_and_release() {
    let source = GATE
        .replace(
            "on freeze hold gate because team;",
            "proc pause(e evidence) {\n    when true hold gate because e;\n};\non freeze call pause(team);",
        )
        .replace(
            "on unfreeze release gate because team;",
            "proc resume(e evidence) {\n    when true release gate because e;\n};\non unfreeze call resume(team);",
        );
    let mut session = ReactiveSession::from_source(&source).expect("loads");
    run(
        &mut session,
        &[
            ("report", r#"{"outcome":"pass"}"#),
            ("decide", "{}"),
            ("freeze", "{}"),
            ("report", r#"{"outcome":"fail"}"#),
            ("unfreeze", "{}"),
        ],
    );
    assert_eq!(
        journal(&session),
        [
            "gate@1 committed tests@1",
            "gate@1 held team",
            "gate@1 deferred tests@2",
            "gate@1 released team",
            "gate@1 reopened tests@2",
        ]
    );
    let save = session.save_json().expect("saves");
    ReactiveSession::restore_json(&source, &save).expect("restores");
}
