//! Diagnostic-only actual-dispatch attribution. Not the unchanged target harness.
//! Per-block clocks and TLS stores perturb timing; target measurements disable this feature.
//! Allocator counts are requested Rust heap bytes, not RSS or allocator overhead.
//! Peak additional dispatch bytes bound collector temporary storage; they also
//! include transactional copies, ordinary departure, and the undrained archive.
//! The same harness is copied onto the frozen f5ec829 source for comparison.
#![allow(unexpected_cfgs)] // The frozen baseline predates collector-metrics.
use caveat_runtime::reactive::{ArchiveItem, ReactiveSession, WithdrawalExtractionProfile};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

struct CountingAllocator;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
fn allocated(size: usize) {
    let now = LIVE.fetch_add(size, Ordering::Relaxed) + size;
    PEAK.fetch_max(now, Ordering::Relaxed);
}
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
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

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
    extraction: WithdrawalExtractionProfile,
}
fn event(session: &mut ReactiveSession, name: &str, reject: bool) -> Sample {
    let before = LIVE.load(Ordering::Relaxed);
    PEAK.store(before, Ordering::Relaxed);
    let start = Instant::now();
    let outcome = session.apply(name, &BTreeMap::new());
    let ns = start.elapsed().as_nanos() as u64;
    let peak = PEAK.load(Ordering::Relaxed).saturating_sub(before);
    assert_eq!(outcome.is_err(), reject, "{name}: {outcome:?}");
    Sample {
        ns,
        peak_additional_bytes: peak,
        extraction: ReactiveSession::take_withdrawal_extraction_profile(),
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
        let before = LIVE.load(Ordering::Relaxed);
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
        self.max_freed = self
            .max_freed
            .max(before.saturating_sub(LIVE.load(Ordering::Relaxed)));
    }
}

fn summary(samples: &[Sample]) -> serde_json::Value {
    if samples.is_empty() {
        return json!({"samples": 0});
    }
    let mut times: Vec<_> = samples.iter().map(|x| x.ns).collect();
    times.sort_unstable();
    let total = |get: fn(&Work) -> usize| samples.iter().map(|x| get(&x.work)).sum::<usize>();
    let extraction_total = |get: fn(&WithdrawalExtractionProfile) -> u64| {
        samples
            .iter()
            .map(|sample| get(&sample.extraction))
            .sum::<u64>()
    };
    json!({"extraction": {
        "blocks":extraction_total(|p|p.blocks),"matches":extraction_total(|p|p.matches),"misses":extraction_total(|p|p.misses),
        "probes":extraction_total(|p|p.probes),"shifted_elements":extraction_total(|p|p.shifted_elements),
        "estimated_shifted_bytes":extraction_total(|p|p.estimated_shifted_bytes),"shared_cow_detaches":extraction_total(|p|p.shared_cow_detaches),
        "cow_cloned_elements":extraction_total(|p|p.cow_cloned_elements),"block_ns":extraction_total(|p|p.block_ns),
        "departure_ns":extraction_total(|p|p.departure_ns),
        "small_phase_samples":if samples.len()<=3 {Some(samples)} else {None}
        },"samples": samples.len(), "median_ns": times[times.len()/2],
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
    let live_heap = LIVE.load(Ordering::Relaxed).saturating_sub(base);
    let saved = session.save_json().unwrap();
    let value: serde_json::Value = serde_json::from_str(&saved).unwrap();
    json!({"serialized_save_bytes":saved.len(), "save_sha256":format!("{:x}",Sha256::digest(saved.as_bytes())),
        "retained_rust_heap_bytes":live_heap,
        "retired_dynamic_records":value["retired"].as_object().unwrap().keys().filter(|name|name.contains('@')).count(),
        "withdrawals":value["withdrawals"].as_array().map_or(0,Vec::len), "undrained":session.undrained()})
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        6,
        "collector_profile SOURCE CYCLES MODE DRAIN_EVERY PROBES"
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
    let mut sink = ArchiveSink::new();
    let base_heap = LIVE.load(Ordering::Relaxed);
    let mut session = ReactiveSession::from_source(&source).unwrap();
    if mode == "initialize" {
        event(&mut session, "initialize", false);
        sink.drain(&mut session);
    }
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
    let after_report_heap = LIVE.load(Ordering::Relaxed);
    let before_report_heap =
        base_heap + before_release["retained_rust_heap_bytes"].as_u64().unwrap() as usize;
    let report_heap = after_report_heap.saturating_sub(before_report_heap);
    let mut released = Vec::with_capacity(1);
    let release_buffer_heap = LIVE
        .load(Ordering::Relaxed)
        .saturating_sub(after_report_heap);
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
    let after_release = retained(&session, base_heap + report_heap + release_buffer_heap);
    println!(
        "{}",
        json!({"schema":"caveat-withdrawal-extraction-diagnostic/1", "diagnostic_only":true, "source":args[1],
        "source_sha256":format!("{:x}",Sha256::digest(source.as_bytes())), "cycles":cycles,"mode":mode,
        "drain_every":drain_every,"instrumented":cfg!(feature="collector-metrics"),
        "growth":summary(&growth),"growth_last_10_percent":summary(&growth[cycles*9/10..]),
        "steady":summary(&steady),"unrelated_changes":summary(&unrelated),"failed_release":summary(&failures),"release":summary(&released),
        "before_release":before_release,"after_release":after_release,
        "archive":{"growth_ndjson_bytes":growth_archive.0,"growth_records":growth_archive.1,"growth_nodes":growth_archive.2,"ndjson_bytes":sink.bytes,"records":sink.records,"provenance_nodes":sink.nodes,
            "ndjson_sha256":format!("{:x}",sink.hash.finalize()),"max_undrained_items":sink.max_queue,
            "items_at_growth_end":queue_at_growth_end,"max_heap_freed_by_drain":sink.max_freed},
        "memory_scope":"requested Rust heap bytes; excludes allocator overhead, stacks and RSS; dispatch peak includes transaction/archive and upper-bounds collector temporary allocations; exact collector-only bytes not isolated",
        "latency_scope":"native apply transaction, no snapshot serialization; allocator accounting and optional work counters enabled; short release sample p99 is descriptive, not a reliable tail estimate"})
    );
}
