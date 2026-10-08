//! Diagnostic-only allocation origins. No maps, allocation, formatting or clocks
//! in the recorder. The ordinary timing build does not compile this module.
use caveat_runtime::reactive::EVENT_PHASE_COUNT;
use serde::Serialize;
use std::alloc::{GlobalAlloc, Layout};
use std::cell::UnsafeCell;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

pub const ORIGIN_BUCKETS: usize = EVENT_PHASE_COUNT + 1;
pub const PRIOR_EVENT: usize = EVENT_PHASE_COUNT;

#[derive(Clone, Copy)]
#[repr(C)]
struct Header {
    epoch: u64,
    phase: usize,
}

pub const HEADER_BYTES: usize = std::mem::size_of::<Header>();

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Flow {
    pub alloc_calls: usize,
    pub alloc_zeroed_calls: usize,
    pub realloc_calls: usize,
    pub dealloc_calls: usize,
    pub allocation_failures: usize,
    pub realloc_failures: usize,
    /// Successful realloc counts a full replacement, not just its growth.
    pub requested_allocated_bytes: usize,
    pub requested_freed_bytes: usize,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Snapshot {
    pub epoch: u64,
    pub event_start_requested_live_bytes: usize,
    pub event_start_system_requested_live_bytes: usize,
    pub requested_live_at_peak_bytes: usize,
    pub requested_additional_peak_bytes: usize,
    /// None means the event-start baseline remained the first global maximum.
    pub requested_peak_phase: Option<usize>,
    pub requested_live_by_origin_at_peak: [usize; ORIGIN_BUCKETS],
    pub live_allocations_by_origin_at_peak: [usize; ORIGIN_BUCKETS],
    pub metadata_bytes_at_requested_peak: usize,
    pub alignment_padding_bytes_at_requested_peak: usize,
    pub system_requested_bytes_at_requested_peak: usize,
    /// This independent peak includes header/padding requests to System.
    pub system_requested_peak_bytes: usize,
    pub system_requested_additional_peak_bytes: usize,
    pub system_requested_peak_phase: Option<usize>,
    pub requested_live_by_origin_at_system_peak: [usize; ORIGIN_BUCKETS],
    pub metadata_bytes_at_system_peak: usize,
    pub alignment_padding_bytes_at_system_peak: usize,
    pub requested_live_at_end_bytes: usize,
    pub requested_live_by_origin_at_end: [usize; ORIGIN_BUCKETS],
    pub system_requested_live_at_end_bytes: usize,
    pub flow_by_operation_phase: [Flow; EVENT_PHASE_COUNT],
}

const ZERO_FLOW: Flow = Flow {
    alloc_calls: 0,
    alloc_zeroed_calls: 0,
    realloc_calls: 0,
    dealloc_calls: 0,
    allocation_failures: 0,
    realloc_failures: 0,
    requested_allocated_bytes: 0,
    requested_freed_bytes: 0,
};

struct State {
    epoch: u64,
    active: bool,
    requested: usize,
    system_requested: usize,
    live_count: usize,
    by_origin: [usize; ORIGIN_BUCKETS],
    counts_by_origin: [usize; ORIGIN_BUCKETS],
    start_requested: usize,
    start_system: usize,
    peak_requested: usize,
    peak_phase: Option<usize>,
    peak_origins: [usize; ORIGIN_BUCKETS],
    peak_counts: [usize; ORIGIN_BUCKETS],
    peak_system_at_requested: usize,
    peak_system: usize,
    peak_system_phase: Option<usize>,
    peak_system_origins: [usize; ORIGIN_BUCKETS],
    peak_system_count: usize,
    flows: [Flow; EVENT_PHASE_COUNT],
}

impl State {
    const fn new() -> Self {
        Self {
            epoch: 0,
            active: false,
            requested: 0,
            system_requested: 0,
            live_count: 0,
            by_origin: [0; ORIGIN_BUCKETS],
            counts_by_origin: [0; ORIGIN_BUCKETS],
            start_requested: 0,
            start_system: 0,
            peak_requested: 0,
            peak_phase: None,
            peak_origins: [0; ORIGIN_BUCKETS],
            peak_counts: [0; ORIGIN_BUCKETS],
            peak_system_at_requested: 0,
            peak_system: 0,
            peak_system_phase: None,
            peak_system_origins: [0; ORIGIN_BUCKETS],
            peak_system_count: 0,
            flows: [ZERO_FLOW; EVENT_PHASE_COUNT],
        }
    }

    fn origin(&self, header: Header) -> usize {
        if header.epoch == self.epoch && header.epoch != 0 {
            header.phase
        } else {
            PRIOR_EVENT
        }
    }

    fn header(&self, phase: usize) -> Header {
        Header {
            epoch: if self.active { self.epoch } else { 0 },
            phase,
        }
    }

    fn add(&mut self, header: Header, requested: usize, system: usize) {
        let origin = self.origin(header);
        self.requested += requested;
        self.system_requested += system;
        self.live_count += 1;
        self.by_origin[origin] += requested;
        self.counts_by_origin[origin] += 1;
    }

    fn remove(&mut self, header: Header, requested: usize, system: usize) {
        let origin = self.origin(header);
        self.requested -= requested;
        self.system_requested -= system;
        self.live_count -= 1;
        self.by_origin[origin] -= requested;
        self.counts_by_origin[origin] -= 1;
    }

    fn peak(&mut self, phase: usize) {
        if !self.active {
            return;
        }
        // Keep the first strict maximum. No allocation occurs to copy arrays.
        if self.requested > self.peak_requested {
            self.peak_requested = self.requested;
            self.peak_phase = Some(phase);
            self.peak_origins = self.by_origin;
            self.peak_counts = self.counts_by_origin;
            self.peak_system_at_requested = self.system_requested;
        }
        if self.system_requested > self.peak_system {
            self.peak_system = self.system_requested;
            self.peak_system_phase = Some(phase);
            self.peak_system_origins = self.by_origin;
            self.peak_system_count = self.live_count;
        }
    }

    fn begin(&mut self) {
        // Epoch rollover would make stale headers ambiguous. Impossible in the
        // bounded harness, but fail without allocation or unwinding if reached.
        if self.epoch == u64::MAX || self.active {
            std::process::abort();
        }
        self.epoch += 1;
        self.active = true;
        self.by_origin = [0; ORIGIN_BUCKETS];
        self.counts_by_origin = [0; ORIGIN_BUCKETS];
        self.by_origin[PRIOR_EVENT] = self.requested;
        self.counts_by_origin[PRIOR_EVENT] = self.live_count;
        self.start_requested = self.requested;
        self.start_system = self.system_requested;
        self.peak_requested = self.requested;
        self.peak_system = self.system_requested;
        self.peak_system_at_requested = self.system_requested;
        self.peak_phase = None;
        self.peak_system_phase = None;
        self.peak_origins = self.by_origin;
        self.peak_counts = self.counts_by_origin;
        self.peak_system_origins = self.by_origin;
        self.peak_system_count = self.live_count;
        self.flows = [ZERO_FLOW; EVENT_PHASE_COUNT];
    }

    fn finish(&mut self) -> Snapshot {
        let header_size = std::mem::size_of::<Header>();
        let metadata = self.peak_counts.iter().sum::<usize>() * header_size;
        let system_peak_metadata = self.peak_system_count * header_size;
        self.active = false;
        Snapshot {
            epoch: self.epoch,
            event_start_requested_live_bytes: self.start_requested,
            event_start_system_requested_live_bytes: self.start_system,
            requested_live_at_peak_bytes: self.peak_requested,
            requested_additional_peak_bytes: self.peak_requested - self.start_requested,
            requested_peak_phase: self.peak_phase,
            requested_live_by_origin_at_peak: self.peak_origins,
            live_allocations_by_origin_at_peak: self.peak_counts,
            metadata_bytes_at_requested_peak: metadata,
            alignment_padding_bytes_at_requested_peak: self.peak_system_at_requested
                - self.peak_requested
                - metadata,
            system_requested_bytes_at_requested_peak: self.peak_system_at_requested,
            system_requested_peak_bytes: self.peak_system,
            system_requested_additional_peak_bytes: self.peak_system - self.start_system,
            system_requested_peak_phase: self.peak_system_phase,
            requested_live_by_origin_at_system_peak: self.peak_system_origins,
            metadata_bytes_at_system_peak: system_peak_metadata,
            alignment_padding_bytes_at_system_peak: self.peak_system
                - self.peak_system_origins.iter().sum::<usize>()
                - system_peak_metadata,
            requested_live_at_end_bytes: self.requested,
            requested_live_by_origin_at_end: self.by_origin,
            system_requested_live_at_end_bytes: self.system_requested,
            flow_by_operation_phase: self.flows,
        }
    }
}

struct LockedState {
    locked: AtomicBool,
    value: UnsafeCell<State>,
}

// Every State access is serialized. System and the phase getter run outside the
// lock; the critical section uses only scalar arithmetic and fixed-array copies.
unsafe impl Sync for LockedState {}

impl LockedState {
    const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(State::new()),
        }
    }

    fn with<R>(&self, operation: impl FnOnce(&mut State) -> R) -> R {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }
        struct Unlock<'a>(&'a AtomicBool);
        impl Drop for Unlock<'_> {
            fn drop(&mut self) {
                self.0.store(false, Ordering::Release);
            }
        }
        let _unlock = Unlock(&self.locked);
        operation(unsafe { &mut *self.value.get() })
    }
}

