//! The View 0.2 delta gate (spec/caveat-view-0.2.md, "The delta gate"). Drives
//! every reactive program it is given with seeded events through the delta
//! path and checks, after every accepted event, that the held full view with
//! the delta applied equals the full view the session returns. The applier is
//! tests/view_delta_apply, written from the spec and not from the builder.
//!
//!   git ls-files '*.cav' | cargo run --release --example view_delta_sweep -- [RUNS] [EVENTS]
//!
//! Runs default to 8 and events to 150. Each line is one program:
//!
//!   PATH checked=N refused=N fatal=N mismatched=N restored=N controls=N/N
//!        dispatch_ns=N delta_ns=N view_ns=N apply_ns=N view_bytes=N delta_bytes=N
//!
//! or `PATH skipped` for a program that does not load as a reactive program or
//! declares no events. `controls` counts corrupted deltas detected out of
//! those tried: a dropped `set` entry, a wrong position in `removed`, and a
//! stale `since`. `dispatch_ns` times the same event on a twin session through
//! the 0.1 view path, so `delta_ns` minus it is what the delta path adds.
use caveat_runtime::reactive::{ParameterDomain, ReactiveSession};
use serde_json::{json, Value};
use std::io::{self, BufRead};
use std::time::Instant;

#[path = "../tests/view_delta_apply/mod.rs"]
mod view_delta_apply;
use view_delta_apply::{apply, same};

struct Seed(u64);

