//! D088: a module's `hold`/`release` reference two whitespace words before
//! `because` was mistaken for an effect verb. Fresh loads use module scope;
//! an ambiguous historical save needs an explicit interpretation.
use caveat_runtime::link::{self, BundlePart};
use caveat_runtime::reactive::{interface_source, ModuleBindings, ReactiveSession, RestoreOptions};
use caveat_runtime::web::WebReactiveSession;
use serde_json::{json, Value};

fn bundle(module: &str, root: &str) -> String {
    link::bundle(&[
        BundlePart {
            name: "m".into(),
            source: module.into(),
        },
        BundlePart {
            name: "main".into(),
            source: root.into(),
        },
    ])
}

fn snapshot(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.snapshot()).unwrap()
}

fn send(session: &mut ReactiveSession, event: &str, payload: &str) {
    let outcome = session.dispatch_outcome_json(event, payload).unwrap();
    assert_eq!(
        serde_json::to_value(outcome).unwrap()["outcome"],
        "accepted"
    );
}

fn assert_scoped_save(session: &ReactiveSession, historical: &str) {
    let mut saved: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    assert_eq!(
        saved.as_object_mut().unwrap().remove("module_bindings"),
        Some(json!("scoped"))
    );
    assert_eq!(saved, serde_json::from_str::<Value>(historical).unwrap());
}

fn numeric_module(word: &str, expression: &str) -> String {
    format!(
        "module m;\nstate {word} = 7;\nstate v = 0;\nfn add(a,b) = a+b;\n\
         event go;\non go set v = {expression};\n"
    )
}

// Written by a native build of v1.0.0, e2a13500, Rust 1.98.1, from the exact
// bundles assembled below, after one m__go. The next event produced 20.
const HOLD_SAVE_100: &str = r#"{"schema":"caveat-reactive-save/0.1","source_id":"fnv1a64:720e76e5fd6168d0:182","source_sha256":"a0b7db01fa116cea31f6e08acd23f0a37cea46f2a7c33584c930bf4edd63134f","sequence":1,"last_event":"m__go","elapsed":0.0,"states":{"m__v":{"value":10.0}},"graph":{}}"#;
const RELEASE_SAVE_100: &str = r#"{"schema":"caveat-reactive-save/0.1","source_id":"fnv1a64:31878c710bde602f:188","source_sha256":"8580e61db2867a79230bf375ce7bdb52335eb02a8a36d8b41248d3c4d4c99779","sequence":1,"last_event":"m__go","elapsed":0.0,"states":{"m__v":{"value":10.0}},"graph":{}}"#;

#[test]
fn native_and_web_interfaces_describe_the_recovered_bundle() {
    let source = bundle(
        &numeric_module("hold", "add( hold , because )"),
        "use m;\nstate because = 3;\n",
    );
    let native = serde_json::to_value(interface_source(&source).unwrap()).unwrap();
    let web: Value =
        serde_json::from_str(&WebReactiveSession::interface(&source).unwrap()).unwrap();
    assert_eq!(native, web);
    assert_eq!(
        native["events"],
        json!([{"name": "m__go", "parameters": []}])
    );
    assert!(native["states"]
        .as_array()
        .unwrap()
        .iter()
        .any(|state| state["name"] == "m__hold"));
}

#[test]
fn refused_bundles_load_and_genuine_100_saves_continue() {
    for (word, saved) in [("hold", HOLD_SAVE_100), ("release", RELEASE_SAVE_100)] {
        let source = bundle(
            &numeric_module(word, &format!("v + add( {word} , because )")),
            "use m;\nstate because = 3;\n",
        );
        // The raw linker now fixes the same binding, so flattening retains
        // the correct module meaning too.
        let ordinary = link::link(&source).unwrap();
        let mut flat = ReactiveSession::from_source(&ordinary).unwrap();
        let mut live = ReactiveSession::from_source(&source).unwrap();
        send(&mut flat, "m__go", "{}");
        send(&mut live, "m__go", "{}");
        assert_eq!(snapshot(&flat)["values"], snapshot(&live)["values"]);
        assert_scoped_save(&live, saved);

        let mut restored = ReactiveSession::restore_json(&source, saved).unwrap();
        assert_eq!(snapshot(&restored), snapshot(&live));
        send(&mut live, "m__go", "{}");
        send(&mut restored, "m__go", "{}");
        assert_eq!(snapshot(&restored)["values"]["m__v"], 20.0);
        assert_eq!(snapshot(&restored), snapshot(&live));

        let changed = source.replacen(" = 7;", " = 8;", 1);
        assert!(ReactiveSession::restore_json(&changed, saved)
            .err()
            .unwrap()
            .contains("different program"));
    }
}

