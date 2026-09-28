//! Measurement-only baseline of the reactive runtime, natively.
//!
//! Reads a plan written by experiments/performance-0.1/run.mjs, times one
//! mode's operations for every event with `std::time::Instant`, and writes the
//! raw nanosecond samples as JSON; run.mjs summarizes them with the same code
//! it uses for the WebAssembly samples. It calls only the runtime's public
//! API. `web::WebReactiveSession` is the type whose methods the WebAssembly
//! build exports, compiled natively, so the `web.*` operations are the same
//! functions the Node benchmark calls through wasm-bindgen.
//!
//!   perf_baseline PLAN.json OUT.json
//!
//! run.mjs builds this file against a runtime tree in a generated crate under
//! that tree's runtime/target/, with the runtime's own release profile. It
//! changes nothing in the tree it measures.
use caveat_runtime::reactive::{ParameterDomain, ReactiveSave, ReactiveSession};
use caveat_runtime::web::WebReactiveSession;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

type Step = (String, String);
type Numeric = BTreeMap<String, f64>;

struct Plan {
    workload: String,
    program: String,
    episodes: Vec<Vec<Step>>,
    repeats: usize,
    mode: String,
    warmup: usize,
    rounds: usize,
    lifecycle_samples: usize,
    lifecycle_warmup: usize,
}

/// Samples of one timed pass: per operation, one slot per event position.
type Pass = Vec<Vec<Option<u64>>>;

struct Measured {
    ops: Vec<&'static str>,
    /// rounds[round][op][event position]
    rounds: Vec<Pass>,
    /// The save of every episode's final session, from the last pass.
    finals: Vec<String>,
    /// What `finals` holds, so run.mjs compares like with like.
    finals_kind: &'static str,
    notes: Vec<String>,
    extra: Map<String, Value>,
}

fn nanos(start: Instant) -> Option<u64> {
    Some(start.elapsed().as_nanos() as u64)
}

fn load_plan(path: &str) -> Plan {
    let text = std::fs::read_to_string(path).expect("readable plan");
    let plan: Value = serde_json::from_str(&text).expect("plan is JSON");
    let count = |key: &str, default: u64| plan[key].as_u64().unwrap_or(default) as usize;
    let episodes = plan["episodes"]
        .as_array()
        .expect("plan.episodes")
        .iter()
        .map(|episode| {
            episode
                .as_array()
                .expect("episode is an array")
                .iter()
                .map(|step| {
                    (
                        step[0].as_str().expect("event name").to_string(),
                        step[1].as_str().expect("payload text").to_string(),
                    )
                })
                .collect()
        })
        .collect();
    Plan {
        workload: plan["workload"].as_str().unwrap_or("").to_string(),
        program: plan["program"].as_str().expect("plan.program").to_string(),
        episodes,
        repeats: count("repeats", 1),
        mode: plan["mode"].as_str().expect("plan.mode").to_string(),
        warmup: count("warmup", 1),
        rounds: count("rounds", 3),
        lifecycle_samples: count("lifecycleSamples", 30),
        lifecycle_warmup: count("lifecycleWarmup", 3),
    }
}

