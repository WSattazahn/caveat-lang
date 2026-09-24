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
        error.contains("NAME followed by evidence, claim, caveat, readings or decisions"),
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

/// Trial v3: a second instrument has its own reading stream, and every rule
/// for the first was written out again for it.
const INSTRUMENTS: &str = r#"
claim ice_safe;
evidence auger from "a hand auger";
evidence sonar from "a sonar gauge";
caveat uncalibrated consequence material;
uncalibrated qualifies sonar;
readings thickness from auger limit 3;
readings echo from sonar limit 3;
decisions rink limit 3;
state lowest = 0;
state deepest = 0;
state first = 0;
event measure cm min 0 max 60;
event scan cm min 0 max 60;
event decide;
fn deeper(most, next) = max(most, next);
define thinnest = if(has_sample(thickness),
    fold_history(echo, fold_history(thickness, latest(thickness), min), min),
    fold_history(echo, latest(echo), min));
on decide when history_count(thickness) + history_count(echo) < 2 reject "Take two readings first.";
"#;

const BY_HAND: &str = r#"
on measure when history_count(thickness) >= 3 reject "That instrument is done.";
on measure when cm >= 10 sample thickness = cm supports ice_safe;
on measure when cm < 10 sample thickness = cm opposes ice_safe;
on measure set lowest = fold_history(thickness, latest(thickness), min);
on measure set deepest = fold_history(thickness, 0, deeper);
on measure set first = history_at(thickness, 0);
on measure when cm < 10 and committed(rink) and not reopened(rink) and latest(rink) >= 10 reopen rink because latest(thickness);
on scan when history_count(echo) >= 3 reject "That instrument is done.";
on scan when cm >= 10 sample echo = cm supports ice_safe;
on scan when cm < 10 sample echo = cm opposes ice_safe;
on scan set lowest = fold_history(echo, latest(echo), min);
on scan set deepest = fold_history(echo, 0, deeper);
on scan set first = history_at(echo, 0);
on scan when cm < 10 and committed(rink) and not reopened(rink) and latest(rink) >= 10 reopen rink because latest(echo);
on decide when committed(rink) and not reopened(rink) reject "A decision is in force.";
on decide commit rink because enough using thinnest;
"#;

const SHARED: &str = r#"
proc take(s readings, d decisions, cm) {
    when history_count(s) >= 3 reject "That instrument is done.";
    when cm >= 10 sample s = cm supports ice_safe;
    when cm < 10 sample s = cm opposes ice_safe;
    set lowest = fold_history(s, latest(s), min);
    set deepest = fold_history(s, 0, deeper);
    set first = history_at(s, 0);
    when cm < 10 and committed(d) and not reopened(d) and latest(d) >= 10 reopen d because latest(s);
};
proc settle(d decisions) {
    when committed(d) and not reopened(d) reject "A decision is in force.";
    commit d because enough using thinnest;
};
on measure call take(thickness, rink, cm);
on scan call take(echo, rink, cm);
on decide call settle(rink);
"#;

fn snapshot_without_source(session: &ReactiveSession) -> serde_json::Value {
    let mut snapshot = serde_json::to_value(session.snapshot()).unwrap();
    snapshot.as_object_mut().unwrap().remove("source_id");
    snapshot
}

#[test]
fn a_procedure_over_histories_behaves_as_the_rules_written_out() {
    let mut by_hand = ReactiveSession::from_source(&format!("{INSTRUMENTS}\n{BY_HAND}")).unwrap();
    let mut shared = ReactiveSession::from_source(&format!("{INSTRUMENTS}\n{SHARED}")).unwrap();
    let steps: [(&str, Option<f64>); 12] = [
        ("decide", None),
        ("measure", Some(14.0)),
        ("scan", Some(12.0)),
        ("decide", None),
        ("decide", None),
        ("scan", Some(6.0)),
        ("decide", None),
        ("measure", Some(30.0)),
        ("scan", Some(40.0)),
        ("scan", Some(8.0)),
        ("measure", Some(9.0)),
        ("measure", Some(11.0)),
    ];
    let mut refused = 0;
    for (event, cm) in steps {
        let payload = cm
            .map(|cm| BTreeMap::from([("cm".to_string(), cm)]))
            .unwrap_or_default();
        let expected = by_hand.apply(event, &payload);
        let actual = shared.apply(event, &payload);
        assert_eq!(
            expected.is_ok(),
            actual.is_ok(),
            "{event} {cm:?}: {expected:?} / {actual:?}"
        );
        refused += usize::from(actual.is_err());
        assert_eq!(
            snapshot_without_source(&shared),
            snapshot_without_source(&by_hand),
            "{event} {cm:?}"
        );
    }
    // The first decide, the decide while one is in force, the fourth scan and
    // the fourth measurement.
    assert_eq!(refused, 4);
    let snapshot = shared.snapshot();
    assert_eq!(
        snapshot.decision_series["rink"].current.as_deref(),
        Some("rink@2")
    );
    assert_eq!(snapshot.values["first"], 14.0);
    assert!(snapshot
        .relations
        .iter()
        .any(|edge| edge.from == "echo@2" && edge.relation == "reopens"));
    assert_eq!(
        snapshot.commitment_grounds["rink@2"].caveats,
        names(&["uncalibrated"])
    );
}

#[test]
fn a_history_parameter_takes_a_declared_history_of_its_kind() {
    for (call, expected) in [
        (
            "call take(auger, rink, 1)",
            "takes the name of a declared reading stream",
        ),
        (
            "call take(rink, rink, 1)",
            "takes the name of a declared reading stream",
        ),
        (
            "call take(thickness, echo, 1)",
            "takes the name of a declared decision series",
        ),
        (
            "call settle(ice_safe)",
            "takes the name of a declared decision series",
        ),
    ] {
        let error = ReactiveSession::from_source(&format!(
            "{INSTRUMENTS}\n{SHARED}\nevent bad; on bad {call};"
        ))
        .map(|_| ())
        .unwrap_err();
        assert!(error.contains(expected), "{call}: {error}");
    }
    for header in ["thickness readings", "rink decisions", "rink readings"] {
        let error = ReactiveSession::from_source(&format!(
            "{INSTRUMENTS}\nproc bad({header}) {{ set lowest = 1; }};"
        ))
        .map(|_| ())
        .unwrap_err();
        assert!(
            error.contains("shadows a declared name"),
            "{header}: {error}"
        );
    }
}

#[test]
fn a_module_can_offer_a_procedure_over_histories() {
    use caveat_runtime::link::{bundle, BundlePart};
    let part = |name: &str, source: &str| BundlePart {
        name: name.into(),
        source: source.into(),
    };
    let source = bundle(&[
        part(
            "gauges",
            "module gauges;\nproc take(s readings, c claim, cm) { sample s = cm supports c; };\n",
        ),
        part(
            "main",
            "use gauges;\nclaim safe;\nevidence auger from \"auger\";\nreadings thickness from auger limit 2;\nevent measure cm min 0 max 60;\non measure call gauges::take(thickness, safe, cm);\n",
        ),
    ]);
    let mut game = ReactiveSession::from_source(&source).unwrap();
    game.apply("measure", &BTreeMap::from([("cm".into(), 12.0)]))
        .unwrap();
    let snapshot = game.snapshot();
    assert_eq!(
        snapshot.reading_streams["thickness"].current.as_deref(),
        Some("thickness@1")
    );
}
