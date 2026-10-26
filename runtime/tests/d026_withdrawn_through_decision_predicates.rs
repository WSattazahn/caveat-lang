//! D026: a value read through `committed(D)`, `reopened(D)` or
//! `in_basis(D, E)` lists the evidence in the decision's basis, and so
//! carries `withdrawn` once that evidence is withdrawn, as a value read
//! through `latest(D)` already did and as a late `qualify` already reached it. Found by the checker of the
//! "Try to break this" page. See spec/caveat-withdrawal-0.1.md ("What
//! `withdraw` does", step 2) and spec/caveat-late-qualification-0.1.md (D015).
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const SOURCE: &str = r#"
claim ok;
caveat stale consequence material;
evidence a from "what the decision used";
evidence other from "something the decision never used";
evidence recheck from "a recheck";
decisions d limit 4;
state after = 0;
state held = 0;
event go;
event pull;
event pull_other;
event age;
event read;
event reopen_it;
event ask;
on go reveal a supports ok;
on go reveal other supports ok;
on go commit d because enough using qualified(1, a);
on pull reveal recheck supports ok;
on pull withdraw a because recheck;
on pull_other reveal recheck supports ok;
on pull_other withdraw other because recheck;
on age qualify a with stale;
on read set after = if(committed(d), 1, 0);
on reopen_it reveal recheck supports ok;
on reopen_it reopen d because recheck;
on ask set held = if(in_basis(d, a), 1, 0);
bind x.t = "none";
bind x.t = "in force" when committed(d);
bind x.r = "not reopened";
bind x.r = "reopened" when reopened(d);
bind x.v = latest(d) when committed(d);
bind x.b = "a in basis" when in_basis(d, a);
"#;

fn run(events: &[&str]) -> Value {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    for event in events {
        let outcome = session
            .dispatch_outcome_json(event, "{}")
            .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
        let outcome: Value = serde_json::to_value(outcome).unwrap();
        assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    }
    serde_json::to_value(session.snapshot()).unwrap()
}

fn names(provenance: &Value, key: &str) -> Vec<String> {
    provenance[key]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|item| item.as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn shown(snapshot: &Value, field: &str) -> Value {
    snapshot["binding_qualifications"]["x"][field].clone()
}

fn carries_withdrawn(provenance: &Value) -> bool {
    names(provenance, "caveats")
        .iter()
        .any(|c| c == "withdrawn")
}

#[test]
fn a_display_read_through_committed_carries_the_withdrawal() {
    let before = run(&["go"]);
    assert!(!carries_withdrawn(&shown(&before, "t")));
    let after = run(&["go", "pull"]);
    let t = shown(&after, "t");
    assert!(names(&t, "evidence").contains(&"a".to_string()), "{t}");
    assert!(carries_withdrawn(&t), "x.t lists withdrawn a: {t}");
    // As `latest(d)` already did.
    assert!(carries_withdrawn(&shown(&after, "v")));
}

#[test]
fn a_display_read_through_reopened_carries_the_withdrawal() {
    let after = run(&["go", "reopen_it", "pull"]);
    assert_eq!(after["bindings"]["x"]["r"], "reopened");
    assert!(
        carries_withdrawn(&shown(&after, "r")),
        "{}",
        shown(&after, "r")
    );
}

#[test]
fn a_state_set_through_committed_after_the_withdrawal_carries_it() {
    let after = run(&["go", "pull", "read"]);
    let state = &after["qualified_values"]["after"]["provenance"];
    assert!(carries_withdrawn(state), "{state}");
    // Set before the withdrawal, it gained the caveat when the withdrawal
    // ran; set after, it now reads the same.
    let earlier = run(&["go", "read", "pull"]);
    assert_eq!(
        &earlier["qualified_values"]["after"],
        &after["qualified_values"]["after"]
    );
}

#[test]
fn withdrawing_evidence_the_decision_never_used_reaches_nothing_through_it() {
    let after = run(&["go", "pull_other"]);
    assert!(
        !carries_withdrawn(&shown(&after, "t")),
        "{}",
        shown(&after, "t")
    );
    assert!(!carries_withdrawn(&shown(&after, "v")));
}

#[test]
fn the_decision_keeps_what_it_was_made_on() {
    let before = run(&["go"]);
    let after = run(&["go", "pull", "read"]);
    for key in ["commitment_bases", "commitment_grounds"] {
        assert_eq!(before[key], after[key], "{key}");
    }
}

#[test]
fn a_late_qualify_already_reached_through_committed() {
    let after = run(&["go", "age"]);
    let t = shown(&after, "t");
    assert!(names(&t, "caveats").contains(&"stale".to_string()), "{t}");
    assert!(!carries_withdrawn(&t));
}

#[test]
fn a_value_read_through_in_basis_carries_the_withdrawal() {
    let before = run(&["go", "ask"]);
    assert_eq!(before["bindings"]["x"]["b"], "a in basis");
    assert!(!carries_withdrawn(&shown(&before, "b")));
    let after = run(&["go", "pull", "ask"]);
    // Grounds never change after the decision: a is still in its basis.
    assert_eq!(after["bindings"]["x"]["b"], "a in basis");
    assert!(
        carries_withdrawn(&shown(&after, "b")),
        "{}",
        shown(&after, "b")
    );
    let state = &after["qualified_values"]["held"]["provenance"];
    assert!(carries_withdrawn(state), "{state}");
    let earlier = run(&["go", "ask", "pull"]);
    assert_eq!(
        &earlier["qualified_values"]["held"],
        &after["qualified_values"]["held"]
    );
}
