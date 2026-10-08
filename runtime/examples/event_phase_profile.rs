//! Fixed event-boundary diagnostics using the original collector probe sequence.
//! Timing mode retains the original counting allocator implementation. Origins
//! mode is a separate build with header/padding overhead and no runtime phase
//! clocks. Requested allocation ownership is not physical memory-copy traffic.
#[cfg(feature = "event-phase-origins")]
use caveat_runtime::reactive::current_event_phase_index;
use caveat_runtime::reactive::{
    ArchiveItem, EventPhaseProfile, ReactiveSession, EVENT_PHASE_COUNT, EVENT_PHASE_NAMES,
};
#[cfg(feature = "event-phase-origins")]
#[path = "event_phase_support/origin_allocator.rs"]
mod origin_allocator;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::alloc::System;
#[cfg(not(feature = "event-phase-origins"))]
use std::alloc::{GlobalAlloc, Layout};
use std::collections::BTreeMap;
use std::io::{self, Write};
#[cfg(not(feature = "event-phase-origins"))]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

#[cfg(not(feature = "event-phase-origins"))]
struct CountingAllocator;
#[cfg(not(feature = "event-phase-origins"))]
static LIVE: AtomicUsize = AtomicUsize::new(0);
#[cfg(not(feature = "event-phase-origins"))]
static PEAK: AtomicUsize = AtomicUsize::new(0);
#[cfg(not(feature = "event-phase-origins"))]
fn allocated(size: usize) {
    let now = LIVE.fetch_add(size, Ordering::Relaxed) + size;
    PEAK.fetch_max(now, Ordering::Relaxed);
}
#[cfg(not(feature = "event-phase-origins"))]
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            allocated(layout.size());
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            allocated(layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(pointer, layout) };
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(pointer, layout, size) };
        if !result.is_null() {
            if size >= layout.size() {
                allocated(size - layout.size());
            } else {
                LIVE.fetch_sub(layout.size() - size, Ordering::Relaxed);
            }
        }
        result
    }
}
#[cfg(not(feature = "event-phase-origins"))]
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[cfg(feature = "event-phase-origins")]
#[global_allocator]
static ALLOCATOR: origin_allocator::OriginAllocator<System> =
    origin_allocator::OriginAllocator::new(System, current_event_phase_index);

fn live_requested() -> usize {
    #[cfg(not(feature = "event-phase-origins"))]
    {
        LIVE.load(Ordering::Relaxed)
    }
    #[cfg(feature = "event-phase-origins")]
    {
        ALLOCATOR.live_requested()
    }
}

#[derive(Default, Serialize, Clone, Copy)]
struct Work {
    dirty_owners: usize,
    references_scanned: usize,
    vertices_visited: usize,
    edges_visited: usize,
    candidates_examined: usize,
    collected: usize,
    peak_work_items: usize,
}
fn work(session: &ReactiveSession) -> Work {
    #[cfg(feature = "collector-metrics")]
    {
        let metrics = session.withdrawal_collection_metrics();
        Work {
            dirty_owners: metrics.dirty_owners,
            references_scanned: metrics.references_scanned,
            vertices_visited: metrics.vertices_visited,
            edges_visited: metrics.edges_visited,
            candidates_examined: metrics.candidates_examined,
            collected: metrics.collected,
            peak_work_items: metrics.peak_work_items,
        }
    }
    #[cfg(not(feature = "collector-metrics"))]
    {
        let _ = session;
        Work::default()
    }
}

#[derive(Serialize)]
struct Sample {
    ns: u64,
    peak_additional_bytes: usize,
    work: Work,
    phases: EventPhaseProfile,
    phase_total_ns: u64,
    /// Timing-only residual includes instrumentation and unmeasured API edges.
    /// Origins mode has no phase clocks, so this is None, not zero phase cost.
    remainder_ns: Option<u64>,
    #[cfg(feature = "event-phase-origins")]
    allocations: origin_allocator::Snapshot,
    #[cfg(not(feature = "event-phase-origins"))]
    allocations: Option<()>,
}
fn event(session: &mut ReactiveSession, name: &str, reject: bool) -> Sample {
    #[cfg(not(feature = "event-phase-origins"))]
    let before = LIVE.load(Ordering::Relaxed);
    #[cfg(not(feature = "event-phase-origins"))]
    PEAK.store(before, Ordering::Relaxed);
    ReactiveSession::begin_event_phase_profile();
    #[cfg(feature = "event-phase-origins")]
    ALLOCATOR.begin_event();
    let start = Instant::now();
    let outcome = session.apply(name, &BTreeMap::new());
    let ns = start.elapsed().as_nanos() as u64;
    #[cfg(not(feature = "event-phase-origins"))]
    let peak = PEAK.load(Ordering::Relaxed).saturating_sub(before);
    #[cfg(feature = "event-phase-origins")]
    let allocations = ALLOCATOR.finish_event();
    #[cfg(feature = "event-phase-origins")]
    let peak = allocations.requested_additional_peak_bytes;
    let phases = ReactiveSession::take_event_phase_profile();
    let phase_total_ns = phases.phase_ns.iter().sum();
    let remainder_ns = if phases.clock_enabled {
        Some(
            ns.checked_sub(phase_total_ns)
                .expect("phase intervals overlap or exceed apply"),
        )
    } else {
        assert_eq!(phase_total_ns, 0);
        None
    };
    assert_eq!(outcome.is_err(), reject, "{name}: {outcome:?}");
    Sample {
        ns,
        peak_additional_bytes: peak,
        phases,
        phase_total_ns,
        remainder_ns,
        #[cfg(feature = "event-phase-origins")]
        allocations,
        #[cfg(not(feature = "event-phase-origins"))]
        allocations: None,
        work: if reject {
            Work::default()
        } else {
            work(session)
        },
    }
}

