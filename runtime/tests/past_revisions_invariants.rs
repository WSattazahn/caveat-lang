// Past Revisions 0.1 under random events: rests_on_withdrawn(revision(D, K))
// answers, for every K at every step, whether the frozen grounds of D@K hold
// evidence withdrawn now (false for an index naming no revision); for the
// current revision it answers as rests_on_withdrawn(D); a true answer carries
// the withdrawals' reasons; the question survives the journal retiring the
// revision's entries; a series read this way keeps every revision's grounds
// held and their withdrawals standing while windows retire other records;
// asking changes nothing; a refusal changes nothing; and a session restored
// from its save at every step runs on as the live one, departures included.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const SOURCE: &str = r#"
claim passing;
evidence tests from "test runner";
renewable tests window 3;
evidence probe from "probe";
readings runs from probe window 3;
evidence operator from "the operator's correction";
decisions ship limit 8 reopened by runs opposing passing;
journal window 2;
state answer = 0;
event first_report;
event report;
event run_ok x min 0 max 3;
event run_bad x min 0 max 3;
event decide;
event decide_runs;
event reconsider;
event correct n min 0 max 9;
event retract_run;
event ask k min -1 max 9;
on first_report when not observed(tests) reveal tests supports passing;
on report when observed(tests) renew tests;
on report reveal tests supports passing;
on run_ok sample runs = x supports passing;
on run_bad sample runs = x opposes passing;
on decide when (not committed(ship) or reopened(ship)) and observed(tests) commit ship because enough using qualified(1, tests);
on decide_runs when (not committed(ship) or reopened(ship)) and has_sample(runs) commit ship because enough using latest(runs);
on reconsider when committed(ship) and not reopened(ship) and observed(tests) reopen ship because tests;
on correct when not observed(occurrence(tests, n)) reject "no such report";
on correct when withdrawn(occurrence(tests, n)) reject "already withdrawn";
on correct when not observed(operator) reveal operator;
on correct withdraw occurrence(tests, n) because operator;
on retract_run when not has_sample(runs) reject "no run";
on retract_run when withdrawn(latest(runs)) reject "already withdrawn";
on retract_run when not observed(operator) reveal operator;
on retract_run withdraw latest(runs) because operator;
on ask set answer = if(rests_on_withdrawn(revision(ship, k)), 1, 0);
bind view.r0 = if(rests_on_withdrawn(revision(ship, 0)), 1, 0);
bind view.r1 = if(rests_on_withdrawn(revision(ship, 1)), 1, 0);
bind view.r2 = if(rests_on_withdrawn(revision(ship, 2)), 1, 0);
bind view.r3 = if(rests_on_withdrawn(revision(ship, 3)), 1, 0);
bind view.r4 = if(rests_on_withdrawn(revision(ship, 4)), 1, 0);
bind view.r5 = if(rests_on_withdrawn(revision(ship, 5)), 1, 0);
bind view.r6 = if(rests_on_withdrawn(revision(ship, 6)), 1, 0);
bind view.r7 = if(rests_on_withdrawn(revision(ship, 7)), 1, 0);
bind view.r8 = if(rests_on_withdrawn(revision(ship, 8)), 1, 0);
bind view.r9 = if(rests_on_withdrawn(revision(ship, 9)), 1, 0);
bind view.half = if(rests_on_withdrawn(revision(ship, 2.5)), 1, 0);
bind view.current = if(rests_on_withdrawn(ship), 1, 0) when committed(ship);
"#;

