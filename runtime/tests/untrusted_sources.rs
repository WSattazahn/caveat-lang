//! `withdraw every NAME because E`, `retaining C because once E`, and one
//! action grounded on two sources by a nested routed block. See
//! spec/caveat-untrusted-sources-0.1.md and the prompt injection experiment.
use caveat_runtime::reactive::ReactiveSession;
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

fn outcome(session: &mut ReactiveSession, event: &str) -> Value {
    serde_json::to_value(
        session
            .dispatch_outcome_json(event, "{}")
            .expect("not fatal"),
    )
    .unwrap()
}

fn refusal(session: &mut ReactiveSession, event: &str) -> String {
    let outcome = outcome(session, event);
    assert_eq!(outcome["outcome"], "rejected", "{outcome}");
    format!(
        "{}/{}",
        outcome["origin"].as_str().unwrap(),
        outcome["code"].as_str().unwrap()
    )
}

fn withdrawn(session: &ReactiveSession) -> Vec<String> {
    snapshot(session)["withdrawals"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|record| record["evidence"].as_str().unwrap().to_string())
        .collect()
}

/// A mail source with two streams read from its text, and a renewable note.
const SOURCE: &str = r#"
claim asked;
evidence mail from "the mail as read";
evidence bad from "a finding that the mail is malicious";
evidence note from "a note the mail left";
renewable note limit 8;
readings asks from mail limit 8;
readings addr from mail limit 8;
state copied = 0;
event read;
event ask;
event address;
event jot;
event keep;
event drop_asks;
event drop_mail;
event drop_note;
event drop_early;
event find;
on read reveal mail;
on ask sample asks = 1 supports asked;
on address sample addr = 7;
on jot when observed(note) renew note;
on jot reveal note;
on keep set copied = latest(asks);
on find reveal bad;
on drop_asks withdraw every asks because bad;
on drop_mail withdraw every mail because bad;
on drop_note withdraw every note because bad;
on drop_early withdraw every asks because note;
"#;

#[test]
fn every_reading_of_a_stream_is_withdrawn_oldest_first() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    run(
        &mut session,
        &[
            ("read", "{}"),
            ("ask", "{}"),
            ("ask", "{}"),
            ("ask", "{}"),
            ("keep", "{}"),
            ("find", "{}"),
        ],
    );
    let accepted =
        serde_json::to_value(session.dispatch_outcome_json("drop_asks", "{}").unwrap()).unwrap();
    assert_eq!(accepted["outcome"], "accepted", "{accepted}");
    assert_eq!(withdrawn(&session), ["asks@1", "asks@2", "asks@3"]);
    // One report per occurrence, as `withdraw` reports one.
    let shown = snapshot(&session);
    assert_eq!(
        shown["effects"],
        json!([
            {"kind": "withdraw", "evidence": "asks@1", "because": "bad"},
            {"kind": "withdraw", "evidence": "asks@2", "because": "bad"},
            {"kind": "withdraw", "evidence": "asks@3", "because": "bad"},
        ])
    );
    // A current value read from a reading now carries `withdrawn`.
    assert!(shown["qualified_values"]["copied"]["provenance"]["caveats"]
        .as_array()
        .unwrap()
        .contains(&json!("withdrawn")));
    // Withdrawn already: nothing more is recorded or reported.
    run(&mut session, &[("drop_asks", "{}")]);
    assert_eq!(withdrawn(&session).len(), 3);
    assert_eq!(snapshot(&session)["effects"], json!([]));
    // A later reading is a new observation.
    run(&mut session, &[("ask", "{}")]);
    assert_eq!(withdrawn(&session).len(), 3);
}

#[test]
fn evidence_takes_its_occurrences_then_the_readings_of_its_streams() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    run(
        &mut session,
        &[
            ("read", "{}"),
            ("ask", "{}"),
            ("address", "{}"),
            ("ask", "{}"),
            ("find", "{}"),
            ("drop_mail", "{}"),
        ],
    );
    // The text, then `addr` before `asks` (name order), each oldest first.
    assert_eq!(withdrawn(&session), ["mail", "addr@1", "asks@1", "asks@2"]);
}

