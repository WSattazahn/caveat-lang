//! D052: restore accepted an observation record on evidence only the
//! program's declarations observe. Only an event's reveal, sample or
//! renewal leaves an observation record, so evidence a declaration relates
//! to a claim has none until a rule reveals it with a stance its
//! declarations do not give it (D043). A save could hold one on evidence no
//! rule or procedure reveals, since the record fit any evidence that is
//! observed, and the restored session then held a record no session of the
//! program holds, which later reads of that evidence carry. Restore now
//! checks an observation record's key against the `reveal`, `sample` and
//! `renew` effects a rule reaches, directly or through the procedures it
//! calls. A reveal of declared evidence with its declared stance, or a
//! neutral one, leaves no record on the evidence's own name. A renewal
//! leaves one on the occurrence it makes only when its guard holds
//! something, and only a condition or call argument on its way that reads a
//! state, the graph or a history can put anything there: a payload field, a
//! constant, the clock and a literal carry no lineage. Conditions are not
//! evaluated, so a record on a name these effects leave records on restores
//! whether or not an event left it there. Every record a session writes
//! restores as before: on evidence a rule reveals with a stance no
//! declaration gives it, on evidence no declaration relates to a claim that
//! a rule reveals neutrally, on the occurrences reveals and renewals leave
//! records on, and on readings. See spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// Sends `event`, which may give its payload after a space, and expects it
/// accepted.
fn send(session: &mut ReactiveSession, event: &str) -> Value {
    let (event, payload) = event.split_once(' ').unwrap_or((event, "{}"));
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    outcome
}

fn saved(session: &ReactiveSession) -> Value {
    serde_json::from_str(&session.save_json().unwrap()).unwrap()
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted {}", save["observation_qualifications"]),
    }
}

/// The session restored from `live`'s own save is `live`'s, and stays so as
/// each of `events` goes to both.
fn plays_on_alike(source: &str, live: &mut ReactiveSession, events: &[&str]) {
    let save = live.save_json().unwrap();
    let mut restored = ReactiveSession::restore_json(source, &save)
        .unwrap_or_else(|error| panic!("restore refused the runtime's own save: {error}"));
    assert_eq!(restored.save_json().unwrap(), save);
    assert_eq!(restored.snapshot(), live.snapshot());
    for event in events {
        assert_eq!(send(&mut restored, event), send(live, event), "{event}");
        assert_eq!(
            restored.save_json().unwrap(),
            live.save_json().unwrap(),
            "{event}"
        );
        assert_eq!(restored.snapshot(), live.snapshot(), "{event}");
    }
}

#[test]
fn an_observation_record_needs_a_rule_that_observes_its_evidence() {
    // The finding's program. `sensor` is observed as the program loads, by
    // its declaration, and no rule reveals it; `go` reveals `other` and
    // examines `stale`.
    let source = "budget 12;\n\
                  claim safe;\n\
                  evidence sensor from \"sensor\";\n\
                  evidence other from \"o\";\n\
                  caveat stale consequence material;\n\
                  sensor supports safe;\n\
                  event go;\n\
                  on go reveal other supports safe;\n\
                  on go examine stale cost 1;\n";
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut save = saved(&live);
    assert_eq!(save["observation_qualifications"], Value::Null);
    save["observation_qualifications"]["sensor"] = json!({});
    refused(
        source,
        &save,
        "observation of sensor: no rule or procedure of this program reveals sensor",
    );
    plays_on_alike(source, &mut live, &[]);
    // `go` leaves `other`'s record, and the save restores; a record on
    // `sensor` is still refused, whatever it holds.
    send(&mut live, "go");
    let mut save = saved(&live);
    assert_eq!(save["observation_qualifications"], json!({"other": {}}));
    plays_on_alike(source, &mut live, &["go"]);
    save["observation_qualifications"]["sensor"] = json!({"evidence": ["other"]});
    refused(
        source,
        &save,
        "observation of sensor: no rule or procedure of this program reveals sensor",
    );
}

