//! Correct module scope and explicitly selected historical continuations.
use caveat_runtime::reactive::{ModuleBindings, ReactiveSave, ReactiveSession, RestoreOptions};
use serde_json::{json, Value};

const SOURCE: &str = include_str!("fixtures/module-bindings/collision.bundle");
const OLD_INITIAL: &str = include_str!("fixtures/module-bindings/collision-v1.4.1-initial.json");
const OLD_100_INITIAL: &str =
    include_str!("fixtures/module-bindings/collision-v1.0.0-initial.json");
const OLD_100_STATE: &str = include_str!("fixtures/module-bindings/collision-v1.0.0-state.json");
const OLD_STATE: &str = include_str!("fixtures/module-bindings/collision-v1.4.1-state.json");
const OLD_NAMED: &str = include_str!("fixtures/module-bindings/collision-v1.4.1-replay-1.json");
const OLD_POSITIONAL: &str =
    include_str!("fixtures/module-bindings/collision-v1.4.1-replay-2.json");
const LEGACY_ONLY: &str = include_str!("fixtures/module-bindings/legacy-only.bundle");

fn options(bindings: ModuleBindings) -> RestoreOptions {
    RestoreOptions {
        module_bindings: Some(bindings),
    }
}

fn value(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.snapshot()).unwrap()["values"]["m__v"].clone()
}

fn send(session: &mut ReactiveSession) {
    let outcome = session.dispatch_outcome_json("m__go", "{}").unwrap();
    assert_eq!(
        serde_json::to_value(outcome).unwrap()["outcome"],
        "accepted"
    );
}

type SaveWriter = fn(&ReactiveSession) -> Result<String, String>;
const WRITERS: [(SaveWriter, &str); 3] = [
    (ReactiveSession::save_json, "caveat-reactive-save/0.1"),
    (ReactiveSession::save_compact_json, "caveat-replay-save/0.1"),
    (
        ReactiveSession::save_compact_positional_json,
        "caveat-replay-save/0.2",
    ),
];

#[test]
fn identical_genuine_initial_saves_cannot_select_a_historical_interpretation() {
    assert_eq!(OLD_INITIAL, OLD_100_INITIAL);
    let error = ReactiveSession::restore_json(SOURCE, OLD_INITIAL).unwrap_err();
    assert!(error.contains("ambiguous"), "{error}");
    assert!(error.contains("moduleBindings"), "{error}");
    for (bindings, expected) in [
        (ModuleBindings::Scoped, 10.0),
        (ModuleBindings::LegacyHoldCapture, 14.0),
    ] {
        let saved: ReactiveSave = serde_json::from_str(OLD_INITIAL).unwrap();
        let mut session =
            ReactiveSession::restore_with_options(SOURCE, &saved, &options(bindings)).unwrap();
        send(&mut session);
        assert_eq!(value(&session), expected);
    }
}

#[test]
fn all_genuine_historical_formats_require_a_choice_and_continue_that_history() {
    for (saved, bindings, expected) in [
        (OLD_100_STATE, ModuleBindings::Scoped, 10.0),
        (OLD_STATE, ModuleBindings::LegacyHoldCapture, 14.0),
        (OLD_NAMED, ModuleBindings::LegacyHoldCapture, 14.0),
        (OLD_POSITIONAL, ModuleBindings::LegacyHoldCapture, 14.0),
    ] {
        let error = ReactiveSession::restore_json(SOURCE, saved).unwrap_err();
        assert!(error.contains("ambiguous"), "{error}");
        let mut session =
            ReactiveSession::restore_json_with_options(SOURCE, saved, &options(bindings)).unwrap();
        assert_eq!(value(&session), expected);
        let marked = session.save_json().unwrap();
        let parsed: Value = serde_json::from_str(&marked).unwrap();
        assert!(parsed["module_bindings"].is_string());
        assert_eq!(parsed["source_id"], "fnv1a64:8cb496ca5fa15ec4:195");
        assert_eq!(
            parsed["source_sha256"],
            "b8f3d86d72c1db1d3ca8ed50967eac9934a1fec07d855c87af7c02cd1890b138"
        );
        let mut resumed = ReactiveSession::restore_json(SOURCE, &marked).unwrap();
        send(&mut session);
        send(&mut resumed);
        assert_eq!(value(&resumed), expected);
        assert_eq!(resumed.save_json().unwrap(), session.save_json().unwrap());
    }
}

#[test]
fn every_save_writer_remembers_the_selected_interpretation_before_replaying() {
    for (bindings, expected, label) in [
        (ModuleBindings::Scoped, 10.0, "scoped"),
        (
            ModuleBindings::LegacyHoldCapture,
            14.0,
            "legacy-hold-capture",
        ),
    ] {
        // Restoring a named replay preserves a replay log. Scoped selection
        // replays its event under the explicitly chosen corrected meaning.
        let session =
            ReactiveSession::restore_json_with_options(SOURCE, OLD_NAMED, &options(bindings))
                .unwrap();
        assert_eq!(value(&session), expected);
        for (write, schema) in WRITERS {
            let saved = write(&session).unwrap();
            let parsed: Value = serde_json::from_str(&saved).unwrap();
            assert_eq!(parsed["schema"], schema);
            assert_eq!(parsed["module_bindings"], label);
            let mut restored = ReactiveSession::restore_json(SOURCE, &saved).unwrap();
            assert_eq!(restored.save_json().unwrap(), session.save_json().unwrap());
            send(&mut restored);
            assert_eq!(value(&restored), expected);
        }
    }
}

