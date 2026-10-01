//! Neutral reveal records observation without inventing a claim relation.
use caveat_runtime::reactive::{ReactiveSession, ReactiveSnapshot};
use serde_json::{json, Value};

fn load(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("load: {error}"))
}
fn send(session: &mut ReactiveSession, event: &str) -> ReactiveSnapshot {
    session
        .dispatch_json(event, "{}")
        .unwrap_or_else(|error| panic!("{event}: {error}"))
}
fn snapshot(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.snapshot()).unwrap()
}
fn resume(source: &str, session: &ReactiveSession) -> ReactiveSession {
    let restored = ReactiveSession::restore_json(source, &session.save_json().unwrap()).unwrap();
    assert_eq!(snapshot(session), snapshot(&restored));
    restored
}

const MEMORY: &str = r#"
claim useful;
evidence memory from "a consulted memory";
evidence correction from "a correction";
caveat stale consequence material;
readings freshness from memory limit 8;
decisions strategy limit 4;
state current = 0;
event consult; event sample; event assess; event age; event correct;
on consult reveal memory;
on consult set current = qualified(90, memory);
on sample sample freshness = 90 supports useful;
on assess commit strategy because enough using latest(freshness);
on age qualify memory with stale;
on correct reveal correction;
on correct withdraw memory because correction;
bind hud.seen = observed(memory);
"#;

#[test]
fn neutral_observation_updates_bindings_and_qualification_without_a_stance() {
    let mut session = load(MEMORY);
    assert_eq!(snapshot(&session)["bindings"]["hud"]["seen"], false);
    let first = send(&mut session, "consult");
    assert_eq!(first.observations, ["memory"]);
    assert_eq!(
        serde_json::to_value(&first.effects).unwrap(),
        json!([{"kind":"reveal","evidence":"memory"}])
    );
    assert_eq!(snapshot(&session)["bindings"]["hud"]["seen"], true);
    assert!(first.relations.is_empty());
    assert!(first.value_grounds["current"].evidence.contains("memory"));
    send(&mut session, "sample");
    let decided = send(&mut session, "assess");
    send(&mut session, "age");
    let aged = session.snapshot();
    assert!(aged.value_grounds["current"].caveats.contains("stale"));
    assert_eq!(aged.reading_streams, decided.reading_streams);
    assert_eq!(aged.commitment_bases, decided.commitment_bases);
    assert_eq!(aged.commitment_grounds, decided.commitment_grounds);
    assert_eq!(aged.decision_journal, decided.decision_journal);
    let later = send(&mut session, "sample");
    assert!(later.reading_streams["freshness"].occurrences[1]
        .provenance
        .caveats
        .contains("stale"));
    assert!(later
        .relations
        .iter()
        .all(|edge| edge.from != "memory"
            || !matches!(edge.relation.as_str(), "supports" | "opposes")));
    resume(MEMORY, &session);
}

#[test]
fn neutral_and_stance_reveals_are_idempotent_without_reordering() {
    let source = r#"
claim fact; evidence a from "a"; evidence b from "b";
event neutral; event stance; event repeat;
on neutral reveal b;
on stance reveal a supports fact;
on stance reveal b opposes fact;
on repeat reveal a; on repeat reveal b;
on repeat reveal a supports fact; on repeat reveal b opposes fact;
on repeat commit choice because enough using qualified(1, a) + qualified(1, b);
"#;
    let mut session = load(source);
    send(&mut session, "neutral");
    send(&mut session, "stance");
    let mut session = resume(source, &session);
    let state = send(&mut session, "repeat");
    assert_eq!(state.observations, ["b", "a"]);
    assert_eq!(
        state
            .relations
            .iter()
            .filter(|edge| matches!(edge.relation.as_str(), "supports" | "opposes"))
            .count(),
        2
    );
    assert_eq!(state.effects.len(), 1);
    assert_eq!(state.decision_journal[0].because, ["b", "a"]);
    resume(source, &session);
}