fn call_module(word: &str, procedure: bool) -> String {
    let (declaration, effect) = if procedure {
        (
            format!("proc {word}(x) {{ set v = v + x + 1; }};"),
            format!("call {word} ( because )"),
        )
    } else {
        (
            format!("fn {word}(x) = x + 1;"),
            format!("set v = v + {word} ( because )"),
        )
    };
    format!("module m;\n{declaration}\nstate v = 0;\nevent go;\non go {effect};\n")
}

// Written by v1.0.0 (e2a13500), built with Rust 1.98.1, after one m__go
// on the exact bundles below. Live and restored continuations both gave 8.
// v1.4.1 refuses all three: the spaced call name is two words before because.
const FN_HOLD_SAVE_100: &str = r#"{"schema":"caveat-reactive-save/0.1","source_id":"fnv1a64:34eec34adb806d20:161","source_sha256":"bb51c7739f5d74aeb0d7b496b248dc26684acfdcdb3020d8e1ccc070b8260ef3","sequence":1,"last_event":"m__go","elapsed":0.0,"states":{"m__v":{"value":4.0}},"graph":{}}"#;
const FN_RELEASE_SAVE_100: &str = r#"{"schema":"caveat-reactive-save/0.1","source_id":"fnv1a64:9042bf268640e867:167","source_sha256":"4c148c7f6f737020280e02ce1dd2a910842f418cdbff350b7d7658bd20f0942e","sequence":1,"last_event":"m__go","elapsed":0.0,"states":{"m__v":{"value":4.0}},"graph":{}}"#;
const PROC_HOLD_SAVE_100: &str = r#"{"schema":"caveat-reactive-save/0.1","source_id":"fnv1a64:a4b240271664296d:171","source_sha256":"2092a3daba2f877842bc44a9d59d94f5626d1e13ca1d1dfed312269db8223dbb","sequence":1,"last_event":"m__go","elapsed":0.0,"states":{"m__v":{"value":4.0}},"graph":{}}"#;

#[test]
fn spaced_function_and_procedure_calls_restore_genuine_100_saves() {
    for (word, procedure, saved) in [
        ("hold", false, FN_HOLD_SAVE_100),
        ("release", false, FN_RELEASE_SAVE_100),
        ("hold", true, PROC_HOLD_SAVE_100),
    ] {
        let source = bundle(
            &call_module(word, procedure),
            "use m;\nstate because = 3;\n",
        );
        assert!(ReactiveSession::from_source(&link::link(&source).unwrap()).is_ok());
        let mut live = ReactiveSession::from_source(&source).unwrap();
        send(&mut live, "m__go", "{}");
        assert_eq!(snapshot(&live)["values"]["m__v"], 4.0);
        assert_scoped_save(&live, saved);

        let mut restored = ReactiveSession::restore_json(&source, saved).unwrap();
        assert_eq!(snapshot(&restored), snapshot(&live));
        send(&mut live, "m__go", "{}");
        send(&mut restored, "m__go", "{}");
        assert_eq!(snapshot(&restored)["values"]["m__v"], 8.0);
        assert_eq!(snapshot(&restored), snapshot(&live));
    }
}

