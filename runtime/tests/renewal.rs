//! Renewable evidence, scheduled qualification and carries(E, C). See
//! spec/caveat-renewal-0.1.md.
use caveat_runtime::reactive::{BindingValue, ReactiveSession, ReactiveSnapshot};
use std::collections::{BTreeMap, BTreeSet};

const WORLD: &str = r#"
claim safe;
caveat secondhand consequence material;
caveat faded consequence material;
evidence bite from "the slime bit it";
secondhand qualifies bite;
renewable bite limit 3;
state support = 0;
event tick dt min 0 max 0.1;
event eat;
event regrow;
on eat reveal bite supports safe;
on eat set support = support + qualified(1, bite);
on regrow renew bite;
"#;

fn load(extra: &str) -> ReactiveSession {
    ReactiveSession::from_source(&format!("{WORLD}\n{extra}")).unwrap()
}

fn send(game: &mut ReactiveSession, event: &str) {
    game.apply(event, &BTreeMap::new()).unwrap();
}

fn wait(game: &mut ReactiveSession, seconds: usize) {
    for _ in 0..seconds * 16 {
        game.apply("tick", &BTreeMap::from([("dt".into(), 0.0625)]))
            .unwrap();
    }
}

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn supporters(snapshot: &ReactiveSnapshot) -> BTreeSet<String> {
    snapshot
        .relations
        .iter()
        .filter(|edge| edge.to == "safe" && edge.relation == "supports")
        .map(|edge| edge.from.clone())
        .collect()
}

#[test]
fn renewing_gives_the_name_a_new_unobserved_occurrence() {
    let mut game = load("bind seen.now = observed(bite);");
    send(&mut game, "eat");
    send(&mut game, "regrow");
    let snapshot = game.snapshot();
    assert_eq!(snapshot.bindings["seen"]["now"], BindingValue::Bool(false));
    assert_eq!(snapshot.renewals["bite"].occurrences, ["bite", "bite@2"]);
    assert!(snapshot
        .effects
        .iter()
        .any(|effect| serde_json::to_value(effect).unwrap()
            == serde_json::json!({"kind": "renew", "evidence": "bite", "occurrence": "bite@2"})));

    send(&mut game, "eat");
    let snapshot = game.snapshot();
    assert_eq!(snapshot.bindings["seen"]["now"], BindingValue::Bool(true));
    // Earlier provenance keeps naming the first occurrence; nothing merges them.
    assert_eq!(supporters(&snapshot), names(&["bite", "bite@2"]));
    let support = &snapshot.qualified_values["support"].provenance;
    assert_eq!(support.evidence, names(&["bite", "bite@2"]));
    // Declared caveats reach every occurrence.
    assert_eq!(support.caveats, names(&["secondhand"]));
}

#[test]
fn a_late_caveat_stays_with_the_occurrence_it_was_about() {
    let mut game = load("event spoil; on spoil qualify bite with faded;");
    send(&mut game, "eat");
    send(&mut game, "spoil");
    send(&mut game, "regrow");
    send(&mut game, "eat");
    let snapshot = game.snapshot();
    let support = &snapshot.qualified_values["support"].provenance;
    assert!(support.caveats.contains("faded"));
    // The new occurrence did not inherit the late caveat: only its declared one.
    let fresh = ReactiveSession::from_source(&format!(
        "{WORLD}\nevent spoil; on spoil qualify bite with faded;\nstate probe = 0;\nevent measure;\non measure set probe = qualified(1, bite);"
    ))
    .unwrap();
    let mut fresh = fresh;
    send(&mut fresh, "eat");
    send(&mut fresh, "spoil");
    send(&mut fresh, "regrow");
    send(&mut fresh, "eat");
    send(&mut fresh, "measure");
    let probe = &fresh.snapshot().qualified_values["probe"].provenance;
    assert_eq!(probe.evidence, names(&["bite@2"]));
    assert_eq!(probe.caveats, names(&["secondhand"]));
}

