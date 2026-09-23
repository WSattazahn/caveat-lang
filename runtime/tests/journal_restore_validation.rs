//! Saved history is checked against source effects, frozen bases and graph edges.
use caveat_runtime::reactive::{ReactiveSave, ReactiveSession};

const PROGRAM: &str = r#"
claim safe;
evidence z_first from "first";
evidence a_second from "second";
evidence doubt from "visitor";
evidence later from "later";
evidence unused from "never seen";
caveat secondhand consequence low;
caveat stale consequence material;
caveat unrelated consequence low;
secondhand qualifies doubt;
decisions route limit 4;
event see;
event worry;
event age;
event recover;
event idle;
event advance dt min 0 max 30;
clock advance every 1;
proc choose(value) { commit route because enough using value; };
on see reveal z_first supports safe;
on see reveal a_second supports safe;
on see call choose(qualified(2, z_first) + qualified(3, a_second));
on worry reveal doubt opposes safe;
on worry reopen route because doubt;
on age qualify z_first with stale;
on age qualify doubt with stale;
on recover reveal later supports safe;
on recover call choose(qualified(7, later));
"#;

fn played() -> ReactiveSession {
    let mut session = ReactiveSession::from_source(PROGRAM).unwrap();
    for (event, parameters) in [
        ("see", "{}"),
        ("advance", r#"{"dt": 2}"#),
        ("worry", "{}"),
        ("age", "{}"),
        ("advance", r#"{"dt": 3}"#),
        ("recover", "{}"),
        ("idle", "{}"),
    ] {
        session.dispatch_json(event, parameters).unwrap();
    }
    session
}

fn rejected(label: &str, mutate: impl FnOnce(&mut ReactiveSave)) {
    let original = played();
    let before = original.snapshot();
    let clean = original.save().unwrap();
    let mut altered = clean.clone();
    mutate(&mut altered);
    let error = match ReactiveSession::restore(PROGRAM, &altered) {
        Ok(_) => panic!("accepted invalid history: {label}"),
        Err(error) => error,
    };
    assert!(error.contains("cannot restore save"), "{label}: {error}");
    // Restore constructs a new session. Rejecting it cannot damage either the
    // active session or the clean save that the host may retry.
    assert_eq!(original.snapshot(), before, "{label}");
    assert_eq!(
        ReactiveSession::restore(PROGRAM, &clean)
            .unwrap()
            .snapshot(),
        before,
        "{label}"
    );
}

#[test]
fn frozen_caveats_and_procedure_decisions_restore_without_becoming_live() {
    let mut original = played();
    let before = original.snapshot();
    assert_eq!(before.decision_journal[0].because, ["z_first", "a_second"]);
    assert!(before.decision_journal[0].caveats.is_empty());
    assert_eq!(before.decision_journal[1].caveats, ["secondhand"]);
    let mut restored =
        ReactiveSession::restore_json(PROGRAM, &original.save_json().unwrap()).unwrap();
    assert_eq!(restored.snapshot(), before);
    for event in ["worry", "recover", "idle"] {
        assert_eq!(
            restored.dispatch_json(event, "{}"),
            original.dispatch_json(event, "{}")
        );
    }
}

#[test]
fn legacy_journal_entries_without_time_or_value_remain_valid() {
    let mut json = serde_json::to_value(played().save().unwrap()).unwrap();
    for entry in json["decision_journal"].as_array_mut().unwrap() {
        entry.as_object_mut().unwrap().remove("elapsed");
        entry.as_object_mut().unwrap().remove("value");
    }
    let mut restored = ReactiveSession::restore_json(PROGRAM, &json.to_string()).unwrap();
    assert!(restored
        .snapshot()
        .decision_journal
        .iter()
        .all(|entry| entry.elapsed.is_none() && entry.value.is_none()));
    let saved = restored.save_json().unwrap();
    assert_eq!(
        ReactiveSession::restore_json(PROGRAM, &saved)
            .unwrap()
            .snapshot(),
        restored.snapshot()
    );
    restored.dispatch_json("worry", "{}").unwrap();
    let new_entry = restored.snapshot().decision_journal.pop().unwrap();
    assert_eq!(new_entry.elapsed, Some(5.0));
    assert_eq!(new_entry.value, Some(7.0));
}

#[test]
fn fabricated_names_kinds_and_historical_caveats_are_rejected() {
    rejected("unknown evidence", |save| {
        save.decision_journal[0].because[0] = "ghost".into()
    });
    rejected("unobserved evidence", |save| {
        save.decision_journal[0].because[0] = "unused".into()
    });
    rejected("unknown caveat", |save| {
        save.decision_journal[1].caveats.push("ghost".into())
    });
    rejected("unrelated caveat", |save| {
        save.decision_journal[1].caveats = vec!["unrelated".into()]
    });
    rejected("future caveat on frozen commitment", |save| {
        save.decision_journal[0].caveats = vec!["stale".into()]
    });
    rejected("unknown commitment", |save| {
        save.decision_journal[0].commitment = "ghost".into()
    });
    rejected("wrong decision", |save| {
        save.decision_journal[0].decision = "other".into()
    });
    rejected("unknown change", |save| {
        save.decision_journal[0].change = "rescued".into()
    });
}

#[test]
fn impossible_event_sequence_and_order_are_rejected() {
    rejected("unknown event", |save| {
        save.decision_journal[0].event = "ghost".into()
    });
    rejected("known event cannot commit", |save| {
        save.decision_journal[0].event = "idle".into()
    });
    rejected("zero sequence", |save| {
        save.decision_journal[0].sequence = 0
    });
    rejected("future sequence", |save| {
        save.decision_journal[2].sequence = save.sequence + 1
    });
    rejected("changed revision sequence", |save| {
        save.decision_journal[0].sequence = 2
    });
    rejected("reopening before commitment", |save| {
        save.decision_journal.swap(0, 1)
    });
    rejected("different events at same sequence", |save| {
        save.decision_journal[1].sequence = 1
    });
    rejected("wrong last event", |save| {
        save.decision_journal[2].sequence = save.sequence;
    });
    rejected("missing last event", |save| save.last_event = None);
    rejected("evidence ordering", |save| {
        save.decision_journal[0].because.reverse()
    });
    rejected("duplicate witness", |save| {
        save.decision_journal[1].because.push("doubt".into())
    });
}

#[test]
fn fabricated_revision_or_reopening_history_is_rejected() {
    rejected("missing commitment entry", |save| {
        save.decision_journal.remove(0);
    });
    rejected("missing reopening entry", |save| {
        save.decision_journal.remove(1);
    });
    rejected("duplicated commitment", |save| {
        save.decision_journal
            .insert(1, save.decision_journal[0].clone());
    });
    rejected("repeated reopening", |save| {
        save.decision_journal
            .insert(2, save.decision_journal[1].clone());
    });
    rejected("invented reopening witness", |save| {
        save.decision_journal[1].because = vec!["z_first".into()]
    });
    rejected("empty reopening", |save| {
        save.decision_journal[1].because.clear()
    });
    rejected("closed graph after reopening", |save| {
        save.graph.open.clear()
    });
    rejected("wrong ordinal", |save| {
        save.decision_series.get_mut("route").unwrap().revisions[1].ordinal = 3
    });
    rejected("wrong predecessor", |save| {
        save.decision_series.get_mut("route").unwrap().revisions[1].previous = None
    });
    rejected("reversed revision list", |save| {
        let series = save.decision_series.get_mut("route").unwrap();
        series.revisions.reverse();
        series.current = Some(series.revisions.last().unwrap().id.clone());
    });
}

#[test]
fn nonfinite_and_inconsistent_times_or_values_are_rejected() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        rejected("nonfinite elapsed", |save| {
            save.decision_journal[1].elapsed = Some(value)
        });
        rejected("nonfinite value", |save| {
            save.decision_journal[1].value = Some(value)
        });
    }
    rejected("negative time", |save| {
        save.decision_journal[0].elapsed = Some(-1.0)
    });
    rejected("future time", |save| {
        save.decision_journal[1].elapsed = Some(6.0)
    });
    rejected("time reverses", |save| {
        save.decision_journal[2].elapsed = Some(1.0)
    });
    rejected("different frozen value", |save| {
        save.decision_journal[0].value = Some(9.0)
    });
    rejected("different reopening value", |save| {
        save.decision_journal[1].value = Some(7.0)
    });
}

