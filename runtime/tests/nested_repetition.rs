//! Nested Repetition 0.1: a `for` block in a `for` block's body.

use caveat_runtime::reactive::{check_source, ReactiveSession};
use caveat_runtime::repeat;

fn statements(source: &str) -> Vec<String> {
    source
        .split(';')
        .map(|statement| statement.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|statement| !statement.is_empty())
        .collect()
}

const ENTITIES: &str = "place repo kind repository;\n\
     entity api kind component at repo;\n\
     entity web kind component at repo;\n\
     entity d1 kind slot at repo;\n\
     entity d2 kind slot at repo;\n\
     entity d3 kind slot at repo;\n";

#[test]
fn a_nested_block_expands_outer_member_major() {
    let expanded = repeat::expand(&format!(
        "{ENTITIES}for component as $c {{\n\
           state $c_count = 0;\n\
           for slot as $k {{ state $c_ack_$k = $index; }};\n\
           bind $c.count = $c_count + $index;\n\
         }};\n"
    ))
    .expect("a nested block expands");
    let mut hand = statements(ENTITIES);
    for (outer, component) in ["api", "web"].iter().enumerate() {
        hand.push(format!("state {component}_count = 0"));
        for (inner, slot) in ["d1", "d2", "d3"].iter().enumerate() {
            hand.push(format!("state {component}_ack_{slot} = {}", inner + 1));
        }
        hand.push(format!(
            "bind {component}.count = {component}_count + {}",
            outer + 1
        ));
    }
    assert_eq!(statements(&expanded), hand);
}

#[test]
fn three_levels_and_a_binding_an_enclosing_one_prefixes() {
    // `$cx` is not `$c` glued to `x`: the longest binding wins, whichever
    // block binds it.
    let expanded = repeat::expand(&format!(
        "{ENTITIES}entity n1 kind note at repo;\n\
         for component as $c {{ for slot as $cx {{ for note as $n {{ claim $c_$cx_$n_$index; }}; }}; }};\n"
    ))
    .expect("three levels expand");
    let claims = statements(&expanded)
        .into_iter()
        .filter(|statement| statement.starts_with("claim"))
        .collect::<Vec<_>>();
    assert_eq!(
        claims,
        [
            "claim api_d1_n1_1",
            "claim api_d2_n1_1",
            "claim api_d3_n1_1",
            "claim web_d1_n1_1",
            "claim web_d2_n1_1",
            "claim web_d3_n1_1",
        ]
    );
}

#[test]
fn a_source_without_a_nested_block_expands_as_before() {
    let source = format!("{ENTITIES}for slot as $k {{ claim $k_seen; }};\n");
    let expanded = repeat::expand(&source).expect("expands");
    assert!(expanded.contains("claim d3_seen;"), "{expanded}");
    assert_eq!(repeat::expand(ENTITIES).unwrap(), ENTITIES);
}

#[test]
fn what_a_nested_block_refuses() {
    for (source, expected) in [
        (
            format!("{ENTITIES}for component as $c {{ for slot as $c {{ claim $c; }}; }};"),
            "`for slot as $c` binds $c, which an enclosing for block binds too",
        ),
        (
            format!("{ENTITIES}for component as $c {{ for slot as $k {{ claim $typo; }}; }};"),
            "$typo is not bound",
        ),
        (
            format!("{ENTITIES}for component as $c {{ for slot as $k {{ claim $k; }} }};"),
            "has no `;` before the `}` that ends the body",
        ),
        (
            format!("{ENTITIES}for component as $c {{ for gate as $k {{ claim $k; }}; }};"),
            "no entity is declared `kind gate`",
        ),
    ] {
        let error = repeat::expand(&source).unwrap_err();
        assert!(error.contains(expected), "{error}\n{source}");
    }
}

const EVENTS: &str = "event ack component kind component, decision min 1 max 3;\n\
     event touch component kind component;\n";

