//! Untrusted Sources 0.1 under random histories: `withdraw every NAME`
//! withdraws exactly the held, observed occurrences the name stands for (a
//! stream's readings oldest first; evidence's occurrences first to current,
//! then the readings of each stream read from it, streams in name order),
//! once each, reporting one withdrawal per occurrence, and is refused as
//! unobserved_evidence when its reason is unobserved even with nothing to
//! list; a `once` reason backs one commitment and a second is refused as
//! reason_spent, while a plain reason spends it too; a value read from a
//! withdrawn reading carries `withdrawn`; every event's save restores; and a
//! session restored from the live save runs on as the live one.
//! See spec/caveat-untrusted-sources-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeSet;

const SOURCE: &str = r#"
claim asked;
caveat untrusted consequence material;
evidence mail from "the mail as read";
evidence bad from "a finding that the mail is malicious";
evidence note from "a note the mail left";
evidence user_ok from "the user confirming one pending send";
untrusted qualifies mail;
renewable note limit 64;
renewable user_ok limit 64;
readings asks from mail limit 64;
readings addr from mail limit 64;
state copied = 0;
decisions send limit 64;
decisions send_plain limit 64;
event read;
event ask;
event address;
event jot;
event keep;
event find;
event confirm;
event act;
event act_plain;
event drop_asks;
event drop_addr;
event drop_mail;
event drop_note;
event drop_unfound;
event reconsider;
on read when not observed(mail) reveal mail;
on ask when observed(mail) sample asks = 1 supports asked;
on address when observed(mail) sample addr = 7;
on jot when observed(note) renew note;
on jot reveal note;
on keep when has_sample(asks) set copied = latest(asks);
on find when not observed(bad) reveal bad;
on confirm when observed(user_ok) renew user_ok;
on confirm reveal user_ok;
on act when has_sample(asks) and observed(user_ok) and (not committed(send) or reopened(send)) commit send because enough using latest(asks) retaining untrusted because once user_ok;
on act_plain when has_sample(asks) and observed(user_ok) and (not committed(send_plain) or reopened(send_plain)) commit send_plain because enough using latest(asks) retaining untrusted because user_ok;
on drop_asks when observed(bad) withdraw every asks because bad;
on drop_addr when observed(bad) withdraw every addr because bad;
on drop_mail when observed(bad) withdraw every mail because bad;
on drop_note when observed(bad) withdraw every note because bad;
on drop_unfound withdraw every asks because bad;
on reconsider when committed(send) and not reopened(send) and observed(bad) reopen send because bad;
on reconsider when committed(send_plain) and not reopened(send_plain) and observed(bad) reopen send_plain because bad;
"#;

const EVENTS: [&str; 22] = [
    "read",
    "ask",
    "ask",
    "ask",
    "address",
    "address",
    "jot",
    "jot",
    "keep",
    "find",
    "confirm",
    "confirm",
    "act",
    "act",
    "act_plain",
    "drop_asks",
    "drop_addr",
    "drop_mail",
    "drop_note",
    "drop_unfound",
    "reconsider",
    "reconsider",
];

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn withdrawals(shown: &Value) -> Vec<String> {
    shown["withdrawals"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|record| record["evidence"].as_str().unwrap().to_string())
        .collect()
}

