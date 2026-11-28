//! D116: Routed Work 0.1 said only that a guard's route ends at the first term
//! of any other form, a rule that covers a `true`, but 1.5.0 reads past a
//! `true`, written out, in parentheses or named by a define. A `true` is true
//! whatever the payload is and rests on nothing, so it neither joins the route
//! nor ends it. spec/caveat-routed-work-0.1.md ("Routes", "Changes") now says
//! so. These tests pin that reading, so that sessions made with 1.5.0, and the
//! saves it wrote, keep their outcomes.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const DECLARATION: &str = "work skips other routes;\n";

/// The steps of `touch`, the procedure each member's second rule calls.
const TOUCH_STEPS: usize = 40;

/// Members in the tests that reach the work limit.
const MEMBERS: usize = 100;

/// Two rules per member, each guarded by `guard`, which may read the event's
/// `report` and `loud`, with `head` before the declarations. The first sets
/// the member's value and the second calls `touch`. A copy whose guard is
/// evaluated counts 2 + `TOUCH_STEPS` steps, whether its guard holds or not
/// (Reactive 0.7), so `MEMBERS` members pass the 4,096-step work limit unless
/// the route sends the other members' copies elsewhere.
fn reports(members: usize, declared: bool, head: &str, guard: &str) -> String {
    let mut source = String::new();
    if declared {
        source.push_str(DECLARATION);
    }
    source.push_str(head);
    source.push_str(
        "place desk kind report;\nclaim ok;\nstate n = 0;\nevent retract report kind report, loud min -1 max 1;\nproc touch() {",
    );
    for _ in 0..TOUCH_STEPS {
        source.push_str(" set n = n + 1;");
    }
    source.push_str(" };\n");
    for index in 0..members {
        source.push_str(&format!("entity r{index} kind report at desk;\n"));
    }
    source.push_str(&format!(
        "for report as $r {{\n    state $r_v = 1;\n    on retract when {guard} set $r_v = 0;\n    on retract when {guard} call touch();\n}};\n"
    ));
    source
}

fn load(source: &str, guard: &str, declared: bool) -> ReactiveSession {
    ReactiveSession::from_source(source)
        .unwrap_or_else(|error| panic!("{guard} (declared {declared}): {error:?}"))
}

fn outcome(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    serde_json::to_value(session.dispatch_outcome_json(event, payload).unwrap()).unwrap()
}

const PAYLOAD: &str = r#"{"report": "r5", "loud": 0}"#;
const ENABLED: &str = "define enabled = true;\n";
const BY_MEMBER: &str = "report == report.$r";
const BY_MEMBER_WHEN_QUIET: &str = "loud == 0 and report == report.$r";
const BY_MEMBER_WHEN_NOT_LOW: &str = "loud >= 0 and report == report.$r";

/// Whether `message` refuses the event where the copies of every member
/// count: at rule 196, the call of the 98th member, whose steps take the count
/// past 4,096. The message also names the procedure's step when that copy's
/// guard holds and the call runs.
fn over_the_limit(message: &Value) -> bool {
    let message = message.as_str().unwrap_or_default();
    message.starts_with("event retract, rule 196: ")
        && message.ends_with("event exceeds work limit 4096")
}

/// Guards whose route names the member: the define each needs, the guard,
/// and the same guard without its `true`. The first, and the two whose
/// member term comes before the `true`, are controls: they route the same on
/// a runtime that ends the route at a `true`.
const ROUTED: [(&str, &str, &str); 11] = [
    ("", BY_MEMBER, BY_MEMBER),
    ("", "true and report == report.$r", BY_MEMBER),
    ("", "(true and report == report.$r)", BY_MEMBER),
    (ENABLED, "enabled and report == report.$r", BY_MEMBER),
    (
        "define on = true;\ndefine enabled = on;\n",
        "enabled and report == report.$r",
        BY_MEMBER,
    ),
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
    (
        "",
        "report == report.$r and true and loud >= 0",
        BY_MEMBER_WHEN_NOT_LOW,
    ),
];