#[test]
fn a_rule_that_reveals_declared_evidence_leaves_a_record_that_restores() {
    // `sensor` is observed as the program loads. Once `peek` has revealed
    // `other`, `go` reveals `sensor` with a stance the declaration does not
    // give it, which leaves its record, and `count` reads `sensor`.
    let source = "claim safe;\n\
                  claim calm;\n\
                  evidence sensor from \"sensor\";\n\
                  evidence other from \"o\";\n\
                  sensor supports safe;\n\
                  state seen = 0;\n\
                  event peek;\n\
                  event go;\n\
                  event count;\n\
                  on peek reveal other supports calm;\n\
                  on go when observed(other) reveal sensor supports calm;\n\
                  on count set seen = seen + qualified(1, sensor);\n";
    let mut live = ReactiveSession::from_source(source).unwrap();
    send(&mut live, "peek");
    send(&mut live, "go");
    let save = saved(&live);
    assert_eq!(
        save["observation_qualifications"],
        json!({"other": {}, "sensor": {"evidence": ["other"]}})
    );
    plays_on_alike(source, &mut live, &["count", "peek", "go", "count"]);
    // What `seen` rests on includes the guard the record holds.
    let snapshot = serde_json::to_value(live.snapshot()).unwrap();
    assert_eq!(
        snapshot["qualified_values"]["seen"]["provenance"]["evidence"],
        json!(["other", "sensor"])
    );
}

/// Renewable `sensor` is observed as the program loads, and `peek` reveals
/// `other`. `renew` makes a new occurrence of `sensor` once `other` is
/// observed, so its guard leaves a record on that occurrence; `refresh`
/// makes one unguarded, which leaves none. `go`, when the program has it,
/// reveals the current occurrence.
fn renewal(go: &str) -> String {
    format!(
        "claim safe;\n\
         evidence sensor from \"sensor\";\n\
         evidence other from \"o\";\n\
         renewable sensor limit 4;\n\
         sensor supports safe;\n\
         event peek;\n\
         event renew;\n\
         event refresh;\n\
         event go;\n\
         on peek reveal other supports safe;\n\
         on renew when observed(other) renew sensor;\n\
         on refresh renew sensor;\n\
         {go}"
    )
}

#[test]
fn a_renewal_occurrence_keeps_its_record() {
    // No rule reveals `sensor`: the record on `sensor@2` is the renewal's.
    let source = renewal("");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "peek");
    send(&mut live, "renew");
    let save = saved(&live);
    assert_eq!(save["renewals"], json!({"sensor": ["sensor", "sensor@2"]}));
    assert_eq!(
        save["observation_qualifications"],
        json!({"other": {}, "sensor@2": {"evidence": ["other"]}})
    );
    plays_on_alike(&source, &mut live, &["renew", "refresh"]);
    // A renewal leaves its record on the occurrence it makes, never on the
    // evidence it renews.
    let mut forged = saved(&ReactiveSession::from_source(&source).unwrap());
    forged["observation_qualifications"]["sensor"] = json!({});
    refused(
        &source,
        &forged,
        "observation of sensor: no rule or procedure of this program reveals sensor",
    );

    // `refresh` leaves no record on `sensor@2`; `go` reveals it, which does.
    let source = renewal("on go when observed(other) reveal sensor supports safe;\n");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "peek");
    send(&mut live, "refresh");
    assert_eq!(
        saved(&live)["observation_qualifications"],
        json!({"other": {}})
    );
    send(&mut live, "go");
    let save = saved(&live);
    assert_eq!(
        save["observation_qualifications"],
        json!({"other": {}, "sensor@2": {"evidence": ["other"]}})
    );
    plays_on_alike(&source, &mut live, &["renew", "go"]);
}

/// `sensor` is observed as the program loads and `peek` reveals `other`.
/// `go` calls `look` with the evidence it names, and `look` reveals that
/// evidence once `other` is observed.
fn procedure(looked: &str) -> String {
    format!(
        "claim safe;\n\
         claim calm;\n\
         evidence sensor from \"sensor\";\n\
         evidence other from \"o\";\n\
         sensor supports safe;\n\
         event peek;\n\
         event go;\n\
         proc look(e evidence) {{\n\
             when observed(other) reveal e supports calm;\n\
         }};\n\
         on peek reveal other supports safe;\n\
         on go call look({looked});\n"
    )
}

#[test]
fn a_reveal_in_a_procedure_a_rule_calls_leaves_a_record_that_restores() {
    let source = procedure("sensor");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "peek");
    send(&mut live, "go");
    let save = saved(&live);
    assert_eq!(
        save["observation_qualifications"],
        json!({"other": {}, "sensor": {"evidence": ["other"]}})
    );
    plays_on_alike(&source, &mut live, &["go", "peek"]);
    // Called with `other`, `look` reveals `other` alone.
    let source = procedure("other");
    let mut save = saved(&ReactiveSession::from_source(&source).unwrap());
    save["observation_qualifications"]["sensor"] = json!({});
    refused(
        &source,
        &save,
        "observation of sensor: no rule or procedure of this program reveals sensor",
    );
}

