//! D100: a value read of a reading stream (`latest`, `history_at`,
//! `history_count` or a fold), and `has_sample`, reads only the stream's
//! readings and its selection, never its observation or `observed` records.
//! Restore previously bounded those reads as any other read of the stream,
//! which pulled `Owner::StreamRecords(s)` (including guards that later joined
//! `s`'s observation records on qualification or withdrawal) into any reading,
//! commitment basis, state grounds or `because` citation reading `s` through
//! one of those forms. Records reads (`observed`, `carries`, `carries_any` and
//! `because` selectors) still read `s`'s observation records, and a state's
//! lineage holding a reading still receives late qualification guards on what
//! it lists. See spec/caveat-save-0.1.md ("Restore validation", "Changes") and
//! spec/caveat-lineage-compaction-0.1.md ("Meaning", "Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn answer(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

fn plays_and_restores(source: &str, events: &[(&str, &str)]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    for (index, (event, payload)) in events.iter().enumerate() {
        let outcome = answer(&mut live, event, payload);
        assert_eq!(
            outcome["outcome"], "accepted",
            "{index}: {event}: {outcome}"
        );
        for run in &mut restored {
            assert_eq!(answer(run, event, payload), outcome, "{index}: {event}");
        }
        let save = live.save_json().unwrap();
        let again = ReactiveSession::restore_json(source, &save)
            .unwrap_or_else(|error| panic!("{index}: {event}: {error}"));
        assert_eq!(again.save_json().unwrap(), save, "{index}: {event}");
        restored.push(again);
    }
    let save = live.save_json().unwrap();
    for run in &restored {
        assert_eq!(run.save_json().unwrap(), save);
    }
    serde_json::from_str(&save).unwrap()
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted {save}"),
    }
}

fn no_rule_carries(place: &str, history: &str) -> String {
    format!("{place}: no rule or declaration of this program carries records of {history} into it")
}

fn state_grounds<'v>(save: &'v mut Value, state: &str) -> &'v mut Value {
    let cell = &mut save["states"][state];
    if cell.get("grounds").is_none() {
        cell["grounds"] = cell["lineage"].clone();
    }
    &mut cell["grounds"]
}