#[test]
fn renewal_is_bounded_and_declared() {
    let mut game = load("");
    send(&mut game, "regrow");
    send(&mut game, "regrow");
    let error = game.apply("regrow", &BTreeMap::new()).unwrap_err();
    assert!(
        error.contains("renewable bite reached its limit 3"),
        "{error}"
    );
    let error = ReactiveSession::from_source(
        "claim safe; evidence chart from \"chart\"; event go; on go renew chart;",
    )
    .map(|_| ())
    .unwrap_err();
    assert!(error.contains("declare it renewable first"), "{error}");
    let error = ReactiveSession::from_source(
        "claim safe; evidence chart from \"chart\"; renewable chart limit 0; event go;",
    )
    .map(|_| ())
    .unwrap_err();
    assert!(error.contains("limit must be in 1..1024"), "{error}");
}

#[test]
fn a_scheduled_caveat_arrives_when_its_time_has_passed() {
    let mut game = load(
        r#"
        on eat qualify bite with faded after 60;
        bind bite.label = "fresh";
        bind bite.label = "faded" when carries(bite, faded) because support;
        "#,
    );
    send(&mut game, "eat");
    assert_eq!(game.snapshot().scheduled_qualifications.len(), 1);
    wait(&mut game, 59);
    let snapshot = game.snapshot();
    assert_eq!(snapshot.elapsed, 59.0);
    assert_eq!(
        snapshot.bindings["bite"]["label"],
        BindingValue::Text("fresh".into())
    );
    assert!(!snapshot.qualified_values["support"]
        .provenance
        .caveats
        .contains("faded"));
    wait(&mut game, 1);
    let snapshot = game.snapshot();
    assert_eq!(
        snapshot.bindings["bite"]["label"],
        BindingValue::Text("faded".into())
    );
    assert!(snapshot.scheduled_qualifications.is_empty());
    assert!(snapshot.qualified_values["support"]
        .provenance
        .caveats
        .contains("faded"));
    assert_eq!(
        snapshot.binding_explanations["bite"]["label"].caveats,
        names(&["secondhand", "faded"])
    );
}

#[test]
fn a_scheduled_caveat_follows_its_occurrence_through_a_renewal() {
    // Round 6, CR10: a taste fades on its own clock even after the mushroom
    // it was about has been eaten and has regrown.
    let mut game = load(
        r#"
        on eat qualify bite with faded after 60;
        bind now.carries = carries(bite, faded);
        "#,
    );
    send(&mut game, "eat");
    wait(&mut game, 45);
    send(&mut game, "regrow");
    send(&mut game, "eat");
    wait(&mut game, 15);
    let snapshot = game.snapshot();
    let support = &snapshot.qualified_values["support"].provenance;
    assert!(
        support.caveats.contains("faded"),
        "the first bite has faded"
    );
    // The name now means the second bite, which has not.
    assert_eq!(
        snapshot.bindings["now"]["carries"],
        BindingValue::Bool(false)
    );
    let pending = &snapshot.scheduled_qualifications;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].evidence, "bite@2");
    wait(&mut game, 45);
    assert_eq!(
        game.snapshot().bindings["now"]["carries"],
        BindingValue::Bool(true)
    );
}

#[test]
fn a_decision_keeps_what_it_was_made_on() {
    let mut game = load(
        r#"
        on eat when not committed(trust) commit trust because enough using qualified(1, bite);
        on eat qualify bite with faded after 1;
        "#,
    );
    send(&mut game, "eat");
    wait(&mut game, 2);
    let snapshot = game.snapshot();
    assert!(!snapshot.commitment_grounds["trust"]
        .caveats
        .contains("faded"));
    assert!(snapshot.qualified_values["support"]
        .provenance
        .caveats
        .contains("faded"));
}

#[test]
fn scheduling_needs_something_to_count_time_and_checks_its_names() {
    let error = ReactiveSession::from_source(
        r#"
        claim safe; caveat faded consequence low; evidence chart from "chart";
        chart supports safe;
        event read;
        on read qualify chart with faded after 5;
        "#,
    )
    .map(|_| ())
    .unwrap_err();
    assert!(error.contains("nothing counts time"), "{error}");
    let error = ReactiveSession::from_source(&format!("{WORLD}\nbind x.y = carries(faded, bite);"))
        .map(|_| ())
        .unwrap_err();
    assert!(error.contains("must name a declared evidence"), "{error}");
}

