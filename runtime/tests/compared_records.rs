//! What an event changes reaches the bindings that read it, however many
//! records the container holds: a release build compares a container of
//! fewer than 8 records whole, and a larger one by the names the event
//! wrote.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeMap;

/// A caveat the program examines as it loads has no examination record, so
/// examining it in `look`, once `e` is observed, adds one: the only record
/// `look` changes. `extra` adds declarations and rules.
fn examined_after_load(budget: u32, extra: &str) -> ReactiveSession {
    ReactiveSession::from_source(&format!(
        r#"
budget {budget};
claim ok;
evidence e from "tool";
caveat c consequence low;
examine c cost 1;
event see;
event look;
{extra}
on see reveal e supports ok;
on look when observed(e) examine c cost 1;
bind v.x = examined(c);
"#
    ))
    .unwrap()
}

/// The evidence the view's explanation of `v.x` lists.
fn explained(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.view()).unwrap()["binding_explanations"]["v"]["x"]["evidence"]
        .clone()
}

/// Applies `see`, then `look`, and checks that `look`'s examination reaches
/// `v.x`: its explanation lists `e` only after it.
fn look_reaches_the_binding(session: &mut ReactiveSession) {
    let none = BTreeMap::new();
    session.apply("see", &none).unwrap();
    assert_eq!(explained(session), serde_json::json!([]));
    session.apply("look", &none).unwrap();
    assert_eq!(explained(session), serde_json::json!(["e"]));
}

#[test]
fn a_record_added_to_a_few_reaches_the_bindings_that_read_it() {
    look_reaches_the_binding(&mut examined_after_load(3, ""));
}

#[test]
fn a_record_added_to_many_reaches_the_bindings_that_read_it() {
    // Eight caveats examined in `prep` give the examination records eight
    // entries before `look`.
    let others: String = (0..8)
        .map(|n| format!("caveat k{n} consequence low;\non prep examine k{n} cost 1;\n"))
        .collect();
    let mut session = examined_after_load(20, &format!("event prep;\n{others}"));
    session.apply("prep", &BTreeMap::new()).unwrap();
    look_reaches_the_binding(&mut session);
}
