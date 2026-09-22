//! Typed event parameters: `NAME kind KIND` and `NAME in A B C`.
//! See spec/caveat-typed-parameters-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::web::WebReactiveSession;

const GARDEN: &str = r#"
place garden kind garden;
entity cave kind mushroom at garden;
entity pool kind mushroom at garden;
entity ruin kind mushroom at garden;
state heavy = 0;
event absorb target kind mushroom, sort in glowcap duskcap;
for mushroom as $m {
    state $m_consumed = 0 min 0 max 1;
    on absorb when target == $index set $m_consumed = 1;
    bind $m.present = $m_consumed == 0;
};
on absorb when sort == sort.duskcap set heavy = 20;
"#;

#[test]
fn a_host_sends_names_and_the_source_reads_positions() {
    let mut game = ReactiveSession::from_source(GARDEN).unwrap();
    let shown = game
        .dispatch_json("absorb", r#"{"target":"pool","sort":"duskcap"}"#)
        .unwrap();
    assert_eq!(shown.values["pool_consumed"], 1.0);
    assert_eq!(shown.values["cave_consumed"], 0.0);
    assert_eq!(shown.values["heavy"], 20.0);
    let shown = game
        .dispatch_json("absorb", r#"{"target":"cave","sort":"glowcap"}"#)
        .unwrap();
    assert_eq!(shown.values["cave_consumed"], 1.0);
    assert_eq!(shown.values["heavy"], 20.0);
}

#[test]
fn positions_are_counted_from_one_like_index_and_named_by_constants() {
    let mut game = ReactiveSession::from_source(&format!(
        "{GARDEN}\nstate chosen = 0;\nstate kind_chosen = 0;\n\
         on absorb set chosen = target;\non absorb set kind_chosen = sort;\n\
         bind hud.ruin = target.ruin;"
    ))
    .unwrap();
    let shown = game
        .dispatch_json("absorb", r#"{"target":"ruin","sort":"glowcap"}"#)
        .unwrap();
    assert_eq!(shown.values["chosen"], 3.0);
    assert_eq!(shown.values["kind_chosen"], 1.0);
    assert_eq!(
        serde_json::to_value(&shown.bindings["hud"]["ruin"]).unwrap(),
        3.0
    );
}

#[test]
fn a_number_is_still_accepted_and_bounds_checked() {
    let mut game = ReactiveSession::from_source(GARDEN).unwrap();
    game.dispatch_json("absorb", r#"{"target":2,"sort":1}"#)
        .unwrap();
    assert!(game
        .dispatch_json("absorb", r#"{"target":4,"sort":1}"#)
        .is_err());
}

#[test]
fn an_unknown_name_or_a_name_for_a_number_is_rejected_without_effect() {
    let mut game =
        ReactiveSession::from_source(&format!("{GARDEN}\nevent tick dt min 0 max 0.1;")).unwrap();
    let before = game.snapshot();
    let error = game
        .dispatch_json("absorb", r#"{"target":"grove","sort":"glowcap"}"#)
        .unwrap_err();
    assert!(error.contains("does not accept grove"), "{error}");
    assert!(error.contains("cave, pool, ruin"), "{error}");
    let error = game
        .dispatch_json("absorb", r#"{"target":"pool","sort":"bluecap"}"#)
        .unwrap_err();
    assert!(error.contains("glowcap, duskcap"), "{error}");
    let error = game.dispatch_json("tick", r#"{"dt":"fast"}"#).unwrap_err();
    assert!(error.contains("expects a number"), "{error}");
    assert!(game
        .dispatch_json(
            "absorb",
            r#"{"target":"pool","target":"cave","sort":"glowcap"}"#
        )
        .is_err());
    assert_eq!(game.snapshot(), before);
}

#[test]
fn the_event_signature_tells_a_host_what_to_send() {
    let game = ReactiveSession::from_source(GARDEN).unwrap();
    let json = serde_json::to_value(game.snapshot()).unwrap();
    let absorb = json["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["name"] == "absorb")
        .unwrap();
    assert_eq!(
        absorb["parameters"][0]["domain"]["entity"]["members"],
        serde_json::json!(["cave", "pool", "ruin"])
    );
    assert_eq!(absorb["parameters"][0]["max"], 3.0);
    assert_eq!(
        absorb["parameters"][1]["domain"]["member"]["members"],
        serde_json::json!(["glowcap", "duskcap"])
    );
}

#[test]
fn bad_declarations_are_rejected() {
    for (source, expected) in [
        (
            "event pick target kind boat;",
            "no entity is declared kind boat",
        ),
        ("event pick sort in;", "event parameter must be"),
        (
            "event a sort in glowcap duskcap; event b sort in duskcap glowcap;",
            "sort.duskcap already names a different value",
        ),
    ] {
        let error = ReactiveSession::from_source(source)
            .err()
            .unwrap_or_else(|| panic!("{source} should be rejected"));
        assert!(error.contains(expected), "{source}: {error}");
    }
}

#[test]
fn the_browser_accepts_names_through_dispatch_view() {
    let mut session = WebReactiveSession::new(GARDEN).unwrap();
    let view: serde_json::Value = serde_json::from_str(
        &session
            .dispatch_view("absorb", r#"{"target":"ruin","sort":"glowcap"}"#)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(view["bindings"]["ruin"]["present"], false);
}