#[test]
fn a_clock_event_counts_time_instead_of_tick() {
    let mut game = ReactiveSession::from_source(
        r#"
        claim safe; caveat faded consequence low; evidence chart from "chart";
        chart supports safe;
        event step dt min 0 max 0.5;
        clock step every 0.5;
        event read;
        on read qualify chart with faded after 1;
        bind chart.faded = carries(chart, faded);
        "#,
    )
    .unwrap();
    game.apply("read", &BTreeMap::new()).unwrap();
    game.apply("step", &BTreeMap::from([("dt".into(), 0.5)]))
        .unwrap();
    assert_eq!(
        game.snapshot().bindings["chart"]["faded"],
        BindingValue::Bool(false)
    );
    game.apply("step", &BTreeMap::from([("dt".into(), 0.5)]))
        .unwrap();
    assert_eq!(
        game.snapshot().bindings["chart"]["faded"],
        BindingValue::Bool(true)
    );
}

#[test]
fn a_procedure_over_renewable_evidence_uses_its_current_occurrence() {
    let mut game = load(
        r#"
        state count = 0;
        proc note(e evidence) { when observed(e) set count = count + 1; };
        event check;
        on check call note(bite);
        "#,
    );
    send(&mut game, "eat");
    send(&mut game, "check");
    send(&mut game, "regrow");
    send(&mut game, "check");
    assert_eq!(game.snapshot().values["count"], 1.0);
}

// F267: a scheduled qualification applies on the first time event at which
// the clock reading minus the reading stored at scheduling, both binary64,
// reaches the delay. Scheduled at the inexact reading 28 × 0.1, a delay of 60
// is not yet due after 960 exact steps of 0.0625 (the difference is
// 59.99999999999999) and applies on the 961st, also across a save and restore
// taken while it waits. Scheduled at an exact reading, it applies on the 960th.
#[test]
fn f267_a_delay_is_due_when_the_difference_of_clock_readings_reaches_it() {
    fn steps(game: &mut ReactiveSession, dt: f64, count: usize) {
        for _ in 0..count {
            game.apply("tick", &BTreeMap::from([("dt".into(), dt)]))
                .unwrap();
        }
    }
    fn faded(game: &ReactiveSession) -> bool {
        game.snapshot().bindings["hud"]["faded"] == BindingValue::Bool(true)
    }
    let extra = "on eat qualify bite with faded after 60;\nbind hud.faded = carries(bite, faded);";
    for (before, late, restored) in [(0, false, false), (28, true, false), (28, true, true)] {
        let mut game = load(extra);
        steps(&mut game, 0.1, before);
        let scheduled_at = game.snapshot().elapsed;
        if before == 28 {
            assert_eq!(scheduled_at, 2.800000000000001);
        }
        send(&mut game, "eat");
        steps(&mut game, 0.0625, 480);
        if restored {
            let saved = game.save_json().unwrap();
            game = ReactiveSession::restore_json(&format!("{WORLD}\n{extra}"), &saved).unwrap();
        }
        steps(&mut game, 0.0625, 480);
        let difference = game.snapshot().elapsed - scheduled_at;
        assert_eq!(difference < 60.0, late, "difference {difference}");
        assert_eq!(faded(&game), !late, "after 960 steps from {scheduled_at}");
        steps(&mut game, 0.0625, 1);
        assert!(faded(&game), "after 961 steps from {scheduled_at}");
    }
}

// F267: far from zero the binary64 spacing is coarse, and a delay can apply
// several events early. Scheduled at 1e11, a delay of 1 with steps of 0.001
// applies on the 993rd step.
#[test]
fn f267_far_from_zero_a_delay_can_apply_early() {
    let mut game = ReactiveSession::from_source(&format!(
        "{}\nevent advance dt min 0 max 1e12;\nclock advance every 0.001;\non eat qualify bite with faded after 1;\nbind hud.faded = carries(bite, faded);",
        WORLD.replace("event tick dt min 0 max 0.1;\n", "")
    ))
    .unwrap();
    game.apply("advance", &BTreeMap::from([("dt".into(), 1e11)]))
        .unwrap();
    send(&mut game, "eat");
    let mut applied = None;
    for step in 1..=1000 {
        game.apply("advance", &BTreeMap::from([("dt".into(), 0.001)]))
            .unwrap();
        if game.snapshot().bindings["hud"]["faded"] == BindingValue::Bool(true) {
            applied = Some(step);
            break;
        }
    }
    assert_eq!(applied, Some(993));
}
