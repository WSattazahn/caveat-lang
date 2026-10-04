//! Restore refuses a caveat's attention that no examination of the loaded
//! source could have left (findings 123, 188–191). See
//! spec/caveat-save-0.1.md and docs/RESTORE_TRUST_BOUNDARY.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// Each way a reactive program examines a caveat: a rule, two rules at
/// different costs, and a procedure specialized for the caveat it is passed.
/// `never` is a caveat nothing examines.
const PROGRAM: &str = r#"
budget 10;
claim safe;
evidence sensor from "sensor";
caveat stale consequence material;
caveat deep consequence low;
caveat looked consequence low;
caveat never consequence low;
state done = 0;
event check;
event probe;
event go;
event look;
proc inspect(c caveat) { examine c cost 2; };
on check examine stale cost 1;
on probe examine deep cost 3;
on go reveal sensor supports safe;
on go examine deep cost 2;
on go when examined(stale) set done = 1;
on look call inspect(looked);
bind hud.done = done;
"#;

fn send(game: &mut ReactiveSession, event: &str) {
    let outcome = serde_json::to_value(
        game.dispatch_outcome_json(event, "{}")
            .unwrap_or_else(|fatal| panic!("{event}: fatal {}", fatal.message)),
    )
    .unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
}

fn played(events: &[&str]) -> ReactiveSession {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    for event in events {
        send(&mut game, event);
    }
    game
}

fn saved(events: &[&str]) -> Value {
    serde_json::to_value(played(events).save().unwrap()).unwrap()
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(game) => panic!(
            "accepted with attention {}",
            serde_json::to_value(game.save().unwrap()).unwrap()["graph"]["attention"]
        ),
    }
}

// A genuine save after no, one and several examinations, by a rule, by the
// cheaper and dearer of two rules and by a procedure, restores as the
// session it was, plays on as it would have, and saves and restores again.
#[test]
fn genuine_saves_after_any_number_of_examinations_restore() {
    for (events, attention, spent) in [
        (&[][..], json!(null), 0),
        (&["check"][..], json!({"stale": "examined"}), 1),
        (
            &["check", "probe", "look", "go", "check"][..],
            json!({"deep": "examined", "looked": "examined", "stale": "examined"}),
            9,
        ),
    ] {
        let mut original = played(events);
        let save = serde_json::to_value(original.save().unwrap()).unwrap();
        assert_eq!(save["graph"]["attention"], attention, "after {events:?}");
        assert_eq!(save["resources"]["spent"], spent, "after {events:?}");
        let mut resumed = ReactiveSession::restore_json(PROGRAM, &save.to_string())
            .unwrap_or_else(|error| panic!("after {events:?}: {error}"));
        assert_eq!(resumed.snapshot(), original.snapshot());
        for event in ["go", "check"] {
            let next = |game: &mut ReactiveSession| {
                serde_json::to_value(game.dispatch_outcome_json(event, "{}").unwrap()).unwrap()
            };
            assert_eq!(next(&mut resumed), next(&mut original), "{event}");
            assert_eq!(resumed.snapshot(), original.snapshot(), "after {event}");
        }
        let again = ReactiveSession::restore_json(PROGRAM, &resumed.save_json().unwrap()).unwrap();
        assert_eq!(again.snapshot(), original.snapshot());
    }
}

// Findings 123, 188–191: a save marked a caveat examined that its budget
// never paid for, and `examined(...)` guards then held as if the
// examination had happened.
#[test]
fn examined_attention_the_budget_never_paid_for_is_refused() {
    // No examination at all, nothing spent.
    let mut save = saved(&[]);
    save["graph"]["attention"] = json!({"stale": "examined"});
    refused(
        PROGRAM,
        &save,
        "examined caveats cost at least 1 attention to examine, more than the 0 the budget spent",
    );
    // One genuine examination, and another caveat marked examined beside it:
    // `deep` costs at least 2 by any rule, so 3 must have been spent.
    let mut save = saved(&["check"]);
    save["graph"]["attention"]["deep"] = "examined".into();
    refused(
        PROGRAM,
        &save,
        "examined caveats cost at least 3 attention to examine, more than the 1 the budget spent",
    );
    // The genuine save's spending lowered under its attention, with the
    // budget's total kept.
    let mut save = saved(&["check", "look"]);
    save["resources"] = json!({"remaining": 8, "spent": 2, "exhausted": false});
    save["effects"] = json!([]);
    refused(
        PROGRAM,
        &save,
        "examined caveats cost at least 3 attention to examine, more than the 2 the budget spent",
    );
}

// An examination of a caveat no rule or procedure examines cannot have
// happened, whatever the budget spent.
#[test]
fn examined_attention_no_rule_could_leave_is_refused() {
    let mut save = saved(&["probe", "probe"]);
    save["graph"]["attention"]["never"] = "examined".into();
    refused(
        PROGRAM,
        &save,
        "caveat never is examined, but no rule or procedure of this program examines it",
    );
}

// `deferred` and `examining` belong to the game session; a reactive event
// only ever leaves a caveat examined.
#[test]
fn attention_only_a_game_session_sets_is_refused() {
    for attention in ["deferred", "examining"] {
        let mut save = saved(&["check"]);
        save["graph"]["attention"]["stale"] = attention.into();
        refused(
            PROGRAM,
            &save,
            &format!("caveat stale is {attention}, which no reactive event leaves"),
        );
    }
}

// Without an attention budget a reactive program cannot examine anything,
// so a save of it holds no examined caveat.
#[test]
fn examined_attention_without_a_budget_is_refused() {
    let source = "claim safe;\nevidence sensor from \"sensor\";\ncaveat stale consequence low;\n\
                  event go;\non go reveal sensor supports safe;\n";
    let mut game = ReactiveSession::from_source(source).unwrap();
    send(&mut game, "go");
    let mut save = serde_json::to_value(game.save().unwrap()).unwrap();
    save["graph"]["attention"] = json!({"stale": "examined"});
    refused(
        source,
        &save,
        "caveat stale is examined, but this program has no attention budget",
    );
}

// The check is of what the source can leave, not of what events did: a save
// whose attention some sequence of this program's events could produce
// restores, though these events did not produce it. Here `probe` examined
// `deep` for 3; `check` then `go` would also have examined `stale` and `deep`
// for 3, so the edited save restores and `examined(stale)` holds after it.
// Hosts that need the account itself to be authentic keep the save text
// intact and trusted (docs/RESTORE_TRUST_BOUNDARY.md).
#[test]
fn attention_the_source_could_leave_restores_without_proof_it_did() {
    let mut save = saved(&["probe"]);
    save["graph"]["attention"]["stale"] = "examined".into();
    let mut restored = ReactiveSession::restore_json(PROGRAM, &save.to_string())
        .unwrap_or_else(|error| panic!("{error}"));
    send(&mut restored, "go");
    assert_eq!(
        serde_json::to_value(restored.view()).unwrap()["bindings"]["hud"]["done"],
        json!(1.0)
    );
    // An attention entry that leaves a caveat as the program loads it, and a
    // genuine examination removed with its examination record, restore too:
    // neither claims an examination. (The record alone left behind is refused
    // since rc.12 PR 3: an examination record belongs to an examined caveat.)
    let mut save = saved(&["check"]);
    save["graph"]["attention"] = json!({"never": "unexamined"});
    save["examination_qualifications"]
        .as_object_mut()
        .unwrap()
        .remove("stale");
    ReactiveSession::restore_json(PROGRAM, &save.to_string())
        .unwrap_or_else(|error| panic!("{error}"));
}
