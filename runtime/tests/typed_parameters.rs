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

// Positions must be whole numbers (spec/caveat-typed-parameters-0.1.md,
// "Numbers for typed parameters"). A fraction inside the range names no
// member: it is refused as input/payload_invalid and the event has no effect.
// The range check comes first, so an isolated out-of-range value keeps
// input/bound_exceeded. Plain numeric parameters are unchanged.
const POSITIONS: &str = r#"
place hall kind hall;
entity north kind door at hall;
entity south kind door at hall;
state value = 0 min -100 max 100;
event read room in attic cellar;
event knock target kind door;
event measure x min 0 max 10;
on read set value = room;
on knock set value = target;
on measure set value = x;
"#;

fn outcome(game: &mut ReactiveSession, event: &str, payload: &str) -> serde_json::Value {
    serde_json::to_value(
        game.dispatch_outcome_json(event, payload)
            .unwrap_or_else(|fatal| panic!("{event} {payload}: fatal {}", fatal.message)),
    )
    .unwrap()
}

fn value(game: &ReactiveSession) -> serde_json::Value {
    serde_json::to_value(game.snapshot()).unwrap()["values"]["value"].clone()
}

fn classified(outcome: &serde_json::Value) -> String {
    match outcome["outcome"].as_str() {
        Some("accepted") => "accepted".into(),
        _ => format!(
            "{}/{}",
            outcome["origin"].as_str().unwrap(),
            outcome["code"].as_str().unwrap()
        ),
    }
}

