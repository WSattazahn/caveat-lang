//! F268: a procedure parameter grounds a value as its argument would inline.
//! A call binds each argument with its full lineage, for the call's guard and
//! frozen arguments, and with its grounds, for `set`, `commit using` and
//! citations. See spec/caveat-explanations-0.2.md and
//! spec/caveat-reactive-0.7.md.
use caveat_runtime::reactive::{Provenance, ReactiveSession, ReactiveSnapshot};
use std::collections::BTreeSet;

// `x` is revealed first and only gates the reveal of `w`, so `x` is lineage
// of every read of `w` and grounds of none.
const GATED: &str = r#"
claim c;
evidence x from "x";
evidence w from "w";
evidence y from "y";
state gate = 0;
state slot = 0;
state flag = 0;
decisions d limit 3;
event first;
event see;
event retract;
event retract_w;
event check;
on first reveal x supports c;
on first set gate = qualified(1, x);
on see when gate == 1 reveal w supports c;
on retract reveal y supports c;
on retract withdraw x because y;
on retract_w reveal y supports c;
on retract_w withdraw w because y;
on check when rests_on_withdrawn(d) set flag = 1;
"#;

fn played(rules: &str, events: &[&str]) -> ReactiveSnapshot {
    let mut session = ReactiveSession::from_source(&format!("{GATED}\n{rules}"))
        .unwrap_or_else(|error| panic!("fixture failed: {error}"));
    for event in events {
        session
            .dispatch_json(event, "{}")
            .unwrap_or_else(|error| panic!("{event}: {error}"));
    }
    session.snapshot()
}

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).into()).collect()
}

fn trace(actual: &Provenance, evidence: &[&str], caveats: &[&str]) {
    assert_eq!(actual.evidence, names(evidence), "evidence");
    assert_eq!(actual.caveats, names(caveats), "caveats");
}

const INLINE_SET: &str = "on see set slot = qualified(7, w);";
const PARAMETER_SET: &str = "proc store(v) { set slot = v; };\n\
                             on see call store(qualified(7, w));";
const INLINE_COMMIT: &str = "on see commit d because enough using qualified(7, w);";
const PARAMETER_COMMIT: &str = "proc decide(v) { commit d because enough using v; };\n\
                                on see call decide(qualified(7, w));";

#[test]
fn f268_a_parameter_grounds_a_set_as_the_inline_argument_would() {
    let inline = played(INLINE_SET, &["first", "see"]);
    let through = played(PARAMETER_SET, &["first", "see"]);
    trace(&inline.value_grounds["slot"], &["w"], &[]);
    trace(&through.value_grounds["slot"], &["w"], &[]);
    // Lineage is unchanged: the guard on w's reveal still reaches the value,
    // and the call still freezes its argument's lineage.
    trace(&inline.qualified_values["slot"].provenance, &["w", "x"], &[]);
    trace(&through.qualified_values["slot"].provenance, &["w", "x"], &[]);
}

#[test]
fn f268_commit_using_a_parameter_freezes_the_inline_grounds() {
    let inline = played(INLINE_COMMIT, &["first", "see"]);
    let through = played(PARAMETER_COMMIT, &["first", "see"]);
    trace(&inline.commitment_grounds["d@1"], &["w"], &[]);
    trace(&through.commitment_grounds["d@1"], &["w"], &[]);
    trace(&inline.commitment_bases["d@1"].provenance, &["w", "x"], &[]);
    trace(&through.commitment_bases["d@1"].provenance, &["w", "x"], &[]);
}

#[test]
fn f268_withdrawing_evidence_a_parameter_never_grounded_does_not_reach_the_decision() {
    for rules in [INLINE_COMMIT, PARAMETER_COMMIT] {
        let shown = played(rules, &["first", "see", "retract", "check"]);
        assert_eq!(shown.values["flag"], 0.0, "{rules}");
        // The decision does rest on w.
        let shown = played(rules, &["first", "see", "retract_w", "check"]);
        assert_eq!(shown.values["flag"], 1.0, "{rules}");
    }
}

#[test]
fn f268_a_nested_argument_reading_a_parameter_keeps_its_grounds() {
    let nested = played(
        "proc store(v) { set slot = v; };\n\
         proc outer(v) { call store(v + 0); };\n\
         on see call outer(qualified(7, w));",
        &["first", "see"],
    );
    trace(&nested.value_grounds["slot"], &["w"], &[]);
    trace(&nested.qualified_values["slot"].provenance, &["w", "x"], &[]);

    // A state read in the nested argument still supplies its own grounds.
    let mixed = played(
        "proc store(v) { set slot = v; };\n\
         proc outer(v) { call store(v + gate); };\n\
         on see call outer(qualified(7, w));",
        &["first", "see"],
    );
    trace(&mixed.value_grounds["slot"], &["w", "x"], &[]);
}

#[test]
fn f268_a_renewed_occurrence_does_not_carry_its_first_life_into_grounds() {
    // Glowcap C4: a regrowth guard read a state grounded on w's first life.
    let source = "claim c; evidence w from \"witness\"; renewable w limit 4;\n\
                  state used = 0; state slot = 0;\n\
                  event see; event regrow;\n\
                  on see reveal w supports c;\n\
                  on see set used = qualified(1, w);\n\
                  on regrow when used == 1 renew w;\n\
                  on regrow set used = 0;\n\
                  proc store(v) { set slot = v; };\n\
                  on see call store(qualified(7, w));";
    let mut session = ReactiveSession::from_source(source).unwrap();
    for event in ["see", "regrow", "see"] {
        session.dispatch_json(event, "{}").unwrap();
    }
    let shown = session.snapshot();
    trace(&shown.value_grounds["slot"], &["w@2"], &[]);
    trace(&shown.qualified_values["slot"].provenance, &["w", "w@2"], &[]);
}

#[test]
fn f268_a_citation_through_a_parameter_reads_its_grounds() {
    let shown = played(
        "proc store(v) { set slot = v + gate because v; };\n\
         on see call store(qualified(7, w));",
        &["first", "see"],
    );
    trace(&shown.value_grounds["slot"], &["w"], &[]);
}

#[test]
fn f268_an_rc11_save_with_wide_parameter_grounds_restores_and_continues() {
    // Written by the published caveat-lang 0.1.0-rc.11 after `first` and
    // `see`; its grounds for slot and d@1 are the wide [w, x].
    let source = include_str!("fixtures/f268-rc11-wide-grounds.cav");
    let saved = include_str!("fixtures/f268-rc11-wide-grounds.save.json");
    let mut session = ReactiveSession::restore_json(source, saved)
        .unwrap_or_else(|error| panic!("rc.11 save refused: {error}"));
    let restored = session.snapshot();
    trace(&restored.value_grounds["slot"], &["w", "x"], &[]);
    trace(&restored.commitment_grounds["d@1"], &["w", "x"], &[]);

    for event in ["retract", "again", "check"] {
        session
            .dispatch_json(event, "{}")
            .unwrap_or_else(|error| panic!("{event}: {error}"));
    }
    let shown = session.snapshot();
    // New work is grounded narrowly; frozen grounds keep what rc.11 froze.
    trace(&shown.value_grounds["slot"], &["w"], &[]);
    trace(&shown.commitment_grounds["d@2"], &["w"], &[]);
    trace(&shown.commitment_grounds["d@1"], &["w", "x"], &[]);
    assert_eq!(shown.values["flag"], 0.0);

    let again = ReactiveSession::restore_json(source, &session.save_json().unwrap()).unwrap();
    assert_eq!(again.snapshot(), shown);
}