#[test]
fn saved_interpretation_cannot_be_overridden_and_unknown_markers_refuse() {
    let mut scoped = ReactiveSession::from_source(SOURCE).unwrap();
    send(&mut scoped);
    for (write, _) in WRITERS {
        let saved = write(&scoped).unwrap();
        let conflict = ReactiveSession::restore_json_with_options(
            SOURCE,
            &saved,
            &options(ModuleBindings::LegacyHoldCapture),
        )
        .unwrap_err();
        assert!(conflict.contains("conflicts"), "{conflict}");
        for invalid in [
            Value::Null,
            json!("unknown"),
            json!(0),
            json!({"scoped": null}),
            json!(["scoped"]),
        ] {
            let mut malformed: Value = serde_json::from_str(&saved).unwrap();
            malformed["module_bindings"] = invalid;
            assert!(ReactiveSession::restore_json(SOURCE, &malformed.to_string()).is_err());
        }
    }
    for invalid in [
        r#"{"moduleBindings":null}"#,
        r#"{"moduleBindings":"unknown"}"#,
        r#"{"moduleBindings":{"scoped":null}}"#,
        r#"{"moduleBinding":"scoped"}"#,
    ] {
        assert!(
            serde_json::from_str::<RestoreOptions>(invalid).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn cached_legacy_restores_do_not_change_fresh_or_scoped_sessions() {
    for _ in 0..3 {
        for (bindings, expected) in [
            (ModuleBindings::LegacyHoldCapture, 14.0),
            (ModuleBindings::Scoped, 10.0),
        ] {
            let mut restored =
                ReactiveSession::restore_json_with_options(SOURCE, OLD_INITIAL, &options(bindings))
                    .unwrap();
            send(&mut restored);
            assert_eq!(value(&restored), expected);
            let mut fresh = ReactiveSession::from_source_cached(SOURCE).unwrap();
            send(&mut fresh);
            assert_eq!(value(&fresh), 10.0);
        }
    }
}

#[test]
fn a_legacy_only_source_can_continue_its_save_without_becoming_a_fresh_default() {
    assert!(ReactiveSession::from_source(LEGACY_ONLY).is_err());
    for saved in [
        include_str!("fixtures/module-bindings/legacy-only-v1.4.1-initial.json"),
        include_str!("fixtures/module-bindings/legacy-only-v1.4.1-state.json"),
        include_str!("fixtures/module-bindings/legacy-only-v1.4.1-replay-1.json"),
        include_str!("fixtures/module-bindings/legacy-only-v1.4.1-replay-2.json"),
    ] {
        let mut session = ReactiveSession::restore_json(LEGACY_ONLY, saved).unwrap();
        send(&mut session);
        assert_eq!(value(&session), 13.0);
        let marked = session.save_json().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&marked).unwrap()["module_bindings"],
            "legacy-hold-capture"
        );
        assert!(ReactiveSession::restore_json_with_options(
            LEGACY_ONLY,
            saved,
            &options(ModuleBindings::Scoped)
        )
        .is_err());
        let mut resumed = ReactiveSession::restore_json(LEGACY_ONLY, &marked).unwrap();
        send(&mut resumed);
        assert_eq!(value(&resumed), 13.0);
    }
    assert!(ReactiveSession::from_source_cached(LEGACY_ONLY).is_err());
}

#[test]
fn unaffected_saves_keep_their_bytes_and_need_no_marker_or_option() {
    let source = "state n = 0; event go; on go set n = n + 1;";
    let mut session = ReactiveSession::from_source(source).unwrap();
    session.dispatch_outcome_json("go", "{}").unwrap();
    for (write, _) in WRITERS {
        let saved = write(&session).unwrap();
        assert!(!saved.contains("module_bindings"));
        for bindings in [ModuleBindings::Scoped, ModuleBindings::LegacyHoldCapture] {
            let restored =
                ReactiveSession::restore_json_with_options(source, &saved, &options(bindings))
                    .unwrap();
            assert_eq!(restored.save_json().unwrap(), session.save_json().unwrap());
        }
        assert!(ReactiveSession::restore_json(source, &saved).is_ok());
    }
}

#[test]
fn an_explicit_choice_does_not_bypass_source_or_saved_value_validation() {
    let options = options(ModuleBindings::LegacyHoldCapture);
    for field in ["source_id", "source_sha256"] {
        let mut forged: Value = serde_json::from_str(OLD_STATE).unwrap();
        forged[field] = json!("wrong");
        assert!(
            ReactiveSession::restore_json_with_options(SOURCE, &forged.to_string(), &options)
                .is_err()
        );
    }
    let mut forged: Value = serde_json::from_str(OLD_STATE).unwrap();
    forged["states"]["nonexistent"] = json!({"value": 1});
    assert!(
        ReactiveSession::restore_json_with_options(SOURCE, &forged.to_string(), &options).is_err()
    );
    // A failed restoration does not contaminate the cached fresh program.
    let mut fresh = ReactiveSession::from_source_cached(SOURCE).unwrap();
    send(&mut fresh);
    assert_eq!(value(&fresh), 10.0);
}
