//! A use of evidence that can only fail is an error when the program loads.
//! See spec/caveat-observation-order-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use std::collections::BTreeMap;

const HEADER: &str = r#"
claim safe;
caveat dim consequence material;
place garden kind garden;
entity cave kind mushroom at garden;
entity pool kind mushroom at garden;
event absorb target kind mushroom, sort in glowcap duskcap;
state consumed = 0;
state count = 0;
"#;

fn load(body: &str) -> Result<ReactiveSession, String> {
    ReactiveSession::from_source(&format!("{HEADER}\n{body}"))
}

fn absorb(session: &mut ReactiveSession, target: f64) -> Result<(), String> {
    session.apply(
        "absorb",
        &BTreeMap::from([("target".into(), target), ("sort".into(), 1.0)]),
    )
}

#[test]
fn a_use_before_the_only_reveal_under_the_same_guard_is_rejected() {
    // Round 6 of the glowcap benchmark: consumption was qualified by the
    // absorption before the rule that revealed it.
    let error = load(
        r#"
        for mushroom as $m {
            evidence absorb_$m from "absorbed";
            define $m_here = target == $index;
            on absorb when $m_here set consumed = qualified(1, absorb_$m);
            on absorb when $m_here and sort == sort.glowcap reveal absorb_$m supports safe;
        };
        "#,
    )
    .unwrap_err();
    assert!(
        error.contains("qualified(…, absorb_cave) runs before rule 2 reveals absorb_cave"),
        "{error}"
    );
    assert!(error.contains("reveal absorb_cave first"), "{error}");
}

#[test]
fn the_same_rules_in_the_right_order_load_and_run() {
    let mut game = load(
        r#"
        evidence seen from "absorbed";
        on absorb when target == target.cave reveal seen supports safe;
        on absorb when target == target.cave set consumed = qualified(1, seen);
        "#,
    )
    .unwrap();
    absorb(&mut game, 1.0).unwrap();
}

#[test]
fn qualify_and_reopen_before_the_reveal_are_rejected_too() {
    let error = load(
        r#"
        evidence seen from "absorbed";
        on absorb qualify seen with dim;
        on absorb reveal seen supports safe;
        "#,
    )
    .unwrap_err();
    assert!(error.contains("qualify seen runs before rule 2"), "{error}");
    let error = load(
        r#"
        evidence first from "first";
        evidence seen from "absorbed";
        first supports safe;
        event start;
        on start commit trust because enough using qualified(1, first);
        on absorb reopen trust because seen;
        on absorb reveal seen opposes safe;
        "#,
    )
    .unwrap_err();
    assert!(
        error.contains("reopen because seen runs before rule 3"),
        "{error}"
    );
}

#[test]
fn a_guard_that_reads_the_graph_is_not_certain_so_the_runtime_reports_it() {
    // Every absorb here fails, but proving that needs to know what
    // committed(trust) will be: the check leaves it to the runtime.
    let mut game = load(
        r#"
        evidence first from "first";
        evidence seen from "absorbed";
        first supports safe;
        on absorb when not committed(trust) commit trust because enough using qualified(1, first);
        on absorb when committed(trust) reopen trust because seen;
        on absorb when committed(trust) reveal seen opposes safe;
        "#,
    )
    .unwrap();
    let error = absorb(&mut game, 1.0).unwrap_err();
    assert!(error.contains("seen"), "{error}");
}

#[test]
fn evidence_nothing_ever_observes_is_rejected() {
    let error = load(
        r#"
        evidence never from "never";
        on absorb set consumed = qualified(1, never);
        "#,
    )
    .unwrap_err();
    assert!(
        error.contains("needs never observed, but nothing observes it"),
        "{error}"
    );
}

#[test]
fn a_procedure_step_is_checked_where_it_runs() {
    let error = load(
        r#"
        evidence seen from "absorbed";
        proc eat() {
            set consumed = qualified(1, seen);
            reveal seen supports safe;
        };
        on absorb call eat();
        "#,
    )
    .unwrap_err();
    assert!(
        error.contains("rule 1 (procedure eat, step 1): qualified(…, seen) runs before rule 1 (procedure eat, step 2) reveals seen"),
        "{error}"
    );
}

#[test]
fn guarding_with_observed_says_the_author_knows() {
    let mut game = load(
        r#"
        evidence seen from "absorbed";
        on absorb when observed(seen) set consumed = qualified(1, seen);
        on absorb reveal seen supports safe;
        "#,
    )
    .unwrap();
    absorb(&mut game, 1.0).unwrap();
    absorb(&mut game, 1.0).unwrap();
}

#[test]
fn a_guard_that_reads_state_could_be_false_the_first_time_so_it_loads() {
    // The first absorb skips the use and reveals; the second can use it.
    let mut game = load(
        r#"
        evidence seen from "absorbed";
        on absorb when count > 0 set consumed = qualified(1, seen);
        on absorb reveal seen supports safe;
        on absorb set count = count + 1;
        "#,
    )
    .unwrap();
    absorb(&mut game, 1.0).unwrap();
    absorb(&mut game, 1.0).unwrap();
}

#[test]
fn a_reveal_under_a_different_guard_is_not_certain_so_it_loads() {
    // Absorbing the pool reveals it; absorbing the cave uses it afterwards.
    let mut game = load(
        r#"
        evidence seen from "absorbed";
        on absorb when target == target.cave set consumed = qualified(1, seen);
        on absorb when target == target.pool reveal seen supports safe;
        "#,
    )
    .unwrap();
    absorb(&mut game, 2.0).unwrap();
    absorb(&mut game, 1.0).unwrap();
}

#[test]
fn evidence_revealed_elsewhere_or_at_load_is_never_rejected() {
    load(
        r#"
        evidence seen from "absorbed";
        event look;
        on look reveal seen supports safe;
        on absorb set consumed = qualified(1, seen);
        on absorb reveal seen supports safe;
        "#,
    )
    .unwrap();
    load(
        r#"
        evidence chart from "archive";
        chart supports safe;
        on absorb set consumed = qualified(1, chart);
        "#,
    )
    .unwrap();
}

#[test]
fn a_use_in_a_branch_that_may_not_be_taken_is_not_certain() {
    load(
        r#"
        evidence seen from "absorbed";
        on absorb set consumed = if(count > 0, qualified(1, seen), 0);
        on absorb reveal seen supports safe;
        "#,
    )
    .unwrap();
}
