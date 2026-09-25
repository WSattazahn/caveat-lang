//! Permission on commitments. See spec/caveat-permission-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

const LEDGER: &str = r#"
identifiers limit 64;
claim ready; claim may_merge; claim changed; claim revoked;
evidence ci from "the checks";
evidence go from "the user's go-ahead, for one head";
evidence broad from "the user's go-ahead for any revision";
evidence push from "a new commit";
evidence revocation from "the user took the go-ahead back";
caveat secondhand consequence low;
secondhand qualifies go;
renewable push limit 16;
readings checks from ci limit 8;
readings approvals from go limit 8;
decisions merge limit 4;
state head = 0;
event pushed commit id;
event check;
event approved commit id;
event approve_any;
event merge;
event merge_any;
event revoke;
event approve_and_merge commit id;
event broken_merge;
on pushed when observed(push) renew push;
on pushed reveal push supports changed;
on pushed set head = commit;
on pushed when committed(merge) and not reopened(merge) reopen merge because push;
on check sample checks = 1 supports ready;
on approved sample approvals = commit supports may_merge;
on approve_any reveal broad supports may_merge;
on merge commit merge because enough using latest(checks) permitted by latest(approvals) for head;
on merge_any commit merge because enough using latest(checks) permitted by broad;
on revoke when not observed(revocation) reveal revocation supports revoked;
on revoke withdraw latest(approvals) because revocation;
on approve_and_merge sample approvals = commit supports may_merge;
on approve_and_merge commit merge because enough using latest(checks) permitted by latest(approvals) for head;
on broken_merge commit merge because enough using 1 / 0 permitted by latest(approvals) for head;
bind status.withdrawn = 0;
bind status.withdrawn = 1 when permission_withdrawn(merge);
"#;

const A: &str = "602bdbec0a047a5319f53e83f336b9f7aec0e5ed";
const B: &str = "31541a5300baf3b6651da2c973b0f43b3e8f8bd5";

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn send(game: &mut ReactiveSession, event: &str, payload: Value) -> Value {
    serde_json::to_value(
        game.dispatch_outcome_json(event, &payload.to_string())
            .unwrap_or_else(|fatal| panic!("{event}: fatal {}", fatal.message)),
    )
    .unwrap()
}

fn go(game: &mut ReactiveSession, event: &str) -> Value {
    send(game, event, json!({}))
}

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn accepted(outcome: Value) {
    assert_eq!(outcome["outcome"], "accepted", "{outcome}");
}

/// A refusal as not_permitted that leaves the session exactly as it was.
fn not_permitted(game: &mut ReactiveSession, event: &str, payload: Value, why: &str) {
    let before = (snapshot(game), game.save_json().unwrap());
    let outcome = send(game, event, payload);
    assert_eq!(
        (
            outcome["outcome"].as_str(),
            outcome["origin"].as_str(),
            outcome["code"].as_str()
        ),
        (Some("rejected"), Some("policy"), Some("not_permitted")),
        "{outcome}"
    );
    assert!(
        outcome["message"].as_str().unwrap().contains(why),
        "{outcome}"
    );
    assert_eq!(
        (snapshot(game), game.save_json().unwrap()),
        before,
        "a refusal changed the session"
    );
}

fn ready_at(game: &mut ReactiveSession, commit: &str) {
    accepted(send(game, "pushed", json!({ "commit": commit })));
    accepted(go(game, "check"));
}

#[test]
fn a_matching_grant_permits_the_commitment_and_is_recorded_apart_from_its_grounds() {
    let mut game = session(LEDGER);
    ready_at(&mut game, A);
    accepted(send(&mut game, "approved", json!({ "commit": A })));
    accepted(go(&mut game, "merge"));
    let shot = snapshot(&game);
    assert_eq!(
        shot["commitment_permissions"]["merge@1"],
        json!({ "grant": "approvals@1", "scope": { "granted": 1.0, "required": 1.0 }, "caveats": ["secondhand"] })
    );
    // Permission is not grounds, but it stays in the lineage.
    assert_eq!(
        shot["commitment_grounds"]["merge@1"]["evidence"],
        json!(["checks@1"])
    );
    let lineage = shot["commitment_bases"]["merge@1"]["provenance"]["evidence"]
        .as_array()
        .unwrap()
        .clone();
    assert!(lineage.contains(&json!("approvals@1")), "{lineage:?}");
    let journal = &shot["decision_journal"][0];
    assert_eq!(
        (journal["because"].clone(), journal["permitted_by"].clone()),
        (json!(["checks@1"]), json!("approvals@1"))
    );
}

