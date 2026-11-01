//! F268 in a program with a window: a procedure parameter reads its
//! argument's lineage with the split that lineage holds between names it read
//! itself and names it inherited through a selection qualification, as the
//! inline argument would; only its grounds are read apart. A name a
//! parameter's lineage only inherited pins nothing downstream.
use caveat_runtime::reactive::ReactiveSession;
use std::collections::BTreeSet;

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).into()).collect()
}

fn play(source: &str, events: &[(&str, &str)]) -> ReactiveSession {
    let mut session = ReactiveSession::from_source(source)
        .unwrap_or_else(|error| panic!("fixture failed: {error}"));
    for (event, payload) in events {
        let outcome =
            serde_json::to_value(session.dispatch_outcome_json(event, payload).unwrap()).unwrap();
        assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    }
    session
}

// `x` gates a sample that never runs, so it reaches every read of `s` as a
// selection qualification: inherited. `t` reads `s` and cites `x`, so its
// grounds read `x` and its lineage inherited it.
const WINDOWED: &str = r#"
claim c;
evidence tool from "the tool";
evidence x from "x";
readings s from tool window 3;
state gate = 0;
state t = 0;
state inline = 0;
state through = 0;
event first;
event skip;
event cite;
event copy;
on first reveal x supports c;
on first set gate = qualified(1, x);
on skip when gate == 0 sample s = 1;
on cite set t = history_count(s) because qualified(1, x);
proc store(v) { set through = v; };
on copy set inline = t;
on copy call store(t);
"#;

#[test]
fn f268_a_parameter_passes_on_what_its_arguments_lineage_inherited() {
    let session = play(
        WINDOWED,
        &[
            ("first", "{}"),
            ("skip", "{}"),
            ("cite", "{}"),
            ("copy", "{}"),
        ],
    );
    let shown = session.snapshot();
    let t = &shown.qualified_values["t"].provenance;
    assert_eq!(t.evidence, names(&["x"]));
    assert_eq!(t.inherited, names(&["x"]));
    assert_eq!(shown.value_grounds["t"].evidence, names(&["x"]));
    assert_eq!(shown.value_grounds["t"].inherited, names(&[]));
    // Read inline or through a parameter, `t`'s lineage passes `x` on as
    // inherited; its grounds, read apart, pass it on as read.
    for state in ["inline", "through"] {
        let lineage = &shown.qualified_values[state].provenance;
        assert_eq!(lineage.evidence, names(&["x"]), "{state}");
        assert_eq!(lineage.inherited, names(&["x"]), "{state} lineage");
        assert_eq!(
            shown.value_grounds[state].inherited,
            names(&[]),
            "{state} grounds"
        );
    }
    // A save keeps the split, and a restored session shows the same.
    let restored = ReactiveSession::restore_json(WINDOWED, &session.save_json().unwrap()).unwrap();
    assert_eq!(restored.snapshot(), shown);
}

// `r@1` reaches `t`'s lineage only through `s`'s selection qualification,
// while `t`'s grounds cite it. A reading of `s2` taken from `t` inherits it,
// so once `t` lets go of it nothing pins it, and `r@2` retires it.
const PINS: &str = r#"
claim c;
evidence meter from "the meter";
evidence tool from "the tool";
readings r from meter window 1;
readings s from tool window 3;
readings s2 from tool window 3;
state t = 0;
state y = 0;
event read v min 0 max 10;
event skip;
event cite;
event pass;
event settle;
event clear;
on read sample r = v;
on skip when latest(r) > 5 sample s = 1;
on cite set t = history_count(s) because latest(r);
on settle set y = latest(s2);
on clear set t = 0;
"#;

#[test]
fn f268_a_record_a_parameter_only_inherited_departs_as_it_would_inline() {
    for take in [
        "proc keep(v) { sample s2 = v; };\non pass call keep(t);",
        "on pass sample s2 = t;",
    ] {
        let session = play(
            &format!("{PINS}\n{take}"),
            &[
                ("read", r#"{"v": 1}"#),
                ("skip", "{}"),
                ("cite", "{}"),
                ("pass", "{}"),
                ("settle", "{}"),
                ("clear", "{}"),
                ("read", r#"{"v": 2}"#),
            ],
        );
        let shown = serde_json::to_value(session.snapshot()).unwrap();
        let held: Vec<&str> = shown["reading_streams"]["r"]["occurrences"]
            .as_array()
            .unwrap()
            .iter()
            .map(|reading| reading["id"].as_str().unwrap())
            .collect();
        assert_eq!(held, ["r@2"], "{take}");
        assert!(
            !shown["symbols"]
                .as_array()
                .unwrap()
                .iter()
                .any(|symbol| symbol["name"] == "r@1"),
            "{take}"
        );
        assert_eq!(
            shown["value_grounds"]["y"]["evidence"],
            serde_json::json!(["s2@1"]),
            "{take}"
        );
    }
}