pub struct OriginAllocator<A> {
    allocator: A,
    phase: fn() -> usize,
    state: LockedState,
}

impl<A> OriginAllocator<A> {
    pub const fn new(allocator: A, phase: fn() -> usize) -> Self {
        Self {
            allocator,
            phase,
            state: LockedState::new(),
        }
    }

    pub fn live_requested(&self) -> usize {
        self.state.with(|s| s.requested)
    }

    pub fn begin_event(&self) {
        self.state.with(State::begin);
    }

    pub fn finish_event(&self) -> Snapshot {
        self.state.with(State::finish)
    }

    fn phase(&self) -> usize {
        let phase = (self.phase)();
        if phase < EVENT_PHASE_COUNT {
            phase
        } else {
            0
        }
    }
}

fn layout_with_header(layout: Layout) -> Option<(Layout, usize)> {
    Layout::new::<Header>()
        .extend(layout)
        .ok()
        .map(|(layout, offset)| (layout.pad_to_align(), offset))
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for OriginAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.allocate(layout, false) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { self.allocate(layout, true) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        let Some((combined, offset)) = layout_with_header(layout) else {
            std::process::abort();
        };
        let base = unsafe { pointer.sub(offset) };
        let header = unsafe { base.cast::<Header>().read() };
        let phase = self.phase();
        self.state.with(|s| {
            s.remove(header, layout.size(), combined.size());
            if s.active {
                s.flows[phase].dealloc_calls += 1;
                s.flows[phase].requested_freed_bytes += layout.size();
            }
        });
        unsafe { self.allocator.dealloc(base, combined) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let phase = self.phase();
        let Some((old_layout, offset)) = layout_with_header(layout) else {
            std::process::abort();
        };
        let new_layout = Layout::from_size_align(size, layout.align())
            .ok()
            .and_then(layout_with_header);
        let base = unsafe { pointer.sub(offset) };
        let old_header = unsafe { base.cast::<Header>().read() };
        let Some((combined, new_offset)) = new_layout else {
            self.failed_realloc(phase);
            return ptr::null_mut();
        };
        // Offset depends only on alignment, which GlobalAlloc::realloc preserves.
        if offset != new_offset {
            std::process::abort();
        }
        let result = unsafe { self.allocator.realloc(base, old_layout, combined.size()) };
        if result.is_null() {
            self.failed_realloc(phase);
            return result;
        }
        self.state.with(|s| {
            s.remove(old_header, layout.size(), old_layout.size());
            let header = s.header(phase);
            unsafe { result.cast::<Header>().write(header) };
            s.add(header, size, combined.size());
            if s.active {
                s.flows[phase].realloc_calls += 1;
                s.flows[phase].requested_allocated_bytes += size;
                s.flows[phase].requested_freed_bytes += layout.size();
                s.peak(phase);
            }
        });
        unsafe { result.add(offset) }
    }
}

impl<A: GlobalAlloc> OriginAllocator<A> {
    unsafe fn allocate(&self, layout: Layout, zeroed: bool) -> *mut u8 {
        let phase = self.phase();
        let combined = layout_with_header(layout);
        let base = match combined {
            Some((combined, _)) => {
                if zeroed {
                    unsafe { self.allocator.alloc_zeroed(combined) }
                } else {
                    unsafe { self.allocator.alloc(combined) }
                }
            }
            None => ptr::null_mut(),
        };
        self.state.with(|s| {
            if s.active {
                if zeroed {
                    s.flows[phase].alloc_zeroed_calls += 1;
                } else {
                    s.flows[phase].alloc_calls += 1;
                }
            }
            if base.is_null() {
                if s.active {
                    s.flows[phase].allocation_failures += 1;
                }
                return;
            }
            let header = s.header(phase);
            unsafe { base.cast::<Header>().write(header) };
            s.add(header, layout.size(), combined.unwrap().0.size());
            if s.active {
                s.flows[phase].requested_allocated_bytes += layout.size();
                s.peak(phase);
            }
        });
        if base.is_null() {
            base
        } else {
            unsafe { base.add(combined.unwrap().1) }
        }
    }

    fn failed_realloc(&self, phase: usize) {
        self.state.with(|s| {
            if s.active {
                s.flows[phase].realloc_calls += 1;
                s.flows[phase].realloc_failures += 1;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::System;
    use std::cell::Cell;

    thread_local! { static PHASE: Cell<usize> = const { Cell::new(0) }; }
    fn phase() -> usize {
        PHASE.get()
    }
    fn set_phase(value: usize) {
        PHASE.set(value);
    }

    fn partition(snapshot: &Snapshot) {
        assert_eq!(
            snapshot
                .requested_live_by_origin_at_peak
                .iter()
                .sum::<usize>(),
            snapshot.requested_live_at_peak_bytes
        );
        assert_eq!(
            snapshot
                .requested_live_by_origin_at_end
                .iter()
                .sum::<usize>(),
            snapshot.requested_live_at_end_bytes
        );
        assert_eq!(
            snapshot.requested_live_at_peak_bytes
                + snapshot.metadata_bytes_at_requested_peak
                + snapshot.alignment_padding_bytes_at_requested_peak,
            snapshot.system_requested_bytes_at_requested_peak
        );
        assert_eq!(
            snapshot
                .requested_live_by_origin_at_system_peak
                .iter()
                .sum::<usize>()
                + snapshot.metadata_bytes_at_system_peak
                + snapshot.alignment_padding_bytes_at_system_peak,
            snapshot.system_requested_peak_bytes
        );
    }

    #[test]
    fn aligned_zeroed_allocations_partition_and_free() {
        let allocator = OriginAllocator::new(System, phase);
        for align in [1, 2, 8, 16, 64, 4096] {
            set_phase(2);
            allocator.begin_event();
            let layout = Layout::from_size_align(37, align).unwrap();
            let p = unsafe { allocator.alloc_zeroed(layout) };
            assert!(!p.is_null());
            assert_eq!(p as usize % align, 0);
            assert!(unsafe { std::slice::from_raw_parts(p, 37) }
                .iter()
                .all(|v| *v == 0));
            unsafe { allocator.dealloc(p, layout) };
            let s = allocator.finish_event();
            partition(&s);
            assert_eq!(s.requested_live_at_peak_bytes, 37);
            assert_eq!(s.requested_live_by_origin_at_peak[2], 37);
            assert_eq!(s.requested_live_at_end_bytes, 0);
            assert_eq!(s.system_requested_live_at_end_bytes, 0);
            assert_eq!(s.flow_by_operation_phase[2].alloc_zeroed_calls, 1);
            assert_eq!(s.flow_by_operation_phase[2].dealloc_calls, 1);
        }
    }

    #[test]
    fn realloc_replaces_origin_preserves_bytes_and_accounts_growth_shrink() {
        let allocator = OriginAllocator::new(System, phase);
        allocator.begin_event();
        set_phase(1);
        let initial = Layout::from_size_align(31, 64).unwrap();
        let p = unsafe { allocator.alloc(initial) };
        assert!(!p.is_null());
        unsafe { p.write_bytes(0x5a, 31) };
        set_phase(4);
        let p = unsafe { allocator.realloc(p, initial, 101) };
        assert!(!p.is_null());
        assert_eq!(p as usize % 64, 0);
        assert!(unsafe { std::slice::from_raw_parts(p, 31) }
            .iter()
            .all(|v| *v == 0x5a));
        let middle = Layout::from_size_align(101, 64).unwrap();
        set_phase(5);
        let p = unsafe { allocator.realloc(p, middle, 9) };
        assert!(!p.is_null());
        assert!(unsafe { std::slice::from_raw_parts(p, 9) }
            .iter()
            .all(|v| *v == 0x5a));
        let s = allocator.finish_event();
        partition(&s);
        assert_eq!(s.requested_live_at_peak_bytes, 101);
        assert_eq!(s.requested_peak_phase, Some(4));
        assert_eq!(s.requested_live_by_origin_at_peak[1], 0);
        assert_eq!(s.requested_live_by_origin_at_peak[4], 101);
        assert_eq!(s.requested_live_by_origin_at_end[5], 9);
        assert_eq!(s.flow_by_operation_phase[4].requested_freed_bytes, 31);
        assert_eq!(s.flow_by_operation_phase[4].requested_allocated_bytes, 101);
        assert_eq!(s.flow_by_operation_phase[5].requested_freed_bytes, 101);
        assert_eq!(s.flow_by_operation_phase[5].requested_allocated_bytes, 9);
        unsafe { allocator.dealloc(p, Layout::from_size_align(9, 64).unwrap()) };
        assert_eq!(allocator.live_requested(), 0);
    }

    #[test]
    fn prior_epoch_and_global_peak_keep_surviving_origin_not_peak_location() {
        let allocator = OriginAllocator::new(System, phase);
        let a_layout = Layout::from_size_align(7, 1).unwrap();
        let a = unsafe { allocator.alloc(a_layout) }; // before any event
        allocator.begin_event();
        set_phase(1);
        let b_layout = Layout::from_size_align(11, 1).unwrap();
        let b = unsafe { allocator.alloc(b_layout) };
        set_phase(2);
        let c_layout = Layout::from_size_align(13, 1).unwrap();
        let c = unsafe { allocator.alloc(c_layout) };
        unsafe { allocator.dealloc(c, c_layout) };
        let first = allocator.finish_event();
        partition(&first);
        assert_eq!(first.requested_peak_phase, Some(2));
        assert_eq!(first.requested_live_by_origin_at_peak[PRIOR_EVENT], 7);
        assert_eq!(first.requested_live_by_origin_at_peak[1], 11);
        assert_eq!(first.requested_live_by_origin_at_peak[2], 13);
        allocator.begin_event();
        set_phase(3);
        unsafe { allocator.dealloc(a, a_layout) };
        let second = allocator.finish_event();
        partition(&second);
        assert_eq!(second.requested_peak_phase, None);
        assert_eq!(second.requested_live_by_origin_at_peak[PRIOR_EVENT], 18);
        assert_eq!(second.requested_live_by_origin_at_end[PRIOR_EVENT], 11);
        assert_eq!(second.requested_additional_peak_bytes, 0);
        unsafe { allocator.dealloc(b, b_layout) };
        assert_eq!(allocator.live_requested(), 0);
    }

    #[test]
    fn realloc_of_prior_event_allocation_transfers_full_new_request() {
        let allocator = OriginAllocator::new(System, phase);
        let layout = Layout::from_size_align(8, 8).unwrap();
        allocator.begin_event();
        set_phase(1);
        let p = unsafe { allocator.alloc(layout) };
        assert!(!p.is_null());
        allocator.finish_event();
        allocator.begin_event();
        set_phase(6);
        let p = unsafe { allocator.realloc(p, layout, 32) };
        assert!(!p.is_null());
        let s = allocator.finish_event();
        partition(&s);
        assert_eq!(s.event_start_requested_live_bytes, 8);
        assert_eq!(s.requested_live_by_origin_at_peak[PRIOR_EVENT], 0);
        assert_eq!(s.requested_live_by_origin_at_peak[6], 32);
        assert_eq!(s.requested_additional_peak_bytes, 24);
        assert_eq!(s.flow_by_operation_phase[6].requested_freed_bytes, 8);
        unsafe { allocator.dealloc(p, Layout::from_size_align(32, 8).unwrap()) };
    }

    struct FailingSystem {
        fail_alloc: AtomicBool,
        fail_realloc: AtomicBool,
    }
    unsafe impl GlobalAlloc for FailingSystem {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            if self.fail_alloc.load(Ordering::Relaxed) {
                ptr::null_mut()
            } else {
                unsafe { System.alloc(layout) }
            }
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            if self.fail_alloc.load(Ordering::Relaxed) {
                ptr::null_mut()
            } else {
                unsafe { System.alloc_zeroed(layout) }
            }
        }
        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            unsafe { System.dealloc(pointer, layout) };
        }
        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            if self.fail_realloc.load(Ordering::Relaxed) {
                ptr::null_mut()
            } else {
                unsafe { System.realloc(pointer, layout, size) }
            }
        }
    }

    #[test]
    fn failed_alloc_and_realloc_do_not_change_live_bytes_or_original_header() {
        let allocator = OriginAllocator::new(
            FailingSystem {
                fail_alloc: AtomicBool::new(false),
                fail_realloc: AtomicBool::new(true),
            },
            phase,
        );
        allocator.begin_event();
        set_phase(1);
        let layout = Layout::from_size_align(23, 16).unwrap();
        let p = unsafe { allocator.alloc(layout) };
        assert!(!p.is_null());
        unsafe { p.write_bytes(0x72, 23) };
        set_phase(2);
        assert!(unsafe { allocator.realloc(p, layout, 300) }.is_null());
        assert!(unsafe { std::slice::from_raw_parts(p, 23) }
            .iter()
            .all(|v| *v == 0x72));
        allocator
            .allocator
            .fail_alloc
            .store(true, Ordering::Relaxed);
        assert!(unsafe { allocator.alloc(layout) }.is_null());
        assert!(unsafe { allocator.alloc_zeroed(layout) }.is_null());
        let s = allocator.finish_event();
        partition(&s);
        assert_eq!(s.requested_live_at_end_bytes, 23);
        assert_eq!(s.requested_live_by_origin_at_end[1], 23);
        assert_eq!(s.flow_by_operation_phase[2].realloc_failures, 1);
        assert_eq!(s.flow_by_operation_phase[2].allocation_failures, 2);
        assert_eq!(s.flow_by_operation_phase[2].requested_allocated_bytes, 0);
        unsafe { allocator.dealloc(p, layout) };
        assert_eq!(allocator.live_requested(), 0);
    }
}
