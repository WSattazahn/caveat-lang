// Latest 0.1 under random events: `qualify latest(NAME)` puts a caveat on
// the stream's current reading or the evidence's current occurrence and on
// nothing else, once per live relation, refusing as unobserved_evidence when
// the stream has no reading; `unqualify latest(NAME)` takes it back from that
// occurrence only, reporting exactly the occurrence it marked and nothing
// when there is nothing live, while `unqualify E` marks every held
// occurrence; a value read afterwards carries the caveat exactly when its
// reading's relation is live; a later reading does not inherit it; a
// decision keeps the caveats it was made on; records on a departed reading
// leave with it; a refusal changes nothing; and a session restored from the
// live one's save before every event runs on as the live one.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeMap;

const SOURCE: &str = r#"
claim ok;
caveat flaky consequence material;
caveat dusty consequence low;
evidence runner from "the test runner";
evidence probe from "probe";
evidence e from "operator";
renewable e window 3;
dusty qualifies runner;
readings checks from runner window 4;
readings probes from probe limit 40;
decisions ship limit 20;
state last = 0;
state lastp = 0;
state hist = 0;
state ev = 0;
state cl = 0;
state pl = 0;
state el = 0;
event report v min 0 max 3;
event probe_it v min 0 max 3;
event see_e;
event renew_e;
event flag;
event unflag;
event flag_p;
event unflag_p;
event flag_e;
event unflag_e;
event unflag_all_e;
event decide;
event reconsider;
event read;
event read_at k min 0 max 9;
event retract;
event ask;
on report sample checks = v supports ok;
on probe_it sample probes = v supports ok;
on see_e when not observed(e) reveal e supports ok;
on renew_e when observed(e) renew e;
on renew_e reveal e supports ok;
on flag qualify latest(checks) with flaky;
on unflag unqualify latest(checks) with flaky;
on flag_p qualify latest(probes) with flaky;
on unflag_p unqualify latest(probes) with flaky;
on flag_e qualify latest(e) with flaky;
on unflag_e unqualify latest(e) with flaky;
on unflag_all_e unqualify e with flaky;
on decide when not committed(ship) or reopened(ship) commit ship because enough using latest(checks) + latest(probes);
on reconsider when committed(ship) and not reopened(ship) and observed(e) reopen ship because e;
on read set last = latest(checks);
on read set lastp = latest(probes);
on read set ev = qualified(1, e);
on read_at set hist = history_at(probes, k);
on retract when has_sample(checks) and observed(e) and not withdrawn(latest(checks)) withdraw latest(checks) because e;
on ask set cl = if(carries(latest(checks), flaky), 1, 0);
on ask set pl = if(carries(latest(probes), flaky), 1, 0);
on ask set el = if(carries(latest(e), flaky), 1, 0);
"#;

