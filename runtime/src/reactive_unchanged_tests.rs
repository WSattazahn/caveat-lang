//! An event that writes back what a shared record already holds leaves the
//! record shared: the session's copy keeps the pointer the session before
//! the event holds, so nothing copies the record and the change check and
//! the view see it unchanged.
use super::*;

fn session(source: &str, events: &[&str]) -> ReactiveSession {
    let mut session = ReactiveSession::from_source(source).unwrap();
    for event in events {
        session.apply(event, &BTreeMap::new()).unwrap();
    }
    session
}

/// The session after `event`, with the one before it.
fn around(session: &mut ReactiveSession, event: &str) -> ReactiveSession {
    let before = session.clone();
    session.apply(event, &BTreeMap::new()).unwrap();
    before
}

/// Rules whose guard is false: each keeps what its guard read as the record
/// of the effect it skipped.
const SKIPPED: &str = r#"
claim c;
evidence tool from "the tool";
evidence x from "x";
readings tests from tool limit 8;
decisions gate limit 8;
event report;
event check;
on report sample tests = 1 supports c;
on check when latest(tests) > 5 reveal x;
on check when latest(tests) > 5 sample tests = 2 supports c;
on check when latest(tests) > 5 commit gate because enough using latest(tests);
on check when latest(tests) > 5 withdraw every tests because x;
"#;

#[test]
fn a_skipped_effect_whose_record_holds_its_guard_writes_nothing() {
    let mut session = session(SKIPPED, &["report", "check"]);
    let before = around(&mut session, "check");
    assert!(Arc::ptr_eq(
        &before.predicate_qualifications,
        &session.predicate_qualifications
    ));
    assert!(Arc::ptr_eq(
        &before.reading_streams,
        &session.reading_streams
    ));
    assert!(Arc::ptr_eq(
        &before.decision_series,
        &session.decision_series
    ));
    // The first skip did write each record.
    assert!(session.predicate_qualifications.contains_key("observed"));
    assert!(session.predicate_qualifications.contains_key(WITHDRAWN));
    assert!(!session.reading_streams["tests"]
        .selection_qualifications
        .is_empty());
    assert!(!session.decision_series["gate"]
        .selection_qualifications
        .is_empty());
}

#[test]
fn a_second_reveal_of_the_same_evidence_leaves_its_record_shared() {
    let mut session = session(
        r#"
claim a;
claim b;
evidence e from "e";
event first;
event second;
on first reveal e supports a;
on second reveal e opposes b;
"#,
        &["first"],
    );
    let before = around(&mut session, "second");
    assert!(Arc::ptr_eq(
        &before.observation_qualifications,
        &session.observation_qualifications
    ));
    assert!(!Arc::ptr_eq(&before.graph, &session.graph));
}

#[test]
fn a_release_on_the_basis_of_its_hold_leaves_the_reopening_record_shared() {
    let mut session = session(
        r#"
claim safe;
evidence review from "a review";
evidence brief from "a brief";
decisions no_send limit 4;
event rule_out;
event hold_it;
event release_it;
on rule_out when not observed(review) reveal review supports safe;
on rule_out commit no_send because enough citing qualified(1, review);
on hold_it when not observed(brief) reveal brief;
on hold_it hold no_send because brief;
on release_it release no_send because brief;
"#,
        &["rule_out", "hold_it"],
    );
    let before = around(&mut session, "release_it");
    assert!(!before.reopening_qualifications.is_empty());
    assert!(Arc::ptr_eq(
        &before.reopening_qualifications,
        &session.reopening_qualifications
    ));
    assert!(!Arc::ptr_eq(
        &before.decision_series,
        &session.decision_series
    ));
}

#[test]
fn qualifying_again_with_the_same_caveat_writes_nothing_it_already_holds() {
    let mut session = session(
        r#"
claim releasable;
caveat flaky consequence material;
evidence tool from "the tool";
evidence x from "x";
readings tests from tool limit 8;
state s = 0;
event report;
event note;
event flag;
on report sample tests = 1 supports releasable;
on report set s = latest(tests);
on note when latest(tests) == 1 reveal x;
on flag qualify latest(tests) with flaky;
"#,
        &["report", "note", "flag"],
    );
    assert!(session.states.cells[0]
        .value
        .provenance
        .caveats
        .contains("flaky"));
    assert!(session.observation_qualifications["x"]
        .caveats
        .contains("flaky"));
    let before = around(&mut session, "flag");
    assert!(Arc::ptr_eq(&before.states.cells, &session.states.cells));
    assert!(Arc::ptr_eq(
        &before.observation_qualifications,
        &session.observation_qualifications
    ));
    assert!(Arc::ptr_eq(
        &before.examination_qualifications,
        &session.examination_qualifications
    ));
    assert!(Arc::ptr_eq(
        &before.predicate_qualifications,
        &session.predicate_qualifications
    ));
    assert!(Arc::ptr_eq(&before.graph, &session.graph));
}

#[test]
fn a_qualification_writes_only_the_maps_whose_records_list_the_evidence() {
    let mut session = session(
        r#"
claim releasable;
caveat flaky consequence material;
caveat slow consequence material;
evidence tool from "the tool";
evidence x from "x";
readings tests from tool limit 8;
budget 10;
event report;
event note;
event look;
event flag;
on report sample tests = 1 supports releasable;
on note when latest(tests) == 1 reveal x;
on look examine slow cost 1;
on flag qualify latest(tests) with flaky;
"#,
        &["report", "note", "look"],
    );
    let before = around(&mut session, "flag");
    assert!(!Arc::ptr_eq(
        &before.observation_qualifications,
        &session.observation_qualifications
    ));
    assert!(Arc::ptr_eq(
        &before.examination_qualifications,
        &session.examination_qualifications
    ));
}

#[test]
fn examining_a_caveat_again_on_the_same_basis_leaves_the_graph_shared() {
    let mut session = session(
        r#"
caveat a consequence material;
budget 10;
event look;
on look examine a cost 1;
"#,
        &["look"],
    );
    let before = around(&mut session, "look");
    assert!(Arc::ptr_eq(&before.graph, &session.graph));
    assert!(Arc::ptr_eq(
        &before.examination_qualifications,
        &session.examination_qualifications
    ));
    assert_eq!(
        session.resources.as_ref().map(|resources| resources.spent),
        Some(2)
    );
}

#[test]
fn an_event_in_a_windowed_program_that_departs_nothing_copies_no_departure_set() {
    let mut session = session(
        r#"
claim seen;
evidence glimpse from "a glimpse";
readings sighting from glimpse window 3;
event look;
event idle;
on look sample sighting = 1 supports seen;
"#,
        &["look"],
    );
    let before = around(&mut session, "idle");
    assert!(Arc::ptr_eq(
        &before.departure_candidates,
        &session.departure_candidates
    ));
    assert!(Arc::ptr_eq(
        &before.withdrawal_reasons,
        &session.withdrawal_reasons
    ));
}
