// Citing 0.1 under random events: a commit with `citing` has the grounds and
// the lineage a `using` commit over the same reads has, the retained caveats
// included, and a basis value of null; its citations are evaluated in order
// and refuse the event on the codes a `using` value meets (a reading of
// unobserved evidence as evaluation/unobserved_evidence, a failed expression
// as evaluation/expression); a decision made with `citing` has no value, so
// `latest` of it refuses as evaluation/expression; its frozen basis never
// changes; withdrawing cited evidence makes `rests_on_withdrawn` true; and a
// restored session is the one that saved. Checked after every event, in
// lockstep between a citing series and its using twin.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const SOURCE: &str = r#"
caveat flaky consequence material;
caveat gaps consequence material;
claim ok;
evidence e from "tool";
evidence f from "operator";
evidence g from "probe";
caveat drift consequence low;
drift qualifies g;
readings s from g limit 40;
state v = 0;
state w = 0;
decisions dc limit 24;
decisions du limit 24;
event see_e;
event see_f;
event read x min 0 max 3;
event set_v;
event flag_e;
event unflag_e;
event retract_e;
event retract_s;
event decide;
event decide_plain;
event reconsider;
event read_dc;
event read_du;
on see_e reveal e supports ok;
on see_f reveal f;
on read sample s = x supports ok;
on set_v when observed(e) set v = qualified(2, e);
on flag_e when observed(e) qualify e with flaky;
on unflag_e unqualify e with flaky;
on retract_e when observed(e) and observed(f) withdraw e because f;
on retract_s when has_sample(s) and observed(f) withdraw latest(s) because f;
on decide when not committed(dc) or reopened(dc) commit dc because enough citing v, latest(s), qualified(1, e) retaining gaps;
on decide when not committed(du) or reopened(du) commit du because enough using v + latest(s) + qualified(1, e) retaining gaps;
on decide_plain when not committed(dc) or reopened(dc) commit dc because enough citing v;
on decide_plain when not committed(du) or reopened(du) commit du because enough using v;
on reconsider when committed(dc) and not reopened(dc) and observed(e) reopen dc because e;
on reconsider when committed(du) and not reopened(du) and observed(e) reopen du because e;
on read_dc when committed(dc) set w = latest(dc);
on read_du when committed(du) set w = latest(du);
bind view.stale_dc = if(rests_on_withdrawn(dc), 1, 0) when committed(dc);
bind view.stale_du = if(rests_on_withdrawn(du), 1, 0) when committed(du);
"#;

