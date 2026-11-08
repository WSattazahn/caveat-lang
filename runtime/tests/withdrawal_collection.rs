//! Proposed withdrawal collection: exact roots retain dependencies; only
//! unreachable retired groups join the ordinary same-event departure batch.
use caveat_runtime::reactive::{ArchiveItem, ReactiveSession};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const SELF: &str = include_str!("../../experiments/departure-gate/withdrawal-fixtures/self.cav");
const MUTUAL: &str =
    include_str!("../../experiments/departure-gate/withdrawal-fixtures/mutual.cav");

fn send(game: &mut ReactiveSession, event: &str) -> Value {
    game.apply(event, &BTreeMap::new())
        .unwrap_or_else(|error| panic!("{event}: {error}"));
    serde_json::to_value(game.snapshot()).unwrap()
}

fn records(archive: &[ArchiveItem]) -> Vec<Value> {
    archive
        .iter()
        .filter_map(|item| match item {
            ArchiveItem::Record(entry) => Some(serde_json::to_value(entry).unwrap()),
            ArchiveItem::Provenance(_) => None,
        })
        .collect()
}

fn names(archive: &[ArchiveItem]) -> Vec<String> {
    records(archive)
        .iter()
        .map(|entry| entry["record"].as_str().unwrap().into())
        .collect()
}

fn anchored(archive: &[ArchiveItem], source_id: &str) {
    let all = serde_json::to_value(archive).unwrap();
    for entry in records(archive) {
        let leaves: Vec<_> = all
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| {
                item["operation"] == "record"
                    && item["record"] == entry["record"]
                    && item["history"] == entry["history"]
                    && item["departed_at"] == entry["departed_at"]
                    && item["source_id"] == source_id
            })
            .collect();
        assert_eq!(
            leaves.len(),
            1,
            "each archived record has one scoped leaf: {entry}"
        );
    }
}

#[test]
fn isolated_self_and_mutual_cycles_depart_with_both_directions_archived() {
    for (source, width) in [(SELF, 1), (MUTUAL, 2)] {
        let mut game = ReactiveSession::from_source(source).unwrap();
        let mut drained = ReactiveSession::from_source(source).unwrap();
        let mut archive = Vec::new();
        for _ in 0..16 {
            send(&mut game, "cycle");
            send(&mut drained, "cycle");
            archive.extend(drained.drain_archive());
            let restored =
                ReactiveSession::restore_json(source, &game.save_json().unwrap()).unwrap();
            assert_eq!(game.snapshot(), restored.snapshot());
        }
        assert_eq!(game.save_json().unwrap(), drained.save_json().unwrap());
        assert_eq!(
            serde_json::to_value(&archive).unwrap(),
            serde_json::to_value(game.drain_archive()).unwrap()
        );
        assert_eq!(records(&archive).len(), 15 * width);
        let snapshot = serde_json::to_value(game.snapshot()).unwrap();
        anchored(&archive, snapshot["source_id"].as_str().unwrap());
        for record in records(&archive) {
            let name = record["record"].as_str().unwrap();
            let reason = record["withdrawal"]["because"].as_str().unwrap();
            if width == 1 {
                assert_eq!(name, reason);
            } else {
                let counterpart = if name.starts_with('a') {
                    name.replacen('a', "b", 1)
                } else {
                    name.replacen('b', "a", 1)
                };
                assert_eq!(reason, counterpart);
                assert!(records(&archive)
                    .iter()
                    .any(|entry| entry["record"] == counterpart
                        && entry["withdrawal"]["because"] == name));
            }
        }
        assert_eq!(
            game.snapshot().retired.len(),
            width,
            "only original declarations remain retired"
        );
    }
}

#[test]
fn final_independent_root_release_collects_and_failures_roll_back() {
    let source =
        include_str!("../../experiments/departure-gate/withdrawal-fixtures/self-own-ground.cav");
    let mut game = ReactiveSession::from_source(source).unwrap();
    for event in ["cycle", "hold", "cycle", "release_first"] {
        send(&mut game, event);
    }
    assert!(game.snapshot().retired.contains_key("a@2"));
    let before = game.save_json().unwrap();
    let snapshot = game.snapshot();
    #[cfg(feature = "collector-metrics")]
    let metrics = game.withdrawal_collection_metrics();
    for refused in ["refuse", "fail"] {
        assert!(game.apply(refused, &BTreeMap::new()).is_err());
        assert_eq!(game.save_json().unwrap(), before);
        assert_eq!(game.snapshot(), snapshot);
        assert!(game.drain_archive().is_empty());
        #[cfg(feature = "collector-metrics")]
        assert_eq!(game.withdrawal_collection_metrics(), metrics);
    }
    let mut restored = ReactiveSession::restore_json(source, &before).unwrap();
    send(&mut game, "release_last");
    send(&mut restored, "release_last");
    assert_eq!(game.save_json().unwrap(), restored.save_json().unwrap());
    assert_eq!(names(&game.drain_archive()), ["a@2"]);
}