/// Plays every episode `warmup + rounds` times, each episode from a freshly
/// opened session (opening is not timed). `step` fills one slot per op.
fn drive<S>(
    plan: &Plan,
    ops: Vec<&'static str>,
    open: impl Fn() -> S,
    finish: impl Fn(&S) -> String,
    mut step: impl FnMut(&mut S, &Step, &mut [Option<u64>]),
) -> Measured {
    let passes = plan.warmup + plan.rounds;
    let per_pass: usize = plan.episodes.iter().map(Vec::len).sum::<usize>() * plan.repeats;
    let mut rounds = Vec::with_capacity(plan.rounds);
    let mut finals = Vec::new();
    let mut slots = vec![None; ops.len()];
    for pass in 0..passes {
        let mut samples: Pass = ops.iter().map(|_| Vec::with_capacity(per_pass)).collect();
        for repeat in 0..plan.repeats {
            for episode in &plan.episodes {
                let mut session = open();
                for event in episode {
                    slots.iter_mut().for_each(|slot| *slot = None);
                    step(&mut session, event, &mut slots);
                    for (op, slot) in slots.iter().enumerate() {
                        samples[op].push(*slot);
                    }
                }
                if pass + 1 == passes && repeat + 1 == plan.repeats {
                    finals.push(finish(&session));
                }
            }
        }
        if pass >= plan.warmup {
            rounds.push(samples);
        }
    }
    Measured {
        ops,
        rounds,
        finals,
        finals_kind: "save-text",
        notes: Vec::new(),
        extra: Map::new(),
    }
}

fn open_core(program: &str) -> ReactiveSession {
    ReactiveSession::from_source(program).expect("program loads")
}

fn open_web(program: &str) -> WebReactiveSession {
    WebReactiveSession::new(program).expect("program loads")
}

fn save_core(session: &ReactiveSession) -> String {
    session.save_json().expect("session saves")
}

fn save_web(session: &WebReactiveSession) -> String {
    session.save().expect("session saves")
}

/// The numeric parameters `apply` takes, resolved from the payload text the
/// way a host's names resolve: a number stays a number; a member or entity
/// name becomes its position counted from 1. Identifier parameters cannot be
/// sent as numbers for a new identifier, so a workload with one has no
/// numeric form (None). The benchmark checks the result: every mode must end
/// in the same saved state.
fn numeric_steps(plan: &Plan) -> Option<Vec<Vec<Numeric>>> {
    let signatures = open_core(&plan.program).snapshot().events;
    plan.episodes
        .iter()
        .map(|episode| {
            episode
                .iter()
                .map(|(event, payload)| {
                    let parameters = &signatures.iter().find(|s| &s.name == event)?.parameters;
                    let object: Map<String, Value> = serde_json::from_str(payload).ok()?;
                    object
                        .into_iter()
                        .map(|(name, value)| {
                            let number = match value {
                                Value::Number(number) => number.as_f64()?,
                                Value::String(text) => {
                                    let parameter = parameters.iter().find(|p| p.name == name)?;
                                    let members = match &parameter.domain {
                                        ParameterDomain::Entity { members, .. }
                                        | ParameterDomain::Member { members } => members,
                                        _ => return None,
                                    };
                                    (members.iter().position(|member| *member == text)? + 1) as f64
                                }
                                _ => return None,
                            };
                            Some((name, number))
                        })
                        .collect()
                })
                .collect()
        })
        .collect()
}

fn run_apply(plan: &Plan) -> Measured {
    let Some(numeric) = numeric_steps(plan) else {
        return Measured {
            ops: vec!["apply"],
            rounds: Vec::new(),
            finals: Vec::new(),
            finals_kind: "none",
            notes: vec!["skipped: a payload has no numeric form (identifier parameter)".into()],
            extra: Map::new(),
        };
    };
    // Steps carry their text; find the numeric form by position.
    let flat: Vec<&Numeric> = numeric.iter().flatten().collect();
    let per_repeat = flat.len();
    let mut position = 0usize;
    let program = plan.program.clone();
    let mut measured = drive(
        plan,
        vec!["apply"],
        || open_core(&program),
        save_core,
        |session, (event, _), slots| {
            let parameters = flat[position % per_repeat];
            position += 1;
            let start = Instant::now();
            let result = session.apply(event, parameters);
            slots[0] = nanos(start);
            black_box(&result);
        },
    );
    measured.extra.insert(
        "note".into(),
        json!("apply takes numeric parameters: no payload JSON, no name resolution, no view or snapshot"),
    );
    measured
}

