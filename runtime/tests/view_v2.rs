//! View 0.2: ordered grounds, `reopened`, and the delta. See
//! spec/caveat-view-0.2.md. Each case checks the delta with the applier in
//! view_delta_apply, which is written from the spec, not from the builder.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

#[path = "view_delta_apply/mod.rs"]
mod view_delta_apply;
use view_delta_apply::{apply, same};

/// Seen `zs` first and `alphas` second, so first-observed order is not name
/// order (`alphas@1` sorts before `zs@1`).
const ORDERED: &str = r#"
claim warm;
evidence zed from "z"; evidence alpha from "a";
readings zs from zed limit 4; readings alphas from alpha limit 4;
decisions route limit 4; journal window 1;
event look_z; event look_a; event decide; event recheck; event idle;
on look_z sample zs = 1 supports warm;
on look_a sample alphas = 2 supports warm;
on decide when not committed(route) commit route because enough using latest(zs) + latest(alphas);
on decide when reopened(route) commit route because enough using latest(zs) + latest(alphas);
on recheck when committed(route) and not reopened(route) reopen route because latest(zs);
on idle when 1 > 2 reject "nothing to do";
bind hud.route = latest(zs) + latest(alphas) when has_sample(zs) and has_sample(alphas) because latest(zs), latest(alphas);
"#;

fn view(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.view_v2()).unwrap()
}

/// Dispatch through the delta path and check the held view stays current:
/// an accepted delta applied to it gives the full view, and a refusal
/// changes nothing. Returns the outcome.
fn step(session: &mut ReactiveSession, held: &mut Value, event: &str, payload: &str) -> Value {
    let outcome = serde_json::to_value(
        session
            .dispatch_view_delta_outcome_json(event, payload)
            .unwrap_or_else(|fatal| panic!("{event}: {fatal}")),
    )
    .unwrap();
    let full = view(session);
    match outcome["outcome"].as_str() {
        Some("accepted") => {
            let next =
                apply(held, &outcome["delta"]).unwrap_or_else(|error| panic!("{event}: {error}"));
            assert!(same(&next, &full), "{event}: applied {next}\nfull {full}");
            *held = next;
        }
        Some("rejected") => {
            assert!(outcome.get("delta").is_none());
            assert!(same(held, &full), "{event}: a refusal changed the view");
        }
        other => panic!("{event}: unexpected outcome {other:?}"),
    }
    outcome
}

fn grounds(view: &Value, commitment: &str) -> Value {
    view["commitment_grounds"][commitment]["evidence"].clone()
}

#[test]
fn grounds_list_evidence_in_first_observed_order_and_keep_it() {
    let mut session = ReactiveSession::from_source(ORDERED).unwrap();
    let mut held = view(&session);
    assert_eq!(held["schema"], "caveat-reactive-view/0.2");
    for event in ["look_z", "look_a", "decide"] {
        step(&mut session, &mut held, event, "{}");
    }
    assert_eq!(grounds(&held, "route@1"), json!(["zs@1", "alphas@1"]));
    // View 0.1 is unchanged: grounds sorted, `open` not `reopened`.
    let old = serde_json::to_value(session.view()).unwrap();
    assert_eq!(old["schema"], "caveat-reactive-view/0.1");
    assert_eq!(
        old["commitment_grounds"]["route@1"]["evidence"],
        json!(["alphas@1", "zs@1"])
    );
    assert_eq!(old["commitments"][0]["open"], false);
    assert!(held["commitments"][0].get("open").is_none());
    assert_eq!(held["commitments"][0]["reopened"], false);

    // Reopen, observe again in the other order, and decide again: the new
    // commitment has its own order and the first keeps its own, after its
    // journal entry has retired (journal window 1) and through a save.
    for event in ["recheck", "look_a", "look_z", "decide"] {
        step(&mut session, &mut held, event, "{}");
    }
    assert_eq!(held["commitments"][0]["reopened"], true);
    assert_eq!(grounds(&held, "route@1"), json!(["zs@1", "alphas@1"]));
    assert_eq!(grounds(&held, "route@2"), json!(["alphas@2", "zs@2"]));
    assert_eq!(held["decision_journal"].as_array().unwrap().len(), 1);
    let save: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    assert_eq!(
        save["evidence_order"],
        json!({"route@1": ["zs@1", "alphas@1"]})
    );
    let restored = ReactiveSession::restore_json(ORDERED, &save.to_string()).unwrap();
    assert!(same(&view(&restored), &held));
    assert_eq!(restored.save_json().unwrap(), session.save_json().unwrap());
}

