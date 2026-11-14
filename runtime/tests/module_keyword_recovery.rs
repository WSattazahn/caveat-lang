//! D088: a module's `hold`/`release` reference two whitespace words before
//! `because` was mistaken for an effect verb. Recover refused reactive
//! bundles without changing the binding of programs that already load.
use caveat_runtime::link::{self, BundlePart};
use caveat_runtime::reactive::{interface_source, ReactiveSession};
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
        // The raw linker's contract remains unchanged. The reactive loader
        // can recover only while it still has the original module scopes.
        let ordinary = link::link(&source).unwrap();
        assert!(ReactiveSession::from_source(&ordinary).is_err());
        let mut live = ReactiveSession::from_source(&source).unwrap();
        send(&mut live, "m__go", "{}");
        assert_eq!(live.save_json().unwrap(), saved);

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
// Recovery keeps the already accepted 1.4.1 binding; it does not claim to
// resolve the older silent-capture ambiguity.
const AMBIGUOUS_INITIAL: &str = r#"{"schema":"caveat-reactive-save/0.1","source_id":"fnv1a64:8cb496ca5fa15ec4:195","source_sha256":"b8f3d86d72c1db1d3ca8ed50967eac9934a1fec07d855c87af7c02cd1890b138","sequence":0,"elapsed":0.0,"states":{},"graph":{}}"#;

#[test]
fn already_accepted_root_bindings_and_their_saves_keep_141_behavior() {
    for word in ["hold", "release"] {
        let source = bundle(
            &numeric_module(word, &format!("add( {word} , because )")),
            &format!("use m;\nstate because = 3;\nstate {word} = 11;\n"),
        );
        let mut live = ReactiveSession::from_source(&source).unwrap();
        let saved = live.save_json().unwrap();
        if word == "hold" {
            assert_eq!(saved, AMBIGUOUS_INITIAL);
        }
        let mut restored = ReactiveSession::restore_json(&source, &saved).unwrap();
        send(&mut live, "m__go", "{}");
        send(&mut restored, "m__go", "{}");
        assert_eq!(snapshot(&live)["values"]["m__v"], 14.0);
        assert_eq!(snapshot(&restored), snapshot(&live));
    }
}

#[test]
fn an_already_accepted_event_parameter_binding_is_not_reinterpreted() {
    // There is no root state named hold. A root-name existence check would
    // still change this accepted program: the bare name is an event input.
    let source = bundle(
        "module m;\nstate hold = 7;\nstate v = 0;\nfn add(a,b) = a+b;\n\
         on go set v = add( hold , because );\n",
        "use m;\nstate because = 3;\nevent go hold min 0 max 100;\n",
    );
    let mut session = ReactiveSession::from_source(&source).unwrap();
    send(&mut session, "go", r#"{"hold":11}"#);
    assert_eq!(snapshot(&session)["values"]["m__v"], 14.0);
    let mut restored =
        ReactiveSession::restore_json(&source, &session.save_json().unwrap()).unwrap();
    send(&mut restored, "go", r#"{"hold":13}"#);
    assert_eq!(snapshot(&restored)["values"]["m__v"], 16.0);
}

#[test]
fn recovery_failure_keeps_the_original_error_and_no_partial_session() {
    let source = bundle(
        &numeric_module("hold", "add( hold , because ) + missing"),
        "use m;\nstate because = 3;\n",
    );
    for _ in 0..2 {
        let error = ReactiveSession::from_source_cached(&source).err().unwrap();
        assert!(error.contains("unknown numeric identifier hold"), "{error}");
        assert!(!error.contains("identifier missing"), "{error}");
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
