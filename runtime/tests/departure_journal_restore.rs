//! Journal fields are a projection of grounds, which can also carry departure
//! markers and inherited-name bookkeeping. Restore preserves that full grounds
//! provenance while checking every field the journal actually represents.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};
use std::collections::BTreeSet;

const PROGRAM: &str = r#"
claim seen;
evidence sensor from "sensor";
evidence other from "other";
caveat stale consequence material;
caveat extra consequence low;
stale qualifies sensor;
readings r from sensor window 1;
readings kept from sensor window 1;
event record; event advance; event skip; event decide; event continue;
on record reveal other;
on record sample r = 1 supports seen;
on record sample kept = latest(r) supports seen;
on advance sample r = 2 supports seen;
on skip when qualified(0, other) > 0 sample kept = 3 supports seen;
on decide commit trust because enough using latest(kept);
on continue reopen trust because other;
on continue sample r = 4 supports seen;
on continue sample kept = latest(r) supports seen;
"#;

fn recorded(source: &str, marker: bool, inherited: bool) -> ReactiveSession {
    let mut session = ReactiveSession::from_source(source).unwrap();
    session.dispatch_json("record", "{}").unwrap();
    if marker {
        session.dispatch_json("advance", "{}").unwrap();
    }
    if inherited {
        session.dispatch_json("skip", "{}").unwrap();
    }
    session.dispatch_json("decide", "{}").unwrap();
    session
}

#[test]
fn authentic_journal_saves_keep_full_grounds_and_continue_identically() {
    for series in [false, true] {
        for journal_window in [false, true] {
            let source = format!(
                "{PROGRAM}\n{}\n{}",
                if series {
                    "decisions trust limit 4;"
                } else {
                    ""
                },
                if journal_window {
                    "journal window 1;"
                } else {
                    ""
                }
            );
            let commitment = if series { "trust@1" } else { "trust" };
            for (marker, inherited) in [(true, false), (false, true), (true, true)] {
                let mut original = recorded(&source, marker, inherited);
                let before = original.snapshot();
                let grounds = &before.commitment_grounds[commitment];
                assert_eq!(grounds.departed.contains_key("r"), marker);
                assert_eq!(grounds.inherited.contains("other"), inherited);
                let own: BTreeSet<_> = grounds.own_evidence().map(String::as_str).collect();
                assert_eq!(
                    own,
                    if marker {
                        BTreeSet::from(["kept@1"])
                    } else {
                        BTreeSet::from(["kept@1", "r@1"])
                    }
                );
                let archive = original.drain_archive();
                if marker {
                    assert!(archive.iter().any(|item| {
                        item.record().is_some_and(|record| record.record == "r@1")
                    }));
                    assert!(grounds.departed["r"].archive_ref.is_some());
                }
                let save = original.save_json().unwrap();
                let mut restored = ReactiveSession::restore_json(&source, &save).unwrap();
                assert_eq!(restored.save_json().unwrap(), save);
                assert_eq!(restored.snapshot(), before);
                assert!(restored.drain_archive().is_empty());
                for session in [&mut original, &mut restored] {
                    session.dispatch_json("continue", "{}").unwrap();
                    assert_eq!(session.snapshot().commitment_grounds[commitment], *grounds);
                }
                assert_eq!(original.save_json().unwrap(), restored.save_json().unwrap());
                assert_eq!(
                    serde_json::to_value(original.drain_archive()).unwrap(),
                    serde_json::to_value(restored.drain_archive()).unwrap()
                );
                ReactiveSession::restore_json(&source, &restored.save_json().unwrap()).unwrap();
            }
        }
    }
}

fn refusal(save: &Value) -> String {
    ReactiveSession::restore_json(PROGRAM, &save.to_string())
        .expect_err("a forged field must still be refused")
}

#[test]
fn journal_projection_still_rejects_changed_witnesses_caveats_and_values() {
    let session = recorded(PROGRAM, true, false);
    let save: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    for (field, value, message) in [
        ("because", json!([]), "commitment witnesses differ"),
        (
            "because",
            json!(["other", "kept@1"]),
            "commitment witnesses differ",
        ),
        ("caveats", json!([]), "commitment witnesses differ"),
        (
            "caveats",
            json!(["extra", "stale"]),
            "commitment witnesses differ",
        ),
        ("value", json!(99), "entry value differs"),
    ] {
        let mut forged = save.clone();
        forged["decision_journal"][0][field] = value;
        let error = refusal(&forged);
        assert!(error.contains(message), "{field}: {error}");
    }
}

#[test]
fn marker_and_inherited_validation_is_not_bypassed_by_journal_projection() {
    let session = recorded(PROGRAM, true, true);
    let save: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    let mut outside_basis = save.clone();
    outside_basis["commitment_bases"]["trust"]["provenance"]
        .as_object_mut()
        .unwrap()
        .remove("departed");
    assert!(refusal(&outside_basis).contains("outside its lineage"));

    let mut invalid_inherited = save.clone();
    invalid_inherited["commitment_grounds"]["trust"]["inherited"] = json!(["sensor"]);
    assert!(refusal(&invalid_inherited).contains("which it does not hold"));

    for (field, value, message) in [
        ("read", json!(0), "counts none"),
        ("departed_at", json!(999), "departed out of sequence"),
    ] {
        let mut forged = save.clone();
        // Keep grounds within the edited basis so the independent marker
        // validator, rather than journal equality, must reject this forgery.
        forged["commitment_grounds"]["trust"]["departed"][0][field] = value.clone();
        forged["commitment_bases"]["trust"]["provenance"]["departed"][0][field] = value;
        let error = refusal(&forged);
        assert!(error.contains(message), "{field}: {error}");
    }
}
