//! D053: restore checked each `examine` the last event reports against the
//! attention budget's `spent` one at a time, never all of them together. One
//! event's examinations are all paid in that event, after what the
//! declarations spent as the program loaded, so a session's `spent` is at
//! least what the declarations spent plus the sum of the last event's
//! examine costs. A save whose `spent` covered each examine alone but not
//! their sum was accepted, though no session of the program writes it, with
//! declarations or without. Restore now refuses it with a message of its
//! own, after every other check, so a save it refused before keeps its
//! message, and the saves sessions write restore as before. The least costs
//! of the caveats examined before the last event are still checked apart
//! from its examines, as the last test pins. See spec/caveat-save-0.1.md
//! ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// `check` examines `a` and `b` for 3 each and `cheap` both for 1, so the
/// least either costs is 1; `one` examines `a` alone for 3, and `twice`
/// examines `a` twice for 2 through a procedure. A guard no event opens
/// keeps `check` from examining `z`.
fn program(declarations: &str) -> String {
    format!(
        "budget 20;\n\
         claim safe;\n\
         caveat a consequence low;\n\
         caveat b consequence low;\n\
         caveat z consequence low;\n\
         {declarations}\
         state gate = 0;\n\
         event check;\n\
         event cheap;\n\
         event one;\n\
         event twice;\n\
         proc look(c caveat) {{ examine c cost 2; }};\n\
         on check examine a cost 3;\n\
         on check examine b cost 3;\n\
         on check when gate > 0 examine z cost 9;\n\
         on cheap examine a cost 1;\n\
         on cheap examine b cost 1;\n\
         on one examine a cost 3;\n\
         on twice call look(a);\n\
         on twice call look(a);\n"
    )
}

fn dispatch(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

fn send(session: &mut ReactiveSession, event: &str) {
    let outcome = dispatch(session, event);
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
}

fn saved(session: &ReactiveSession) -> Value {
    serde_json::from_str(&session.save_json().unwrap()).unwrap()
}

/// `save` with the budget of 20 having spent `spent`.
fn spending(save: &Value, spent: u64) -> Value {
    let mut save = save.clone();
    save["resources"]["spent"] = spent.into();
    save["resources"]["remaining"] = (20 - spent).into();
    save
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted {save}"),
    }
}

/// The session restored from its own save, checked to be the same session.
fn round_trip(source: &str, session: &ReactiveSession) -> ReactiveSession {
    let save = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(source, &save)
        .unwrap_or_else(|error| panic!("refused the runtime's own save {save}: {error}"));
    assert_eq!(restored.save_json().unwrap(), save);
    assert_eq!(restored.snapshot(), session.snapshot());
    restored
}

#[test]
fn the_last_events_examines_fit_spent_together() {
    let source = "budget 20;\n\
                  claim safe;\n\
                  caveat a consequence low;\n\
                  caveat b consequence low;\n\
                  event check;\n\
                  event cheap;\n\
                  on check examine a cost 3;\n\
                  on check examine b cost 3;\n\
                  on cheap examine a cost 1;\n\
                  on cheap examine b cost 1;\n";
    let mut live = ReactiveSession::from_source(source).unwrap();
    send(&mut live, "check");
    let mut save = saved(&live);
    assert!(ReactiveSession::restore_json(source, &save.to_string()).is_ok());
    // Each examine fits the 3 alone, and `a` and `b` cost at least 1 each,
    // but `check` paid 6.
    save["resources"]["spent"] = 3.into();
    save["resources"]["remaining"] = 17.into();
    refused(
        source,
        &save,
        "examine effects spend 6 attention together, more than the 3 the budget spent",
    );
}

#[test]
fn each_examine_the_last_event_reports_counts() {
    let source = program("");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "check");
    let save = saved(&live);
    assert_eq!(save["resources"]["spent"], 6);
    assert_eq!(
        save["effects"],
        json!([
            {"kind": "examine", "caveat": "a", "cost": 3},
            {"kind": "examine", "caveat": "b", "cost": 3},
        ])
    );
    // Spent 6 restores, 5 does not.
    round_trip(&source, &live);
    refused(
        &source,
        &spending(&save, 5),
        "examine effects spend 6 attention together, more than the 5 the budget spent",
    );
    // A caveat examined twice in one event is paid for twice.
    send(&mut live, "twice");
    let save = saved(&live);
    assert_eq!(save["resources"]["spent"], 10);
    assert_eq!(
        save["effects"],
        json!([
            {"kind": "examine", "caveat": "a", "cost": 2},
            {"kind": "examine", "caveat": "a", "cost": 2},
        ])
    );
    round_trip(&source, &live);
    refused(
        &source,
        &spending(&save, 3),
        "examine effects spend 4 attention together, more than the 3 the budget spent",
    );
}

