// Foreclosure 0.1 under random events: an irreversible commit is refused as
// evaluation/caveated_irreversible exactly when its grounds carry a caveat of
// consequence material or worse that it does not retain, so it never rests on
// withdrawn evidence; once it is made, every commit of what it forecloses (a
// series or a plain commitment) is refused as evaluation/foreclosed naming
// d@1, before any other check, within an event too; reopening the decision,
// by hand or by trigger, keeps it current, keeps the foreclosure and never
// makes room for a second revision; a refusal changes nothing; a restored
// session is the one that saved; and the same program without the two words
// runs in lockstep, step for step, differing only by the two new refusals.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const HEAD: &str = r#"
caveat heavy consequence material;
caveat shaky consequence high;
caveat light consequence low;
claim ok;
evidence e from "tool";
evidence f from "operator";
evidence g from "probe";
evidence r from "reviewer";
evidence h from "hunch";
heavy qualifies f;
light qualifies g;
readings s from e limit 40;
readings t from f limit 40;
readings u from g limit 40;
"#;

const IRREVERSIBLE: &str =
    "decisions d limit 1 irreversible forecloses rs, plain reopened by u opposing ok;\n";
const PLAIN: &str = "decisions d limit 1 reopened by u opposing ok;\n";

const TAIL: &str = r#"
decisions rs limit 30;
decisions other limit 30;
event read_s x min 0 max 3;
event read_t x min 0 max 3;
event read_u x min 0 max 3;
event read_u_opp x min 0 max 3;
event see_r;
event see_h;
event flag_h;
event unflag_h;
event retract_s;
event d_s;
event d_h;
event d_t;
event d_t_ret;
event d_u;
event rs_it;
event rs_then_d;
event d_then_rs;
event plain_it;
event other_it;
event reopen_rs;
event reopen_d;
event reopen_other;
on read_s sample s = x supports ok;
on read_t sample t = x supports ok;
on read_u sample u = x supports ok;
on read_u_opp sample u = x opposes ok;
on see_r reveal r;
on see_h reveal h supports ok;
on flag_h when observed(h) qualify h with shaky;
on unflag_h when observed(h) unqualify h with shaky;
on retract_s when has_sample(s) and not observed(r) reveal r;
on retract_s when has_sample(s) withdraw latest(s) because r;
on d_s commit d because enough using latest(s);
on d_h commit d because enough using qualified(1, h);
on d_t commit d because enough using latest(t);
on d_t_ret commit d because enough using latest(t) retaining heavy;
on d_u commit d because enough using latest(u);
on rs_it commit rs because enough using 1;
on rs_then_d commit rs because enough using 2;
on rs_then_d commit d because enough using latest(u);
on d_then_rs commit d because enough using latest(u);
on d_then_rs commit rs because enough using 3;
on plain_it commit plain because enough;
on other_it commit other because enough using 4;
on reopen_rs when committed(rs) and not reopened(rs) and observed(r) reopen rs because r;
on reopen_d when committed(d) and not reopened(d) and observed(r) reopen d because r;
on reopen_other when committed(other) and not reopened(other) and observed(r) reopen other because r;
"#;