#[test]
fn an_older_save_takes_the_order_its_journal_entry_gives_and_never_invents_one() {
    let mut session = ReactiveSession::from_source(ORDERED).unwrap();
    for event in ["look_z", "look_a", "decide"] {
        session.dispatch_outcome_json(event, "{}").unwrap();
    }
    let mut save: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    // This save keeps the entry, so the order comes from it.
    save.as_object_mut().unwrap().remove("evidence_order");
    let restored = ReactiveSession::restore_json(ORDERED, &save.to_string()).unwrap();
    assert_eq!(
        grounds(&view(&restored), "route@1"),
        json!(["zs@1", "alphas@1"])
    );
    assert_eq!(restored.save_json().unwrap(), session.save_json().unwrap());

    // With the entry retired and no recorded order, name order it is.
    for event in ["recheck", "look_a", "decide"] {
        session.dispatch_outcome_json(event, "{}").unwrap();
    }
    let mut save: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    assert!(save["evidence_order"].get("route@1").is_some());
    save.as_object_mut().unwrap().remove("evidence_order");
    let restored = ReactiveSession::restore_json(ORDERED, &save.to_string()).unwrap();
    assert_eq!(
        grounds(&view(&restored), "route@1"),
        json!(["alphas@1", "zs@1"])
    );
}

#[test]
fn a_recorded_order_must_reorder_exactly_its_grounds() {
    let mut session = ReactiveSession::from_source(ORDERED).unwrap();
    for event in ["look_z", "look_a", "decide"] {
        session.dispatch_outcome_json(event, "{}").unwrap();
    }
    let save: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    for (order, why) in [
        (
            json!({"route@1": ["alphas@1", "zs@1"]}),
            "name order is never recorded",
        ),
        (json!({"route@1": ["zs@1"]}), "a name is missing"),
        (
            json!({"route@1": ["zs@1", "alphas@1", "zs@1"]}),
            "a name repeats",
        ),
        (
            json!({"route@1": ["zs@1", "zs@1"]}),
            "a name repeats in place of another",
        ),
        (
            json!({"route@1": ["zs@1", "other@1"]}),
            "a name is not in the grounds",
        ),
        (json!({"route@9": ["zs@1", "alphas@1"]}), "no such grounds"),
    ] {
        let mut altered = save.clone();
        altered["evidence_order"] = order;
        let error = ReactiveSession::restore_json(ORDERED, &altered.to_string())
            .err()
            .unwrap_or_else(|| panic!("restored although {why}"));
        assert!(error.contains("evidence_order"), "{why}: {error}");
    }
}