/// Guards with no route that names the member, because the run ends at a
/// term of another form first or holds no `NAME == VALUE` term, so every copy
/// counts: the definition each needs, the guard, and a guard that decides the
/// same.
const UNROUTED: [(&str, &str, &str); 8] = [
    ("", "1 == 1 and report == report.$r", BY_MEMBER),
    ("", "not false and report == report.$r", BY_MEMBER),
    ("", "false or report == report.$r", BY_MEMBER),
    (
        "fn enabled() = true;\n",
        "enabled() and report == report.$r",
        BY_MEMBER,
    ),
    ("", BY_MEMBER_WHEN_NOT_LOW, BY_MEMBER_WHEN_NOT_LOW),
    (
        "",
        "true and loud >= 0 and report == report.$r",
        BY_MEMBER_WHEN_NOT_LOW,
    ),
    ("", "true and loud > 0", "loud > 0"),
    ("", "true", "true"),
];

#[test]
fn a_true_is_read_past_so_copies_routed_elsewhere_cost_no_work() {
    for (head, guard, _) in ROUTED {
        let source = reports(MEMBERS, true, head, guard);
        let mut session = load(&source, guard, true);
        let result = outcome(&mut session, "retract", PAYLOAD);
        assert_eq!(
            result["outcome"], "accepted",
            "{guard}: {}",
            result["message"]
        );
        let values = &result["snapshot"]["values"];
        assert_eq!(values["r5_v"], 0.0, "{guard}");
        assert_eq!(values["r4_v"], 1.0, "{guard}");
        assert_eq!(values["r99_v"], 1.0, "{guard}");
        assert_eq!(values["n"], TOUCH_STEPS as f64, "{guard}");
    }
}

#[test]
fn a_term_of_another_form_still_ends_the_route() {
    for (head, guard, _) in UNROUTED {
        let source = reports(MEMBERS, true, head, guard);
        let mut session = load(&source, guard, true);
        let result = outcome(&mut session, "retract", PAYLOAD);
        assert_eq!(
            result["outcome"], "rejected",
            "{guard}: {}",
            result["message"]
        );
        assert_eq!(result["origin"], "limit", "{guard}");
        assert_eq!(result["code"], "work_limit", "{guard}");
        assert!(
            over_the_limit(&result["message"]),
            "{guard}: {}",
            result["message"]
        );
    }
}

#[test]
fn without_the_declaration_every_copy_counts() {
    for (head, guard, _) in ROUTED.into_iter().chain(UNROUTED) {
        let source = reports(MEMBERS, false, head, guard);
        let mut session = load(&source, guard, false);
        let result = outcome(&mut session, "retract", PAYLOAD);
        assert_eq!(
            result["outcome"], "rejected",
            "{guard}: {}",
            result["message"]
        );
        assert_eq!(result["code"], "work_limit", "{guard}");
        assert!(
            over_the_limit(&result["message"]),
            "{guard}: {}",
            result["message"]
        );
    }
}

/// A session whose first event, `PAYLOAD`, is accepted only because the route
/// reads past the `true` that `enabled` names.
fn relies_on_it() -> String {
    reports(MEMBERS, true, ENABLED, "enabled and report == report.$r")
}