const EVENTS: [(&str, &str); 24] = [
    ("first_report", "{}"),
    ("report", "{}"),
    ("report", "{}"),
    ("report", "{}"),
    ("run_ok", r#"{"x": 1}"#),
    ("run_ok", r#"{"x": 2}"#),
    ("run_bad", r#"{"x": 0}"#),
    ("decide", "{}"),
    ("decide", "{}"),
    ("decide_runs", "{}"),
    ("decide_runs", "{}"),
    ("reconsider", "{}"),
    ("reconsider", "{}"),
    ("reconsider", "{}"),
    ("correct", r#"{"n": 1}"#),
    ("correct", r#"{"n": 2}"#),
    ("correct", r#"{"n": 3}"#),
    ("correct", r#"{"n": 5}"#),
    ("retract_run", "{}"),
    ("retract_run", "{}"),
    ("ask", r#"{"k": 1}"#),
    ("ask", r#"{"k": 3}"#),
    ("ask", r#"{"k": 7}"#),
    ("ask", r#"{"k": 0}"#),
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

fn revisions(shown: &Value) -> Vec<String> {
    shown["decision_series"]["ship"]["revisions"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|item| item["id"].as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn withdrawals(shown: &Value) -> Vec<(String, String)> {
    shown["withdrawals"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|item| {
                    (
                        item["evidence"].as_str().unwrap().to_string(),
                        item["because"].as_str().unwrap().to_string(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Whether `name` is still a held record: the declared evidence, a renewal
/// occurrence or a reading.
fn held(shown: &Value, name: &str) -> bool {
    strings(&shown["renewals"]["tests"]["occurrences"])
        .iter()
        .any(|held| held == name)
        || shown["reading_streams"]["runs"]["occurrences"]
            .as_array()
            .is_some_and(|list| list.iter().any(|item| item["id"] == name))
        || shown["symbols"]
            .as_array()
            .unwrap()
            .iter()
            .any(|symbol| symbol["name"] == name && symbol["kind"] == "evidence")
}

fn observed(shown: &Value, occurrence: &str) -> bool {
    shown["relations"].as_array().unwrap().iter().any(|edge| {
        edge["from"] == occurrence
            && (edge["relation"] == "supports" || edge["relation"] == "opposes")
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

fn binding(shown: &Value, name: &str) -> Option<f64> {
    shown["bindings"]["view"][name].as_f64()
}

#[test]
fn the_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn a_past_revision_answers_for_its_frozen_grounds_at_every_step() {
    let mut seed: u64 = 0x2d8f_61c4_9ab3_e507;
    let (
        mut accepted,
        mut commits,
        mut asked_true,
        mut retired_true,
        mut departed,
        mut refused,
        mut retired_named,
    ) = (0, 0, 0, 0, 0, 0, 0);
    for run in 0..150 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        // A session restored from the live one's save before every event, so
        // the index restore rebuilds and the one events keep must agree.
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        // Each revision's grounds, frozen at its commit.
        let mut grounds: Vec<(String, Vec<String>)> = Vec::new();
        let mut seen_records: Vec<String> = Vec::new();
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
            let (event, payload) = EVENTS[pick];
            trail.push(event);
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
            let revs = revisions(&before);
            let current = before["decision_series"]["ship"]["current"]
                .as_str()
                .map(str::to_string);
            let fires = current
                .as_ref()
                .is_none_or(|revision| open(&before, revision));
            let n = payload
                .split(':')
                .nth(1)
                .and_then(|text| text.trim().trim_end_matches('}').parse::<usize>().ok());
            let expected: Option<(&str, &str)> = match event {
                "decide" if fires && observed(&before, "tests") && revs.len() >= 8 => {
                    Some(("limit", "history_limit"))
                }
                "decide_runs"
                    if fires
                        && before["reading_streams"]["runs"]["occurrences"]
                            .as_array()
                            .is_some_and(|list| !list.is_empty())
                        && revs.len() >= 8 =>
                {
                    Some(("limit", "history_limit"))
                }
                "correct" => {
                    let n = n.unwrap();
                    let name = if n == 1 {
                        "tests".to_string()
                    } else {
                        format!("tests@{n}")
                    };
                    // A retired occurrence, held only by a pin, is not one
                    // occurrence(E, I) names (Earlier Occurrence 0.1).
                    if !observed(&before, &name) || before["retired"][&name].is_number() {
                        retired_named += 1;
                        Some(("policy", "reject"))
                    } else if withdrawals(&before).iter().any(|(e, _)| *e == name) {
                        Some(("policy", "reject"))
                    } else {
                        None
                    }
                }
                "retract_run" => {
                    let latest = before["reading_streams"]["runs"]["occurrences"]
                        .as_array()
                        .and_then(|list| list.last())
                        .map(|item| item["id"].as_str().unwrap().to_string());
                    match latest {
                        None => Some(("policy", "reject")),
                        Some(id) if withdrawals(&before).iter().any(|(e, _)| *e == id) => {
                            Some(("policy", "reject"))
                        }
                        Some(_) => None,
                    }
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
            assert_eq!(
                outcome["outcome"], "accepted",
                "{at}: {outcome}\nrenewals {}\nwithdrawals {}\nretired {}",
                before["renewals"], before["withdrawals"], before["retired"]
            );
            accepted += 1;
            let shown = snapshot(&game);
            assert_eq!(
                snapshot(&mirror),
                shown,
                "{at}: the restored session's snapshot differs"
            );
            // New revisions freeze their grounds.
            for revision in revisions(&shown) {
                if !grounds.iter().any(|(id, _)| *id == revision) {
                    commits += 1;
                    let evidence = strings(&shown["commitment_grounds"][&revision]["evidence"]);
                    assert!(!evidence.is_empty(), "{at}: {revision} has no grounds");
                    grounds.push((revision, evidence));
                }
            }
            let now = withdrawals(&shown);
            // The question, for every index at once.
            for k in 0..=9usize {
                let expected = grounds
                    .get(k.wrapping_sub(1))
                    .filter(|_| k >= 1)
                    .map(|(_, evidence)| {
                        if now.iter().any(|(e, _)| evidence.contains(e)) {
                            1.0
                        } else {
                            0.0
                        }
                    })
                    .unwrap_or(0.0);
                assert_eq!(
                    binding(&shown, &format!("r{k}")),
                    Some(expected),
                    "{at}: revision(ship, {k}) with grounds {grounds:?} and withdrawals {now:?}"
                );
                if expected == 1.0 {
                    asked_true += 1;
                    // The answer carries a withdrawal's reason.
                    let lineage = strings(
                        &shown["binding_qualifications"]["view"][&format!("r{k}")]["evidence"],
                    );
                    assert!(
                        lineage.iter().any(|name| name == "operator"),
                        "{at}: r{k} true without the withdrawal's reason: {lineage:?}"
                    );
                    // The journal may have retired the revision's entries.
                    let (id, _) = &grounds[k - 1];
                    let journaled = shown["decision_journal"]
                        .as_array()
                        .is_some_and(|list| list.iter().any(|entry| entry["commitment"] == *id));
                    if !journaled {
                        retired_true += 1;
                    }
                }
            }
            assert_eq!(
                binding(&shown, "half"),
                Some(0.0),
                "{at}: revision(ship, 2.5)"
            );
            if let Some(current) = shown["decision_series"]["ship"]["current"].as_str() {
                let k: usize = current.split('@').nth(1).unwrap().parse().unwrap();
                assert_eq!(
                    binding(&shown, "current"),
                    binding(&shown, &format!("r{k}")),
                    "{at}: rests_on_withdrawn(ship) and revision(ship, {k}) differ"
                );
            }
            if event == "ask" {
                let k = n.unwrap_or(0);
                let expected = binding(&shown, &format!("r{k}")).unwrap();
                assert_eq!(
                    shown["qualified_values"]["answer"]["value"].as_f64(),
                    Some(expected),
                    "{at}: answer"
                );
                // Asking changed no record.
                let mut quiet = before.clone();
                let mut now_shown = shown.clone();
                for field in [
                    "qualified_values",
                    "values",
                    "value_grounds",
                    "sequence",
                    "last_event",
                    "effects",
                    "elapsed",
                ] {
                    quiet[field] = Value::Null;
                    now_shown[field] = Value::Null;
                }
                for (field, value) in quiet.as_object().unwrap() {
                    assert_eq!(&now_shown[field], value, "{at}: asking changed {field}");
                }
            }
            // Every revision's grounds stay held, and their withdrawals stand.
            for (revision, evidence) in &grounds {
                for name in evidence {
                    assert!(
                        held(&shown, name),
                        "{at}: {name}, grounds of {revision}, departed"
                    );
                }
                assert_eq!(
                    &strings(&shown["commitment_grounds"][revision]["evidence"]),
                    evidence,
                    "{at}: {revision}'s grounds changed"
                );
            }
            for (evidence, _) in &withdrawals(&before) {
                if grounds.iter().any(|(_, names)| names.contains(evidence)) {
                    assert!(
                        now.iter().any(|(e, _)| e == evidence),
                        "{at}: the withdrawal of {evidence}, in a revision's grounds, left"
                    );
                }
            }
            // Records outside every revision's grounds do depart under the windows.
            let records: Vec<String> = strings(&shown["renewals"]["tests"]["occurrences"])
                .into_iter()
                .chain(
                    shown["reading_streams"]["runs"]["occurrences"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|item| item["id"].as_str().unwrap().to_string()),
                )
                .collect();
            for name in &seen_records {
                if !records.contains(name) && !held(&shown, name) {
                    departed += 1;
                }
            }
            seen_records = records;
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
    eprintln!("{accepted} accepted events, {refused} refused, {commits} revisions, {asked_true} true answers ({retired_true} with the revision's journal entries retired), {departed} departures of other records, {retired_named} corrections naming a retired report");
    assert!(
        commits > 400 && asked_true > 3000 && retired_true > 1500 && departed > 300,
        "{commits} / {asked_true} / {retired_true} / {departed}"
    );
}
