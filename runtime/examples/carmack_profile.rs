//! Where the time per event goes, by event kind, for one program and one
//! fixed event sequence. Each event runs through the same transaction a host
//! reaches with `dispatchView`; the fixed phase boundaries of the
//! event-phase-profile build split it into rules, record keeping and the
//! transaction itself, and the view's JSON is timed on its own. After each
//! repetition the session is saved both ways and restored. See
//! bench/carmack/README.md.
//!
//!   cargo run --release --features event-phase-profile --example carmack_profile -- PROGRAM EVENTS_JSON REPS
//!
//! EVENTS_JSON is an array of [event, payload] pairs, replayed in order on a
//! fresh session each repetition.
use caveat_runtime::reactive::{
    DispatchViewResult, EventPhaseProfile, ReactiveSession, EVENT_PHASE_COUNT, EVENT_PHASE_NAMES,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Instant;

/// Each phase belongs to one share. Rules: the program's handlers and its
/// bindings, including the lineage each value carries as it is computed (that
/// is inline and has no boundary of its own). Record: withdrawal collection,
/// compaction, the archive and its provenance. Transaction: validating the
/// payload's bounds, copying the session so a refusal leaves it untouched,
/// and committing or rolling back.
const SHARES: [(&str, &[&str]); 3] = [
    ("rules", &["evaluation", "binding_evaluation"]),
    (
        "record",
        &[
            "collection",
            "compaction",
            "archive_construction_and_removal",
            "final_settlement",
        ],
    ),
    (
        "transaction",
        &[
            "transaction_preparation",
            "commit_cleanup",
            "rollback_cleanup",
            "apply_wrapper",
        ],
    ),
];

#[derive(Default)]
struct Kind {
    dispatch_ns: Vec<u64>,
    view_json_ns: Vec<u64>,
    phase_ns: [u64; EVENT_PHASE_COUNT],
    /// Dispatch time outside every phase: payload parsing and member
    /// resolution, and building the view record.
    unphased_ns: u64,
    rejected: usize,
}

fn median(values: &mut [u64]) -> u64 {
    values.sort_unstable();
    values.get(values.len() / 2).copied().unwrap_or(0)
}

fn p95(values: &mut [u64]) -> u64 {
    values.sort_unstable();
    if values.is_empty() {
        return 0;
    }
    values[((values.len() * 95).div_ceil(100)).saturating_sub(1)]
}

fn timed<T>(run: impl FnOnce() -> T) -> (T, u64) {
    let start = Instant::now();
    let value = run();
    (value, start.elapsed().as_nanos() as u64)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 4, "carmack_profile PROGRAM EVENTS_JSON REPS");
    let source = std::fs::read_to_string(&args[1]).unwrap();
    let events: Vec<(String, Value)> =
        serde_json::from_str(&std::fs::read_to_string(&args[2]).unwrap()).unwrap();
    let reps: usize = args[3].parse().unwrap();
    assert!(reps > 0 && !events.is_empty());
    let payloads: Vec<(String, String)> = events
        .iter()
        .map(|(event, payload)| (event.clone(), payload.to_string()))
        .collect();

    let mut kinds: BTreeMap<String, Kind> = BTreeMap::new();
    let (mut load, mut save_state, mut save_compact) = (Vec::new(), Vec::new(), Vec::new());
    let (mut restore_state, mut restore_compact) = (Vec::new(), Vec::new());
    let (mut state_bytes, mut compact_bytes) = (0, 0);
    for _ in 0..reps {
        let (session, ns) = timed(|| ReactiveSession::from_source(&source).unwrap());
        let mut session = session;
        load.push(ns);
        for (event, payload) in &payloads {
            let kind = kinds.entry(event.clone()).or_default();
            ReactiveSession::begin_event_phase_profile();
            let start = Instant::now();
            let outcome = session.dispatch_view_outcome_json(event, payload).unwrap();
            let dispatch_ns = start.elapsed().as_nanos() as u64;
            let profile: EventPhaseProfile = ReactiveSession::take_event_phase_profile();
            let accepted = matches!(outcome.result, DispatchViewResult::Accepted { .. });
            let (text, view_ns) = timed(|| serde_json::to_string(&outcome).unwrap());
            std::hint::black_box(text);
            if !accepted {
                kind.rejected += 1;
            }
            let phased: u64 = profile.phase_ns.iter().sum();
            for (total, ns) in kind.phase_ns.iter_mut().zip(profile.phase_ns) {
                *total += ns;
            }
            kind.unphased_ns += dispatch_ns.saturating_sub(phased);
            kind.dispatch_ns.push(dispatch_ns);
            kind.view_json_ns.push(view_ns);
        }
        let (state, ns) = timed(|| session.save_json().unwrap());
        save_state.push(ns);
        let (compact, ns) = timed(|| session.save_compact_positional_json().unwrap());
        save_compact.push(ns);
        state_bytes = state.len();
        compact_bytes = compact.len();
        let (restored, ns) = timed(|| ReactiveSession::restore_json(&source, &state).unwrap());
        restore_state.push(ns);
        drop(restored);
        let (restored, ns) = timed(|| ReactiveSession::restore_json(&source, &compact).unwrap());
        restore_compact.push(ns);
        drop(restored);
    }

    let mut report = serde_json::Map::new();
    let (mut all_dispatch, mut all_view) = (0u64, 0u64);
    let mut all_phase = [0u64; EVENT_PHASE_COUNT];
    let mut all_unphased = 0u64;
    let mut all_samples = Vec::new();
    for (name, kind) in &mut kinds {
        let samples = kind.dispatch_ns.len() as u64;
        let dispatch_total: u64 = kind.dispatch_ns.iter().sum();
        let view_total: u64 = kind.view_json_ns.iter().sum();
        all_dispatch += dispatch_total;
        all_view += view_total;
        all_unphased += kind.unphased_ns;
        all_samples.extend(
            kind.dispatch_ns
                .iter()
                .zip(&kind.view_json_ns)
                .map(|(a, b)| a + b),
        );
        for (total, ns) in all_phase.iter_mut().zip(kind.phase_ns) {
            *total += ns;
        }
        report.insert(
            name.clone(),
            json!({
                "events": samples,
                "rejected": kind.rejected,
                "dispatch_median_ns": median(&mut kind.dispatch_ns),
                "dispatch_p95_ns": p95(&mut kind.dispatch_ns),
                "view_json_median_ns": median(&mut kind.view_json_ns),
                "mean_ns": shares(&kind.phase_ns, kind.unphased_ns, view_total, samples),
            }),
        );
    }
    let events_total = all_samples.len() as u64;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "caveat-carmack-native-profile/1",
            "program": args[1],
            "events_per_rep": payloads.len(),
            "reps": reps,
            "phase_names": EVENT_PHASE_NAMES,
            "shares": SHARES.iter().map(|(name, phases)| json!({"share": name, "phases": phases})).collect::<Vec<_>>(),
            "all": {
                "events": events_total,
                "event_median_ns": median(&mut all_samples),
                "event_p95_ns": p95(&mut all_samples),
                "dispatch_mean_ns": all_dispatch / events_total,
                "mean_ns": shares(&all_phase, all_unphased, all_view, events_total),
            },
            "by_kind": report,
            "load_median_ns": median(&mut load),
            "save": {
                "state_bytes": state_bytes,
                "compact_bytes": compact_bytes,
                "state_median_ns": median(&mut save_state),
                "compact_median_ns": median(&mut save_compact),
                "restore_state_median_ns": median(&mut restore_state),
                "restore_compact_median_ns": median(&mut restore_compact),
            },
            "scope": "Native release build with fixed phase clocks; each boundary costs one timestamp. Rules include the lineage values carry as they are computed, which has no boundary of its own. view_json is serializing the view the host receives; building the view record is inside unphased.",
        }))
        .unwrap()
    );
}

/// Mean nanoseconds per event in each share, plus the per-phase means.
fn shares(phase_ns: &[u64; EVENT_PHASE_COUNT], unphased: u64, view: u64, events: u64) -> Value {
    let mut out = serde_json::Map::new();
    for (share, phases) in SHARES {
        let total: u64 = phases
            .iter()
            .map(|phase| {
                let index = EVENT_PHASE_NAMES.iter().position(|n| n == phase).unwrap();
                phase_ns[index]
            })
            .sum();
        out.insert(share.into(), json!(total / events.max(1)));
    }
    out.insert("unphased".into(), json!(unphased / events.max(1)));
    out.insert("view_json".into(), json!(view / events.max(1)));
    let phases: serde_json::Map<String, Value> = EVENT_PHASE_NAMES
        .iter()
        .zip(phase_ns)
        .filter(|(_, ns)| **ns > 0)
        .map(|(name, ns)| ((*name).into(), json!(ns / events.max(1))))
        .collect();
    out.insert("phases".into(), Value::Object(phases));
    Value::Object(out)
}
