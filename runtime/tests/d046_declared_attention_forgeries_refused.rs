//! D046: restore accepted four kinds of save that no session of the program
//! leaves, each about what its declarations leave or spend as it loads
//! (D043). A save could mark `unexamined` a caveat the declarations leave
//! examined, examining or deferred, though no event leaves a caveat
//! unexamined; on one they leave examined, the restored session then read
//! `examined(…)` as false. A save could hold an examination record for a
//! caveat no rule or procedure examines, though only an event's examination
//! leaves one. The least attention the budget's `spent` must cover left out
//! what the declarations spent, and a caveat the declarations leave examined
//! even when its examination record says an event examined it again. And a
//! last event's `examine` had only to fit `spent`, not what the budget spent
//! after the declarations. Restore now refuses each, and the saves sessions
//! write restore as before. See spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// `go` reveals the sensor and counts while `stale` is examined; `check`
/// examines `stale` and `probe` examines `deep`, each for 3. No rule
/// examines `aside`.
fn program(declarations: &str) -> String {
    format!(
        "budget 12;\n\
         claim safe;\n\
         evidence sensor from \"sensor\";\n\
         caveat stale consequence material;\n\
         caveat deep consequence low;\n\
         caveat aside consequence low;\n\
         {declarations}\
         state seen = 0;\n\
         event go;\n\
         event check;\n\
         event probe;\n\
         on go reveal sensor supports safe;\n\
         on go when examined(stale) set seen = seen + 1;\n\
         on check examine stale cost 3;\n\
         on probe examine deep cost 3;\n"
    )
}

fn send(session: &mut ReactiveSession, event: &str) {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
}

fn saved(session: &ReactiveSession) -> Value {
    serde_json::from_str(&session.save_json().unwrap()).unwrap()
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted {save}"),
    }
}

/// The session restored from `save`, checked to be the same as `live`.
fn restores(source: &str, live: &ReactiveSession, save: &Value) {
    let restored = ReactiveSession::restore_json(source, &save.to_string())
        .unwrap_or_else(|error| panic!("refused {save}: {error}"));
    assert_eq!(restored.snapshot(), live.snapshot());
}

#[test]
fn a_caveat_the_declarations_attend_to_is_not_saved_unexamined() {
    for (declarations, left) in [
        ("examine stale cost 1;\n", "examined"),
        ("examine stale;\n", "examining"),
        ("defer stale;\n", "deferred"),
    ] {
        let source = program(declarations);
        let live = ReactiveSession::from_source(&source).unwrap();
        restores(&source, &live, &saved(&live));
        let mut save = saved(&live);
        save["graph"]["attention"]["stale"] = "unexamined".into();
        refused(
            &source,
            &save,
            &format!(
                "caveat stale is unexamined, but the declarations leave it {left}, and no \
                 reactive event leaves a caveat unexamined"
            ),
        );
    }
    // A caveat the declarations leave unexamined may still be saved so.
    let source = program("examine stale cost 1;\n");
    let live = ReactiveSession::from_source(&source).unwrap();
    let mut save = saved(&live);
    save["graph"]["attention"]["deep"] = "unexamined".into();
    restores(&source, &live, &save);
}

#[test]
fn an_examination_record_needs_a_rule_or_procedure_that_examines_its_caveat() {
    // The declaration leaves `aside` examined, with no record.
    let source = program("examine aside cost 2;\n");
    let live = ReactiveSession::from_source(&source).unwrap();
    let mut save = saved(&live);
    assert_eq!(save["graph"]["attention"], json!({"aside": "examined"}));
    assert_eq!(save["examination_qualifications"]["aside"], Value::Null);
    save["examination_qualifications"]["aside"] = json!({});
    refused(
        &source,
        &save,
        "examination of aside: no rule or procedure of this program examines aside",
    );
    restores(&source, &live, &saved(&live));
}

#[test]
fn a_record_counts_a_caveat_the_declarations_leave_examined() {
    // The record says an event examined `stale` again, which `check` does
    // for 3, but the budget spent only the declaration's 1.
    let source = program("examine stale cost 1;\n");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    let mut save = saved(&live);
    save["examination_qualifications"]["stale"] = json!({});
    refused(
        &source,
        &save,
        "examined caveats cost at least 3 attention to examine and the declarations spent 1, 4 \
         in all, more than the 1 the budget spent",
    );
    // The save `check` leaves holds that record, and restores.
    send(&mut live, "check");
    let save = saved(&live);
    assert_eq!(save["resources"]["spent"], 4);
    assert_eq!(save["examination_qualifications"], json!({"stale": {}}));
    restores(&source, &live, &save);
}

#[test]
fn the_budget_spent_what_the_declarations_spent_besides() {
    // The declaration spent 2 on `aside`, and `check` examines `stale` for 3
    // more: a save that marks `stale` examined spent at least 5.
    let source = program("examine aside cost 2;\n");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    let mut save = saved(&live);
    assert_eq!(save["resources"]["spent"], 2);
    save["graph"]["attention"]["stale"] = "examined".into();
    save["examination_qualifications"]["stale"] = json!({});
    save["resources"]["spent"] = 3.into();
    save["resources"]["remaining"] = 9.into();
    refused(
        &source,
        &save,
        "examined caveats cost at least 3 attention to examine and the declarations spent 2, 5 \
         in all, more than the 3 the budget spent",
    );
    send(&mut live, "check");
    send(&mut live, "go");
    let save = saved(&live);
    assert_eq!(save["resources"]["spent"], 5);
    restores(&source, &live, &save);
    assert_eq!(live.snapshot().values["seen"], 1.0);
}

#[test]
fn the_last_event_spent_no_more_than_the_budget_spent_after_the_declarations() {
    // Here `probe` examines `stale` for 1, so `stale` costs at least 1, and
    // `check` examines it for 3. The declaration spent 2 on `aside`.
    let source = program("examine aside cost 2;\n").replace(
        "on probe examine deep cost 3;\n",
        "on probe examine stale cost 1;\n",
    );
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "check");
    let save = saved(&live);
    assert_eq!(save["resources"]["spent"], 5);
    restores(&source, &live, &save);
    // A save that says `check` spent 3 of a budget that spent 4, 2 of them
    // the declaration's, is refused, though `stale`'s least cost fits.
    let mut forged = save.clone();
    forged["resources"]["spent"] = 4.into();
    forged["resources"]["remaining"] = 8.into();
    refused(
        &source,
        &forged,
        "examine effect spends more attention than the budget spent after the 2 the \
         declarations spent",
    );
}
