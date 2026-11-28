// D116: Routed Work 0.1 now says that a `true` in a guard's route, written
// out, in parentheses or named by a define, is passed over: it holds whatever
// the payload is and rests on nothing, so it neither joins the route nor ends
// it (spec/caveat-routed-work-0.1.md, Routes). That is how 1.5.0 reads
// guards. Before, the spec's text said only that the run ends at the first
// term of any other form, which read as if a `true` ended it, as it did on
// main before #501. These tests pin 1.5.0's reading, so that sessions and
// saves made with 1.5.0 keep their outcomes.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const DECLARATION: &str = "work skips other routes;\n";

/// One `retract` rule per member, each guarded by `guard`, which may read the
/// event's `report` and `loud`, with `head` before the declarations.
fn reports(members: usize, declared: bool, head: &str, guard: &str) -> String {
    let mut source = String::new();
    if declared {
        source.push_str(DECLARATION);
    }
    source.push_str(head);
    source.push_str(
        "place desk kind report;\nclaim ok;\nevent retract report kind report, loud min -1 max 1;\n",
    );
    for index in 0..members {
        source.push_str(&format!("entity r{index} kind report at desk;\n"));
    }
    source.push_str(&format!(
        "for report as $r {{\n    state $r_v = 1;\n    on retract when {guard} set $r_v = 0;\n}};\n"
    ));
    source
}

fn outcome(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    serde_json::to_value(session.dispatch_outcome_json(event, payload).unwrap()).unwrap()
}

const PAYLOAD: &str = r#"{"report": "r5", "loud": 0}"#;
const ENABLED: &str = "define enabled = true;\n";
const BY_MEMBER: &str = "report == report.$r";
const BY_MEMBER_WHEN_QUIET: &str = "loud == 0 and report == report.$r";
const BY_MEMBER_WHEN_NOT_LOW: &str = "loud >= 0 and report == report.$r";

/// Guards whose route names the member, some with a `true` passed over: the
/// define each needs, the guard, and the same guard without the `true`.
const ROUTED: [(&str, &str, &str); 9] = [
    ("", BY_MEMBER, BY_MEMBER),
    ("", "true and report == report.$r", BY_MEMBER),
    ("", "(true and report == report.$r)", BY_MEMBER),
    (ENABLED, "enabled and report == report.$r", BY_MEMBER),
    ("", "true and true and report == report.$r", BY_MEMBER),
    ("", "report == report.$r and true", BY_MEMBER),
    (ENABLED, "report == report.$r and enabled", BY_MEMBER),
    (
        "",
        "loud == 0 and true and report == report.$r",
        BY_MEMBER_WHEN_QUIET,
    ),
    (
        "",
        "loud == 0 and (true and report == report.$r)",
        BY_MEMBER_WHEN_QUIET,
    ),
];

/// Guards whose run ends at a term of another form before the term that
/// names the member, so every copy counts, with a guard that decides the same.
const UNROUTED: [(&str, &str, &str); 5] = [
    ("", "1 == 1 and report == report.$r", BY_MEMBER),
    ("", "not false and report == report.$r", BY_MEMBER),
    ("", "false or report == report.$r", BY_MEMBER),
    ("", BY_MEMBER_WHEN_NOT_LOW, BY_MEMBER_WHEN_NOT_LOW),
    (
        "",
        "true and loud >= 0 and report == report.$r",
        BY_MEMBER_WHEN_NOT_LOW,
    ),
];

#[test]
fn d116_a_true_is_passed_over_so_copies_routed_elsewhere_cost_no_work() {
    for (head, guard, _) in ROUTED {
        let source = reports(4_200, true, head, guard);
        let mut session = ReactiveSession::from_source(&source).unwrap();
        let result = outcome(&mut session, "retract", PAYLOAD);
        assert_eq!(
            result["outcome"], "accepted",
            "{guard}: {}",
            result["message"]
        );
        let values = &result["snapshot"]["values"];
        assert_eq!(values["r5_v"], 0.0, "{guard}");
        assert_eq!(values["r4_v"], 1.0, "{guard}");
        assert_eq!(values["r4199_v"], 1.0, "{guard}");
    }
}

