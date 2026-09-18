use caveat_runtime::eval::{self, EventKind};
use caveat_runtime::game_session::GameSession;
use caveat_runtime::{parser, Attention, NodeKind, Relation};

fn evaluate(source: &str) -> Result<eval::Evaluation, String> {
    eval::evaluate(&parser::parse(source).expect("fixture should parse"))
}

#[test]
fn all_attention_operations_reject_non_caveats_without_panicking() {
    for declaration in [
        "claim target;",
        "evidence target from sensor;",
        "commit target because enough;",
    ] {
        for operation in [
            "examine target;",
            "defer target;",
            "examine target cost 1;",
            "investigate scan options target; inspect scan target cost 1;",
        ] {
            let source = format!("budget 2; {declaration} {operation}");
            let error = evaluate(&source).expect_err("wrong node kind must be rejected");
            assert!(error.contains("target must be a caveat: target"), "{error}");
        }
    }
}

#[test]
fn direct_and_conditional_reopening_reject_non_commitments_without_panicking() {
    for declaration in [
        "claim target;",
        "evidence target from sensor;",
        "caveat target consequence low;",
    ] {
        for operation in [
            "reopen target because reason;",
            "commit trigger because enough; when_committed trigger reopen target because reason;",
        ] {
            let source = format!("evidence reason from observation; {declaration} {operation}");
            assert_eq!(
                evaluate(&source).unwrap_err(),
                "reopen target must be a commitment: target"
            );
        }
    }
}

#[test]
fn unknown_targets_keep_the_existing_name_resolution_error() {
    for source in [
        "examine missing;",
        "defer missing;",
        "budget 1; examine missing cost 1;",
        "claim reason; reopen missing because reason;",
        "claim reason; commit trigger because enough; when_committed trigger reopen missing because reason;",
    ] {
        assert_eq!(evaluate(source).unwrap_err(), "unknown symbol: missing");
    }
}

#[test]
fn an_unselected_conditional_remains_unexecuted() {
    let evaluation = evaluate(
        "claim target; evidence reason from sensor; when_committed unselected reopen target because reason;",
    ).unwrap();
    assert!(evaluation.graph.edges.is_empty());
    assert!(!evaluation
        .history
        .iter()
        .any(|event| matches!(event.kind, EventKind::Reopened { .. })));
}

#[test]
fn valid_attention_and_reopening_keep_their_budget_history_and_edges() {
    let evaluation = evaluate(
        r#"
budget 3;
caveat uncertainty consequence material;
evidence measurement from "instrument";
examine uncertainty;
defer uncertainty;
examine uncertainty cost 1;
investigate scan options uncertainty;
inspect scan uncertainty cost 1;
commit proceed because enough retaining uncertainty;
reopen proceed because measurement;
when_committed proceed reopen proceed because uncertainty;
"#,
    )
    .unwrap();
    let uncertainty = evaluation.symbols["uncertainty"];
    let proceed = evaluation.symbols["proceed"];
    assert!(matches!(
        evaluation.graph.nodes[&uncertainty],
        NodeKind::Caveat {
            attention: Attention::Examined,
            ..
        }
    ));
    assert!(matches!(
        evaluation.graph.nodes[&proceed],
        NodeKind::Commitment { open: true, .. }
    ));
    let ledger = evaluation.resources.unwrap();
    assert_eq!((ledger.initial, ledger.spent, ledger.remaining), (3, 2, 1));
    assert_eq!(evaluation.graph.relations(Relation::Reopens).count(), 2);
    assert_eq!(
        evaluation
            .history
            .iter()
            .filter(|event| matches!(event.kind, EventKind::Reopened { .. }))
            .count(),
        2
    );
}

#[test]
fn wrong_kind_effect_preserves_the_live_game_and_can_be_recovered_from() {
    let source = r#"
place entry kind room;
place exit kind room;
entity hatch kind door at entry;
connect entry to exit via hatch;
start_at entry;
action leave from entry to exit steps open hatch, through hatch;
action stay from entry to entry steps stay;
evidence sensor from "pressure sensor";
choice route options leave, stay;
select route stay;
when_committed leave reopen sensor because sensor;
"#;
    let mut game = GameSession::from_source(source).unwrap();
    let before = game.snapshot().unwrap();
    let save = game.save();
    assert_eq!(
        game.apply("leave").unwrap_err(),
        "reopen target must be a commitment: sensor"
    );
    assert_eq!(game.snapshot().unwrap(), before);
    assert_eq!(game.save(), save);
    let recovered = game.apply("stay").unwrap();
    assert_eq!(recovered.state.current_place, "entry");
    assert!(recovered.state.open_entities.is_empty());
    assert_eq!(recovered.selections, ["stay"]);
}

#[test]
fn wrong_kind_inspection_does_not_charge_attention_or_change_feedback() {
    let source = r#"
budget 2;
claim mistake;
caveat valid consequence low;
place room kind chamber;
start_at room;
action mistake from room to room steps stay;
action valid from room to room steps stay;
investigate scan options mistake, valid;
inspect scan valid cost 1;
"#;
    let mut game = GameSession::from_source(source).unwrap();
    let before = game.snapshot().unwrap();
    assert_eq!(
        game.apply("mistake").unwrap_err(),
        "investigation target must be a caveat: mistake"
    );
    assert_eq!(game.snapshot().unwrap(), before);
    let recovered = game.apply("valid").unwrap();
    assert_eq!(recovered.budget.unwrap().remaining, 1);
    assert_eq!(recovered.selections, ["valid"]);
}