#[test]
fn renewable_evidence_lists_observed_occurrences_first_to_current() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    run(
        &mut session,
        &[
            ("jot", "{}"),
            ("jot", "{}"),
            ("jot", "{}"),
            ("find", "{}"),
            ("drop_note", "{}"),
        ],
    );
    assert_eq!(withdrawn(&session), ["note", "note@2", "note@3"]);

    // A fresh, unobserved occurrence is not listed.
    let fresh = format!("{SOURCE}\nevent regrow;\non regrow renew note;\n");
    let mut session = ReactiveSession::from_source(&fresh).unwrap();
    run(
        &mut session,
        &[
            ("jot", "{}"),
            ("regrow", "{}"),
            ("find", "{}"),
            ("drop_note", "{}"),
        ],
    );
    assert_eq!(withdrawn(&session), ["note"]);
}

#[test]
fn the_reason_must_be_observed_even_when_nothing_is_listed() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    // No reading, unobserved reason.
    assert_eq!(
        refusal(&mut session, "drop_early"),
        "evaluation/unobserved_evidence"
    );
    // No reading, observed reason: accepted, and nothing recorded.
    run(&mut session, &[("find", "{}")]);
    let accepted = outcome(&mut session, "drop_asks");
    assert_eq!(accepted["outcome"], "accepted", "{accepted}");
    assert!(withdrawn(&session).is_empty());
    assert_eq!(snapshot(&session)["effects"], json!([]));
}

#[test]
fn a_refused_event_withdraws_nothing() {
    let source = format!("{SOURCE}\non drop_asks reject \"Not now.\";\n");
    let mut session = ReactiveSession::from_source(&source).unwrap();
    run(
        &mut session,
        &[("read", "{}"), ("ask", "{}"), ("find", "{}")],
    );
    assert_eq!(refusal(&mut session, "drop_asks"), "policy/reject");
    assert!(withdrawn(&session).is_empty());
}

#[test]
fn withdrawals_by_every_save_and_restore() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    run(
        &mut session,
        &[
            ("read", "{}"),
            ("ask", "{}"),
            ("address", "{}"),
            ("jot", "{}"),
            ("find", "{}"),
            ("drop_mail", "{}"),
            ("drop_note", "{}"),
        ],
    );
    let saved = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(SOURCE, &saved).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&session));
    assert_eq!(restored.save_json().unwrap(), saved);
    // A withdrawal no rule of its event could make is still refused.
    let mut save: Value = serde_json::from_str(&saved).unwrap();
    for record in save["withdrawals"].as_array_mut().unwrap() {
        if record["evidence"] == "asks@1" {
            record["event"] = json!("drop_note");
        }
    }
    let error = ReactiveSession::restore_json(SOURCE, &save.to_string())
        .expect_err("a withdrawal its event cannot make");
    assert!(error.contains("no withdraw of that event"), "{error}");
}

#[test]
fn a_window_lists_held_readings_only() {
    let source = r#"
evidence mail from "the mail";
evidence bad from "a finding";
readings asks from mail window 2;
event read;
event ask;
event drop;
on read reveal mail;
on ask sample asks = 1;
on drop reveal bad;
on drop withdraw every asks because bad;
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(
        &mut session,
        &[
            ("read", "{}"),
            ("ask", "{}"),
            ("ask", "{}"),
            ("ask", "{}"),
            ("drop", "{}"),
        ],
    );
    assert_eq!(withdrawn(&session), ["asks@2", "asks@3"]);
}

#[test]
fn a_procedure_withdraws_every_reading_of_its_evidence_parameter() {
    let source = r#"
evidence mail from "the mail";
evidence bad from "a finding";
readings asks from mail limit 4;
event read;
event ask;
event drop;
proc drop_all(e evidence) {
    reveal bad;
    withdraw every e because bad;
};
on read reveal mail;
on ask sample asks = 1;
on drop call drop_all(mail);
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(
        &mut session,
        &[("read", "{}"), ("ask", "{}"), ("drop", "{}")],
    );
    assert_eq!(withdrawn(&session), ["mail", "asks@1"]);
}

#[test]
fn withdraw_every_load_rules_and_the_name_every() {
    for (line, expected) in [
        (
            "on find withdraw every nothing because bad;",
            "name a reading stream or evidence",
        ),
        (
            "on find withdraw every copied because bad;",
            "name a reading stream or evidence",
        ),
        ("on find withdraw every asks addr because bad;", ""),
        ("on find withdraw every latest(asks) because bad;", ""),
        ("on find withdraw every asks because asks;", ""),
    ] {
        let error = ReactiveSession::from_source(&format!("{SOURCE}\n{line}"))
            .err()
            .unwrap_or_else(|| panic!("{line} should not load"));
        assert!(error.contains(expected), "{line}: {error}");
    }
    // Evidence named `every` keeps today's meaning.
    let source = r#"
evidence every from "a report";
evidence bad from "a finding";
event go;
on go reveal every;
on go reveal bad;
on go withdraw every because bad;
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(&mut session, &[("go", "{}")]);
    assert_eq!(withdrawn(&session), ["every"]);
}