#[test]
fn module_function_and_procedure_calls_do_not_capture_root_names() {
    for (word, procedure) in [("hold", false), ("release", false), ("hold", true)] {
        let root = if procedure {
            format!(
                "use m;\nstate because = 3;\nstate v = 0;\n\
                 proc {word}(x) {{ set v = v + x + 10; }};\n"
            )
        } else {
            format!("use m;\nstate because = 3;\nfn {word}(x) = x + 10;\n")
        };
        let source = bundle(&call_module(word, procedure), &root);
        let ordinary = link::link(&source).unwrap();
        let mut flat = ReactiveSession::from_source(&ordinary).unwrap();
        let mut live = ReactiveSession::from_source(&source).unwrap();
        send(&mut flat, "m__go", "{}");
        send(&mut live, "m__go", "{}");
        assert_eq!(snapshot(&live)["values"], snapshot(&flat)["values"]);
        assert_eq!(snapshot(&live)["values"]["m__v"], 4.0);
        if procedure {
            assert_eq!(snapshot(&live)["values"]["v"], 0.0);
        }
        let mut restored =
            ReactiveSession::restore_json(&source, &live.save_json().unwrap()).unwrap();
        send(&mut live, "m__go", "{}");
        send(&mut restored, "m__go", "{}");
        assert_eq!(snapshot(&restored)["values"]["m__v"], 8.0);
        assert_eq!(snapshot(&restored), snapshot(&live));
    }
}

#[test]
fn a_guard_reference_is_not_a_hold_or_release_effect() {
    for word in ["hold", "release"] {
        let source = bundle(
            &format!(
                "module m;\nstate {word} = 7;\nstate v = 0;\nevent go;\n\
                 on go when {word} == because set v = v + 1;\n"
            ),
            "use m;\nstate because = 7;\n",
        );
        let mut session = ReactiveSession::from_source(&source).unwrap();
        send(&mut session, "m__go", "{}");
        assert_eq!(snapshot(&session)["values"]["m__v"], 1.0);
    }
}

#[test]
fn procedure_steps_comments_and_last_statements_use_the_rule_parser() {
    for procedures in [false, true] {
        let effects = if procedures {
            // A last procedure step needs no semicolon. The first word
            // `hold`/`release` is in the guard; the second is the effect.
            "proc pause() { when hold > because hold gate because team };\n\
             proc resume() { when release > because release gate because team };\n\
             on go call pause();\non thaw call resume()"
        } else {
            "on go when hold > because hold gate because team;\n\
             on thaw when release > because release gate because team"
        };
        let source = bundle(
            &format!(
                "module m;\nstate hold = 7;\nstate release = 8;\nstate v = 0;\n\
                 fn add(a,b) = a+b;\n\
                 evidence team from \"é; hold gate because text\";\n\
                 decisions gate limit 3;\nevent go;\nevent thaw;\n\
                 on go set v = add( hold # a comment touching the reference\n , because );\n\
                 on go when false reject \"release ; because\";\n\
                 on go reveal team;\non go commit gate because enough using 1;\n\
                 bind hud.frozen = on_hold(gate);\n{effects}"
            ),
            "use m;\nstate because = 3;\n",
        );
        let mut session = ReactiveSession::from_source(&source)
            .unwrap_or_else(|error| panic!("procedures={procedures}: {error}"));
        send(&mut session, "m__go", "{}");
        assert_eq!(snapshot(&session)["values"]["m__v"], 10.0);
        assert_eq!(snapshot(&session)["bindings"]["hud"]["frozen"], true);
        let save = session.save_json().unwrap();
        let mut restored = ReactiveSession::restore_json(&source, &save).unwrap();
        send(&mut session, "m__thaw", "{}");
        send(&mut restored, "m__thaw", "{}");
        assert_eq!(snapshot(&session)["bindings"]["hud"]["frozen"], false);
        assert_eq!(snapshot(&restored), snapshot(&session));
        assert_eq!(
            session
                .snapshot()
                .decision_journal
                .iter()
                .map(|entry| entry.change.as_str())
                .collect::<Vec<_>>(),
            ["committed", "held", "released"]
        );
    }
}

// Both v1.0.0 (e2a13500) and v1.4.1 (5e31e83b), built natively with Rust
// 1.98.1, wrote these exact initial bytes. Continuing the same bytes gives
// 10 in v1.0.0 and 14 in v1.4.1: no save field distinguishes those histories.
// A caller must choose which history those bytes are meant to continue.
const AMBIGUOUS_INITIAL: &str = r#"{"schema":"caveat-reactive-save/0.1","source_id":"fnv1a64:8cb496ca5fa15ec4:195","source_sha256":"b8f3d86d72c1db1d3ca8ed50967eac9934a1fec07d855c87af7c02cd1890b138","sequence":0,"elapsed":0.0,"states":{},"graph":{}}"#;

