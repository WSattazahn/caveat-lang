//! D061: in a program without a window, restore accepted a provenance naming
//! a record that no rule or declaration of the program can carry into it. In
//! `SEEN` the only rule that sets `seen` reads `seen` alone, yet a save whose
//! `seen` lineage cites the observed evidence `e` restored, and later events
//! kept `e` there. D054 bounds what each part of a session holds by what the
//! program's rules and declarations can carry into it, but restore read that
//! bound only in a program with a window, where a record can depart and leave
//! a marker; a program without one holds names all the same, and its grounds
//! and caveats follow them.
//!
//! Restore now reads the bound in every program. One part is wider without a
//! window: a procedure's parameter there grounds on what its argument
//! carries, its whole lineage, as releases before rc.12 grounded it (F268).
//! Only a program without a window can have a save that old, since windows
//! came in rc.15, and `parameter_grounds.rs` restores the rc.11 save that
//! holds such grounds. See spec/caveat-save-0.1.md ("Changes").
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
    let live = ReactiveSession::from_source(source).unwrap();
    plays_from(source, live, events)
}

/// Restores `save` and plays `events` on, as `plays_and_restores` does.
fn plays_on(source: &str, save: &Value, events: &[(&str, &str)]) -> Value {
    let restored = ReactiveSession::restore_json(source, &save.to_string())
        .unwrap_or_else(|error| panic!("refused: {error}"));
    plays_from(source, restored, events)
}

fn plays_from(source: &str, mut live: ReactiveSession, events: &[(&str, &str)]) -> Value {
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

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted {save}"),
    }
}

fn no_rule_carries(place: &str, history: &str) -> String {
    format!("{place}: no rule or declaration of this program carries records of {history} into it")
}

/// The finding's program: `go` counts, reading `seen` alone.
const SEEN: &str = "claim safe;
evidence e from \"e\";
state seen = 0;
event seee;
event go;
on seee reveal e supports safe;
on go set seen = seen + 1;
";

#[test]
fn a_state_cannot_cite_evidence_no_rule_carries_into_it() {
    let save = plays_and_restores(SEEN, &[("seee", "{}"), ("go", "{}")]);
    assert_eq!(save["states"]["seen"], json!({ "value": 1.0 }), "{save}");
    let mut forged = save.clone();
    forged["states"]["seen"]["lineage"] = json!({ "caveats": [], "evidence": ["e"] });
    refused(SEEN, &forged, &no_rule_carries("state seen", "e"));
    // Where a rule carries `e` into `seen`, the runtime itself writes that
    // lineage, and it restores.
    let carries = SEEN.replace("seen + 1;", "seen + qualified(1, e);");
    assert_ne!(carries, SEEN);
    let save = plays_and_restores(&carries, &[("seee", "{}"), ("go", "{}")]);
    assert_eq!(
        save["states"]["seen"]["lineage"]["evidence"],
        json!(["e"]),
        "{save}"
    );
}

/// No window anywhere: `close` resolves `d` because of `e`, and `pull`
/// withdraws `g` because of `f`.
const CAUSES: &str = "claim seen;
evidence e from \"e\";
evidence f from \"f\";
evidence g from \"g\";
decisions d limit 4;
state n = 0;
event see;
event decide;
event close;
event pull;
event count;
on see reveal e supports seen;
on see reveal f supports seen;
on see reveal g supports seen;
on decide commit d because enough using 1;
on close resolve d = 1 because e;
on pull withdraw g because f;
on count set n = n + 1;
";

const CAUSES_PLAYED: [(&str, &str); 5] = [
    ("see", "{}"),
    ("decide", "{}"),
    ("close", "{}"),
    ("pull", "{}"),
    ("count", "{}"),
];

