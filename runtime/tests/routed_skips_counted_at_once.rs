//! Compiler step 3.3: a rule the payload routes elsewhere, in a program
//! without a window, has its work counted at once (its own step and, through
//! a call, each step of its procedure) instead of being skipped step by step.
//! When the budget cannot hold the whole count, the rule is skipped step by
//! step as before, so the work limit refuses the same rule with the same
//! text. A route reads its parameters by their place among the event's
//! declared parameters.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const DECLARATION: &str = "work skips other routes;\n";

fn outcome(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    serde_json::to_value(session.dispatch_outcome_json(event, payload).unwrap()).unwrap()
}

fn refused_at(result: &Value, event: &str, rule: usize) {
    assert_eq!(result["outcome"], "rejected", "{result}");
    assert_eq!(result["origin"], "limit", "{result}");
    assert_eq!(result["code"], "work_limit", "{result}");
    assert_eq!(
        result["message"],
        format!("event {event}, rule {rule}: event exceeds work limit 4096"),
    );
}

/// `members` members, each with one rule routed by `report` that `effect`
/// runs, after `head`.
fn members(head: &str, members: usize, effect: &str) -> String {
    let mut source = format!(
        "{head}place desk kind report;\nstate n = 0;\nstate last = -1;\nevent retract report kind report;\n"
    );
    for index in 0..members {
        source.push_str(&format!("entity r{index} kind report at desk;\n"));
    }
    source.push_str(&format!(
        "for report as $r routed by report {{\n    on retract {effect};\n}};\n"
    ));
    source
}

