//! D048: a `set` whose guard was false added the guard's provenance to its
//! target's lineage only when the guard named evidence or caveats the
//! lineage lacked. A guard that held nothing else but a departure marker, a
//! kept caveat, or a read of a name the lineage held only as inherited
//! changed nothing, so the target lost a history it rested on, a caveat a
//! take-back cannot remove, or an own read. Reactive 0.3 adds the whole
//! guard's provenance, and the skipped `sample` and `commit` already did. A
//! skipped `set` now does too, and one whose guard would take the lineage
//! past the provenance limit still refuses the event. See
//! spec/caveat-reactive-0.3.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn send(session: &mut ReactiveSession, event: &str, payload: &str) {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
}

/// The save after `events`, checked to restore.
fn saved(source: &str, events: &[(&str, &str)]) -> Value {
    let mut session = ReactiveSession::from_source(source).unwrap();
    for (event, payload) in events {
        send(&mut session, event, payload);
    }
    let saved = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(source, &saved)
        .unwrap_or_else(|error| panic!("restore refused the runtime's own save: {error}"));
    assert_eq!(restored.save_json().unwrap(), saved);
    serde_json::from_str(&saved).unwrap()
}

/// The departure markers state `b`'s lineage holds: history, read, from and
/// through.
fn markers(save: &Value) -> Vec<(String, u64, u64, u64)> {
    save["states"]["b"]["lineage"]["departed"]
        .as_array()
        .map(|markers| {
            markers
                .iter()
                .map(|marker| {
                    (
                        marker["history"].as_str().unwrap().to_string(),
                        marker["read"].as_u64().unwrap(),
                        marker["from"].as_u64().unwrap(),
                        marker["through"].as_u64().unwrap(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `close` resolves `d` on runs@1, which departs at the next reading; `poke`
/// is skipped, its guard having read the outcome.
const OUTCOME_GUARD: &str = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
readings runs from feed window 1;
decisions d limit 2;
state b = 0;
event go;
event mark;
event reading v min 0 max 9;
event close;
event poke;
on go reveal e supports c;
on go commit d because enough using 1;
on mark set b = qualified(0, e);
on reading sample runs = v;
on close resolve d = latest(runs) because e;
on poke when resolved(d) and outcome(d) > 100 set b = b + 1;
";

#[test]
fn a_skipped_set_keeps_the_marker_of_an_outcome_its_guard_read() {
    let start = [
        ("go", "{}"),
        ("mark", "{}"),
        ("reading", r#"{"v":5}"#),
        ("close", "{}"),
    ];
    // runs@1 departs before `poke`, or after it. Departing after it is a
    // control: the guard read the live record, which the parent added too.
    let before = [&start[..], &[("reading", r#"{"v":2}"#), ("poke", "{}")]].concat();
    let after = [&start[..], &[("poke", "{}"), ("reading", r#"{"v":2}"#)]].concat();
    let runs = vec![("runs".to_string(), 1, 1, 1)];
    assert_eq!(markers(&saved(OUTCOME_GUARD, &after)), runs);
    assert_eq!(markers(&saved(OUTCOME_GUARD, &before)), runs);
}

/// `shadow` is skipped, so `s`'s lineage takes runs@1, which departs at the
/// next reading; `poke` is skipped, its guard having read `s`.
const STATE_GUARD: &str = "claim c;
evidence feed from \"feed\";
readings runs from feed window 1;
state s = 0;
state b = 0;
event reading v min 0 max 9;
event shadow;
event poke;
on reading sample runs = v;
on shadow when latest(runs) > 100 set s = 1;
on poke when s > 100 set b = b + 1;
";

#[test]
fn a_skipped_set_keeps_the_marker_of_a_state_its_guard_read() {
    let start = [("reading", r#"{"v":1}"#), ("shadow", "{}")];
    // As above, departing after `poke` is a control.
    let before = [&start[..], &[("reading", r#"{"v":2}"#), ("poke", "{}")]].concat();
    let after = [&start[..], &[("poke", "{}"), ("reading", r#"{"v":2}"#)]].concat();
    let runs = vec![("runs".to_string(), 1, 1, 1)];
    assert_eq!(markers(&saved(STATE_GUARD, &after)), runs);
    assert_eq!(markers(&saved(STATE_GUARD, &before)), runs);
}

/// `trust` is committed while `witness_pool` is doubted, so its basis keeps
/// the doubt; `poke`'s guard reads the basis, and `undoubt` takes the doubt
/// back from the witness.
const KEPT: &str = r#"
caveat doubted consequence material;
evidence witness_pool from "the slime saw a creature eat it";
state memory = 0;
event see;
event doubt;
event undoubt;
event decide;
event poke;
on see reveal witness_pool;
on see set memory = qualified(1, witness_pool);
on doubt qualify witness_pool with doubted;
on undoubt unqualify witness_pool with doubted;
on decide commit trust because enough using memory;
on poke when committed(trust) and memory > LIMIT set memory = memory + 1;
"#;

#[test]
fn a_skipped_set_keeps_a_caveat_its_guard_read_through_a_record() {
    let events = [
        ("see", "{}"),
        ("doubt", "{}"),
        ("decide", "{}"),
        ("poke", "{}"),
        ("undoubt", "{}"),
    ];
    // Skipped or applied, `memory` rests on the basis and keeps the doubt.
    for limit in ["100", "0"] {
        let save = saved(&KEPT.replace("LIMIT", limit), &events);
        let lineage = &save["states"]["memory"]["lineage"];
        assert_eq!(lineage["caveats"], json!(["doubted"]), "{limit}: {lineage}");
        assert_eq!(lineage["kept"], json!(["doubted"]), "{limit}: {lineage}");
    }
}

/// `lb` is skipped while `latest(a)` is large, so `b`'s selection holds
/// `a`'s current reading and `take` inherits it through `latest(b)`.
/// `probe`'s guard reads that reading itself.
const INHERITING: &str = r#"
claim seen;
evidence glimpse from "a glimpse";
readings a from glimpse window 2;
readings b from glimpse window 2;
state copy = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event take;
event probe;
on la sample a = v supports seen;
on lb when latest(a) < 5 sample b = v supports seen;
on take set copy = latest(b) because latest(b);
on probe when latest(a) > 100 set copy = 0;
"#;

#[test]
fn a_skipped_set_makes_a_name_its_guard_read_an_own_read() {
    let inherit = [
        ("la", r#"{"v":1}"#),
        ("lb", r#"{"v":3}"#),
        ("la", r#"{"v":7}"#),
        ("lb", r#"{"v":4}"#),
        ("take", "{}"),
    ];
    let save = saved(INHERITING, &inherit);
    assert_eq!(
        save["states"]["copy"]["lineage"]["inherited"],
        json!(["a@2"])
    );
    let save = saved(INHERITING, &[&inherit[..], &[("probe", "{}")]].concat());
    let lineage = &save["states"]["copy"]["lineage"];
    assert!(lineage["evidence"]
        .as_array()
        .unwrap()
        .contains(&json!("a@2")));
    assert_eq!(lineage["inherited"], Value::Null, "{lineage}");
}

/// `ga` and `gb` each renew their evidence and add a qualified read of it to
/// `x` and `y`; `poke` is skipped, its guard having read `y`, whose lineage
/// names 600 occurrences of `b`.
const LIMIT_PROGRAM: &str = r#"
claim c;
evidence a from "a source";
evidence b from "b source";
renewable a limit 1024;
renewable b limit 1024;
state x = 0 min 0 max 1000000;
state y = 0 min 0 max 1000000;
event ga;
event gb;
event poke;
on ga renew a;
on ga reveal a supports c;
on ga set x = x + qualified(1, a);
on gb renew b;
on gb reveal b supports c;
on gb set y = y + qualified(1, b);
on poke when y > 1000000 set x = x + 1;
"#;

/// A skipped `set` whose guard would take its target's lineage past the
/// provenance limit refuses the event, as the parent did.
#[test]
fn a_skipped_set_past_the_provenance_limit_is_refused() {
    let mut session = ReactiveSession::from_source(LIMIT_PROGRAM).unwrap();
    for event in ["ga", "gb"] {
        for _ in 1..=600 {
            send(&mut session, event, "{}");
        }
    }
    let before = session.save_json().unwrap();
    let refused =
        serde_json::to_value(session.dispatch_outcome_json("poke", "{}").unwrap()).unwrap();
    assert_eq!(refused["outcome"], "rejected", "{refused}");
    assert_eq!(refused["code"], "provenance_limit", "{refused}");
    assert_eq!(
        refused["message"],
        "event poke, rule 7: value provenance exceeds limit 1024 identifiers"
    );
    assert_eq!(session.save_json().unwrap(), before);
}
