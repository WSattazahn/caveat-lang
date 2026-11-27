//! The Conway test of 1.4: every pair of the features 1.4 added, run
//! together in one program, for combinations no single spec covers and seams
//! where two disagree. Agents 0.1, Only If 0.1, Carries Any 0.1, Foreclosure
//! 0.2 (reversible foreclosure), Strict Irreversible 0.1, Untrusted Sources
//! 0.1 (`withdraw every`, `once` reasons, the undo rule), Past Revisions 0.1
//! and the what-if path of Dispatch 0.1. Each targeted test is named for the
//! pair it checks; the random run checks every pair's invariant after every
//! step, with saves and restores in place and the what-if answer beside each
//! dispatch.
use caveat_runtime::web::WebReactiveSession;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// An inbox agent. A sub-agent reads the newsletter (untrusted, material) and
/// samples its instruction and address; the user's own mail is `stale` (low).
/// The main agent may rule sends out (a reversible foreclosure), confirm one
/// send at a time (a `once` reason), send (strict irreversible), pay (strict,
/// with `only_if` reading the newsletter only in its condition) and count the
/// votes (`only_if`). An auditor withdraws everything the newsletter said.
const SOURCE: &str = r#"
claim asked;
claim safe;
evidence mail from "the newsletter as read";
evidence user_mail from "the user's request";
evidence bad from "a finding that the newsletter is malicious";
evidence user_ok from "the user confirming one pending send";
evidence review from "the review that ruled sends out";
evidence recheck from "a finding that the review was wrong";
evidence brief from "the brief that holds the ruling";
caveat untrusted consequence material;
caveat stale consequence low;
untrusted qualifies mail;
stale qualifies user_mail;
renewable user_ok limit 64;
readings asks from mail limit 8;
readings addr from mail limit 8;
readings uasks from user_mail limit 8;
decisions no_send limit 4 forecloses send;
decisions send limit 1 irreversible strict;
decisions plain_send limit 1 irreversible;
decisions payout limit 1 irreversible strict;
decisions plain_payout limit 1 irreversible;
decisions tally limit 8;
state tally_withdrawn = 0;
state first_tally_withdrawn = 0;
state addr_doubted = 0;
state addr_doubted_high = 0;
agent main sends confirm, rule_out, rule_out_on_mail, reconsider, retract, hold_it, release_it, send_it, send_plain, pay, pay_plain, pay_retained;
agent sub sends read_mail, ask, address;
agent auditor sends find_bad;
event read_mail; event read_user; event ask; event address; event uask;
event confirm; event rule_out; event rule_out_on_mail; event reconsider; event retract;
event hold_it; event release_it;
event send_it; event send_plain; event pay; event pay_plain; event pay_retained;
event count; event recount; event look; event find_bad;
define m_yes = has_sample(asks) and not withdrawn(latest(asks));
define u_yes = has_sample(uasks) and not withdrawn(latest(uasks));
on read_mail when not observed(mail) reveal mail supports asked;
on read_user when not observed(user_mail) reveal user_mail supports asked;
on ask sample asks = 1 supports asked;
on address sample addr = 7;
on uask sample uasks = 1 supports asked;
on confirm when observed(user_ok) renew user_ok;
on confirm reveal user_ok;
on rule_out when not observed(review) reveal review supports safe;
on rule_out commit no_send because enough citing qualified(1, review);
on rule_out_on_mail commit no_send because enough citing latest(asks);
on reconsider when not observed(recheck) reveal recheck opposes safe;
on reconsider reopen no_send because recheck;
on retract when not observed(recheck) reveal recheck opposes safe;
on retract when observed(review) withdraw review because recheck;
on retract when committed(no_send) and not reopened(no_send) and rests_on_withdrawn(no_send) reopen no_send because recheck;
on hold_it when not observed(brief) reveal brief;
on hold_it hold no_send because brief;
on release_it release no_send because brief;
on send_it when not has_sample(asks) or not has_sample(addr) reject "nothing to send";
on send_it when carries_any(latest(addr), high) reject "address in doubt";
on send_it commit send because enough using latest(asks) + latest(addr) retaining untrusted because once user_ok;
on send_plain when not has_sample(asks) reject "nothing to send";
on send_plain commit plain_send because enough using 1 retaining untrusted because once user_ok;
on pay commit payout because enough using only_if(m_yes, 1) + 40;
on pay_plain commit plain_payout because enough using only_if(m_yes, 1) + 40;
on pay_retained commit payout because enough using only_if(m_yes, 1) + 40 retaining untrusted;
on count commit tally because enough using only_if(m_yes, latest(asks)) + only_if(u_yes, latest(uasks));
on recount when not observed(recheck) reveal recheck opposes safe;
on recount reopen tally because recheck;
on look set tally_withdrawn = if(rests_on_withdrawn(tally), 1, 0);
on look set first_tally_withdrawn = if(rests_on_withdrawn(revision(tally, 1)), 1, 0);
on look set addr_doubted = if(carries_any(latest(addr)), 1, 0);
on look set addr_doubted_high = if(carries_any(latest(addr), high), 1, 0);
on find_bad when not observed(bad) reveal bad opposes asked;
on find_bad withdraw every mail because bad;
"#;

