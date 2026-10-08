//! Supplemental native allocation probe; runtime behavior is unchanged.
//! Compare the same logical save before and after restore to expose retained
//! allocation capacity separately from last-event reports and archive payloads.
use caveat_runtime::reactive::ReactiveSession;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;
static LIVE: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            LIVE.fetch_add(layout.size(), Ordering::Relaxed);
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            LIVE.fetch_add(layout.size(), Ordering::Relaxed);
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
                LIVE.fetch_add(size - layout.size(), Ordering::Relaxed);
            } else {
                LIVE.fetch_sub(layout.size() - size, Ordering::Relaxed);
            }
        }
        result
    }
}
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

// These records contain no heap allocations. Producing one cannot inflate a
// later checkpoint; JSON strings/maps exist only during the completed call.
#[derive(Serialize)]
struct Checkpoint {
    requested_runtime_heap_bytes: usize,
    serialized_save_bytes: usize,
    save_sha256_bytes: [u8; 32],
    effects: usize,
    serialized_effects_bytes: usize,
    retired_dynamic_records: usize,
    withdrawals: usize,
    undrained: usize,
}
fn checkpoint(session: &ReactiveSession, base: usize) -> Checkpoint {
    let requested_runtime_heap_bytes = LIVE.load(Ordering::Relaxed).saturating_sub(base);
    let save = session.save_json().unwrap();
    let value: Value = serde_json::from_str(&save).unwrap();
    Checkpoint {
        requested_runtime_heap_bytes,
        serialized_save_bytes: save.len(),
        save_sha256_bytes: Sha256::digest(save.as_bytes()).into(),
        effects: value["effects"].as_array().map_or(0, Vec::len),
        serialized_effects_bytes: value
            .get("effects")
            .map_or(0, |effects| serde_json::to_vec(effects).unwrap().len()),
        retired_dynamic_records: value["retired"].as_object().map_or(0, |entries| {
            entries.keys().filter(|name| name.contains('@')).count()
        }),
        withdrawals: value["withdrawals"].as_array().map_or(0, Vec::len),
        undrained: session.undrained(),
    }
}
fn apply(session: &mut ReactiveSession, event: &str) {
    session.apply(event, &BTreeMap::new()).unwrap();
}
fn drain(session: &mut ReactiveSession) -> usize {
    let entries = session.drain_archive();
    let records = entries
        .iter()
        .filter(|entry| entry.record().is_some())
        .count();
    drop(entries);
    records
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "collector_release_memory SOURCE CYCLES");
    let mut source = std::fs::read_to_string(&args[1]).unwrap();
    source.push_str("\nevent collector_memory_noop;\n");
    let cycles: usize = args[2].parse().unwrap();
    assert!((1..=4000).contains(&cycles));
    let base = LIVE.load(Ordering::Relaxed);
    let mut session = ReactiveSession::from_source(&source).unwrap();
    for _ in 0..cycles {
        apply(&mut session, "cycle");
        assert_eq!(
            drain(&mut session),
            0,
            "own grounds hold every old group until release"
        );
    }
    let before_release = checkpoint(&session, base);
    apply(&mut session, "release");
    let archived_records = drain(&mut session);
    let after_release = checkpoint(&session, base);
    assert_eq!(after_release.retired_dynamic_records, 0);
    assert_eq!(archived_records, before_release.retired_dynamic_records);
    apply(&mut session, "collector_memory_noop");
    assert_eq!(drain(&mut session), 0);
    let after_noop = checkpoint(&session, base);
    assert_eq!(
        after_noop.effects, 0,
        "next accepted event clears the release report"
    );
    let same_save = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(&source, &same_save).unwrap();
    assert_eq!(
        restored.save_json().unwrap(),
        same_save,
        "restore preserves the identical logical save"
    );
    drop(session);
    drop(same_save);
    let after_restore = checkpoint(&restored, base);
    assert_eq!(
        after_restore.save_sha256_bytes,
        after_noop.save_sha256_bytes
    );
    drop(restored);
    let requested_bytes_after_dropping_session = LIVE.load(Ordering::Relaxed).saturating_sub(base);
    println!(
        "{}",
        json!({
            "schema":"caveat-collector-release-memory/1",
            "source":args[1],"cycles":cycles,
            "source_sha256":format!("{:x}",Sha256::digest(source.as_bytes())),
            "harness_sha256":format!("{:x}",Sha256::digest(include_bytes!("collector_release_memory.rs"))),
            "before_release":before_release,"after_release_and_drain":after_release,
            "after_next_accepted_noop_and_drain":after_noop,"after_same_save_restore_and_drop_original":after_restore,
            "archived_records":archived_records,"same_save_restore_equal":true,
            "requested_bytes_after_dropping_session":requested_bytes_after_dropping_session,
            "scope":"Single-threaded native requested Rust heap bytes relative to the pre-session baseline; excludes source/argument buffers, report serialization, allocator overhead, stacks and RSS. Every archive buffer is dropped before checkpoints. Checkpoint records allocate no retained heap. Not an isolated collector-only allocation count."
        })
    );
}