#[test]
fn mixed_sampling_and_neutral_order_includes_source_observations() {
    let source = r#"
claim fact; evidence initial from "initial"; initial supports fact;
evidence neutral from "consulted"; evidence sensor from "sensor";
readings readings from sensor limit 4;
event first; event next;
on first sample readings = 1 supports fact;
on first reveal neutral;
on first sample readings = 2 opposes fact;
on first commit choice because enough using qualified(1, initial) + history_at(readings, 0) + qualified(2, neutral) + latest(readings);
on next sample readings = 3 supports fact;
"#;
    let mut session = load(source);
    let state = send(&mut session, "first");
    assert_eq!(
        state.observations,
        ["initial", "readings@1", "neutral", "readings@2"]
    );
    assert_eq!(state.decision_journal[0].because, state.observations);
    let mut session = resume(source, &session);
    assert_eq!(
        send(&mut session, "next").observations.last().unwrap(),
        "readings@3"
    );
}

#[test]
fn typed_procedures_and_skipped_neutral_guards_keep_control_out_of_grounds() {
    let source = r#"
claim ready;
evidence gate from "gate"; gate supports ready;
evidence memory from "memory";
caveat uncertain consequence material; uncertain qualifies gate;
state control = 0; state content = 0;
event skip; event enter;
proc consult(source evidence, ignored) { reveal source; };
on skip when qualified(0, gate) > 1 reveal memory;
on enter when qualified(1, gate) > 0 call consult(memory, qualified(2, gate));
on enter set content = qualified(5, memory);
bind hud.seen = observed(memory);
"#;
    let mut session = load(source);
    let skipped = send(&mut session, "skip");
    assert!(skipped.observations.is_empty());
    assert_eq!(snapshot(&session)["bindings"]["hud"]["seen"], false);
    assert!(skipped.binding_qualifications["hud"]["seen"]
        .evidence
        .contains("gate"));
    let entered = send(&mut session, "enter");
    assert_eq!(entered.observations, ["gate", "memory"]);
    assert!(entered.qualified_values["content"]
        .provenance
        .evidence
        .contains("gate"));
    assert!(entered.qualified_values["content"]
        .provenance
        .caveats
        .contains("uncertain"));
    assert_eq!(
        entered.value_grounds["content"]
            .evidence
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        ["memory"]
    );
    assert!(entered.value_grounds["content"].caveats.is_empty());
    resume(source, &session);
}

#[test]
fn observation_before_use_check_accepts_neutral_reveal_but_not_sampling_template() {
    let source = "evidence memory from \"memory\"; caveat stale consequence material; event go;";
    load(&format!(
        "{source} on go reveal memory; on go qualify memory with stale;"
    ));
    for body in [
        "on go qualify memory with stale; on go reveal memory;",
        "on go qualify memory with stale;",
        "claim ready; readings samples from memory limit 2; on go sample samples = 1 supports ready; on go qualify memory with stale;",
    ] {
        assert!(ReactiveSession::from_source(&format!("{source} {body}")).unwrap_err().contains("observ"));
    }
    assert!(
        ReactiveSession::from_source("claim fact; event go; on go reveal fact;")
            .unwrap_err()
            .contains("evidence")
    );
}

#[test]
fn rejected_events_roll_back_neutral_observation_guards_and_renewal_ids() {
    let source = r#"
evidence memory from "memory"; renewable memory limit 3;
caveat stale consequence material;
event fail; event consult;
on fail reveal memory; on fail qualify memory with stale; on fail renew memory;
on fail reveal memory; on fail reject "try again";
on consult reveal memory;
bind hud.seen = observed(memory);
"#;
    let mut session = load(source);
    let before = session.save_json().unwrap();
    let outcome = session.dispatch_outcome_json("fail", "{}").unwrap();
    assert_eq!(
        serde_json::to_value(outcome).unwrap()["outcome"],
        "rejected"
    );
    assert_eq!(session.save_json().unwrap(), before);
    assert_eq!(send(&mut session, "consult").observations, ["memory"]);
    resume(source, &session);
}