struct ArchiveSink {
    hash: Sha256,
    bytes: u64,
    records: usize,
    nodes: usize,
    max_queue: usize,
    max_freed: usize,
}
impl Write for ArchiveSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.hash.update(bytes);
        self.bytes += bytes.len() as u64;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl ArchiveSink {
    fn new() -> Self {
        Self {
            hash: Sha256::new(),
            bytes: 0,
            records: 0,
            nodes: 0,
            max_queue: 0,
            max_freed: 0,
        }
    }
    fn drain(&mut self, session: &mut ReactiveSession) {
        self.max_queue = self.max_queue.max(session.undrained());
        let before = live_requested();
        let archive = session.drain_archive();
        for item in &archive {
            match item {
                ArchiveItem::Record(_) => self.records += 1,
                ArchiveItem::Provenance(_) => self.nodes += 1,
            }
            serde_json::to_writer(&mut *self, item).unwrap();
            self.write_all(b"\n").unwrap();
        }
        drop(archive);
        self.max_freed = self.max_freed.max(before.saturating_sub(live_requested()));
    }
}

fn summary(samples: &[Sample]) -> serde_json::Value {
    if samples.is_empty() {
        return json!({"samples": 0});
    }
    let mut times: Vec<_> = samples.iter().map(|x| x.ns).collect();
    times.sort_unstable();
    let total = |get: fn(&Work) -> usize| samples.iter().map(|x| get(&x.work)).sum::<usize>();
    json!({"samples": samples.len(), "median_ns": times[times.len()/2],
        "p99_ns": times[(times.len()*99).div_ceil(100).saturating_sub(1)], "max_ns": times.last(),
        "mean_ns": times.iter().map(|n| *n as u128).sum::<u128>() / times.len() as u128,
        "max_dispatch_additional_heap_bytes": samples.iter().map(|x| x.peak_additional_bytes).max(),
        "collector_work": {"dirty_owners":total(|m|m.dirty_owners), "references_scanned":total(|m|m.references_scanned),
            "vertices_visited":total(|m|m.vertices_visited), "edges_visited":total(|m|m.edges_visited),
            "candidates_examined":total(|m|m.candidates_examined), "collected":total(|m|m.collected),
            "max_work_items":samples.iter().map(|x|x.work.peak_work_items).max()}})
}

fn retained(session: &ReactiveSession, base: usize) -> serde_json::Value {
    // Capture before allocating the save/report representation.
    let live_heap = live_requested().saturating_sub(base);
    let saved = session.save_json().unwrap();
    let value: serde_json::Value = serde_json::from_str(&saved).unwrap();
    json!({"serialized_save_bytes":saved.len(), "save_sha256":format!("{:x}",Sha256::digest(saved.as_bytes())),
        "retained_rust_heap_bytes":live_heap,
        "retired_dynamic_records":value["retired"].as_object().unwrap().keys().filter(|name|name.contains('@')).count(),
        "withdrawals":value["withdrawals"].as_array().map_or(0,Vec::len), "undrained":session.undrained()})
}

