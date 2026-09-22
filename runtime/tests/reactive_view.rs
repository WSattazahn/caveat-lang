//! The per-event view: the parts of a snapshot a host redraws after an event.
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::web::WebReactiveSession;

const PROGRAM: &str = r#"
claim safe;
evidence chart from "chart";
evidence buoy from "buoy";
decisions trust limit 4;
state support = 0;
event read;
event doubt;
cue ping toast "read" 1;
on read reveal chart supports safe;
on read set support = qualified(1, chart);
on read commit trust because enough using support;
on read emit ping;
on doubt reveal buoy opposes safe;
on doubt reopen trust because buoy;
on doubt when support > 5 reject "impossible";
bind hud.text = "trusting" when committed(trust) because support;
"#;

#[test]
fn the_view_matches_the_snapshot_it_summarises() {
    let mut viewed = ReactiveSession::from_source(PROGRAM).unwrap();
    let mut full = ReactiveSession::from_source(PROGRAM).unwrap();
    for event in ["read", "doubt"] {
        let view = viewed.dispatch_view_json(event, "{}").unwrap();
        let snapshot = full.dispatch_json(event, "{}").unwrap();
        assert_eq!(view.sequence, snapshot.sequence);
        assert_eq!(view.last_event, snapshot.last_event);
        assert_eq!(view.bindings, snapshot.bindings);
        assert_eq!(view.binding_explanations, snapshot.binding_explanations);
        assert_eq!(view.cues, snapshot.cues);
        assert_eq!(view.effects, snapshot.effects);
        assert_eq!(view.commitments, snapshot.commitments);
        assert_eq!(view.commitment_grounds, snapshot.commitment_grounds);
        assert_eq!(view.decision_series, snapshot.decision_series);
        assert_eq!(view.relations, snapshot.relations);
    }
}

#[test]
fn a_failed_view_dispatch_changes_nothing() {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    game.dispatch_view_json("read", "{}").unwrap();
    let before = game.snapshot();
    assert!(game.dispatch_view_json("nowhere", "{}").is_err());
    assert!(game.dispatch_view_json("read", r#"{"x":1}"#).is_err());
    assert_eq!(game.snapshot(), before);
}

#[test]
fn the_browser_view_is_compact_json_without_static_declarations() {
    let mut session = WebReactiveSession::new(PROGRAM).unwrap();
    let text = session.dispatch_view("read", "{}").unwrap();
    assert!(!text.contains('\n'), "compact JSON");
    let view: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(view["schema"], "caveat-reactive-view/0.1");
    assert_eq!(view["bindings"]["hud"]["text"], "trusting");
    assert_eq!(
        view["binding_explanations"]["hud"]["text"]["evidence"],
        serde_json::json!(["chart"])
    );
    for absent in [
        "world",
        "symbols",
        "events",
        "qualified_values",
        "predicate_qualifications",
    ] {
        assert!(
            view.get(absent).is_none(),
            "{absent} is not part of the view"
        );
    }
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&session.view()).unwrap(),
        view
    );
}