#[test]
fn renewable_neutral_evidence_qualifies_on_its_own_clock_and_withdraws() {
    let source = r#"
evidence memory from "memory"; evidence correction from "correction";
renewable memory limit 3;
caveat stale consequence material; caveat declared consequence low; declared qualifies memory;
state old = 0; state current = 0;
event consult; event renew; event advance dt min 0 max 10; clock advance every 1; event correct;
on consult reveal memory; on consult set current = qualified(1, memory);
on consult qualify memory with stale after 2;
on renew set old = current; on renew renew memory;
on correct reveal correction; on correct withdraw memory because correction;
bind hud.seen = observed(memory);
"#;
    let mut session = load(source);
    send(&mut session, "consult");
    send(&mut session, "renew");
    assert_eq!(snapshot(&session)["bindings"]["hud"]["seen"], false);
    session.dispatch_json("advance", "{\"dt\":2}").unwrap();
    assert!(session.snapshot().value_grounds["old"]
        .caveats
        .contains("stale"));
    let current = send(&mut session, "consult");
    assert!(!current.value_grounds["current"].caveats.contains("stale"));
    assert!(current.value_grounds["current"]
        .caveats
        .contains("declared"));
    let withdrawn = send(&mut session, "correct");
    assert_eq!(withdrawn.withdrawals[0].evidence, "memory@2");
    assert!(withdrawn.value_grounds["current"]
        .caveats
        .contains("withdrawn"));
    assert!(!withdrawn.value_grounds["old"].caveats.contains("withdrawn"));
    resume(source, &session);
}

#[test]
fn neutral_named_grant_uses_existing_explicit_permission_policy() {
    let source = r#"
evidence approval from "explicit authorization"; event authorize; event act;
on authorize reveal approval;
on act commit action because enough using 1 permitted by approval;
"#;
    let mut session = load(source);
    assert_eq!(
        serde_json::to_value(session.dispatch_outcome_json("act", "{}").unwrap()).unwrap()["code"],
        "not_permitted"
    );
    send(&mut session, "authorize");
    let accepted = send(&mut session, "act");
    assert_eq!(accepted.commitment_permissions["action"].grant, "approval");
    assert!(accepted.commitment_grounds["action"].evidence.is_empty());
    resume(source, &session);
}

#[test]
fn old_stance_only_saves_and_snapshots_keep_their_shape() {
    let source = "claim fact; evidence e from \"e\"; event go; event neutral; on go reveal e supports fact; on neutral reveal e;";
    let mut session = load(source);
    send(&mut session, "go");
    assert!(snapshot(&session).get("observations").is_none());
    assert!(serde_json::from_str::<Value>(&session.save_json().unwrap())
        .unwrap()
        .get("observations")
        .is_none());
    assert_eq!(
        snapshot(&session)["effects"],
        json!([{"kind":"reveal","evidence":"e","relation":"supports","target":"fact"}])
    );
    let mut restored = resume(source, &session);
    send(&mut session, "neutral");
    send(&mut restored, "neutral");
    assert_eq!(session.save_json().unwrap(), restored.save_json().unwrap());
    assert!(snapshot(&session).get("observations").is_none());
    assert!(session.snapshot().effects.is_empty());
}

#[test]
fn malformed_observation_ledgers_and_reveal_effects_are_rejected() {
    let mut session = load(MEMORY);
    send(&mut session, "consult");
    let original: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    let rejected = |save: Value| {
        assert!(
            ReactiveSession::restore_json(MEMORY, &save.to_string()).is_err(),
            "accepted malformed save: {save}"
        )
    };
    for observations in [
        json!([]),
        json!(null),
        json!(["memory", "memory"]),
        json!(["useful"]),
        json!(["missing"]),
    ] {
        let mut save = original.clone();
        save["observations"] = observations;
        rejected(save);
    }
    let mut missing = original.clone();
    missing.as_object_mut().unwrap().remove("observations");
    rejected(missing);
    for effect in [
        json!({"kind":"reveal","evidence":"memory","relation":"supports"}),
        json!({"kind":"reveal","evidence":"memory","target":"useful"}),
        json!({"kind":"reveal","evidence":"memory","relation":"qualifies","target":"useful"}),
        json!({"kind":"reveal","evidence":"memory","relation":"supports","target":"correction"}),
        json!({"kind":"reveal","evidence":"useful"}),
        json!({"kind":"reveal","evidence":"memory","relation":"supports","target":"useful"}),
    ] {
        let mut save = original.clone();
        save["effects"] = json!([effect]);
        rejected(save);
    }
}