fn run_core_dispatch(plan: &Plan, which: &'static str) -> Measured {
    let program = plan.program.clone();
    drive(
        plan,
        vec![which],
        || open_core(&program),
        save_core,
        |session, (event, payload), slots| match which {
            "dispatch_view_json" => {
                let start = Instant::now();
                let view = session.dispatch_view_json(event, payload);
                slots[0] = nanos(start);
                black_box(&view);
            }
            "dispatch_json" => {
                let start = Instant::now();
                let snapshot = session.dispatch_json(event, payload);
                slots[0] = nanos(start);
                black_box(&snapshot);
            }
            "dispatch_outcome_json" => {
                let start = Instant::now();
                let outcome = session.dispatch_outcome_json(event, payload);
                slots[0] = nanos(start);
                black_box(&outcome);
            }
            _ => unreachable!(),
        },
    )
}

fn run_web_dispatch(plan: &Plan, which: &'static str) -> Measured {
    let program = plan.program.clone();
    drive(
        plan,
        vec![which],
        || open_web(&program),
        save_web,
        |session, (event, payload), slots| {
            let start = Instant::now();
            let result = match which {
                "web.dispatch_view" => session.dispatch_view(event, payload),
                "web.dispatch_outcome" => session.dispatch_outcome(event, payload),
                "web.dispatch" => session.dispatch(event, payload),
                _ => unreachable!(),
            };
            slots[0] = nanos(start);
            drop(black_box(result));
        },
    )
}

/// Read-only operations on the state each event leaves, after an untimed
/// dispatch. Builds are timed without dropping what they built; the drop of
/// a snapshot and of a session clone are timed on their own.
fn run_read(plan: &Plan) -> Measured {
    let program = plan.program.clone();
    let mut sizes = (0usize, 0usize, 0usize, 0usize);
    let ops = vec![
        "view",
        "view.serialize",
        "snapshot",
        "snapshot.serialize",
        "snapshot.serialize_pretty",
        "snapshot.drop",
        "save",
        "save_json",
        "clone",
        "clone.drop",
    ];
    let mut measured = drive(
        plan,
        ops,
        || open_core(&program),
        save_core,
        |session, (event, payload), slots| {
            drop(black_box(session.dispatch_view_json(event, payload)));
            let start = Instant::now();
            let view = session.view();
            slots[0] = nanos(start);
            let start = Instant::now();
            let view_text = serde_json::to_string(&view).expect("view serializes");
            slots[1] = nanos(start);
            black_box(&view);
            let start = Instant::now();
            let snapshot = session.snapshot();
            slots[2] = nanos(start);
            let start = Instant::now();
            let compact = serde_json::to_string(&snapshot).expect("snapshot serializes");
            slots[3] = nanos(start);
            let start = Instant::now();
            let pretty =
                caveat_runtime::map::to_json_pretty(&snapshot).expect("snapshot serializes");
            slots[4] = nanos(start);
            let start = Instant::now();
            drop(black_box(snapshot));
            slots[5] = nanos(start);
            let start = Instant::now();
            let save = session.save();
            slots[6] = nanos(start);
            black_box(&save);
            let start = Instant::now();
            let save_text = session.save_json();
            slots[7] = nanos(start);
            let start = Instant::now();
            let copy = session.clone();
            slots[8] = nanos(start);
            let start = Instant::now();
            drop(black_box(copy));
            slots[9] = nanos(start);
            sizes = (
                view_text.len(),
                compact.len(),
                pretty.len(),
                save_text.map(|text| text.len()).unwrap_or(0),
            );
        },
    );
    measured.extra.insert(
        "bytesAtEnd".into(),
        json!({"view": sizes.0, "snapshotCompact": sizes.1, "snapshotPretty": sizes.2, "save": sizes.3}),
    );
    measured
}