/// Who may send what, as SOURCE declares it.
const AGENTS: &[(&str, &[&str])] = &[
    (
        "main",
        &[
            "confirm",
            "rule_out",
            "rule_out_on_mail",
            "reconsider",
            "retract",
            "hold_it",
            "release_it",
            "send_it",
            "send_plain",
            "pay",
            "pay_plain",
            "pay_retained",
        ],
    ),
    ("sub", &["read_mail", "ask", "address"]),
    ("auditor", &["find_bad"]),
];

const EVENTS: &[&str] = &[
    "read_mail",
    "read_user",
    "ask",
    "address",
    "uask",
    "confirm",
    "rule_out",
    "rule_out_on_mail",
    "reconsider",
    "retract",
    "hold_it",
    "release_it",
    "send_it",
    "send_plain",
    "pay",
    "pay_plain",
    "pay_retained",
    "count",
    "recount",
    "look",
    "find_bad",
];

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn session() -> WebReactiveSession {
    WebReactiveSession::new(SOURCE).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

/// The agent that may send `event`, by the declaration table.
fn sender(event: &str) -> Option<&'static str> {
    AGENTS
        .iter()
        .find(|(_, events)| events.contains(&event))
        .map(|(agent, _)| *agent)
}

fn dispatch(game: &mut WebReactiveSession, event: &str, agent: Option<&str>) -> Value {
    let outcome = match agent {
        Some(agent) => game.dispatch_outcome_as(event, "{}", agent),
        None => game.dispatch_outcome(event, "{}"),
    };
    json(&outcome.unwrap_or_else(|fatal| panic!("{event} by {agent:?} was fatal: {fatal}")))
}

/// Accepted, sent by the agent the declaration names.
fn accept(game: &mut WebReactiveSession, event: &str) -> Value {
    let outcome = dispatch(game, event, sender(event));
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    outcome
}

fn checkpoint(game: &WebReactiveSession) -> (String, String, String) {
    (game.snapshot(), game.view(), game.save().unwrap())
}

/// Refused as `origin/code`, sent by the declared agent, and nothing changed.
fn refuse(game: &mut WebReactiveSession, event: &str, code: &str) -> String {
    let before = checkpoint(game);
    let outcome = dispatch(game, event, sender(event));
    assert_eq!(outcome["outcome"], "rejected", "{event}: {outcome}");
    assert_eq!(
        format!(
            "{}/{}",
            outcome["origin"].as_str().unwrap(),
            outcome["code"].as_str().unwrap()
        ),
        code,
        "{event}: {outcome}"
    );
    assert_eq!(
        checkpoint(game),
        before,
        "{event}: the refusal changed something"
    );
    outcome["message"].as_str().unwrap().to_string()
}

fn snapshot(game: &WebReactiveSession) -> Value {
    json(&game.snapshot())
}

fn state(game: &WebReactiveSession, name: &str) -> f64 {
    let value = &snapshot(game)["values"][name];
    value
        .as_f64()
        .or_else(|| value["value"].as_f64())
        .unwrap_or_else(|| panic!("{name}: {value}"))
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|item| item.as_str().unwrap().to_string())
        .collect()
}