#[test]
fn a_sample_leaves_its_record_on_its_reading() {
    // The stream's template `probe` is observed as the program loads, and
    // `take` samples the stream once `peek` has revealed `other`.
    let source = "claim safe;\n\
                  evidence probe from \"probe\";\n\
                  evidence other from \"o\";\n\
                  probe supports safe;\n\
                  readings temps from probe limit 4;\n\
                  event peek;\n\
                  event take;\n\
                  on peek reveal other supports safe;\n\
                  on take when observed(other) sample temps = 3 supports safe;\n";
    let mut live = ReactiveSession::from_source(source).unwrap();
    send(&mut live, "peek");
    send(&mut live, "take");
    let save = saved(&live);
    let records = save["observation_qualifications"].as_object().unwrap();
    assert_eq!(
        records.keys().collect::<Vec<_>>(),
        ["other", "temps@1"],
        "{records:?}"
    );
    plays_on_alike(source, &mut live, &["take", "peek"]);
    // The record goes on the reading, never on the template.
    let mut forged = save.clone();
    forged["observation_qualifications"]["probe"] = json!({});
    refused(
        source,
        &forged,
        "observation of probe: no rule or procedure of this program reveals probe",
    );
}

/// `sensor` is observed as the program loads, by its declaration that it
/// supports `safe`, and `peek` reveals `other`. Once `other` is observed,
/// `go` reveals `sensor` with `stance`, or neutrally when it is empty.
/// `declarations` and `rules` add to the program.
fn declared_sensor(stance: &str, declarations: &str, rules: &str) -> String {
    format!(
        "claim safe;\n\
         evidence sensor from \"sensor\";\n\
         evidence other from \"o\";\n\
         {declarations}\
         sensor supports safe;\n\
         event peek;\n\
         event go;\n\
         event refresh;\n\
         on peek reveal other supports safe;\n\
         on go when observed(other) reveal sensor{stance};\n\
         {rules}"
    )
}

const NO_NEW_STANCE: &str = "observation of sensor: no rule or procedure of this program \
                             reveals sensor with a stance no declaration gives it";

#[test]
fn a_reveal_that_gives_declared_evidence_no_new_stance_leaves_no_record() {
    // A reveal with the stance the declaration gives `sensor` finds that
    // relation already there, and a neutral one finds `sensor` already
    // observed, so neither leaves a record.
    for stance in [" supports safe", ""] {
        let source = declared_sensor(stance, "", "");
        let mut live = ReactiveSession::from_source(&source).unwrap();
        let mut save = saved(&live);
        save["observation_qualifications"]["sensor"] = json!({});
        refused(&source, &save, NO_NEW_STANCE);
        send(&mut live, "peek");
        send(&mut live, "go");
        let mut save = saved(&live);
        assert_eq!(
            save["observation_qualifications"],
            json!({"other": {}}),
            "{stance}"
        );
        plays_on_alike(&source, &mut live, &["go", "peek"]);
        save["observation_qualifications"]["sensor"] = json!({"evidence": ["other"]});
        refused(&source, &save, NO_NEW_STANCE);
    }
}

#[test]
fn a_reveal_with_a_stance_of_its_own_leaves_a_record_on_declared_evidence() {
    // The declaration has `sensor` support `safe`; opposing it is a stance
    // of the rule's own, which leaves a record.
    let source = declared_sensor(" opposes safe", "", "");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    send(&mut live, "peek");
    send(&mut live, "go");
    assert_eq!(
        saved(&live)["observation_qualifications"],
        json!({"other": {}, "sensor": {"evidence": ["other"]}})
    );
    plays_on_alike(&source, &mut live, &["go", "peek"]);
}

#[test]
fn a_neutral_reveal_of_undeclared_evidence_leaves_a_record_that_restores() {
    // No declaration relates `plain` to a claim, so it is not observed until
    // `look` reveals it, neutrally, which leaves its record.
    let source = "claim safe;\n\
                  evidence plain from \"p\";\n\
                  event look;\n\
                  on look reveal plain;\n";
    let mut live = ReactiveSession::from_source(source).unwrap();
    send(&mut live, "look");
    assert_eq!(
        saved(&live)["observation_qualifications"],
        json!({"plain": {}})
    );
    plays_on_alike(source, &mut live, &["look"]);
}

