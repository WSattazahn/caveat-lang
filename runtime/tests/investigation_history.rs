use caveat_runtime::{
    ast::Statement,
    eval::{self, EventKind},
    parser, Attention, NodeKind, Relation,
};

fn run_with(investigation: &str, action: &str) -> eval::Evaluation {
    let mut program = parser::parse(include_str!("../../game/the_door_round2.cav")).unwrap();
    for statement in &mut program.statements {
        match statement {
            Statement::Inspect {
                investigation: name,
                caveat,
                ..
            } if name == "predoor" => *caveat = investigation.into(),
            Statement::Select { choice, option } if choice == "door_decision" => {
                *option = action.into()
            }
            _ => {}
        }
    }
    eval::evaluate(&program).unwrap()
}

fn commitment_snapshot<'a>(
    evaluation: &'a eval::Evaluation,
    action: &str,
) -> &'a eval::EpistemicSnapshot {
    let commitment = evaluation.symbols[action];
    evaluation
        .history
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::Committed {
                commitment: id,
                snapshot,
                ..
            } if *id == commitment => Some(snapshot),
            _ => None,
        })
        .expect("selected action should produce a commitment snapshot")
}

fn attention(evaluation: &eval::Evaluation, action: &str, caveat: &str) -> Attention {
    match commitment_snapshot(evaluation, action)
        .nodes
        .get(&evaluation.symbols[caveat])
        .expect("caveat should exist in commitment snapshot")
    {
        NodeKind::Caveat { attention, .. } => *attention,
        _ => panic!("attention target should be a caveat"),
    }
}

#[test]
fn latch_reveals_maintenance_not_camera_diagnostic() {
    let evaluation = run_with("latch_sensor_recently_serviced", "open");
    assert_eq!(
        attention(&evaluation, "open", "latch_sensor_recently_serviced"),
        Attention::Examined
    );
    assert_eq!(
        attention(&evaluation, "open", "camera_has_blind_spot"),
        Attention::Unexamined
    );

    let snapshot = commitment_snapshot(&evaluation, "open");
    let maintenance = evaluation.symbols["maintenance_record"];
    let diagnostic = evaluation.symbols["coverage_test"];
    assert!(snapshot
        .edges
        .iter()
        .any(|edge| edge.from == maintenance && edge.relation == Relation::Opposes));
    assert!(!snapshot
        .edges
        .iter()
        .any(|edge| edge.from == diagnostic && edge.relation == Relation::Opposes));
}

#[test]
fn camera_reveals_diagnostic_not_maintenance() {
    let evaluation = run_with("camera_has_blind_spot", "wait");
    assert_eq!(
        attention(&evaluation, "wait", "camera_has_blind_spot"),
        Attention::Examined
    );
    assert_eq!(
        attention(&evaluation, "wait", "latch_sensor_recently_serviced"),
        Attention::Unexamined
    );

    let snapshot = commitment_snapshot(&evaluation, "wait");
    let maintenance = evaluation.symbols["maintenance_record"];
    let diagnostic = evaluation.symbols["coverage_test"];
    assert!(snapshot
        .edges
        .iter()
        .any(|edge| edge.from == diagnostic && edge.relation == Relation::Opposes));
    assert!(!snapshot
        .edges
        .iter()
        .any(|edge| edge.from == maintenance && edge.relation == Relation::Opposes));
}