fn restored(game: &WebReactiveSession) -> WebReactiveSession {
    let saved = game.save().unwrap();
    let twin = WebReactiveSession::restore(SOURCE, &saved)
        .unwrap_or_else(|error| panic!("restore: {error}"));
    assert_eq!(
        twin.save().unwrap(),
        saved,
        "a restored session saves the same"
    );
    assert_eq!(
        twin.snapshot(),
        game.snapshot(),
        "a restored session is the same session"
    );
    twin
}

/// Agents × what-if. The agentless what-if path answers as a dispatch with
/// no agent would: an event some agent sends is always `wrong_agent`,
/// whatever the agent would get (the 1.4.0 behavior, which stays). From
/// D027, `what_if_view_outcome_as` answers as that agent's dispatch would.
#[test]
fn what_if_answers_an_agents_event_as_sent_by_nobody() {
    let mut game = session();
    for event in ["read_mail", "ask", "address"] {
        accept(&mut game, event);
    }
    accept(&mut game, "confirm");
    let before = checkpoint(&game);
    let asked = json(&game.what_if_view_outcome("send_it", "{}").unwrap());
    assert_eq!(checkpoint(&game), before, "asking changed something");
    assert_eq!(asked["outcome"], "rejected", "{asked}");
    assert_eq!(asked["code"], "wrong_agent", "{asked}");
    // What the main agent would get is not what the host was told.
    let mut twin = restored(&game);
    let theirs = json(
        &twin
            .dispatch_view_outcome_as("send_it", "{}", "main")
            .unwrap(),
    );
    assert_eq!(theirs["outcome"], "accepted", "{theirs}");
    // Asked as the agent, the answer is that agent's dispatch, byte for byte,
    // and asking still changes nothing (D027).
    let asked_as = game
        .what_if_view_outcome_as("send_it", "{}", "main")
        .unwrap();
    assert_eq!(
        checkpoint(&game),
        before,
        "asking as an agent changed something"
    );
    let mut twin = restored(&game);
    assert_eq!(
        asked_as,
        twin.dispatch_view_outcome_as("send_it", "{}", "main")
            .unwrap()
    );
    // And the agentless answer is, byte for byte, the agentless dispatch.
    let mut twin = restored(&game);
    assert_eq!(
        game.what_if_view_outcome("send_it", "{}").unwrap(),
        twin.dispatch_view_outcome("send_it", "{}").unwrap()
    );
    // An event no agent lists is answered as the spec says.
    let mut twin = restored(&game);
    assert_eq!(
        game.what_if_view_outcome("count", "{}").unwrap(),
        twin.dispatch_view_outcome("count", "{}").unwrap()
    );
    assert_eq!(
        json(&game.what_if_view_outcome("count", "{}").unwrap())["outcome"],
        "accepted"
    );
}

/// Strict Irreversible × Only If. A caveat read only by an `only_if`
/// condition is lineage, not grounds, so a plain irreversible commit does
/// not ask for it and a strict one does.
#[test]
fn strict_reads_the_caveat_an_only_if_condition_read() {
    let mut game = session();
    for event in ["read_mail", "ask"] {
        accept(&mut game, event);
    }
    let message = refuse(&mut game, "pay", "evaluation/caveated_irreversible");
    assert!(
        message.contains("payout rests on untrusted (material), which its commit does not retain"),
        "{message}"
    );
    accept(&mut game, "pay_plain");
    let shown = snapshot(&game);
    assert_eq!(
        strings(&shown["commitment_grounds"]["plain_payout@1"]["caveats"]),
        Vec::<String>::new(),
        "the condition's caveat is not on the grounds"
    );
    assert_eq!(
        strings(&shown["commitment_bases"]["plain_payout@1"]["provenance"]["caveats"]),
        ["untrusted"],
        "but it is in the lineage"
    );
    assert_eq!(shown["commitment_bases"]["plain_payout@1"]["value"], 41.0);
    // Retaining it commits the strict series, with the caveat attached.
    accept(&mut game, "pay_retained");
    let shown = snapshot(&game);
    assert_eq!(
        strings(&shown["commitment_grounds"]["payout@1"]["caveats"]),
        ["untrusted"]
    );
    assert_eq!(
        strings(&shown["commitment_grounds"]["payout@1"]["evidence"]),
        Vec::<String>::new()
    );
    assert_eq!(
        strings(&shown["commitment_bases"]["payout@1"]["provenance"]["evidence"]),
        ["asks@1"]
    );
    restored(&game);
}