fn run_web_read(plan: &Plan) -> Measured {
    let program = plan.program.clone();
    drive(
        plan,
        vec!["web.view", "web.snapshot", "web.save"],
        || open_web(&program),
        save_web,
        |session, (event, payload), slots| {
            drop(black_box(session.dispatch_view(event, payload)));
            let start = Instant::now();
            let view = session.view();
            slots[0] = nanos(start);
            drop(black_box(view));
            let start = Instant::now();
            let snapshot = session.snapshot();
            slots[1] = nanos(start);
            drop(black_box(snapshot));
            let start = Instant::now();
            let save = session.save();
            slots[2] = nanos(start);
            drop(black_box(save));
        },
    )
}

/// Load, save and restore at the state the first episode ends in. Each op is
/// sampled `lifecycle_samples` times after `lifecycle_warmup` untimed calls;
/// every restored session must save to the same text it was restored from.
fn run_lifecycle(plan: &Plan) -> Measured {
    let program = plan.program.as_str();
    let mut session = open_core(program);
    let mut web = open_web(program);
    for (event, payload) in plan.episodes.first().into_iter().flatten() {
        drop(black_box(session.dispatch_view_json(event, payload)));
        drop(black_box(web.dispatch_view(event, payload)));
    }
    let saved = save_core(&session);
    assert_eq!(saved, save_web(&web), "native and web sessions diverged");
    let parsed: ReactiveSave = serde_json::from_str(&saved).expect("save parses");
    let ops = vec![
        "from_source",
        "web.new",
        "save",
        "save_json",
        "web.save",
        "restore.parse",
        "restore",
        "restore_json",
        "web.restore",
    ];
    let total = plan.lifecycle_warmup + plan.lifecycle_samples;
    let mut samples: Pass = ops.iter().map(|_| Vec::with_capacity(total)).collect();
    let mut restored_saves = Vec::new();
    for index in 0..total {
        let mut slot = [None; 9];
        let start = Instant::now();
        let loaded = ReactiveSession::from_source(program);
        slot[0] = nanos(start);
        drop(black_box(loaded));
        let start = Instant::now();
        let loaded = WebReactiveSession::new(program);
        slot[1] = nanos(start);
        drop(black_box(loaded));
        let start = Instant::now();
        let save = session.save();
        slot[2] = nanos(start);
        drop(black_box(save));
        let start = Instant::now();
        let save = session.save_json();
        slot[3] = nanos(start);
        drop(black_box(save));
        let start = Instant::now();
        let save = web.save();
        slot[4] = nanos(start);
        drop(black_box(save));
        let start = Instant::now();
        let parsed_again: Result<ReactiveSave, _> = serde_json::from_str(&saved);
        slot[5] = nanos(start);
        drop(black_box(parsed_again));
        let start = Instant::now();
        let restored = ReactiveSession::restore(program, &parsed).expect("restores");
        slot[6] = nanos(start);
        drop(black_box(restored));
        let start = Instant::now();
        let restored = ReactiveSession::restore_json(program, &saved).expect("restores");
        slot[7] = nanos(start);
        if index == 0 {
            restored_saves.push(save_core(&restored));
        }
        drop(black_box(restored));
        let start = Instant::now();
        let restored = WebReactiveSession::restore(program, &saved).expect("restores");
        slot[8] = nanos(start);
        if index == 0 {
            restored_saves.push(save_web(&restored));
        }
        drop(black_box(restored));
        if index >= plan.lifecycle_warmup {
            for (op, value) in slot.iter().enumerate() {
                samples[op].push(*value);
            }
        }
    }
    let mut notes = Vec::new();
    if restored_saves.iter().any(|text| *text != saved) {
        notes.push("CORRECTNESS: a restored session saves differently from its save".into());
    }
    let mut extra = Map::new();
    extra.insert("saveBytes".into(), json!(saved.len()));
    Measured {
        ops,
        rounds: vec![samples],
        finals: vec![saved],
        finals_kind: "first-episode-save-text",
        notes,
        extra,
    }
}