fn origin_layout() -> serde_json::Value {
    #[cfg(feature = "event-phase-origins")]
    {
        json!({"header_bytes":origin_allocator::HEADER_BYTES,
        "origin_buckets":origin_allocator::ORIGIN_BUCKETS,
        "prior_event_index":origin_allocator::PRIOR_EVENT,
        "origin_phase_names":EVENT_PHASE_NAMES,
        "peak_tie_rule":"first strict maximum; null phase means the event-start baseline remained the maximum",
        "outside_phase":"allocations during active event outside a named phase; other threads use outside if present"})
    }
    #[cfg(not(feature = "event-phase-origins"))]
    {
        serde_json::Value::Null
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        7,
        "event_phase_profile SOURCE CYCLES MODE DRAIN_EVERY PROBES timing|origins"
    );
    let profiling_mode = if cfg!(feature = "event-phase-origins") {
        "origins"
    } else {
        "timing"
    };
    assert_eq!(
        args[6], profiling_mode,
        "allocator mode is fixed at compile time"
    );
    let mut source = std::fs::read_to_string(&args[1]).unwrap();
    source.push_str("\nevent profile_probe;\nevidence profile_side from \"profile side observation\"; renewable profile_side window 1;\nevent profile_mutate;\non profile_mutate renew profile_side;\non profile_mutate reveal profile_side supports seen;\n");
    let cycles: usize = args[2].parse().unwrap();
    let mode = args[3].as_str();
    let drain_every: usize = args[4].parse().unwrap();
    let probes: usize = args[5].parse().unwrap();
    assert!(cycles > 0 && cycles <= 4000 && drain_every > 0 && probes <= 10000);
    let mut growth = Vec::with_capacity(cycles);
    let mut steady = Vec::with_capacity(probes);
    let mut unrelated = Vec::with_capacity(probes);
    let mut failures = Vec::with_capacity(3);
    let mut released = Vec::with_capacity(1);
    let sample_buffer_requested_bytes = (growth.capacity()
        + steady.capacity()
        + unrelated.capacity()
        + failures.capacity()
        + released.capacity())
        * std::mem::size_of::<Sample>();
    let mut sink = ArchiveSink::new();
    let base_heap = live_requested();
    let mut session = ReactiveSession::from_source(&source).unwrap();
    let initialization = if mode == "initialize" {
        let sample = event(&mut session, "initialize", false);
        sink.drain(&mut session);
        Some(sample)
    } else {
        None
    };
    for n in 1..=cycles {
        growth.push(event(&mut session, "cycle", false));
        if n % drain_every == 0 {
            sink.drain(&mut session);
        }
    }
    let queue_at_growth_end = session.undrained();
    sink.drain(&mut session);
    let growth_archive = (sink.bytes, sink.records, sink.nodes);
    for _ in 0..probes {
        steady.push(event(&mut session, "profile_probe", false));
        sink.drain(&mut session);
    }
    for _ in 0..probes {
        unrelated.push(event(&mut session, "profile_mutate", false));
        sink.drain(&mut session);
    }
    let before_release = retained(&session, base_heap);
    // Keep this report allocation outside the session memory comparison.
    let after_report_heap = live_requested();
    let before_report_heap =
        base_heap + before_release["retained_rust_heap_bytes"].as_u64().unwrap() as usize;
    let report_heap = after_report_heap.saturating_sub(before_report_heap);
    if mode == "release" {
        let saved = session.save_json().unwrap();
        for _ in 0..3 {
            failures.push(event(&mut session, "fail", true));
            assert_eq!(
                session.save_json().unwrap(),
                saved,
                "failed release mutated save"
            );
            assert_eq!(session.undrained(), 0, "failed release published archive");
        }
        drop(saved);
        released.push(event(&mut session, "release", false));
        sink.drain(&mut session);
    }
    let after_release = retained(&session, base_heap + report_heap);
    println!(
        "{}",
        json!({"schema":"caveat-event-phase-native-profile/1", "profiling_mode":profiling_mode, "source":args[1],
        "phase_names":EVENT_PHASE_NAMES, "phase_count":EVENT_PHASE_COUNT,
        "sample_buffer_requested_bytes":sample_buffer_requested_bytes,
        "harness_baseline_requested_bytes":base_heap,
        "origin_layout":origin_layout(),
        "event_samples":{"initialize":initialization,"growth":growth,"steady":steady,
            "unrelated_changes":unrelated,"failed_release":failures,"release":released},
        "source_sha256":format!("{:x}",Sha256::digest(source.as_bytes())), "cycles":cycles,"mode":mode,
        "drain_every":drain_every,"instrumented":cfg!(feature="collector-metrics"),
        "growth":summary(&growth),"growth_last_10_percent":summary(&growth[cycles*9/10..]),
        "steady":summary(&steady),"unrelated_changes":summary(&unrelated),"failed_release":summary(&failures),"release":summary(&released),
        "before_release":before_release,"after_release":after_release,
        "archive":{"growth_ndjson_bytes":growth_archive.0,"growth_records":growth_archive.1,"growth_nodes":growth_archive.2,"ndjson_bytes":sink.bytes,"records":sink.records,"provenance_nodes":sink.nodes,
            "ndjson_sha256":format!("{:x}",sink.hash.finalize()),"max_undrained_items":sink.max_queue,
            "items_at_growth_end":queue_at_growth_end,"max_heap_freed_by_drain":sink.max_freed},
        "memory_scope":"requested Rust heap includes requested container capacity; excludes allocator free lists, metadata, stack and RSS. Origin headers/padding are reported separately. Global event peak includes pre-event session and harness bytes: use event-start baseline for additional peak; prior_event includes fixed harness buffers. The origin partition is at one global peak, not a sum of phase peaks.",
        "latency_scope":"native apply includes fixed phase bookkeeping. Origins mode disables runtime phase clocks but performs per-allocation TLS phase queries, header/counter work and fixed-array peak copies inside apply. Only begin/reset and final profile extraction/snapshot return are outside the apply clock, along with later report serialization and archive drain. Raw arrays are emitted only after all events. No stable-tail or predicted improvement claim.",
        "realloc_convention":"successful realloc debits the entire old requested block from its recorded origin and credits the entire new request to the current event/phase, even in-place; failure preserves old origin. Counts are logical requests, not copied bytes or hidden transient System realloc storage. Prior epochs collapse into prior_event at event start; no per-object histories or maps."})
    );
}