/// Two irreversible sends resting on mail text, each confirmed by the user.
const CONFIRM: &str = r#"
claim asked;
caveat untrusted consequence high;
evidence mail from "the mail";
untrusted qualifies mail;
evidence user_ok from "the user confirming one pending send";
renewable user_ok limit 8;
decisions send_a limit 1 irreversible;
decisions send_b limit 1 irreversible;
decisions send_c limit 1 irreversible;
event read;
event confirm;
event act_a;
event act_b;
event act_c;
on read reveal mail supports asked;
on confirm when observed(user_ok) renew user_ok;
on confirm reveal user_ok;
on act_a commit send_a because enough using qualified(1, mail) retaining untrusted because once user_ok;
on act_b commit send_b because enough using qualified(1, mail) retaining untrusted because once user_ok;
on act_c commit send_c because enough using qualified(1, mail) retaining untrusted because user_ok;
"#;

#[test]
fn one_confirmation_backs_one_send() {
    let mut session = ReactiveSession::from_source(CONFIRM).unwrap();
    run(
        &mut session,
        &[("read", "{}"), ("confirm", "{}"), ("act_a", "{}")],
    );
    let before = session.save_json().unwrap();
    assert_eq!(refusal(&mut session, "act_b"), "evaluation/reason_spent");
    assert_eq!(
        session.save_json().unwrap(),
        before,
        "a refused commit records nothing"
    );
    run(&mut session, &[("confirm", "{}"), ("act_b", "{}")]);
    assert_eq!(
        snapshot(&session)["retention_reasons"],
        json!({"send_a@1": {"untrusted": "user_ok"}, "send_b@1": {"untrusted": "user_ok@2"}})
    );
}

#[test]
fn a_reason_is_spent_by_any_commitment_that_holds_it() {
    // `send_c` names `user_ok` without `once`, and still spends it.
    let mut session = ReactiveSession::from_source(CONFIRM).unwrap();
    run(
        &mut session,
        &[("read", "{}"), ("confirm", "{}"), ("act_c", "{}")],
    );
    assert_eq!(refusal(&mut session, "act_a"), "evaluation/reason_spent");
    // Without `once`, one confirmation covers every later send, as in 1.3.0.
    let open = CONFIRM.replace("because once user_ok", "because user_ok");
    let mut session = ReactiveSession::from_source(&open).unwrap();
    run(
        &mut session,
        &[
            ("read", "{}"),
            ("confirm", "{}"),
            ("act_a", "{}"),
            ("act_b", "{}"),
        ],
    );
}

#[test]
fn an_unobserved_reason_is_refused_before_a_spent_one() {
    let source = format!(
        "{CONFIRM}\nevidence later from \"later\";\nevent act_d;\nevent see;\non see reveal later;\n\
         on act_d commit send_c because enough using qualified(1, mail) retaining untrusted because once user_ok, other because later;\n\
         caveat other consequence low;\n"
    );
    let mut session = ReactiveSession::from_source(&source).unwrap();
    run(
        &mut session,
        &[("read", "{}"), ("confirm", "{}"), ("act_a", "{}")],
    );
    assert_eq!(
        refusal(&mut session, "act_d"),
        "evaluation/unobserved_evidence"
    );
}

#[test]
fn two_caveats_of_one_commit_may_share_a_once_reason() {
    let source = r#"
caveat a consequence low;
caveat b consequence low;
evidence ok from "a confirmation";
decisions go limit 4;
event confirm;
event act;
on confirm reveal ok;
on act commit go because enough retaining a because once ok, b because once ok;
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(&mut session, &[("confirm", "{}"), ("act", "{}")]);
    assert_eq!(
        snapshot(&session)["retention_reasons"],
        json!({"go@1": {"a": "ok", "b": "ok"}})
    );
}

#[test]
fn once_adds_no_record() {
    let open = CONFIRM.replace("because once user_ok", "because user_ok");
    let events = [
        ("read", "{}"),
        ("confirm", "{}"),
        ("act_a", "{}"),
        ("confirm", "{}"),
        ("act_b", "{}"),
    ];
    let mut once = ReactiveSession::from_source(CONFIRM).unwrap();
    let mut plain = ReactiveSession::from_source(&open).unwrap();
    run(&mut once, &events);
    run(&mut plain, &events);
    let (mut once, mut plain) = (snapshot(&once), snapshot(&plain));
    for shown in [&mut once, &mut plain] {
        let shown = shown.as_object_mut().unwrap();
        shown.remove("source_id");
        shown.remove("source_sha256");
    }
    assert_eq!(once, plain);
}

