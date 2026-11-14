//! D086: an evidence declaration's source is text, quoted or not:
//! `evidence NAME from SOURCE`. Every release from rc.1 to 1.4.1 loads
//! `evidence e2 from sum(for beast as $x: s * 2);` with those words as the
//! source, and loads it inside a `for beast as $x` block too. Main read an
//! across-members form in the words and refused both programs ("no entity is
//! declared `kind beast`, so `sum(for beast as $x: …)` has nothing to
//! expand", and "`sum(for beast as $x: …)` binds $x, which an enclosing for
//! block binds too"). A source is now left as written; a form elsewhere still
//! expands.
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::repeat;
use serde_json::{json, Value};

const HEAD: &str =
    "claim c;\nevidence e from \"e\";\nstate s = 0;\nevent go;\non go set s = s + 1;\n";

const BEASTS: &str = "place p kind room;\nentity a kind beast at p;\nentity b kind beast at p;\n";

fn after_two_events(source: &str) -> Value {
    let mut session = ReactiveSession::from_source(source)
        .unwrap_or_else(|error| panic!("refused: {error}\n{source}"));
    for _ in 0..2 {
        session
            .dispatch_json("go", "{}")
            .unwrap_or_else(|error| panic!("go: {error}\n{source}"));
    }
    serde_json::to_value(session.snapshot()).unwrap()["values"]["s"].clone()
}

#[test]
fn a_source_naming_a_kind_no_entity_has_loads_as_written() {
    let source = format!("{HEAD}evidence e2 from sum(for beast as $x: s * 2);\n");
    assert_eq!(repeat::expand(&source).expect("expands"), source);
    assert_eq!(after_two_events(&source), json!(2.0));
}

#[test]
fn a_source_in_a_block_that_binds_the_same_name_loads() {
    let source = format!(
        "{HEAD}{BEASTS}for beast as $x {{ evidence e_$x from sum(for beast as $x: s); }};\n"
    );
    let expanded = repeat::expand(&source).expect("expands");
    for member in ["a", "b"] {
        let copy = format!("evidence e_{member} from sum(for beast as {member}: s);");
        assert!(expanded.contains(&copy), "{expanded}");
    }
    assert_eq!(after_two_events(&source), json!(2.0));
}

/// A block copies a source's words with its own bindings, as every release
/// did, so a `$` word in them that the block does not bind is refused, as
/// every release refused it. Main had expanded the form in the source, which
/// bound `$y`.
#[test]
fn a_block_reads_the_dollar_words_of_a_source_as_its_own() {
    let source = format!(
        "{HEAD}{BEASTS}for beast as $x {{ evidence e_$x from sum(for beast as $y: s); }};\n"
    );
    let error = ReactiveSession::from_source(&source).expect_err(&source);
    assert!(
        error.contains("$y is not bound in this for block"),
        "{error}"
    );
}

#[test]
fn a_form_outside_a_source_still_expands() {
    let source = format!(
        "{HEAD}{BEASTS}evidence e2 from sum(for beast as $x: s);\nstate t = 0;\non go set t = sum(for beast as $x: 1);\n"
    );
    let expanded = repeat::expand(&source).expect("expands");
    assert!(
        expanded.contains("evidence e2 from sum(for beast as $x: s);"),
        "{expanded}"
    );
    assert!(
        expanded.contains("on go set t = ((1) + (1));"),
        "{expanded}"
    );
    let mut session = ReactiveSession::from_source(&source).expect("loads");
    session.dispatch_json("go", "{}").expect("go");
    let view = serde_json::to_value(session.snapshot()).unwrap();
    assert_eq!(view["values"]["t"], json!(2.0));
}
