// Routed Work 0.1 (spec/caveat-routed-work-0.1.md): with `work skips other
// routes;`, a top-level rule whose route the payload makes false costs no
// work. Gap 2b of the third agent test: one routed rule over 4,200 members was
// refused with `limit/work_limit` at rule 4,097, though only one copy could
// run. A program without the declaration counts every rule, as in 1.4; every
// other rule still counts; and a routed program gives the same outcomes and
// snapshots with the declaration, without it, and as a twin whose selection
// is not a route.
use caveat_runtime::link::{self, BundlePart};
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const DECLARATION: &str = "work skips other routes;\n";

/// One `retract` rule per member, selected by `report`, over `members`
/// members: through a routed block, or by hand with the member constant.
fn reports(members: usize, routed: bool) -> String {
    let mut source =
        String::from("place desk kind report;\nclaim ok;\nevent retract report kind report;\n");
    for index in 0..members {
        source.push_str(&format!("entity r{index} kind report at desk;\n"));
    }
    if routed {
        source.push_str("for report as $r routed by report {\n    state $r_v = 1;\n    on retract set $r_v = 0;\n};\n");
    } else {
        source.push_str("for report as $r {\n    state $r_v = 1;\n    on retract when report == report.$r set $r_v = 0;\n};\n");
    }
    source
}

fn outcome(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    serde_json::to_value(session.dispatch_outcome_json(event, payload).unwrap()).unwrap()
}