/// Carries Any × `withdraw every`, and Only If × `withdraw every` × Past
/// Revisions. `withdraw every mail` puts `withdrawn` (material) on every
/// reading from the newsletter, which `carries_any` sees at `material` and
/// not at `high`; the tally that rested on the newsletter's reading rests on
/// withdrawn evidence; the next tally reads it only in a condition, so its
/// grounds are clean, but the revision carries `withdrawn` by its lineage.
#[test]
fn withdraw_every_reaches_carries_any_and_only_if_alike() {
    let mut game = session();
    for event in [
        "read_mail",
        "read_user",
        "ask",
        "address",
        "uask",
        "count",
        "look",
    ] {
        accept(&mut game, event);
    }
    assert_eq!(state(&game, "addr_doubted"), 1.0, "untrusted is material");
    assert_eq!(state(&game, "addr_doubted_high"), 0.0);
    assert_eq!(state(&game, "tally_withdrawn"), 0.0);
    let accepted = accept(&mut game, "find_bad");
    let effects = accepted["snapshot"]["effects"].as_array().unwrap();
    let withdrawn: Vec<&str> = effects
        .iter()
        .filter(|effect| effect["kind"] == "withdraw")
        .map(|effect| effect["evidence"].as_str().unwrap())
        .collect();
    assert_eq!(
        withdrawn,
        ["mail", "addr@1", "asks@1"],
        "the source, then each stream in name order"
    );
    accept(&mut game, "look");
    assert_eq!(state(&game, "addr_doubted"), 1.0);
    assert_eq!(
        state(&game, "addr_doubted_high"),
        0.0,
        "withdrawn is material, not high"
    );
    assert_eq!(
        state(&game, "tally_withdrawn"),
        1.0,
        "tally@1 rests on asks@1"
    );
    assert_eq!(state(&game, "first_tally_withdrawn"), 1.0);
    for event in ["recount", "count", "look"] {
        accept(&mut game, event);
    }
    let shown = snapshot(&game);
    assert_eq!(
        strings(&shown["commitment_grounds"]["tally@2"]["evidence"]),
        ["uasks@1"],
        "the withdrawn vote stays out of the grounds"
    );
    assert_eq!(
        strings(&shown["commitment_bases"]["tally@2"]["provenance"]["evidence"]),
        ["asks@1", "bad", "recheck", "uasks@1"],
        "and in the lineage, with the withdrawal's reason (withdrawn(latest(asks)) \
         was true) and the reopening's cause (the predecessor's basis)"
    );
    assert!(
        strings(&shown["commitment_bases"]["tally@2"]["provenance"]["caveats"])
            .contains(&"withdrawn".to_string()),
        "a value whose lineage holds withdrawn evidence carries withdrawn: {}",
        shown["commitment_bases"]["tally@2"]
    );
    assert_eq!(
        state(&game, "tally_withdrawn"),
        0.0,
        "tally@2 does not rest on it"
    );
    assert_eq!(
        state(&game, "first_tally_withdrawn"),
        1.0,
        "tally@1 still does"
    );
    // Strict × withdraw every: the send's guard and grounds now carry
    // withdrawn, which nothing retains.
    accept(&mut game, "confirm");
    let message = refuse(&mut game, "send_it", "evaluation/caveated_irreversible");
    assert!(message.contains("withdrawn (material)"), "{message}");
    restored(&game);
}

