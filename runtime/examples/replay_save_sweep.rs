//! Drives every reactive program it is given with seeded events and, after
//! each accepted event, restores the session's compact save. A replay save
//! must restore to the session that wrote it: the same snapshot and the same
//! state save, and the same outcome for the next event. Prints one line per
//! program and exits with status 1 on any difference.
//!
//!   git ls-files '*.cav' | cargo run --release --example replay_save_sweep -- [RUNS] [EVENTS]
//!
//! Runs default to 4 and events to 150. A program that does not load as a
//! reactive program is listed as `skipped`. See spec/caveat-replay-save-0.1.md.
use caveat_runtime::reactive::{ParameterDomain, ReactiveSession, REPLAY_SAVE_SCHEMA};
use std::io::{self, BufRead};

struct Seed(u64);

impl Seed {
    fn next(&mut self) -> u64 {
        // xorshift64*
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
    match seed.below(7) {
        0 => min,
        1 => max,
        2 if min <= 0.0 && max >= 0.0 => -0.0,
        3 => (min + (max - min) * (seed.below(1001) as f64 / 1000.0)).round(),
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

fn outcome(session: &mut ReactiveSession, event: &str, body: &str) -> String {
    match session.dispatch_outcome_json(event, body) {
        Ok(outcome) => serde_json::to_string(&outcome).unwrap(),
        Err(fatal) => format!("fatal {}", fatal.message),
    }
}

/// The program's line, and whether every replay save restored exactly.
fn sweep(path: &str, source: &str, runs: u64, events: usize) -> (String, bool) {
    let Ok(initial) = ReactiveSession::from_source(source) else {
        return (format!("{path} skipped"), true);
    };
    let signatures = initial.snapshot().events;
    if signatures.is_empty() {
        return (format!("{path} skipped"), true);
    }
    let (mut replays, mut largest, mut states) = (0usize, 0usize, 0usize);
    for run in 0..runs {
        let mut seed = Seed(0x9E3779B97F4A7C15 ^ (run + 1).wrapping_mul(0xD1B54A32D192ED03));
        let mut session = ReactiveSession::from_source(source).unwrap();
        for _ in 0..events {
            let signature = &signatures[seed.below(signatures.len())];
            let body = payload(&mut seed, &signature.parameters);
            let text = outcome(&mut session, &signature.name, &body);
            if text.starts_with("fatal") {
                break;
            }
            if text.contains("\"outcome\":\"rejected\"") {
                continue;
            }
            let compact = session.save_compact_json().unwrap();
            if !compact.contains(REPLAY_SAVE_SCHEMA) {
                states += 1;
                continue;
            }
            replays += 1;
            largest = largest.max(compact.len());
            let mut restored = match ReactiveSession::restore_json(source, &compact) {
                Ok(restored) => restored,
                Err(error) => return (format!("{path} FAILED restore: {error}"), false),
            };
            let same = serde_json::to_string(&restored.snapshot()).unwrap()
                == serde_json::to_string(&session.snapshot()).unwrap()
                && restored.save_json() == session.save_json()
                && restored.save_compact_json() == session.save_compact_json();
            if !same {
                return (format!("{path} FAILED after {replays} replay saves"), false);
            }
            let signature = &signatures[seed.below(signatures.len())];
            let body = payload(&mut seed, &signature.parameters);
            let mut probe = session.clone();
            if outcome(&mut probe, &signature.name, &body)
                != outcome(&mut restored, &signature.name, &body)
            {
                return (
                    format!("{path} FAILED next event after a replay save"),
                    false,
                );
            }
        }
    }
    (
        format!("{path} replay={replays} largest={largest} state={states}"),
        true,
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let runs = args
        .first()
        .and_then(|value| value.parse().ok())
        .unwrap_or(4);
    let events = args
        .get(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(150);
    let mut passed = true;
    for line in io::stdin().lock().lines() {
        let path = line.unwrap();
        let path = path.trim();
        if path.is_empty() {
            continue;
        }
        match std::fs::read_to_string(path) {
            Ok(source) => {
                let (line, ok) = sweep(path, &source, runs, events);
                passed &= ok;
                println!("{line}");
            }
            Err(error) => println!("{path} unreadable {error}"),
        }
    }
    if !passed {
        std::process::exit(1);
    }
}
