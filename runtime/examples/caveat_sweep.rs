//! The corpus sweep of spec/caveat-caveat-sources-0.1.md: drives every
//! reactive program it is given with seeded events and prints, after each
//! event, every state's caveats in its value and its grounds, and every view
//! binding's. Run it on two builds and compare the output with
//! experiments/caveat-sources-sweep/sweep.mjs: a caveat one build lists and
//! the other does not is a gain or a loss, counted by direction.
//!
//!   git ls-files '*.cav' | cargo run --release --example caveat_sweep -- [RUNS] [EVENTS]
//!
//! Runs default to 8 and events to 150, the save sweep's. Each line is
//! `PATH RUN STEP EVENT OUTCOME`, then one `  STATE value|grounds CAVEATS`
//! line per state that carries a caveat and one `  BINDING.FIELD binding
//! CAVEATS` line per binding field that does. A program that does not load as
//! a reactive program is listed as `skipped`.
use caveat_runtime::reactive::{ParameterDomain, ReactiveSession};
use serde_json::Value;
use std::io::{self, BufRead, Write};

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

fn caveats(provenance: &Value) -> String {
    provenance["caveats"]
        .as_array()
        .map(|names| {
            names
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default()
}

fn sweep(path: &str, source: &str, runs: u64, events: usize, out: &mut impl Write) {
    let Ok(initial) = ReactiveSession::from_source(source) else {
        writeln!(out, "{path} skipped").unwrap();
        return;
    };
    let signatures = initial.snapshot().events;
    if signatures.is_empty() {
        writeln!(out, "{path} skipped").unwrap();
        return;
    }
    for run in 0..runs {
        let mut seed = Seed(0x9E3779B97F4A7C15 ^ (run + 1).wrapping_mul(0xD1B54A32D192ED03));
        let mut session = initial.clone();
        for step in 0..events {
            let signature = &signatures[seed.below(signatures.len())];
            let body = payload(&mut seed, &signature.parameters);
            let outcome = match session.dispatch_outcome_json(&signature.name, &body) {
                Ok(outcome) => {
                    let text = serde_json::to_string(&outcome).unwrap();
                    if text.contains("\"outcome\":\"rejected\"") {
                        "refused"
                    } else {
                        "accepted"
                    }
                }
                Err(_) => "fatal",
            };
            writeln!(out, "{path} {run} {step} {} {outcome}", signature.name).unwrap();
            if outcome == "fatal" {
                break;
            }
            let shown = serde_json::to_value(session.snapshot()).unwrap();
            let grounds = &shown["value_grounds"];
            if let Some(values) = shown["qualified_values"].as_object() {
                for (state, value) in values {
                    let held = caveats(&value["provenance"]);
                    if !held.is_empty() {
                        writeln!(out, "  {state} value {held}").unwrap();
                    }
                    let held = caveats(&grounds[state]);
                    if !held.is_empty() {
                        writeln!(out, "  {state} grounds {held}").unwrap();
                    }
                }
            }
            if let Some(bindings) = shown["binding_qualifications"].as_object() {
                for (binding, fields) in bindings {
                    for (field, provenance) in fields.as_object().into_iter().flatten() {
                        let held = caveats(provenance);
                        if !held.is_empty() {
                            writeln!(out, "  {binding}.{field} binding {held}").unwrap();
                        }
                    }
                }
            }
        }
    }
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
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    for line in io::stdin().lock().lines() {
        let path = line.unwrap();
        let path = path.trim();
        if path.is_empty() {
            continue;
        }
        match std::fs::read_to_string(path) {
            Ok(source) => sweep(path, &source, runs, events, &mut out),
            Err(error) => writeln!(out, "{path} unreadable {error}").unwrap(),
        }
    }
}
