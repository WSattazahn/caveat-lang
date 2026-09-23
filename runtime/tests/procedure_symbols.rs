//! Procedures whose parameters name graph symbols. See
//! spec/caveat-procedure-symbols-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use std::collections::{BTreeMap, BTreeSet};

const WORLD: &str = r#"
claim glowing_is_safe;
caveat secondhand consequence material;
place garden kind garden;
entity cave kind mushroom at garden;
entity pool kind mushroom at garden;
state support = 0;
state contradiction = 0;
event absorb target kind mushroom, sort in glowcap duskcap;
event witness target kind mushroom, sort in glowcap duskcap;
"#;

fn load(body: &str) -> Result<ReactiveSession, String> {
    ReactiveSession::from_source(&format!("{WORLD}\n{body}"))
}

fn send(session: &mut ReactiveSession, event: &str, target: f64, sort: f64) -> Result<(), String> {
    session.apply(
        event,
        &BTreeMap::from([("target".into(), target), ("sort".into(), sort)]),
    )
}

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

/// Round 6, CR9: a second way to observe a mushroom shares one procedure
/// instead of restating every absorb rule.
const LEARN: &str = r#"
proc learn(e evidence, sort) {
    when sort == 1 reveal e supports glowing_is_safe;
    when sort == 1 set support = support + qualified(1, e);
    when sort == 1 and contradiction == 0 and not committed(trust) commit trust because enough using qualified(1, e);
    when sort == 2 reveal e opposes glowing_is_safe;
    when sort == 2 set contradiction = contradiction + qualified(1, e);
    when sort == 2 and committed(trust) reopen trust because e;
};
for mushroom as $m {
    evidence absorb_$m from "absorbed";
    evidence witness_$m from "seen eaten";
    secondhand qualifies witness_$m;
    on absorb when target == $index call learn(absorb_$m, sort);
    on witness when target == $index call learn(witness_$m, sort);
};
"#;

#[test]
fn one_procedure_serves_every_way_of_observing() {
    let mut game = load(LEARN).unwrap();
    send(&mut game, "witness", 1.0, 1.0).unwrap();
    let snapshot = game.snapshot();
    let support = &snapshot.qualified_values["support"].provenance;
    assert_eq!(support.evidence, names(&["witness_cave"]));
    assert_eq!(support.caveats, names(&["secondhand"]));
    assert_eq!(
        snapshot.commitment_grounds["trust"].caveats,
        names(&["secondhand"])
    );
    send(&mut game, "absorb", 2.0, 2.0).unwrap();
    let snapshot = game.snapshot();
    assert_eq!(snapshot.values["contradiction"], 1.0);
    assert!(snapshot
        .relations
        .iter()
        .any(|edge| edge.from == "absorb_pool" && edge.relation == "reopens"));
}

#[test]
fn each_call_is_a_procedure_of_its_own_named_for_what_it_passes() {
    let mut game = load(&format!(
        "{LEARN}\nevent spoil; on spoil call learn(absorb_cave, 3);\nproc noop() {{ set support = support; }};"
    ))
    .unwrap();
    // Qualifying with unrevealed evidence fails inside the specialization.
    let program = format!(
        "{WORLD}\nevidence seen from \"seen\";\nevent look;\nevent eat;\non eat reveal seen supports glowing_is_safe;\nproc tally(e evidence) {{ set support = qualified(1, e); }};\non look call tally(seen);"
    );
    let mut other = ReactiveSession::from_source(&program).unwrap();
    let error = other.apply("look", &BTreeMap::new()).unwrap_err();
    assert!(error.contains("procedure tally[seen], step 1"), "{error}");
    game.apply("spoil", &BTreeMap::new()).unwrap();
}

#[test]
fn names_pass_through_nested_calls() {
    let mut game = load(
        r#"
        evidence chart from "chart";
        proc bear(e evidence, c claim) { reveal e supports c; };
        proc note(e evidence) { call bear(e, glowing_is_safe); set support = qualified(1, e); };
        event read;
        on read call note(chart);
        "#,
    )
    .unwrap();
    game.apply("read", &BTreeMap::new()).unwrap();
    assert_eq!(
        game.snapshot().qualified_values["support"]
            .provenance
            .evidence,
        names(&["chart"])
    );
}

#[test]
fn a_symbol_parameter_takes_a_declared_name_of_its_kind() {
    for (call, expected) in [
        (
            "call learn(glowing_is_safe, sort)",
            "takes the name of a declared evidence",
        ),
        (
            "call learn(support + 1, sort)",
            "takes the name of a declared evidence",
        ),
        (
            "call learn(nowhere, sort)",
            "takes the name of a declared evidence",
        ),
        ("call learn(absorb_cave)", "expects 2 arguments, got 1"),
    ] {
        let error = load(&format!("{LEARN}\nevent bad; on bad {call};"))
            .map(|_| ())
            .unwrap_err();
        assert!(error.contains(expected), "{call}: {error}");
    }
}

#[test]
fn a_symbol_parameter_cannot_shadow_a_declared_name() {
    let error = load(
        r#"
        evidence chart from "chart";
        proc bad(chart evidence) { reveal chart supports glowing_is_safe; };
        "#,
    )
    .unwrap_err();
    assert!(
        error.contains("parameter chart shadows a declared name"),
        "{error}"
    );
    let error = load("proc bad(e evidence, e) { set support = e; };").unwrap_err();
    assert!(error.contains("duplicate parameter e"), "{error}");
    let error = load("proc bad(e place) { set support = 1; };").unwrap_err();
    assert!(
        error.contains("NAME evidence, NAME claim or NAME caveat"),
        "{error}"
    );
}

#[test]
fn a_module_can_offer_a_procedure_over_symbols() {
    use caveat_runtime::link::{bundle, BundlePart};
    let part = |name: &str, source: &str| BundlePart {
        name: name.into(),
        source: source.into(),
    };
    let source = bundle(&[
        part(
            "senses",
            "module senses;\nproc learn(e evidence, c claim) { reveal e supports c; };\n",
        ),
        part(
            "main",
            "use senses;\nclaim safe;\nevidence chart from \"chart\";\nevent read;\non read call senses::learn(chart, safe);\n",
        ),
    ]);
    let mut game = ReactiveSession::from_source(&source).unwrap();
    game.apply("read", &BTreeMap::new()).unwrap();
    assert!(game
        .snapshot()
        .relations
        .iter()
        .any(|edge| edge.from == "chart" && edge.to == "safe" && edge.relation == "supports"));
}

#[test]
fn recursion_through_templates_is_still_a_cycle() {
    let error = load(
        r#"
        evidence chart from "chart";
        proc again(e evidence) { call again(e); };
        event go;
        on go call again(chart);
        "#,
    )
    .unwrap_err();
    assert!(error.contains("recursive procedure cycle"), "{error}");
}
