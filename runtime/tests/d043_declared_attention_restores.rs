//! D043: restore refused saves of a program whose declarations examine or
//! defer a caveat, the save written before any event among them. A
//! declaration such as `examine stale cost 1;`, `examine stale;` or `defer
//! stale;` leaves the caveat `examined`, `examining` or `deferred` as the
//! program loads, and a save holds that attention until an event examines the
//! caveat, but restore checked it as if an event had left it: "caveat stale is
//! examining, which no reactive event leaves", "caveat stale is examined, but
//! no rule or procedure of this program examines it", or, when a rule could
//! examine it for 3 and the budget had spent only the declaration's 1,
//! "examined caveats cost at least 3 attention to examine, more than the 1 the
//! budget spent". Restore now accepts the attention the declarations leave,
//! and leaves a caveat they leave `examined` out of the least cost the
//! budget's `spent` must cover. See spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// `go` reveals the sensor and counts while `stale` is examined. With a
/// budget, `check` examines `stale` and `probe` examines `deep`, each for 3;
/// a program without one cannot examine in a rule. No rule examines `aside`.
fn program(budget: bool, declarations: &str) -> String {
    let (head, examinations) = if budget {
        (
            "budget 12;\n",
            "on check examine stale cost 3;\non probe examine deep cost 3;\n",
        )
    } else {
        ("", "")
    };
    format!(
        "{head}\
         claim safe;\n\
         evidence sensor from \"sensor\";\n\
         caveat stale consequence material;\n\
         caveat deep consequence low;\n\
         caveat aside consequence low;\n\
         {declarations}\
         state seen = 0;\n\
         event go;\n\
         event check;\n\
         event probe;\n\
         on go reveal sensor supports safe;\n\
         on go when examined(stale) set seen = seen + 1;\n\
         {examinations}"
    )
}

