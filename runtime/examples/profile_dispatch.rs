//! Where does a reactive event's time go? Replays the glowcap benchmark's
//! events (three observations, then ticks) against a program and reports the
//! median cost of each stage:
//!
//!   cargo run --release --no-default-features --example profile_dispatch -- PROGRAM.cav [TICKS]
use caveat_runtime::reactive::ReactiveSession;
use std::collections::BTreeMap;
use std::time::Instant;

fn median(samples: &mut [f64]) -> f64 {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    samples[samples.len() / 2]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .expect("usage: profile_dispatch PROGRAM.cav [TICKS]");
    let ticks: usize = args
        .next()
        .map_or(2000, |n| n.parse().expect("TICKS is a number"));
    let source = std::fs::read_to_string(&path).expect("readable program");

    let started = Instant::now();
    let mut session = ReactiveSession::from_source(&source).expect("program loads");
    println!(
        "load            {:>9.1} µs",
        started.elapsed().as_secs_f64() * 1e6
    );

    let mut events: Vec<(&str, BTreeMap<String, f64>)> = vec![
        (
            "absorb",
            BTreeMap::from([("target".into(), 1.0), ("sort".into(), 1.0)]),
        ),
        (
            "absorb",
            BTreeMap::from([("target".into(), 2.0), ("sort".into(), 2.0)]),
        ),
        (
            "taste",
            BTreeMap::from([("target".into(), 3.0), ("sort".into(), 1.0)]),
        ),
    ];
    events.extend((0..ticks).map(|_| ("tick", BTreeMap::from([("dt".into(), 0.05)]))));

    let (mut clone, mut apply, mut view, mut json) = (vec![], vec![], vec![], vec![]);
    let mut bytes = 0;
    for (event, parameters) in &events {
        let t = Instant::now();
        let copy = session.clone();
        clone.push(t.elapsed().as_secs_f64() * 1e6);
        drop(copy);
        let t = Instant::now();
        session.apply(event, parameters).expect("event accepted");
        apply.push(t.elapsed().as_secs_f64() * 1e6);
        let t = Instant::now();
        let shown = session.view();
        view.push(t.elapsed().as_secs_f64() * 1e6);
        let t = Instant::now();
        let text = serde_json::to_string(&shown).unwrap();
        json.push(t.elapsed().as_secs_f64() * 1e6);
        bytes = text.len();
    }
    println!(
        "clone session   {:>9.1} µs (inside apply)",
        median(&mut clone)
    );
    println!(
        "apply           {:>9.1} µs (clone + rules + bindings)",
        median(&mut apply)
    );
    println!("build view      {:>9.1} µs", median(&mut view));
    println!(
        "serialize view  {:>9.1} µs ({bytes} bytes)",
        median(&mut json)
    );
}
