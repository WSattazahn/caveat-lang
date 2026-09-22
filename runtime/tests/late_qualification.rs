//! `qualify EVIDENCE with CAVEAT`: a caveat learned after the fact.
//! See spec/caveat-late-qualification-0.1.md.
use caveat_runtime::reactive::{Provenance, ReactiveSession};
use std::collections::BTreeSet;

const PROGRAM: &str = r#"
claim safe;
evidence taste from "a taste";
evidence sight from "a sighting";
caveat faded consequence material;
caveat memory_is_short consequence low;
memory_is_short qualifies faded;
state from_taste = 0;
state from_sight = 0;
state both = 0;
event observe;
event forget;
on observe reveal taste supports safe;
on observe reveal sight supports safe;
on observe set from_taste = qualified(1, taste);
on observe set from_sight = qualified(1, sight);
on observe set both = from_taste + from_sight;
on observe commit trust because enough using from_taste;
on forget qualify taste with faded;
bind hud.text = "known" when from_taste > 0 because from_taste;
"#;

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).into()).collect()
}

fn caveats(provenance: &Provenance) -> BTreeSet<String> {
    provenance.caveats.clone()
}

#[test]
fn everything_built_on_the_evidence_gains_the_caveat() {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    game.dispatch_json("observe", "{}").unwrap();
    let shown = game.dispatch_json("forget", "{}").unwrap();
    let expected = names(&["faded", "memory_is_short"]);
    assert_eq!(
        caveats(&shown.qualified_values["from_taste"].provenance),
        expected
    );
    assert_eq!(caveats(&shown.value_grounds["from_taste"]), expected);
    assert_eq!(
        caveats(&shown.qualified_values["both"].provenance),
        expected
    );
    assert_eq!(caveats(&shown.value_grounds["both"]), expected);
    // What does not depend on the taste is untouched.
    assert!(shown.qualified_values["from_sight"]
        .provenance
        .caveats
        .is_empty());
    assert!(shown.value_grounds["from_sight"].caveats.is_empty());
    // A shown explanation citing it carries it on the same event.
    assert_eq!(
        caveats(&shown.binding_explanations["hud"]["text"]),
        expected
    );
}

#[test]
fn a_decision_already_made_keeps_what_it_was_made_on() {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    game.dispatch_json("observe", "{}").unwrap();
    let shown = game.dispatch_json("forget", "{}").unwrap();
    assert!(shown.commitment_bases["trust"]
        .provenance
        .caveats
        .is_empty());
    assert!(shown.commitment_grounds["trust"].caveats.is_empty());
    assert!(shown.decision_journal[0].caveats.is_empty());
}

#[test]
fn later_uses_of_the_evidence_inherit_it_through_the_graph() {
    let mut game = ReactiveSession::from_source(&format!(
        "{PROGRAM}\nstate later = 0;\nevent reuse;\non reuse set later = qualified(2, taste);"
    ))
    .unwrap();
    game.dispatch_json("observe", "{}").unwrap();
    game.dispatch_json("forget", "{}").unwrap();
    let shown = game.dispatch_json("reuse", "{}").unwrap();
    assert_eq!(
        caveats(&shown.value_grounds["later"]),
        names(&["faded", "memory_is_short"])
    );
    assert!(shown
        .relations
        .iter()
        .any(|r| r.from == "faded" && r.relation == "qualifies" && r.to == "taste"));
}

#[test]
fn qualifying_is_idempotent_reported_and_atomic() {
    let mut game = ReactiveSession::from_source(&format!(
        "{PROGRAM}\nevent broken;\non broken qualify taste with faded;\non broken set both = 1 / 0;"
    ))
    .unwrap();
    game.dispatch_json("observe", "{}").unwrap();
    let before = game.snapshot();
    assert!(game.dispatch_json("broken", "{}").is_err());
    assert_eq!(game.snapshot(), before, "a failed event qualifies nothing");
    let first = game.dispatch_json("forget", "{}").unwrap();
    assert!(first.effects.iter().any(|e| {
        let e = serde_json::to_value(e).unwrap();
        e["kind"] == "qualify" && e["evidence"] == "taste" && e["caveat"] == "faded"
    }));
    let second = game.dispatch_json("forget", "{}").unwrap();
    assert_eq!(second.qualified_values, first.qualified_values);
    assert_eq!(
        second
            .relations
            .iter()
            .filter(|r| r.relation == "qualifies" && r.to == "taste")
            .count(),
        1
    );
}

#[test]
fn unobserved_evidence_and_unknown_names_are_rejected() {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    let error = game.dispatch_json("forget", "{}").unwrap_err();
    assert!(
        error.contains("cannot qualify unobserved evidence taste"),
        "{error}"
    );
    for body in [
        "event e; on e qualify nowhere with faded;",
        "event e; on e qualify taste with nothing_known;",
        "event e; on e qualify taste faded;",
    ] {
        let source =
            format!("claim safe; evidence taste from \"t\"; caveat faded consequence low; {body}");
        assert!(ReactiveSession::from_source(&source).is_err(), "{body}");
    }
}