/// Each way the declarations leave a caveat's attention: whether the program
/// has a budget, the declarations, the attention the save holds, and what
/// `seen` counts after the events `played` sends. The last declaration of a
/// caveat decides its attention.
const DECLARED: [(bool, &str, &str, f64); 7] = [
    (
        true,
        "examine stale cost 1;\n",
        r#"{"stale":"examined"}"#,
        2.0,
    ),
    (true, "examine stale;\n", r#"{"stale":"examining"}"#, 1.0),
    (false, "examine stale;\n", r#"{"stale":"examining"}"#, 0.0),
    (false, "defer stale;\n", r#"{"stale":"deferred"}"#, 0.0),
    (
        true,
        "examine stale cost 1;\nexamine stale cost 2;\nexamine deep cost 1;\n",
        r#"{"deep":"examined","stale":"examined"}"#,
        2.0,
    ),
    (
        true,
        "examine aside cost 2;\n",
        r#"{"aside":"examined"}"#,
        1.0,
    ),
    (
        true,
        "examine stale cost 1;\ndefer stale;\n",
        r#"{"stale":"deferred"}"#,
        1.0,
    ),
];

/// `go`, then, with a budget, `check` examining `stale` and `go` again and
/// `probe` examining `deep`.
fn played(budget: bool) -> &'static [&'static str] {
    if budget {
        &["go", "check", "go", "probe"]
    } else {
        &["go", "go"]
    }
}

fn send(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    outcome
}

fn saved(session: &ReactiveSession) -> Value {
    serde_json::from_str(&session.save_json().unwrap()).unwrap()
}

/// The session restored from its own save, checked to be the same session.
fn round_trip(source: &str, session: &ReactiveSession, label: &str) -> ReactiveSession {
    let save = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(source, &save)
        .unwrap_or_else(|error| panic!("{label}: restore refused the runtime's own save: {error}"));
    assert_eq!(restored.save_json().unwrap(), save, "{label}");
    assert_eq!(restored.snapshot(), session.snapshot(), "{label}");
    restored
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted {}", save["graph"]["attention"]),
    }
}

#[test]
fn a_save_keeps_the_attention_a_declaration_leaves() {
    for (budget, declarations, attention, seen) in DECLARED {
        let source = program(budget, declarations);
        let label = declarations.trim_end();
        let mut live = ReactiveSession::from_source(&source).unwrap();
        // The save written before any event holds the declared attention.
        assert_eq!(
            saved(&live)["graph"]["attention"],
            serde_json::from_str::<Value>(attention).unwrap(),
            "{label}"
        );
        let mut restored = round_trip(&source, &live, label);
        // Both play on alike, and every save along the way restores.
        for event in played(budget) {
            let answer = send(&mut live, event);
            assert_eq!(send(&mut restored, event), answer, "{label}: {event}");
            restored = round_trip(&source, &live, &format!("{label}: {event}"));
        }
        assert_eq!(live.snapshot().values["seen"], seen, "{label}");
    }
}

#[test]
fn a_caveat_the_declarations_leave_examined_counts_once_an_event_examines_it() {
    // The declaration spent 1 on `stale`, and `check` examines it for 3,
    // which the budget has not spent before `check` runs. After it, the
    // examination record counts `stale` (D046).
    let source = program(true, "examine stale cost 1;\n");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    assert_eq!(saved(&live)["resources"]["spent"], 1);
    round_trip(&source, &live, "before check");
    send(&mut live, "check");
    assert_eq!(saved(&live)["resources"]["spent"], 4);
    round_trip(&source, &live, "after check");
}

#[test]
fn attention_no_declaration_or_event_leaves_is_still_refused() {
    // A declaration leaves `stale` deferred or examining; no event leaves the
    // other.
    for (declarations, forged) in [
        ("defer stale;\n", "examining"),
        ("examine stale;\n", "deferred"),
    ] {
        let source = program(false, declarations);
        let mut save = saved(&ReactiveSession::from_source(&source).unwrap());
        save["graph"]["attention"]["stale"] = forged.into();
        refused(
            &source,
            &save,
            &format!("caveat stale is {forged}, which no reactive event leaves"),
        );
    }
    // A caveat a declaration leaves examining is examined by no event of a
    // program without a budget.
    let source = program(false, "examine stale;\n");
    let mut save = saved(&ReactiveSession::from_source(&source).unwrap());
    save["graph"]["attention"]["stale"] = "examined".into();
    save["examination_qualifications"]["stale"] = json!({});
    refused(
        &source,
        &save,
        "caveat stale is examined, but this program has no attention budget",
    );
    // A caveat the declarations leave examining or deferred is examined only
    // by an event, which the budget pays for and which leaves a record:
    // `check` examines `stale` for 3. A declaration that examined `stale`
    // before a later one deferred it pays for none of that, and since D046
    // what it spent is counted besides.
    let short = "examined caveats cost at least 3 attention to examine, more than the 0 the \
                 budget spent";
    for (declarations, spent, expected) in [
        ("examine stale;\n", 0, short),
        ("defer stale;\n", 0, short),
        (
            "examine stale cost 1;\ndefer stale;\n",
            1,
            "examined caveats cost at least 3 attention to examine and the declarations spent \
             1, 4 in all, more than the 1 the budget spent",
        ),
    ] {
        let source = program(true, declarations);
        let mut live = ReactiveSession::from_source(&source).unwrap();
        let mut save = saved(&live);
        assert_eq!(save["resources"]["spent"], spent, "{declarations}");
        save["graph"]["attention"]["stale"] = "examined".into();
        save["examination_qualifications"]["stale"] = json!({});
        refused(&source, &save, expected);
        send(&mut live, "check");
        let mut save = saved(&live);
        save["examination_qualifications"]
            .as_object_mut()
            .unwrap()
            .remove("stale");
        refused(
            &source,
            &save,
            "examined caveat stale has no examination record",
        );
    }
    // `deep`, which the declarations leave unexamined, is checked as before:
    // `probe` examines it for 3, and the budget spent only the declaration's
    // 1, which since D046 is counted too. The declared `stale` adds nothing
    // to the least cost, though `check` could examine it for 3.
    let source = program(true, "examine stale cost 1;\n");
    let mut save = saved(&ReactiveSession::from_source(&source).unwrap());
    save["graph"]["attention"]["deep"] = "examined".into();
    save["examination_qualifications"]["deep"] = json!({});
    refused(
        &source,
        &save,
        "examined caveats cost at least 3 attention to examine and the declarations spent 1, 4 \
         in all, more than the 1 the budget spent",
    );
    // A caveat no rule or procedure examines is refused beside a declared
    // one.
    let source = program(true, "examine stale cost 1;\n").replace(
        "on probe examine deep cost 3;\n",
        "on probe reveal sensor supports safe;\n",
    );
    let mut save = saved(&ReactiveSession::from_source(&source).unwrap());
    save["graph"]["attention"]["deep"] = "examined".into();
    save["examination_qualifications"]["deep"] = json!({});
    refused(
        &source,
        &save,
        "caveat deep is examined, but no rule or procedure of this program examines it",
    );
}