#[test]
fn later_readable_history_preserves_the_reason_until_the_holder_departs() {
    let source = format!("{}\nevent replace; event clear; on replace sample kept = 0 supports seen; on clear set result = 0;",
        include_str!("../../experiments/departure-gate/withdrawal-fixtures/later-history-read.cav"));
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["initialize", "cycle", "probe"] {
        send(&mut game, event);
    }
    assert!(game.snapshot().retired.contains_key("a@2"));
    let snapshot = serde_json::to_value(game.snapshot()).unwrap();
    assert_eq!(snapshot["bindings"]["hud"]["withdrawn"], true, "{snapshot}");
    send(&mut game, "clear");
    send(&mut game, "replace");
    let archive = game.drain_archive();
    assert_eq!(names(&archive), ["a@2", "kept@1"]);
    let entries = records(&archive);
    assert!(entries[1]["reading"]["provenance"]["evidence"]
        .as_array()
        .unwrap()
        .contains(&json!("a@2")));
    ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
}

#[test]
fn ordinary_departing_holder_releases_a_cycle_in_the_same_batch() {
    let source = r#"
claim seen; evidence a from "a"; renewable a window 1;
evidence s from "s"; readings r from s window 1;
event create; event advance;
on create renew a; on create reveal a supports seen; on create withdraw a because a;
on create when withdrawn(a) sample r = 1 supports seen;
on advance renew a; on advance sample r = 2 supports seen;
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    send(&mut game, "create");
    send(&mut game, "advance");
    assert_eq!(names(&game.drain_archive()), ["a@2", "r@1"]);
    ReactiveSession::restore_json(source, &game.save_json().unwrap()).unwrap();
}

#[test]
fn reachable_chain_survives_and_unchanged_events_do_not_trace_it() {
    let source = format!(
        "{}\nevent idle;",
        include_str!("../../experiments/departure-gate/withdrawal-fixtures/reachable-chain.cav")
    );
    let mut game = ReactiveSession::from_source(&source).unwrap();
    send(&mut game, "initialize");
    for _ in 0..24 {
        send(&mut game, "cycle");
    }
    assert!(game.snapshot().retired.contains_key("a@2"));
    assert!(game.drain_archive().is_empty());
    for _ in 0..3 {
        send(&mut game, "idle");
        #[cfg(feature = "collector-metrics")]
        {
            let metrics = game.withdrawal_collection_metrics();
            assert_eq!(metrics.vertices_visited, 0);
            assert_eq!(metrics.edges_visited, 0);
            assert_eq!(metrics.candidates_examined, 0);
        }
    }
}

/// `anyall`: four groups of a reading window, a decision series and a state
/// read through bindings, under a journal window, with no withdrawal. Every
/// retired reading stays pinned, in one retained region per group that
/// grows with the session; the base collector's work per event grew with
/// the square of the session's length here. With `chain`, every `look` also
/// extends the reachable withdrawal chain of `reachable-chain.cav`.
fn anyall(chain: bool) -> String {
    let mut source = String::from(
        "claim seen;\nevidence glimpse from \"a glimpse\";\njournal window 3;\n\
         event look v min 0 max 10;\nevent decide;\n",
    );
    for group in 0..4 {
        source += &format!(
            "readings s{group} from glimpse window 3;\ndecisions d{group} limit 120;\n\
             state t{group} = 0 min 0 max 1000;\n\
             on look when not has_sample(s{group}) sample s{group} = v supports seen;\n\
             on look when has_sample(s{group}) and latest(s{group}) >= 0 sample s{group} = v supports seen;\n\
             on look set t{group} = latest(s{group}) because latest(s{group});\n\
             on decide when committed(d{group}) and not reopened(d{group}) reopen d{group} because latest(s{group});\n\
             on decide commit d{group} because enough using history_count(s{group});\n\
             bind h.c{group} = history_count(s{group});\nbind h.t{group} = t{group};\n"
        );
    }
    if chain {
        source += "evidence a from \"a\"; renewable a window 1;\n\
                   evidence b from \"b\"; renewable b window 1;\n\
                   event initialize; on initialize reveal b supports seen;\n\
                   on look renew a; on look reveal a supports seen; on look withdraw a because b;\n\
                   on look renew b; on look reveal b supports seen; on look withdraw b because a;\n";
    }
    source
}

