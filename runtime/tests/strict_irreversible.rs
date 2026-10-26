//! `decisions NAME limit 1 irreversible strict`: an irreversible commit that
//! also names the caveats its control carries, not only its grounds. See
//! spec/caveat-strict-irreversible-0.1.md.
use caveat_runtime::web::WebReactiveSession;
use serde_json::Value;

/// The why-demo's refund and payout. The customer's email only counts toward
/// the guard, so it is control: the payout's grounds never carry
/// `self_reported`.
fn refund(payout: &str, retaining: &str) -> String {
    format!(
        r#"
claim refund_warranted;
evidence order_record from "the order system";
evidence customer_email from "the customer's own email";
evidence approval from "a supervisor's approval";
caveat self_reported consequence material;
caveat rushed consequence high;
caveat one_click consequence low;
self_reported qualifies customer_email;
rushed qualifies approval;
one_click qualifies order_record;
readings approvals from approval limit 4;
decisions {payout};
state support = 0 min 0 max 10;
state amount = 0 min 0 max 200;
state low_support = 0 min 0 max 10;
event read_order; event read_email; event approve; event pay; event pay_low; event pay_permitted;
on read_order reveal order_record supports refund_warranted;
on read_order set amount = amount + qualified(40, order_record);
on read_order set low_support = low_support + qualified(3, order_record);
on read_email reveal customer_email supports refund_warranted;
on read_email set support = support + qualified(3, customer_email);
on approve sample approvals = 1 supports refund_warranted;
on pay when support >= 3 commit payout because enough using 40{retaining};
on pay_low when low_support >= 3 commit payout because enough using 40;
on pay_permitted commit payout because enough using 40 permitted by latest(approvals);
"#
    )
}

const PLAIN: &str = "payout limit 1 irreversible";
const STRICT: &str = "payout limit 1 irreversible strict";

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn session(source: &str) -> WebReactiveSession {
    WebReactiveSession::new(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn accept(game: &mut WebReactiveSession, event: &str) {
    let result = json(&game.dispatch_outcome(event, "{}").unwrap());
    assert_eq!(result["outcome"], "accepted", "{event}: {result}");
}

/// Refused with `caveated_irreversible`, and nothing changed.
fn refuse(game: &mut WebReactiveSession, event: &str) -> String {
    let before = (game.snapshot(), game.save().unwrap());
    let result = json(&game.dispatch_outcome(event, "{}").unwrap());
    assert_eq!(result["outcome"], "rejected", "{event}: {result}");
    assert_eq!(result["origin"], "evaluation", "{event}: {result}");
    assert_eq!(result["code"], "caveated_irreversible", "{event}: {result}");
    assert_eq!((game.snapshot(), game.save().unwrap()), before, "{event}");
    result["message"].as_str().unwrap().to_string()
}

#[test]
fn without_strict_a_caveat_read_only_by_the_guard_is_not_checked() {
    // Foreclosure 0.1: the guard is not grounds and is not read.
    let mut game = session(&refund(PLAIN, ""));
    accept(&mut game, "read_email");
    accept(&mut game, "pay");
}

#[test]
fn strict_reads_the_caveats_of_the_guard() {
    let mut game = session(&refund(STRICT, ""));
    accept(&mut game, "read_email");
    let message = refuse(&mut game, "pay");
    assert!(
        message.ends_with(
            "irreversible payout rests on self_reported (material), which its commit does not retain"
        ),
        "{message}"
    );
    let snapshot = json(&game.snapshot());
    assert!(snapshot["decision_series"]["payout"]["revisions"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn retaining_the_caveat_commits_with_it_on_purpose() {
    let mut game = session(&refund(STRICT, " retaining self_reported"));
    accept(&mut game, "read_email");
    accept(&mut game, "pay");
    let snapshot = json(&game.snapshot());
    assert_eq!(
        snapshot["commitment_grounds"]["payout@1"]["caveats"],
        serde_json::json!(["self_reported"])
    );
}

#[test]
fn a_low_caveat_in_the_guard_is_not_read() {
    let mut game = session(&refund(STRICT, ""));
    accept(&mut game, "read_order");
    accept(&mut game, "pay_low");
}

#[test]
fn strict_reads_the_caveats_of_a_permission_grant() {
    let mut plain = session(&refund(PLAIN, ""));
    accept(&mut plain, "approve");
    accept(&mut plain, "pay_permitted");

    let mut game = session(&refund(STRICT, ""));
    accept(&mut game, "approve");
    let message = refuse(&mut game, "pay_permitted");
    assert!(
        message.ends_with(
            "irreversible payout rests on rushed (high), which its commit does not retain"
        ),
        "{message}"
    );
}

#[test]
fn strict_adds_no_snapshot_field() {
    // The snapshot's `irreversible` lists the series as before; `strict` is
    // read from the source.
    let plain = json(&session(&refund(PLAIN, "")).snapshot());
    let strict = json(&session(&refund(STRICT, "")).snapshot());
    assert_eq!(plain["irreversible"], strict["irreversible"]);
    assert_eq!(strict["irreversible"], serde_json::json!({"payout": []}));
}

#[test]
fn strict_keeps_its_foreclosures_and_triggers() {
    let source = refund(
        "payout limit 1 irreversible strict forecloses undo reopened by approvals; decisions undo limit 4",
        " retaining self_reported",
    );
    let mut game = session(&source);
    accept(&mut game, "read_email");
    accept(&mut game, "pay");
    let snapshot = json(&game.snapshot());
    assert_eq!(
        snapshot["irreversible"],
        serde_json::json!({"payout": ["undo"]})
    );
}

#[test]
fn a_save_restores_and_strict_still_applies() {
    let source = refund(STRICT, "");
    let mut game = session(&source);
    accept(&mut game, "read_email");
    let save = game.save().unwrap();
    let mut restored = WebReactiveSession::restore(&source, &save).unwrap();
    refuse(&mut restored, "pay");
}

#[test]
fn load_rules() {
    let error = |declaration: &str| {
        WebReactiveSession::new(&refund(declaration, ""))
            .err()
            .unwrap_or_else(|| panic!("{declaration} loaded"))
            .to_string()
    };
    assert!(
        error("payout limit 4 strict").contains("strict follows irreversible"),
        "{}",
        error("payout limit 4 strict")
    );
    assert!(
        error("payout limit 1 strict irreversible").contains("strict follows irreversible"),
        "{}",
        error("payout limit 1 strict irreversible")
    );
    assert!(
        error("payout limit 1 irreversible forecloses undo strict")
            .contains("forecloses takes decision names"),
        "{}",
        error("payout limit 1 irreversible forecloses undo strict")
    );
    assert!(error("payout limit 4 irreversible strict").contains("holds one revision"));
}

#[test]
fn the_word_reserves_nothing() {
    // `strict` is read only straight after `irreversible`, so it stays a
    // legal name, also for a stream named in `reopened by`.
    let source = r#"
claim ok;
evidence strict from "a check";
readings checks from strict limit 4;
decisions go limit 1 irreversible strict reopened by checks;
decisions other limit 4 reopened by checks;
state strict_count = 0 min 0 max 4;
event look;
on look reveal strict supports ok;
on look commit go because enough using qualified(1, strict);
"#;
    let mut game = session(source);
    accept(&mut game, "look");
}