#[test]
fn named_cases_keep_a_host_view_current() {
    let mut session = ReactiveSession::from_source(ORDERED).unwrap();
    let mut held = view(&session);

    // An accepted event.
    let outcome = step(&mut session, &mut held, "look_z", "{}");
    assert_eq!(outcome["delta"]["since"], 0);
    assert_eq!(outcome["delta"]["sequence"], 1);
    assert_eq!(
        outcome["delta"]["relations"]["appended"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    // An explanation-only change: the binding cannot show yet (no alphas), so
    // look_a both sets the binding and its explanation.
    let outcome = step(&mut session, &mut held, "look_a", "{}");
    assert!(outcome["delta"]["binding_explanations"]["set"]["hud"]["route"].is_object());

    // A refusal: the same as dispatch_outcome reports, with no delta.
    let mut twin = session.clone();
    let refused = step(&mut session, &mut held, "nonexistent", "{}");
    let expected =
        serde_json::to_value(twin.dispatch_outcome_json("nonexistent", "{}").unwrap()).unwrap();
    assert_eq!(refused, expected);

    // An accepted event that changes nothing else.
    let outcome = step(&mut session, &mut held, "idle", "{}");
    let delta = &outcome["delta"];
    assert_eq!(delta["bindings"], json!({"set": {}, "removed": []}));
    assert_eq!(delta["commitments"], json!({"set": [], "removed": []}));
    assert_eq!(
        delta["decision_journal"],
        json!({"removed": [], "appended": []})
    );
    assert_eq!(delta["relations"], json!({"removed": [], "appended": []}));

    // A commitment, a reopening, and a journal window retiring an entry.
    for event in ["decide", "recheck", "look_z", "decide"] {
        step(&mut session, &mut held, event, "{}");
    }

    // A sequence mismatch: a delta for another view does not apply, and the
    // host takes a full view instead.
    let stale = held.clone();
    let outcome = step(&mut session, &mut held, "look_a", "{}");
    let mut wrong = stale.clone();
    wrong["sequence"] = json!(0);
    assert!(apply(&wrong, &outcome["delta"]).is_err());

    // A restore: the restored session's full view is the held view, and its
    // deltas continue from it.
    let mut restored =
        ReactiveSession::restore_json(ORDERED, &session.save_json().unwrap()).unwrap();
    assert!(same(&view(&restored), &held));
    step(&mut restored, &mut held, "look_z", "{}");
}

/// The program of tests/binding_explanations.rs's citation case: a caveat
/// learned late on the clock, a renewal, a withdrawal, a reopening and
/// explanation changes.
const PLAY: &str = r#"
claim safe;
evidence bite from "a bite"; evidence chart from "tidal archive";
evidence gauge from "live instrument"; evidence recheck from "a second look";
caveat faded consequence low;
renewable bite limit 4; readings flow from gauge limit 8; decisions route limit 4;
state from_bite = 0; state from_chart = 0; state derived = 0;
event tick dt min 0 max 0.1; event eat; event regrow; event chart; event note;
event read x min 0 max 9; event decide; event other;
on eat reveal bite supports safe;
on eat set from_bite = from_bite + qualified(1, bite);
on chart reveal chart supports safe;
on chart set from_chart = qualified(1, chart);
on eat qualify bite with faded after 0.15;
on regrow renew bite;
on note qualify chart with faded;
on note reveal recheck supports safe;
on note withdraw bite because recheck;
on chart set derived = from_chart * 2;
on read sample flow = x supports safe;
on read when committed(route) and not reopened(route) reopen route because latest(flow);
on decide when not committed(route) commit route because enough using latest(flow);
on decide when reopened(route) commit route because enough using latest(flow);
bind hud.bite = from_bite when from_bite > 0 because from_bite;
bind hud.text = "fresh" because nothing;
bind hud.text = "faded" when carries(bite, faded) because carries(bite, faded);
bind hud.value = derived when derived > 0 because from_chart;
bind hud.route = "decided" when committed(route) because committed(route);
bind hud.open = "open" when reopened(route) because reopened(route), latest(flow);
bind hud.reading = latest(flow) when has_sample(flow) because latest(flow);
bind hud.withdrawn = "withdrawn" when withdrawn(bite) because withdrawn(bite);
"#;

#[test]
fn deltas_stay_exact_through_late_qualification_departure_and_collection() {
    let mut session = ReactiveSession::from_source(PLAY).unwrap();
    let mut held = view(&session);
    for (event, payload) in [
        ("eat", "{}"),
        ("chart", "{}"),
        ("read", r#"{"x": 1}"#),
        ("decide", "{}"),
        ("tick", r#"{"dt": 0.1}"#),
        ("regrow", "{}"),
        ("eat", "{}"),
        ("tick", r#"{"dt": 0.1}"#),
        ("tick", r#"{"dt": 0.1}"#),
        ("read", r#"{"x": 2}"#),
        ("decide", "{}"),
        ("read", r#"{"x": 3}"#),
        ("note", "{}"),
        ("chart", "{}"),
        ("other", "{}"),
    ] {
        let outcome = step(&mut session, &mut held, event, payload);
        assert_eq!(outcome["outcome"], "accepted", "{event}");
    }
    assert_eq!(held["bindings"]["hud"]["text"], "faded");

    for (source, events) in [
        // Departure from a reading window, with the journal window.
        (
            r#"
claim seen; evidence glimpse from "glimpse";
readings grants from glimpse window 1;
decisions trust limit 4; journal window 1;
event look; event decide;
on look sample grants = 1 supports seen;
on decide when committed(trust) and not reopened(trust) reopen trust because latest(grants);
on decide commit trust because enough permitted by latest(grants);
"#,
            vec![
                "look", "decide", "look", "decide", "look", "decide", "decide",
            ],
        ),
        // Withdrawal collection.
        (
            include_str!("../../experiments/departure-gate/withdrawal-fixtures/self.cav"),
            vec!["cycle"; 6],
        ),
    ] {
        let mut session = ReactiveSession::from_source(source).unwrap();
        let mut held = view(&session);
        for event in events {
            step(&mut session, &mut held, event, "{}");
        }
        assert!(held["sequence"].as_u64().unwrap() > 0);
    }
}

#[test]
fn corrupted_deltas_are_detected() {
    let mut session = ReactiveSession::from_source(ORDERED).unwrap();
    let held = view(&session);
    let outcome = serde_json::to_value(
        session
            .dispatch_view_delta_outcome_json("look_z", "{}")
            .unwrap(),
    )
    .unwrap();
    let full = view(&session);
    let delta = &outcome["delta"];
    assert!(same(&apply(&held, delta).unwrap(), &full));

    // A dropped `set` entry.
    let mut session2 = ReactiveSession::from_source(ORDERED).unwrap();
    let mut held2 = view(&session2);
    for event in ["look_z", "look_a"] {
        let outcome = serde_json::to_value(
            session2
                .dispatch_view_delta_outcome_json(event, "{}")
                .unwrap(),
        )
        .unwrap();
        let mut corrupted = outcome["delta"].clone();
        if event == "look_a" {
            assert!(!corrupted["bindings"]["set"].as_object().unwrap().is_empty());
            corrupted["bindings"]["set"] = json!({});
            assert!(apply(&held2, &corrupted).map_or(true, |next| !same(&next, &view(&session2))));
        }
        held2 = apply(&held2, &outcome["delta"]).unwrap();
    }

    // A wrong position in `removed`, and a stale `since`.
    let mut wrong = delta.clone();
    wrong["relations"]["removed"] = json!([0]);
    assert!(apply(&held, &wrong).map_or(true, |next| !same(&next, &full)));
    let mut stale = delta.clone();
    stale["since"] = json!(7);
    assert!(apply(&held, &stale).is_err());
}