/// `anyall`'s script: three looks and a decision, `events` in all, after
/// the chain's `initialize`.
fn anyall_script(events: usize, chain: bool) -> Vec<(&'static str, BTreeMap<String, f64>)> {
    let start = chain.then(|| ("initialize", BTreeMap::new()));
    start
        .into_iter()
        .chain((0..events).map(|n| match n % 4 {
            3 => ("decide", BTreeMap::new()),
            _ => ("look", BTreeMap::from([("v".into(), (n * 7 % 11) as f64)])),
        }))
        .collect()
}

#[test]
fn a_cycle_that_keeps_the_chain_rooted_does_not_trace_it_again() {
    // Each cycle retires the two records at the chain's live end. The one
    // retired first is held by the other, which a current record holds, so
    // both stay reached without the older chain being visited again.
    let chain =
        include_str!("../../experiments/departure-gate/withdrawal-fixtures/reachable-chain.cav");
    let cycles = std::iter::once(("initialize", BTreeMap::new()))
        .chain(std::iter::repeat_n(("cycle", BTreeMap::new()), 24))
        .collect::<Vec<_>>();
    for (source, script) in [
        (chain.to_string(), cycles),
        (anyall(true), anyall_script(96, true)),
    ] {
        let mut game = ReactiveSession::from_source(&source).unwrap();
        for (n, (event, payload)) in script.iter().enumerate() {
            game.apply(event, payload)
                .unwrap_or_else(|error| panic!("{event}: {error}"));
            #[cfg(feature = "collector-metrics")]
            if n > 3 {
                let metrics = game.withdrawal_collection_metrics();
                assert!(metrics.vertices_visited <= 2, "event {n}: {metrics:?}");
                assert_eq!(metrics.collected, 0, "event {n}");
            }
            #[cfg(not(feature = "collector-metrics"))]
            let _ = n;
        }
        assert!(game.snapshot().retired.contains_key("a@2"));
        assert!(names(&game.drain_archive())
            .iter()
            .all(|name| !name.starts_with("a@") && !name.starts_with("b@")));
        // Restore seeds every retired reason; the next accepted event reaches
        // the whole chain again from its root and still collects nothing.
        let mut restored =
            ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
        let (event, payload) = &script[script.len() - 4];
        restored.apply(event, payload).unwrap();
        game.apply(event, payload).unwrap();
        let archive = names(&game.drain_archive());
        assert_eq!(names(&restored.drain_archive()), archive);
        assert!(archive
            .iter()
            .all(|name| !name.starts_with("a@") && !name.starts_with("b@")));
        assert_eq!(restored.save_json().unwrap(), game.save_json().unwrap());
    }
}

/// The work an event's collection does depends on what the event changed,
/// not on how long the session has run: the last decision round of a
/// session four times longer visits no more. The base collector traced the
/// whole retained graph here at every event.
#[cfg(feature = "collector-metrics")]
#[test]
fn anyall_collection_work_per_event_does_not_grow_with_the_session() {
    for chain in [false, true] {
        let source = anyall(chain);
        let tail = |events: usize| {
            let mut game = ReactiveSession::from_source(&source).unwrap();
            let mut metrics = Vec::new();
            for (event, payload) in anyall_script(events, chain) {
                game.apply(event, &payload)
                    .unwrap_or_else(|error| panic!("{event}: {error}"));
                metrics.push(game.withdrawal_collection_metrics());
            }
            metrics.split_off(metrics.len() - 4)
        };
        let (short, long) = (tail(40), tail(160));
        for (short, long) in short.iter().zip(&long) {
            assert_eq!(
                (long.vertices_visited, long.edges_visited),
                (short.vertices_visited, short.edges_visited),
                "chain {chain}: {short:?} then {long:?}"
            );
            assert_eq!(long.collected, 0, "chain {chain}");
        }
    }
}

