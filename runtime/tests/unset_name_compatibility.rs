//! Names accepted before the unset-state profile keep their meaning,
//! regardless of whitespace around an initializer or inside a call.
use caveat_runtime::link::{self, BundlePart};
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

fn value(session: &ReactiveSession, name: &str) -> Value {
    serde_json::to_value(session.snapshot()).unwrap()["values"][name].clone()
}

fn send(session: &mut ReactiveSession, event: &str) {
    let outcome = session.dispatch_outcome_json(event, "{}").unwrap();
    assert_eq!(
        serde_json::to_value(outcome).unwrap()["outcome"],
        "accepted"
    );
}

fn round_trip_and_continue(source: &str, session: &mut ReactiveSession, event: &str) {
    let mut restored = ReactiveSession::restore(source, &session.save().unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(restored.snapshot()).unwrap(),
        serde_json::to_value(session.snapshot()).unwrap()
    );
    send(session, event);
    send(&mut restored, event);
    assert_eq!(
        serde_json::to_value(restored.snapshot()).unwrap(),
        serde_json::to_value(session.snapshot()).unwrap()
    );
}

#[test]
fn an_initializer_can_read_a_state_named_unset_with_any_assignment_spacing() {
    for assignment in [" = ", "= ", " =", "="] {
        for bounds in ["", " min 0 max 100"] {
            let source = format!(
                "state unset = 3;\nstate value{assignment}unset{bounds};\n\
                 event go;\non go set value = value + unset;\n"
            );
            let mut session = ReactiveSession::from_source(&source)
                .unwrap_or_else(|error| panic!("{source}: {error}"));
            assert_eq!(value(&session, "value"), 3.0, "{source}");
            send(&mut session, "go");
            assert_eq!(value(&session, "value"), 6.0);
            round_trip_and_continue(&source, &mut session, "go");
            assert_eq!(value(&session, "value"), 9.0);
        }
    }
}

#[test]
fn an_initializer_can_call_a_function_named_unset() {
    for assignment in [" = ", "= ", " =", "="] {
        let source = format!(
            "fn unset(a) = a + 3;\nstate value{assignment}unset (1);\n\
             event go;\non go set value = unset(value);\n"
        );
        let mut session = ReactiveSession::from_source(&source)
            .unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(value(&session, "value"), 4.0, "{source}");
        send(&mut session, "go");
        assert_eq!(value(&session, "value"), 7.0);
        round_trip_and_continue(&source, &mut session, "go");
        assert_eq!(value(&session, "value"), 10.0);
    }
}

#[test]
fn a_module_keeps_unset_scoped_in_initializers_and_call_arguments() {
    for assignment in [" = ", "= ", " =", "="] {
        for root_name in ["", "state unset = 99;\n"] {
            let source = link::bundle(&[
                BundlePart {
                    name: "m".into(),
                    source: format!(
                        "module m;\nstate unset = 3;\nstate value{assignment}unset;\n\
                         fn f(a) = a + 1;\nevent go;\non go set value = f( unset );\n"
                    ),
                },
                BundlePart {
                    name: "main".into(),
                    source: format!("use m;\n{root_name}"),
                },
            ]);
            let mut session = ReactiveSession::from_source(&source)
                .unwrap_or_else(|error| panic!("{source}: {error}"));
            assert_eq!(value(&session, "m__value"), 3.0, "{source}");
            send(&mut session, "m__go");
            assert_eq!(value(&session, "m__value"), 4.0, "{source}");
            round_trip_and_continue(&source, &mut session, "m__go");
        }
    }
}
