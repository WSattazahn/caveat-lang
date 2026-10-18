// State Caveats 0.1 under random events: `reopen D because caveated(S, C)`
// selects every observed evidence identity in S's grounds that currently
// carries C, directly, through another caveat or through a claim it bears
// on, ordered by first observation; an empty selection is refused as
// evaluation/empty_caveated_selection and rolls the event back; one
// execution appends one `reopened` entry naming the newly added witnesses,
// and a selection of only recorded witnesses appends nothing; identities are
// concrete, never re-resolved through a renewable alias; and `has_caveat`
// reads the grounds, not the control lineage, so a guard's stale read does
// not make it true. Checked against the snapshot after every event, with
// saves and restores in place.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const SOURCE: &str = r#"
caveat stale consequence material;
caveat old consequence material;
caveat doubtful consequence low;
old qualifies doubtful;
claim fresh;
evidence a from "sensor a";
renewable a limit 5;
evidence b from "sensor b";
evidence c from "sensor c";
evidence guard_e from "operator";
state s = 0;
state ctl = 0;
decisions plan limit 8;
event see_a;
event see_b;
event see_c;
event see_guard;
event new_a;
event read_ab;
event read_abc;
event read_extra;
event flag_a;
event flag_b;
event flag_c_doubtful;
event stale_a;
event unstale_a;
event flag_guard;
event decide;
event reconsider;
event reconsider_fresh;
event control;
on see_a reveal a supports fresh;
on see_b reveal b supports fresh;
on see_c reveal c supports fresh;
on see_guard reveal guard_e;
on new_a renew a;
on read_ab when observed(a) and observed(b) set s = qualified(1, a) + qualified(1, b);
on read_abc when observed(a) and observed(b) and observed(c) set s = qualified(1, a) + qualified(1, b) + qualified(1, c);
on read_extra when observed(a) set s = qualified(1, a, old);
on flag_a when observed(a) qualify a with old;
on flag_b when observed(b) qualify b with old;
on flag_c_doubtful when observed(c) qualify c with doubtful;
on stale_a when observed(a) qualify a with stale;
on unstale_a unqualify a with stale;
on flag_guard when observed(guard_e) qualify guard_e with old;
on decide when observed(a) commit plan because enough using s;
on reconsider when committed(plan) reopen plan because caveated(s, old);
on reconsider_fresh when committed(plan) and not reopened(plan) reopen plan because caveated(s, old);
on control when observed(guard_e) set ctl = s;
bind view.stale = if(has_caveat(s, old), 1, 0);
bind view.ctl_stale = if(has_caveat(ctl, old), 1, 0);
"#;