/// Foreclosure 0.2 × `withdraw every` × Holds. A foreclosure grounded on the
/// newsletter stands after the newsletter is withdrawn until a rule reopens
/// it, held or not; a hold defers that reopening only while the revision
/// rests on nothing withdrawn (Holds 0.1, "A withdrawal is not held"), so
/// the reopening of the withdrawn-on revision goes ahead and lifts the
/// exclusion.
#[test]
fn withdraw_every_lifts_a_foreclosure_only_through_a_reopening() {
    let mut game = session();
    for event in ["read_mail", "ask", "address", "confirm", "rule_out_on_mail"] {
        accept(&mut game, event);
    }
    refuse(&mut game, "send_it", "evaluation/foreclosed");
    accept(&mut game, "find_bad");
    // Rests on withdrawn evidence, still in force: foreclosed first, before
    // the caveat check.
    let message = refuse(&mut game, "send_it", "evaluation/foreclosed");
    assert!(
        message.ends_with("send is foreclosed by no_send@1"),
        "{message}"
    );
    accept(&mut game, "reconsider");
    let message = refuse(&mut game, "send_it", "evaluation/caveated_irreversible");
    assert!(message.contains("withdrawn (material)"), "{message}");

    // Held: the withdrawal alone reopens nothing, and the exclusion stands.
    let mut game = session();
    for event in [
        "read_mail",
        "ask",
        "address",
        "confirm",
        "rule_out_on_mail",
        "hold_it",
    ] {
        accept(&mut game, event);
    }
    refuse(&mut game, "send_it", "evaluation/foreclosed");
    accept(&mut game, "find_bad");
    let journal = snapshot(&game)["decision_journal"].clone();
    let last = journal.as_array().unwrap().last().unwrap().clone();
    assert_eq!(
        (last["decision"].as_str(), last["change"].as_str()),
        (Some("no_send"), Some("held")),
        "{journal}"
    );
    refuse(&mut game, "send_it", "evaluation/foreclosed");
    // A reopening while on hold would be deferred, but this revision rests on
    // withdrawn evidence, so it goes ahead and the hold ends with it.
    accept(&mut game, "retract");
    let journal = snapshot(&game)["decision_journal"].clone();
    let last = journal.as_array().unwrap().last().unwrap().clone();
    assert_eq!(
        (last["decision"].as_str(), last["change"].as_str()),
        (Some("no_send"), Some("reopened")),
        "{journal}"
    );
    assert_eq!(last["agent"], "main", "{last}");
    refuse(&mut game, "send_it", "evaluation/caveated_irreversible");
    // Held and resting on nothing withdrawn: the same reopening waits.
    let mut game = session();
    for event in [
        "read_mail",
        "ask",
        "address",
        "confirm",
        "rule_out",
        "hold_it",
        "reconsider",
    ] {
        accept(&mut game, event);
    }
    refuse(&mut game, "send_it", "evaluation/foreclosed");
    accept(&mut game, "release_it");
    accept(&mut game, "send_it");
    restored(&game);
}

/// `once` × Foreclosure 0.2 × Strict Irreversible: the order of the checks.
/// `foreclosed` first, then `reason_spent`, then `caveated_irreversible`.
#[test]
fn a_spent_reason_is_checked_after_foreclosure_and_before_the_caveats() {
    let mut game = session();
    for event in ["read_mail", "ask", "address", "confirm", "send_plain"] {
        accept(&mut game, event);
    }
    // user_ok is spent by plain_send@1; send_it would also be caveated were
    // the retention honored, but the spent reason is reported.
    let message = refuse(&mut game, "send_it", "evaluation/reason_spent");
    assert!(
        message.ends_with("user_ok is already the reason of a caveat plain_send@1 retains"),
        "{message}"
    );
    accept(&mut game, "rule_out");
    refuse(&mut game, "send_it", "evaluation/foreclosed");
    accept(&mut game, "reconsider");
    refuse(&mut game, "send_it", "evaluation/reason_spent");
    accept(&mut game, "confirm");
    accept(&mut game, "send_it");
    let shown = snapshot(&game);
    assert_eq!(
        shown["retention_reasons"]["plain_send@1"]["untrusted"],
        "user_ok"
    );
    assert_eq!(
        shown["retention_reasons"]["send@1"]["untrusted"],
        "user_ok@2"
    );
    assert_eq!(
        strings(&shown["commitment_grounds"]["send@1"]["evidence"]),
        ["addr@1", "asks@1", "user_ok@2"]
    );
    restored(&game);
}

