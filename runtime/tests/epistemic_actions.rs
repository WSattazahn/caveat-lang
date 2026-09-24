use caveat_runtime::ast::EpistemicCondition;
use caveat_runtime::game_session::{GamePending, GameSession};
use caveat_runtime::map::CaveatMap;
#[cfg(feature = "games")]
use caveat_runtime::web::WebGameSession;

const SOURCE: &str = r#"
budget 2;
claim safe;
evidence reading from "lens recorder";
evidence contradiction from "independent alarm";
evidence later from "gate sensor";
caveat lens consequence material;
caveat noise consequence low;
place dock kind corridor;
place exit kind chamber;
entity gate kind door at dock;
entity console kind console at dock;
connect dock to exit via gate;
start_at dock;
action lens from dock to dock steps inspect console, observe reading;
action noise from dock to dock steps inspect console;
action launch from dock to exit steps open gate, through gate;
action wait from dock to dock steps stay;
require launch observed reading;
require launch examined lens;
resolve launch as premature when observed later;
resolve launch as informed when observed reading;
resolve launch as uncertain otherwise;
resolve wait as waited otherwise;
display informed "The instrument informed this departure.";
investigate scan options lens, noise;
inspect scan lens cost 1;
reveal lens then reading supports safe;
reveal lens then contradiction opposes safe;
choice departure options launch, wait retaining lens, noise;
select departure launch;
when_committed launch later opposes safe;
"#;

fn options(pending: GamePending) -> Vec<String> {
    match pending {
        GamePending::Choice { options, .. } | GamePending::Investigate { options, .. } => options,
        _ => Vec::new(),
    }
}

#[test]
fn declared_evidence_and_future_defaults_do_not_unlock_actions() {
    let source = SOURCE.replace(
        "select departure launch",
        "select departure invalid_future_default",
    );
    let mut game = GameSession::from_source(&source).unwrap();
    let first = game.snapshot().unwrap();
    assert!(first.symbols.iter().any(|symbol| symbol.name == "reading"));
    assert!(first.outcome.is_none());
    assert!(
        first.blocked_actions.is_empty(),
        "future decisions stay hidden"
    );
    let snapshot = game.apply("noise").unwrap();
    assert_eq!(options(snapshot.pending.clone()), ["wait"]);
    assert_eq!(snapshot.blocked_actions.len(), 1);
    let blocked = &snapshot.blocked_actions[0];
    assert_eq!(blocked.action, "launch");
    assert_eq!(
        blocked.reasons,
        [
            EpistemicCondition::Observed {
                symbol: "reading".into()
            },
            EpistemicCondition::Examined {
                symbol: "lens".into()
            },
        ]
    );
    assert!(!snapshot.relations.iter().any(|edge| edge.from == "reading"));
    assert!(snapshot.outcome.is_none());
}

#[test]
fn rejected_guarded_and_out_of_turn_actions_preserve_every_state_field() {
    let mut game = GameSession::from_source(SOURCE).unwrap();
    let initial = game.snapshot().unwrap();
    assert!(game.apply("launch").is_err());
    assert_eq!(game.snapshot().unwrap(), initial);
    game.apply("noise").unwrap();
    let before = game.snapshot().unwrap();
    let saved = game.save();
    assert!(game
        .apply("launch")
        .unwrap_err()
        .contains("not currently available"));
    assert_eq!(game.snapshot().unwrap(), before);
    assert_eq!(game.save(), saved);
    assert!(before.state.open_entities.is_empty());
    assert_eq!(before.state.current_place, "dock");
    assert!(before.outcome.is_none());
}

#[test]
fn all_guards_are_required_and_live_evidence_can_unlock_them() {
    let source = SOURCE.replace(
        "require launch examined lens;",
        "require launch examined noise;",
    );
    let mut game = GameSession::from_source(&source).unwrap();
    let partially_unlocked = game.apply("lens").unwrap();
    assert_eq!(options(partially_unlocked.pending), ["wait"]);
    assert_eq!(
        partially_unlocked.blocked_actions[0].reasons,
        [EpistemicCondition::Examined {
            symbol: "noise".into()
        },]
    );

    let mut game = GameSession::from_source(SOURCE).unwrap();
    let unlocked = game.apply("lens").unwrap();
    assert_eq!(options(unlocked.pending), ["launch", "wait"]);
    assert!(unlocked.blocked_actions.is_empty());
}

#[test]
fn contradiction_is_retained_and_observed_does_not_mean_supported_or_true() {
    let source = SOURCE.replace(
        "require launch observed reading;",
        "require launch observed contradiction;",
    );
    let mut game = GameSession::from_source(&source).unwrap();
    let snapshot = game.apply("lens").unwrap();
    assert_eq!(options(snapshot.pending), ["launch", "wait"]);
    assert!(snapshot
        .relations
        .iter()
        .any(|edge| edge.from == "reading" && edge.relation == "supports"));
    assert!(snapshot
        .relations
        .iter()
        .any(|edge| edge.from == "contradiction" && edge.relation == "opposes"));
    let result = game.apply("launch").unwrap();
    assert_eq!(result.discoveries.len(), 3);
    assert_eq!(result.commitments[0].retained, ["lens", "noise"]);
}

