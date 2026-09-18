use caveat_runtime::game_session::{GamePending, GameSession};
use caveat_runtime::session::Session;
use caveat_runtime::web::{WebGameSession, WebSession};

const SOURCE: &str = r#"
scene "A station with an uncertain route.";
display scan "Check the instrument";
display scan_body "Choose the evidence you can afford.";
display launch_result "The gate opens.";
budget 2;
claim route_safe;
evidence recorder from "optical recorder";
caveat lens_fault consequence material;
caveat static_noise consequence low;
static_noise qualifies recorder;
place dock kind corridor;
place reactor kind chamber;
place vault kind chamber;
entity gate_a kind door at dock;
entity terminal kind console at dock;
connect dock to reactor via gate_a;
connect reactor to vault;
start_at dock;
action lens_fault from dock to dock steps inspect terminal, observe recorder;
action static_noise from dock to dock steps inspect terminal;
action launch from dock to reactor steps operate gate_a, open gate_a, through gate_a;
action wait from dock to dock steps stay;
action finish from reactor to vault steps move vault, observe final_report;
investigate scan options lens_fault, static_noise;
inspect scan lens_fault cost 1;
reveal lens_fault then recorder supports route_safe;
choice route options launch, wait retaining static_noise;
select route launch;
evidence breach_report from "pressure recorder";
evidence final_report from "vault sensor";
when_committed launch breach_report opposes route_safe;
when_committed launch reopen launch because breach_report;
choice resolution options finish retaining lens_fault;
select resolution finish;
when_committed finish final_report supports route_safe;
when_committed finish reopen launch because final_report;
"#;

#[test]
fn a_game_requires_plans_even_when_the_entire_action_layer_is_missing() {
    let source =
        "place room kind chamber; start_at room; choice exit options leave; select exit leave;";
    assert!(GameSession::from_source(source)
        .unwrap_err()
        .contains("player-facing action leave has no world action plan"));
}

#[test]
fn initial_snapshot_is_a_live_prefix_and_keeps_arbitrary_source_labels() {
    let session = GameSession::from_source(SOURCE).unwrap();
    let snapshot = session.snapshot().unwrap();
    assert_eq!(snapshot.turn, 0);
    assert_eq!(snapshot.state.current_place, "dock");
    assert!(snapshot.state.open_entities.is_empty());
    assert!(snapshot.commitments.is_empty());
    assert!(snapshot.discoveries.is_empty());
    assert!(snapshot.last_execution.is_none());
    assert_eq!(snapshot.budget.unwrap().spent, 0);
    assert!(!snapshot
        .symbols
        .iter()
        .any(|symbol| symbol.name == "breach_report"));
    assert!(!snapshot
        .relations
        .iter()
        .any(|relation| relation.relation == "supports"));
    assert_eq!(
        snapshot.labels["scan_body"],
        "Choose the evidence you can afford."
    );
    assert_eq!(snapshot.labels["launch_result"], "The gate opens.");
    assert!(matches!(
        snapshot.pending,
        GamePending::Investigate {
            cost: 1,
            budget: 2,
            ..
        }
    ));
}

#[test]
fn chosen_actions_accumulate_evidence_commitments_and_persistent_world_changes() {
    let mut session = GameSession::from_source(SOURCE).unwrap();
    let inspected = session.apply("lens_fault").unwrap();
    assert_eq!(inspected.discoveries.len(), 1);
    assert_eq!(inspected.discoveries[0].evidence, "recorder");
    assert_eq!(inspected.budget.as_ref().unwrap().remaining, 1);
    assert!(inspected.symbols.iter().any(|symbol| {
        symbol.name == "lens_fault" && symbol.attention.as_deref() == Some("examined")
    }));

    let launched = session.apply("launch").unwrap();
    assert_eq!(launched.state.current_place, "reactor");
    assert_eq!(launched.state.open_entities, ["gate_a"]);
    assert_eq!(launched.last_execution.as_ref().unwrap().commands.len(), 3);
    assert_eq!(launched.discoveries.len(), 2);
    assert_eq!(launched.commitments.len(), 1);
    assert!(launched.commitments[0].open);
    assert_eq!(launched.commitments[0].retained, ["static_noise"]);
    assert_eq!(launched.commitments[0].reopened_by, ["breach_report"]);

    let finished = session.apply("finish").unwrap();
    assert_eq!(finished.state.current_place, "vault");
    assert_eq!(finished.state.open_entities, ["gate_a"]);
    assert_eq!(finished.commitments.len(), 2);
    let launch = finished
        .commitments
        .iter()
        .find(|commitment| commitment.action == "launch")
        .unwrap();
    assert_eq!(launch.reopened_by, ["breach_report", "final_report"]);
    assert_eq!(finished.discoveries.len(), 3);
    assert_eq!(finished.turn, 3);
    assert_eq!(finished.pending, GamePending::Complete);
}

#[test]
fn future_scripted_defaults_are_not_evaluated_or_used_as_player_choices() {
    // A source placeholder is neither committed nor validated as a live choice.
    let source = SOURCE.replace("select route launch", "select route absent_default");
    let mut session = GameSession::from_source(&source).unwrap();
    session.apply("static_noise").unwrap();
    let snapshot = session.apply("launch").unwrap();
    assert_eq!(snapshot.selections, ["static_noise", "launch"]);
    assert!(snapshot
        .discoveries
        .iter()
        .all(|discovery| discovery.evidence != "recorder"));
}