fn withdraw_effects(shown: &Value) -> Vec<String> {
    shown["effects"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter(|effect| effect["kind"] == "withdraw")
        .map(|effect| {
            assert_eq!(effect["because"], "bad");
            effect["evidence"].as_str().unwrap().to_string()
        })
        .collect()
}

fn caveats_of(shown: &Value, state: &str) -> Vec<String> {
    shown["qualified_values"][state]["provenance"]["caveats"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|caveat| caveat.as_str().unwrap().to_string())
        .collect()
}

/// A renewable's k-th occurrence: the bare name, then NAME@k.
fn occurrence(name: &str, k: usize) -> String {
    if k <= 1 {
        name.to_string()
    } else {
        format!("{name}@{k}")
    }
}

#[derive(Default)]
struct Model {
    mail: bool,
    bad: bool,
    asks: usize,
    addr: usize,
    notes: usize,
    confirmations: usize,
    withdrawn: Vec<String>,
    spent: BTreeSet<String>,
    send_in_force: bool,
    plain_in_force: bool,
    /// The reading `copied` was last set from.
    copied_from: Option<String>,
}

impl Model {
    fn reading_ids(stream: &str, count: usize) -> Vec<String> {
        (1..=count).map(|i| format!("{stream}@{i}")).collect()
    }

    /// What `withdraw every NAME` lists (Untrusted Sources 0.1, section 4).
    fn every(&self, name: &str) -> Vec<String> {
        match name {
            "asks" => Self::reading_ids("asks", self.asks),
            "addr" => Self::reading_ids("addr", self.addr),
            "mail" => {
                let mut listed = Vec::new();
                if self.mail {
                    listed.push("mail".to_string());
                }
                listed.extend(Self::reading_ids("addr", self.addr));
                listed.extend(Self::reading_ids("asks", self.asks));
                listed
            }
            "note" => (1..=self.notes).map(|k| occurrence("note", k)).collect(),
            _ => unreachable!(),
        }
    }

    /// Withdraw what the name lists, once each; the new withdrawals in order.
    fn withdraw_every(&mut self, name: &str) -> Vec<String> {
        let mut new = Vec::new();
        for id in self.every(name) {
            if !self.withdrawn.contains(&id) {
                self.withdrawn.push(id.clone());
                new.push(id);
            }
        }
        new
    }
}

#[test]
fn the_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn every_withdraws_what_the_name_stands_for_and_once_spends_a_reason() {
    let mut seed: u64 = 0x6e7a_9c2b_d4f1_0a35;
    let mut next = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };
    let (
        mut accepted,
        mut refused,
        mut withdrawn_total,
        mut every_withdrawing,
        mut every_empty,
        mut spent_refusals,
        mut sends,
        mut plain_sends,
        mut copied_withdrawn,
        mut round_trips,
    ) = (
        0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize,
    );
    for run in 0..200 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut model = Model::default();
        let mut before = snapshot(&game);
        for step in 0..60 {
            let at = format!("run {run} step {step}");
            let pick = next() % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                let save = game.save_json().unwrap();
                let restored = ReactiveSession::restore_json(SOURCE, &save)
                    .unwrap_or_else(|error| panic!("{at}: restore: {error}"));
                assert_eq!(restored.save_json().unwrap(), save, "{at}: save round trip");
                assert_eq!(snapshot(&restored), before, "{at}: restored snapshot");
                game = restored;
                round_trips += 1;
                continue;
            }
            let event = EVENTS[pick];
            let save_before = game.save_json().unwrap();
            let mut mirror = ReactiveSession::restore_json(SOURCE, &save_before).unwrap();
            let outcome: Value =
                serde_json::to_value(game.dispatch_outcome_json(event, "{}").unwrap()).unwrap();
            let mirrored: Value =
                serde_json::to_value(mirror.dispatch_outcome_json(event, "{}").unwrap()).unwrap();
            assert_eq!(
                mirrored, outcome,
                "{at}: the restored session ran differently"
            );

            let user_ok = occurrence("user_ok", model.confirmations);
            let can_send = model.asks > 0 && model.confirmations > 0;
            let act_fires = can_send && !model.send_in_force;
            let plain_fires = can_send && !model.plain_in_force;
            let expected: Option<&str> = match event {
                "drop_unfound" if !model.bad => Some("unobserved_evidence"),
                "act" if act_fires && model.spent.contains(&user_ok) => Some("reason_spent"),
                _ => None,
            };
            if let Some(code) = expected {
                assert_eq!(outcome["outcome"], "rejected", "{at}: {event}: {outcome}");
                assert_eq!(outcome["code"], code, "{at}: {event}: {outcome}");
                assert_eq!(
                    game.save_json().unwrap(),
                    save_before,
                    "{at}: a refusal changed the save"
                );
                refused += 1;
                if code == "reason_spent" {
                    spent_refusals += 1;
                }
                continue;
            }
            assert_eq!(outcome["outcome"], "accepted", "{at}: {event}: {outcome}");
            accepted += 1;
            let shown = snapshot(&game);
            assert_eq!(
                snapshot(&mirror),
                shown,
                "{at}: the restored session's snapshot differs"
            );
            let mut new_withdrawals: Vec<String> = Vec::new();
            match event {
                "read" => model.mail = true,
                "ask" if model.mail => model.asks += 1,
                "address" if model.mail => model.addr += 1,
                "jot" => model.notes += 1,
                "keep" if model.asks > 0 => {
                    model.copied_from = Some(format!("asks@{}", model.asks));
                }
                "find" => model.bad = true,
                "confirm" => model.confirmations += 1,
                "act" if act_fires => {
                    model.spent.insert(user_ok.clone());
                    model.send_in_force = true;
                    sends += 1;
                }
                "act_plain" if plain_fires => {
                    model.spent.insert(user_ok.clone());
                    model.plain_in_force = true;
                    plain_sends += 1;
                }
                "reconsider" if model.bad => {
                    model.send_in_force = false;
                    model.plain_in_force = false;
                }
                "drop_asks" | "drop_unfound" if model.bad => {
                    new_withdrawals = model.withdraw_every("asks");
                }
                "drop_addr" if model.bad => new_withdrawals = model.withdraw_every("addr"),
                "drop_mail" if model.bad => new_withdrawals = model.withdraw_every("mail"),
                "drop_note" if model.bad => new_withdrawals = model.withdraw_every("note"),
                _ => {}
            }
            if event.starts_with("drop_") && model.bad {
                if new_withdrawals.is_empty() {
                    every_empty += 1;
                } else {
                    every_withdrawing += 1;
                    withdrawn_total += new_withdrawals.len();
                }
            }
            assert_eq!(
                withdraw_effects(&shown),
                new_withdrawals,
                "{at}: {event}: the withdrawals reported"
            );
            assert_eq!(
                withdrawals(&shown),
                model.withdrawn,
                "{at}: {event}: the withdrawal ledger"
            );
            // A value read from a withdrawn reading carries `withdrawn`, as does
            // one whose lineage holds the withdrawn mail: every reading of
            // `asks` is sampled under `observed(mail)`, so mail is in its
            // lineage (Withdrawal 0.1, "every current value whose lineage or
            // grounds hold it").
            if let Some(from) = &model.copied_from {
                let carries = caveats_of(&shown, "copied").contains(&"withdrawn".to_string());
                assert_eq!(
                    carries,
                    model.withdrawn.contains(from) || model.withdrawn.iter().any(|id| id == "mail"),
                    "{at}: {event}: copied read {from}: {:?}",
                    caveats_of(&shown, "copied")
                );
                if carries {
                    copied_withdrawn += 1;
                }
            }
            // Every event's save restores, to the same save and snapshot (D021).
            let save = game.save_json().unwrap();
            let restored = ReactiveSession::restore_json(SOURCE, &save)
                .unwrap_or_else(|error| panic!("{at}: restore after {event}: {error}"));
            assert_eq!(restored.save_json().unwrap(), save, "{at}: save round trip");
            assert_eq!(snapshot(&restored), shown, "{at}: restored snapshot");
            before = shown;
        }
    }
    eprintln!("{accepted} accepted events, {refused} refused ({spent_refusals} as reason_spent), {every_withdrawing} withdraw-every effects withdrawing {withdrawn_total} occurrences, {every_empty} withdrawing nothing, {sends} once sends, {plain_sends} plain sends, {copied_withdrawn} values carrying withdrawn, {round_trips} round trips");
    assert!(
        accepted > 5000
            && refused > 150
            && spent_refusals > 50
            && every_withdrawing > 350
            && withdrawn_total > 800
            && every_empty > 400
            && sends > 100
            && plain_sends > 80
            && copied_withdrawn > 1000
            && round_trips > 500,
        "coverage"
    );
}