#[test]
fn fresh_module_bindings_are_correct_and_old_ambiguous_saves_need_a_choice() {
    for word in ["hold", "release"] {
        let source = bundle(
            &numeric_module(word, &format!("add( {word} , because )")),
            &format!("use m;\nstate because = 3;\nstate {word} = 11;\n"),
        );
        let mut live = ReactiveSession::from_source(&source).unwrap();
        let saved = live.save_json().unwrap();
        if word == "hold" {
            assert_scoped_save(&live, AMBIGUOUS_INITIAL);
            let error = ReactiveSession::restore_json(&source, AMBIGUOUS_INITIAL)
                .err()
                .unwrap();
            assert!(error.contains("ambiguous"), "{error}");
            for (binding, expected, label) in [
                (ModuleBindings::Scoped, 10.0, "scoped"),
                (
                    ModuleBindings::LegacyHoldCapture,
                    14.0,
                    "legacy-hold-capture",
                ),
            ] {
                let options = RestoreOptions {
                    module_bindings: Some(binding),
                };
                let mut selected = ReactiveSession::restore_json_with_options(
                    &source,
                    AMBIGUOUS_INITIAL,
                    &options,
                )
                .unwrap();
                let marked = selected.save_json().unwrap();
                assert_eq!(
                    serde_json::from_str::<Value>(&marked).unwrap()["module_bindings"],
                    label
                );
                let mut continued = ReactiveSession::restore_json(&source, &marked).unwrap();
                send(&mut selected, "m__go", "{}");
                send(&mut continued, "m__go", "{}");
                assert_eq!(snapshot(&selected)["values"]["m__v"], expected);
                assert_eq!(snapshot(&continued), snapshot(&selected));
            }
        }
        let mut restored = ReactiveSession::restore_json(&source, &saved).unwrap();
        send(&mut live, "m__go", "{}");
        send(&mut restored, "m__go", "{}");
        assert_eq!(snapshot(&live)["values"]["m__v"], 10.0);
        assert_eq!(snapshot(&restored), snapshot(&live));
    }
}

#[test]
fn a_module_reference_does_not_capture_an_event_parameter() {
    // There is no root state named hold. A root-name existence check would
    // miss this collision: the accidentally bare name is an event input.
    let source = bundle(
        "module m;\nstate hold = 7;\nstate v = 0;\nfn add(a,b) = a+b;\n\
         on go set v = add( hold , because );\n",
        "use m;\nstate because = 3;\nevent go hold min 0 max 100;\n",
    );
    let mut session = ReactiveSession::from_source(&source).unwrap();
    send(&mut session, "go", r#"{"hold":11}"#);
    assert_eq!(snapshot(&session)["values"]["m__v"], 10.0);
    let mut restored =
        ReactiveSession::restore_json(&source, &session.save_json().unwrap()).unwrap();
    send(&mut restored, "go", r#"{"hold":13}"#);
    assert_eq!(snapshot(&restored)["values"]["m__v"], 10.0);
}

#[test]
fn scoped_load_failure_reports_the_actual_unresolved_name_and_no_partial_session() {
    let source = bundle(
        &numeric_module("hold", "add( hold , because ) + missing"),
        "use m;\nstate because = 3;\n",
    );
    for _ in 0..2 {
        let error = ReactiveSession::from_source_cached(&source).err().unwrap();
        assert!(error.contains("identifier missing"), "{error}");
    }
    let valid = bundle(
        &numeric_module("hold", "add( hold , because )"),
        "use m;\nstate because = 3;\n",
    );
    let first = ReactiveSession::from_source_cached(&valid).unwrap();
    let second = ReactiveSession::from_source_cached(&valid).unwrap();
    assert_eq!(snapshot(&first), snapshot(&second));
    assert_eq!(snapshot(&first)["values"]["m__v"], json!(0.0));
}
