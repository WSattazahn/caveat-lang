//! A renewal evaluates a binding scoped to histories again only if its last
//! evaluation looked up the renewed evidence's occurrences. A scoped binding
//! names only histories, but their readings and decisions list occurrences of
//! renewable evidence. These programs renew such evidence, doubt its
//! occurrences and take the doubts back, withdraw and retire them, and check
//! after every event that the session shows what a restored session, which
//! evaluates every binding in full, shows. See
//! spec/caveat-incremental-evaluation-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

fn shown(session: &ReactiveSession) -> Value {
    let snapshot = serde_json::to_value(session.snapshot()).unwrap();
    serde_json::json!({
        "bindings": snapshot["bindings"],
        "lineage": snapshot["binding_qualifications"],
        "explanations": snapshot["binding_explanations"],
    })
}

fn same_as_restored(source: &str, session: &ReactiveSession, event: &str) -> Value {
    let live = shown(session);
    let restored = ReactiveSession::restore_json(source, &session.save_json().unwrap())
        .unwrap_or_else(|error| panic!("{event}: {error}"));
    assert_eq!(live, shown(&restored), "after {event}");
    live
}

/// Dispatches each event, checking after each that the session shows what a
/// full evaluation shows. Returns what it shows after each.
fn run(source: &str, events: &[&str]) -> Vec<Value> {
    let mut session = ReactiveSession::from_source(source).unwrap();
    let mut after = Vec::new();
    for event in events {
        let outcome = session
            .dispatch_outcome_json(event, "{}")
            .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
        let outcome: Value = serde_json::to_value(outcome).unwrap();
        assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
        after.push(same_as_restored(source, &session, event));
    }
    after
}

fn caveats<'a>(shown: &'a Value, target: &str, property: &str) -> Vec<&'a str> {
    shown["lineage"][target][property]["caveats"]
        .as_array()
        .map(|list| list.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// Readings of renewable evidence, and a decision that rests on other
/// renewable evidence, across renewals, doubts about one occurrence or all of
/// them, and a withdrawal of the occurrence the decision rests on.
const RENEWED: &str = r#"
claim ok;
caveat k consequence material;
caveat secondhand consequence material;
evidence ci from "the CI run";
renewable ci limit 8;
secondhand qualifies ci;
evidence tree from "the release tree";
renewable tree limit 8;
secondhand qualifies tree;
evidence recheck from "a recheck";
recheck supports ok;
readings s from ci limit 8;
decisions d limit 4;
event run;
event rerun;
event report;
event build;
event rebuild;
event decide;
event doubt;
event doubt_first;
event undoubt;
event pull;
on run reveal ci supports ok;
on rerun renew ci;
on report sample s = 1 supports ok;
on build reveal tree supports ok;
on rebuild renew tree;
on decide commit d because enough using qualified(1, tree);
on doubt qualify ci with k;
on doubt qualify tree with k;
on doubt_first qualify occurrence(tree, 1) with k;
on undoubt unqualify tree with k;
on pull withdraw occurrence(tree, 1) because recheck;
bind x.n = history_count(s);
bind x.s = latest(s) when has_sample(s);
bind x.v = latest(d) when committed(d);
bind y.seen = 1 when observed(ci);
"#;

#[test]
fn readings_and_a_decision_that_list_renewed_evidence() {
    let after = run(
        RENEWED,
        &[
            "run",
            "report",
            "build",
            "decide",
            "rerun",
            "rebuild",
            "run",
            "build",
            "report",
            "doubt",
            "doubt_first",
            "undoubt",
            "rerun",
            "rebuild",
            "pull",
            "report",
        ],
    );
    // The decision rests on the first occurrence of the tree. A doubt about
    // the current one passes it by; one about the first reaches it, and
    // taking the doubt back from every occurrence clears it.
    assert!(!caveats(&after[9], "x", "v").contains(&"k"));
    assert!(caveats(&after[10], "x", "v").contains(&"k"));
    assert!(!caveats(&after[11], "x", "v").contains(&"k"));
    // An occurrence carries the declared caveat.
    assert!(caveats(&after[3], "x", "v").contains(&"secondhand"));
    // Withdrawing the first occurrence reaches the decision; the readings
    // list occurrences of the other evidence.
    assert!(!caveats(&after[13], "x", "v").contains(&"withdrawn"));
    assert!(caveats(&after[14], "x", "v").contains(&"withdrawn"));
    assert!(!caveats(&after[14], "x", "s").contains(&"withdrawn"));
}

/// A window of two occurrences: each renewal past the second retires the
/// oldest, which the first decision rests on.
const WINDOWED: &str = r#"
claim ok;
evidence ci from "the CI run";
renewable ci window 2;
readings s from ci limit 8;
decisions d limit 4;
event run;
event rerun;
event report;
event decide;
event reconsider;
on run reveal ci supports ok;
on rerun renew ci;
on report sample s = 1 supports ok;
on decide commit d because enough using qualified(1, ci);
on reconsider reopen d because ci;
bind x.n = history_count(s);
bind x.v = latest(d) when committed(d);
bind x.r = "open";
bind x.r = "reopened" when reopened(d);
"#;

#[test]
fn renewals_that_retire_what_a_decision_rests_on() {
    run(
        WINDOWED,
        &[
            "run",
            "report",
            "decide",
            "rerun",
            "run",
            "report",
            "rerun",
            "rerun",
            "run",
            "reconsider",
            "decide",
            "rerun",
            "report",
        ],
    );
}