#[test]
fn failed_epistemic_effect_rolls_back_opening_movement_feedback_and_history() {
    let source = SOURCE.replace(
        "reopen launch because breach_report",
        "reopen unknown_commitment because breach_report",
    );
    let mut session = GameSession::from_source(&source).unwrap();
    session.apply("lens_fault").unwrap();
    let before = session.snapshot().unwrap();
    let save_before = session.save();
    assert!(session
        .apply("launch")
        .unwrap_err()
        .contains("unknown symbol"));
    assert_eq!(session.snapshot().unwrap(), before);
    assert_eq!(session.save(), save_before);
    let snapshot = session.apply("wait").unwrap();
    assert_eq!(snapshot.state.current_place, "dock");
    assert!(snapshot.state.open_entities.is_empty());
}

#[test]
fn out_of_turn_actions_and_post_completion_actions_are_atomic() {
    let mut session = GameSession::from_source(SOURCE).unwrap();
    let initial = session.snapshot().unwrap();
    assert!(session.apply("launch").is_err());
    assert_eq!(session.snapshot().unwrap(), initial);
    for selection in ["lens_fault", "launch", "finish"] {
        session.apply(selection).unwrap();
    }
    let complete = session.snapshot().unwrap();
    assert!(session.apply("finish").is_err());
    assert_eq!(session.snapshot().unwrap(), complete);
}

#[test]
fn blocked_world_and_budget_are_distinguished_from_completion() {
    let mut session = GameSession::from_source(SOURCE).unwrap();
    session.apply("static_noise").unwrap();
    let snapshot = session.apply("wait").unwrap();
    assert!(matches!(snapshot.pending, GamePending::Blocked { .. }));
    assert!(session.apply("finish").is_err());

    let source = SOURCE.replace("budget 2", "budget 0");
    let mut session = GameSession::from_source(&source).unwrap();
    assert!(matches!(
        session.pending().unwrap(),
        GamePending::Blocked { budget: 0, .. }
    ));
    let before = session.snapshot().unwrap();
    assert!(session.apply("lens_fault").unwrap_err().contains("budget"));
    assert_eq!(session.snapshot().unwrap(), before);
}

#[test]
fn saves_replay_deterministically_from_every_interaction_boundary() {
    let mut session = GameSession::from_source(SOURCE).unwrap();
    for selection in [None, Some("lens_fault"), Some("launch"), Some("finish")] {
        if let Some(selection) = selection {
            session.apply(selection).unwrap();
        }
        let save = session.save();
        let restored = GameSession::restore(SOURCE, &save).unwrap();
        assert_eq!(restored.snapshot().unwrap(), session.snapshot().unwrap());
        assert_eq!(restored.save(), save);
        assert_eq!(
            serde_json::to_string(&restored.snapshot().unwrap()).unwrap(),
            serde_json::to_string(&session.snapshot().unwrap()).unwrap()
        );
    }
}

#[test]
fn restore_rejects_changed_source_schema_and_tampered_selection_history() {
    let session = GameSession::from_source(SOURCE).unwrap();
    let save = session.save();
    let changed_source = format!("{SOURCE}\n");
    assert!(GameSession::restore(&changed_source, &save)
        .unwrap_err()
        .contains("different CAVEAT source"));
    let mut modified: serde_json::Value = serde_json::from_str(&save).unwrap();
    modified["schema"] = "caveat-save/99".into();
    assert!(GameSession::restore(SOURCE, &modified.to_string())
        .unwrap_err()
        .contains("schema"));
    modified["schema"] = "caveat-save/0.1".into();
    modified["selections"] = serde_json::json!(["finish"]);
    assert!(GameSession::restore(SOURCE, &modified.to_string())
        .unwrap_err()
        .contains("turn 1"));
    assert!(GameSession::restore(SOURCE, "not json").is_err());
}

#[test]
fn legacy_session_preserves_selection_and_feedback_after_late_evaluation_failure() {
    let source = r#"
budget 1;
claim safe;
evidence reading from "instrument";
caveat first consequence low;
caveat second consequence low;
investigate scan options first, second;
inspect scan first cost 1;
reveal first then reading supports safe;
investigate retry options first, second;
inspect retry first cost 1;
"#;
    let mut session = Session::from_source(source).unwrap();
    session.apply("first").unwrap();
    let program = session.program().clone();
    let discoveries = session.discoveries().to_vec();
    let pending = session.pending().unwrap();
    assert!(session.apply("second").unwrap_err().contains("budget"));
    assert_eq!(session.program(), &program);
    assert_eq!(session.discoveries(), discoveries);
    assert_eq!(session.pending().unwrap(), pending);
    assert!(session.apply("unknown").is_err());
    assert_eq!(session.discoveries(), discoveries);
}

#[test]
fn web_bridge_returns_serialized_live_state_and_restores_it() {
    let mut session = WebGameSession::new(SOURCE).unwrap();
    let initial: serde_json::Value = serde_json::from_str(&session.snapshot()).unwrap();
    assert_eq!(initial["schema"], "caveat-game/0.1");
    assert_eq!(initial["pending"]["kind"], "investigate");
    let applied: serde_json::Value =
        serde_json::from_str(&session.apply("lens_fault").unwrap()).unwrap();
    assert_eq!(applied["turn"], 1);
    assert_eq!(applied["last_execution"]["action"], "lens_fault");
    let restored = WebGameSession::restore(SOURCE, &session.save()).unwrap();
    assert_eq!(restored.snapshot(), session.snapshot());

    let legacy = WebSession::new(SOURCE).unwrap();
    assert!(legacy.summary().contains("budget=2/2"));
}