fn add_marker(provenance: &mut Value, marker: Value) {
    let mut departed = provenance["departed"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    departed.push(marker);
    departed.sort_by(|x, y| x["history"].as_str().cmp(&y["history"].as_str()));
    provenance["departed"] = json!(departed);
}

fn with_kept(save: &Value, state: &str, caveat: &str) -> Value {
    let mut forged = save.clone();
    let held = state_grounds(&mut forged, state);
    for field in ["caveats", "kept"] {
        let mut names: Vec<String> = held[field]
            .as_array()
            .into_iter()
            .flatten()
            .map(|name| name.as_str().unwrap().to_string())
            .collect();
        if !names.iter().any(|name| name == caveat) {
            names.push(caveat.to_string());
            names.sort();
            held[field] = json!(names);
        }
    }
    forged
}

/// `b` is sampled under a guard reading `latest(c)`, so `observation of b@1`
/// lists `c@1`; `q` qualifies `latest(c)` under a guard on `latest(a)`, so
/// `a` joins `observation of b@1` (`StreamRecords(b)`) but never `b@1`'s
/// reading (`Stream(b)`). `read_expr` reads `b` via one of the five value/
/// sample forms into stream `d`, state `s` (lineage + grounds), commitment `r`,
/// and a state `t` citing `latest(d)`.
fn stream_read_program(read_expr: &str) -> String {
    format!(
        "claim seen;
evidence ga from \"ga\";
evidence gb from \"gb\";
evidence gc from \"gc\";
evidence gd from \"gd\";
caveat k consequence low;
readings a from ga window 1;
readings b from gb window 4;
readings c from gc window 4;
readings d from gd window 4;
decisions r limit 4;
state s = 0;
state t = 0;
state obs = 0;
state car = 0;
fn add(acc, v) = acc + v;
event la v min 0 max 10;
event lb v min 0 max 10;
event lc v min 0 max 10;
event ld v min 0 max 10;
event q;
event use_s;
event use_r;
event cite_d;
event check_obs;
event check_car;
on la sample a = v supports seen;
on lc sample c = v supports seen;
on lb when latest(c) > 0 sample b = v supports seen;
on q when latest(a) < 5 qualify latest(c) with k;
on ld sample d = {read_expr} supports seen;
on use_s set s = {read_expr};
on use_r commit r because enough using {read_expr};
on cite_d set t = latest(d) because latest(d);
on check_obs when observed(reading(b, 1)) set obs = 1;
on check_car when carries(latest(b), k) set car = 1;
"
    )
}

/// None of the five value/sample reads (`latest`, `history_at`,
/// `history_count`, `fold_history`, `has_sample`) brings `StreamRecords(b)`
/// into a downstream reading, commitment basis/grounds, state grounds, or
/// cited grounds; records reads (`observed(latest(b))`, `carries(latest(b), k)`)
/// still legitimately carry `a` from `observation of b@1`.
#[test]
fn value_and_has_sample_reads_exclude_stream_observation_records() {
    let cases = [
        ("latest", "latest(b)"),
        ("history_at", "history_at(b, 0)"),
        ("history_count", "history_count(b)"),
        ("fold", "fold_history(b, 0, add)"),
        ("has_sample", "if(has_sample(b), 1, 0)"),
    ];
    for (label, expr) in cases {
        let source = stream_read_program(expr);
        let save = plays_and_restores(
            &source,
            &[
                ("lc", r#"{"v":1}"#),
                ("la", r#"{"v":1}"#),
                ("lb", r#"{"v":2}"#),
                ("q", "{}"),
                ("ld", r#"{"v":3}"#),
                ("use_s", "{}"),
                ("use_r", "{}"),
                ("cite_d", "{}"),
                ("check_obs", "{}"),
                ("check_car", "{}"),
                ("la", r#"{"v":2}"#),
            ],
        );

        // `observation of b@1`, `obs` (read `observed(latest(b))`) and `car`
        // (read `carries(latest(b), k)`) legitimately carry `a`'s marker.
        let marker = save["observation_qualifications"]["b@1"]["departed"][0].clone();
        assert_eq!(marker["history"], "a", "{label}: {save}");
        assert_eq!(
            save["states"]["obs"]["lineage"]["departed"][0]["history"], "a",
            "{label}: {save}"
        );
        assert_eq!(
            save["states"]["car"]["lineage"]["departed"][0]["history"], "a",
            "{label}: {save}"
        );

        // Reading `d@1` and commitment `r@1` basis cannot carry `a`'s marker.
        for (pointer, place) in [
            ("/reading_streams/d/occurrences/0/provenance", "reading d@1"),
            ("/commitment_bases/r@1/provenance", "commitment r@1"),
        ] {
            let mut forged = save.clone();
            add_marker(forged.pointer_mut(pointer).unwrap(), marker.clone());
            refused(&source, &forged, &no_rule_carries(place, "a"));
        }

        // Commitment `r@1` grounds cannot hold `a`'s marker beyond its basis.
        let mut forged_rg = save.clone();
        add_marker(&mut forged_rg["commitment_grounds"]["r@1"], marker.clone());
        refused(
            &source,
            &forged_rg,
            "commitment r@1 grounds hold departed a records outside its lineage",
        );

        // Forged marker in `state s grounds` (even when `state s` lineage
        // holds `a`, `state s grounds` itself cannot carry `a`).
        let mut forged_s = save.clone();
        let grounds_s = state_grounds(&mut forged_s, "s");
        add_marker(grounds_s, marker.clone());
        add_marker(&mut forged_s["states"]["s"]["lineage"], marker.clone());
        refused(&source, &forged_s, &no_rule_carries("state s grounds", "a"));

        // Forged marker in `state t grounds` beyond its lineage: `cite_d`
        // cites `latest(d)` whose sample read `b` through a value/sample form,
        // so `Stream(d)` does not carry `a` and `t`'s grounds cannot hold `a`.
        let mut forged_t = save.clone();
        add_marker(state_grounds(&mut forged_t, "t"), marker.clone());
        refused(
            &source,
            &forged_t,
            "state t grounds hold departed a records outside its lineage",
        );
    }
}

/// Same D100 narrowing in a program without a window (D061): a forged record
/// name `a@1` in `reading d@1`, `commitment r@1`, or `state s grounds` is
/// refused, while `observation of b@1` and `observed(latest(b))` still carry
/// `a@1`.
#[test]
fn windowless_stream_value_read_refuses_observation_record_name() {
    let source = "claim seen;
evidence ga from \"ga\";
evidence gb from \"gb\";
evidence gc from \"gc\";
evidence gd from \"gd\";
caveat k consequence low;
readings a from ga limit 4;
readings b from gb limit 4;
readings c from gc limit 4;
readings d from gd limit 4;
decisions r limit 4;
state s = 0;
state obs = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event lc v min 0 max 10;
event ld;
event q;
event use_s;
event use_r;
event check_obs;
on la sample a = v supports seen;
on lc sample c = v supports seen;
on lb when latest(c) > 0 sample b = v supports seen;
on q when latest(a) < 5 qualify latest(c) with k;
on ld sample d = latest(b) supports seen;
on use_s set s = latest(b);
on use_r commit r because enough using latest(b);
on check_obs when observed(reading(b, 1)) set obs = 1;
";
    let save = plays_and_restores(
        source,
        &[
            ("lc", r#"{"v":1}"#),
            ("la", r#"{"v":1}"#),
            ("lb", r#"{"v":2}"#),
            ("q", "{}"),
            ("ld", "{}"),
            ("use_s", "{}"),
            ("use_r", "{}"),
            ("check_obs", "{}"),
        ],
    );
    assert!(save["observation_qualifications"]["b@1"]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .any(|name| name == "a@1"));
    assert!(save["states"]["obs"]["lineage"]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .any(|name| name == "a@1"));

    let mut forged_d = save.clone();
    forged_d["reading_streams"]["d"]["occurrences"][0]["provenance"]["evidence"]
        .as_array_mut()
        .unwrap()
        .push(json!("a@1"));
    refused(source, &forged_d, &no_rule_carries("reading d@1", "a"));

    let mut forged_s = save.clone();
    forged_s["states"]["s"]["lineage"]["evidence"]
        .as_array_mut()
        .unwrap()
        .push(json!("a@1"));
    state_grounds(&mut forged_s, "s")["evidence"]
        .as_array_mut()
        .unwrap()
        .push(json!("a@1"));
    refused(source, &forged_s, &no_rule_carries("state s grounds", "a"));
}

/// D100 also closes the kept take-back caveat route through a cited reading
/// sampled from another stream: `shaky` qualifies only `a`'s readings, and `a`
/// only joins `observation of b@1`, never `b@1`'s reading or `d@1`'s reading.
/// So citing `latest(d)` does not let `t`'s grounds keep `shaky` beyond `t`'s
/// lineage.
#[test]
fn cited_reading_does_not_bring_kept_caveat_from_another_streams_observation_record() {
    let source = "claim seen;
evidence ga from \"ga\";
evidence gb from \"gb\";
evidence gc from \"gc\";
evidence gd from \"gd\";
caveat k consequence low;
caveat shaky consequence low;
readings a from ga window 1;
readings b from gb window 4;
readings c from gc window 4;
readings d from gd window 4;
state t = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event lc v min 0 max 10;
event ld;
event q;
event qa;
event ua;
event cite_d;
on la sample a = v supports seen;
on lc sample c = v supports seen;
on lb when latest(c) > 0 sample b = v supports seen;
on q when latest(a) < 5 qualify latest(c) with k;
on qa qualify latest(a) with shaky;
on ua unqualify latest(a) with shaky;
on ld sample d = latest(b) supports seen;
on cite_d set t = latest(d) because latest(d);
";
    let save = plays_and_restores(
        source,
        &[
            ("lc", r#"{"v":1}"#),
            ("la", r#"{"v":1}"#),
            ("qa", "{}"),
            ("lb", r#"{"v":2}"#),
            ("q", "{}"),
            ("ua", "{}"),
            ("ld", "{}"),
            ("cite_d", "{}"),
            ("la", r#"{"v":2}"#),
        ],
    );
    let forged = with_kept(&save, "t", "shaky");
    refused(
        source,
        &forged,
        "state t grounds include caveat shaky outside its lineage",
    );
}

/// `has_sample(b)` still carries `b`'s selection qualifications (a skipped
/// sample's guard) both before and after `b` has a reading, and a live state
/// set from `latest(b)` before `q` runs still receives `q`'s guard in its
/// lineage (not its grounds) via `joined`.
#[test]
fn has_sample_selection_and_live_state_lineage_still_restore() {
    let source = "claim seen;
evidence ga from \"ga\";
evidence gb from \"gb\";
evidence gc from \"gc\";
evidence ge from \"ge\";
caveat k consequence low;
readings a from ga window 1;
readings b from gb window 4;
readings c from gc window 4;
readings e from ge window 1;
state s = 0;
state hs_before = 0;
state hs_after = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event lc v min 0 max 10;
event le v min 0 max 10;
event q;
event set_s;
event check_before;
event check_after;
on la sample a = v supports seen;
on lc sample c = v supports seen;
on le sample e = v supports seen;
on lb when latest(e) > 5 and latest(c) > 0 sample b = v supports seen;
on q when latest(a) < 5 qualify latest(c) with k;
on set_s set s = latest(b);
on check_before set hs_before = if(has_sample(b), 1, 0);
on check_after set hs_after = if(has_sample(b), 1, 0);
";
    let save = plays_and_restores(
        source,
        &[
            ("lc", r#"{"v":1}"#),
            ("la", r#"{"v":1}"#),
            ("le", r#"{"v":1}"#),
            // Skipped sample puts `e@1` into `b`'s selection qualifications.
            ("lb", r#"{"v":2}"#),
            ("check_before", "{}"),
            // Now sample `b` succeeds, clearing selection and taking `e@2` + `c@1`.
            ("le", r#"{"v":8}"#),
            ("lb", r#"{"v":2}"#),
            ("check_after", "{}"),
            // `s` reads `latest(b)` before `q`, so `s` holds `c@1` and `q`
            // joins `a@1` into `s`'s lineage (not `s`'s grounds).
            ("set_s", "{}"),
            ("q", "{}"),
            ("la", r#"{"v":2}"#),
            ("le", r#"{"v":9}"#),
        ],
    );
    assert_eq!(
        save["states"]["hs_before"]["lineage"]["departed"][0]["history"],
        "e"
    );
    assert_eq!(
        save["states"]["s"]["lineage"]["departed"][0]["history"],
        "a"
    );
}