#[test]
fn authentic_older_saves_rebuild_without_collecting_until_an_accepted_event() {
    let fixtures: Vec<Value> = [
        include_str!("fixtures/collector-baseline-f5ec829.json"),
        include_str!("fixtures/collector-baseline-rc15.json"),
    ]
    .into_iter()
    .map(|text| serde_json::from_str(text).unwrap())
    .collect();
    for case in fixtures
        .iter()
        .flat_map(|fixture| fixture["cases"].as_array().unwrap())
    {
        let source = case["source"].as_str().unwrap();
        let save = case["save"].as_str().unwrap();
        let mut game = ReactiveSession::restore_json(source, save).unwrap();
        // These saves predate source digests: saving again adds the digest
        // and marks their events as not recorded, and changes nothing else.
        let mut expected = serde_json::from_str::<Value>(save).unwrap();
        expected["source_sha256"] = game.snapshot().source_sha256.into();
        if expected["sequence"] != 0 {
            expected["source_unrecorded_through"] = expected["sequence"].clone();
        }
        assert_eq!(
            serde_json::from_str::<Value>(&game.save_json().unwrap()).unwrap(),
            expected
        );
        assert!(game.drain_archive().is_empty());
        assert!(game.apply("collector_refuse", &BTreeMap::new()).is_err());
        assert!(game.drain_archive().is_empty());
        assert!(game.snapshot().retired.contains_key("a@2"));
        send(&mut game, "collector_noop");
        let expected = if case["name"] == "self" {
            vec!["a@2", "a@3"]
        } else {
            vec!["a@2", "a@3", "b@2", "b@3"]
        };
        assert_eq!(names(&game.drain_archive()), expected);
        ReactiveSession::restore_json(source, &game.save_json().unwrap()).unwrap();
    }
}

#[test]
fn pending_target_pins_survive_until_the_last_scheduled_application() {
    let source = format!("{SELF}\ncaveat stale consequence material; event later; event tick dt min 0 max 0.1;\non later qualify a with stale after 0.1; on later qualify a with stale after 0.2;");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["cycle", "later", "cycle"] {
        send(&mut game, event);
    }
    let mut restored = ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    for session in [&mut game, &mut restored] {
        session
            .apply("tick", &BTreeMap::from([("dt".into(), 0.1)]))
            .unwrap();
        assert!(session.snapshot().retired.contains_key("a@2"));
        session
            .apply("tick", &BTreeMap::from([("dt".into(), 0.1)]))
            .unwrap();
        assert_eq!(names(&session.drain_archive()), ["a@2"]);
        ReactiveSession::restore_json(&source, &session.save_json().unwrap()).unwrap();
    }
    assert_eq!(game.save_json().unwrap(), restored.save_json().unwrap());
}

#[test]
fn pending_guard_lineage_and_its_transfer_are_conservative_roots() {
    let source = format!("{SELF}\nevidence b from \"b\"; caveat stale consequence material; state holder = 0;\nevent later; event tick dt min 0 max 0.1; event clear;\non later reveal b; on later set holder = qualified(1,b);\non later when qualified(1,a) == 1 qualify b with stale after 0.1;\non later when qualified(1,a) == 1 qualify b with stale after 0.2;\non clear set holder = 0;");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["cycle", "later", "cycle"] {
        send(&mut game, event);
    }
    assert!(game.snapshot().retired.contains_key("a@2"));
    for _ in 0..2 {
        game.apply("tick", &BTreeMap::from([("dt".into(), 0.1)]))
            .unwrap();
        assert!(game.snapshot().retired.contains_key("a@2"));
        ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    }
    send(&mut game, "clear");
    assert_eq!(names(&game.drain_archive()), ["a@2"]);
}

#[test]
fn permission_grants_and_historical_commitment_bases_keep_exact_reasons() {
    for basis in ["using qualified(1,a)", "permitted by a"] {
        let cycles = SELF.replace(
            "on cycle withdraw a because a;",
            "event retract; on retract withdraw a because a;",
        );
        let source = format!("{cycles}\ndecisions trust limit 4; journal window 1; event decide; event replace;\non decide commit trust because enough {basis};\non replace reopen trust because a; on replace commit trust because enough;");
        let mut game = ReactiveSession::from_source(&source).unwrap();
        for event in ["cycle", "decide", "retract", "cycle", "replace", "cycle"] {
            send(&mut game, event);
        }
        assert!(game.snapshot().retired.contains_key("a@2"), "{basis}");
        ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    }
}