#[test]
fn the_last_events_examines_fit_what_the_budget_spent_after_the_declarations() {
    // The declaration spent 2 on `z` as the program loaded, and `check` 6
    // more. A save that says the budget spent 5 leaves `check` 3: each of
    // its examines fits that alone, and the least costs, 1 for each of `a`
    // and `b` besides the declaration's 2, fit the 5.
    let source = program("examine z cost 2;\n");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "check");
    let save = saved(&live);
    assert_eq!(save["resources"]["spent"], 8);
    round_trip(&source, &live);
    for spent in [5, 7] {
        refused(
            &source,
            &spending(&save, spent),
            &format!(
                "examine effects spend 6 attention together and the declarations spent 2, 8 \
                 in all, more than the {spent} the budget spent"
            ),
        );
    }
}

#[test]
fn the_last_events_examines_count_a_caveat_the_declarations_examined() {
    // The declaration examined `a` for 2 as the program loaded, and `check`
    // examines it again for 3, paying in full as for any other caveat. A
    // save that says the budget spent 5 or 7 leaves `check` less than the 6
    // it paid, though each of its examines fits that alone and the least
    // costs, 1 for each of `a` and `b`, fit it besides the declaration's 2.
    let source = program("examine a cost 2;\n");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "check");
    let save = saved(&live);
    assert_eq!(save["resources"]["spent"], 8);
    round_trip(&source, &live);
    for spent in [5, 7] {
        refused(
            &source,
            &spending(&save, spent),
            &format!(
                "examine effects spend 6 attention together and the declarations spent 2, 8 \
                 in all, more than the {spent} the budget spent"
            ),
        );
    }
}

#[test]
fn an_examine_whose_examination_the_save_dropped_still_counts() {
    // The last event paid for each examine it reports, whatever the save
    // says of the caveat it examined. With the declaration that examined
    // `a` for 2, a save after `check` that drops the examination record
    // `check` left for `a` shows `a` examined by the declaration alone,
    // which needs no record and which the least costs leave out. Without
    // the declaration, a save that drops `a`'s examination together with
    // its record shows `a` unexamined. Each save says the budget spent 3
    // less than it did: each of `check`'s examines fits that alone, and so
    // do the least costs, but the 6 `check` paid does not.
    let source = program("examine a cost 2;\n");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "check");
    let mut save = spending(&saved(&live), 5);
    save["examination_qualifications"]
        .as_object_mut()
        .unwrap()
        .remove("a")
        .unwrap();
    refused(
        &source,
        &save,
        "examine effects spend 6 attention together and the declarations spent 2, 8 in all, \
         more than the 5 the budget spent",
    );
    let source = program("");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "check");
    let mut save = spending(&saved(&live), 3);
    save["graph"]["attention"]
        .as_object_mut()
        .unwrap()
        .remove("a")
        .unwrap();
    save["examination_qualifications"]
        .as_object_mut()
        .unwrap()
        .remove("a")
        .unwrap();
    refused(
        &source,
        &save,
        "examine effects spend 6 attention together, more than the 3 the budget spent",
    );
}