const EVENTS: [&str; 22] = [
    "see_a",
    "see_b",
    "see_c",
    "see_guard",
    "new_a",
    "read_ab",
    "read_abc",
    "read_extra",
    "flag_a",
    "flag_b",
    "flag_c_doubtful",
    "stale_a",
    "unstale_a",
    "flag_guard",
    "decide",
    "reconsider",
    "reconsider_fresh",
    "control",
    "decide",
    "reconsider",
    "flag_a",
    "read_abc",
];

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn strings(list: &Value) -> Vec<String> {
    list.as_array()
        .map(|items| {
            items
                .iter()
                .map(|item| item.as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Whether `occurrence` currently carries `old`: a live `old qualifies`
/// relation, or a live relation of a caveat `old` qualifies (`doubtful`).
fn carries_stale(shown: &Value, occurrence: &str) -> bool {
    let relations = shown["relations"].as_array().unwrap();
    let live = |from: &str, to: &str| {
        let count = relations
            .iter()
            .filter(|edge| {
                edge["from"] == from && edge["to"] == to && edge["relation"] == "qualifies"
            })
            .count();
        let taken = shown["taken_back"].as_array().map_or(0, |list| {
            list.iter()
                .filter(|record| record["caveat"] == from && record["evidence"] == to)
                .count()
        });
        count > taken
    };
    // old qualifies doubtful, so a doubtful occurrence carries old through it.
    live("old", occurrence) || (live("doubtful", occurrence) && live("old", "doubtful"))
}

/// Where each occurrence was first observed: the observation ledger when the
/// session keeps one, else the place of its first `supports` or `opposes`
/// relation, as the decision journal orders its `because`.
fn observation_rank(shown: &Value) -> BTreeMap<String, usize> {
    let ledger = strings(&shown["observations"]);
    if !ledger.is_empty() {
        return ledger
            .into_iter()
            .enumerate()
            .map(|(index, name)| (name, index))
            .collect();
    }
    let mut rank = BTreeMap::new();
    for (index, edge) in shown["relations"].as_array().unwrap().iter().enumerate() {
        if edge["relation"] == "supports" || edge["relation"] == "opposes" {
            rank.entry(edge["from"].as_str().unwrap().to_string())
                .or_insert(index);
        }
    }
    rank
}

#[test]
fn the_selector_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn the_caveated_selector_picks_exactly_the_stale_grounds_in_observation_order() {
    let mut seed: u64 = 0xf1bb_cdcb_7d8d_6f5e;
    let (mut accepted, mut selections, mut empty, mut idempotent, mut through_claim_or_caveat) =
        (0, 0, 0, 0, 0);
    for run in 0..300 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        for step in 0..50 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                trail.push("save");
                let saved = game.save_json().unwrap();
                game = ReactiveSession::restore_json(SOURCE, &saved).unwrap_or_else(|error| {
                    panic!("run {run} step {step} restore: {error}: {trail:?}")
                });
                assert_eq!(
                    snapshot(&game),
                    before,
                    "run {run} step {step} after restore: {trail:?}"
                );
                continue;
            }
            let event = EVENTS[pick];
            trail.push(event);
            let save_before = game.save_json().unwrap();
            let outcome =
                serde_json::to_value(game.dispatch_outcome_json(event, "{}").unwrap_or_else(
                    |fatal| panic!("run {run} step {step} fatal {}: {trail:?}", fatal.message),
                ))
                .unwrap();
            let at = format!("run {run} step {step} after {event}: {trail:?}");
            if event == "reconsider" || event == "reconsider_fresh" {
                let current = before["decision_series"]["plan"]["current"].as_str();
                let open = current.is_some_and(|id| {
                    before["commitments"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|item| item["action"] == id && item["open"] == true)
                });
                let fires = current.is_some() && (event == "reconsider" || !open);
                if fires {
                    let current = current.unwrap();
                    // The selection the spec describes, from the state before.
                    let grounds = strings(&before["value_grounds"]["s"]["evidence"]);
                    let rank = observation_rank(&before);
                    let mut selected = grounds
                        .iter()
                        .filter(|occurrence| carries_stale(&before, occurrence))
                        .cloned()
                        .collect::<Vec<_>>();
                    selected.sort_by_key(|name| rank.get(name).copied().unwrap_or(usize::MAX));
                    let recorded = before["commitments"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|item| item["action"] == current)
                        .map(|item| strings(&item["reopened_by"]))
                        .unwrap_or_default();
                    if selected.is_empty() {
                        empty += 1;
                        assert_eq!(
                            (outcome["origin"].as_str(), outcome["code"].as_str()),
                            (Some("evaluation"), Some("empty_caveated_selection")),
                            "{at}: grounds {grounds:?}: {outcome}"
                        );
                        assert_eq!(
                            game.save_json().unwrap(),
                            save_before,
                            "{at}: an empty selection changed the save"
                        );
                        continue;
                    }
                    assert_eq!(
                        outcome["outcome"], "accepted",
                        "{at}: selected {selected:?}: {outcome}"
                    );
                    let shown = snapshot(&game);
                    let new_witnesses = selected
                        .iter()
                        .filter(|name| !recorded.contains(name))
                        .cloned()
                        .collect::<Vec<_>>();
                    let added = shown["decision_journal"].as_array().unwrap().len()
                        - before["decision_journal"].as_array().unwrap().len();
                    if new_witnesses.is_empty() {
                        idempotent += 1;
                        assert_eq!(
                            added, 0,
                            "{at}: a selection of recorded witnesses appended an entry"
                        );
                    } else {
                        selections += 1;
                        assert_eq!(added, 1, "{at}: one execution appends one entry");
                        let entry = shown["decision_journal"]
                            .as_array()
                            .unwrap()
                            .last()
                            .unwrap();
                        assert_eq!(entry["change"], "reopened", "{at}");
                        assert_eq!(entry["commitment"], current, "{at}");
                        assert_eq!(
                            strings(&entry["because"]),
                            new_witnesses,
                            "{at}: the entry names the new witnesses in observation order: {entry}"
                        );
                        if selected.iter().any(|name| name == "c") {
                            through_claim_or_caveat += 1;
                        }
                    }
                    // Every selected witness is now recorded, concretely.
                    let now_recorded = shown["commitments"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|item| item["action"] == current)
                        .map(|item| strings(&item["reopened_by"]))
                        .unwrap_or_default();
                    for name in &selected {
                        assert!(
                            now_recorded.contains(name),
                            "{at}: {name} selected but not recorded: {now_recorded:?}"
                        );
                    }
                    let expected = recorded
                        .iter()
                        .chain(new_witnesses.iter())
                        .cloned()
                        .collect::<BTreeSet<_>>();
                    assert_eq!(
                        now_recorded.iter().cloned().collect::<BTreeSet<_>>(),
                        expected,
                        "{at}: recorded witnesses"
                    );
                }
            }
            if outcome["outcome"] != "accepted" {
                continue;
            }
            accepted += 1;
            let shown = snapshot(&game);
            // has_caveat reads the grounds; ctl's guard read of guard_e is control.
            let grounds_stale =
                strings(&shown["value_grounds"]["s"]["caveats"]).contains(&"old".to_string());
            assert_eq!(
                shown["bindings"]["view"]["stale"].as_f64(),
                Some(if grounds_stale { 1.0 } else { 0.0 }),
                "{at}"
            );
            let ctl_grounds = &shown["value_grounds"]["ctl"];
            assert!(
                !strings(&ctl_grounds["evidence"]).contains(&"guard_e".to_string()),
                "{at}: a guard read entered the grounds: {ctl_grounds}"
            );
            let ctl_stale = strings(&ctl_grounds["caveats"]).contains(&"old".to_string());
            assert_eq!(
                shown["bindings"]["view"]["ctl_stale"].as_f64(),
                Some(if ctl_stale { 1.0 } else { 0.0 }),
                "{at}"
            );
            before = shown;
        }
    }
    eprintln!("{accepted} accepted events; {selections} selections, {empty} empty, {idempotent} idempotent, {through_claim_or_caveat} with a witness through another caveat");
    assert!(
        selections > 100 && empty > 200 && idempotent > 50 && through_claim_or_caveat > 20,
        "{selections} / {empty} / {idempotent} / {through_claim_or_caveat}"
    );
}