/// Agents × every record: the journal, the readings, the withdrawals and
/// `last_agent` carry the sender, and a save restores them.
#[test]
fn every_record_names_the_agent_that_wrote_it() {
    let mut game = session();
    for event in [
        "read_mail",
        "read_user",
        "ask",
        "address",
        "uask",
        "count",
        "rule_out",
        "confirm",
        "send_plain",
        "find_bad",
    ] {
        accept(&mut game, event);
    }
    let shown = snapshot(&game);
    for entry in shown["decision_journal"].as_array().unwrap() {
        let event = entry["event"].as_str().unwrap();
        match sender(event) {
            Some(agent) => assert_eq!(entry["agent"], agent, "{entry}"),
            None => assert!(entry.get("agent").is_none(), "{entry}"),
        }
    }
    for (stream, agent) in [
        ("asks", Some("sub")),
        ("addr", Some("sub")),
        ("uasks", None),
    ] {
        let occurrence = &shown["reading_streams"][stream]["occurrences"][0];
        match agent {
            Some(agent) => assert_eq!(occurrence["agent"], agent, "{occurrence}"),
            None => assert!(occurrence.get("agent").is_none(), "{occurrence}"),
        }
    }
    let withdrawals = shown["withdrawals"].as_array().unwrap();
    assert_eq!(withdrawals.len(), 3, "{withdrawals:?}");
    for record in withdrawals {
        assert_eq!(record["agent"], "auditor", "{record}");
        assert_eq!(record["event"], "find_bad", "{record}");
    }
    assert_eq!(shown["last_agent"], "auditor");
    restored(&game);
    // A save that says the sub-agent made the main agent's commit does not
    // restore, and one that says nobody withdrew where the auditor did.
    let mut forged = json(&game.save().unwrap());
    let entry = forged["decision_journal"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["decision"] == "no_send")
        .unwrap();
    entry["agent"] = Value::from("sub");
    let error = WebReactiveSession::restore(SOURCE, &forged.to_string())
        .err()
        .unwrap();
    assert!(error.contains("agent"), "{error}");
}

