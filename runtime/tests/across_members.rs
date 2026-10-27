//! Across Members 0.1: `sum(for KIND as $NAME: E)`, `any(…)` and `all(…)`.
//! See spec/caveat-across-members-0.1.md.

use caveat_runtime::reactive::{check_source, BindingValue, ReactiveSession};
use caveat_runtime::repeat;

const HEAD: &str = "place field kind gate;\n\
     entity north kind gate at field;\n\
     entity south kind gate at field;\n\
     entity ocelot kind informant at field;\n\
     entity eva kind informant at field;\n\
     entity snake kind informant at field;\n";

#[test]
fn a_sum_expands_to_the_hand_written_sum_on_one_line() {
    let source = format!("{HEAD}state a_ocelot = 1;\ndefine total = sum(for informant as $i: a_$i);\nbind hud.total = total;\n");
    let expanded = repeat::expand(&source).expect("expands");
    assert_eq!(
        expanded,
        format!("{HEAD}state a_ocelot = 1;\ndefine total = ((a_ocelot) + (a_eva) + (a_snake));\nbind hud.total = total;\n")
    );
    assert_eq!(
        expanded.lines().count(),
        source.lines().count(),
        "no line moves"
    );
}

#[test]
fn any_joins_with_or_and_all_with_and() {
    let expanded = repeat::expand(&format!(
        "{HEAD}define one = any(for informant as $i: observed($i_word));\ndefine every = all(for gate as $g: $g_ok == 1);\n"
    ))
    .expect("expands");
    assert!(expanded.contains(
        "define one = ((observed(ocelot_word)) or (observed(eva_word)) or (observed(snake_word)));"
    ));
    assert!(expanded.contains("define every = ((north_ok == 1) and (south_ok == 1));"));
}

#[test]
fn index_is_the_members_position_inside_the_form() {
    let expanded = repeat::expand(&format!(
        "{HEAD}define weights = sum(for informant as $i: $index * w_$i);\n"
    ))
    .expect("expands");
    assert!(expanded.contains("((1 * w_ocelot) + (2 * w_eva) + (3 * w_snake))"));
}

#[test]
fn inside_a_block_the_enclosing_binding_is_the_blocks_member() {
    let expanded = repeat::expand(&format!(
        "{HEAD}for gate as $g {{\n    define $g_count = sum(for informant as $i: $g_$i_ok);\n    bind $g.n = $index;\n}};\n"
    ))
    .expect("expands");
    assert!(expanded
        .contains("define north_count = ((north_ocelot_ok) + (north_eva_ok) + (north_snake_ok));"));
    assert!(expanded
        .contains("define south_count = ((south_ocelot_ok) + (south_eva_ok) + (south_snake_ok));"));
    assert!(
        expanded.contains("bind north.n = 1;"),
        "the block's own $index is its member's"
    );
    assert!(expanded.contains("bind south.n = 2;"));
}

#[test]
fn forms_nest() {
    let expanded = repeat::expand(&format!(
        "{HEAD}define pairs = sum(for gate as $g: sum(for informant as $i: $g_$i));\n"
    ))
    .expect("expands");
    assert!(expanded.contains(
        "define pairs = ((((north_ocelot) + (north_eva) + (north_snake))) + (((south_ocelot) + (south_eva) + (south_snake))));"
    ));
}

#[test]
fn quoted_text_comments_and_longer_names_are_left_alone() {
    let source = format!(
        "{HEAD}evidence note from \"sum(for informant as $i: x)\";\n# any(for informant as $i: x)\ndefine my_sum = 1;\ndefine t = my_sum(for);\n"
    );
    assert_eq!(repeat::expand(&source).expect("no form"), source);
}

#[test]
fn a_comment_inside_the_form_is_left_out_of_the_copies() {
    let expanded = repeat::expand(&format!(
        "{HEAD}define total = sum(for informant as $i: a_$i # each report\n);\n"
    ))
    .expect("expands");
    assert!(expanded.contains("define total = ((a_ocelot) + (a_eva) + (a_snake));"));
}