#[test]
fn restore_rejects_lost_stances_source_order_and_reading_chronology() {
    let source = r#"
claim fact; evidence a from "a"; evidence b from "b"; evidence neutral from "neutral";
a supports fact; b opposes fact; readings stream from a limit 4; event go;
on go reveal neutral; on go sample stream = 1 supports fact;
"#;
    let mut session = load(source);
    send(&mut session, "go");
    send(&mut session, "go");
    let original: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    for order in [
        json!(["a", "neutral", "stream@1", "stream@2"]),
        json!(["b", "a", "neutral", "stream@1", "stream@2"]),
        json!(["a", "b", "neutral", "stream@2", "stream@1"]),
    ] {
        let mut save = original.clone();
        save["observations"] = order;
        assert!(ReactiveSession::restore_json(source, &save.to_string()).is_err());
    }
    let unrelated = "evidence e from \"e\"; event go;";
    let mut save: Value = serde_json::from_str(&load(unrelated).save_json().unwrap()).unwrap();
    save["observations"] = json!(["e"]);
    assert!(ReactiveSession::restore_json(unrelated, &save.to_string()).is_err());
}

// Removing any one record from a featureful save must reject restoration or
// leave a usable session. This exercises the consistency boundaries jointly.
#[test]
fn featureful_save_record_removals_never_restore_an_unusable_session() {
    let source = r#"
claim fact; evidence memory from "memory"; evidence correction from "correction";
renewable memory limit 4; caveat stale consequence material;
readings samples from memory limit 8; decisions choice limit 4;
state basis = 0;
event observe; event assess; event age; event correct; event renew;
on observe reveal memory; on observe set basis = qualified(1, memory);
on observe sample samples = basis supports fact;
on assess when history_count(samples) > 0 and not committed(choice) commit choice because enough using latest(samples);
on age when observed(memory) qualify memory with stale;
on correct reveal correction;
on correct when observed(memory) withdraw memory because correction;
on renew when observed(memory) renew memory;
bind hud.seen = observed(memory);
"#;
    let mut session = load(source);
    for event in ["observe", "assess", "age", "correct", "renew", "observe"] {
        send(&mut session, event);
    }
    let original: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    fn paths(value: &Value, prefix: &str, result: &mut Vec<(String, String)>) {
        match value {
            Value::Object(fields) => {
                for (key, child) in fields {
                    result.push((prefix.into(), key.clone()));
                    paths(child, &format!("{prefix}/{key}"), result);
                }
            }
            Value::Array(items) => {
                for (index, child) in items.iter().enumerate() {
                    result.push((prefix.into(), index.to_string()));
                    paths(child, &format!("{prefix}/{index}"), result);
                }
            }
            _ => {}
        }
    }
    let mut removals = Vec::new();
    paths(&original, "", &mut removals);
    assert!(removals.len() > 100);
    for (parent, key) in removals {
        let mut edited = original.clone();
        match edited.pointer_mut(&parent).unwrap() {
            Value::Object(fields) => {
                fields.remove(&key);
            }
            Value::Array(items) => {
                items.remove(key.parse::<usize>().unwrap());
            }
            _ => unreachable!(),
        }
        if let Ok(mut restored) = ReactiveSession::restore_json(source, &edited.to_string()) {
            for event in ["observe", "assess", "age", "correct", "renew", "observe"] {
                restored
                    .dispatch_outcome_json(event, "{}")
                    .unwrap_or_else(|fatal| {
                        panic!("removed {parent}/{key}; {event}: {}", fatal.message)
                    });
                let saved = restored.save_json().unwrap();
                ReactiveSession::restore_json(source, &saved)
                    .unwrap_or_else(|error| panic!("removed {parent}/{key}; resave: {error}"));
            }
        }
    }
}

#[test]
fn scheduled_neutral_qualification_requires_its_observation_after_restore() {
    let source = r#"
    evidence memory from "memory"; caveat stale consequence material;
    event consult; event idle; event advance dt min 0 max 5; clock advance every 1;
    on consult reveal memory;
    on consult qualify memory with stale after 2;
    "#;
    let mut session = load(source);
    send(&mut session, "consult");
    send(&mut session, "idle");
    let mut edited: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    edited.as_object_mut().unwrap().remove("observations");
    assert!(ReactiveSession::restore_json(source, &edited.to_string())
        .unwrap_err()
        .contains("not observed"));
    let mut restored = resume(source, &session);
    restored.dispatch_json("advance", "{\"dt\":2}").unwrap();
    assert!(restored
        .snapshot()
        .relations
        .iter()
        .any(|edge| edge.from == "stale" && edge.to == "memory" && edge.relation == "qualifies"));
}
