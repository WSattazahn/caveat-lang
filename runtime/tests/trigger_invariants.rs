// Reopening Triggers 0.1, Changed Triggers 0.1 and Neutral Sample 0.1 under
// random events: a series reopens exactly when a revision is in force and a
// reading recorded by the event matches one of its triggers, once per series,
// with the journal naming the first matching reading in rule order; a
// filtered trigger matches only a reading with that stance, so never a
// neutral one; a `changed` trigger matches only a first reading or one whose
// value differs from the stream's previous reading, retired, departed or
// withdrawn as that may be; a reading that matches nothing leaves every
// decision as it was; a neutral reading has no stance in its record or in
// the graph and joins the observation ledger in order; and `latest` reads
// the newest reading whatever its stance. Checked after every event, with
// saves and restores in place and the archive drained.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeMap;

const SOURCE: &str = r#"
claim ok;
caveat flaky consequence material;
evidence h from "sensor";
evidence g from "probe";
flaky qualifies g;
evidence f from "operator";
readings s from h window 3;
readings t from g limit 40;
decisions plain limit 24 reopened by s, t;
decisions chg limit 24 reopened by changed s, changed t;
decisions sup limit 24 reopened by s supporting ok, t supporting ok;
decisions chg_sup limit 24 reopened by changed s supporting ok;
decisions opp limit 24 reopened by t opposing ok;
decisions mixed limit 24 reopened by changed s, t opposing ok;
state a = 0;
event read_s v min 0 max 2;
event read_s_sup v min 0 max 2;
event read_t v min 0 max 2;
event read_t_sup v min 0 max 2;
event read_t_opp v min 0 max 2;
event read_both v min 0 max 2;
event read_s_guarded v min 0 max 2;
event toggle;
event see_f;
event retract_s;
event retract_t;
event decide_plain;
event decide_chg;
event decide_sup;
event decide_chg_sup;
event decide_opp;
event decide_mixed;
on read_s sample s = v;
on read_s_sup sample s = v supports ok;
on read_t sample t = v;
on read_t_sup sample t = v supports ok;
on read_t_opp sample t = v opposes ok;
on read_both sample s = v supports ok;
on read_both sample t = v;
on read_s_guarded when a == 1 sample s = v;
on toggle set a = 1 - a;
on see_f reveal f;
on retract_s when has_sample(s) and observed(f) withdraw latest(s) because f;
on retract_t when has_sample(t) and observed(f) withdraw latest(t) because f;
on decide_plain when has_sample(s) and has_sample(t) and (not committed(plain) or reopened(plain)) commit plain because enough citing latest(s), latest(t);
on decide_chg when has_sample(s) and has_sample(t) and (not committed(chg) or reopened(chg)) commit chg because enough citing latest(s), latest(t);
on decide_sup when has_sample(s) and has_sample(t) and (not committed(sup) or reopened(sup)) commit sup because enough citing latest(s), latest(t);
on decide_chg_sup when has_sample(s) and (not committed(chg_sup) or reopened(chg_sup)) commit chg_sup because enough citing latest(s);
on decide_opp when has_sample(t) and (not committed(opp) or reopened(opp)) commit opp because enough citing latest(t);
on decide_mixed when has_sample(s) and has_sample(t) and (not committed(mixed) or reopened(mixed)) commit mixed because enough citing latest(s), latest(t);
bind view.latest_s = latest(s) when has_sample(s);
bind view.latest_t = latest(t) when has_sample(t);
"#;

