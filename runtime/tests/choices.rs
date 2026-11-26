//! `choose(CONDITION, YES, NO)`, `lowest(A, B)` and `highest(A, B)`:
//! comparison enters lineage, not grounds. See spec/caveat-choices-0.1.md.
use caveat_runtime::link::{self, BundlePart};
use caveat_runtime::reactive::{check_source, ReactiveSession};
use caveat_runtime::source_library::{FunctionValue, SourceLibrary};
use serde_json::{json, Value};

fn snapshot(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.snapshot()).unwrap()
}

fn run(session: &mut ReactiveSession, events: &[(&str, &str)]) {
    for (event, payload) in events {
        session
            .dispatch_json(event, payload)
            .unwrap_or_else(|error| panic!("{event} {payload}: {error}"));
    }
}

fn names(value: &Value) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_str().unwrap().to_string())
        .collect()
}

/// Deterministic PRNG so fuzz tests are reproducible across platforms.
struct Lcg(u64);

impl Lcg {
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    fn next_f64(&mut self) -> f64 {
        let bits = self.next_u64();
        // Mix exact integers, ties, tiny decimals, and signed values in [-1e6, 1e6].
        match bits % 7 {
            0 => 0.0,
            1 => -0.0,
            2 => ((bits >> 8) % 21) as f64 - 10.0,
            3 => 42.5,
            _ => {
                let unit = ((bits >> 11) as f64) / ((1u64 << 53) as f64);
                (unit * 2.0 - 1.0) * 1_000_000.0
            }
        }
    }
}

#[test]
fn lowest_and_highest_match_min_and_max_on_random_finite_inputs_and_ties() {
    let source = r#"
evidence ea from "a";
evidence eb from "b";
readings a from ea limit 1;
readings b from eb limit 1;
state lo = 0;
state hi = 0;
state m_lo = 0;
state m_hi = 0;
event put x min -10000000 max 10000000, y min -10000000 max 10000000;
on put sample a = x;
on put sample b = y;
on put set lo = lowest(latest(a), latest(b));
on put set hi = highest(latest(a), latest(b));
on put set m_lo = min(latest(a), latest(b));
on put set m_hi = max(latest(a), latest(b));
"#;
    let mut rng = Lcg(0x00C0_FFEE_2026_1125);
    let mut pairs = vec![
        (0.0, 0.0),
        (0.0, -0.0),
        (-0.0, 0.0),
        (-0.0, -0.0),
        (5.0, 5.0),
        (-3.5, -3.5),
        (-10.0, 10.0),
        (10.0, -10.0),
    ];
    for _ in 0..120 {
        let x = rng.next_f64();
        let y = if rng.next_u64().is_multiple_of(5) {
            x
        } else {
            rng.next_f64()
        };
        pairs.push((x, y));
    }

    for (x, y) in pairs {
        let mut session = ReactiveSession::from_source(source).unwrap();
        let payload = json!({ "x": x, "y": y }).to_string();
        run(&mut session, &[("put", &payload)]);
        let view = snapshot(&session);
        let lo = view["values"]["lo"].as_f64().unwrap();
        let hi = view["values"]["hi"].as_f64().unwrap();
        let m_lo = view["values"]["m_lo"].as_f64().unwrap();
        let m_hi = view["values"]["m_hi"].as_f64().unwrap();
        assert_eq!(lo.to_bits(), m_lo.to_bits(), "lowest({x:?}, {y:?})");
        assert_eq!(hi.to_bits(), m_hi.to_bits(), "highest({x:?}, {y:?})");

        let expected_lo_ground = if x < y { "a@1" } else { "b@1" };
        let expected_hi_ground = if x > y { "a@1" } else { "b@1" };
        assert_eq!(
            names(&view["value_grounds"]["lo"]["evidence"]),
            [expected_lo_ground],
            "lowest grounds for ({x:?}, {y:?})"
        );
        assert_eq!(
            names(&view["value_grounds"]["hi"]["evidence"]),
            [expected_hi_ground],
            "highest grounds for ({x:?}, {y:?})"
        );
        assert_eq!(
            names(&view["qualified_values"]["lo"]["provenance"]["evidence"]),
            ["a@1", "b@1"]
        );
        assert_eq!(
            names(&view["qualified_values"]["hi"]["provenance"]["evidence"]),
            ["a@1", "b@1"]
        );
    }
}

