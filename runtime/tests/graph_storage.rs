//! Retiring graph nodes must release their storage even though node IDs are
//! never reused. A small live graph must not keep a slot for every past node.
use caveat_runtime::{EpistemicGraph, NodeKind};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};

struct Counted;
static LIVE_BYTES: AtomicIsize = AtomicIsize::new(0);

// SAFETY: the allocator forwards the unchanged pointers, sizes and alignments
// to System. The counter records only successful allocations and changes no
// allocation's ownership or lifetime.
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            LIVE_BYTES.fetch_add(layout.size() as isize, Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            LIVE_BYTES.fetch_add(layout.size() as isize, Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let resized = unsafe { System.realloc(pointer, layout, size) };
        if !resized.is_null() {
            LIVE_BYTES.fetch_add(size as isize - layout.size() as isize, Ordering::Relaxed);
        }
        resized
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        LIVE_BYTES.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counted = Counted;

fn next_node(graph: &mut EpistemicGraph) -> u64 {
    let id = graph.add(NodeKind::Observation {
        description: "current observation".into(),
    });
    if id > 4 {
        assert!(graph.nodes.remove(&(id - 4)).is_some());
    }
    id
}

#[test]
fn retired_nodes_do_not_leave_storage_proportional_to_all_past_ids() {
    let mut graph = EpistemicGraph::new();
    for _ in 0..1_000 {
        next_node(&mut graph);
    }
    assert_eq!(graph.nodes.len(), 4);
    let before = LIVE_BYTES.load(Ordering::Relaxed);
    let mut last = 0;
    for _ in 0..10_000 {
        last = next_node(&mut graph);
    }
    let growth = LIVE_BYTES.load(Ordering::Relaxed) - before;
    assert_eq!(last, 11_000, "retirement must not recycle node IDs");
    assert_eq!(graph.nodes.len(), 4);
    for id in last - 3..=last {
        assert!(graph.nodes.contains_key(&id));
    }
    // This is a retained-storage bound, not a zero-allocation requirement or
    // a timing budget. Allow ample allocator/test-harness slack; retaining one
    // slot per retired ID grows by hundreds of kilobytes on this history.
    assert!(
        growth < 64 * 1024,
        "four live nodes retained {growth} more bytes after 10,000 retirements"
    );
}