#[test]
fn malformed_forms_are_refused_with_what_to_write() {
    for (form, expected) in [
        (
            "sum(for tower as $t: x)",
            "no entity is declared `kind tower`",
        ),
        (
            "sum(for informant as $index: x)",
            "$index is provided by the form",
        ),
        (
            "sum(for informant as i: x)",
            "expected `for KIND as $NAME: EXPRESSION`",
        ),
        (
            "sum(for informant $i: x)",
            "expected `for KIND as $NAME: EXPRESSION`",
        ),
        (
            "sum(for informant as $i x)",
            "expected `for KIND as $NAME: EXPRESSION`",
        ),
        (
            "sum(for informant as $i: )",
            "has no expression after its `:`",
        ),
        ("sum(for informant as $i: (x)", "is not closed"),
        ("sum(for informant as $i: $j)", "$j is not bound"),
    ] {
        let error = repeat::expand(&format!("{HEAD}define t = {form};\n")).expect_err(form);
        assert!(error.contains(expected), "{form}: {error}");
    }
    let error = repeat::expand(&format!(
        "{HEAD}for gate as $g {{ define $g_t = sum(for informant as $g: x_$g); }};\n"
    ))
    .expect_err("rebinding");
    assert!(
        error.contains("an enclosing for block binds too"),
        "{error}"
    );
}

const INTEL: &str = "event intel informant kind informant, gate kind gate, says in clear guarded;\n\
     event go gate kind gate;\n\
     for informant as $i { evidence $i_word from \"what $i says\"; };\n\
     for gate as $g {\n\
       claim $g_clear;\n\
       decisions $g_plan limit 4;\n\
       for informant as $i {\n\
         readings $g_$i from $i_word limit 4;\n\
         on intel when gate == gate.$g and informant == informant.$i and says == says.clear sample $g_$i = 1 supports $g_clear;\n\
         define $g_$i_ok = if(has_sample($g_$i), latest($g_$i), 0);\n\
       };\n\
       on go when gate == gate.$g and TOTAL == 0 reject \"None of your intel says that gate is clear.\";\n\
       on go when gate == gate.$g commit $g_plan because enough using TOTAL;\n\
       bind $g.heard = TOTAL;\n\
     };\n";

#[test]
fn a_program_with_a_sum_runs_as_its_hand_written_twin() {
    let summed = format!(
        "{HEAD}{}",
        INTEL.replace("TOTAL", "sum(for informant as $i: $g_$i_ok)")
    );
    let hand = format!(
        "{HEAD}{}",
        INTEL.replace("TOTAL", "$g_ocelot_ok + $g_eva_ok + $g_snake_ok")
    );
    let mut one = ReactiveSession::from_source(&summed).expect("the sum loads");
    let mut two = ReactiveSession::from_source(&hand).expect("the hand-written twin loads");
    for (event, payload) in [
        ("go", r#"{"gate":"north"}"#),
        (
            "intel",
            r#"{"informant":"eva","gate":"north","says":"clear"}"#,
        ),
        (
            "intel",
            r#"{"informant":"snake","gate":"south","says":"guarded"}"#,
        ),
        ("go", r#"{"gate":"south"}"#),
        ("go", r#"{"gate":"north"}"#),
    ] {
        let left = one.dispatch_json(event, payload);
        let right = two.dispatch_json(event, payload);
        assert_eq!(left.is_ok(), right.is_ok(), "{event} {payload}");
        assert_eq!(
            one.snapshot().bindings,
            two.snapshot().bindings,
            "{event} {payload}"
        );
    }
    assert_eq!(
        one.snapshot().bindings["north"]["heard"],
        BindingValue::Number(1.0)
    );
    assert_eq!(one.snapshot().commitments, two.snapshot().commitments);
}

#[test]
fn check_reads_a_program_with_a_sum_as_its_expansion() {
    let summed = format!(
        "{HEAD}{}",
        INTEL.replace("TOTAL", "sum(for informant as $i: $g_$i_ok)")
    );
    let hand = format!(
        "{HEAD}{}",
        INTEL.replace("TOTAL", "$g_ocelot_ok + $g_eva_ok + $g_snake_ok")
    );
    let codes = |source: &str| {
        check_source(source)
            .expect("checks")
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>()
    };
    assert_eq!(codes(&summed), codes(&hand));
}
