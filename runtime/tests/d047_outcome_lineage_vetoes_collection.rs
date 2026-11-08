//! D047: withdrawal collection could take a record that only a decision
//! outcome's lineage named. A self-withdrawn occurrence is pinned by its own
//! withdrawal reason, so without collection it stays retired, but rule 4 of
//! Withdrawal Collection 0.1 named no outcome among the holders that keep a
//! record from collection. Once collection took it, `outcome(D)` lost
//! `withdrawn`, and so did a commitment whose basis read the outcome and a
//! state that copied it, though collection promises the values, caveats and
//! decision status a runtime without it gives. An outcome's lineage now
//! keeps a record from collection, as a basis does. See
//! spec/caveat-withdrawal-collection-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// `res` resolves `d` on what the latest `a` says; `wd` withdraws that
/// occurrence because of itself, and the next `cycle` retires it. `check`
/// commits `d2` using the outcome, `react` notes when `d2` rests on a
/// withdrawal, and `copy` copies the outcome.
const PROGRAM: &str = "claim c;
claim seen;
evidence e from \"e\";
evidence a from \"a\";
renewable a window 1;
decisions d limit 4;
decisions d2 limit 4;
state w = 0;
state noticed = 0;
event go;
event cycle;
event res;
event wd;
event noop;
event check;
event react;
event copy;
on go reveal e supports c;
on go commit d because enough using 1;
on cycle renew a;
on cycle reveal a supports seen;
on res resolve d = qualified(0, a) because e;
on wd withdraw a because a;
on check when resolved(d) commit d2 because enough using outcome(d);
on react when rests_on_withdrawn(d2) set noticed = 1;
on copy when resolved(d) set w = outcome(d);
bind hud.o = outcome(d) when resolved(d);
";

const EVENTS: [&str; 9] = [
    "go", "cycle", "res", "wd", "noop", "cycle", "check", "react", "copy",
];

/// Sends `event` and returns the records the archive took.
fn send(session: &mut ReactiveSession, event: &str) -> (Value, Vec<String>) {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    let departed = session
        .drain_archive()
        .into_iter()
        .map(|item| serde_json::to_value(&item).unwrap())
        .filter(|item| item["kind"] != "provenance")
        .map(|item| item["record"].as_str().unwrap().to_string())
        .collect();
    (outcome, departed)
}

fn caveats(provenance: &Value) -> Value {
    provenance["caveats"].clone()
}

#[test]
fn an_outcome_keeps_withdrawn_when_collection_could_take_its_record() {
    let mut live = ReactiveSession::from_source(PROGRAM).unwrap();
    let mut restored: Option<ReactiveSession> = None;
    let mut withdrawn = String::new();
    for (index, event) in EVENTS.into_iter().enumerate() {
        let (answer, departed) = send(&mut live, event);
        let snapshot = serde_json::to_value(live.snapshot()).unwrap();
        if event == "wd" {
            withdrawn = snapshot["withdrawals"][0]["evidence"]
                .as_str()
                .unwrap()
                .to_string();
        }
        if index >= 3 {
            // The withdrawn occurrence stays retired, pinned by its own
            // withdrawal reason, and the outcome keeps `withdrawn`.
            assert!(
                !departed.contains(&withdrawn),
                "{event}: {withdrawn} departed"
            );
            assert_eq!(
                caveats(&snapshot["binding_qualifications"]["hud"]["o"]),
                serde_json::json!(["withdrawn"]),
                "{event}: {}",
                snapshot["binding_qualifications"]["hud"]["o"]
            );
        }
        if index >= 5 {
            assert!(
                snapshot["retired"].get(&withdrawn).is_some(),
                "{event}: {}",
                snapshot["retired"]
            );
        }
        // Every save restores, and a session restored after the withdrawal
        // plays on as the live one does.
        let saved = live.save_json().unwrap();
        if let Some(session) = restored.as_mut() {
            assert_eq!(send(session, event).0, answer, "{event}");
            assert_eq!(session.save_json().unwrap(), saved, "save after {event}");
        }
        let session = ReactiveSession::restore_json(PROGRAM, &saved)
            .unwrap_or_else(|error| panic!("the save after {event} was refused: {error}"));
        if event == "wd" {
            restored = Some(session);
        }
    }
    // A commitment whose basis read the outcome rests on the withdrawal, and
    // a state that copied the outcome carries it.
    let snapshot = serde_json::to_value(live.snapshot()).unwrap();
    assert_eq!(
        caveats(&snapshot["commitment_grounds"]["d2@1"]),
        serde_json::json!(["withdrawn"]),
        "{}",
        snapshot["commitment_grounds"]
    );
    assert_eq!(snapshot["values"]["noticed"], 1.0);
    assert_eq!(
        caveats(&snapshot["qualified_values"]["w"]["provenance"]),
        serde_json::json!(["withdrawn"]),
        "{}",
        snapshot["qualified_values"]["w"]
    );
}

