//! Drives every reactive program it is given with seeded events and prints one
//! digest line per program over each outcome, save and restored save. Run it
//! on two builds and compare the output: a change that must leave existing
//! programs byte-identical leaves every line the same.
//!
//!   git ls-files '*.cav' | cargo run --release --example save_sweep -- [RUNS] [EVENTS]
//!
//! Runs default to 8 and events to 150. A program that does not load as a
//! reactive program is listed as `skipped`.
//!
//! The fields saves gained after rc.15 by design, `source_sha256` and
//! `source_unrecorded_through` (spec/caveat-save-0.1.md, "Source digest") and
//! `evidence_order` (spec/caveat-view-0.2.md), are taken out of saves and
//! snapshots before digesting, so the comparison with rc.15 covers everything
//! else. The runtime's own tests check those fields.
use caveat_runtime::reactive::{ParameterDomain, ReactiveSession};
use std::io::{self, BufRead};

/// `text` without the fields added since rc.15, wherever they occur (a save,
/// or an outcome's snapshot). They are removed as text, so every other byte is
/// compared as written.
fn comparable(text: &str) -> String {
    let mut text = text.to_string();
    for field in [",\"source_sha256\":\"", ",\"source_unrecorded_through\":"] {
        while let Some(start) = text.find(field) {
            let value = start + field.len();
            let end = value + text[value..].find([',', '}']).expect("a field ends");
            text.replace_range(start..end, "");
        }
    }
    // An object of name lists: names hold no braces, so the first `}` after
    // the opening one closes it.
    let field = ",\"evidence_order\":{";
    while let Some(start) = text.find(field) {
        let value = start + field.len();
        let end = value + text[value..].find('}').expect("an order ends") + 1;
        text.replace_range(start..end, "");
    }
    text
}

struct Fnv(u64);

impl Fnv {
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
        self.write_separator();
    }

    fn write_separator(&mut self) {
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

fn sweep(path: &str, source: &str, runs: u64, events: usize) -> String {
    let Ok(initial) = ReactiveSession::from_source(source) else {
        return format!("{path} skipped");
    };
    let signatures = initial.snapshot().events;
    if signatures.is_empty() {
        return format!("{path} skipped");
    }
    let mut digest = Fnv(0xcbf29ce484222325);
    let (mut saves, mut refused, mut fatal) = (0usize, 0usize, 0usize);
    for run in 0..runs {
        let mut seed = Seed(0x9E3779B97F4A7C15 ^ (run + 1).wrapping_mul(0xD1B54A32D192ED03));
        let mut session = initial.clone();
        for _ in 0..events {
            let signature = &signatures[seed.below(signatures.len())];
            let body = payload(&mut seed, &signature.parameters);
            digest.write(signature.name.as_bytes());
            digest.write(body.as_bytes());
            match session.dispatch_outcome_json(&signature.name, &body) {
                Ok(outcome) => {
                    let text = serde_json::to_string(&outcome).unwrap();
                    if text.contains("\"outcome\":\"rejected\"") {
                        refused += 1;
                    }
                    digest.write(comparable(&text).as_bytes());
                }
                Err(error) => {
                    fatal += 1;
                    digest.write(format!("fatal {error:?}").as_bytes());
                    break;
                }
            }
            match session.save_json() {
                Ok(save) => {
                    saves += 1;
                    digest.write(comparable(&save).as_bytes());
                    match ReactiveSession::restore_json(source, &save) {
                        Ok(restored) => {
                            digest.write(comparable(&restored.save_json().unwrap()).as_bytes())
                        }
                        Err(error) => digest.write(format!("restore refused {error}").as_bytes()),
                    }
                }
                Err(error) => digest.write(format!("save failed {error}").as_bytes()),
            }
        }
    }
    format!(
        "{path} {:016x} saves={saves} refused={refused} fatal={fatal}",
        digest.0
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
