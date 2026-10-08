//! The outcome sweep of spec/caveat-departure-0.1.md: drives each windowed
//! program it is given with seeded events and prints, per run, the sequence of
//! dispatch outcomes (accepted, or the refusal's origin and code). Run it on
//! this build and on rc.15's retirement-only build and compare the output:
//! departure never changes an outcome, so every line is the same.
//!
//!   ls experiments/departure-gate/*.cav | cargo run --release --example outcome_sweep -- [RUNS] [EVENTS]
//!
//! Runs default to 32 and events to 400. Each line names the program, the run,
//! its counts and a digest of its outcome sequence; with `--full` the sequence
//! itself follows, one outcome per line.
use caveat_runtime::reactive::{ParameterDomain, ReactiveSession};
use std::io::{self, BufRead};

struct Fnv(u64);

impl Fnv {
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
        self.0 ^= 0xff;
        self.0 = self.0.wrapping_mul(0x100000001b3);
    }
}

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

/// An outcome as the sweep compares it: `accepted`, or the refusal's origin
/// and code, or a fatal error.
fn outcome(
    session: &mut ReactiveSession,
    event: &str,
    body: &str,
    departs: &mut usize,
) -> (String, bool) {
    match session.dispatch_outcome_json(event, body) {
        Ok(outcome) => {
            let value = serde_json::to_value(&outcome).unwrap();
            if value["outcome"] == "accepted" {
                *departs += value["snapshot"]["effects"]
                    .as_array()
                    .map_or(0, |effects| {
                        effects
                            .iter()
                            .filter(|effect| effect["kind"] == "depart")
                            .count()
                    });
                ("accepted".into(), false)
            } else {
                (
                    format!(
                        "{} {}/{}",
                        value["outcome"].as_str().unwrap_or("?"),
                        value["origin"].as_str().unwrap_or("?"),
                        value["code"].as_str().unwrap_or("?")
                    ),
                    false,
                )
            }
        }
        Err(error) => (format!("fatal {}", error.message), true),
    }
}

fn sweep(path: &str, source: &str, runs: u64, events: usize, full: bool) {
    let Ok(initial) = ReactiveSession::from_source(source) else {
        println!("{path} skipped");
        return;
    };
    let signatures = initial.snapshot().events;
    // Departures are counted on stderr, so that the compared lines are the
    // same on a build that departs nothing.
    let mut departs = 0;
    for run in 0..runs {
        let mut seed = Seed(0x9E3779B97F4A7C15 ^ (run + 1).wrapping_mul(0xD1B54A32D192ED03));
        let mut session = initial.clone();
        let mut digest = Fnv(0xcbf29ce484222325);
        let (mut accepted, mut refused) = (0usize, 0usize);
        let mut lines = Vec::new();
        let mut fatal = false;
        for _ in 0..events {
            let signature = &signatures[seed.below(signatures.len())];
            let body = payload(&mut seed, &signature.parameters);
            let (text, ended) = outcome(&mut session, &signature.name, &body, &mut departs);
            if text == "accepted" {
                accepted += 1;
            } else if !ended {
                refused += 1;
            }
            digest.write(signature.name.as_bytes());
            digest.write(body.as_bytes());
            digest.write(text.as_bytes());
            if full {
                lines.push(format!("  {} {body} {text}", signature.name));
            }
            if ended {
                fatal = true;
                break;
            }
        }
        println!(
            "{path} run={run} accepted={accepted} refused={refused} fatal={fatal} {:016x}",
            digest.0
        );
        for line in lines {
            println!("{line}");
        }
    }
    eprintln!("{path} departs={departs}");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let full = args.iter().any(|arg| arg == "--full");
    let numbers: Vec<&String> = args.iter().filter(|arg| *arg != "--full").collect();
    let runs = numbers
        .first()
        .and_then(|value| value.parse().ok())
        .unwrap_or(32);
    let events = numbers
        .get(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(400);
    for line in io::stdin().lock().lines() {
        let path = line.unwrap();
        let path = path.trim();
        if path.is_empty() {
            continue;
        }
        match std::fs::read_to_string(path) {
            Ok(source) => sweep(path, &source, runs, events, full),
            Err(error) => println!("{path} unreadable {error}"),
        }
    }
}