#[test]
fn a_plain_commitment_and_grouped_reopening_restore() {
    let source = r#"
claim safe;
evidence z_first from "first";
evidence a_second from "second";
caveat stale consequence material;
state basis = 0;
event see;
event age;
on see reveal z_first supports safe;
on see reveal a_second supports safe;
on see set basis = qualified(1, z_first) + qualified(1, a_second);
on see commit route because enough using basis;
on age qualify z_first with stale;
on age qualify a_second with stale;
on age reopen route because caveated(basis, stale);
"#;
    let mut original = ReactiveSession::from_source(source).unwrap();
    original.dispatch_json("see", "{}").unwrap();
    original.dispatch_json("age", "{}").unwrap();
    let snapshot = original.snapshot();
    assert_eq!(snapshot.decision_journal.len(), 2);
    assert_eq!(
        snapshot.decision_journal[1].because,
        ["z_first", "a_second"]
    );
    let mut restored =
        ReactiveSession::restore_json(source, &original.save_json().unwrap()).unwrap();
    assert_eq!(restored.snapshot(), snapshot);
    assert_eq!(
        restored.dispatch_json("age", "{}"),
        original.dispatch_json("age", "{}")
    );
    assert_eq!(restored.snapshot().decision_journal.len(), 2);
}

#[test]
fn a_source_clock_that_allows_reversal_is_not_treated_as_monotonic() {
    let source = PROGRAM.replace("advance dt min 0 max 30", "advance dt min -1 max 30");
    let mut original = ReactiveSession::from_source(&source).unwrap();
    for (event, parameters) in [
        ("see", "{}"),
        ("advance", r#"{"dt": 2}"#),
        ("worry", "{}"),
        ("advance", r#"{"dt": -1}"#),
        ("recover", "{}"),
    ] {
        original.dispatch_json(event, parameters).unwrap();
    }
    let restored = ReactiveSession::restore(&source, &original.save().unwrap()).unwrap();
    assert_eq!(restored.snapshot(), original.snapshot());
}

#[test]
fn two_commitments_in_one_event_keep_their_graph_creation_order() {
    let source = "event choose; on choose commit left because enough; on choose commit right because enough;";
    let mut original = ReactiveSession::from_source(source).unwrap();
    original.dispatch_json("choose", "{}").unwrap();
    let mut save = original.save().unwrap();
    assert_eq!(
        ReactiveSession::restore(source, &save).unwrap().snapshot(),
        original.snapshot()
    );
    save.decision_journal.swap(0, 1);
    assert!(ReactiveSession::restore(source, &save).is_err());
}

#[test]
fn a_negative_source_clock_and_pending_qualification_resume_exactly() {
    let source = r#"
claim safe;
evidence sight from "scout";
caveat stale consequence material;
state basis = 0;
event advance dt min -1 max 1;
clock advance every 1;
event decide;
on decide reveal sight supports safe;
on decide set basis = qualified(1, sight);
on decide qualify sight with stale after 1.5;
on decide commit choice because enough using basis;
on advance when committed(choice) and not reopened(choice) and has_caveat(basis, stale)
    reopen choice because caveated(basis, stale);
"#;
    let mut original = ReactiveSession::from_source(source).unwrap();
    original.dispatch_json("advance", r#"{"dt": -1}"#).unwrap();
    original.dispatch_json("decide", "{}").unwrap();
    let save = original.save().unwrap();
    assert_eq!(save.elapsed, -1.0);
    assert_eq!(save.decision_journal[0].elapsed, Some(-1.0));
    assert_eq!(save.scheduled_qualifications[0].scheduled_at, -1.0);
    let mut restored =
        ReactiveSession::restore_json(source, &original.save_json().unwrap()).unwrap();
    assert_eq!(restored.snapshot(), original.snapshot());
    for dt in [-1.0, 1.0, 1.0, 0.5] {
        let parameters = format!(r#"{{"dt": {dt}}}"#);
        assert_eq!(
            restored.dispatch_json("advance", &parameters),
            original.dispatch_json("advance", &parameters)
        );
        restored = ReactiveSession::restore_json(source, &restored.save_json().unwrap()).unwrap();
        assert_eq!(restored.snapshot(), original.snapshot());
    }
    let snapshot = restored.snapshot();
    assert!(snapshot.scheduled_qualifications.is_empty());
    assert_eq!(snapshot.decision_journal.len(), 2);
    assert!(snapshot.decision_journal[0].caveats.is_empty());
    assert_eq!(snapshot.decision_journal[1].elapsed, Some(0.5));
    assert_eq!(snapshot.decision_journal[1].caveats, ["stale"]);
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut altered = save.clone();
        altered.elapsed = value;
        assert!(ReactiveSession::restore(source, &altered).is_err());
    }
    rejected("negative elapsed on a nonnegative source clock", |save| {
        save.elapsed = -1.0
    });
}