#[test]
fn a_neutral_reveal_of_evidence_a_declaration_relates_to_no_claim_leaves_a_record() {
    // A declared `reopen` relates `sensor` to the commitment `hold`, not to
    // a claim, so `sensor` is not observed as the program loads, and a
    // neutral reveal leaves a record on it that restores.
    let source = "claim safe;\n\
                  evidence sensor from \"sensor\";\n\
                  commit hold because enough;\n\
                  reopen hold because sensor;\n\
                  event see;\n\
                  on see reveal sensor;\n";
    let mut live = ReactiveSession::from_source(source).unwrap();
    send(&mut live, "see");
    let mut save = saved(&live);
    assert_eq!(save["observation_qualifications"], json!({"sensor": {}}));
    plays_on_alike(source, &mut live, &["see"]);
    // An event observed `sensor`, so its record has to be there.
    save["observation_qualifications"]
        .as_object_mut()
        .unwrap()
        .remove("sensor");
    refused(
        source,
        &save,
        "observed evidence sensor has no observation record",
    );
}

#[test]
fn a_reveal_of_declared_evidence_leaves_a_record_on_a_renewed_occurrence() {
    // `refresh` renews `sensor` without a guard. The new occurrence has no
    // stance and is not observed, so `go` leaves its record there, whatever
    // stance the declaration gives `sensor`, but never on `sensor` itself.
    for stance in [" supports safe", ""] {
        let source = declared_sensor(
            stance,
            "renewable sensor limit 4;\n",
            "on refresh renew sensor;\n",
        );
        let mut live = ReactiveSession::from_source(&source).unwrap();
        send(&mut live, "peek");
        send(&mut live, "refresh");
        assert_eq!(
            saved(&live)["observation_qualifications"],
            json!({"other": {}}),
            "{stance}"
        );
        send(&mut live, "go");
        let mut save = saved(&live);
        assert_eq!(
            save["observation_qualifications"],
            json!({"other": {}, "sensor@2": {"evidence": ["other"]}}),
            "{stance}"
        );
        plays_on_alike(&source, &mut live, &["refresh", "go", "peek"]);
        save["observation_qualifications"]["sensor"] = json!({"evidence": ["other"]});
        refused(&source, &save, NO_NEW_STANCE);
    }
}

#[test]
fn a_renewal_without_a_guard_leaves_no_record() {
    // No rule reveals `sensor`, and `refresh` renews it with nothing in its
    // guard, from a rule or from a procedure the rule passes only the
    // evidence's name.
    for renews in [
        "on refresh renew sensor;\n",
        "proc again(e evidence) { renew e; };\non refresh call again(sensor);\n",
    ] {
        let source = renewal_only(renews);
        let mut live = ReactiveSession::from_source(&source).unwrap();
        send(&mut live, "peek");
        send(&mut live, "refresh");
        let mut save = saved(&live);
        assert_eq!(save["renewals"], json!({"sensor": ["sensor", "sensor@2"]}));
        assert_eq!(save["observation_qualifications"], json!({"other": {}}));
        plays_on_alike(&source, &mut live, &["refresh", "peek"]);
        save["observation_qualifications"]["sensor@2"] = json!({"evidence": ["other"]});
        refused(&source, &save, NO_GUARD);
    }
}

const NO_GUARD: &str = "observation of sensor@2: no rule or procedure of this program reveals \
                        sensor, or renews it with a guard";

#[test]
fn a_renewal_guarded_only_by_payload_fields_or_literals_leaves_no_record() {
    // Only a read of a state, the graph or a history puts anything in a
    // guard. A payload field, a literal and the clock carry no lineage, and
    // neither does a procedure's parameter bound to them, so however these
    // conditions and arguments fall, `refresh` leaves no record on the
    // occurrence it makes.
    for renews in [
        "on refresh when n > 0 renew sensor;\n",
        "on refresh when 1 > 0 renew sensor;\n",
        "on refresh when elapsed() >= 0 renew sensor;\n",
        "proc again(k) { renew sensor; };\non refresh call again(1);\n",
        "proc again(k) { renew sensor; };\non refresh call again(n);\n",
        "proc again(k) { when k > 0 renew sensor; };\non refresh call again(n);\n",
        "proc inner(m) { when m > 0 renew sensor; };\n\
         proc again(k) { call inner(k + 1); };\non refresh call again(n);\n",
    ] {
        let source = renewal_only(renews).replace("event refresh;", "event refresh n min 0 max 1;");
        let mut live = ReactiveSession::from_source(&source).unwrap();
        send(&mut live, "peek");
        send(&mut live, "refresh {\"n\": 1}");
        let mut save = saved(&live);
        assert_eq!(
            save["renewals"],
            json!({"sensor": ["sensor", "sensor@2"]}),
            "{renews}"
        );
        assert_eq!(
            save["observation_qualifications"],
            json!({"other": {}}),
            "{renews}"
        );
        plays_on_alike(
            &source,
            &mut live,
            &["refresh {\"n\": 0}", "refresh {\"n\": 1}", "peek"],
        );
        save["observation_qualifications"]["sensor@2"] = json!({"evidence": ["other"]});
        refused(&source, &save, NO_GUARD);
    }
}