#[test]
fn transferred_skipped_guard_lineage_vetoes_only_additional_collection() {
    let source = format!("{SELF}\nstate result = 0; event skip; event clear;\non skip when qualified(0,a) > 0 set result = 1;\non clear set result = 0;");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["cycle", "skip", "cycle"] {
        send(&mut game, event);
    }
    assert!(game.snapshot().retired.contains_key("a@2"));
    send(&mut game, "clear");
    assert_eq!(names(&game.drain_archive()), ["a@2"]);
}

#[test]
fn record_owned_qualification_inside_a_cycle_is_not_an_external_root() {
    let source = r#"
claim seen; evidence a from "a"; renewable a window 1; evidence b from "b"; renewable b window 1;
event cycle;
on cycle renew a; on cycle renew b; on cycle reveal b supports seen;
on cycle when qualified(1,b) == 1 reveal a supports seen;
on cycle withdraw a because b; on cycle withdraw b because a;
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    send(&mut game, "cycle");
    send(&mut game, "cycle");
    let archive = game.drain_archive();
    assert_eq!(names(&archive), ["a@2", "b@2"]);
    assert!(
        records(&archive)[0]["qualifications"]["observation"]["evidence"]
            .as_array()
            .unwrap()
            .contains(&json!("b@2"))
    );
    ReactiveSession::restore_json(source, &game.save_json().unwrap()).unwrap();
}

#[test]
fn a_mutual_group_waits_for_two_roots_and_archives_atomically_after_retry() {
    let source =
        include_str!("../../experiments/departure-gate/withdrawal-fixtures/mutual-own-ground.cav");
    let mut game = ReactiveSession::from_source(source).unwrap();
    for event in ["cycle", "cycle", "hold", "cycle", "release_first"] {
        send(&mut game, event);
    }
    // a@2/b@2 already departed; refusal must preserve that undrained handover.
    let before = game.save_json().unwrap();
    let mut expected = game.clone();
    for event in ["refuse", "fail"] {
        assert!(game.apply(event, &BTreeMap::new()).is_err());
        assert_eq!(game.save_json().unwrap(), before);
    }
    send(&mut game, "release_last");
    send(&mut expected, "release_last");
    assert_eq!(game.save_json().unwrap(), expected.save_json().unwrap());
    assert_eq!(
        serde_json::to_value(game.drain_archive()).unwrap(),
        serde_json::to_value(expected.drain_archive()).unwrap()
    );
    assert!(!game.snapshot().retired.contains_key("a@3"));
    assert!(!game.snapshot().retired.contains_key("b@3"));
}

#[test]
fn failed_due_event_keeps_pending_target_and_guard_indexes() {
    let source = format!("{SELF}\ncaveat stale consequence material; state divisor = 1; event later; event tick dt min 0 max 0.1; event fail dt min 0 max 0.1;\non later when qualified(1,a) == 1 qualify a with stale after 0.1;\non fail set divisor = 0; bind hud.guard = 1 / divisor;");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["cycle", "later", "cycle"] {
        send(&mut game, event);
    }
    let before = game.save_json().unwrap();
    assert!(game
        .apply("fail", &BTreeMap::from([("dt".into(), 0.1)]))
        .is_err());
    assert_eq!(game.save_json().unwrap(), before);
    assert!(game.drain_archive().is_empty());
    game.apply("tick", &BTreeMap::from([("dt".into(), 0.1)]))
        .unwrap();
    assert_eq!(names(&game.drain_archive()), ["a@2"]);
}

#[test]
fn procedure_arguments_and_skipped_procedure_guards_keep_their_actual_holders() {
    for rule in [
        "on mark call keep(qualified(1,a));",
        "on mark when qualified(0,a) > 0 call keep(1);",
    ] {
        let source = format!("{SELF}\nstate result = 0; event mark; event clear;\nproc keep(x) {{ set result = x; }};\n{rule}\non clear set result = 0;");
        let mut game = ReactiveSession::from_source(&source).unwrap();
        for event in ["cycle", "mark", "cycle"] {
            send(&mut game, event);
        }
        assert!(game.snapshot().retired.contains_key("a@2"), "{rule}");
        let saved: Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
        if rule.contains("when") {
            assert!(saved["states"]["result"]["grounds"]["evidence"].is_null());
        }
        send(&mut game, "clear");
        assert_eq!(names(&game.drain_archive()), ["a@2"]);
    }
}