#[test]
fn examine_costs_past_u64_max_together_are_refused() {
    // `check` spends the whole budget, 2^64 - 1, in two examines. A third
    // examine of `a` added to the save fits `spent` alone, and the three
    // come to more than any `u64` holds: they are summed without wrapping
    // or saturating.
    let source = "budget 18446744073709551615;\n\
                  claim safe;\n\
                  caveat a consequence low;\n\
                  caveat b consequence low;\n\
                  event check;\n\
                  event cheap;\n\
                  on check examine a cost 9223372036854775808;\n\
                  on check examine b cost 9223372036854775807;\n\
                  on cheap examine a cost 1;\n\
                  on cheap examine b cost 1;\n";
    let mut live = ReactiveSession::from_source(source).unwrap();
    send(&mut live, "check");
    let save = saved(&live);
    assert_eq!(save["resources"]["spent"], u64::MAX);
    round_trip(source, &live);
    let mut forged = save.clone();
    forged["effects"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind": "examine", "caveat": "a", "cost": 1u64 << 63}));
    refused(
        source,
        &forged,
        "examine effects spend 27670116110564327423 attention together, more than the \
         18446744073709551615 the budget spent",
    );
}

#[test]
fn a_save_the_bindings_refuse_keeps_that_message() {
    // The bindings are evaluated after every other check but this one: a
    // save whose state makes a binding divide by zero is refused for that,
    // though its budget's `spent` is short of `check`'s examines too.
    let source = "budget 20;\n\
                  claim safe;\n\
                  caveat a consequence low;\n\
                  caveat b consequence low;\n\
                  state n = 1;\n\
                  event check;\n\
                  event cheap;\n\
                  on check examine a cost 3;\n\
                  on check examine b cost 3;\n\
                  on cheap examine a cost 1;\n\
                  on cheap examine b cost 1;\n\
                  bind hud.x = 10 / n;\n";
    let mut live = ReactiveSession::from_source(source).unwrap();
    send(&mut live, "check");
    round_trip(source, &live);
    let mut save = spending(&saved(&live), 3);
    refused(
        source,
        &save,
        "examine effects spend 6 attention together, more than the 3 the budget spent",
    );
    save["states"]["n"]["value"] = 0.into();
    refused(
        source,
        &save,
        "binding hud.x: division by zero in expression",
    );
}

#[test]
fn saves_sessions_write_restore_and_play_on_alike() {
    // Two rules examine two caveats in `check`, a procedure called twice
    // examines `a` twice in `twice`, and the guard skips `check`'s examine of
    // `z`. The third `check` is refused for want of attention: with nothing
    // declared it has 4 left, so its first examine pays 3 and its second
    // finds 1, and with the declaration it has 2, short of the first. The
    // second `check` stays the last event. `one`, at 3, is refused too:
    // after `cheap` the budget has 2 left, or none with the declaration.
    // Each save restores as the session that wrote it, which then answers
    // the next event as that session does.
    let played = ["check", "twice", "check", "check", "cheap", "one"];
    for (declarations, spent) in [
        ("", [6, 10, 16, 16, 18, 18]),
        ("examine z cost 2;\n", [8, 12, 18, 18, 20, 20]),
    ] {
        let source = program(declarations);
        let mut live = ReactiveSession::from_source(&source).unwrap();
        let mut restored = round_trip(&source, &live);
        let mut rejected = Vec::new();
        for (event, spent) in played.into_iter().zip(spent) {
            let answer = dispatch(&mut live, event);
            assert_eq!(
                dispatch(&mut restored, event),
                answer,
                "{declarations}{event}"
            );
            if answer["outcome"] != "accepted" {
                assert_eq!(answer["code"], "attention_limit", "{declarations}{event}");
                rejected.push(event);
            }
            assert_eq!(
                saved(&live)["resources"]["spent"],
                spent,
                "{declarations}{event}"
            );
            restored = round_trip(&source, &live);
        }
        assert_eq!(rejected, ["check", "one"], "{declarations}");
        assert_eq!(saved(&live)["last_event"], "cheap", "{declarations}");
    }
}

#[test]
fn an_examine_that_does_not_fit_alone_keeps_its_message() {
    // `one` examines `a` alone for 3, and `a` costs at least 1.
    let source = program("");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "one");
    let save = saved(&live);
    assert_eq!(save["resources"]["spent"], 3);
    round_trip(&source, &live);
    refused(
        &source,
        &spending(&save, 2),
        "examine effect spends more attention than the budget spent",
    );
    // `check`'s two examines, either of which spends more than all the
    // budget spent: the message for one examine still.
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "check");
    let checked = saved(&live);
    refused(
        &source,
        &spending(&checked, 2),
        "examine effect spends more attention than the budget spent",
    );
    // A save refused for something else keeps that message, though the
    // budget's `spent` is short of `check`'s examines too.
    let mut save = spending(&checked, 3);
    save["examination_qualifications"]
        .as_object_mut()
        .unwrap()
        .remove("a");
    refused(
        &source,
        &save,
        "examined caveat a has no examination record",
    );
    // With the declaration's 2 spent besides.
    let source = program("examine z cost 2;\n");
    for event in ["one", "check"] {
        let mut live = ReactiveSession::from_source(&source).unwrap();
        send(&mut live, event);
        refused(
            &source,
            &spending(&saved(&live), 4),
            "examine effect spends more attention than the budget spent after the 2 the \
             declarations spent",
        );
    }
}

#[test]
fn the_caveats_examined_before_the_last_event_are_counted_apart() {
    // The check adds the last event's examines to what the declarations
    // spent, not to the least costs of the caveats examined before it, which
    // the least-cost check counts on their own. `look` paid 5 for `c` and
    // `check` then 6 for `a` and `b`, 11 in all. A save that says the budget
    // spent 7 restores, though no session writes it: `check`'s examines fit
    // the 7, and so do the least costs, 1 for each of `a` and `b` and 5 for
    // `c`. Below 7 the least costs no longer fit.
    let source = "budget 20;\n\
                  claim safe;\n\
                  caveat a consequence low;\n\
                  caveat b consequence low;\n\
                  caveat c consequence low;\n\
                  event check;\n\
                  event cheap;\n\
                  event look;\n\
                  on check examine a cost 3;\n\
                  on check examine b cost 3;\n\
                  on cheap examine a cost 1;\n\
                  on cheap examine b cost 1;\n\
                  on look examine c cost 5;\n";
    let mut live = ReactiveSession::from_source(source).unwrap();
    send(&mut live, "look");
    send(&mut live, "check");
    let save = saved(&live);
    assert_eq!(save["resources"]["spent"], 11);
    round_trip(source, &live);
    let save = spending(&save, 7);
    ReactiveSession::restore_json(source, &save.to_string())
        .unwrap_or_else(|error| panic!("refused {save}: {error}"));
    refused(
        source,
        &spending(&save, 6),
        "examined caveats cost at least 7 attention to examine, more than the 6 the budget spent",
    );
}
