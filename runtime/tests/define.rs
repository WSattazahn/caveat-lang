//! `define NAME = EXPRESSION;`: a named expression over state and the graph,
//! inlined wherever it is read. See spec/caveat-define-0.1.md.
use caveat_runtime::reactive::{BindingValue, ReactiveSession};

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

const BELIEF: &str = r#"
claim safe;
evidence first from "first mushroom";
evidence second from "second mushroom";
state support = 0;
state contradiction = 0;
event good;
event bad;
on good reveal first supports safe;
on good set support = support + qualified(1, first);
on bad reveal second opposes safe;
on bad set contradiction = contradiction + qualified(1, second);
define probably_safe = support > 0 and contradiction == 0;
define uncertain = support > 0 and contradiction > 0;
bind belief.state = "none";
bind belief.state = "probably_safe" when probably_safe;
bind belief.state = "uncertain" when uncertain because contradiction;
"#;

#[test]
fn a_define_reads_like_the_expression_it_names() {
    let mut game = session(BELIEF);
    let shown = game.dispatch_json("good", "{}").unwrap();
    assert_eq!(
        shown.bindings["belief"]["state"],
        BindingValue::Text("probably_safe".into())
    );
    let shown = game.dispatch_json("bad", "{}").unwrap();
    assert_eq!(
        shown.bindings["belief"]["state"],
        BindingValue::Text("uncertain".into())
    );
    // Lineage and grounds are those of the inlined expression.
    let lineage = &shown.binding_qualifications["belief"]["state"];
    assert!(lineage.evidence.contains("first") && lineage.evidence.contains("second"));
    let cited = &shown.binding_explanations["belief"]["state"];
    assert_eq!(
        cited.evidence.iter().cloned().collect::<Vec<_>>(),
        vec!["second".to_string()]
    );
}

#[test]
fn a_define_can_use_another_define_and_a_function() {
    let mut game = session(&format!(
        "{BELIEF}\nfn doubled(x) = x * 2;\ndefine weight = doubled(support) - contradiction;\n\
         define trusted = probably_safe and weight >= 2;\n\
         bind hud.trusted = trusted;"
    ));
    let shown = game.dispatch_json("good", "{}").unwrap();
    assert_eq!(shown.bindings["hud"]["trusted"], BindingValue::Bool(true));
}

#[test]
fn a_define_works_in_rule_guards_values_and_procedures() {
    let mut game = session(&format!(
        "{BELIEF}\nstate hits = 0;\nevent check;\n\
         define score = support * 10;\n\
         proc tally() {{ when probably_safe set hits = hits + score; }};\n\
         on check when probably_safe set hits = hits + 1;\n\
         on check call tally();"
    ));
    game.dispatch_json("good", "{}").unwrap();
    let shown = game.dispatch_json("check", "{}").unwrap();
    assert_eq!(shown.values["hits"], 11.0);
}

#[test]
fn a_define_expands_per_member_in_a_for_block() {
    let mut game = session(
        r#"
        place garden kind garden;
        entity cave kind mushroom at garden;
        entity pool kind mushroom at garden;
        for mushroom as $m {
            state $m_consumed = 0 min 0 max 1;
            state $m_known = 0 min 0 max 2;
            event absorb_$m;
            define $m_unknown = $m_consumed == 0 and $m_known == 0;
            on absorb_$m set $m_consumed = 1;
            bind $m.canTaste = $m_unknown;
        };
        "#,
    );
    let shown = game.dispatch_json("absorb_pool", "{}").unwrap();
    assert_eq!(shown.bindings["cave"]["canTaste"], BindingValue::Bool(true));
    assert_eq!(
        shown.bindings["pool"]["canTaste"],
        BindingValue::Bool(false)
    );
}

#[test]
fn bad_defines_are_rejected_before_execution() {
    for (source, expected) in [
        (
            "define loop = loop + 1; bind hud.x = loop;",
            "refers to itself",
        ),
        (
            "define a = b; define b = a; bind hud.x = a;",
            "refers to itself",
        ),
        ("define unused = nowhere + 1;", "nowhere"),
        (
            "state level = 0; define level = 1;",
            "collides with a state",
        ),
        (
            "fn half(x) = x / 2; define half = 1;",
            "collides with a function",
        ),
        (
            "event move step min 0 max 1; define step = 1;",
            "collides with an event parameter",
        ),
        (
            "define twice = 1; define twice = 2;",
            "duplicate define twice",
        ),
        ("define broken 1;", "define expects NAME = EXPRESSION"),
    ] {
        let error = ReactiveSession::from_source(&format!(
            "event e;
{source}"
        ))
        .err()
        .unwrap_or_else(|| panic!("{source} should be rejected"));
        assert!(error.contains(expected), "{source}: {error}");
    }
}

#[test]
fn a_pure_function_cannot_read_a_define() {
    let error = ReactiveSession::from_source(
        "event e; state level = 1; define high = level > 0; fn gate(x) = high; bind hud.x = gate(1);",
    )
    .unwrap_err();
    assert!(error.contains("high"), "{error}");
}

#[test]
fn a_define_may_read_an_event_parameter_where_that_parameter_exists() {
    // Found by the glowcap round 4 rewrite: `define $m_here = target == $index;`
    // was rejected on its own even though every use was inside an event rule.
    let source = r#"
        place garden kind garden;
        entity cave kind mushroom at garden;
        entity pool kind mushroom at garden;
        event absorb target kind mushroom;
        for mushroom as $m {
            state $m_consumed = 0;
            define $m_here = target == $index;
            on absorb when $m_here set $m_consumed = 1;
        };
    "#;
    let mut game = session(source);
    let shown = game
        .dispatch_json("absorb", r#"{"target":"pool"}"#)
        .unwrap();
    assert_eq!(shown.values["pool_consumed"], 1.0);
    assert_eq!(shown.values["cave_consumed"], 0.0);
    // A binding has no event parameters, so using the define there is an error.
    let error =
        ReactiveSession::from_source(&format!("{source}\nbind hud.here = cave_here;")).unwrap_err();
    assert!(error.contains("target"), "{error}");
}