#[test]
fn resolutions_use_pre_action_knowledge_and_source_order() {
    let mut game = GameSession::from_source(SOURCE).unwrap();
    game.apply("lens").unwrap();
    let snapshot = game.apply("launch").unwrap();
    assert!(snapshot.relations.iter().any(|edge| edge.from == "later"));
    let outcome = snapshot.outcome.unwrap();
    assert_eq!(outcome.action, "launch");
    assert_eq!(
        outcome.id, "informed",
        "action's own new evidence cannot change its basis"
    );
    assert_eq!(outcome.turn, 2);
    assert_eq!(
        outcome.basis,
        [EpistemicCondition::Observed {
            symbol: "reading".into()
        }]
    );

    let source = SOURCE.replace("resolve launch as informed when observed reading;", "resolve launch as examined_first when examined lens; resolve launch as informed when observed reading;");
    let mut game = GameSession::from_source(&source).unwrap();
    game.apply("lens").unwrap();
    assert_eq!(
        game.apply("launch").unwrap().outcome.unwrap().id,
        "examined_first"
    );
}

#[test]
fn fallback_outcomes_and_selected_rule_replay_identically() {
    let source = SOURCE
        .replace("require launch observed reading;", "")
        .replace("require launch examined lens;", "");
    for (investigation, expected) in [("noise", "uncertain"), ("lens", "informed")] {
        let mut game = GameSession::from_source(&source).unwrap();
        game.apply(investigation).unwrap();
        let snapshot = game.apply("launch").unwrap();
        assert_eq!(snapshot.outcome.as_ref().unwrap().id, expected);
        assert_eq!(
            snapshot.outcome.as_ref().unwrap().basis.is_empty(),
            investigation == "noise"
        );
        assert_eq!(
            GameSession::restore(&source, &game.save())
                .unwrap()
                .snapshot()
                .unwrap(),
            snapshot
        );
        let changed = source.replace("as uncertain otherwise", "as changed otherwise");
        assert!(GameSession::restore(&changed, &game.save())
            .unwrap_err()
            .contains("different CAVEAT source"));
    }
}

#[test]
fn examined_requires_finished_attention_and_a_fully_blocked_choice_is_not_complete() {
    let source = r#"
        budget 1; caveat risk consequence high;
        place room kind chamber; start_at room;
        action depart from room to room steps stay;
        require depart examined risk;
        examine risk;
        choice decision options depart; select decision depart;
    "#;
    let game = GameSession::from_source(source).unwrap();
    assert!(matches!(
        game.pending().unwrap(),
        GamePending::Blocked { .. }
    ));
    assert_eq!(
        game.snapshot().unwrap().blocked_actions[0].reasons[0].symbol(),
        "risk"
    );
    let source = source.replace("examine risk;", "examine risk cost 1;");
    let game = GameSession::from_source(&source).unwrap();
    assert_eq!(options(game.pending().unwrap()), ["depart"]);
}

#[test]
fn malformed_references_and_ambiguous_rules_are_rejected_when_compiling() {
    let cases = [
        ("require launch observed reading;", "require absent observed reading;", "unknown action"),
        ("require launch observed reading;", "require launch observed absent;", "unknown evidence"),
        ("require launch observed reading;", "require launch observed lens;", "requires evidence"),
        ("require launch examined lens;", "require launch examined reading;", "requires caveat"),
        ("resolve launch as uncertain otherwise;", "", "exactly one otherwise"),
        ("resolve launch as uncertain otherwise;", "resolve launch as uncertain otherwise; resolve launch as duplicate otherwise;", "final resolution"),
        ("resolve launch as premature when observed later;", "resolve launch as premature otherwise;", "final resolution"),
        ("require launch observed reading;", "require launch observed reading; require launch observed reading;", "duplicate requirement"),
        ("resolve launch as informed when observed reading;", "resolve launch as informed when observed reading; resolve launch as duplicate when observed reading;", "duplicate resolution"),
        ("resolve launch as informed when observed reading;", "resolve launch as informed when examined reading;", "requires caveat"),
    ];
    for (original, replacement, diagnostic) in cases {
        let source = SOURCE.replace(original, replacement);
        let error = GameSession::from_source(&source).unwrap_err();
        assert!(error.contains(diagnostic), "{replacement}: {error}");
    }
    assert!(
        CaveatMap::from_source(&SOURCE.replace("observed reading;", "believed reading;"))
            .unwrap_err()
            .contains("line")
    );
}

#[cfg(feature = "games")]
#[test]
fn maps_describe_rules_while_web_snapshots_expose_only_reached_results() {
    let map = CaveatMap::from_source(SOURCE).unwrap();
    assert_eq!(map.requirements.len(), 2);
    assert_eq!(map.resolutions.len(), 4);
    assert_eq!(map.inspect("launch").requirements.len(), 2);
    assert_eq!(map.inspect("launch").resolutions.len(), 3);
    assert_eq!(map.inspect("reading").requirements.len(), 1);
    assert_eq!(map.inspect("informed").resolutions.len(), 1);
    let mut web = WebGameSession::new(SOURCE).unwrap();
    let initial: serde_json::Value = serde_json::from_str(&web.snapshot()).unwrap();
    assert!(initial.get("resolutions").is_none());
    assert!(initial["world"].get("resolutions").is_none());
    assert!(initial["outcome"].is_null());
    let pending: serde_json::Value = serde_json::from_str(&web.apply("noise").unwrap()).unwrap();
    assert_eq!(
        pending["blocked_actions"][0]["reasons"][0]["kind"],
        "observed"
    );
    assert_eq!(
        pending["blocked_actions"][0]["reasons"][0]["symbol"],
        "reading"
    );
    let result: serde_json::Value = serde_json::from_str(&web.apply("wait").unwrap()).unwrap();
    assert_eq!(result["outcome"]["id"], "waited");
    assert_eq!(result["outcome"]["basis"], serde_json::json!([]));
    assert_eq!(result["pending"]["kind"], "complete");
    assert_eq!(
        WebGameSession::restore(SOURCE, &web.save())
            .unwrap()
            .snapshot(),
        web.snapshot()
    );
}
