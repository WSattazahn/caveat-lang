use caveat_runtime::{ast::Statement,eval::{self,EventKind},parser,Attention,NodeKind};

fn run_with(investigated:&str, action:&str) -> eval::Evaluation {
    let src = include_str!("../../game/the_door_round2.cav");
    let mut p = parser::parse(src).unwrap();
    for s in &mut p.statements {
        match s {
            Statement::Inspect { investigation, caveat, .. } if investigation == "predoor" => *caveat = investigated.into(),
            Statement::Select { choice, option } if choice == "door_decision" => *option = action.into(),
            _ => {}
        }
    }
    eval::evaluate(&p).unwrap()
}

fn attention_in_commit(e:&eval::Evaluation, action:&str, caveat:&str)->Attention {
    let commitment=e.symbols[action];
    let caveat_id=e.symbols[caveat];
    let snapshot=e.history.iter().find_map(|ev|match &ev.kind {
        EventKind::Committed{commitment:c,snapshot,..} if *c==commitment => Some(snapshot),
        _=>None
    }).unwrap();
    match snapshot.nodes.get(&caveat_id).unwrap() {
        NodeKind::Caveat{attention,..}=>*attention,
        _=>panic!("expected caveat")
    }
}

#[test]
fn latch_strategy_changes_what_was_known_at_open() {
    let e=run_with("latch_sensor_recently_serviced","open");
    assert_eq!(attention_in_commit(&e,"open","latch_sensor_recently_serviced"),Attention::Examined);
    assert_eq!(attention_in_commit(&e,"open","camera_has_blind_spot"),Attention::Unexamined);
}

#[test]
fn camera_strategy_changes_what_was_known_at_wait() {
    let e=run_with("camera_has_blind_spot","wait");
    assert_eq!(attention_in_commit(&e,"wait","camera_has_blind_spot"),Attention::Examined);
    assert_eq!(attention_in_commit(&e,"wait","latch_sensor_recently_serviced"),Attention::Unexamined);
}