#[test]
fn once_load_rules_and_the_name_once() {
    for (line, expected) in [
        (
            "on act_a commit send_a because enough retaining untrusted because once;",
            "names evidence",
        ),
        (
            "on act_a commit send_a because enough retaining untrusted because once once user_ok;",
            "retaining expects",
        ),
        (
            "on act_a commit send_a because enough retaining untrusted once user_ok;",
            "retaining expects",
        ),
        (
            "on act_a commit send_a because enough retaining untrusted because once latest(x);",
            "",
        ),
    ] {
        let error = ReactiveSession::from_source(&format!("{CONFIRM}\n{line}"))
            .err()
            .unwrap_or_else(|| panic!("{line} should not load"));
        assert!(error.contains(expected), "{line}: {error}");
    }
    // Evidence named `once` is still a reason by that name.
    let source = r#"
caveat gap consequence low;
evidence once from "a note";
decisions keep limit 2;
event go;
on go reveal once;
on go commit keep because enough retaining gap because once;
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    run(&mut session, &[("go", "{}")]);
    assert_eq!(
        snapshot(&session)["retention_reasons"],
        json!({"keep@1": {"gap": "once"}})
    );
}

#[test]
fn once_confirmations_save_and_restore() {
    let mut session = ReactiveSession::from_source(CONFIRM).unwrap();
    run(
        &mut session,
        &[("read", "{}"), ("confirm", "{}"), ("act_a", "{}")],
    );
    let saved = session.save_json().unwrap();
    let mut restored = ReactiveSession::restore_json(CONFIRM, &saved).unwrap();
    assert_eq!(refusal(&mut restored, "act_b"), "evaluation/reason_spent");
}

/// The experiment's recipients, one decision per instruction source.
const RECIPIENTS: &str = r#"
claim action_requested;
place task kind source;
entity user kind source at task;
entity newsletter kind source at task;
entity invoice_mail kind source at task;
caveat untrusted consequence high;
event read source kind source;
event instruct source kind source;
event recipient source kind source;
event send_to instr kind source, recip kind source;
for source as $m routed by source {
    evidence $m_text from "the text of $m";
    readings $m_asks from $m_text limit 8;
    readings $m_addr from $m_text limit 8;
    on read reveal $m_text;
    on instruct sample $m_asks = 2 supports action_requested;
    on recipient sample $m_addr = 1;
};
for source as $i routed by instr {
    decisions $i_send limit 1 irreversible;
    for source as $r routed by recip {
        on send_to when not has_sample($i_asks) or not has_sample($r_addr)
            reject "Missing instruction or recipient.";
        on send_to commit $i_send because enough using latest($i_asks) + latest($r_addr);
    };
};
untrusted qualifies newsletter_text;
untrusted qualifies invoice_mail_text;
"#;

#[test]
fn a_nested_routed_block_grounds_one_decision_on_both_sources() {
    let mut session = ReactiveSession::from_source(RECIPIENTS).unwrap();
    let payload = |source: &str| format!(r#"{{"source":"{source}"}}"#);
    for source in ["user", "newsletter"] {
        session.dispatch_json("read", &payload(source)).unwrap();
    }
    session.dispatch_json("instruct", &payload("user")).unwrap();
    session
        .dispatch_json("recipient", &payload("newsletter"))
        .unwrap();
    session
        .dispatch_json("recipient", &payload("user"))
        .unwrap();
    // The recipient came from the newsletter: refused as resting on untrusted.
    let refused = serde_json::to_value(
        session
            .dispatch_outcome_json("send_to", r#"{"instr":"user","recip":"newsletter"}"#)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(refused["code"], "caveated_irreversible", "{refused}");
    // The same send to the user's own address: one decision, both sources.
    session
        .dispatch_json("send_to", r#"{"instr":"user","recip":"user"}"#)
        .unwrap();
    let shown = snapshot(&session);
    assert_eq!(
        shown["commitment_grounds"]["user_send@1"]["evidence"],
        json!(["user_addr@1", "user_asks@1"])
    );
    assert_eq!(shown["decision_series"].as_object().unwrap().len(), 3);
    // A pair whose instruction source said nothing is refused by the program.
    let missing = serde_json::to_value(
        session
            .dispatch_outcome_json("send_to", r#"{"instr":"invoice_mail","recip":"user"}"#)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(missing["code"], "reject", "{missing}");
}