/// Agents × Untrusted Sources: the nested routed block that grounds one send
/// on two sources, with strict series, a `once` confirmation and agents on
/// each side of the handoff.
#[test]
fn a_two_source_send_is_strict_once_and_by_the_right_agent() {
    const RECIPIENTS: &str = r#"
claim action_requested;
place task kind source;
entity user kind source at task;
entity newsletter kind source at task;
evidence user_ok from "the user confirming one pending send";
renewable user_ok limit 8;
caveat untrusted consequence material;
agent reader sends read, instruct, recipient;
agent main sends confirm, send_to;
event read source kind source;
event instruct source kind source;
event recipient source kind source;
event confirm;
event send_to instr kind source, recip kind source;
for source as $m routed by source {
    evidence $m_text from "the text of $m";
    readings $m_asks from $m_text limit 8;
    readings $m_addr from $m_text limit 8;
    on read reveal $m_text;
    on instruct sample $m_asks = 2 supports action_requested;
    on recipient sample $m_addr = 1;
};
on confirm when observed(user_ok) renew user_ok;
on confirm reveal user_ok;
for source as $i routed by instr {
    decisions $i_send limit 1 irreversible strict;
    for source as $r routed by recip {
        on send_to when not has_sample($i_asks) or not has_sample($r_addr)
            reject "Missing instruction or recipient.";
        on send_to commit $i_send because enough using latest($i_asks) + latest($r_addr)
            retaining untrusted because once user_ok;
    };
};
untrusted qualifies newsletter_text;
"#;
    let mut game = WebReactiveSession::new(RECIPIENTS).unwrap_or_else(|error| panic!("{error}"));
    let send = |game: &mut WebReactiveSession, event: &str, payload: &str, agent: &str| {
        json(&game.dispatch_outcome_as(event, payload, agent).unwrap())
    };
    for source in ["user", "newsletter"] {
        let payload = format!(r#"{{"source":"{source}"}}"#);
        assert_eq!(
            send(&mut game, "read", &payload, "reader")["outcome"],
            "accepted"
        );
        assert_eq!(
            send(&mut game, "instruct", &payload, "reader")["outcome"],
            "accepted"
        );
        assert_eq!(
            send(&mut game, "recipient", &payload, "reader")["outcome"],
            "accepted"
        );
    }
    // The agent check comes before the route and the guard: the reader may
    // not send, whatever the pair.
    let outcome = send(
        &mut game,
        "send_to",
        r#"{"instr":"user","recip":"user"}"#,
        "reader",
    );
    assert_eq!(outcome["code"], "wrong_agent", "{outcome}");
    // Unobserved reason first, as Untrusted Sources says.
    let outcome = send(
        &mut game,
        "send_to",
        r#"{"instr":"user","recip":"user"}"#,
        "main",
    );
    assert_eq!(outcome["code"], "unobserved_evidence", "{outcome}");
    assert_eq!(
        send(&mut game, "confirm", "{}", "main")["outcome"],
        "accepted"
    );
    // The newsletter's address is untrusted; retained because the user
    // confirmed, so the strict commit goes through on one confirmation.
    let outcome = send(
        &mut game,
        "send_to",
        r#"{"instr":"user","recip":"newsletter"}"#,
        "main",
    );
    assert_eq!(outcome["outcome"], "accepted", "{outcome}");
    let shown = json(&game.snapshot());
    assert_eq!(
        strings(&shown["commitment_grounds"]["user_send@1"]["evidence"]),
        ["newsletter_addr@1", "user_asks@1", "user_ok"]
    );
    assert_eq!(shown["decision_journal"][0]["agent"], "main");
    // The confirmation is spent: the newsletter's own send needs another.
    let outcome = send(
        &mut game,
        "send_to",
        r#"{"instr":"newsletter","recip":"user"}"#,
        "main",
    );
    assert_eq!(outcome["code"], "reason_spent", "{outcome}");
    assert_eq!(
        send(&mut game, "confirm", "{}", "main")["outcome"],
        "accepted"
    );
    let outcome = send(
        &mut game,
        "send_to",
        r#"{"instr":"newsletter","recip":"user"}"#,
        "main",
    );
    assert_eq!(outcome["outcome"], "accepted", "{outcome}");
    let saved = game.save().unwrap();
    let twin = WebReactiveSession::restore(RECIPIENTS, &saved).unwrap();
    assert_eq!(twin.snapshot(), game.snapshot());
}

/// Random events over SOURCE, each sent by a random agent (the right one,
/// another, none or an undeclared one), with saves and restores in place and
/// the what-if answer beside every dispatch. After every step:
/// - `wrong_agent` exactly when the declaration table says so, before every
///   other check;
/// - a refusal changed nothing;
/// - the what-if answer is the agentless dispatch's, byte for byte, and
///   asking changed nothing;
/// - no `send` was committed while `no_send` was in force, by the journal;
/// - no `user_ok` occurrence is the reason of two commitments' caveats;
/// - every record of an accepted event names its agent, or none;
/// - every strict commit's lineage carries no unretained material caveat;
/// - a restored session is the same session.
#[test]
fn random_events_keep_every_pairs_invariant() {
    const SENDERS: &[Option<&str>] = &[
        Some("main"),
        Some("sub"),
        Some("auditor"),
        None,
        Some("mallory"),
    ];
    let mut seed: u64 = 0x1f3a_5c2b_9d7e_4a61;
    let mut next = move |bound: usize| {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (seed >> 33) as usize % bound
    };
    let mut codes = BTreeMap::<String, usize>::new();
    for run in 0..200 {
        let mut game = session();
        let mut trail: Vec<String> = Vec::new();
        for step in 0..48 {
            let pick = next(EVENTS.len() + 1);
            if pick == EVENTS.len() {
                trail.push("save".into());
                game = restored(&game);
                continue;
            }
            let event = EVENTS[pick];
            // The right agent three times in four, so the program gets somewhere.
            let agent = if next(4) == 0 {
                SENDERS[next(SENDERS.len())]
            } else {
                sender(event)
            };
            trail.push(format!("{event}@{}", agent.unwrap_or("-")));
            let before = checkpoint(&game);
            // What-if beside the dispatch: the agentless answer, and no change.
            let asked = game.what_if_view_outcome(event, "{}").unwrap();
            assert_eq!(
                checkpoint(&game),
                before,
                "run {run} step {step}: asking changed something {trail:?}"
            );
            let mut twin = restored(&game);
            assert_eq!(
                asked,
                twin.dispatch_view_outcome(event, "{}").unwrap(),
                "run {run} step {step} {trail:?}"
            );
            let outcome = dispatch(&mut game, event, agent);
            let wrong = match (sender(event), agent) {
                (_, Some("mallory")) => true,
                (Some(listed), named) => named != Some(listed),
                (None, _) => false,
            };
            if outcome["outcome"] != "accepted" {
                let code = format!(
                    "{}/{}",
                    outcome["origin"].as_str().unwrap(),
                    outcome["code"].as_str().unwrap()
                );
                assert_eq!(
                    code == "input/wrong_agent",
                    wrong,
                    "run {run} step {step}: {outcome} {trail:?}"
                );
                *codes.entry(code).or_default() += 1;
                assert_eq!(
                    checkpoint(&game),
                    before,
                    "run {run} step {step}: the refusal changed something {trail:?}"
                );
                continue;
            }
            assert!(
                !wrong,
                "run {run} step {step}: accepted from the wrong agent {trail:?}"
            );
            let shown = snapshot(&game);
            let sequence = shown["sequence"].as_u64().unwrap();
            // Foreclosure, by the journal.
            let mut in_force = false;
            for entry in shown["decision_journal"].as_array().unwrap() {
                match (
                    entry["decision"].as_str().unwrap(),
                    entry["change"].as_str().unwrap(),
                ) {
                    ("no_send", "committed") => in_force = true,
                    ("no_send", "reopened") => in_force = false,
                    ("send", "committed") => assert!(
                        !in_force,
                        "run {run} step {step}: {entry} while ruled out {trail:?}"
                    ),
                    _ => {}
                }
                // Agents on this event's records.
                if entry["sequence"].as_u64() == Some(sequence) {
                    assert_eq!(
                        entry.get("agent").and_then(Value::as_str),
                        agent,
                        "run {run} step {step}: {entry} {trail:?}"
                    );
                }
            }
            for record in shown["withdrawals"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default()
            {
                if record["sequence"].as_u64() == Some(sequence) {
                    assert_eq!(
                        record.get("agent").and_then(Value::as_str),
                        agent,
                        "run {run} step {step}: {record} {trail:?}"
                    );
                }
            }
            for (_, stream) in shown["reading_streams"].as_object().unwrap() {
                for occurrence in stream["occurrences"].as_array().unwrap() {
                    if occurrence["sequence"].as_u64() == Some(sequence) {
                        assert_eq!(
                            occurrence.get("agent").and_then(Value::as_str),
                            agent,
                            "run {run} step {step}: {occurrence} {trail:?}"
                        );
                    }
                }
            }
            match agent {
                Some(agent) => assert_eq!(shown["last_agent"], agent),
                None => assert!(
                    shown.get("last_agent").is_none_or(Value::is_null),
                    "{}",
                    shown["last_agent"]
                ),
            }
            // Once: a reason spent by one commitment only.
            let mut spent = BTreeSet::new();
            for (commitment, reasons) in shown["retention_reasons"]
                .as_object()
                .map(|map| map.iter().collect::<Vec<_>>())
                .unwrap_or_default()
            {
                let mut own = BTreeSet::new();
                for (_, reason) in reasons.as_object().unwrap() {
                    let reason = reason.as_str().unwrap().to_string();
                    if reason.starts_with("user_ok") && own.insert(reason.clone()) {
                        assert!(spent.insert(reason.clone()), "run {run} step {step}: {reason} spent twice, again by {commitment} {trail:?}");
                    }
                }
            }
            // Strict: no unretained material caveat in a strict commit's lineage.
            for series in ["send", "payout"] {
                if let Some(current) = shown["decision_series"][series]["current"].as_str() {
                    let lineage =
                        strings(&shown["commitment_bases"][current]["provenance"]["caveats"]);
                    let retained = shown["commitments"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|commitment| commitment["action"] == current)
                        .map(|commitment| strings(&commitment["retained"]))
                        .unwrap_or_default();
                    for caveat in lineage {
                        if caveat == "untrusted" || caveat == "withdrawn" {
                            // `withdrawn` applied after the commit is a late
                            // caveat and not what the commit read.
                            if caveat == "untrusted" {
                                assert!(retained.contains(&caveat), "run {run} step {step}: {current} carries {caveat} unretained {trail:?}");
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("{codes:?}");
    for code in [
        "input/wrong_agent",
        "evaluation/foreclosed",
        "evaluation/reason_spent",
        "evaluation/caveated_irreversible",
        "evaluation/decision_in_force",
        "policy/reject",
    ] {
        assert!(
            codes.contains_key(code),
            "the random check never reached {code}: {codes:?}"
        );
    }
}