#[test]
fn a_missing_withdrawn_or_mismatched_grant_is_refused_and_the_session_goes_on() {
    // No grant yet.
    let mut game = session(LEDGER);
    ready_at(&mut game, A);
    not_permitted(
        &mut game,
        "merge",
        json!({}),
        "approvals holds no grant yet",
    );
    // A grant for another head.
    accepted(send(&mut game, "approved", json!({ "commit": A })));
    ready_at(&mut game, B);
    not_permitted(
        &mut game,
        "merge",
        json!({}),
        "approvals@1 permits 1, not 2",
    );
    // A withdrawn grant.
    accepted(send(&mut game, "approved", json!({ "commit": B })));
    accepted(go(&mut game, "revoke"));
    not_permitted(&mut game, "merge", json!({}), "approvals@2 was withdrawn");
    // Named evidence not observed.
    not_permitted(&mut game, "merge_any", json!({}), "broad is not observed");
    // Still usable: a fresh grant for the head permits it.
    accepted(send(&mut game, "approved", json!({ "commit": B })));
    accepted(go(&mut game, "merge"));
    assert_eq!(
        snapshot(&game)["commitment_permissions"]["merge@1"]["grant"],
        "approvals@3"
    );
}

#[test]
fn a_refusal_rolls_back_the_grant_and_identifiers_the_same_event_brought() {
    let mut game = session(LEDGER);
    ready_at(&mut game, A);
    // The event samples a grant for a new commit, then its commit is refused.
    not_permitted(
        &mut game,
        "approve_and_merge",
        json!({ "commit": "never-seen" }),
        "permits 2, not 1",
    );
    let shot = snapshot(&game);
    assert_eq!(shot["identifiers"], json!([A]));
    assert!(shot["reading_streams"]["approvals"]["occurrences"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn other_failures_keep_their_own_classification() {
    let mut game = session(LEDGER);
    ready_at(&mut game, A);
    accepted(send(&mut game, "approved", json!({ "commit": A })));
    let error = game
        .dispatch_outcome_json("broken_merge", "{}")
        .unwrap_err();
    assert_eq!(error.code, "unclassified", "{}", error.message);
    assert!(
        !error.message.contains("not permitted"),
        "{}",
        error.message
    );

    let limited = LEDGER.replace("decisions merge limit 4;", "decisions merge limit 1;");
    let mut game = session(&limited);
    ready_at(&mut game, A);
    accepted(send(&mut game, "approved", json!({ "commit": A })));
    accepted(go(&mut game, "merge"));
    ready_at(&mut game, B);
    accepted(send(&mut game, "approved", json!({ "commit": B })));
    let outcome = go(&mut game, "merge");
    assert_eq!(
        (outcome["origin"].as_str(), outcome["code"].as_str()),
        (Some("limit"), Some("history_limit"))
    );

    for (clause, expected) in [
        (
            "permitted by nowhere",
            "nowhere must name a declared evidence",
        ),
        (
            "permitted by latest(nowhere)",
            "nowhere must name a declared reading stream",
        ),
        (
            "permitted by broad for head",
            "needs a grant from latest(STREAM)",
        ),
        ("permitted by", "permitted by needs a grant"),
        (
            "permitted by latest(approvals) for",
            "permitted by … for needs an expression",
        ),
    ] {
        let source = format!("{LEDGER}\nevent odd;\non odd commit merge because enough {clause};");
        let error = ReactiveSession::from_source(&source).err().unwrap();
        assert!(error.contains(expected), "{clause}: {error}");
    }
}

#[test]
fn later_grants_withdrawals_and_head_changes_do_not_rewrite_an_earlier_permission() {
    let mut game = session(LEDGER);
    ready_at(&mut game, A);
    accepted(send(&mut game, "approved", json!({ "commit": A })));
    accepted(go(&mut game, "merge"));
    let first = snapshot(&game)["commitment_permissions"]["merge@1"].clone();
    // A newer grant for the same head, then withdrawn: not merge@1's grant.
    accepted(send(&mut game, "approved", json!({ "commit": A })));
    accepted(go(&mut game, "revoke"));
    let shot = snapshot(&game);
    assert_eq!(shot["withdrawals"][0]["evidence"], "approvals@2");
    assert_eq!(shot["bindings"]["status"]["withdrawn"], 0.0);
    assert_eq!(shot["commitment_permissions"]["merge@1"], first);

    // The head changes: the ledger's own rule reopens; the record stays.
    ready_at(&mut game, B);
    let shot = snapshot(&game);
    assert_eq!(shot["decision_journal"][1]["change"], "reopened");
    assert_eq!(shot["commitment_permissions"]["merge@1"], first);
    accepted(send(&mut game, "approved", json!({ "commit": B })));
    accepted(go(&mut game, "merge"));
    assert_eq!(
        snapshot(&game)["commitment_permissions"]["merge@2"]["grant"],
        "approvals@3"
    );
}

#[test]
fn permission_withdrawn_asks_about_the_grant_recorded_on_the_current_revision() {
    let mut game = session(LEDGER);
    ready_at(&mut game, A);
    accepted(send(&mut game, "approved", json!({ "commit": A })));
    accepted(go(&mut game, "merge"));
    assert_eq!(snapshot(&game)["bindings"]["status"]["withdrawn"], 0.0);
    accepted(go(&mut game, "revoke"));
    let shot = snapshot(&game);
    assert_eq!(shot["withdrawals"][0]["evidence"], "approvals@1");
    assert_eq!(shot["bindings"]["status"]["withdrawn"], 1.0);
    // Nothing reopened by itself.
    assert_eq!(shot["decision_journal"].as_array().unwrap().len(), 1);
}

#[test]
fn an_explicitly_named_broader_grant_permits_without_scope() {
    let mut game = session(LEDGER);
    ready_at(&mut game, A);
    accepted(go(&mut game, "approve_any"));
    accepted(go(&mut game, "merge_any"));
    assert_eq!(
        snapshot(&game)["commitment_permissions"]["merge@1"],
        json!({ "grant": "broad" })
    );
    // The scoped clause does not fall back to it.
    ready_at(&mut game, B);
    not_permitted(
        &mut game,
        "merge",
        json!({}),
        "approvals holds no grant yet",
    );
}

#[test]
fn saves_restore_granted_refused_and_skipped_paths_and_refuse_inconsistent_permissions() {
    let skipped = format!("{LEDGER}\nevent maybe;\non maybe when head == 99 commit merge because enough using latest(checks) permitted by latest(approvals) for head;");
    let mut game = session(&skipped);
    ready_at(&mut game, A);
    accepted(go(&mut game, "maybe"));
    not_permitted(&mut game, "merge", json!({}), "no grant yet");
    accepted(send(&mut game, "approved", json!({ "commit": A })));
    accepted(go(&mut game, "merge"));
    let saved = game.save_json().unwrap();
    let mut restored = ReactiveSession::restore_json(&skipped, &saved).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&game));
    for session in [&mut game, &mut restored] {
        accepted(go(session, "revoke"));
        accepted(send(session, "pushed", json!({ "commit": B })));
    }
    assert_eq!(snapshot(&restored), snapshot(&game));

    let parsed: Value = serde_json::from_str(&saved).unwrap();
    let with = |change: &dyn Fn(&mut Value)| {
        let mut save = parsed.clone();
        change(&mut save);
        save.to_string()
    };
    let record = parsed["commitment_permissions"]["merge@1"].clone();
    for (text, expected) in [
        (
            with(&|save| save["commitment_permissions"] = json!({ "merge@9": record.clone() })),
            "unknown commitment merge@9",
        ),
        (
            with(&|save| save["commitment_permissions"]["merge@1"]["grant"] = json!("nowhere")),
            "names unknown evidence nowhere",
        ),
        (
            with(&|save| {
                save["commitment_permissions"]["merge@1"]["caveats"] = json!(["imagined"])
            }),
            "imagined",
        ),
        (
            with(&|save| {
                save.as_object_mut()
                    .unwrap()
                    .remove("commitment_permissions");
            }),
            "disagrees with its record",
        ),
        (
            with(&|save| {
                save["decision_journal"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("permitted_by");
            }),
            "disagrees with its record",
        ),
        (
            with(&|save| save["decision_journal"][0]["permitted_by"] = json!("approvals@9")),
            "disagrees with its record",
        ),
    ] {
        let error = ReactiveSession::restore_json(&skipped, &text)
            .err()
            .unwrap();
        assert!(error.contains(expected), "{expected}: {error}");
    }
}
