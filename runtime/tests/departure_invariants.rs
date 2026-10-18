// Windows 0.1 and Departure 0.1 under random events: a windowed history keeps
// its newest N records live and retires the oldest first, reporting it;
// numbering never reuses a name; at the end of every accepted event each
// retired record nothing pins departs, and a departed record is gone from
// the graph, the records and the observation order, with its archive entry
// drained by the host; what the session holds, plus what it archived, is
// every record it ever made; a refusal retires and departs nothing; and a
// restored session is the one that saved. Checked against the snapshot and
// the archive after every event, with saves and restores in place.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const SOURCE: &str = r#"
caveat flaky consequence material;
claim ok;
evidence e from "tool";
renewable e window 2;
evidence f from "operator";
evidence h from "sensor";
readings s from h window 3;
decisions d limit 8;
journal window 4;
state v_s = 0;
state v_e = 0;
state a = 0;
event see_e;
event see_f;
event new_e;
event read;
event decide;
event decide_e;
event reconsider;
event retract_s;
event retract_e;
event doubt_e;
event undoubt_e;
event later;
event set_vs;
event set_ve;
event forget;
event bump;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on see_e reveal e supports ok;
on see_f reveal f;
on new_e renew e;
on read sample s = a supports ok;
on decide when has_sample(s) commit d because enough using latest(s);
on decide_e when observed(e) commit d because enough using qualified(a, e);
on reconsider when committed(d) and has_sample(s) reopen d because latest(s);
on retract_s when has_sample(s) and observed(f) withdraw latest(s) because f;
on retract_e when observed(e) and observed(f) withdraw e because f;
on doubt_e when observed(e) qualify e with flaky;
on undoubt_e unqualify e with flaky;
on later when observed(e) qualify e with flaky after 0.2;
on set_vs when has_sample(s) set v_s = latest(s);
on set_ve when observed(e) set v_e = qualified(a, e);
on forget set v_s = 0;
on forget set v_e = 0;
on bump set a = a + 1;
bind view.count_s = history_count(s);
bind view.committed = if(committed(d), 1, 0);
"#;