#[test]
fn d116_a_term_of_another_form_still_ends_the_route() {
    for (head, guard, _) in UNROUTED {
        let source = reports(4_200, true, head, guard);
        let mut session = ReactiveSession::from_source(&source).unwrap();
        let result = outcome(&mut session, "retract", PAYLOAD);
        assert_eq!(
            result["outcome"], "rejected",
            "{guard}: {}",
            result["message"]
        );
        assert_eq!(result["origin"], "limit", "{guard}");
        assert_eq!(result["code"], "work_limit", "{guard}");
        assert_eq!(
            result["message"], "event retract, rule 4097: event exceeds work limit 4096",
            "{guard}"
        );
    }
}

#[test]
fn d116_without_the_declaration_every_copy_counts() {
    for (head, guard, _) in ROUTED.into_iter().chain(UNROUTED) {
        let source = reports(4_200, false, head, guard);
        let mut session = ReactiveSession::from_source(&source).unwrap();
        let result = outcome(&mut session, "retract", PAYLOAD);
        assert_eq!(
            result["outcome"], "rejected",
            "{guard}: {}",
            result["message"]
        );
        assert_eq!(result["code"], "work_limit", "{guard}");
    }
}

#[test]
fn d116_a_session_that_relies_on_it_restores_from_each_save() {
    // A session whose event was accepted because a `true` was passed over
    // saves as any session does, and each of its saves restores and plays on
    // with the same outcomes.
    let source = reports(4_200, true, ENABLED, "enabled and report == report.$r");
    let mut session = ReactiveSession::from_source(&source).unwrap();
    assert_eq!(
        outcome(&mut session, "retract", PAYLOAD)["outcome"],
        "accepted"
    );
    for saved in [
        session.save_json().unwrap(),
        session.save_compact_json().unwrap(),
        session.save_compact_positional_json().unwrap(),
    ] {
        let mut restored = ReactiveSession::restore_json(&source, &saved).unwrap();
        let result = outcome(&mut restored, "retract", r#"{"report": "r7", "loud": 0}"#);
        assert_eq!(result["outcome"], "accepted", "{}", result["message"]);
        let values = &result["snapshot"]["values"];
        assert_eq!(values["r5_v"], 0.0);
        assert_eq!(values["r7_v"], 0.0);
        assert_eq!(values["r4_v"], 1.0);
    }
}

/// The values after each of three events, with 100 members, so that every
/// event is under the work limit.
fn play(declared: bool, head: &str, guard: &str) -> Vec<Value> {
    let source = reports(100, declared, head, guard);
    let mut session = ReactiveSession::from_source(&source).unwrap();
    [
        r#"{"report": "r5", "loud": 0}"#,
        r#"{"report": "r7", "loud": 1}"#,
        r#"{"report": "r0", "loud": -1}"#,
    ]
    .iter()
    .map(|payload| {
        let result = outcome(&mut session, "retract", payload);
        assert_eq!(
            result["outcome"], "accepted",
            "{guard} {payload}: {}",
            result["message"]
        );
        result["snapshot"]["values"].clone()
    })
    .collect()
}

#[test]
fn d116_the_guards_decide_as_before_under_the_limit() {
    // Only the count depends on the route. With the declaration or without
    // it, each guard changes the same values as the same guard without its
    // `true`.
    let quiet = play(false, "", BY_MEMBER_WHEN_QUIET);
    let not_low = play(false, "", BY_MEMBER_WHEN_NOT_LOW);
    assert_ne!(play(false, "", BY_MEMBER), quiet);
    assert_ne!(quiet, not_low);
    for (head, guard, same) in ROUTED.into_iter().chain(UNROUTED) {
        let expected = play(false, "", same);
        for declared in [true, false] {
            assert_eq!(play(declared, head, guard), expected, "{guard}");
        }
    }
}
