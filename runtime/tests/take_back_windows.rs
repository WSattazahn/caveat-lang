// Take-back 0.1 under windows and departure, as a randomized check: the
// invariant the take_back.rs check states, over a program whose renewable
// evidence, reading stream and journal retire and depart records, with
// guarded renewals and reveals, same-named caveats through a guard, late
// qualifications, withdrawals under a guard, decisions pinning records, and
// saves and restores in place. After every accepted event:
//
// - a value carries `flaky` exactly when a held occurrence in its lineage has
//   a live `flaky qualifies` relation, or it keeps `flaky`; `has_caveat` the
//   same over its grounds ("Meaning" 3 and 4, "Kept caveats");
// - a take-back marks relations on held occurrences only: a retired
//   occurrence's relation stays live ("Departure and collection");
// - when a record departs, every current value that held it keeps each
//   take-back caveat its live relations gave it ("Departure and collection");
// - the archive the host drains holds the departures reported, and a restored
//   session is the one that saved.
//
// In a debug build the runtime also checks its withdrawal collector against
// a rebuild after every event, which is where D016 showed.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeSet;

const SOURCE: &str = r#"
caveat flaky consequence material;
claim c;
evidence e from "tool";
renewable e window 2;
evidence f from "operator";
renewable f window 2;
evidence probe from "probe";
readings s from probe window 3;
evidence op from "operator check";
decisions trust limit 6;
journal window 3;
state held = 0;
state v = 0;
state w = 0;
state fz = 0;
state g = 0;
event report;
event plain;
event via_f;
event via_fz;
event see_f;
event read;
event flag;
event unflag;
event flag_f;
event unflag_f;
event flag_late;
event check;
event show;
event decide;
event doubt;
event withdraw;
event see_op;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on decide when observed(e) commit trust because enough using v;
on doubt when committed(trust) and observed(e) reopen trust because e;
on see_op reveal op;
on withdraw when observed(op) and observed(e) withdraw e because op;
on report when observed(e) renew e;
on report reveal e;
on plain when not observed(e) renew e;
on plain reveal e;
on via_f when observed(f) renew e;
on via_f when observed(f) reveal e;
on see_f when observed(f) renew f;
on see_f reveal f;
on see_f when observed(f) set fz = 1;
on via_fz when fz == 1 renew e;
on via_fz when fz == 1 reveal e;
on via_fz when observed(e) set v = qualified(1, e);
on read when observed(e) sample s = 1 supports c;
on read when has_sample(s) set w = latest(s);
on flag when observed(e) qualify e with flaky;
on unflag unqualify e with flaky;
on flag_f when observed(f) qualify f with flaky;
on unflag_f when observed(f) unqualify f with flaky;
on flag_late when observed(e) qualify e with flaky after 0.2;
on show when observed(f) and observed(e) reveal e supports c;
on report set v = qualified(1, e);
on plain set v = qualified(1, e);
on via_f when observed(e) set v = qualified(1, e);
on show when observed(e) set v = qualified(1, e);
on check when observed(e) set held = v + qualified(0, e);
on check when observed(f) set g = qualified(1, f) + v;
bind view.v = if(has_caveat(v, flaky), 1, 0);
bind view.held = if(has_caveat(held, flaky), 1, 0);
"#;

