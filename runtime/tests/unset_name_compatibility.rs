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

fn unset_effect_module(effect: &str, root: &str) -> String {
    link::bundle(&[
        BundlePart {
            name: "m".into(),
            source: format!(
                "module m;\n# é; unset ignored\nstate unset = 3;\n\
                 evidence trace from \"中文; unset value {{ text }}\";\n\
                 state value unset min 0 max 10;\n\
                 state seen = 0;\nfn f(x) = x + 1;\n\
                 proc assign(x) {{ set value = x; set seen = f( unset ); }};\n\
                 event report;\nevent clear;\nevent read;\nevent overflow;\n\
                 on report call assign( unset );\n\
                 on read set seen = value;\non overflow set seen = seen + 1;\n\
                 on overflow set value = 11;\n{effect}\n"
            ),
        },
        BundlePart {
            name: "main".into(),
            source: format!("use m;\n{root}"),
        },
    ])
}

#[test]
fn actual_unset_effects_in_procedures_and_rules_keep_their_verb() {
    for final_semicolon in ["", ";"] {
        for guard in ["", "when unset == 3 "] {
            for root in ["", "state unset = 99;\n"] {
                for effect in [
                    format!("on clear {guard}unset value;"),
                    format!(
                        "proc erase() {{ # é; unset ignored\n\
                         {guard}unset // 中文; unset ignored\n\
                         value{final_semicolon}\n}};\non clear call erase();"
                    ),
                ] {
                    let source = unset_effect_module(&effect, root);
                    // The ordinary linker must preserve the effect too; this
                    // does not depend on hold/release recovery in the loader.
                    let linked = link::link(&source).unwrap();
                    let mut flat = ReactiveSession::from_source(&linked).unwrap();
                    send(&mut flat, "m__report");
                    send(&mut flat, "m__clear");
                    assert_eq!(value(&flat, "m__value"), Value::Null);

                    let mut session = ReactiveSession::from_source(&source)
                        .unwrap_or_else(|error| panic!("{source}: {error}"));
                    send(&mut session, "m__report");
                    assert_eq!(value(&session, "m__value"), 3.0);
                    assert_eq!(value(&session, "m__seen"), 4.0);
                    assert_eq!(value(&session, "m__unset"), 3.0);
                    if !root.is_empty() {
                        assert_eq!(value(&session, "unset"), 99.0);
                    }
                    round_trip_and_continue(&source, &mut session, "m__clear");
                    let snapshot = serde_json::to_value(session.snapshot()).unwrap();
                    assert_eq!(snapshot["unset"], serde_json::json!(["m__value"]));
                    assert_eq!(value(&session, "m__value"), Value::Null);
                    round_trip_and_continue(&source, &mut session, "m__report");
                    assert_eq!(value(&session, "m__value"), 3.0);
                }
            }
        }
    }
}

#[test]
fn recovered_unset_procedures_keep_read_refusals_and_bounds_atomic() {
    let source = unset_effect_module(
        "proc erase() { when unset == 3 unset value };\non clear call erase();",
        "state unset = 99;\n",
    );
    let mut session = ReactiveSession::from_source(&source).unwrap();
    for (event, code) in [
        ("m__read", "unset_state"),
        ("m__overflow", "bound_exceeded"),
    ] {
        let before = serde_json::to_value(session.snapshot()).unwrap();
        let outcome = session.dispatch_outcome_json(event, "{}").unwrap();
        let outcome = serde_json::to_value(outcome).unwrap();
        assert_eq!(outcome["outcome"], "rejected", "{outcome}");
        assert_eq!(outcome["code"], code, "{outcome}");
        assert_eq!(serde_json::to_value(session.snapshot()).unwrap(), before);
    }
    send(&mut session, "m__report");
    let before = serde_json::to_value(session.snapshot()).unwrap();
    let outcome = session.dispatch_outcome_json("m__overflow", "{}").unwrap();
    assert_eq!(
        serde_json::to_value(outcome).unwrap()["outcome"],
        "rejected"
    );
    assert_eq!(serde_json::to_value(session.snapshot()).unwrap(), before);
    send(&mut session, "m__clear");
    assert_eq!(value(&session, "m__value"), Value::Null);

    // Rebuild the bundle after changing its part's byte length.
    let parts = link::split_bundle(&source).unwrap();
    let nonoptional = link::bundle(
        &parts
            .into_iter()
            .map(|mut part| {
                part.source = part.source.replace(
                    "state value unset min 0 max 10;",
                    "state value = 0 min 0 max 10;",
                );
                part
            })
            .collect::<Vec<_>>(),
    );
    assert!(ReactiveSession::from_source(&nonoptional)
        .err()
        .unwrap()
        .contains("m__value always has a value"));
}

#[test]
fn a_module_named_unset_keeps_qualified_targets_and_procedure_calls() {
    let source = link::bundle(&[
        BundlePart {
            name: "unset".into(),
            source: "module unset;\nstate unset = 3;\nstate value unset;\nstate direct unset;\n\
                     proc clear() { when unset == 3 unset value };\n\
                     event report;\non report set value = unset;\n"
                .into(),
        },
        BundlePart {
            name: "main".into(),
            source: "use unset;\nstate unset = 99;\nevent clear;\nevent qualified;\n\
                     on unset::report set unset::direct = unset::unset;\n\
                     on clear call unset::clear();\non qualified unset unset::direct;\n"
                .into(),
        },
    ]);
    let mut session = ReactiveSession::from_source(&source).unwrap();
    for (event, target) in [("clear", "unset__value"), ("qualified", "unset__direct")] {
        send(&mut session, "unset__report");
        assert_eq!(value(&session, target), 3.0);
        round_trip_and_continue(&source, &mut session, event);
        assert_eq!(value(&session, target), Value::Null);
        assert_eq!(value(&session, "unset__unset"), 3.0);
        assert_eq!(value(&session, "unset"), 99.0);
    }
}

#[test]
fn an_unset_declaration_after_unicode_does_not_capture_a_root_name() {
    // The old declaration tag used character offsets as byte offsets. With
    // these exact LF bytes it protected `local` in the initializer instead
    // of `unset`, so this unpublished draft program silently initialized
    // result from the root's 11. The module's own value must remain 7.
    let source = link::bundle(&[
        BundlePart {
            name: "m".into(),
            source: format!(
                "module m;\n# {}\nstate local = 7;\nstate result = local;\n\
                 state value unset;\nevent go;\n",
                "é".repeat(19)
            ),
        },
        BundlePart {
            name: "main".into(),
            source: "use m;\ndefine local = 11;\n".into(),
        },
    ]);
    let mut session = ReactiveSession::from_source(&source).unwrap();
    assert_eq!(value(&session, "m__result"), 7.0);
    round_trip_and_continue(&source, &mut session, "m__go");
    assert_eq!(value(&session, "m__result"), 7.0);
}