#[test]
fn names_and_whole_positions_are_accepted_as_before() {
    for (event, key, sent, expected) in [
        ("read", "room", r#""attic""#, 1.0),
        ("read", "room", r#""cellar""#, 2.0),
        ("read", "room", "1", 1.0),
        ("read", "room", "2", 2.0),
        ("read", "room", "2.0", 2.0),
        ("read", "room", "1e0", 1.0),
        ("read", "room", "0.2e1", 2.0),
        ("knock", "target", r#""south""#, 2.0),
        ("knock", "target", "1", 1.0),
        ("knock", "target", "2.0", 2.0),
    ] {
        let mut game = ReactiveSession::from_source(POSITIONS).unwrap();
        let payload = format!(r#"{{"{key}":{sent}}}"#);
        assert_eq!(
            classified(&outcome(&mut game, event, &payload)),
            "accepted",
            "{event} {payload}"
        );
        assert_eq!(value(&game), expected, "{event} {payload}");
    }
}

#[test]
fn a_fraction_inside_the_range_names_no_member_and_changes_nothing() {
    for (event, key, sent) in [
        ("read", "room", "1.5"),
        ("read", "room", "1.0000001"),
        ("read", "room", "1.999"),
        ("knock", "target", "1.5"),
        ("knock", "target", "1.25"),
    ] {
        let mut game = ReactiveSession::from_source(POSITIONS).unwrap();
        outcome(&mut game, "measure", r#"{"x":7}"#);
        let before = (game.snapshot(), game.save_json().unwrap());
        let payload = format!(r#"{{"{key}":{sent}}}"#);
        let refused = outcome(&mut game, event, &payload);
        assert_eq!(
            classified(&refused),
            "input/payload_invalid",
            "{event} {payload}"
        );
        assert!(
            refused["message"]
                .as_str()
                .unwrap()
                .contains("whole number"),
            "{refused}"
        );
        assert_eq!(
            (game.snapshot(), game.save_json().unwrap()),
            before,
            "{event} {payload}"
        );
        // The session goes on.
        assert_eq!(
            classified(&outcome(&mut game, event, &format!(r#"{{"{key}":2}}"#))),
            "accepted"
        );
        assert_eq!(value(&game), 2.0);
    }
}

#[test]
fn an_isolated_out_of_range_value_keeps_its_code() {
    for (event, key, sent) in [
        ("read", "room", "0"),
        ("read", "room", "3"),
        ("read", "room", "-1"),
        ("read", "room", "0.5"),
        ("read", "room", "2.5"),
        ("knock", "target", "0"),
        ("knock", "target", "3"),
        ("knock", "target", "2.5"),
    ] {
        let mut game = ReactiveSession::from_source(POSITIONS).unwrap();
        let payload = format!(r#"{{"{key}":{sent}}}"#);
        assert_eq!(
            classified(&outcome(&mut game, event, &payload)),
            "input/bound_exceeded",
            "{event} {payload}"
        );
    }
}

#[test]
fn a_plain_number_may_still_be_fractional() {
    let mut game = ReactiveSession::from_source(POSITIONS).unwrap();
    assert_eq!(
        classified(&outcome(&mut game, "measure", r#"{"x":1.5}"#)),
        "accepted"
    );
    assert_eq!(value(&game), 1.5);
    assert_eq!(
        classified(&outcome(&mut game, "measure", r#"{"x":10.5}"#)),
        "input/bound_exceeded"
    );
    assert_eq!(value(&game), 1.5);
}

#[test]
fn identifier_parameters_are_unchanged() {
    let source =
        "identifiers limit 2;\nstate held = 0;\nevent name who id;\non name set held = who;";
    let mut game = ReactiveSession::from_source(source).unwrap();
    assert_eq!(
        classified(&outcome(&mut game, "name", r#"{"who":"sam"}"#)),
        "accepted"
    );
    let refused = outcome(&mut game, "name", r#"{"who":1}"#);
    assert_eq!(classified(&refused), "input/payload_invalid");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("send its text"),
        "{refused}"
    );
    let mut handle = std::collections::BTreeMap::new();
    handle.insert("who".to_string(), 1.5);
    let error = game.apply("name", &handle).unwrap_err();
    assert!(error.contains("not the handle of an identifier"), "{error}");
    handle.insert("who".to_string(), 1.0);
    game.apply("name", &handle).unwrap();
}

#[test]
fn the_native_apply_path_refuses_a_fractional_position_too() {
    let mut game = ReactiveSession::from_source(POSITIONS).unwrap();
    let before = game.snapshot();
    let mut parameters = std::collections::BTreeMap::new();
    parameters.insert("room".to_string(), 1.5);
    let error = game.apply("read", &parameters).unwrap_err();
    assert!(error.contains("whole number"), "{error}");
    assert_eq!(game.snapshot(), before);
    parameters.insert("room".to_string(), 2.0);
    game.apply("read", &parameters).unwrap();
    assert_eq!(value(&game), 2.0);
}

#[test]
fn a_fractional_room_can_no_longer_reach_a_decision() {
    // The shape found by trial 07: a room reading grounds a decision, and the
    // displayed room maps every value other than 1 to the cellar.
    let source = r#"
claim here;
evidence porter from "the porter";
readings reports from porter limit 4;
decisions search limit 2;
event report room in attic cellar;
event choose;
on report sample reports = room supports here;
on choose when has_sample(reports) and not committed(search)
    commit search because enough using latest(reports);
bind choice.room = "none";
bind choice.room = if(latest(search) == 1, "attic", "cellar") when committed(search);
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    assert_eq!(
        classified(&outcome(&mut game, "report", r#"{"room":1.5}"#)),
        "input/payload_invalid"
    );
    assert_eq!(classified(&outcome(&mut game, "choose", "{}")), "accepted");
    let snapshot = serde_json::to_value(game.snapshot()).unwrap();
    assert_eq!(
        snapshot["decision_series"]["search"]["revisions"],
        serde_json::json!([])
    );
    assert_eq!(snapshot["bindings"]["choice"]["room"], "none");
    outcome(&mut game, "report", r#"{"room":"cellar"}"#);
    outcome(&mut game, "choose", "{}");
    let snapshot = serde_json::to_value(game.snapshot()).unwrap();
    assert_eq!(snapshot["commitment_bases"]["search@1"]["value"], 2.0);
    assert_eq!(snapshot["bindings"]["choice"]["room"], "cellar");
}