const EVENTS: [(&str, &str); 19] = [
    ("see_e", "{}"),
    ("see_f", "{}"),
    ("new_e", "{}"),
    ("read", "{}"),
    ("read", "{}"),
    ("decide", "{}"),
    ("decide_e", "{}"),
    ("reconsider", "{}"),
    ("retract_s", "{}"),
    ("retract_e", "{}"),
    ("doubt_e", "{}"),
    ("undoubt_e", "{}"),
    ("later", "{}"),
    ("set_vs", "{}"),
    ("set_ve", "{}"),
    ("forget", "{}"),
    ("bump", "{}"),
    ("tick", r#"{"dt": 0.1}"#),
    ("tick", r#"{"dt": 0.05}"#),
];

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn view(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.view()).unwrap()
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

/// The number of a record name: `s@4` and `e@4` are 4, and a renewable
/// evidence's bare name is its first occurrence.
fn number(record: &str) -> u64 {
    record
        .split_once('@')
        .map_or(1, |(_, number)| number.parse().unwrap())
}

/// A reading is `s@N` from 1; an occurrence is `e` and then `e@N` from 2.
fn name(history: &str, number: u64) -> String {
    if history == "e" && number == 1 {
        history.to_string()
    } else {
        format!("{history}@{number}")
    }
}

/// Held records of a history: live and retired, by name.
fn held(shown: &Value, history: &str) -> Vec<String> {
    match history {
        "s" => shown["reading_streams"]["s"]["occurrences"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["id"].as_str().unwrap().to_string())
            .collect(),
        "e" => strings(&shown["renewals"]["e"]["occurrences"]),
        _ => unreachable!(),
    }
}

/// Everything in the session that pins a retired record, by name.
fn pins(shown: &Value) -> BTreeSet<String> {
    let mut pinned = BTreeSet::new();
    for grounds in shown["value_grounds"]
        .as_object()
        .map(|map| map.values().collect::<Vec<_>>())
        .unwrap_or_default()
    {
        pinned.extend(strings(&grounds["evidence"]));
    }
    let commitments = shown["commitments"].as_array().cloned().unwrap_or_default();
    for (_, series) in shown["decision_series"].as_object().unwrap() {
        if let Some(current) = series["current"].as_str() {
            let in_force = commitments
                .iter()
                .any(|item| item["action"] == current && item["open"] == false);
            if in_force {
                pinned.extend(strings(&shown["commitment_grounds"][current]["evidence"]));
            }
        }
    }
    for entry in shown["decision_journal"].as_array().unwrap() {
        if entry["retired"] == true {
            continue;
        }
        pinned.extend(strings(&entry["because"]));
        if let Some(grant) = entry["permitted_by"].as_str() {
            pinned.insert(grant.to_string());
        }
    }
    for scheduled in shown["scheduled_qualifications"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        pinned.insert(scheduled["evidence"].as_str().unwrap().to_string());
    }
    pinned.insert(held(shown, "e").last().unwrap().clone());
    for record in shown["commitment_permissions"]
        .as_object()
        .map(|map| map.values().collect::<Vec<_>>())
        .unwrap_or_default()
    {
        pinned.insert(record["grant"].as_str().unwrap().to_string());
    }
    for withdrawal in shown["withdrawals"].as_array().cloned().unwrap_or_default() {
        pinned.insert(withdrawal["because"].as_str().unwrap().to_string());
    }
    pinned
}

struct Tally {
    newest: BTreeMap<&'static str, u64>,
    archived: BTreeSet<String>,
}

fn check(shown: &Value, tally: &Tally, at: &str) {
    let retired = shown["retired"]
        .as_object()
        .map(|map| map.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();
    let pinned = pins(shown);
    for record in &retired {
        // A renewable evidence's declared first occurrence never departs.
        assert!(
            pinned.contains(record) || record == "e",
            "{at}: retired {record} is pinned by nothing yet has not departed; pins {pinned:?}"
        );
    }
    let mut departed = BTreeSet::new();
    for (history, window) in [("s", 3), ("e", 2)] {
        let held = held(shown, history);
        let newest = tally.newest[history];
        let live = held
            .iter()
            .filter(|record| !retired.contains(*record))
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            live.len() <= window,
            "{at}: {history} has {} live records over its window {window}: {held:?}",
            live.len()
        );
        if newest > 0 {
            let expected = ((newest + 1).saturating_sub(window.min(newest as usize) as u64)
                ..=newest)
                .map(|number| name(history, number))
                .collect::<Vec<_>>();
            assert_eq!(
                live, expected,
                "{at}: {history}'s live records are not its newest {window}: retired {retired:?}"
            );
        }
        for record in &held {
            assert!(
                number(record) <= newest,
                "{at}: {history} holds {record} beyond its newest {newest}"
            );
        }
        let held_set = held.iter().cloned().collect::<BTreeSet<_>>();
        for number in 1..=newest {
            let record = name(history, number);
            if !held_set.contains(&record) {
                departed.insert(record);
            }
        }
    }
    // What the session holds plus what it archived is every record it made.
    let archived_records = tally
        .archived
        .iter()
        .filter(|record| {
            record.starts_with("s@") || record.starts_with("e@") || *record == "s" || *record == "e"
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        archived_records, departed,
        "{at}: the archive and the departed records disagree"
    );
    // A departed record is gone from everything live.
    for edge in shown["relations"].as_array().unwrap() {
        for end in ["from", "to"] {
            let node = edge[end].as_str().unwrap();
            assert!(
                !departed.contains(node),
                "{at}: a relation still names departed {node}: {edge}"
            );
        }
    }
    for withdrawal in shown["withdrawals"].as_array().cloned().unwrap_or_default() {
        assert!(
            !departed.contains(withdrawal["evidence"].as_str().unwrap()),
            "{at}: a withdrawal still names a departed record: {withdrawal}"
        );
    }
    for scheduled in shown["scheduled_qualifications"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        assert!(
            !departed.contains(scheduled["evidence"].as_str().unwrap()),
            "{at}: a scheduled qualification waits on a departed record: {scheduled}"
        );
    }
    // The journal keeps at most its window live.
    let journal = shown["decision_journal"].as_array().unwrap();
    let live_entries = journal
        .iter()
        .filter(|entry| entry["retired"] != true)
        .count();
    assert!(
        live_entries <= 4,
        "{at}: {live_entries} live journal entries over the window"
    );
    assert_eq!(
        shown["bindings"]["view"]["count_s"].as_f64().unwrap() as usize,
        held(shown, "s")
            .iter()
            .filter(|record| !retired.contains(*record))
            .count(),
        "{at}: history_count(s)"
    );
}

#[test]
fn the_departure_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn windows_retire_and_departure_takes_exactly_what_nothing_pins() {
    let mut seed: u64 = 0x1f83_d9ab_fb41_bd6b;
    let (mut accepted, mut departures, mut retirements, mut refusals) = (0, 0, 0, 0);
    for run in 0..200 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        let mut tally = Tally {
            newest: BTreeMap::from([("s", 0), ("e", 1)]),
            archived: BTreeSet::new(),
        };
        let mut before = snapshot(&game);
        check(&before, &tally, &format!("run {run} start"));
        for step in 0..70 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                trail.push(("save", ""));
                let saved = game.save_json().unwrap();
                let restored =
                    ReactiveSession::restore_json(SOURCE, &saved).unwrap_or_else(|error| {
                        panic!("run {run} step {step} restore: {error}: {trail:?}")
                    });
                assert_eq!(
                    snapshot(&restored),
                    before,
                    "run {run} step {step} snapshot after restore: {trail:?}"
                );
                assert_eq!(
                    restored.save_json().unwrap(),
                    saved,
                    "run {run} step {step} save after restore: {trail:?}"
                );
                assert_eq!(
                    restored.undrained(),
                    0,
                    "run {run} step {step}: a restored session has a pending archive"
                );
                game = restored;
                continue;
            }
            let (event, payload) = EVENTS[pick];
            trail.push((event, payload));
            let save_before = game.save_json().unwrap();
            let view_before = view(&game);
            let undrained_before = game.undrained();
            let outcome =
                serde_json::to_value(game.dispatch_outcome_json(event, payload).unwrap_or_else(
                    |fatal| panic!("run {run} step {step} fatal {}: {trail:?}", fatal.message),
                ))
                .unwrap();
            let at = format!("run {run} step {step} after {event}: {trail:?}");
            if outcome["outcome"] != "accepted" {
                refusals += 1;
                assert_eq!(
                    game.save_json().unwrap(),
                    save_before,
                    "{at}: a refusal changed the save"
                );
                assert_eq!(
                    snapshot(&game),
                    before,
                    "{at}: a refusal changed the snapshot"
                );
                assert_eq!(view(&game), view_before, "{at}: a refusal changed the view");
                assert_eq!(
                    game.undrained(),
                    undrained_before,
                    "{at}: a refusal archived something"
                );
                continue;
            }
            accepted += 1;
            let shown = snapshot(&game);
            // Numbering and retirement, from the effects.
            let effects = shown["effects"].as_array().unwrap().clone();
            for (index, effect) in effects.iter().enumerate() {
                match effect["kind"].as_str().unwrap() {
                    "sample" => {
                        let id = effect["id"].as_str().unwrap();
                        tally.newest.insert("s", tally.newest["s"] + 1);
                        assert_eq!(
                            id,
                            name("s", tally.newest["s"]),
                            "{at}: a reading's number does not continue"
                        );
                        let live_before = held(&before, "s")
                            .iter()
                            .filter(|record| before["retired"].get(*record).is_none())
                            .cloned()
                            .collect::<Vec<_>>();
                        if live_before.len() == 3 {
                            retirements += 1;
                            let previous = effects.get(index.wrapping_sub(1));
                            assert_eq!(previous.map(|effect| (effect["kind"].as_str().unwrap(), effect["record"].as_str().unwrap())), Some(("retire", live_before[0].as_str())), "{at}: a full window must retire its oldest live record first: {effects:?}");
                        }
                    }
                    "renew" => {
                        let occurrence = effect["occurrence"].as_str().unwrap();
                        tally.newest.insert("e", tally.newest["e"] + 1);
                        assert_eq!(
                            occurrence,
                            name("e", tally.newest["e"]),
                            "{at}: an occurrence's number does not continue"
                        );
                        let live_before = held(&before, "e")
                            .iter()
                            .filter(|record| before["retired"].get(*record).is_none())
                            .cloned()
                            .collect::<Vec<_>>();
                        if live_before.len() == 2 {
                            retirements += 1;
                            let previous = effects.get(index.wrapping_sub(1));
                            assert_eq!(previous.map(|effect| (effect["kind"].as_str().unwrap(), effect["record"].as_str().unwrap())), Some(("retire", live_before[0].as_str())), "{at}: a full window must retire its oldest live occurrence first: {effects:?}");
                        }
                    }
                    "depart" => {
                        departures += 1;
                        // Reported after every other effect.
                        assert!(
                            effects[index..]
                                .iter()
                                .all(|later| later["kind"] == "depart"),
                            "{at}: a departure reported before another effect: {effects:?}"
                        );
                    }
                    _ => {}
                }
            }
            // Drain, and keep every archived record.
            let drained = serde_json::to_value(game.drain_archive()).unwrap();
            let departed_now = effects
                .iter()
                .filter(|effect| effect["kind"] == "depart")
                .map(|effect| effect["record"].as_str().unwrap().to_string())
                .collect::<BTreeSet<_>>();
            let archived_now = drained
                .as_array()
                .unwrap()
                .iter()
                .filter(|item| item.get("kind") != Some(&Value::String("provenance".into())))
                .map(|item| item["record"].as_str().unwrap().to_string())
                .collect::<BTreeSet<_>>();
            assert_eq!(
                archived_now, departed_now,
                "{at}: the drained records are not the departures reported"
            );
            for record in &archived_now {
                assert!(
                    tally.archived.insert(record.clone()),
                    "{at}: {record} archived twice"
                );
            }
            assert_eq!(game.undrained(), 0, "{at}");
            check(&shown, &tally, &at);
            before = shown;
        }
    }
    eprintln!("{accepted} accepted events, {refusals} refusals, {retirements} retirements, {departures} departures");
    assert!(
        departures > 800 && retirements > 1000,
        "the alphabet barely reached departure: {retirements} / {departures}"
    );
}