/// Renewable `sensor` is observed as the program loads, `peek` reveals
/// `other` and no rule reveals `sensor`; `renews` says how `refresh`
/// renews it.
fn renewal_only(renews: &str) -> String {
    format!(
        "claim safe;\n\
         evidence sensor from \"sensor\";\n\
         evidence other from \"o\";\n\
         renewable sensor limit 4;\n\
         sensor supports safe;\n\
         event peek;\n\
         event refresh;\n\
         event retry;\n\
         on peek reveal other supports safe;\n\
         {renews}"
    )
}

#[test]
fn a_renewal_keeps_its_record_whatever_gives_it_a_guard() {
    // A condition on the step, on the rule that calls the procedure, or a
    // numeric argument the call passes, here or through a call further in,
    // puts what it reads in the renewal's guard: the graph here, or a state
    // that `peek` gives the lineage of `other`.
    for renews in [
        "proc again() { when observed(other) renew sensor; };\non refresh call again();\n",
        "proc again() { renew sensor; };\non refresh when observed(other) call again();\n",
        "proc again(n) { renew sensor; };\non refresh call again(qualified(1, other));\n",
        "proc inner() { renew sensor; };\nproc again(n) { call inner(); };\n\
         on refresh call again(qualified(1, other));\n",
        "proc again(e evidence, n) { renew e; };\n\
         on refresh call again(sensor, qualified(1, other));\n",
        // The same procedure, called with a guard and without one.
        "proc again() { renew sensor; };\non refresh when observed(other) call again();\n\
         on retry call again();\n",
        "state level = 0;\non peek set level = qualified(1, other);\n\
         on refresh when level > 0 renew sensor;\n",
        "state level = 0;\non peek set level = qualified(1, other);\n\
         proc again(n) { when n > 0 renew sensor; };\non refresh call again(level);\n",
    ] {
        let source = renewal_only(renews);
        let mut live = ReactiveSession::from_source(&source).unwrap();
        send(&mut live, "peek");
        send(&mut live, "refresh");
        assert_eq!(
            saved(&live)["observation_qualifications"],
            json!({"other": {}, "sensor@2": {"evidence": ["other"]}}),
            "{renews}"
        );
        plays_on_alike(&source, &mut live, &["refresh", "peek"]);
    }
}

#[test]
fn a_guarded_renewal_after_an_unguarded_one_in_the_source_keeps_its_record() {
    // `retry` renews `sensor` with nothing in its guard, and comes first in
    // the source; `refresh` renews it once `other` is observed, directly or
    // through the procedure both call. The guarded renewal's record
    // restores.
    for renews in [
        "on retry renew sensor;\non refresh when observed(other) renew sensor;\n",
        "proc again() { renew sensor; };\non retry call again();\n\
         on refresh when observed(other) call again();\n",
    ] {
        let source = renewal_only(renews);
        let mut live = ReactiveSession::from_source(&source).unwrap();
        send(&mut live, "peek");
        send(&mut live, "refresh");
        assert_eq!(
            saved(&live)["observation_qualifications"],
            json!({"other": {}, "sensor@2": {"evidence": ["other"]}}),
            "{renews}"
        );
        plays_on_alike(&source, &mut live, &["retry", "refresh", "peek"]);
    }
}

#[test]
fn an_occurrence_no_rule_reveals_or_renews_takes_no_record() {
    // The save lists an occurrence of `sensor` that no rule makes, and no
    // rule reveals `sensor`, so no event could leave a record on it.
    let source = "claim safe;\n\
                  evidence sensor from \"sensor\";\n\
                  evidence other from \"o\";\n\
                  renewable sensor limit 4;\n\
                  sensor supports safe;\n\
                  event peek;\n\
                  on peek reveal other supports safe;\n";
    let mut live = ReactiveSession::from_source(source).unwrap();
    send(&mut live, "peek");
    let mut save = saved(&live);
    save["graph"]["nodes"] = json!([{"kind": "occurrence", "name": "sensor@2"}]);
    save["renewals"] = json!({"sensor": ["sensor", "sensor@2"]});
    save["observation_qualifications"]["sensor@2"] = json!({"evidence": ["other"]});
    refused(
        source,
        &save,
        "observation of sensor@2: no rule or procedure of this program reveals or renews sensor",
    );
}