#[test]
fn stream_and_series_selection_holders_veto_until_successful_overwrite() {
    for (declaration, skip, clear, retained) in [
        (
            "evidence s from \"s\"; readings r from s window 1;",
            "sample r = 1 supports seen",
            "sample r = 0 supports seen",
            false,
        ),
        (
            "decisions d limit 3;",
            "commit d because enough",
            "commit d because enough",
            false,
        ),
    ] {
        let source = format!("{SELF}\n{declaration} event skip; event clear;\non skip when qualified(0,a) > 0 {skip}; on clear {clear};");
        let mut game = ReactiveSession::from_source(&source).unwrap();
        for event in ["cycle", "skip", "cycle"] {
            send(&mut game, event);
        }
        assert!(game.snapshot().retired.contains_key("a@2"));
        send(&mut game, "clear");
        // Both successful operations replace their selection holder; a skipped
        // selection must not become the next record's own evidence.
        assert_eq!(game.snapshot().retired.contains_key("a@2"), retained);
        ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    }
}

#[test]
fn declaration_qualifiers_are_roots_and_successful_predicate_clear_releases() {
    for (extra, mark, clear, retained) in [
        (
            "evidence b from \"b\";",
            "when qualified(1,a) == 1 reveal b",
            "reveal b",
            true,
        ),
        (
            "caveat fog consequence material; budget 4;",
            "when qualified(1,a) == 1 examine fog cost 1",
            "examine fog cost 1",
            true,
        ),
        (
            "evidence b from \"b\";",
            "when qualified(0,a) > 0 reveal b",
            "reveal b",
            false,
        ),
    ] {
        let source =
            format!("{SELF}\n{extra} event mark; event clear; on mark {mark}; on clear {clear};");
        let mut game = ReactiveSession::from_source(&source).unwrap();
        for event in ["cycle", "mark", "cycle"] {
            send(&mut game, event);
        }
        assert!(game.snapshot().retired.contains_key("a@2"), "{mark}");
        send(&mut game, "clear");
        assert_eq!(
            game.snapshot().retired.contains_key("a@2"),
            retained,
            "{mark}"
        );
        ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    }
}