fn ack_states(source: &str, events: &[(&str, u32)]) -> Vec<f64> {
    let mut session = ReactiveSession::from_source(source).expect("loads");
    let mut values = Default::default();
    for (component, decision) in events {
        values = session
            .dispatch_json(
                "ack",
                &format!(r#"{{"component":"{component}","decision":{decision}}}"#),
            )
            .expect("ack is accepted")
            .values;
    }
    let values: std::collections::BTreeMap<String, f64> = values;
    ["api", "web"]
        .iter()
        .flat_map(|component| ["d1", "d2", "d3"].map(|slot| format!("{component}_ack_{slot}")))
        .map(|name| values[&name])
        .collect()
}

#[test]
fn a_routed_block_routes_the_rules_of_a_block_nested_in_it() {
    let source = format!(
        "{ENTITIES}{EVENTS}for component as $c routed by component {{\n\
           for slot as $k {{\n\
             state $c_ack_$k = 0;\n\
             on ack when decision == $index set $c_ack_$k = $c_ack_$k + 1;\n\
             bind $c.$k = $c_ack_$k;\n\
           }};\n\
         }};\n"
    );
    let expanded = repeat::expand(&source).expect("expands");
    assert!(
        expanded.contains(
            "on ack when component == 2 and decision == 3 set web_ack_d3 = web_ack_d3 + 1;"
        ),
        "{expanded}"
    );
    assert_eq!(
        ack_states(&source, &[("web", 3), ("api", 1), ("web", 3)]),
        [1.0, 0.0, 0.0, 0.0, 0.0, 2.0]
    );
}

#[test]
fn routes_nest_outermost_first() {
    let source = format!(
        "{ENTITIES}event pick component kind component, slot kind slot;\n\
         for component as $c routed by component {{\n\
           for slot as $k routed by slot {{ on pick when $k.x >= 0 set $c_picked = $index; }};\n\
         }};\n"
    );
    let expanded = repeat::expand(&source).expect("expands");
    assert!(
        expanded.contains(
            "on pick when component == 1 and slot == 2 and d2.x >= 0 set api_picked = 2;"
        ),
        "{expanded}"
    );
}

/// Found by a fresh author: a nested routed rule whose guard has an `or` and
/// names the enclosing block's binding lost its parentheses, so another
/// member's copy refused the event.
#[test]
fn a_nested_routed_guard_with_or_stays_one_condition() {
    let source = format!(
        "{ENTITIES}evidence tool from \"a tool\";\n\
         event report component kind component, slot kind slot;\n\
         event pick component kind component, slot kind slot, n min 0 max 8;\n\
         for component as $c routed by component {{\n\
           for slot as $k routed by slot {{\n\
             readings $c_$k from tool limit 8;\n\
             on report sample $c_$k = 1;\n\
             on pick when n < 1 or n > history_count($c_$k) reject \"no such report\";\n\
           }};\n\
         }};\n"
    );
    let expanded = repeat::expand(&source).expect("expands");
    assert!(
        expanded.contains(
            "on pick when component == 1 and slot == 2 and (n < 1 or n > history_count(api_d2)) reject"
        ),
        "{expanded}"
    );
    let mut session = ReactiveSession::from_source(&source).expect("loads");
    for _ in 0..3 {
        session
            .dispatch_json("report", r#"{"component":"web","slot":"d2"}"#)
            .expect("a report");
    }
    for n in [1, 3] {
        let outcome = session
            .clone()
            .dispatch_outcome_json(
                "pick",
                &format!(r#"{{"component":"web","slot":"d2","n":{n}}}"#),
            )
            .expect("not fatal");
        assert!(
            serde_json::to_string(&outcome)
                .unwrap()
                .contains(r#""outcome":"accepted""#),
            "report {n} of 3 exists"
        );
    }
    let outcome = session
        .dispatch_outcome_json("pick", r#"{"component":"web","slot":"d2","n":4}"#)
        .expect("not fatal");
    assert!(serde_json::to_string(&outcome)
        .unwrap()
        .contains("no such report"));
}

#[test]
fn a_routed_rule_must_tell_its_own_blocks_copies_apart() {
    // Inside the nested block, `$c` is the enclosing block's: the three copies
    // of this rule for d1, d2 and d3 would be the same rule.
    let source = format!(
        "{ENTITIES}event pick component kind component, slot kind slot;\n\
         for component as $c routed by component {{\n\
           for slot as $k routed by slot {{ on pick set $c_n = 1; }};\n\
         }};\n"
    );
    let error = repeat::expand(&source).unwrap_err();
    assert!(
        error.contains(
            "`on pick` in `for slot as $k routed by slot` mentions neither `$k` nor `$index`"
        ),
        "{error}"
    );
    // An event named only with the enclosing block's binding.
    let source = format!(
        "{ENTITIES}event pick_api slot kind slot;\nevent pick_web slot kind slot;\n\
         for component as $c {{\n\
           for slot as $k routed by slot {{ on pick_$c set $k_n = 1; }};\n\
         }};\n"
    );
    let error = repeat::expand(&source).unwrap_err();
    assert!(
        error.contains("named with an enclosing for block's binding only"),
        "{error}"
    );
}

#[test]
fn a_nested_block_declares_events_an_enclosing_routed_block_reads() {
    let source = format!(
        "{ENTITIES}for slot as $k {{ for component as $c {{ claim $k_$c_seen; }}; }};\n\
         event mark component kind component;\n\
         for component as $c routed by component {{ on mark set $c_marks = 1; }};\n"
    );
    repeat::expand(&source).expect("expands");
}

#[test]
fn check_reads_the_rules_of_a_nested_block() {
    // A plain outer block whose nested rule does not select its component:
    // every component's copy runs on each ack. C003 reports it.
    let source = format!(
        "{ENTITIES}{EVENTS}for component as $c {{\n\
           for slot as $k {{\n\
             state $c_ack_$k = 0;\n\
             on ack when decision == $index set $c_ack_$k = 1;\n\
             bind $c.$k = $c_ack_$k;\n\
           }};\n\
         }};\n"
    );
    let report = check_source(&source).expect("checks");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "C003"),
        "{report:?}"
    );
    // Selecting the component clears it.
    let selected = source.replace(
        "on ack when decision == $index",
        "on ack when component == component.$c and decision == $index",
    );
    let report = check_source(&selected).expect("checks");
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "C003"),
        "{report:?}"
    );
}