const EVENTS: [(&str, &str); 30] = [
    ("read_s", r#"{"x": 1}"#),
    ("read_s", r#"{"x": 2}"#),
    ("read_t", r#"{"x": 1}"#),
    ("read_u", r#"{"x": 1}"#),
    ("read_u_opp", r#"{"x": 0}"#),
    ("see_r", "{}"),
    ("see_r", "{}"),
    ("see_h", "{}"),
    ("flag_h", "{}"),
    ("flag_h", "{}"),
    ("unflag_h", "{}"),
    ("retract_s", "{}"),
    ("retract_s", "{}"),
    ("d_s", "{}"),
    ("d_s", "{}"),
    ("d_h", "{}"),
    ("d_h", "{}"),
    ("d_t", "{}"),
    ("d_t", "{}"),
    ("d_t_ret", "{}"),
    ("d_u", "{}"),
    ("rs_it", "{}"),
    ("rs_it", "{}"),
    ("rs_then_d", "{}"),
    ("d_then_rs", "{}"),
    ("plain_it", "{}"),
    ("other_it", "{}"),
    ("reopen_rs", "{}"),
    ("reopen_d", "{}"),
    ("reopen_other", "{}"),
];

fn anonymous(mut shown: Value) -> Value {
    if let Some(object) = shown.as_object_mut() {
        object.remove("source_id");
        object.remove("source_sha256");
        if let Some(inner) = object.get_mut("snapshot") {
            *inner = anonymous(inner.take());
        }
    }
    shown
}

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

/// The snapshot with the field the words add taken out, so the twins compare.
fn comparable(mut shown: Value) -> Value {
    shown = anonymous(shown);
    if let Some(object) = shown.as_object_mut() {
        object.remove("irreversible");
        if let Some(inner) = object.get_mut("snapshot") {
            *inner = comparable(inner.take());
        }
    }
    shown
}

fn current(shown: &Value, series: &str) -> Option<String> {
    shown["decision_series"][series]["current"]
        .as_str()
        .map(str::to_string)
}

fn revisions(shown: &Value, series: &str) -> usize {
    shown["decision_series"][series]["revisions"]
        .as_array()
        .map(Vec::len)
        .unwrap_or(0)
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

fn retained(shown: &Value, revision: &str) -> Vec<String> {
    shown["commitments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["action"] == revision)
        .and_then(|item| item["retained"].as_array())
        .map(|list| {
            list.iter()
                .map(|c| c.as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn latest_reading(shown: &Value, stream: &str) -> Option<String> {
    shown["reading_streams"][stream]["occurrences"]
        .as_array()
        .and_then(|list| list.last())
        .map(|item| item["id"].as_str().unwrap().to_string())
}

fn withdrawn(shown: &Value, occurrence: &str) -> bool {
    shown["withdrawals"]
        .as_array()
        .is_some_and(|list| list.iter().any(|item| item["evidence"] == occurrence))
}

fn observed(shown: &Value, evidence: &str) -> bool {
    shown["relations"].as_array().unwrap().iter().any(|edge| {
        edge["from"] == evidence
            && (edge["relation"] == "supports" || edge["relation"] == "opposes")
    })
}

fn consequence(caveat: &str) -> &'static str {
    match caveat {
        "heavy" | "withdrawn" => "material",
        "shaky" => "high",
        "light" => "low",
        other => panic!("unknown caveat {other}"),
    }
}

fn is_material(caveat: &str) -> bool {
    matches!(consequence(caveat), "material" | "high" | "catastrophic")
}

/// A series' own refusal of a plain commit, before the words are read: full,
/// then in force.
fn series_refusal(before: &Value, series: &str) -> Option<(&'static str, &'static str)> {
    if revisions(before, series) >= 30 {
        return Some(("limit", "history_limit"));
    }
    match current(before, series) {
        Some(revision) if !open(before, &revision) => Some(("evaluation", "decision_in_force")),
        _ => None,
    }
}

/// What a commit of `d` resting on `stream` is refused as, with `retaining`
/// named: the series' room, the expression, then the unretained caveats of
/// consequence material or worse. A `None` stream has empty grounds.
fn d_refusal(
    before: &Value,
    flagged: bool,
    stream: Option<&str>,
    retaining: &[&str],
) -> (Option<(&'static str, &'static str)>, Vec<&'static str>) {
    if revisions(before, "d") >= 1 {
        return (Some(("limit", "history_limit")), vec![]);
    }
    let Some(stream) = stream else {
        return (None, vec![]);
    };
    let mut caveats: Vec<&'static str> = Vec::new();
    if stream == "h" {
        if !observed(before, "h") {
            return (Some(("evaluation", "unobserved_evidence")), vec![]);
        }
        if flagged {
            caveats.push("shaky");
        }
    } else {
        let Some(latest) = latest_reading(before, stream) else {
            return (Some(("evaluation", "expression")), vec![]);
        };
        if stream == "s" && withdrawn(before, &latest) {
            caveats.push("withdrawn");
        }
    }
    match stream {
        "s" | "h" => {}
        "t" => caveats.push("heavy"),
        "u" => caveats.push("light"),
        _ => unreachable!(),
    }
    let unretained: Vec<&'static str> = caveats
        .into_iter()
        .filter(|caveat| is_material(caveat) && !retaining.contains(caveat))
        .collect();
    if unretained.is_empty() {
        (None, vec![])
    } else {
        (Some(("evaluation", "caveated_irreversible")), unretained)
    }
}

#[test]
fn both_programs_load() {
    ReactiveSession::from_source(&format!("{HEAD}{IRREVERSIBLE}{TAIL}")).unwrap();
    ReactiveSession::from_source(&format!("{HEAD}{PLAIN}{TAIL}")).unwrap();
}

#[test]
fn an_irreversible_decision_names_its_caveats_and_closes_what_it_forecloses() {
    let source = format!("{HEAD}{IRREVERSIBLE}{TAIL}");
    let twin_source = format!("{HEAD}{PLAIN}{TAIL}");
    let mut seed: u64 = 0x7c1e_4a93_d5b0_28f7;
    let (
        mut accepted,
        mut d_commits,
        mut caveated,
        mut on_withdrawn,
        mut foreclosed,
        mut in_event,
        mut reopened_d,
        mut closed_after_reopen,
    ) = (0, 0, 0, 0, 0, 0, 0, 0);
    for run in 0..300 {
        let mut game = ReactiveSession::from_source(&source).unwrap();
        let mut twin = ReactiveSession::from_source(&twin_source).unwrap();
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        let mut d_record: Option<Value> = None;
        // Whether shaky qualifies h now: the last of qualify and unqualify
        // that ran (the relation stays in the graph once taken back).
        let mut flagged = false;
        for step in 0..60 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                trail.push("save");
                let saved = game.save_json().unwrap();
                assert!(
                    !saved.contains("irreversible") && !saved.contains("forecloses"),
                    "run {run} step {step}: the save holds the words: {trail:?}"
                );
                game = ReactiveSession::restore_json(&source, &saved).unwrap_or_else(|error| {
                    panic!("run {run} step {step} restore: {error}: {trail:?}")
                });
                assert_eq!(
                    snapshot(&game),
                    before,
                    "run {run} step {step} after restore: {trail:?}"
                );
                let twin_saved = twin.save_json().unwrap();
                twin = ReactiveSession::restore_json(&twin_source, &twin_saved).unwrap_or_else(
                    |error| panic!("run {run} step {step} twin restore: {error}: {trail:?}"),
                );
                continue;
            }
            let (event, payload) = EVENTS[pick];
            trail.push(event);
            let at = format!("run {run} step {step} after {event} {payload}: {trail:?}");
            let save_before = game.save_json().unwrap();
            let twin_before = twin.save_json().unwrap();
            let outcome = serde_json::to_value(
                game.dispatch_outcome_json(event, payload)
                    .unwrap_or_else(|fatal| panic!("{at}: fatal {}", fatal.message)),
            )
            .unwrap();
            let d_made = revisions(&before, "d") >= 1;
            // The expected refusal, in the runtime's order of checks: a
            // foreclosed action first, then the series' own room, then the
            // grounds.
            let (expected, unretained): (Option<(&str, &str)>, Vec<&str>) = match event {
                "d_s" => d_refusal(&before, flagged, Some("s"), &[]),
                "d_h" => d_refusal(&before, flagged, Some("h"), &[]),
                "d_t" => d_refusal(&before, flagged, Some("t"), &[]),
                "d_t_ret" => d_refusal(&before, flagged, Some("t"), &["heavy"]),
                "d_u" => d_refusal(&before, flagged, Some("u"), &[]),
                "rs_it" if d_made => (Some(("evaluation", "foreclosed")), vec![]),
                "rs_it" => (series_refusal(&before, "rs"), vec![]),
                "plain_it" if d_made => (Some(("evaluation", "foreclosed")), vec![]),
                "plain_it" if before["commitment_bases"]["plain"].is_object() => {
                    (Some(("evaluation", "already_committed")), vec![])
                }
                "plain_it" => (None, vec![]),
                "other_it" => (series_refusal(&before, "other"), vec![]),
                // rs commits first: foreclosed, then its own room, then d.
                "rs_then_d" if d_made => (Some(("evaluation", "foreclosed")), vec![]),
                "rs_then_d" => match series_refusal(&before, "rs") {
                    Some(refusal) => (Some(refusal), vec![]),
                    None => d_refusal(&before, flagged, Some("u"), &[]),
                },
                // d commits first: its own checks, then rs is closed by it.
                "d_then_rs" => match d_refusal(&before, flagged, Some("u"), &[]) {
                    (Some(refusal), unretained) => (Some(refusal), unretained),
                    (None, _) => (Some(("evaluation", "foreclosed")), vec![]),
                },
                _ => (None, vec![]),
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
                assert_eq!(
                    snapshot(&game),
                    before,
                    "{at}: a refusal changed the snapshot"
                );
                let message = outcome["message"].as_str().unwrap();
                match code {
                    "foreclosed" => {
                        foreclosed += 1;
                        // The twin, with nothing foreclosed, runs the event on
                        // its own rules; then it is put back.
                        let twin_outcome = serde_json::to_value(
                            twin.dispatch_outcome_json(event, payload).unwrap(),
                        )
                        .unwrap();
                        let twins_own = match event {
                            "plain_it" if before["commitment_bases"]["plain"].is_object() => {
                                Some(("evaluation", "already_committed"))
                            }
                            "plain_it" => None,
                            // d is made, so the twin's d has no room either.
                            "rs_then_d" => {
                                series_refusal(&before, "rs").or(Some(("limit", "history_limit")))
                            }
                            _ => series_refusal(&before, "rs"),
                        };
                        match twins_own {
                            Some((origin, code)) => assert_eq!(
                                (
                                    twin_outcome["origin"].as_str(),
                                    twin_outcome["code"].as_str()
                                ),
                                (Some(origin), Some(code)),
                                "{at}: twin {twin_outcome}"
                            ),
                            None => assert_eq!(
                                twin_outcome["outcome"], "accepted",
                                "{at}: twin {twin_outcome}"
                            ),
                        }
                        twin = ReactiveSession::restore_json(&twin_source, &twin_before).unwrap();
                        let action = if event == "plain_it" { "plain" } else { "rs" };
                        assert!(
                            message.ends_with(&format!("{action} is foreclosed by d@1")),
                            "{at}: {message}"
                        );
                        if matches!(event, "rs_then_d" | "d_then_rs") {
                            in_event += 1;
                        }
                        if current(&before, "d").is_some_and(|d| open(&before, &d)) {
                            closed_after_reopen += 1;
                        }
                    }
                    "caveated_irreversible" => {
                        caveated += 1;
                        let listed = unretained
                            .iter()
                            .map(|caveat| format!("{caveat} ({})", consequence(caveat)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        assert!(
                            message.ends_with(&format!(
                                "irreversible d rests on {listed}, which its commit does not retain"
                            )),
                            "{at}: expected {listed}: {message}"
                        );
                        if unretained.contains(&"withdrawn") {
                            on_withdrawn += 1;
                        }
                        // The twin, which reads no caveat, accepts the same event.
                        let twin_outcome = serde_json::to_value(
                            twin.dispatch_outcome_json(event, payload).unwrap(),
                        )
                        .unwrap();
                        assert_eq!(
                            twin_outcome["outcome"], "accepted",
                            "{at}: twin {twin_outcome}"
                        );
                        // Put the twin back: the refusal left the original as it was.
                        twin = ReactiveSession::restore_json(&twin_source, &twin_before).unwrap();
                    }
                    _ => {}
                }
                if code != "caveated_irreversible" && code != "foreclosed" {
                    // A refusal the words did not add: the twin refuses alike.
                    let twin_outcome =
                        serde_json::to_value(twin.dispatch_outcome_json(event, payload).unwrap())
                            .unwrap();
                    assert_eq!(
                        (
                            twin_outcome["origin"].as_str(),
                            twin_outcome["code"].as_str()
                        ),
                        (Some(origin), Some(code)),
                        "{at}: twin {twin_outcome}"
                    );
                }
                continue;
            }
            assert_eq!(outcome["outcome"], "accepted", "{at}: {outcome}");
            accepted += 1;
            if observed(&before, "h") {
                match event {
                    "flag_h" => flagged = true,
                    "unflag_h" => flagged = false,
                    _ => {}
                }
            }
            let shown = snapshot(&game);
            // The twin accepts the same event into the same state.
            let twin_outcome =
                serde_json::to_value(twin.dispatch_outcome_json(event, payload).unwrap()).unwrap();
            assert_eq!(
                comparable(twin_outcome.clone()),
                comparable(outcome.clone()),
                "{at}: the twin's outcome differs"
            );
            assert_eq!(
                comparable(snapshot(&twin)),
                comparable(shown.clone()),
                "{at}: the twin's snapshot differs"
            );
            assert_eq!(
                shown["irreversible"],
                serde_json::json!({"d": ["rs", "plain"]}),
                "{at}: irreversible"
            );
            assert!(revisions(&shown, "d") <= 1, "{at}: d has a second revision");
            // d, once made, stays current, open or not, with its record frozen.
            if let Some(record) = &d_record {
                assert_eq!(
                    current(&shown, "d").as_deref(),
                    Some("d@1"),
                    "{at}: d moved"
                );
                assert_eq!(
                    &shown["commitment_grounds"]["d@1"], record,
                    "{at}: d@1's grounds changed"
                );
                assert_eq!(
                    revisions(&shown, "rs"),
                    revisions(&before, "rs"),
                    "{at}: rs committed after d"
                );
                assert_eq!(
                    shown["commitment_bases"]["plain"].is_object(),
                    before["commitment_bases"]["plain"].is_object(),
                    "{at}: plain committed after d"
                );
                if event == "reopen_d" && open(&shown, "d@1") && !open(&before, "d@1") {
                    reopened_d += 1;
                }
            } else if current(&shown, "d").is_some() {
                d_commits += 1;
                let grounds = shown["commitment_grounds"]["d@1"].clone();
                let kept = retained(&shown, "d@1");
                for caveat in grounds["caveats"].as_array().unwrap() {
                    let caveat = caveat.as_str().unwrap();
                    assert!(
                        !is_material(caveat) || kept.iter().any(|k| k == caveat),
                        "{at}: d@1 rests on {caveat} unretained: {grounds}"
                    );
                    assert_ne!(caveat, "withdrawn", "{at}: d@1 rests on withdrawn evidence");
                }
                for evidence in grounds["evidence"].as_array().unwrap() {
                    assert!(
                        !withdrawn(&shown, evidence.as_str().unwrap()),
                        "{at}: d@1 was made on a withdrawn reading: {grounds}"
                    );
                }
                if event == "rs_then_d" {
                    let rs_now = current(&shown, "rs").unwrap();
                    let rs_seq = shown["decision_series"]["rs"]["revisions"]
                        .as_array()
                        .unwrap()
                        .last()
                        .unwrap()["sequence"]
                        .as_u64()
                        .unwrap();
                    let d_seq = shown["decision_series"]["d"]["revisions"][0]["sequence"]
                        .as_u64()
                        .unwrap();
                    assert!(rs_seq <= d_seq, "{at}: {rs_now} after d@1");
                }
                d_record = Some(grounds);
            }
            // Every event's save restores, to the same snapshot (D021).
            let saved_game = game.save_json().unwrap();
            let restored_game = ReactiveSession::restore_json(&source, &saved_game)
                .unwrap_or_else(|error| panic!("{at}: restore after the event: {error}"));
            assert_eq!(
                restored_game.save_json().unwrap(),
                saved_game,
                "{at}: save round trip"
            );
            assert_eq!(
                snapshot(&restored_game),
                snapshot(&game),
                "{at}: snapshot after restore"
            );
            // Every event's save restores, to the same snapshot (D021).
            let saved_twin = twin.save_json().unwrap();
            let restored_twin = ReactiveSession::restore_json(&twin_source, &saved_twin)
                .unwrap_or_else(|error| panic!("{at}: restore after the event: {error}"));
            assert_eq!(
                restored_twin.save_json().unwrap(),
                saved_twin,
                "{at}: save round trip"
            );
            assert_eq!(
                snapshot(&restored_twin),
                snapshot(&twin),
                "{at}: snapshot after restore"
            );
            before = shown;
        }
    }
    eprintln!("{accepted} accepted events, {d_commits} irreversible commits, {caveated} refused as caveated_irreversible ({on_withdrawn} on withdrawn evidence), {foreclosed} refused as foreclosed ({in_event} within an event, {closed_after_reopen} with d reopened), {reopened_d} reopenings of d");
    assert!(
        d_commits > 200
            && caveated > 80
            && on_withdrawn > 10
            && foreclosed > 800
            && in_event > 200
            && closed_after_reopen > 400
            && reopened_d > 50,
        "{d_commits} / {caveated} / {on_withdrawn} / {foreclosed} / {in_event} / {closed_after_reopen} / {reopened_d}"
    );
}