impl Seed {
    fn next(&mut self) -> u64 {
        // xorshift64*, as save_sweep
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

fn number(seed: &mut Seed, min: f64, max: f64) -> f64 {
    match seed.below(6) {
        0 => min,
        1 => max,
        2 => (min + (max - min) * (seed.below(1001) as f64 / 1000.0)).round(),
        _ => min + (max - min) * (seed.below(1_000_001) as f64 / 1_000_000.0),
    }
}

fn payload(seed: &mut Seed, parameters: &[caveat_runtime::reactive::Parameter]) -> String {
    let fields: Vec<String> = parameters
        .iter()
        .map(|parameter| {
            let value = match &parameter.domain {
                ParameterDomain::Numeric => {
                    let value = number(seed, parameter.min.value(), parameter.max.value());
                    serde_json::to_string(&value).unwrap()
                }
                ParameterDomain::Entity { members, .. } | ParameterDomain::Member { members } => {
                    serde_json::to_string(&members[seed.below(members.len())]).unwrap()
                }
                ParameterDomain::Identifier { .. } => format!("\"id{}\"", seed.below(12)),
            };
            format!(
                "{}:{value}",
                serde_json::to_string(&parameter.name).unwrap()
            )
        })
        .collect();
    format!("{{{}}}", fields.join(","))
}

fn view(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.view_v2()).unwrap()
}

/// Each corruption the gate must detect, where the delta gives it something
/// to corrupt: applying it fails, or gives something other than `full`.
fn controls(held: &Value, delta: &Value, full: &Value) -> (usize, usize) {
    let mut corrupted = Vec::new();
    for member in [
        "bindings",
        "binding_explanations",
        "commitment_grounds",
        "decision_series",
    ] {
        if let Some((name, _)) = delta[member]["set"]
            .as_object()
            .and_then(|set| set.iter().next())
        {
            let mut altered = delta.clone();
            altered[member]["set"]
                .as_object_mut()
                .unwrap()
                .remove(&name.clone());
            corrupted.push(altered);
        }
    }
    for member in ["decision_journal", "relations"] {
        let length = held[member].as_array().map_or(0, Vec::len);
        let removed = delta[member]["removed"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if let Some(first) = removed.first().and_then(Value::as_u64) {
            // The entry before or after the one removed, when it differs.
            for other in [first.checked_sub(1), Some(first + 1)]
                .into_iter()
                .flatten()
            {
                if (other as usize) < length
                    && !removed.contains(&json!(other))
                    && !same(&held[member][other as usize], &held[member][first as usize])
                {
                    let mut altered = delta.clone();
                    altered[member]["removed"][0] = json!(other);
                    let mut positions = altered[member]["removed"].as_array().unwrap().clone();
                    positions.sort_by_key(|position| position.as_u64());
                    altered[member]["removed"] = Value::Array(positions);
                    corrupted.push(altered);
                    break;
                }
            }
        }
    }
    let mut stale = delta.clone();
    stale["since"] = json!(delta["since"].as_u64().unwrap() + 1);
    corrupted.push(stale);
    let detected = corrupted
        .iter()
        .filter(|altered| apply(held, altered).map_or(true, |next| !same(&next, full)))
        .count();
    (detected, corrupted.len())
}

#[derive(Default)]
struct Tally {
    checked: usize,
    refused: usize,
    fatal: usize,
    mismatched: usize,
    restored: usize,
    detected: usize,
    controls: usize,
    dispatch_ns: u128,
    delta_ns: u128,
    view_ns: u128,
    apply_ns: u128,
    view_bytes: usize,
    delta_bytes: usize,
}

fn sweep(path: &str, source: &str, runs: u64, events: usize) -> String {
    let Ok(initial) = ReactiveSession::from_source(source) else {
        return format!("{path} skipped");
    };
    let signatures = initial.snapshot().events;
    if signatures.is_empty() {
        return format!("{path} skipped");
    }
    let mut tally = Tally::default();
    for run in 0..runs {
        let mut seed = Seed(0x9E3779B97F4A7C15 ^ (run + 1).wrapping_mul(0xD1B54A32D192ED03));
        let mut session = initial.clone();
        let mut twin = initial.clone();
        let mut held = view(&session);
        for index in 0..events {
            let signature = &signatures[seed.below(signatures.len())];
            let body = payload(&mut seed, &signature.parameters);

            let started = Instant::now();
            let _ = twin
                .dispatch_view_outcome_json(&signature.name, &body)
                .map(|_| ());
            tally.dispatch_ns += started.elapsed().as_nanos();

            let started = Instant::now();
            let outcome = session.dispatch_view_delta_outcome_json(&signature.name, &body);
            let text = outcome.map(|outcome| serde_json::to_string(&outcome).unwrap());
            tally.delta_ns += started.elapsed().as_nanos();
            let Ok(text) = text else {
                tally.fatal += 1;
                break;
            };
            let outcome: Value = serde_json::from_str(&text).unwrap();

            let started = Instant::now();
            let full_text = serde_json::to_string(&session.view_v2()).unwrap();
            tally.view_ns += started.elapsed().as_nanos();
            let full: Value = serde_json::from_str(&full_text).unwrap();

            if outcome["outcome"] != "accepted" {
                tally.refused += 1;
                if !same(&held, &full) {
                    tally.mismatched += 1;
                }
                continue;
            }
            let delta = &outcome["delta"];
            tally.view_bytes += full_text.len();
            tally.delta_bytes += serde_json::to_string(delta).unwrap().len();
            let started = Instant::now();
            let next = apply(&held, delta);
            tally.apply_ns += started.elapsed().as_nanos();
            tally.checked += 1;
            match next {
                Ok(next) if same(&next, &full) => {
                    let (detected, tried) = controls(&held, delta, &full);
                    tally.detected += detected;
                    tally.controls += tried;
                    held = next;
                }
                _ => {
                    tally.mismatched += 1;
                    held = full;
                }
            }
            // A restore starts a new session whose full view is the held one.
            if index % 25 == 24 {
                if let Ok(save) = session.save_json() {
                    if let Ok(restored) = ReactiveSession::restore_json(source, &save) {
                        tally.restored += 1;
                        if !same(&view(&restored), &held) {
                            tally.mismatched += 1;
                        }
                        session = restored;
                    }
                }
            }
        }
    }
    let Tally {
        checked,
        refused,
        fatal,
        mismatched,
        restored,
        detected,
        controls,
        dispatch_ns,
        delta_ns,
        view_ns,
        apply_ns,
        view_bytes,
        delta_bytes,
    } = tally;
    format!(
        "{path} checked={checked} refused={refused} fatal={fatal} mismatched={mismatched} \
         restored={restored} controls={detected}/{controls} dispatch_ns={dispatch_ns} \
         delta_ns={delta_ns} view_ns={view_ns} apply_ns={apply_ns} view_bytes={view_bytes} \
         delta_bytes={delta_bytes}"
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let runs = args
        .first()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8);
    let events = args
        .get(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(150);
    for line in io::stdin().lock().lines() {
        let path = line.unwrap();
        let path = path.trim();
        if path.is_empty() {
            continue;
        }
        match std::fs::read_to_string(path) {
            Ok(source) => println!("{}", sweep(path, &source, runs, events)),
            Err(error) => println!("{path} unreadable {error}"),
        }
    }
}
