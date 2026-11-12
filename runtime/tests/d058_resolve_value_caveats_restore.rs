//! D058: restore refused the save written after a `resolve` whose value
//! attached a caveat with `qualified(…)`, or read one with `examined(…)`:
//! "outcome of r@1: k3 cannot qualify any of its evidence". An outcome's
//! lineage takes its value's lineage, that of `K` in `revision(D, K)` and
//! that of an occurrence's index. Restore bounds the caveats a saved
//! lineage carries by the expressions of the source that can attach one,
//! and it read those of every effect but `resolve`. It now reads a
//! resolve's too. See spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn send(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    outcome
}

/// Plays `events`. After each, the save restores and saves the same bytes
/// again, and every session restored so far answers the events still to
/// come as the live one does. Returns the last save.
fn plays_and_restores(source: &str, events: &[(&str, &str)]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    for (index, (event, payload)) in events.iter().enumerate() {
        let answer = send(&mut live, event, payload);
        for run in &mut restored {
            assert_eq!(send(run, event, payload), answer, "{index}: {event}");
        }
        let save = live.save_json().unwrap();
        let again = ReactiveSession::restore_json(source, &save)
            .unwrap_or_else(|error| panic!("{index}: {event}: {error}"));
        assert_eq!(again.save_json().unwrap(), save, "{index}: {event}");
        restored.push(again);
    }
    let save = live.save_json().unwrap();
    for run in &restored {
        assert_eq!(run.save_json().unwrap(), save);
    }
    serde_json::from_str(&save).unwrap()
}

/// The save after `events`, none of it restored: for a forgery, which is
/// refused whether or not the save it was made from restores.
fn played(source: &str, events: &[(&str, &str)]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    for (event, payload) in events {
        send(&mut live, event, payload);
    }
    serde_json::from_str(&live.save_json().unwrap()).unwrap()
}

/// `go` observes `e`, `sc` takes a reading of `c`, `cr` commits `r`, and
/// `rs` resolves it with the rule `RESOLVE` stands for. `k4` is declared
/// and nothing attaches it.
const PROGRAM: &str = "claim seen;
evidence e from \"e\";
evidence gc from \"gc\";
caveat k3 consequence low;
caveat k4 consequence low;
readings c from gc limit 4;
decisions r limit 4;
event go v min 0 max 10;
on go reveal e supports seen;
event sc v min 0 max 10;
on sc sample c = v;
event cr v min 0 max 10;
on cr commit r because enough using 1;
event rs v min 0 max 10;
RESOLVE
";