const EVENTS: [(&str, &str); 33] = [
    ("report", r#"{"v": 1}"#),
    ("report", r#"{"v": 2}"#),
    ("report", r#"{"v": 3}"#),
    ("probe_it", r#"{"v": 1}"#),
    ("probe_it", r#"{"v": 2}"#),
    ("see_e", "{}"),
    ("renew_e", "{}"),
    ("renew_e", "{}"),
    ("flag", "{}"),
    ("flag", "{}"),
    ("flag", "{}"),
    ("unflag", "{}"),
    ("unflag", "{}"),
    ("flag_p", "{}"),
    ("flag_p", "{}"),
    ("unflag_p", "{}"),
    ("flag_e", "{}"),
    ("flag_e", "{}"),
    ("unflag_e", "{}"),
    ("unflag_all_e", "{}"),
    ("decide", "{}"),
    ("decide", "{}"),
    ("reconsider", "{}"),
    ("read", "{}"),
    ("read", "{}"),
    ("read", "{}"),
    ("read_at", r#"{"k": 0}"#),
    ("read_at", r#"{"k": 2}"#),
    ("read_at", r#"{"k": 5}"#),
    ("retract", "{}"),
    ("ask", "{}"),
    ("ask", "{}"),
    ("ask", "{}"),
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

fn readings(shown: &Value, stream: &str) -> Vec<String> {
    shown["reading_streams"][stream]["occurrences"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|item| item["id"].as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn latest(shown: &Value, stream: &str) -> Option<String> {
    readings(shown, stream).last().cloned()
}

fn occurrences(shown: &Value) -> Vec<String> {
    strings(&shown["renewals"]["e"]["occurrences"])
}

fn observed(shown: &Value, occurrence: &str) -> bool {
    shown["relations"].as_array().unwrap().iter().any(|edge| {
        edge["from"] == occurrence
            && (edge["relation"] == "supports" || edge["relation"] == "opposes")
    })
}

fn value_caveats(shown: &Value, state: &str) -> Vec<String> {
    strings(&shown["qualified_values"][state]["provenance"]["caveats"])
}

/// Whether some stored observation or predicate qualification keeps flaky.
fn kept_flaky(shown: &Value) -> bool {
    let keeps = |record: &Value| {
        record["kept"]
            .as_array()
            .is_some_and(|list| list.iter().any(|item| item == "flaky"))
    };
    shown["observation_qualifications"]
        .as_object()
        .is_some_and(|records| records.values().any(keeps))
        || shown["predicate_qualifications"]
            .as_object()
            .is_some_and(|kinds| {
                kinds
                    .values()
                    .any(|targets| targets.as_object().is_some_and(|t| t.values().any(keeps)))
            })
}

fn open(shown: &Value, revision: &str) -> bool {
    shown["commitments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["action"] == revision)
        .map(|item| item["open"] == true)
        .unwrap_or(true)
}

#[test]
fn the_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn the_latest_reading_is_qualified_and_taken_back_alone() {
    let mut seed: u64 = 0x6e2b_c8d1_43f9_a075;
    let (
        mut accepted,
        mut refused,
        mut flagged,
        mut taken_one,
        mut taken_all,
        mut read_flaky,
        mut commits_flaky,
        mut departed_flagged,
        mut asked_flaky,
    ) = (0, 0, 0, 0, 0, 0, 0, 0, 0);
    for run in 0..200 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        // Whether flaky's relation on each reading or occurrence is live now.
        let mut live: BTreeMap<String, bool> = BTreeMap::new();
        // Whether an occurrence of e departed its window while flaky's
        // relation on it was live (see below).
        let mut kept_e = false;
        // What each revision was made on, frozen at its commit.
        let mut made_on: Vec<(String, Vec<String>)> = Vec::new();
        for step in 0..60 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                trail.push("save".to_string());
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
            trail.push(format!("{event} {payload}"));
            let at = format!("run {run} step {step} after {event} {payload}: {trail:?}");
            let save_before = game.save_json().unwrap();
            let mut mirror = ReactiveSession::restore_json(SOURCE, &save_before)
                .unwrap_or_else(|error| panic!("{at}: mirror restore: {error}"));
            let outcome = serde_json::to_value(
                game.dispatch_outcome_json(event, payload)
                    .unwrap_or_else(|fatal| panic!("{at}: fatal {}", fatal.message)),
            )
            .unwrap();
            let mirrored = serde_json::to_value(
                mirror
                    .dispatch_outcome_json(event, payload)
                    .unwrap_or_else(|fatal| panic!("{at}: mirror fatal {}", fatal.message)),
            )
            .unwrap();
            assert_eq!(
                mirrored, outcome,
                "{at}: the restored session ran differently"
            );
            let checks_latest = latest(&before, "checks");
            let probes_latest = latest(&before, "probes");
            let e_current = occurrences(&before)
                .last()
                .cloned()
                .unwrap_or_else(|| "e".to_string());
            let e_observed = observed(&before, &e_current);
            let current = before["decision_series"]["ship"]["current"]
                .as_str()
                .map(str::to_string);
            let fires = current
                .as_ref()
                .is_none_or(|revision| open(&before, revision));
            let k = payload
                .split(':')
                .nth(1)
                .and_then(|text| text.trim().trim_end_matches('}').parse::<usize>().ok());
            let expected: Option<(&str, &str)> = match event {
                "flag" if checks_latest.is_none() => Some(("evaluation", "unobserved_evidence")),
                "flag_p" if probes_latest.is_none() => Some(("evaluation", "unobserved_evidence")),
                "flag_e" if !e_observed => Some(("evaluation", "unobserved_evidence")),
                "decide" if fires && (checks_latest.is_none() || probes_latest.is_none()) => {
                    Some(("evaluation", "expression"))
                }
                "decide" if fires && made_on.len() >= 20 => Some(("limit", "history_limit")),
                "read" if checks_latest.is_none() || probes_latest.is_none() => {
                    Some(("evaluation", "expression"))
                }
                "read" if !e_observed => Some(("evaluation", "unobserved_evidence")),
                "read_at" if k.unwrap() >= readings(&before, "probes").len() => {
                    Some(("evaluation", "expression"))
                }
                _ => None,
            };
            if let Some((origin, code)) = expected {
                assert_eq!(
                    (outcome["origin"].as_str(), outcome["code"].as_str()),
                    (Some(origin), Some(code)),
                    "{at}: {outcome}"
                );
                assert_eq!(
                    game.save_json().unwrap(),
                    save_before,
                    "{at}: a refusal changed the save"
                );
                refused += 1;
                continue;
            }
            assert_eq!(outcome["outcome"], "accepted", "{at}: {outcome}");
            accepted += 1;
            let shown = snapshot(&game);
            assert_eq!(
                snapshot(&mirror),
                shown,
                "{at}: the restored session's snapshot differs"
            );
            // An occurrence of e that departed during this event, while its
            // flaky relation was live, leaves flaky kept in every later
            // occurrence's record, which lists it through the renewal guard
            // (Take-back 0.1, "Departure and collection"). A value read in
            // this same event already sees that.
            {
                let held_now = occurrences(&shown);
                if live.iter().any(|(target, is_live)| {
                    *is_live && target.starts_with("e@") && !held_now.contains(target)
                }) {
                    kept_e = true;
                }
            }
            // The model: what the event did to flaky's relations.
            let effects = shown["effects"].as_array().unwrap();
            let unqualified: Option<Vec<String>> = effects
                .iter()
                .find(|effect| effect["kind"] == "unqualify")
                .map(|effect| strings(&effect["occurrences"]));
            let qualified: Vec<String> = effects
                .iter()
                .filter(|effect| effect["kind"] == "qualify")
                .map(|effect| effect["evidence"].as_str().unwrap().to_string())
                .collect();
            let marked = |target: &str, live: &mut BTreeMap<String, bool>| -> Vec<String> {
                if live.get(target).copied().unwrap_or(false) {
                    live.insert(target.to_string(), false);
                    vec![target.to_string()]
                } else {
                    vec![]
                }
            };
            let expected_marked: Option<Vec<String>> = match event {
                "flag" => {
                    let target = checks_latest.clone().unwrap();
                    assert_eq!(qualified, vec![target.clone()], "{at}: qualify reported");
                    if !live.get(&target).copied().unwrap_or(false) {
                        flagged += 1;
                    }
                    live.insert(target, true);
                    None
                }
                "flag_p" => {
                    let target = probes_latest.clone().unwrap();
                    assert_eq!(qualified, vec![target.clone()], "{at}: qualify reported");
                    live.insert(target, true);
                    None
                }
                "flag_e" => {
                    assert_eq!(qualified, vec![e_current.clone()], "{at}: qualify reported");
                    live.insert(e_current.clone(), true);
                    None
                }
                "unflag" => Some(
                    checks_latest
                        .as_deref()
                        .map(|target| marked(target, &mut live))
                        .unwrap_or_default(),
                ),
                "unflag_p" => Some(
                    probes_latest
                        .as_deref()
                        .map(|target| marked(target, &mut live))
                        .unwrap_or_default(),
                ),
                "unflag_e" => Some(marked(&e_current, &mut live)),
                "unflag_all_e" => {
                    let mut all = Vec::new();
                    // A take-back reaches held occurrences that have not
                    // retired (Take-back 0.1 under windows).
                    for occurrence in occurrences(&before) {
                        if before["retired"][&occurrence].is_number() {
                            continue;
                        }
                        all.extend(marked(&occurrence, &mut live));
                    }
                    Some(all)
                }
                _ => {
                    assert!(
                        qualified.is_empty() && unqualified.is_none(),
                        "{at}: {effects:?}"
                    );
                    None
                }
            };
            if let Some(expected) = expected_marked {
                // A take-back reports exactly what it marked, or nothing.
                let reported = unqualified.clone().unwrap_or_default();
                assert_eq!(
                    reported, expected,
                    "{at}: take-back reported {unqualified:?}"
                );
                assert_eq!(
                    unqualified.is_some(),
                    !expected.is_empty(),
                    "{at}: {effects:?}"
                );
                match (event, expected.len()) {
                    (_, 0) => {}
                    ("unflag_all_e", _) => taken_all += 1,
                    _ => taken_one += 1,
                }
            }
            // A new reading or occurrence starts without the caveat.
            for reading in readings(&shown, "checks")
                .into_iter()
                .chain(readings(&shown, "probes"))
                .chain(occurrences(&shown))
            {
                live.entry(reading).or_insert(false);
            }
            let carries = |target: &str| live.get(target).copied().unwrap_or(false);
            // A value read now carries the caveat exactly when its source's
            // relation is live; a reading of checks always copies dusty.
            if event == "read" {
                let c = latest(&shown, "checks").unwrap();
                let p = latest(&shown, "probes").unwrap();
                let caveats = value_caveats(&shown, "last");
                assert_eq!(
                    caveats.contains(&"flaky".to_string()),
                    carries(&c),
                    "{at}: last read {c}: {caveats:?}"
                );
                assert!(caveats.contains(&"dusty".to_string()), "{at}: {caveats:?}");
                let caveats = value_caveats(&shown, "lastp");
                assert_eq!(
                    caveats.contains(&"flaky".to_string()),
                    carries(&p),
                    "{at}: lastp read {p}: {caveats:?}"
                );
                assert!(!caveats.contains(&"dusty".to_string()), "{at}: {caveats:?}");
                // qualified(1, e) reads every held occurrence of e (Renewal
                // 0.1), so it carries flaky while any of them does.
                let held_e = occurrences(&shown);
                let current = if held_e.iter().any(|occurrence| carries(occurrence)) {
                    "one of e's occurrences".to_string()
                } else {
                    "none of e's occurrences".to_string()
                };
                let caveats = value_caveats(&shown, "ev");
                let expected = held_e.iter().any(|occurrence| carries(occurrence)) || kept_e;
                // D020: a stored observation or predicate qualification
                // that took flaky from an unguarded qualify must not keep it,
                // so a take-back reaches values read through it.
                assert!(
                    !(caveats.contains(&"flaky".to_string()) && !expected && kept_flaky(&shown)),
                    "{at}: ev kept flaky through a stored record (D020): {caveats:?}"
                );
                assert_eq!(
                    caveats.contains(&"flaky".to_string()),
                    expected,
                    "{at}: ev read {current}: {caveats:?}"
                );
                if carries(&c) || carries(&p) || held_e.iter().any(|occurrence| carries(occurrence))
                {
                    read_flaky += 1;
                }
            }
            // carries(latest(NAME), flaky) asks about the current reading
            // or occurrence alone, and is false with none (Latest 0.1).
            if event == "ask" {
                let expected = [
                    ("cl", checks_latest.as_deref().is_some_and(carries)),
                    ("pl", probes_latest.as_deref().is_some_and(carries)),
                    ("el", e_observed && carries(&e_current)),
                ];
                for (state, carried) in expected {
                    let read = shown["qualified_values"][state]["value"].as_f64().unwrap();
                    assert_eq!(
                        read,
                        if carried { 1.0 } else { 0.0 },
                        "{at}: {state} on {checks_latest:?} {probes_latest:?} {e_current}"
                    );
                    if carried {
                        asked_flaky += 1;
                    }
                }
            }
            if event == "read_at" {
                let reading = readings(&shown, "probes")[k.unwrap()].clone();
                let caveats = value_caveats(&shown, "hist");
                assert_eq!(
                    caveats.contains(&"flaky".to_string()),
                    carries(&reading),
                    "{at}: history_at read {reading}: {caveats:?}"
                );
            }
            // A decision keeps the caveats it was made on.
            for revision in strings(&Value::Array(
                shown["decision_series"]["ship"]["revisions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|item| item["id"].clone())
                    .collect(),
            )) {
                if !made_on.iter().any(|(id, _)| *id == revision) {
                    let c = checks_latest.clone().unwrap();
                    let p = probes_latest.clone().unwrap();
                    let caveats = strings(&shown["commitment_grounds"][&revision]["caveats"]);
                    assert_eq!(
                        caveats.contains(&"flaky".to_string()),
                        carries(&c) || carries(&p),
                        "{at}: {revision} made on {c} and {p}: {caveats:?}"
                    );
                    if caveats.contains(&"flaky".to_string()) {
                        commits_flaky += 1;
                    }
                    made_on.push((revision, caveats));
                }
            }
            for (revision, caveats) in &made_on {
                assert_eq!(
                    &strings(&shown["commitment_grounds"][revision]["caveats"]),
                    caveats,
                    "{at}: {revision}'s grounds changed"
                );
            }
            // Relations and take-back records name held readings only.
            let held: Vec<String> = readings(&shown, "checks")
                .into_iter()
                .chain(readings(&shown, "probes"))
                .chain(occurrences(&shown))
                .collect();
            for edge in shown["relations"].as_array().unwrap() {
                if edge["from"] == "flaky" {
                    let to = edge["to"].as_str().unwrap();
                    assert!(
                        held.contains(&to.to_string()) || to == "e",
                        "{at}: flaky qualifies {to}, which departed"
                    );
                }
            }
            for record in shown["taken_back"].as_array().into_iter().flatten() {
                let evidence = record["evidence"].as_str().unwrap();
                assert!(
                    held.contains(&evidence.to_string()) || evidence == "e",
                    "{at}: a take-back of {evidence}, which departed"
                );
            }
            for (target, is_live) in &live {
                if *is_live && target.starts_with("checks@") && !held.contains(target) {
                    departed_flagged += 1;
                }
            }
            live.retain(|target, _| held.contains(target) || target == "e");
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
    eprintln!("{accepted} accepted events, {refused} refused, {flagged} readings flagged, {taken_one} take-backs of one occurrence, {taken_all} of every held occurrence, {read_flaky} reads carrying flaky, {commits_flaky} decisions made with flaky, {departed_flagged} flagged readings departed, {asked_flaky} carries(latest) true");
    assert!(
        flagged > 250
            && taken_one > 150
            && taken_all > 50
            && read_flaky > 250
            && commits_flaky > 50
            && departed_flagged > 30
            && asked_flaky > 0,
        "{flagged} / {taken_one} / {taken_all} / {read_flaky} / {commits_flaky} / {departed_flagged} / {asked_flaky}"
    );
}