/// A series `d` whose `res` resolves the revision `resolve` names, which the
/// binding `bind` reads, and whose `re` and `go2` make a second revision.
fn past_program(resolve: &str, bind: &str) -> String {
    format!(
        "claim c;
claim seen;
evidence e from \"e\";
evidence a from \"a\";
renewable a window 1;
decisions d limit 4;
event go;
event re;
event go2;
event cycle;
event res;
event wd;
event noop;
on go reveal e supports c;
on go commit d because enough using 1;
on cycle renew a;
on cycle reveal a supports seen;
on res {resolve};
on re reopen d because e;
on go2 commit d because enough using 2;
on wd withdraw a because a;
bind hud.o = {bind};
"
    )
}

/// Plays `events` on `program`: once `wd` withdraws the occurrence the
/// outcome rests on, it never departs and the outcome keeps `withdrawn`, and
/// every save restores.
fn plays_without_losing_withdrawn(program: &str, events: &[&str]) {
    let mut live = ReactiveSession::from_source(program).unwrap();
    let mut withdrawn = String::new();
    for event in events {
        let (_, departed) = send(&mut live, event);
        let snapshot = serde_json::to_value(live.snapshot()).unwrap();
        if *event == "wd" {
            withdrawn = snapshot["withdrawals"][0]["evidence"]
                .as_str()
                .unwrap()
                .to_string();
        }
        if !withdrawn.is_empty() {
            assert!(
                !departed.contains(&withdrawn),
                "{event}: {withdrawn} departed"
            );
            assert_eq!(
                caveats(&snapshot["binding_qualifications"]["hud"]["o"]),
                serde_json::json!(["withdrawn"]),
                "{event}: {}",
                snapshot["binding_qualifications"]["hud"]["o"]
            );
        }
        let saved = live.save_json().unwrap();
        ReactiveSession::restore_json(program, &saved)
            .unwrap_or_else(|error| panic!("the save after {event} was refused: {error}"));
    }
    assert!(!withdrawn.is_empty());
}

/// Every revision's outcome keeps its records, a superseded one's too, since
/// `outcome(revision(d, 1))` still reads it.
#[test]
fn a_superseded_revisions_outcome_keeps_its_record() {
    let program = past_program(
        "resolve revision(d, 1) = qualified(0, a) because e",
        "outcome(revision(d, 1)) when resolved(revision(d, 1))",
    );
    plays_without_losing_withdrawn(
        &program,
        &[
            "go", "cycle", "res", "re", "go2", "wd", "noop", "cycle", "noop",
        ],
    );
}

/// The outcome of a revision after the first keeps its records too.
#[test]
fn a_later_revisions_outcome_keeps_its_record() {
    let program = past_program(
        "resolve d = qualified(0, a) because e",
        "outcome(d) when resolved(d)",
    );
    plays_without_losing_withdrawn(
        &program,
        &[
            "go", "re", "go2", "cycle", "res", "wd", "noop", "cycle", "noop",
        ],
    );
}