fn timer_overhead() -> Value {
    let mut samples: Vec<u64> = (0..10_000)
        .map(|_| {
            let start = Instant::now();
            start.elapsed().as_nanos() as u64
        })
        .collect();
    samples.sort_unstable();
    json!({
        "what": "Instant::now() then elapsed(), empty",
        "minNs": samples[0],
        "medianNs": samples[(samples.len() - 1) / 2],
        "p99Ns": samples[(samples.len() - 1) * 99 / 100],
    })
}

#[cfg(windows)]
fn process_affinity() -> Value {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn GetProcessAffinityMask(process: isize, mask: *mut usize, system: *mut usize) -> i32;
        fn GetPriorityClass(process: isize) -> u32;
    }
    let (mut mask, mut system) = (0usize, 0usize);
    // SAFETY: the pseudo handle needs no closing and both out-pointers are valid.
    let (ok, priority) = unsafe {
        let process = GetCurrentProcess();
        (
            GetProcessAffinityMask(process, &mut mask, &mut system),
            GetPriorityClass(process),
        )
    };
    if ok == 0 {
        return Value::Null;
    }
    json!({
        "processMask": format!("0x{mask:X}"),
        "systemMask": format!("0x{system:X}"),
        "logicalProcessors": mask.count_ones(),
        "priorityClass": format!("0x{priority:X}"),
    })
}

#[cfg(not(windows))]
fn process_affinity() -> Value {
    Value::Null
}

fn main() {
    let mut args = std::env::args().skip(1);
    let plan_path = args
        .next()
        .expect("usage: perf_baseline PLAN.json OUT.json");
    let out_path = args
        .next()
        .expect("usage: perf_baseline PLAN.json OUT.json");
    let plan = load_plan(&plan_path);
    let started = Instant::now();
    let measured = match plan.mode.as_str() {
        "apply" => run_apply(&plan),
        "dispatch_view_json" => run_core_dispatch(&plan, "dispatch_view_json"),
        "dispatch_json" => run_core_dispatch(&plan, "dispatch_json"),
        "dispatch_outcome_json" => run_core_dispatch(&plan, "dispatch_outcome_json"),
        "web.dispatch_view" => run_web_dispatch(&plan, "web.dispatch_view"),
        "web.dispatch_outcome" => run_web_dispatch(&plan, "web.dispatch_outcome"),
        "web.dispatch" => run_web_dispatch(&plan, "web.dispatch"),
        "read" => run_read(&plan),
        "web.read" => run_web_read(&plan),
        "lifecycle" => run_lifecycle(&plan),
        other => panic!("unknown native mode {other}"),
    };
    let mut ops = Map::new();
    for (index, name) in measured.ops.iter().enumerate() {
        let rounds: Vec<&Vec<Option<u64>>> =
            measured.rounds.iter().map(|round| &round[index]).collect();
        ops.insert((*name).into(), json!({ "unit": "ns", "rounds": rounds }));
    }
    let output = json!({
        "schema": "caveat-performance-raw/0.1",
        "engine": "native",
        "workload": plan.workload,
        "mode": plan.mode,
        "method": {
            "warmupPasses": plan.warmup,
            "rounds": plan.rounds,
            "repeats": plan.repeats,
            "lifecycleSamples": plan.lifecycle_samples,
            "lifecycleWarmup": plan.lifecycle_warmup,
            "timer": "std::time::Instant",
            "timerOverhead": timer_overhead(),
            "debugAssertions": cfg!(debug_assertions),
        },
        "process": {
            "pid": std::process::id(),
            "affinity": process_affinity(),
        },
        "ops": ops,
        "finals": { "kind": measured.finals_kind, "values": measured.finals },
        "notes": measured.notes,
        "extra": measured.extra,
        "wallSeconds": started.elapsed().as_secs_f64(),
    });
    std::fs::write(
        &out_path,
        serde_json::to_string(&output).expect("output serializes"),
    )
    .expect("writable output");
}