const EVENTS: [(&str, &str); 24] = [
    ("read_s", r#"{"v": 0}"#),
    ("read_s", r#"{"v": 1}"#),
    ("read_s_sup", r#"{"v": 0}"#),
    ("read_s_sup", r#"{"v": 1}"#),
    ("read_t", r#"{"v": 0}"#),
    ("read_t", r#"{"v": 1}"#),
    ("read_t_sup", r#"{"v": 1}"#),
    ("read_t_opp", r#"{"v": 0}"#),
    ("read_both", r#"{"v": 0}"#),
    ("read_both", r#"{"v": 1}"#),
    ("read_s_guarded", r#"{"v": 2}"#),
    ("toggle", "{}"),
    ("see_f", "{}"),
    ("retract_s", "{}"),
    ("retract_t", "{}"),
    ("decide_plain", "{}"),
    ("decide_chg", "{}"),
    ("decide_sup", "{}"),
    ("decide_chg_sup", "{}"),
    ("decide_opp", "{}"),
    ("decide_mixed", "{}"),
    ("decide_plain", "{}"),
    ("decide_chg", "{}"),
    ("decide_mixed", "{}"),
];

/// A trigger from the source: (stream, changed, stance filter).
type Trigger = (&'static str, bool, Option<&'static str>);

/// Each series' triggers, from the source.
const SERIES: [(&str, &[Trigger]); 6] = [
    ("plain", &[("s", false, None), ("t", false, None)]),
    ("chg", &[("s", true, None), ("t", true, None)]),
    (
        "sup",
        &[
            ("s", false, Some("supports")),
            ("t", false, Some("supports")),
        ],
    ),
    ("chg_sup", &[("s", true, Some("supports"))]),
    ("opp", &[("t", false, Some("opposes"))]),
    ("mixed", &[("s", true, None), ("t", false, Some("opposes"))]),
];

/// Every reading a stream ever took, in order: (id, value, stance).
type Model = BTreeMap<String, Vec<(String, f64, Option<String>)>>;

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

fn in_force(shown: &Value, series: &str) -> Option<String> {
    let current = shown["decision_series"][series]["current"].as_str()?;
    let open = shown["commitments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["action"] == current)
        .map(|item| item["open"] == true)
        .unwrap_or(true);
    (!open).then(|| current.to_string())
}

/// The readings in the snapshot, by stream, in occurrence order.
fn readings(shown: &Value) -> Model {
    let mut model = Model::new();
    if let Some(streams) = shown["reading_streams"].as_object() {
        for (stream, record) in streams {
            let entries = record["occurrences"]
                .as_array()
                .map(|list| {
                    list.iter()
                        .map(|reading| {
                            let stance = reading.get("relation").map(|relation| {
                                assert!(
                                    reading.get("claim").is_some(),
                                    "a reading with a relation and no claim: {reading}"
                                );
                                relation.as_str().unwrap().to_string()
                            });
                            assert!(
                                stance.is_some() || reading.get("claim").is_none(),
                                "a reading with a claim and no relation: {reading}"
                            );
                            (
                                reading["id"].as_str().unwrap().to_string(),
                                reading["value"].as_f64().unwrap(),
                                stance,
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            model.insert(stream.clone(), entries);
        }
    }
    model
}

#[test]
fn the_trigger_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn triggers_reopen_exactly_the_revisions_in_force_that_a_matching_reading_reaches() {
    let mut seed: u64 = 0x7c3a_9f1b_2d58_e604;
    let (mut accepted, mut reopenings, mut changed_skips, mut neutral_skips, mut neutral_readings) =
        (0, 0, 0, 0, 0);
    for run in 0..150 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        // Every reading ever taken, kept past retirement, departure and withdrawal.
        let mut history = Model::new();
        for step in 0..60 {
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
            let (event, payload) = EVENTS[pick];
            trail.push(event);
            let save_before = game.save_json().unwrap();
            let outcome =
                serde_json::to_value(game.dispatch_outcome_json(event, payload).unwrap_or_else(
                    |fatal| panic!("run {run} step {step} fatal {}: {trail:?}", fatal.message),
                ))
                .unwrap();
            let at = format!("run {run} step {step} after {event} {payload}: {trail:?}");
            if outcome["outcome"] != "accepted" {
                assert_eq!(
                    game.save_json().unwrap(),
                    save_before,
                    "{at}: a refused event changed the save: {outcome}"
                );
                continue;
            }
            accepted += 1;
            let shown = snapshot(&game);
            game.drain_archive();

            // The readings this event recorded, in rule order (s before t in
            // read_both, as its rules are written).
            let old = readings(&before);
            let new = readings(&shown);
            let mut taken: Vec<(String, String, f64, Option<String>)> = Vec::new();
            for (stream, entries) in &new {
                let seen = old.get(stream).cloned().unwrap_or_default();
                for (id, value, stance) in entries {
                    if !seen.iter().any(|(known, _, _)| known == id) {
                        taken.push((stream.clone(), id.clone(), *value, stance.clone()));
                    }
                }
            }
            taken.sort_by(|left, right| left.0.cmp(&right.0));
            assert!(
                taken.len() <= 2 && (taken.len() < 2 || event == "read_both"),
                "{at}: {taken:?}"
            );
            for (stream, id, value, stance) in &taken {
                // The reading's stance is the rule's.
                let expected = match event {
                    "read_s_sup" | "read_t_sup" => Some("supports"),
                    "read_t_opp" => Some("opposes"),
                    "read_both" if stream == "s" => Some("supports"),
                    _ => None,
                };
                assert_eq!(stance.as_deref(), expected, "{at}: {id}");
                if stance.is_none() {
                    neutral_readings += 1;
                    // No stance edge in the graph, and the ledger holds it.
                    assert!(
                        shown["relations"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .all(|edge| edge["from"] != id.as_str()
                                || (edge["relation"] != "supports"
                                    && edge["relation"] != "opposes")),
                        "{at}: {id} has a stance relation"
                    );
                }
                // The ledger, once a neutral observation needs it, holds every
                // reading in order and ends with the one just taken.
                if shown.get("observations").is_some() {
                    assert_eq!(
                        strings(&shown["observations"]).last(),
                        taken.last().map(|(_, last, _, _)| last),
                        "{at}: the ledger does not end with the reading just taken"
                    );
                    assert!(
                        strings(&shown["observations"]).contains(id),
                        "{at}: {id} is not observed"
                    );
                }
                // A changed trigger compares with the stream's previous reading,
                // whatever became of it.
                let previous = history.get(stream).and_then(|list| list.last().cloned());
                history.entry(stream.clone()).or_default().push((
                    id.clone(),
                    *value,
                    stance.clone(),
                ));
                let _ = previous;
            }
            // The ledger exists exactly once something was observed without a stance.
            let neutral_seen = history
                .values()
                .any(|list| list.iter().any(|(_, _, stance)| stance.is_none()))
                || strings(&shown["observations"]).contains(&"f".to_string());
            assert_eq!(
                shown.get("observations").is_some(),
                neutral_seen
                    || shown["observations"]
                        .as_array()
                        .is_some_and(|list| list.iter().any(|item| item == "f")),
                "{at}: observations {:?}",
                shown.get("observations")
            );
            if let Some(ledger) = shown.get("observations") {
                let ledger = strings(ledger);
                for list in history.values() {
                    for (id, _, _) in list {
                        let live = new
                            .values()
                            .any(|entries| entries.iter().any(|(known, _, _)| known == id));
                        assert_eq!(ledger.contains(id), live, "{at}: {id} in the ledger");
                    }
                }
            }
            // `latest` is the newest reading, neutral or not.
            for stream in ["s", "t"] {
                let newest = history.get(stream).and_then(|list| list.last());
                assert_eq!(
                    shown["bindings"]["view"][format!("latest_{stream}")].as_f64(),
                    newest.map(|(_, value, _)| *value),
                    "{at}: latest({stream})"
                );
            }

            // Which readings each series' triggers match, in rule order.
            let old_journal = before["decision_journal"].as_array().unwrap();
            let new_journal = shown["decision_journal"].as_array().unwrap();
            assert!(
                new_journal.len() >= old_journal.len()
                    && new_journal[..old_journal.len()] == old_journal[..],
                "{at}: the journal is not append-only"
            );
            let added = &new_journal[old_journal.len()..];
            for (series, triggers) in SERIES {
                let revision = in_force(&before, series);
                let matching = taken.iter().find(|(stream, id, value, stance)| {
                    triggers.iter().any(|(watched, changed, filter)| {
                        if watched != stream {
                            return false;
                        }
                        if let Some(filter) = filter {
                            if stance.as_deref() != Some(*filter) {
                                if stance.is_none() {
                                    neutral_skips += 1;
                                }
                                return false;
                            }
                        }
                        if *changed {
                            let list = &history[stream];
                            let position =
                                list.iter().position(|(known, _, _)| known == id).unwrap();
                            if position > 0 && list[position - 1].1 == *value {
                                changed_skips += 1;
                                return false;
                            }
                        }
                        true
                    })
                });
                let reopened: Vec<&Value> = added
                    .iter()
                    .filter(|entry| {
                        entry["change"] == "reopened"
                            && entry["commitment"]
                                .as_str()
                                .is_some_and(|id| id.starts_with(&format!("{series}@")))
                    })
                    .collect();
                match (revision, matching) {
                    (Some(current), Some((_, id, _, _))) => {
                        reopenings += 1;
                        assert_eq!(reopened.len(), 1, "{at}: {series}: {added:?}");
                        assert_eq!(
                            reopened[0]["commitment"],
                            current.as_str(),
                            "{at}: {series}"
                        );
                        assert_eq!(
                            strings(&reopened[0]["because"]),
                            vec![id.clone()],
                            "{at}: {series} reopened because of the wrong reading"
                        );
                        assert_eq!(
                            in_force(&shown, series),
                            None,
                            "{at}: {series} still in force"
                        );
                    }
                    _ => {
                        assert!(
                            reopened.is_empty(),
                            "{at}: {series} reopened with nothing to reopen it: {added:?}"
                        );
                        if !event.starts_with("decide") {
                            // Departure may rewrite what a selection read; the
                            // revisions and the current one stay.
                            for field in ["current", "revisions"] {
                                assert_eq!(
                                    shown["decision_series"][series][field],
                                    before["decision_series"][series][field],
                                    "{at}: {series} {field} changed"
                                );
                            }
                        }
                    }
                }
            }
            // Every event's save restores, to the same save and snapshot (D021).
            let saved_after = game.save_json().unwrap();
            let restored_after = ReactiveSession::restore_json(SOURCE, &saved_after)
                .unwrap_or_else(|error| {
                    panic!("run {run} step {step}: restore after the event: {error}")
                });
            assert_eq!(
                restored_after.save_json().unwrap(),
                saved_after,
                "run {run} step {step}: save round trip after the event"
            );
            assert_eq!(
                snapshot(&restored_after),
                shown,
                "run {run} step {step}: snapshot after restore"
            );
            before = shown;
        }
    }
    eprintln!("{accepted} accepted events, {reopenings} trigger reopenings, {changed_skips} unchanged readings skipped, {neutral_skips} neutral readings skipped by a filter, {neutral_readings} neutral readings");
    assert!(
        reopenings > 500 && changed_skips > 300 && neutral_skips > 300 && neutral_readings > 2000,
        "{reopenings} / {changed_skips} / {neutral_skips} / {neutral_readings}"
    );
}