#[test]
fn causes_and_bases_hold_only_what_their_rules_carry() {
    let save = plays_and_restores(CAUSES, &CAUSES_PLAYED);
    assert_eq!(
        save["decision_series"]["d"]["revisions"][0]["outcome"]["because"],
        json!(["e"]),
        "{save}"
    );
    assert_eq!(save["withdrawals"][0]["because"], json!("f"), "{save}");
    // Only a resolve's `because` names what an outcome rests on.
    let mut forged = save.clone();
    forged["decision_series"]["d"]["revisions"][0]["outcome"]["because"] = json!(["e", "f"]);
    refused(
        CAUSES,
        &forged,
        &no_rule_carries("outcome of d@1 because", "f"),
    );
    // A withdrawal's reason is held to the effects of the event that made
    // it first, as before, which no other cause is.
    let mut forged = save.clone();
    forged["withdrawals"][0]["because"] = json!("e");
    refused(
        CAUSES,
        &forged,
        "the withdrawal of g by pull: no withdraw of that event withdraws g because e",
    );
    // `d` commits using a number, so its basis reads no evidence.
    let mut forged = save.clone();
    forged["commitment_bases"]["d@1"]["provenance"]["evidence"] = json!(["g"]);
    refused(CAUSES, &forged, &no_rule_carries("commitment d@1", "g"));
}

/// `see` reveals `w` under a guard on `gate`, which carries `x`, so what
/// reads `w` through its records takes `x` with it: `slot`, set through a
/// procedure's parameter, and `direct`, set inline, both have lineage
/// `[w, x]` and grounds `[w]`.
const PARAMETER: &str = "claim c;
evidence x from \"x\";
evidence w from \"w\";
state gate = 0;
state slot = 0;
state direct = 0;
event first;
event see;
on first reveal x supports c;
on first set gate = qualified(1, x);
on see when gate == 1 reveal w supports c;
proc store(v) { set slot = v; };
on see call store(qualified(7, w));
on see set direct = qualified(7, w);
";

/// `save` with `state`'s grounds left out, so that they are its lineage, as
/// a release before rc.12 wrote a parameter's grounds.
fn grounds_as_lineage(save: &Value, state: &str) -> Value {
    let mut forged = save.clone();
    assert_eq!(
        forged["states"][state]["lineage"]["evidence"],
        json!(["w", "x"]),
        "{save}"
    );
    assert_eq!(
        forged["states"][state]["grounds"]["evidence"],
        json!(["w"]),
        "{save}"
    );
    forged["states"][state]
        .as_object_mut()
        .unwrap()
        .remove("grounds");
    forged
}

#[test]
fn a_parameter_grounds_on_its_arguments_lineage_only_without_a_window() {
    let played = [("first", "{}"), ("see", "{}")];
    let save = plays_and_restores(PARAMETER, &played);
    // The grounds a release before rc.12 gave `slot` restore, and later
    // events ground `slot` as this runtime does.
    let wide = grounds_as_lineage(&save, "slot");
    let restored = ReactiveSession::restore_json(PARAMETER, &wide.to_string())
        .unwrap_or_else(|error| panic!("refused: {error}"));
    let grounds = &restored.snapshot().value_grounds["slot"];
    assert_eq!(
        grounds
            .evidence
            .iter()
            .map(|name| &**name)
            .collect::<Vec<&str>>(),
        ["w", "x"]
    );
    let after = plays_on(PARAMETER, &wide, &[("see", "{}"), ("first", "{}")]);
    assert_eq!(
        after["states"]["slot"]["grounds"]["evidence"],
        json!(["w"]),
        "{after}"
    );
    // No release grounded a state set inline on more than its value.
    refused(
        PARAMETER,
        &grounds_as_lineage(&save, "direct"),
        &no_rule_carries("state direct grounds", "x"),
    );
    // A save of a program with a window is never older than rc.15.
    let windowed = format!("journal window 8;\n{PARAMETER}");
    let save = plays_and_restores(&windowed, &played);
    refused(
        &windowed,
        &grounds_as_lineage(&save, "slot"),
        &no_rule_carries("state slot grounds", "x"),
    );
}