#[test]
fn a_budget_that_runs_out_among_skipped_rules_refuses_the_same_rule() {
    // Each copy counts one step, and r5's runs: the 4,097th rule finds the
    // budget spent.
    let mut session =
        ReactiveSession::from_source(&members("", 4_200, "set last = $index")).unwrap();
    let result = outcome(&mut session, "retract", r#"{"report": "r5"}"#);
    refused_at(&result, "retract", 4_097);
    // 4,096 copies spend the budget exactly.
    let mut session =
        ReactiveSession::from_source(&members("", 4_096, "set last = $index")).unwrap();
    let result = outcome(&mut session, "retract", r#"{"report": "r5"}"#);
    assert_eq!(result["outcome"], "accepted", "{result}");
    // `$index` counts members from 1, and r5 is the sixth.
    assert_eq!(result["snapshot"]["values"]["last"], 6.0);
}

#[test]
fn a_skipped_call_counts_each_step_of_its_procedure() {
    // `bump` has two steps, so each copy counts three: the rule's own step and
    // one for each of bump's. r0's copy runs first and counts three as well.
    let head = "proc bump(member) { set n = n + 1; set last = member; };\n";
    let effect = "call bump($index)";
    // 1,365 copies count 4,095 steps.
    let mut session = ReactiveSession::from_source(&members(head, 1_365, effect)).unwrap();
    let result = outcome(&mut session, "retract", r#"{"report": "r0"}"#);
    assert_eq!(result["outcome"], "accepted", "{result}");
    assert_eq!(result["snapshot"]["values"]["n"], 1.0);
    assert_eq!(result["snapshot"]["values"]["last"], 1.0);
    // With 1,366, the last copy has one step left: its own step is counted,
    // then bump's first step finds the budget spent, inside the skipped call.
    let mut session = ReactiveSession::from_source(&members(head, 1_366, effect)).unwrap();
    let result = outcome(&mut session, "retract", r#"{"report": "r0"}"#);
    refused_at(&result, "retract", 1_366);
    // With the declaration, the copies routed elsewhere count nothing.
    let mut session =
        ReactiveSession::from_source(&members(&format!("{DECLARATION}{head}"), 4_200, effect))
            .unwrap();
    let result = outcome(&mut session, "retract", r#"{"report": "r4199"}"#);
    assert_eq!(result["outcome"], "accepted", "{result}");
    assert_eq!(result["snapshot"]["values"]["last"], 4_200.0);
}

#[test]
fn a_skipped_call_counts_the_steps_of_nested_calls() {
    // `outer` counts its two steps, and its call to `inner` counts inner's
    // two: a skipped copy counts five with its own step.
    let head = "proc inner(member) { set n = n + 1; set last = member; };\n\
                proc outer(member) { set n = n + 0; call inner(member); };\n";
    let effect = "call outer($index)";
    // 819 copies count 4,095 steps.
    let mut session = ReactiveSession::from_source(&members(head, 819, effect)).unwrap();
    let result = outcome(&mut session, "retract", r#"{"report": "r0"}"#);
    assert_eq!(result["outcome"], "accepted", "{result}");
    assert_eq!(result["snapshot"]["values"]["n"], 1.0);
    // With 820, the last copy has one step left.
    let mut session = ReactiveSession::from_source(&members(head, 820, effect)).unwrap();
    let result = outcome(&mut session, "retract", r#"{"report": "r0"}"#);
    refused_at(&result, "retract", 820);
}

#[test]
fn a_route_reads_its_parameter_by_its_declared_place() {
    // The routes read `zone`, declared second of three and last in name
    // order; a route read at another place, or from the payload's names in
    // their order, would skip the rule that runs or run the one skipped.
    for head in ["", DECLARATION] {
        let source = format!(
            "{head}state a = 0;\nstate b = 0;\n\
             event put level min 0 max 9, zone min 0 max 9, slot min 0 max 9;\n\
             on put when zone == 3 set a = level;\non put when zone == 4 set b = level;\n"
        );
        let mut session = ReactiveSession::from_source(&source).unwrap();
        let result = outcome(&mut session, "put", r#"{"level": 7, "zone": 4, "slot": 3}"#);
        assert_eq!(result["outcome"], "accepted", "{result}");
        assert_eq!(result["snapshot"]["values"]["a"], 0.0, "{head}");
        assert_eq!(result["snapshot"]["values"]["b"], 7.0, "{head}");
        let result = outcome(&mut session, "put", r#"{"level": 3, "zone": 3, "slot": 4}"#);
        assert_eq!(result["snapshot"]["values"]["a"], 3.0, "{head}");
        assert_eq!(result["snapshot"]["values"]["b"], 7.0, "{head}");
    }
}

#[test]
fn a_route_reads_past_terms_the_payload_matches_up_to_a_term_it_cannot_read() {
    // Each guard's route reaches its member term, which routes every copy but
    // r3's elsewhere: past a first term every copy's payload matches (`loud
    // == -1`), and up to, not through, a term it cannot read (`loud == n`,
    // where `n` is a state). With the declaration the 4,199 copies routed
    // elsewhere count nothing, so the event is accepted; a route read only up
    // to its first term, or dropped whole at a term it cannot read, would
    // count every copy and be refused at rule 4,097, as the program without
    // the declaration is.
    for (guard, payload) in [
        (
            "loud == -1 and report == $index",
            r#"{"report": "r3", "loud": -1}"#,
        ),
        (
            "report == $index and loud == n",
            r#"{"report": "r3", "loud": 0}"#,
        ),
    ] {
        let mut source = format!(
            "{DECLARATION}place desk kind report;\nstate n = 0;\nevent ping report kind report, loud min -1 max 1;\n"
        );
        for index in 0..4_200 {
            source.push_str(&format!("entity r{index} kind report at desk;\n"));
        }
        source.push_str(&format!(
            "for report as $r {{\n    on ping when {guard} set n = n + 1;\n}};\n"
        ));
        let mut session = ReactiveSession::from_source(&source).unwrap();
        let result = outcome(&mut session, "ping", payload);
        assert_eq!(result["outcome"], "accepted", "{guard}: {result}");
        assert_eq!(result["snapshot"]["values"]["n"], 1.0, "{guard}");
        let undeclared = source.replacen(DECLARATION, "", 1);
        let mut session = ReactiveSession::from_source(&undeclared).unwrap();
        refused_at(&outcome(&mut session, "ping", payload), "ping", 4_097);
    }
}

#[test]
fn a_windowed_program_still_skips_step_by_step() {
    // A window's collector reads each skipped effect, so nothing is counted
    // at once there; the refusal names the same rule as without a window.
    let mut session =
        ReactiveSession::from_source(&members("journal window 4;\n", 4_200, "set last = $index"))
            .unwrap();
    let result = outcome(&mut session, "retract", r#"{"report": "r5"}"#);
    refused_at(&result, "retract", 4_097);
    let mut session = ReactiveSession::from_source(&members(
        &format!("{DECLARATION}journal window 4;\n"),
        4_200,
        "set last = $index",
    ))
    .unwrap();
    let result = outcome(&mut session, "retract", r#"{"report": "r5"}"#);
    assert_eq!(result["outcome"], "accepted", "{result}");
    assert_eq!(result["snapshot"]["values"]["last"], 6.0);
}
