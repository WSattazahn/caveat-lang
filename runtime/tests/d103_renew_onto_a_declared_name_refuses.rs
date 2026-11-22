//! D103: a `renew` that would make a name the program declares ended the
//! session as fatal `unclassified` ("generated occurrence foo@2 already
//! exists"), although the runtime had already rolled the event back. A
//! program may declare evidence, a claim, a caveat or a commitment named
//! `foo@2` beside `renewable foo` (load refuses such a name only beside a
//! reading stream or a decision series), and no renewal makes that name:
//! renewals are made in order, so none gets past it either. Such a `renew`
//! is now refused as `limit/renewal_limit`, as one past the declared limit
//! is, and once `foo` holds the occurrence before the name, so is every later
//! one. The rollback and the diagnostic text are unchanged. See
//! spec/caveat-renewal-0.1.md, spec/caveat-windows-0.1.md and
//! spec/caveat-dispatch-0.1.md ("Changes").
use caveat_runtime::link::{self, BundlePart};
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn answer(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

fn accepted(session: &mut ReactiveSession, event: &str) {
    let outcome = answer(session, event);
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
}

/// `event` is refused as `limit/renewal_limit` with a message ending in
/// `message`, and leaves the save, snapshot and view as they were. The save
/// restores, and the restored session refuses `event` the same way.
fn refused(source: &str, session: &mut ReactiveSession, event: &str, message: &str) {
    let save = session.save_json().unwrap();
    let snapshot = serde_json::to_value(session.snapshot()).unwrap();
    let view = serde_json::to_string(&session.view()).unwrap();
    let outcome = answer(session, event);
    assert_eq!(outcome["outcome"], "rejected", "{event}: {outcome}");
    assert_eq!(outcome["origin"], "limit", "{event}: {outcome}");
    assert_eq!(outcome["code"], "renewal_limit", "{event}: {outcome}");
    assert!(
        outcome["message"].as_str().unwrap().ends_with(message),
        "{event}: {outcome}"
    );
    assert_eq!(session.save_json().unwrap(), save, "{event}");
    assert_eq!(
        serde_json::to_value(session.snapshot()).unwrap(),
        snapshot,
        "{event}"
    );
    assert_eq!(
        serde_json::to_string(&session.view()).unwrap(),
        view,
        "{event}"
    );
    let mut restored = ReactiveSession::restore_json(source, &save).unwrap();
    assert_eq!(answer(&mut restored, event), outcome, "{event}: restored");
    // The view path answers with the same refusal.
    let mut again = ReactiveSession::restore_json(source, &save).unwrap();
    let view_outcome = again
        .dispatch_view_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal on the view path: {}", fatal.message));
    assert_eq!(
        serde_json::to_value(view_outcome).unwrap(),
        outcome,
        "{event}: view path"
    );
}

/// `peek` reveals `foo`, `refresh` renews it and `sy` reads whether it is
/// observed, beside `declared` and `renewal`.
fn program(declared: &str, renewal: &str) -> String {
    format!(
        "claim c;
evidence foo from \"f\";
{declared}
{renewal};
state y = 0;
event peek;
event refresh;
event sy;
on peek reveal foo supports c;
on refresh renew foo;
on sy set y = if(observed(foo), 1, 0);"
    )
}

const TAKEN: &str = "generated occurrence foo@2 already exists";

/// Each kind of node the program may name `foo@2` stops the first renewal,
/// with a limit or a window, and every later one; the session goes on.
#[test]
fn a_renew_onto_a_declared_name_is_refused() {
    for declared in [
        "evidence foo@2 from \"g\";",
        "claim foo@2;",
        "caveat foo@2 consequence low;",
        "commit foo@2 because enough;",
    ] {
        for renewal in ["renewable foo limit 3", "renewable foo window 1"] {
            let source = program(declared, renewal);
            let mut live = ReactiveSession::from_source(&source).unwrap();
            accepted(&mut live, "peek");
            refused(&source, &mut live, "refresh", TAKEN);
            assert_eq!(
                answer(&mut live.clone(), "refresh")["message"],
                "event refresh, rule 2: generated occurrence foo@2 already exists",
                "{declared} {renewal}"
            );
            // `foo` stays current and observed, and nothing retired.
            accepted(&mut live, "sy");
            let snapshot = serde_json::to_value(live.snapshot()).unwrap();
            assert_eq!(
                snapshot["renewals"]["foo"]["occurrences"],
                json!(["foo"]),
                "{declared} {renewal}"
            );
            let save: Value = serde_json::from_str(&live.save_json().unwrap()).unwrap();
            assert_eq!(save["states"]["y"]["value"], 1.0, "{declared} {renewal}");
            assert!(save["renewals"].is_null(), "{declared} {renewal}: {save}");
            assert!(save["retired"].is_null(), "{declared} {renewal}: {save}");
            accepted(&mut live, "peek");
            refused(&source, &mut live, "refresh", TAKEN);
        }
    }
}

/// A declared `foo@4` leaves `foo` the renewals before it, and stops the one
/// that would make it and every later one.
#[test]
fn renewals_stop_at_a_later_declared_name() {
    for renewal in ["renewable foo limit 5", "renewable foo window 2"] {
        let source = program("evidence foo@4 from \"g\";", renewal);
        let mut live = ReactiveSession::from_source(&source).unwrap();
        for event in ["peek", "refresh", "peek", "refresh", "peek"] {
            accepted(&mut live, event);
        }
        let held: Value = serde_json::from_str(&live.save_json().unwrap()).unwrap();
        let held = held["renewals"].clone();
        assert_eq!(held["foo"].as_array().unwrap().last().unwrap(), "foo@3");
        for _ in 0..2 {
            refused(
                &source,
                &mut live,
                "refresh",
                "generated occurrence foo@4 already exists",
            );
        }
        let save: Value = serde_json::from_str(&live.save_json().unwrap()).unwrap();
        assert_eq!(save["renewals"], held, "{renewal}");
    }
}

/// A `renew` past the declared limit is refused for the limit first, as
/// before, whatever name it would make.
#[test]
fn the_limit_is_refused_first() {
    let source = program("evidence foo@2 from \"g\";", "renewable foo limit 1");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    refused(
        &source,
        &mut live,
        "refresh",
        "renewable foo reached its limit 1",
    );
    let source = program("claim foo@3;", "renewable foo limit 2");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    accepted(&mut live, "refresh");
    refused(
        &source,
        &mut live,
        "refresh",
        "renewable foo reached its limit 2",
    );
}

/// A `renew` in a procedure, under a guard, is refused the same way, with the
/// procedure's context in its message, as the fatal error had it.
#[test]
fn a_renew_in_a_procedure_is_refused() {
    let source = "claim c;
evidence foo from \"f\";
claim foo@2;
renewable foo limit 3;
state g = 1;
event refresh;
proc again() { when g > 0 renew foo; };
on refresh call again();";
    let mut live = ReactiveSession::from_source(source).unwrap();
    refused(
        source,
        &mut live,
        "refresh",
        "event refresh, rule 1: procedure again, step 1: generated occurrence foo@2 already exists",
    );
}

/// An event that renews `foo` twice, where the second renewal would make the
/// declared `foo@3`, is refused whole: its first renewal is not kept. So a
/// later event that renews `foo` once makes `foo@2`, and only then is every
/// `renew` of `foo` refused.
#[test]
fn a_refused_event_keeps_no_earlier_renewal() {
    let source = "claim c;
evidence foo from \"f\";
claim foo@3;
renewable foo limit 5;
event twice;
event once;
on twice renew foo;
on twice renew foo;
on once renew foo;";
    let mut live = ReactiveSession::from_source(source).unwrap();
    refused(
        source,
        &mut live,
        "twice",
        "event twice, rule 2: generated occurrence foo@3 already exists",
    );
    accepted(&mut live, "once");
    let snapshot = serde_json::to_value(live.snapshot()).unwrap();
    assert_eq!(
        snapshot["renewals"]["foo"]["occurrences"],
        json!(["foo", "foo@2"])
    );
    refused(
        source,
        &mut live,
        "once",
        "event once, rule 3: generated occurrence foo@3 already exists",
    );
    refused(
        source,
        &mut live,
        "twice",
        "event twice, rule 1: generated occurrence foo@3 already exists",
    );
}

/// In a bundle the linker qualifies a module's `foo@2` when the module
/// declares `foo` too, as here, so the module's `lib__foo@2` stops its
/// `lib__foo`'s renewals.
#[test]
fn a_renew_in_a_bundle_is_refused() {
    let lib = "module lib;
claim c;
evidence foo from \"f\";
evidence foo@2 from \"g\";
renewable foo limit 3;
event peek;
event refresh;
on peek reveal foo supports c;
on refresh renew foo;";
    let main = "use lib;
state x = 0;
event sx;
on sx set x = 1;";
    let parts: Vec<BundlePart> = [("lib", lib), ("main", main)]
        .iter()
        .map(|(name, source)| BundlePart {
            name: name.to_string(),
            source: source.to_string(),
        })
        .collect();
    let source = link::bundle(&parts);
    let mut live = ReactiveSession::from_source(&source).unwrap();
    accepted(&mut live, "lib__peek");
    refused(
        &source,
        &mut live,
        "lib__refresh",
        "generated occurrence lib__foo@2 already exists",
    );
    accepted(&mut live, "sx");
}