#[test]
fn reopened_commitment_keeps_reason_graph_and_current_status() {
    let source = format!("{SELF}\ndecisions d limit 3; event decide; event reopen; on decide commit d because enough;\non reopen when qualified(1,a) == 1 reopen d because a;\nbind hud.open = reopened(d);");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["cycle", "decide", "reopen", "cycle"] {
        send(&mut game, event);
    }
    assert!(game.snapshot().retired.contains_key("a@2"));
    let snapshot = serde_json::to_value(game.snapshot()).unwrap();
    assert_eq!(snapshot["bindings"]["hud"]["open"], true);
    assert!(snapshot["relations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|edge| edge["from"] == "a@2" && edge["relation"] == "reopens"));
    let restored = ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    assert_eq!(game.snapshot(), restored.snapshot());
}

#[test]
fn journal_retirement_releases_only_its_pin_while_other_owners_retain() {
    let source = format!("{SELF}\ndecisions d limit 4; journal window 1; state held = 0; event decide; event replace; event clear;\non decide set held = qualified(1,a); on decide commit d because enough using held;\non replace reopen d because a; on replace commit d because enough; on clear set held = 0;");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["cycle", "decide", "cycle", "replace", "clear"] {
        send(&mut game, event);
    }
    assert!(
        game.snapshot().retired.contains_key("a@2"),
        "retained historical basis still references the exact withdrawn subject"
    );
    assert!(names(&game.drain_archive())
        .iter()
        .any(|name| name.starts_with("journal@")));
    ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
}

#[test]
fn explicit_citations_keep_exact_old_membership_and_refuse_current_alias_substitution() {
    for (citation, accepted) in [("held", true), ("qualified(1,a)", false)] {
        let source = format!("{SELF}\nstate held = 0; state enabled = 0; event hold; event show; event clear;\non hold set held = qualified(1,a); on show set enabled = 1;\non clear set enabled = 0; on clear set held = 0;\nbind hud.cited = held when enabled == 1 because {citation};");
        let mut game = ReactiveSession::from_source(&source).unwrap();
        for event in ["cycle", "hold", "cycle"] {
            send(&mut game, event);
        }
        let before = game.save_json().unwrap();
        let outcome =
            serde_json::to_value(game.dispatch_outcome_json("show", "{}").unwrap()).unwrap();
        if accepted {
            assert_eq!(outcome["outcome"], "accepted", "{outcome}");
        } else {
            assert_eq!(outcome["outcome"], "rejected", "{outcome}");
            assert_eq!(outcome["origin"], "evaluation");
            assert_eq!(outcome["code"], "ungrounded_citation");
            assert_eq!(game.save_json().unwrap(), before);
        }
        assert!(game.snapshot().retired.contains_key("a@2"));
        send(&mut game, "clear");
        assert_eq!(names(&game.drain_archive()), ["a@2"]);
        ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    }
}

#[test]
fn a_skipped_uncreated_commitment_is_a_real_owner_on_restore_and_clear() {
    let source = format!("{SELF}\nevent mark; event finish;\non mark when qualified(0,a) > 0 commit trust because enough;\non finish commit trust because enough;");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["cycle", "mark", "cycle"] {
        send(&mut game, event);
    }
    assert!(game.snapshot().retired.contains_key("a@2"));
    let mut restored = ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    for session in [&mut game, &mut restored] {
        assert!(session.drain_archive().is_empty());
        send(session, "finish");
        assert_eq!(names(&session.drain_archive()), ["a@2"]);
    }
    assert_eq!(game.save_json().unwrap(), restored.save_json().unwrap());
}

// Collection searches from what an event lost: a reason that lost a pin
// (`pending_target_pins_survive_until_the_last_scheduled_application`, and
// `shared_reason_multiplicity_does_not_hide_a_scheduled_target_pin` in
// collector_index_compaction.rs) or a holder
// (`transferred_skipped_guard_lineage_vetoes_only_additional_collection`), or
// retired (`isolated_self_and_mutual_cycles_depart_with_both_directions_archived`),
// a reason a record departing in `D0` held
// (`later_readable_history_preserves_the_reason_until_the_holder_departs`), and
// after a restore every retired reason
// (`authentic_older_saves_rebuild_without_collecting_until_an_accepted_event`).
// Each of those tests, and each of the five tests below, fails on its own
// assertions if the search misses that loss.

#[test]
fn a_withdrawal_added_after_its_reason_lost_a_pin_collects_when_the_reason_retires() {
    // The due qualification releases a@2's pin before the withdrawal makes
    // it a reason, so only its retirement later in the event marks it.
    let source = format!("{SELF}\ncaveat stale consequence material; event fresh; event later; event tick dt min 0 max 0.1;\non fresh renew a; on fresh reveal a supports seen; on later qualify a with stale after 0.1;\non tick withdraw a because a; on tick renew a;");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["fresh", "later"] {
        send(&mut game, event);
    }
    let mut restored = ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    for session in [&mut game, &mut restored] {
        session
            .apply("tick", &BTreeMap::from([("dt".into(), 0.1)]))
            .unwrap();
        assert_eq!(names(&session.drain_archive()), ["a@2"]);
    }
    assert_eq!(game.save_json().unwrap(), restored.save_json().unwrap());
}

#[test]
fn a_holder_that_retires_leaves_the_reasons_it_held_collectible() {
    // b@2 holds a@2 and a@2 holds b@2. a@2 retires first and stays, held by
    // the current b@2; when b@2 retires nothing current holds either.
    let source = r#"
claim seen; evidence a from "a"; renewable a window 1; evidence b from "b"; renewable b window 1;
event make; event retire_a; event retire_b;
on make renew a; on make renew b; on make reveal a supports seen; on make reveal b supports seen;
on make withdraw a because b; on make withdraw b because a;
on retire_a renew a; on retire_b renew b;
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    send(&mut game, "make");
    send(&mut game, "retire_a");
    assert!(game.drain_archive().is_empty());
    assert!(game.snapshot().retired.contains_key("a@2"));
    send(&mut game, "retire_b");
    assert_eq!(names(&game.drain_archive()), ["a@2", "b@2"]);
    ReactiveSession::restore_json(source, &game.save_json().unwrap()).unwrap();
}

#[test]
fn a_reason_whose_last_current_holder_departs_ordinarily_joins_that_batch() {
    // b@2's qualifier names a@2, and a scheduled qualification pins b@2
    // after it retires. When that applies, b@2 departs in D0 and nothing
    // else happens to a@2.
    let source = format!("{SELF}\nevidence b from \"b\"; renewable b window 1; caveat stale consequence material;\nevent mark; event later; event advance; event tick dt min 0 max 0.1;\non mark renew b; on mark when qualified(1,a) == 1 reveal b supports seen;\non later qualify b with stale after 0.1; on advance renew b;");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["cycle", "mark", "later", "cycle", "advance"] {
        send(&mut game, event);
    }
    assert!(game.drain_archive().is_empty());
    assert!(game.snapshot().retired.contains_key("a@2"));
    assert!(game.snapshot().retired.contains_key("b@2"));
    game.apply("tick", &BTreeMap::from([("dt".into(), 0.1)]))
        .unwrap();
    assert_eq!(names(&game.drain_archive()), ["a@2", "b@2"]);
    ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
}

#[test]
fn a_wide_ordinary_holder_still_seeds_the_reason_it_held() {
    // As above, but b@2's qualifier also reads c and d, so b@2 references
    // more records than there are withdrawal reasons (a@2 and a@3), and the
    // reasons it held are found from the reasons' side, by their holders.
    // A search that finds none from that side never starts from a@2, and
    // a@2 stays after b@2 departs.
    let source = format!("{SELF}\nevidence b from \"b\"; renewable b window 1; evidence c from \"c\"; evidence d from \"d\";\ncaveat stale consequence material;\nevent mark; event later; event advance; event tick dt min 0 max 0.1;\non mark reveal c supports seen; on mark reveal d supports seen; on mark renew b;\non mark when qualified(1,a) == 1 and qualified(1,c) >= 0 and qualified(1,d) >= 0 reveal b supports seen;\non later qualify b with stale after 0.1; on advance renew b;");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    for event in ["cycle", "mark", "later", "cycle", "advance"] {
        send(&mut game, event);
    }
    assert!(game.drain_archive().is_empty());
    assert!(game.snapshot().retired.contains_key("a@2"));
    assert!(game.snapshot().retired.contains_key("b@2"));
    let mut restored = ReactiveSession::restore_json(&source, &game.save_json().unwrap()).unwrap();
    for session in [&mut game, &mut restored] {
        session
            .apply("tick", &BTreeMap::from([("dt".into(), 0.1)]))
            .unwrap();
        assert_eq!(names(&session.drain_archive()), ["a@2", "b@2"]);
    }
    assert_eq!(game.save_json().unwrap(), restored.save_json().unwrap());
}

#[test]
fn cutting_a_chain_from_its_current_root_collects_all_of_it_at_once() {
    // Self-withdrawn records linked by observation qualifiers,
    // a@4 -> b@3 -> a@3 -> b@2 -> a@2, rooted by the current a@4 and b@3.
    // `cut` renews a and b with no link, so a@4 and b@3 retire and the whole
    // chain departs in that event. Only a@4 and b@3 are seeds; the rest is
    // reached from them through their own references, which a search that
    // leaves a record only through its holders never follows.
    let source = r#"
claim seen; evidence a from "a"; renewable a window 1; evidence b from "b"; renewable b window 1;
event start; event link; event cut;
on start renew a; on start reveal a supports seen; on start withdraw a because a;
on link renew b; on link when qualified(1,a) == 1 reveal b supports seen; on link withdraw b because b;
on link renew a; on link when qualified(1,b) == 1 reveal a supports seen; on link withdraw a because a;
on cut renew a; on cut reveal a supports seen; on cut renew b; on cut reveal b supports seen;
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    for event in ["start", "link", "link"] {
        send(&mut game, event);
    }
    assert!(game.drain_archive().is_empty());
    let mut restored = ReactiveSession::restore_json(source, &game.save_json().unwrap()).unwrap();
    for session in [&mut game, &mut restored] {
        send(session, "cut");
        assert_eq!(
            names(&session.drain_archive()),
            ["a@2", "a@3", "a@4", "b@2", "b@3"]
        );
    }
    assert_eq!(game.save_json().unwrap(), restored.save_json().unwrap());
}