/// Restores `saved` and plays on with a second report, which is accepted as
/// the first was.
fn plays_on(source: &str, form: &str, saved: &str) {
    let mut restored = ReactiveSession::restore_json(source, saved)
        .unwrap_or_else(|error| panic!("{form}: {error:?}"));
    let result = outcome(&mut restored, "retract", r#"{"report": "r7", "loud": 0}"#);
    assert_eq!(
        result["outcome"], "accepted",
        "{form}: {}",
        result["message"]
    );
    let values = &result["snapshot"]["values"];
    assert_eq!(values["r5_v"], 0.0, "{form}");
    assert_eq!(values["r7_v"], 0.0, "{form}");
    assert_eq!(values["r4_v"], 1.0, "{form}");
    assert_eq!(values["n"], (2 * TOUCH_STEPS) as f64, "{form}");
}

fn schema(saved: &str) -> String {
    let saved: Value = serde_json::from_str(saved).unwrap();
    saved["schema"].as_str().unwrap().to_owned()
}

#[test]
fn a_session_that_relies_on_it_restores_from_each_save() {
    let source = relies_on_it();
    let mut session = load(&source, "enabled", true);
    let result = outcome(&mut session, "retract", PAYLOAD);
    assert_eq!(result["outcome"], "accepted", "{}", result["message"]);
    for (form, saved) in [
        ("caveat-reactive-save/0.1", session.save_json().unwrap()),
        (
            "caveat-replay-save/0.1",
            session.save_compact_json().unwrap(),
        ),
        (
            "caveat-replay-save/0.2",
            session.save_compact_positional_json().unwrap(),
        ),
    ] {
        assert_eq!(schema(&saved), form);
        plays_on(&source, form, &saved);
    }
}

/// The saves 1.5.0 wrote for `relies_on_it()` after `PAYLOAD`, one of each
/// form. A save binds its source's SHA-256, so if `reports` changes, write
/// them again with 1.5.0.
const SAVES_FROM_1_5_0: [(&str, &str); 3] = [
    (
        "caveat-reactive-save/0.1",
        r#"{"schema":"caveat-reactive-save/0.1","source_id":"fnv1a64:2df65b83e163dbed:4130","source_sha256":"c5cb222dc33d54c23869d6984f3015b9cd11fc8dcd09f80686f1869e6ea2538e","sequence":1,"last_event":"retract","elapsed":0.0,"states":{"n":{"value":40.0},"r5_v":{"value":0.0}},"graph":{}}"#,
    ),
    (
        "caveat-replay-save/0.1",
        r#"{"schema":"caveat-replay-save/0.1","source_id":"fnv1a64:2df65b83e163dbed:4130","source_sha256":"c5cb222dc33d54c23869d6984f3015b9cd11fc8dcd09f80686f1869e6ea2538e","events":[["retract",{"loud":0,"report":6}]]}"#,
    ),
    (
        "caveat-replay-save/0.2",
        r#"{"schema":"caveat-replay-save/0.2","sha256":"xcsiLcM9VMI4adaYTzAVuc0R_I3NCfgGhvGGnm6iU44","events":"060"}"#,
    ),
];

#[test]
fn the_saves_1_5_0_wrote_restore_and_play_on() {
    let source = relies_on_it();
    for (form, saved) in SAVES_FROM_1_5_0 {
        assert_eq!(schema(saved), form);
        plays_on(&source, form, saved);
    }
}

/// Members in `play`, few enough that every event stays under the limit.
const PLAY_MEMBERS: usize = 30;

/// The values after each of three events, every one under the work limit.
fn play(declared: bool, head: &str, guard: &str) -> Vec<Value> {
    let source = reports(PLAY_MEMBERS, declared, head, guard);
    let mut session = load(&source, guard, declared);
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
            "{guard} (declared {declared}) {payload}: {}",
            result["message"]
        );
        result["snapshot"]["values"].clone()
    })
    .collect()
}

#[test]
fn the_guards_decide_as_before_under_the_limit() {
    // Only the count depends on the route. With the declaration or without
    // it, each guard changes the same values as the same guard without its
    // `true`.
    let by_member = play(false, "", BY_MEMBER);
    let quiet = play(false, "", BY_MEMBER_WHEN_QUIET);
    let not_low = play(false, "", BY_MEMBER_WHEN_NOT_LOW);
    assert_ne!(by_member, quiet);
    assert_ne!(by_member, not_low);
    assert_ne!(quiet, not_low);
    for (head, guard, same) in ROUTED.into_iter().chain(UNROUTED) {
        let expected = play(false, "", same);
        for declared in [true, false] {
            assert_eq!(
                play(declared, head, guard),
                expected,
                "{guard} (declared {declared})"
            );
        }
    }
}