#[test]
fn choose_grounds_and_lineage_for_state_and_commit() {
    let source = r#"
evidence cond_ev from "selector";
evidence yes_ev from "yes branch";
evidence no_ev from "no branch";
caveat cond_cav consequence material;
caveat yes_cav consequence low;
caveat no_cav consequence high;
cond_cav qualifies cond_ev;
yes_cav qualifies yes_ev;
no_cav qualifies no_ev;
readings c from cond_ev limit 2;
readings y from yes_ev limit 2;
readings n from no_ev limit 2;
decisions pick limit 4;
state picked = 0;
event load cv min 0 max 1, yv min 0 max 100, nv min 0 max 100;
event decide;
on load sample c = cv;
on load sample y = yv;
on load sample n = nv;
on decide set picked = choose(latest(c) == 1, latest(y), latest(n));
on decide commit pick because enough using choose(latest(c) == 1, latest(y), latest(n));
"#;
    // True branch chosen: grounds hold only y@1 (+ yes_cav); lineage holds c@1 + y@1, NOT n@1 (short-circuited).
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(
        &mut session,
        &[("load", r#"{"cv":1,"yv":40,"nv":90}"#), ("decide", "{}")],
    );
    let view = snapshot(&session);
    assert_eq!(view["values"]["picked"], json!(40.0));
    assert_eq!(names(&view["value_grounds"]["picked"]["evidence"]), ["y@1"]);
    assert_eq!(
        names(&view["value_grounds"]["picked"]["caveats"]),
        ["yes_cav"]
    );
    assert_eq!(
        names(&view["qualified_values"]["picked"]["provenance"]["evidence"]),
        ["c@1", "y@1"]
    );
    assert_eq!(
        names(&view["qualified_values"]["picked"]["provenance"]["caveats"]),
        ["cond_cav", "yes_cav"]
    );
    assert_eq!(
        names(&view["commitment_grounds"]["pick@1"]["evidence"]),
        ["y@1"]
    );
    assert_eq!(
        names(&view["commitment_grounds"]["pick@1"]["caveats"]),
        ["yes_cav"]
    );
    assert_eq!(
        names(&view["commitment_bases"]["pick@1"]["provenance"]["evidence"]),
        ["c@1", "y@1"]
    );

    // False branch chosen: grounds hold only n@1 (+ no_cav); lineage holds c@1 + n@1, NOT y@1.
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(
        &mut session,
        &[("load", r#"{"cv":0,"yv":40,"nv":90}"#), ("decide", "{}")],
    );
    let view = snapshot(&session);
    assert_eq!(view["values"]["picked"], json!(90.0));
    assert_eq!(names(&view["value_grounds"]["picked"]["evidence"]), ["n@1"]);
    assert_eq!(
        names(&view["value_grounds"]["picked"]["caveats"]),
        ["no_cav"]
    );
    assert_eq!(
        names(&view["commitment_grounds"]["pick@1"]["evidence"]),
        ["n@1"]
    );
    assert_eq!(
        names(&view["commitment_bases"]["pick@1"]["provenance"]["evidence"]),
        ["c@1", "n@1"]
    );
}

#[test]
fn caveats_on_losing_argument_or_condition_respect_irreversible_vs_strict() {
    let source = r#"
evidence ea from "clean";
evidence eb from "doubted";
caveat stale consequence material;
stale qualifies eb;
readings a from ea limit 1;
readings b from eb limit 1;
decisions normal_irrev limit 1 irreversible;
decisions strict_irrev limit 1 irreversible strict;
event load;
event commit_normal;
event commit_strict;
event commit_strict_retained;
on load sample a = 10;
on load sample b = 50;
on commit_normal commit normal_irrev because enough using lowest(latest(a), latest(b));
on commit_strict commit strict_irrev because enough using lowest(latest(a), latest(b));
on commit_strict_retained commit strict_irrev because enough using lowest(latest(a), latest(b)) retaining stale;
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(&mut session, &[("load", "{}")]);
    // Non-strict irreversible succeeds without retaining `stale`, because `b@1` is only in lineage.
    run(&mut session, &[("commit_normal", "{}")]);
    let view = snapshot(&session);
    assert_eq!(
        names(&view["commitment_grounds"]["normal_irrev@1"]["evidence"]),
        ["a@1"]
    );
    assert_eq!(
        names(&view["commitment_grounds"]["normal_irrev@1"]["caveats"]),
        Vec::<String>::new()
    );
    assert_eq!(
        names(&view["commitment_bases"]["normal_irrev@1"]["provenance"]["caveats"]),
        ["stale"]
    );

    // Strict irreversible inspects lineage, so committing without retaining `stale` is refused.
    let refused = session
        .dispatch_outcome_json("commit_strict", "{}")
        .unwrap();
    let refused_json = serde_json::to_value(&refused).unwrap();
    assert_eq!(refused_json["outcome"], json!("rejected"));
    assert_eq!(refused_json["origin"], json!("evaluation"));
    assert_eq!(refused_json["code"], json!("caveated_irreversible"));

    // Retaining `stale` allows strict irreversible commit while grounds remain only `a@1`.
    run(&mut session, &[("commit_strict_retained", "{}")]);
    let view = snapshot(&session);
    assert_eq!(
        names(&view["commitment_grounds"]["strict_irrev@1"]["evidence"]),
        ["a@1"]
    );
}

#[test]
fn withdrawal_of_returned_argument_reaches_decision_while_losing_and_condition_do_not() {
    let source = r#"
evidence ec from "gate";
evidence ea from "low";
evidence eb from "high";
evidence audit from "audit";
readings c from ec limit 1;
readings a from ea limit 1;
readings b from eb limit 1;
decisions d_low limit 2;
decisions d_choose limit 2;
state low_withdrawn = 0;
state choose_withdrawn = 0;
state low_val = 0;
event load;
event decide;
event retract_b;
event retract_c;
event retract_a;
event probe;
on load sample c = 1;
on load sample a = 12;
on load sample b = 40;
on decide set low_val = lowest(latest(a), latest(b));
on decide commit d_low because enough using lowest(latest(a), latest(b));
on decide commit d_choose because enough using choose(latest(c) == 1, latest(a), latest(b));
on retract_b when not observed(audit) reveal audit;
on retract_b withdraw latest(b) because audit;
on retract_c when not observed(audit) reveal audit;
on retract_c withdraw latest(c) because audit;
on retract_a when not observed(audit) reveal audit;
on retract_a withdraw latest(a) because audit;
on probe set low_withdrawn = if(rests_on_withdrawn(d_low), 1, 0);
on probe set choose_withdrawn = if(rests_on_withdrawn(d_choose), 1, 0);
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(&mut session, &[("load", "{}"), ("decide", "{}")]);
    // Withdrawing losing `b` and condition `c` leaves neither decision resting on withdrawn evidence.
    run(
        &mut session,
        &[("retract_b", "{}"), ("retract_c", "{}"), ("probe", "{}")],
    );
    let view = snapshot(&session);
    assert_eq!(view["values"]["low_withdrawn"], json!(0.0));
    assert_eq!(view["values"]["choose_withdrawn"], json!(0.0));
    assert_eq!(view["values"]["low_val"], json!(12.0));
    assert_eq!(
        names(&view["value_grounds"]["low_val"]["caveats"]),
        Vec::<String>::new()
    );

    // Withdrawing returned `a` flips rests_on_withdrawn true on both decisions.
    run(&mut session, &[("retract_a", "{}"), ("probe", "{}")]);
    let view = snapshot(&session);
    assert_eq!(view["values"]["low_withdrawn"], json!(1.0));
    assert_eq!(view["values"]["choose_withdrawn"], json!(1.0));
}

#[test]
fn choose_short_circuits_and_lowest_highest_evaluate_both_eagerly() {
    let source = r#"
evidence eb from "b";
readings b from eb limit 1;
state u unset;
state out = 0;
event safe;
event bad_low;
event bad_high;
on safe set out = choose(has_sample(b), latest(b), 5)
    + choose(true, 10, 1 / 0)
    + choose(false, 1 / 0, 20)
    + choose(is_set(u), u, 7);
on bad_low set out = lowest(1, 1 / 0);
on bad_high set out = highest(1 / 0, 1);
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(&mut session, &[("safe", "{}")]);
    assert_eq!(snapshot(&session)["values"]["out"], json!(42.0));

    for event in ["bad_low", "bad_high"] {
        let outcome = session.dispatch_outcome_json(event, "{}").unwrap();
        let value = serde_json::to_value(&outcome).unwrap();
        assert_eq!(value["outcome"], json!("rejected"), "{event}");
        assert_eq!(value["origin"], json!("evaluation"), "{event}");
        assert_eq!(value["code"], json!("expression"), "{event}");
    }
}

#[test]
fn linear_deep_nesting_and_composition_with_only_if() {
    // 24 right-nested `lowest` calls: verifies linear AST expansion and single-winner grounds.
    let mut source = String::new();
    for i in 1..=24 {
        source.push_str(&format!(
            "evidence e{i} from \"r{i}\";\nreadings r{i} from e{i} limit 1;\n"
        ));
    }
    source.push_str(
        "state best = 0;\nstate nested = 0;\ndecisions route limit 1;\nevent load;\nevent eval;\n",
    );
    for i in 1..=24 {
        let val = if i == 17 { 3 } else { 100 + i };
        source.push_str(&format!("on load sample r{i} = {val};\n"));
    }
    let mut chain = "latest(r24)".to_string();
    for i in (1..24).rev() {
        chain = format!("lowest(latest(r{i}), {chain})");
    }
    source.push_str(&format!("on eval set best = {chain};\n"));
    source.push_str(&format!(
        "on eval commit route because enough using {chain};\n"
    ));
    // Nested choose inside choose condition, lowest inside choose condition, choose inside lowest, and only_if.
    source.push_str(
        "on eval set nested = lowest(\
            choose(choose(lowest(latest(r1), latest(r2)) > 50, 1, 0) == 1, latest(r3), latest(r4)), \
            only_if(latest(r5) > 0, latest(r17))\
        );\n",
    );

    let mut session = ReactiveSession::from_source(&source).unwrap();
    run(&mut session, &[("load", "{}"), ("eval", "{}")]);
    let view = snapshot(&session);
    assert_eq!(view["values"]["best"], json!(3.0));
    assert_eq!(names(&view["value_grounds"]["best"]["evidence"]), ["r17@1"]);
    assert_eq!(
        names(&view["commitment_grounds"]["route@1"]["evidence"]),
        ["r17@1"]
    );
    assert_eq!(
        names(&view["commitment_bases"]["route@1"]["provenance"]["evidence"]).len(),
        24
    );
    assert_eq!(view["values"]["nested"], json!(3.0));
    assert_eq!(
        names(&view["value_grounds"]["nested"]["evidence"]),
        ["r17@1"]
    );
}

#[test]
fn choices_survive_state_compact_and_positional_save_restore() {
    let source = r#"
evidence ea from "a";
evidence eb from "b";
evidence ec from "c";
evidence audit from "audit";
readings a from ea limit 2;
readings b from eb limit 2;
readings c from ec limit 2;
decisions pick limit 4;
state val = 0;
event load av min 0 max 100, bv min 0 max 100, cv min 0 max 1;
event decide;
event retract_b;
on load sample a = av;
on load sample b = bv;
on load sample c = cv;
on decide set val = choose(latest(c) == 1, lowest(latest(a), latest(b)), highest(latest(a), latest(b)));
on decide commit pick because enough using choose(latest(c) == 1, lowest(latest(a), latest(b)), highest(latest(a), latest(b)));
on retract_b reveal audit;
on retract_b withdraw latest(b) because audit;
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(
        &mut session,
        &[
            ("load", r#"{"av":15,"bv":45,"cv":1}"#),
            ("decide", "{}"),
            ("retract_b", "{}"),
        ],
    );
    let expected = snapshot(&session);
    for saved in [
        session.save_json().unwrap(),
        session.save_compact_json().unwrap(),
        session.save_compact_positional_json().unwrap(),
    ] {
        let restored = ReactiveSession::restore_json(source, &saved).unwrap();
        assert_eq!(snapshot(&restored), expected);
    }
}

#[test]
fn authored_choose_lowest_highest_functions_are_kept_everywhere() {
    // Same arity as built-in forms.
    let source = r#"
fn choose(a, b, c) = a * 100 + b * 10 + c;
fn lowest(a, b) = a + b + 1;
fn highest(a, b) = a * b + 2;
fn wrapper(x) = choose(x, 2, 3) + lowest(x, 4) + highest(x, 5);
state out = 0;
event go;
on go set out = wrapper(1);
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(&mut session, &[("go", "{}")]);
    // choose(1,2,3)=123, lowest(1,4)=6, highest(1,5)=7 => 136
    assert_eq!(snapshot(&session)["values"]["out"], json!(136.0));

    // Different arity from built-in forms.
    let diff_arity = r#"
fn choose(x) = x + 10;
fn lowest(x, y, z) = x + y + z;
fn highest() = 7;
state out = 0;
event go;
on go set out = choose(1) + lowest(2, 3, 4) + highest();
"#;
    let mut session = ReactiveSession::from_source(diff_arity).unwrap();
    run(&mut session, &[("go", "{}")]);
    assert_eq!(snapshot(&session)["values"]["out"], json!(27.0));

    // Built-in choose/lowest/highest inside pure functions and SourceLibrary.
    let builtin_in_fn = r#"
fn clamp_like(x, lo, hi) = lowest(highest(x, lo), hi);
fn sign_mag(x) = choose(x >= 0, x, -x);
"#;
    let library = SourceLibrary::from_module_source(builtin_in_fn).unwrap();
    assert_eq!(
        library
            .call_numbers("clamp_like", &[25.0, 0.0, 10.0])
            .unwrap()
            .value,
        FunctionValue::Number(10.0)
    );
    assert_eq!(
        library.call_numbers("sign_mag", &[-7.0]).unwrap().value,
        FunctionValue::Number(7.0)
    );
}

#[test]
fn choices_check_types_arity_and_observation_order() {
    for (body, expected) in [
        ("choose(1, 2, 3)", "boolean"),
        ("choose(true, \"x\", 1)", "number"),
        ("choose(true, 1, \"x\")", "number"),
        ("choose(true, true, false)", "number"),
        ("lowest(true, 1)", "number"),
        ("highest(1, \"x\")", "number"),
        ("choose(true, 1)", "choose"),
        ("lowest(1)", "lowest"),
        ("highest(1, 2, 3)", "highest"),
    ] {
        let source = format!("state s = 0;\nevent go;\non go set s = {body};\n");
        let error = ReactiveSession::from_source(&source).expect_err(body);
        assert!(
            error.to_lowercase().contains(expected),
            "{body}: expected '{expected}' in '{error}'"
        );
    }

    // Observation order: lowest evaluates both arguments unconditionally, choose only its condition.
    let bad_order = r#"
evidence e from "never observed";
state s = 0;
event go;
on go set s = lowest(1, qualified(2, e));
"#;
    assert!(ReactiveSession::from_source(bad_order).is_err());

    let ok_order = r#"
evidence e from "never observed";
state s = 0;
event go;
on go set s = choose(false, qualified(1, e), 5);
"#;
    let mut session = ReactiveSession::from_source(ok_order).unwrap();
    run(&mut session, &[("go", "{}")]);
    assert_eq!(snapshot(&session)["values"]["s"], json!(5.0));
}

#[test]
fn check_c013_treats_choose_as_guarding_and_lowest_highest_as_eager() {
    let guarded = r#"
state u unset;
state out = 0;
event go;
on go set out = choose(is_set(u), u, 0) + choose(not is_set(u), 0, u);
"#;
    let report = check_source(guarded).unwrap();
    assert!(
        report.diagnostics.iter().all(|w| w.code != "C013"),
        "{:?}",
        report.diagnostics
    );

    for expr in ["lowest(u, 0)", "highest(0, u)"] {
        let unguarded =
            format!("state u unset;\nstate out = 0;\nevent go;\non go set out = {expr};\n");
        let report = check_source(&unguarded).unwrap();
        assert!(
            report.diagnostics.iter().any(|w| w.code == "C013"),
            "{expr}: expected C013, got {:?}",
            report.diagnostics
        );
    }
}

#[test]
fn modules_work_with_builtin_choices_authored_functions_and_ordinary_declarations() {
    // Module using built-in lowest/highest/choose + module defining fn lowest + module declaring event choose / state lowest / define highest.
    let bundle = link::bundle(&[
        BundlePart {
            name: "pick".into(),
            source: "module pick;\nstate v = 0;\nevent run;\non run set v = lowest(8, 3);\n".into(),
        },
        BundlePart {
            name: "custom".into(),
            source: "module custom;\nfn lowest(a, b) = a + b;\nstate v = 0;\nevent run;\non run set v = lowest(8, 3);\n".into(),
        },
        BundlePart {
            name: "names".into(),
            source: "module names;\nstate lowest = 5;\ndefine highest = lowest + 1;\nevent choose;\non choose set lowest = highest;\n".into(),
        },
        BundlePart {
            name: "main".into(),
            source: "use pick;\nuse custom;\nuse names;\nstate root_val = 0;\nevent go;\non go set root_val = lowest(10, 4) + highest(2, 9) + choose(true, 1, 2);\n".into(),
        },
    ]);
    let mut session = ReactiveSession::from_source(&bundle).unwrap();
    run(
        &mut session,
        &[
            ("go", "{}"),
            ("pick__run", "{}"),
            ("custom__run", "{}"),
            ("names__choose", "{}"),
        ],
    );
    let view = snapshot(&session);
    assert_eq!(view["values"]["root_val"], json!(14.0));
    assert_eq!(view["values"]["pick__v"], json!(3.0));
    assert_eq!(view["values"]["custom__v"], json!(11.0));
    assert_eq!(view["values"]["names__lowest"], json!(6.0));
}

#[test]
fn third_agent_test_bridge_record_rests_only_on_returned_report_per_part() {
    // Reproduces the motivating bridge scenario from spec/caveat-choices-0.1.md:
    // 3 bridge parts (deck, pier, cable), each with 3 inspection reports.
    // Retracting non-lowest reports must stay out of commitment_grounds and leave
    // rests_on_withdrawn false on the bridge rating and downstream convoy permit;
    // retracting the single winning report flips rests_on_withdrawn true downstream.
    let source = r#"
evidence deck_a_src from "inspector A on deck";
evidence deck_b_src from "inspector B on deck";
evidence deck_c_src from "inspector C on deck";
evidence pier_a_src from "inspector A on pier";
evidence pier_b_src from "inspector B on pier";
evidence pier_c_src from "inspector C on pier";
evidence cable_a_src from "inspector A on cable";
evidence cable_b_src from "inspector B on cable";
evidence cable_c_src from "inspector C on cable";
evidence qa_audit from "QA retraction notice";

readings deck_a from deck_a_src limit 1;
readings deck_b from deck_b_src limit 1;
readings deck_c from deck_c_src limit 1;
readings pier_a from pier_a_src limit 1;
readings pier_b from pier_b_src limit 1;
readings pier_c from pier_c_src limit 1;
readings cable_a from cable_a_src limit 1;
readings cable_b from cable_b_src limit 1;
readings cable_c from cable_c_src limit 1;

decisions deck_rating limit 2;
decisions pier_rating limit 2;
decisions cable_rating limit 2;
decisions bridge_rating limit 2;
decisions convoy_permit limit 2;

state bridge_doubted = 0;
state convoy_doubted = 0;

event inspect_all;
event rate_bridge;
event issue_permit;
event retract_non_lowest;
event retract_winner;
event check_doubt;

on inspect_all sample deck_a = 80;
on inspect_all sample deck_b = 65;
on inspect_all sample deck_c = 90;
on inspect_all sample pier_a = 72;
on inspect_all sample pier_b = 45;
on inspect_all sample pier_c = 60;
on inspect_all sample cable_a = 88;
on inspect_all sample cable_b = 70;
on inspect_all sample cable_c = 70;

on rate_bridge commit deck_rating because enough using lowest(latest(deck_a), lowest(latest(deck_b), latest(deck_c)));
on rate_bridge commit pier_rating because enough using lowest(latest(pier_a), lowest(latest(pier_b), latest(pier_c)));
on rate_bridge commit cable_rating because enough using lowest(latest(cable_a), lowest(latest(cable_b), latest(cable_c)));
on rate_bridge commit bridge_rating because enough using
    lowest(grounds_of(deck_rating), lowest(grounds_of(pier_rating), grounds_of(cable_rating)));

on issue_permit when grounds_of(bridge_rating) < 40 reject "Bridge rating below convoy minimum.";
on issue_permit commit convoy_permit because enough using grounds_of(bridge_rating);

on retract_non_lowest when not observed(qa_audit) reveal qa_audit;
on retract_non_lowest withdraw latest(deck_a) because qa_audit;
on retract_non_lowest withdraw latest(deck_c) because qa_audit;
on retract_non_lowest withdraw latest(pier_a) because qa_audit;
on retract_non_lowest withdraw latest(pier_c) because qa_audit;
on retract_non_lowest withdraw latest(cable_a) because qa_audit;
on retract_non_lowest withdraw latest(cable_b) because qa_audit;

on retract_winner when not observed(qa_audit) reveal qa_audit;
on retract_winner withdraw latest(pier_b) because qa_audit;

on check_doubt set bridge_doubted = if(rests_on_withdrawn(bridge_rating), 1, 0);
on check_doubt set convoy_doubted = if(rests_on_withdrawn(convoy_permit), 1, 0);
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(
        &mut session,
        &[
            ("inspect_all", "{}"),
            ("rate_bridge", "{}"),
            ("issue_permit", "{}"),
        ],
    );
    let view = snapshot(&session);
    // Each part rests ONLY on its single winning report (tie on cable_b=70 vs cable_c=70 goes to B = cable_c).
    assert_eq!(
        names(&view["commitment_grounds"]["deck_rating@1"]["evidence"]),
        ["deck_b@1"]
    );
    assert_eq!(
        names(&view["commitment_grounds"]["pier_rating@1"]["evidence"]),
        ["pier_b@1"]
    );
    assert_eq!(
        names(&view["commitment_grounds"]["cable_rating@1"]["evidence"]),
        ["cable_c@1"]
    );
    // Overall bridge rating (45) and convoy permit rest ONLY on pier_b@1 (not 9 reports!).
    assert_eq!(
        view["commitment_bases"]["bridge_rating@1"]["value"],
        json!(45.0)
    );
    assert_eq!(
        names(&view["commitment_grounds"]["bridge_rating@1"]["evidence"]),
        ["pier_b@1"]
    );
    assert_eq!(
        names(&view["commitment_grounds"]["convoy_permit@1"]["evidence"]),
        ["pier_b@1"]
    );

    // Retracting all 6 non-winning reports leaves rests_on_withdrawn false on bridge_rating and convoy_permit.
    run(
        &mut session,
        &[("retract_non_lowest", "{}"), ("check_doubt", "{}")],
    );
    let view = snapshot(&session);
    assert_eq!(view["values"]["bridge_doubted"], json!(0.0));
    assert_eq!(view["values"]["convoy_doubted"], json!(0.0));

    // Retracting pier_b@1 flips rests_on_withdrawn true on both.
    run(
        &mut session,
        &[("retract_winner", "{}"), ("check_doubt", "{}")],
    );
    let view = snapshot(&session);
    assert_eq!(view["values"]["bridge_doubted"], json!(1.0));
    assert_eq!(view["values"]["convoy_doubted"], json!(1.0));
    let refused = session.dispatch_outcome_json("issue_permit", "{}").unwrap();
    assert_eq!(
        serde_json::to_value(&refused).unwrap()["origin"],
        json!("evaluation")
    );
}
