//! Compiler step 4b: an evaluation shows its stale bindings in the order of
//! their targets and properties, which load fixes, whatever order they were
//! declared in, and meets the first refusal among their lineages and
//! citations in that order. A condition or value that cannot be evaluated is
//! met before any of those, in declaration order.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const PLAY: &str = r#"
claim safe;
evidence bite from "a bite";
evidence chart from "tidal archive";
caveat faded consequence low;
state from_chart = 0;
event eat;
event chart;
event note;
event other;
on eat reveal bite supports safe;
on chart reveal chart supports safe;
on chart set from_chart = qualified(1, chart);
on note qualify bite with faded;
"#;

fn outcome(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    serde_json::to_value(session.dispatch_outcome_json(event, payload).unwrap()).unwrap()
}

/// Each binding of `body` starts to show on `note`, citing `from_chart`,
/// which its value and conditions never read: the refusal names the binding
/// `first` names.
fn refuses_first(body: &str, first: &str) {
    let mut session = ReactiveSession::from_source(&format!("{PLAY}\n{body}")).unwrap();
    for event in ["chart", "eat"] {
        assert_eq!(outcome(&mut session, event, "{}")["outcome"], "accepted");
    }
    let result = outcome(&mut session, "note", "{}");
    assert_eq!(result["outcome"], "rejected", "{result}");
    assert_eq!(result["code"], "ungrounded_citation", "{result}");
    let message = result["message"].as_str().unwrap();
    assert!(
        message.ends_with(&format!(
            "binding {first} cites evidence chart that its value and conditions never read"
        )),
        "{message}"
    );
    // The refused event left the bindings as they were.
    assert_eq!(outcome(&mut session, "other", "{}")["outcome"], "accepted");
}

#[test]
fn a_refusal_names_the_first_binding_by_target_not_by_declaration() {
    refuses_first(
        r#"
        bind zed.text = "faded" when carries(bite, faded) because from_chart;
        bind hud.text = "faded" when carries(bite, faded) because from_chart;
        "#,
        "hud.text",
    );
}

#[test]
fn a_refusal_names_the_first_binding_by_property_within_a_target() {
    refuses_first(
        r#"
        bind hud.zed = "faded" when carries(bite, faded) because from_chart;
        bind hud.alpha = "faded" when carries(bite, faded) because from_chart;
        "#,
        "hud.alpha",
    );
}

#[test]
fn a_refusal_names_the_first_binding_by_target_before_property() {
    // By target, hud.zed comes first; by property, or as declared, zed.alpha.
    refuses_first(
        r#"
        bind zed.alpha = "faded" when carries(bite, faded) because from_chart;
        bind hud.zed = "faded" when carries(bite, faded) because from_chart;
        "#,
        "hud.zed",
    );
}

#[test]
fn a_value_refusal_names_the_first_binding_by_declaration() {
    let wide = "A".repeat(65_520);
    let source = format!(
        r#"
claim safe;
evidence bite from "a bite";
evidence chart from "tidal archive";
caveat faded consequence low;
state from_chart = 0;
state big = 0 min 0 max 2000000;
event eat;
event chart;
event note;
on eat reveal bite supports safe;
on chart reveal chart supports safe;
on chart set from_chart = qualified(1, chart);
on note qualify bite with faded;
on note set big = 1234567.123456789;
bind zed.text = "{wide}" + text(big) when carries(bite, faded);
bind hud.text = "faded" when carries(bite, faded) because from_chart;
"#
    );
    let mut session = ReactiveSession::from_source(&source).unwrap();
    for event in ["chart", "eat"] {
        assert_eq!(outcome(&mut session, event, "{}")["outcome"], "accepted");
    }
    let refused = outcome(&mut session, "note", "{}");
    assert_eq!(refused["outcome"], "rejected", "{refused}");
    assert_eq!(refused["code"], "text_limit", "{refused}");
    // zed.text is declared first; hud.text, first by target, is never reached.
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .starts_with("binding zed.text: "),
        "{refused}"
    );
}

#[test]
fn values_already_shown_are_replaced_and_new_ones_added_in_place() {
    // `hud.count` is shown from load and changes on every chart; `hud.seen`
    // first shows on the second chart, on a target already shown, and stops
    // showing on eat.
    let mut session = ReactiveSession::from_source(&format!(
        "{PLAY}\nstate charts = 0;\non chart set charts = charts + 1;\n\
         bind hud.count = charts;\n\
         bind hud.seen = charts when charts > 1 and not observed(bite);\n"
    ))
    .unwrap();
    let shown = |session: &mut ReactiveSession, event: &str| {
        let result = outcome(session, event, "{}");
        assert_eq!(result["outcome"], "accepted", "{result}");
        session.snapshot().bindings
    };
    let bindings = shown(&mut session, "chart");
    assert_eq!(
        serde_json::to_value(&bindings).unwrap(),
        serde_json::json!({"hud": {"count": 1.0}})
    );
    let bindings = shown(&mut session, "chart");
    assert_eq!(
        serde_json::to_value(&bindings).unwrap(),
        serde_json::json!({"hud": {"count": 2.0, "seen": 2.0}})
    );
    let bindings = shown(&mut session, "eat");
    assert_eq!(
        serde_json::to_value(&bindings).unwrap(),
        serde_json::json!({"hud": {"count": 2.0}})
    );
}