const EVENTS: [(&str, &str); 18] = [
    ("see_e", "{}"),
    ("see_f", "{}"),
    ("read", r#"{"x": 1}"#),
    ("read", r#"{"x": 3}"#),
    ("set_v", "{}"),
    ("flag_e", "{}"),
    ("unflag_e", "{}"),
    ("retract_e", "{}"),
    ("retract_s", "{}"),
    ("decide", "{}"),
    ("decide", "{}"),
    ("decide_plain", "{}"),
    ("reconsider", "{}"),
    ("reconsider", "{}"),
    ("read_dc", "{}"),
    ("read_du", "{}"),
    ("decide", "{}"),
    ("set_v", "{}"),
];

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn observed(shown: &Value, occurrence: &str) -> bool {
    shown["relations"].as_array().unwrap().iter().any(|edge| {
        edge["from"] == occurrence
            && (edge["relation"] == "supports" || edge["relation"] == "opposes")
    }) || shown["observations"]
        .as_array()
        .is_some_and(|list| list.iter().any(|item| item == occurrence))
}

fn current(shown: &Value, series: &str) -> Option<String> {
    shown["decision_series"][series]["current"]
        .as_str()
        .map(str::to_string)
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

/// A revision's record, with its series name taken out so twins compare.
fn record(shown: &Value, revision: &str) -> (Value, Value, Value) {
    (
        shown["commitment_grounds"][revision].clone(),
        shown["commitment_bases"][revision]["provenance"].clone(),
        shown["commitments"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["action"] == revision)
            .map(|item| item["retained"].clone())
            .unwrap_or(Value::Null),
    )
}

#[test]
fn the_citing_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn a_citing_commit_has_its_using_twins_grounds_and_no_value() {
    let mut seed: u64 = 0x3b9d_5e2a_c710_f846;
    let (mut accepted, mut commits, mut unobserved, mut no_reading, mut stale, mut read_refused) =
        (0, 0, 0, 0, 0, 0);
    for run in 0..300 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        // Every revision's record at the commit, frozen from then on.
        let mut frozen: Vec<(String, (Value, Value, Value))> = Vec::new();
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
            let outcome = serde_json::to_value(
                game.dispatch_outcome_json(event, payload)
                    .unwrap_or_else(|fatal| panic!("{at}: fatal {}", fatal.message)),
            )
            .unwrap();
            // The two series move together: committed, open and current revision
            // numbers agree before the event, so one rule fires when the other does.
            let dc_fires = current(&before, "dc").is_none_or(|revision| open(&before, &revision));
            let du_fires = current(&before, "du").is_none_or(|revision| open(&before, &revision));
            assert_eq!(dc_fires, du_fires, "{at}: the twins drifted apart");
            let has_sample = before["reading_streams"]["s"]["occurrences"]
                .as_array()
                .is_some_and(|list| !list.is_empty());
            let expected = match event {
                "decide" if dc_fires && !has_sample => Some(("evaluation", "expression")),
                "decide" if dc_fires && !observed(&before, "e") => {
                    Some(("evaluation", "unobserved_evidence"))
                }
                "read_dc" if current(&before, "dc").is_some() => Some(("evaluation", "expression")),
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
                match code {
                    "unobserved_evidence" => unobserved += 1,
                    _ if event == "read_dc" => read_refused += 1,
                    _ => no_reading += 1,
                }
                continue;
            }
            assert_eq!(outcome["outcome"], "accepted", "{at}: {outcome}");
            accepted += 1;
            let shown = snapshot(&game);
            if matches!(event, "decide" | "decide_plain") && dc_fires {
                commits += 1;
                let dc = current(&shown, "dc").unwrap();
                let du = current(&shown, "du").unwrap();
                assert_eq!(
                    dc.split('@').nth(1),
                    du.split('@').nth(1),
                    "{at}: the twins' revision numbers differ"
                );
                let (grounds, lineage, retained) = record(&shown, &dc);
                assert_eq!(
                    (grounds.clone(), lineage.clone(), retained.clone()),
                    record(&shown, &du),
                    "{at}: the citing record differs from its using twin"
                );
                assert_eq!(
                    shown["commitment_bases"][&dc]["value"],
                    Value::Null,
                    "{at}: a citing decision has a value"
                );
                assert!(
                    shown["commitment_bases"][&du]["value"].is_number(),
                    "{at}: the using twin has no value"
                );
                if event == "decide" {
                    let evidence = grounds["evidence"].as_array().unwrap();
                    assert!(
                        evidence.iter().any(|item| item == "e"),
                        "{at}: e not cited: {grounds}"
                    );
                    assert!(
                        evidence
                            .iter()
                            .any(|item| item.as_str().unwrap().starts_with("s@")),
                        "{at}: the reading not cited: {grounds}"
                    );
                    assert!(
                        grounds["caveats"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|item| item == "gaps"),
                        "{at}: gaps not retained in the grounds: {grounds}"
                    );
                }
                frozen.push((dc.clone(), (grounds, lineage, retained)));
                frozen.push((du.clone(), record(&shown, &du)));
            }
            for (revision, then) in &frozen {
                assert_eq!(&record(&shown, revision), then, "{at}: {revision} changed");
            }
            if event == "read_du" && current(&before, "du").is_some() {
                let du = current(&before, "du").unwrap();
                assert_eq!(
                    shown["values"]["w"], shown["commitment_bases"][&du]["value"],
                    "{at}: latest(du)"
                );
            }
            // Stale through cited grounds, exactly as the twin.
            if let Some(dc) = current(&shown, "dc") {
                let withdrawn: Vec<String> = shown["withdrawals"]
                    .as_array()
                    .map(|list| {
                        list.iter()
                            .map(|item| item["evidence"].as_str().unwrap().to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                let grounds = shown["commitment_grounds"][&dc]["evidence"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|item| item.as_str().unwrap().to_string())
                    .collect::<Vec<_>>();
                let expected = if grounds.iter().any(|name| withdrawn.contains(name)) {
                    1.0
                } else {
                    0.0
                };
                assert_eq!(
                    shown["bindings"]["view"]["stale_dc"].as_f64(),
                    Some(expected),
                    "{at}: rests_on_withdrawn(dc)"
                );
                assert_eq!(
                    shown["bindings"]["view"]["stale_du"].as_f64(),
                    Some(expected),
                    "{at}: rests_on_withdrawn(du)"
                );
                if expected == 1.0 {
                    stale += 1;
                }
            }
            before = shown;
        }
    }
    eprintln!("{accepted} accepted events, {commits} twin commits, {unobserved} refused for unobserved cited evidence, {no_reading} refused for a citation without a reading, {read_refused} reads of a citing decision refused, {stale} readings stale through cited grounds");
    assert!(
        commits > 500 && unobserved > 150 && no_reading > 200 && read_refused > 300 && stale > 1000,
        "{commits} / {unobserved} / {no_reading} / {read_refused} / {stale}"
    );
}
