//! D083: D061 let a procedure parameter, in a program without a window,
//! ground on what its argument carries, its whole lineage, as releases before
//! rc.12 grounded it (F268), since only such a program can have a save that
//! old. But a program without a window can still use something no release
//! before rc.12 loads, and then it has no such save either. `LATER` sets `g`
//! with `only_if`, which came in 1.4.0, so `g`, and `sc` through `gp`'s
//! parameter, carry `x` without grounding on it; yet a save whose `sc`
//! grounds were its lineage restored, and a commitment using `sc` then
//! rested on `x`: `rests_on_withdrawn` answered yes where the session that
//! was saved answers no.
//!
//! The allowance now holds only in a program a release before rc.12 may have
//! loaded: one with no window that, as it loaded, keeps none of the later
//! constructs `may_load_before_rc12` lists. `older()` sets `g` under a guard
//! instead, a program rc.11 loads, and keeps it. See spec/caveat-save-0.1.md
//! ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn send(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    outcome
}

/// Plays `events` and returns the save, which restores to the same bytes.
fn played(source: &str, events: &[&str]) -> Value {
    let mut session = ReactiveSession::from_source(source).unwrap();
    for event in events {
        send(&mut session, event);
    }
    let save = session.save_json().unwrap();
    let again = ReactiveSession::restore_json(source, &save)
        .unwrap_or_else(|error| panic!("refused: {error}"));
    assert_eq!(again.save_json().unwrap(), save);
    serde_json::from_str(&save).unwrap()
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted {save}"),
    }
}

/// Restores `save`, plays `events` on, and returns the session.
fn plays_on(source: &str, save: &Value, events: &[&str]) -> ReactiveSession {
    let mut session = ReactiveSession::restore_json(source, &save.to_string())
        .unwrap_or_else(|error| panic!("refused: {error}"));
    for event in events {
        send(&mut session, event);
    }
    session
}

fn no_rule_carries(place: &str, history: &str) -> String {
    format!("{place}: no rule or declaration of this program carries records of {history} into it")
}

/// The finding's program, without a window. `go` passes `g` to `gp`, whose
/// parameter sets `sc`; `use` commits `k` using `sc`; `pull` withdraws `x`.
const LATER: &str = "claim c;
evidence x from \"x\";
evidence y from \"y\";
state g = 0;
state sc = 0;
state flag = 0;
decisions k limit 4;
event first;
event go;
event use;
event pull;
event ask;
on first reveal x supports c;
on first reveal y supports c;
on first set g = only_if(qualified(1, x) > 0, 1);
proc gp(a) { set sc = a; };
on go call gp(g);
on use commit k because enough using sc;
on pull withdraw x because y;
on ask when rests_on_withdrawn(k) set flag = 1;
";

/// `LATER` with `g` set under a guard on `x`, as rc.11 can: `g` and `sc`
/// carry `x` again without grounding on it.
fn older() -> String {
    let older = LATER.replace(
        "on first set g = only_if(qualified(1, x) > 0, 1);",
        "on first when qualified(1, x) > 0 set g = 1;",
    );
    assert_ne!(older, LATER);
    older
}

/// `save` with `state`'s grounds set to its lineage, `[x]`, as a release
/// before rc.12 grounded a parameter.
fn wide_state(save: &Value, state: &str) -> Value {
    let mut forged = save.clone();
    assert_eq!(
        forged["states"][state],
        json!({ "value": 1.0, "lineage": { "evidence": ["x"] }, "grounds": {} }),
        "{save}"
    );
    forged["states"][state]["grounds"] = json!({ "evidence": ["x"] });
    forged
}

#[test]
fn a_later_program_refuses_a_parameters_old_grounds() {
    let save = played(LATER, &["first", "go"]);
    assert_eq!(save["states"]["g"]["grounds"], json!({}), "{save}");
    refused(
        LATER,
        &wide_state(&save, "sc"),
        &no_rule_carries("state sc grounds", "x"),
    );
    // A program rc.11 loads keeps the grounds such a release wrote, and a
    // commitment using `sc` rests on `x`, as it did there.
    let older = older();
    let save = played(&older, &["first", "go"]);
    let session = plays_on(&older, &wide_state(&save, "sc"), &["use", "pull", "ask"]);
    let after: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    assert_eq!(after["states"]["flag"]["value"], json!(1.0), "{after}");
    // As saved, `sc` and `k@1` ground on nothing, so `k@1` does not rest
    // on `x`, and `flag` keeps its declared value, which a save leaves out.
    let session = plays_on(&older, &save, &["use", "pull", "ask"]);
    let after: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    assert_eq!(after["states"]["flag"], Value::Null, "{after}");
}

/// `save` with `k@1`'s grounds, and its `committed` entry's `because`, set
/// to `[x]`.
fn wide_commitment(save: &Value) -> Value {
    let mut forged = save.clone();
    assert_eq!(forged["commitment_grounds"]["k@1"], json!({}), "{save}");
    forged["commitment_grounds"]["k@1"] = json!({ "evidence": ["x"] });
    let entry = forged["decision_journal"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["change"] == "committed" && entry["commitment"] == "k@1")
        .unwrap();
    entry["because"] = json!(["x"]);
    forged
}

#[test]
fn a_later_program_refuses_a_commitments_old_grounds_through_a_parameter() {
    let commit = |source: &str| {
        source
            .replace(
                "proc gp(a) { set sc = a; };",
                "proc mk(a) { commit k because enough using a; };",
            )
            .replace("on go call gp(g);", "on go call mk(g);")
    };
    let later = commit(LATER);
    assert!(later.contains("call mk(g)") && later.contains("proc mk(a)"));
    let save = played(&later, &["first", "go"]);
    refused(
        &later,
        &wide_commitment(&save),
        &no_rule_carries("commitment k@1 grounds", "x"),
    );
    let older = commit(&older());
    let save = played(&older, &["first", "go"]);
    plays_on(&older, &wide_commitment(&save), &["pull", "ask"]);
}