#[test]
fn gap_2b_routed_copies_for_other_members_cost_no_work() {
    for routed in [true, false] {
        let source = format!("{DECLARATION}{}", reports(4_200, routed));
        let mut session = ReactiveSession::from_source(&source).unwrap();
        let result = outcome(&mut session, "retract", r#"{"report": "r5"}"#);
        assert_eq!(
            result["outcome"], "accepted",
            "routed={routed}: {}",
            result["message"]
        );
        let values = &result["snapshot"]["values"];
        assert_eq!(values["r5_v"], 0.0, "routed={routed}");
        assert_eq!(values["r4_v"], 1.0, "routed={routed}");
        assert_eq!(values["r4199_v"], 1.0, "routed={routed}");
        // The last member, whose copy is the last rule, is reached too.
        let result = outcome(&mut session, "retract", r#"{"report": "r4199"}"#);
        assert_eq!(result["outcome"], "accepted", "routed={routed}");
        assert_eq!(
            result["snapshot"]["values"]["r4199_v"], 0.0,
            "routed={routed}"
        );
    }
}

#[test]
fn without_the_declaration_every_copy_still_counts() {
    // The 1.4 count, kept for every program that does not opt in.
    for routed in [true, false] {
        let mut session = ReactiveSession::from_source(&reports(4_200, routed)).unwrap();
        let result = outcome(&mut session, "retract", r#"{"report": "r5"}"#);
        assert_eq!(result["outcome"], "rejected", "routed={routed}");
        assert_eq!(result["origin"], "limit", "routed={routed}");
        assert_eq!(result["code"], "work_limit", "routed={routed}");
        // The same program under the limit is accepted either way.
        let mut session = ReactiveSession::from_source(&reports(4_000, routed)).unwrap();
        let result = outcome(&mut session, "retract", r#"{"report": "r5"}"#);
        assert_eq!(result["outcome"], "accepted", "routed={routed}");
    }
}

#[test]
fn a_skipped_call_in_a_windowed_program_counts_no_work_either() {
    // A windowed program reads a skipped effect, and each step of a procedure
    // it calls, for its withdrawal collector; with the declaration, a rule
    // routed elsewhere still counts none of that.
    let mut source = format!(
        "{DECLARATION}journal window 4;\nstate n = 0;\nstate last = -1;\nproc bump(member) {{ set n = n + 1; set last = member; }};\n"
    );
    source.push_str(&reports(4_200, true).replace(
        "    on retract set $r_v = 0;\n",
        "    on retract set $r_v = 0;\n    on retract call bump($index);\n",
    ));
    let mut session = ReactiveSession::from_source(&source).unwrap();
    let result = outcome(&mut session, "retract", r#"{"report": "r7"}"#);
    assert_eq!(result["outcome"], "accepted", "{}", result["message"]);
    assert_eq!(result["snapshot"]["values"]["n"], 1.0);
    // `$index` counts members from 1, and r7 is the eighth.
    assert_eq!(result["snapshot"]["values"]["last"], 8.0);
    assert_eq!(result["snapshot"]["values"]["r7_v"], 0.0);
    let undeclared = source.replacen(DECLARATION, "", 1);
    let mut session = ReactiveSession::from_source(&undeclared).unwrap();
    let result = outcome(&mut session, "retract", r#"{"report": "r7"}"#);
    assert_eq!(result["code"], "work_limit");
}

#[test]
fn signed_numbers_thresholds_and_defined_numbers_are_route_values() {
    for (head, guard) in [
        ("", "loud == -1"),
        ("", "loud == +1"),
        ("threshold quiet = -1;\n", "loud == quiet"),
        ("define quiet = -1;\n", "loud == quiet"),
    ] {
        let source = |declared: bool| {
            let mut source = format!(
                "{}{head}place desk kind report;\nstate n = 0;\nevent ping report kind report, loud min -1 max 1;\n",
                if declared { DECLARATION } else { "" }
            );
            for index in 0..4_200 {
                source.push_str(&format!("entity r{index} kind report at desk;\n"));
            }
            source.push_str(&format!(
                "for report as $r {{\n    on ping when {guard} and report == $index set n = n + 1;\n}};\n"
            ));
            source
        };
        // `loud` is 0, so every copy is routed elsewhere by its first term.
        let mut session = ReactiveSession::from_source(&source(true)).unwrap();
        let result = outcome(&mut session, "ping", r#"{"report": "r3", "loud": 0}"#);
        assert_eq!(
            result["outcome"], "accepted",
            "{guard}: {}",
            result["message"]
        );
        let mut session = ReactiveSession::from_source(&source(false)).unwrap();
        let result = outcome(&mut session, "ping", r#"{"report": "r3", "loud": 0}"#);
        assert_eq!(result["code"], "work_limit", "{guard}");
    }
}

#[test]
fn rules_that_are_not_routed_elsewhere_still_count() {
    // The same 4,200 copies with no selection all run, and are refused at
    // the rule past the budget, as before.
    let mut source = format!("{DECLARATION}place desk kind report;\nstate n = 0;\nevent ping;\n");
    for index in 0..4_200 {
        source.push_str(&format!("entity r{index} kind report at desk;\n"));
    }
    source.push_str("for report as $r {\n    on ping set n = n + 0;\n};\n");
    let mut session = ReactiveSession::from_source(&source).unwrap();
    let result = outcome(&mut session, "ping", "{}");
    assert_eq!(result["outcome"], "rejected");
    assert_eq!(result["origin"], "limit");
    assert_eq!(result["code"], "work_limit");
    // A route the payload matches counts its rule even when the rest of the
    // guard is false, and so does a guard whose selection comes after
    // another term, which ends its route, or is written the other way round,
    // or compares with a state: 4,200 copies of any of them are refused.
    for guard in [
        "loud == 1 and n > 5",
        "n >= 0 and report == $index",
        "$index == report",
        "report == n + $index",
        "loud == n and report == $index",
    ] {
        let mut source = format!(
            "{DECLARATION}place desk kind report;\nstate n = 0;\nevent ping report kind report, loud min 0 max 1;\n",
        );
        for index in 0..4_200 {
            source.push_str(&format!("entity r{index} kind report at desk;\n"));
        }
        source.push_str(&format!(
            "for report as $r {{\n    on ping when {guard} set n = n + 0;\n}};\n"
        ));
        let mut session = ReactiveSession::from_source(&source).unwrap();
        let result = outcome(&mut session, "ping", r#"{"report": "r3", "loud": 1}"#);
        assert_eq!(
            result["code"], "work_limit",
            "{guard}: {}",
            result["outcome"]
        );
    }
}

fn load_error(source: &str) -> String {
    ReactiveSession::from_source(source)
        .err()
        .unwrap_or_else(|| panic!("should not load: {source}"))
}

#[test]
fn the_declaration_is_one_exact_statement() {
    let program = "claim wet;\nevidence gauge from \"the rain gauge\";\nevent go;\n";
    // Declared twice.
    let message = load_error(&format!("{DECLARATION}{program}{DECLARATION}"));
    assert!(
        message.contains("work skips other routes is declared more than once"),
        "{message}"
    );
    // Any other statement that begins with `work` is read as before: a
    // relation about evidence named `work`, and nothing else.
    let source = format!("{program}evidence work from \"a work log\";\nwork supports wet;\nstate routes = 0;\non go set routes = routes + 1;\n");
    let mut session = ReactiveSession::from_source(&source).unwrap();
    assert_eq!(
        outcome(&mut session, "go", "{}")["snapshot"]["values"]["routes"],
        1.0
    );
    for near in [
        "work skips routes;",
        "work skips other routes now;",
        "work;",
    ] {
        let message = load_error(&format!("{program}{near}\n"));
        assert!(
            message.contains("cannot parse statement"),
            "{near}: {message}"
        );
    }
}

fn part(name: &str, source: &str) -> BundlePart {
    BundlePart {
        name: name.into(),
        source: source.into(),
    }
}

#[test]
fn a_module_may_not_declare_it_but_may_use_its_words_as_names() {
    let bundle = link::bundle(&[
        part("m", "module m;\nwork skips other routes;\n"),
        part("main", "use m;\nevent go;\n"),
    ]);
    let message = link::link(&bundle).expect_err("a module may not declare it");
    assert!(
        message.contains("module m declares `work skips other routes;`"),
        "{message}"
    );
    let bundle = link::bundle(&[
        part(
            "m",
            "module m;\nstate work = 0;\nstate skips = 0;\nstate other = 0;\nstate routes = 0;\nevent go;\non go set work = skips + other + routes + 1;\n",
        ),
        part("main", "use m;\nwork skips other routes;\n"),
    ]);
    let linked = link::link(&bundle).expect("the program may declare it");
    let mut session = ReactiveSession::from_source(&linked).unwrap();
    assert_eq!(
        outcome(&mut session, "m__go", "{}")["snapshot"]["values"]["m__work"],
        1.0
    );
}

// A windowed program, so a skipped effect reaches the withdrawal collector,
// with a call in a routed rule. The twin writes its selection `$index ==
// target`, which is not a route, so its guards are evaluated and its skipped
// effects retained the old way. Driven in lockstep, with saves and restores,
// the two give the same outcome and snapshot after every event.
const HEAD: &str = r#"
budget 40;
claim wet;
place field kind field;
entity north kind plot at field;
entity south kind plot at field;
entity east kind plot at field;
evidence gauge from "the rain gauge";
state total = 0;
decisions irrigate limit 24;
journal window 6;
event read target kind plot, value min 0 max 9;
event ping target kind plot, loud min 0 max 1;
event decide target kind plot;
event tick dt min 0 max 0.1;
proc note(c caveat) { examine c cost 1; };
on tick set total = total + 1;
"#;

const BODY: &str = r#"
    caveat $p_fog consequence low;
    readings $p_readings from gauge window 3;
    state $p_n = 0;
    on read when SELECT and value == 9 reject "Nine is not a reading for $p.";
    on read when SELECT set $p_n = value;
    on read when SELECT and (value > 2 or value == 0) sample $p_readings = value supports wet;
    on read when SELECT and value > 4 call note($p_fog);
    on ping when SELECT and loud == 0 examine $p_fog cost 1;
    on ping when SELECT and loud == 1 and has_sample($p_readings) and committed(irrigate) and not reopened(irrigate) reopen irrigate because latest($p_readings);
    evidence $p_recheck from "a recheck of the $p gauge";
    on ping when SELECT and loud == 1 and has_sample($p_readings) reveal $p_recheck;
    on ping when SELECT and loud == 1 and has_sample($p_readings) and not withdrawn(latest($p_readings)) withdraw latest($p_readings) because $p_recheck;
    on decide when SELECT and has_sample($p_readings) and (not committed(irrigate) or reopened(irrigate)) commit irrigate because enough using latest($p_readings) + $index;
"#;

const EVENTS: [(&str, &str); 12] = [
    ("read", r#"{"target": "north", "value": 0}"#),
    ("read", r#"{"target": "south", "value": 3}"#),
    ("read", r#"{"target": "east", "value": 5}"#),
    ("read", r#"{"target": "north", "value": 9}"#),
    ("read", r#"{"target": "east", "value": 7}"#),
    ("ping", r#"{"target": "north", "loud": 1}"#),
    ("ping", r#"{"target": "south", "loud": 0}"#),
    ("ping", r#"{"target": "east", "loud": 1}"#),
    ("decide", r#"{"target": "north"}"#),
    ("decide", r#"{"target": "south"}"#),
    ("decide", r#"{"target": "east"}"#),
    ("tick", r#"{"dt": 0.1}"#),
];

fn program(select: &str) -> String {
    format!(
        "{HEAD}for plot as $p {{{}}};\nbind view.total = total;\n",
        BODY.replace("SELECT", select)
    )
}

/// Takes the source's identity out of an outcome or snapshot: its id and
/// digest, and the archive digests, which name the source they were made
/// under (runtime/src/reactive_archive.rs), wherever they stand.
fn anonymous(mut shown: Value) -> Value {
    match &mut shown {
        Value::Object(object) => {
            object.remove("source_id");
            object.remove("source_sha256");
            for (key, value) in object.iter_mut() {
                if key == "archive_ref" && value.is_string() {
                    *value = Value::from("archive");
                } else {
                    *value = anonymous(value.take());
                }
            }
        }
        Value::Array(items) => {
            for item in items.iter_mut() {
                *item = anonymous(item.take());
            }
        }
        _ => {}
    }
    shown
}

fn snapshot(session: &ReactiveSession) -> Value {
    anonymous(serde_json::to_value(session.snapshot()).unwrap())
}

#[test]
fn a_windowed_routed_program_runs_as_its_unrouted_twin() {
    // The routed program with and without the declaration, and its twin,
    // whose guards are evaluated; none of these events reaches the limit.
    let sources = [
        format!("{DECLARATION}{}", program("target == $index")),
        program("target == $index"),
        format!("{DECLARATION}{}", program("$index == target")),
    ];
    let mut seed: u64 = 0x2b2b_5eed_0a11_d00d;
    let (mut accepted, mut refused) = (0, 0);
    for run in 0..120 {
        let mut sessions: Vec<ReactiveSession> = sources
            .iter()
            .map(|source| ReactiveSession::from_source(source).unwrap())
            .collect();
        let mut trail = Vec::new();
        for step in 0..40 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 1);
            if pick == EVENTS.len() {
                trail.push("save");
                for (session, source) in sessions.iter_mut().zip(&sources) {
                    *session = ReactiveSession::restore_json(source, &session.save_json().unwrap())
                        .unwrap_or_else(|error| {
                            panic!("run {run} step {step}: {error}: {trail:?}")
                        });
                }
                continue;
            }
            let (event, payload) = EVENTS[pick];
            trail.push(payload);
            let at = format!("run {run} step {step} after {event} {payload}: {trail:?}");
            let shown: Vec<(Value, Value)> = sessions
                .iter_mut()
                .map(|session| {
                    let result = anonymous(outcome(session, event, payload));
                    (result, snapshot(session))
                })
                .collect();
            for (index, other) in shown.iter().enumerate().skip(1) {
                assert_eq!(shown[0].0, other.0, "{at}: outcome {index} differs");
                assert_eq!(shown[0].1, other.1, "{at}: snapshot {index} differs");
            }
            if shown[0].0["outcome"] == "accepted" {
                accepted += 1;
            } else {
                refused += 1;
            }
        }
    }
    // Both kinds of outcome are reached, so the comparison is not vacuous.
    assert!(
        accepted > 1_000 && refused > 100,
        "accepted {accepted}, refused {refused}"
    );
}