const EVENTS: [(&str, &str); 4] = [
    ("go", r#"{"v":1}"#),
    ("sc", r#"{"v":2}"#),
    ("cr", r#"{"v":1}"#),
    ("rs", r#"{"v":5}"#),
];

/// The value attaches `k3` to `e`.
const BY_VALUE: &str = "on rs resolve r = qualified(v, e, k3) because latest(c);";

fn program(resolve: &str) -> String {
    PROGRAM.replace("RESOLVE", resolve)
}

/// The lineage of the outcome of `r@1` in `save`.
fn lineage(save: &Value) -> &Value {
    &save["decision_series"]["r"]["revisions"][0]["outcome"]["lineage"]
}

fn lineage_mut(save: &mut Value) -> &mut Value {
    &mut save["decision_series"]["r"]["revisions"][0]["outcome"]["lineage"]
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted: {expected}"),
    }
}

#[test]
fn an_outcome_whose_value_attaches_a_caveat_restores() {
    let save = plays_and_restores(&program(BY_VALUE), &EVENTS);
    assert_eq!(
        lineage(&save),
        &json!({"caveats": ["k3"], "evidence": ["c@1", "e"]}),
        "{save}"
    );
}

#[test]
fn an_outcome_whose_revision_index_attaches_a_caveat_restores() {
    let save = plays_and_restores(
        &program("on rs resolve revision(r, qualified(1, e, k3)) = v because latest(c);"),
        &EVENTS,
    );
    assert_eq!(
        lineage(&save),
        &json!({"caveats": ["k3"], "evidence": ["c@1", "e"]}),
        "{save}"
    );
}

#[test]
fn an_outcome_whose_occurrence_index_attaches_a_caveat_restores() {
    let save = plays_and_restores(
        &program(
            "evidence tests from \"t\";
renewable tests limit 8;
on go reveal tests supports seen;
on rs resolve r = v because occurrence(tests, qualified(1, e, k3));",
        ),
        &EVENTS,
    );
    assert_eq!(
        lineage(&save),
        &json!({"caveats": ["k3"], "evidence": ["e", "tests"]}),
        "{save}"
    );
}

/// `examined(k)` carries `k` with no evidence.
#[test]
fn an_outcome_whose_value_reads_an_examined_caveat_restores() {
    let save = plays_and_restores(
        &program(
            "budget 20;
caveat k consequence low;
on go examine k cost 1;
on rs resolve r = if(examined(k), v, 0) because latest(c);",
        ),
        &EVENTS,
    );
    assert_eq!(
        lineage(&save),
        &json!({"caveats": ["k"], "evidence": ["c@1"]}),
        "{save}"
    );
}

#[test]
fn an_outcome_a_procedure_resolves_restores() {
    let save = plays_and_restores(
        &program(
            "proc close(x) {
    resolve r = qualified(x, e, k3) because latest(c);
};
on rs call close(v);",
        ),
        &EVENTS,
    );
    assert_eq!(
        lineage(&save),
        &json!({"caveats": ["k3"], "evidence": ["c@1", "e"]}),
        "{save}"
    );
}

/// A state and a commitment that read the outcome take its lineage, and
/// their saves restore as the outcome's does.
#[test]
fn what_reads_such_an_outcome_restores() {
    let source = program(&format!(
        "{BY_VALUE}
state kept = 0;
decisions next limit 4;
event rd;
on rd set kept = outcome(r);
on rd when not committed(next) commit next because enough using outcome(r);"
    ));
    let mut events = EVENTS.to_vec();
    events.extend([("rd", "{}"), ("sc", r#"{"v":3}"#), ("rd", "{}")]);
    let save = plays_and_restores(&source, &events);
    assert_eq!(
        save["states"]["kept"]["lineage"],
        json!({"caveats": ["k3"], "evidence": ["c@1", "e"]}),
        "{save}"
    );
    assert_eq!(
        save["commitment_bases"]["next@1"]["provenance"],
        json!({"caveats": ["k3"], "evidence": ["c@1", "e"]}),
        "{save}"
    );
}

/// The bound is still a bound: a caveat no expression of the source
/// attaches is refused in an outcome's lineage, with the message it had.
#[test]
fn a_caveat_nothing_attaches_is_still_refused_in_an_outcome() {
    let source = program(BY_VALUE);
    let mut save = played(&source, &EVENTS);
    lineage_mut(&mut save)["caveats"] = json!(["k4"]);
    refused(
        &source,
        &save,
        "outcome of r@1: k4 cannot qualify any of its evidence",
    );
}

/// The value attaches `k3` to `e` alone, so a lineage without `e` cannot
/// carry it.
#[test]
fn a_caveat_of_evidence_the_outcome_does_not_name_is_still_refused() {
    let source = program(BY_VALUE);
    let mut save = played(&source, &EVENTS);
    lineage_mut(&mut save)["evidence"] = json!(["c@1"]);
    refused(
        &source,
        &save,
        "outcome of r@1: k3 cannot qualify any of its evidence",
    );
}

/// Only a `resolve` that a rule reaches counts: `spare` attaches `k3` to
/// `e`, and nothing calls it.
#[test]
fn a_resolve_no_rule_reaches_attaches_nothing() {
    let source = program(
        "proc spare(x) {
    resolve r = qualified(x, e, k3) because latest(c);
};
on rs resolve r = qualified(v, e) because latest(c);",
    );
    let mut save = plays_and_restores(&source, &EVENTS);
    assert_eq!(
        lineage(&save),
        &json!({"caveats": [], "evidence": ["c@1", "e"]}),
        "{save}"
    );
    lineage_mut(&mut save)["caveats"] = json!(["k3"]);
    refused(
        &source,
        &save,
        "outcome of r@1: k3 cannot qualify any of its evidence",
    );
}
