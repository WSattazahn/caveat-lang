//! Speed plan item 6: within one event, the lineage a `qualified` read gives
//! evidence is kept and handed out again until anything is written to the
//! graph, the names or the take-backs (runtime/src/reactive_reads.rs). These
//! programs write between two reads of the same evidence in one event, so a
//! lineage kept past the write would show in the second read. Each second
//! read must give what the same read gives in an event of its own, after
//! the write. Debug builds also make every kept read again and compare.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

fn run(source: &str, events: &[&str]) -> ReactiveSession {
    let mut session = ReactiveSession::from_source(source).unwrap();
    for event in events {
        let outcome = session
            .dispatch_outcome_json(event, "{}")
            .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
        let outcome: Value = serde_json::to_value(outcome).unwrap();
        assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    }
    session
}

/// The caveats of state `name`'s lineage and of its grounds.
fn caveats(session: &ReactiveSession, name: &str) -> (Value, Value) {
    let snapshot = serde_json::to_value(session.snapshot()).unwrap();
    (
        snapshot["qualified_values"][name]["provenance"]["caveats"].clone(),
        snapshot["value_grounds"][name]["caveats"].clone(),
    )
}

const DECLARATIONS: &str = r#"
claim c;
caveat k consequence material;
caveat m consequence material;
m qualifies c;
evidence e from "x";
state before = 0;
state after = 0;
state alone = 0;
event setup;
event go;
event read;
on setup reveal e supports c;
on read set alone = qualified(1, e);
"#;

/// `go` reads `e`, makes `write`, and reads `e` again; `read`, after `go`,
/// reads it in an event of its own.
fn second_read_sees_the_write(setup: &[&str], write: &str, expect: &[&str]) {
    let source = format!(
        "{DECLARATIONS}{}\non go set before = qualified(1, e);\non go {write};\non go set after = qualified(1, e);\n",
        setup
            .iter()
            .map(|line| format!("on setup {line};\n"))
            .collect::<String>()
    );
    let session = run(&source, &["setup", "go", "read"]);
    let (after, after_grounds) = caveats(&session, "after");
    let (alone, alone_grounds) = caveats(&session, "alone");
    assert_eq!(after, alone, "{write}: the read after the write");
    assert_eq!(
        after_grounds, alone_grounds,
        "{write}: the grounds after the write"
    );
    let expect = Value::from(expect.to_vec());
    assert_eq!(after, expect, "{write}");
}

#[test]
fn a_qualification_between_two_reads_reaches_the_second() {
    second_read_sees_the_write(&[], "qualify e with k", &["k", "m"]);
}

#[test]
fn a_take_back_between_two_reads_reaches_the_second() {
    second_read_sees_the_write(&["qualify e with k"], "unqualify e with k", &["m"]);
}

#[test]
fn a_relation_revealed_between_two_reads_reaches_the_second() {
    let source = format!(
        "{DECLARATIONS}claim d;\nk qualifies d;\non go set before = qualified(1, e);\non go reveal e opposes d;\non go set after = qualified(1, e);\n"
    );
    let session = run(&source, &["setup", "go", "read"]);
    assert_eq!(caveats(&session, "after"), caveats(&session, "alone"));
    assert_eq!(caveats(&session, "after").0, Value::from(vec!["k", "m"]));
    assert_eq!(caveats(&session, "before").0, Value::from(vec!["m"]));
}

/// With nothing written between them, the two reads of one event agree,
/// and with the read in an event of its own.
#[test]
fn two_reads_with_nothing_written_between_agree() {
    let source = format!(
        "{DECLARATIONS}on setup qualify e with k;\non go set before = qualified(1, e) + qualified(2, e);\non go set after = qualified(1, e);\n"
    );
    let session = run(&source, &["setup", "go", "read"]);
    assert_eq!(caveats(&session, "before"), caveats(&session, "after"));
    assert_eq!(caveats(&session, "after"), caveats(&session, "alone"));
    assert_eq!(caveats(&session, "after").0, Value::from(vec!["k", "m"]));
}