const EVENTS: [(&str, &str); 22] = [
    ("report", "{}"),
    ("plain", "{}"),
    ("via_f", "{}"),
    ("via_fz", "{}"),
    ("see_f", "{}"),
    ("read", "{}"),
    ("flag", "{}"),
    ("unflag", "{}"),
    ("flag_f", "{}"),
    ("unflag_f", "{}"),
    ("flag_late", "{}"),
    ("check", "{}"),
    ("show", "{}"),
    ("decide", "{}"),
    ("doubt", "{}"),
    ("withdraw", "{}"),
    ("see_op", "{}"),
    ("tick", r#"{"dt": 0.1}"#),
    ("tick", r#"{"dt": 0.1}"#),
    ("report", "{}"),
    ("flag", "{}"),
    ("unflag", "{}"),
];

const STATES: [&str; 5] = ["v", "held", "w", "fz", "g"];

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

fn holds(list: &Value) -> bool {
    list.as_array()
        .is_some_and(|caveats| caveats.iter().any(|caveat| caveat == "flaky"))
}

/// Live: more `flaky qualifies occurrence` relations than take-back records.
fn live(shown: &Value, occurrence: &str) -> bool {
    let relations = shown["relations"].as_array().map_or(0, |list| {
        list.iter()
            .filter(|edge| {
                edge["from"] == "flaky"
                    && edge["to"] == occurrence
                    && edge["relation"] == "qualifies"
            })
            .count()
    });
    let taken = shown["taken_back"].as_array().map_or(0, |list| {
        list.iter()
            .filter(|record| record["caveat"] == "flaky" && record["evidence"] == occurrence)
            .count()
    });
    relations > taken
}

fn check(shown: &Value, at: &str) {
    for state in STATES {
        let provenance = &shown["qualified_values"][state]["provenance"];
        let lineage_live = strings(&provenance["evidence"])
            .iter()
            .any(|occurrence| live(shown, occurrence));
        let carried = holds(&provenance["caveats"]);
        let kept = holds(&provenance["kept"]);
        assert_eq!(
            carried,
            lineage_live || kept,
            "{at}: {state} lineage {provenance}\nrelations {}\ntaken_back {}",
            shown["relations"],
            shown["taken_back"]
        );
        let grounds = &shown["value_grounds"][state];
        let grounds_live = strings(&grounds["evidence"])
            .iter()
            .any(|occurrence| live(shown, occurrence));
        assert_eq!(
            holds(&grounds["caveats"]),
            grounds_live || holds(&grounds["kept"]),
            "{at}: {state} grounds {grounds}"
        );
    }
    let flag = |truth: bool| Some(if truth { 1.0 } else { 0.0 });
    for state in ["v", "held"] {
        assert_eq!(
            shown["bindings"]["view"][state].as_f64(),
            flag(holds(&shown["value_grounds"][state]["caveats"])),
            "{at}: has_caveat({state}, flaky)"
        );
    }
}

#[test]
fn the_windowed_take_back_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn take_backs_under_windows_keep_caveats_matched_to_held_evidence() {
    let mut seed: u64 = 0xcbbb_9d5d_c105_9ed8;
    let (
        mut accepted,
        mut take_backs,
        mut departures,
        mut kept_by_departure,
        mut retired_untouched,
    ) = (0, 0, 0, 0, 0);
    for run in 0..200 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        for step in 0..60 {
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
                    "run {run} step {step} after restore: {trail:?}"
                );
                assert_eq!(
                    restored.save_json().unwrap(),
                    saved,
                    "run {run} step {step} save after restore: {trail:?}"
                );
                game = restored;
                continue;
            }
            let (event, payload) = EVENTS[pick];
            trail.push((event, payload));
            let outcome =
                serde_json::to_value(game.dispatch_outcome_json(event, payload).unwrap_or_else(
                    |fatal| panic!("run {run} step {step} fatal {}: {trail:?}", fatal.message),
                ))
                .unwrap();
            if outcome["outcome"] != "accepted" {
                continue;
            }
            accepted += 1;
            let shown = snapshot(&game);
            let at = format!("run {run} step {step} after {event}: {trail:?}");
            check(&shown, &at);
            let effects = shown["effects"].as_array().unwrap();
            let departed = effects
                .iter()
                .filter(|effect| effect["kind"] == "depart")
                .map(|effect| effect["record"].as_str().unwrap().to_string())
                .collect::<Vec<_>>();
            // Records depart with their occurrence, and the rest stay in order.
            let retired_before = before["retired"]
                .as_object()
                .map(|map| map.keys().cloned().collect::<BTreeSet<_>>())
                .unwrap_or_default();
            let staying = before["taken_back"]
                .as_array()
                .map(|list| {
                    list.iter()
                        .filter(|record| {
                            !departed.contains(&record["evidence"].as_str().unwrap().to_string())
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let now = shown["taken_back"].as_array().cloned().unwrap_or_default();
            assert!(
                now.len() >= staying.len() && now[..staying.len()] == staying[..],
                "{at}: take-back records lost or reordered\n{staying:?}\n{now:?}"
            );
            // A take-back marks held occurrences only.
            for record in now.iter().skip(staying.len()) {
                take_backs += 1;
                let occurrence = record["evidence"].as_str().unwrap();
                assert!(
                    !retired_before.contains(occurrence),
                    "{at}: a take-back marked retired {occurrence}"
                );
                assert_eq!(
                    record["event"], event,
                    "{at}: a take-back record names another event: {record}"
                );
                assert_eq!(record["sequence"], shown["sequence"], "{at}");
                let reported = effects.iter().any(|effect| {
                    effect["kind"] == "unqualify"
                        && strings(&effect["occurrences"]).contains(&occurrence.to_string())
                });
                assert!(
                    reported,
                    "{at}: no unqualify effect names {occurrence}: {effects:?}"
                );
            }
            if event == "unflag" || event == "unflag_f" {
                // A retired occurrence with a live relation stays live.
                for (occurrence, _) in before["retired"]
                    .as_object()
                    .map(|map| map.iter().collect::<Vec<_>>())
                    .unwrap_or_default()
                {
                    if live(&before, occurrence) && shown["retired"].get(occurrence).is_some() {
                        retired_untouched += 1;
                        assert!(
                            live(&shown, occurrence),
                            "{at}: a take-back reached retired {occurrence}"
                        );
                    }
                }
            }
            // Departure keeps what a departed record's live relations gave.
            departures += departed.len();
            for record in &departed {
                if !live(&before, record) {
                    continue;
                }
                for state in STATES {
                    let old = &before["qualified_values"][state]["provenance"];
                    let new = &shown["qualified_values"][state]["provenance"];
                    let old_evidence = strings(&old["evidence"]);
                    if !old_evidence.contains(record) {
                        continue;
                    }
                    // The value was not recomputed this event: its evidence is
                    // the old evidence less what departed.
                    let compacted = old_evidence
                        .iter()
                        .filter(|name| !departed.contains(name))
                        .cloned()
                        .collect::<Vec<_>>();
                    if strings(&new["evidence"]) != compacted {
                        continue;
                    }
                    kept_by_departure += 1;
                    assert!(holds(&new["caveats"]) && holds(&new["kept"]), "{at}: {state} held departed {record} with a live flaky relation and does not keep flaky: {new}");
                }
            }
            let drained = serde_json::to_value(game.drain_archive()).unwrap();
            let archived = drained
                .as_array()
                .unwrap()
                .iter()
                .filter(|item| item.get("kind") != Some(&Value::String("provenance".into())))
                .map(|item| item["record"].as_str().unwrap().to_string())
                .collect::<Vec<_>>();
            assert_eq!(
                archived, departed,
                "{at}: the archive and the reported departures disagree"
            );
            before = shown;
        }
    }
    eprintln!("{accepted} accepted events, {take_backs} take-back records, {departures} departures, {kept_by_departure} caveats kept by departure, {retired_untouched} retired relations left live by a take-back");
    assert!(take_backs > 300 && departures > 800 && kept_by_departure > 20 && retired_untouched > 50, "the alphabet barely reached the rules: {take_backs} / {departures} / {kept_by_departure} / {retired_untouched}");
}
