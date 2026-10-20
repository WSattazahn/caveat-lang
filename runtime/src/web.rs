use crate::map::to_json_pretty;
use crate::reactive::{ReactiveSession, ViewDeltaBase};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// The WebAssembly build's allocator: the system allocator, recording
/// whether an allocation has ever failed. A failed allocation aborts the
/// instance with the trap `unreachable`, which says nothing about memory; the
/// record lets a host name the cause afterwards (`out_of_memory`). A host
/// that never frees its sessions exhausts memory this way (Glowcap round 8,
/// item 7).
#[cfg(target_arch = "wasm32")]
mod allocation {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicBool, Ordering};

    pub static FAILED: AtomicBool = AtomicBool::new(false);

    pub struct Recording;

    fn record(pointer: *mut u8) -> *mut u8 {
        if pointer.is_null() {
            FAILED.store(true, Ordering::Relaxed);
        }
        pointer
    }

    // SAFETY: each method forwards to `System` with the caller's arguments
    // unchanged, and only reads the pointer it returns.
    unsafe impl GlobalAlloc for Recording {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            record(unsafe { System.alloc(layout) })
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            record(unsafe { System.alloc_zeroed(layout) })
        }

        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            record(unsafe { System.realloc(pointer, layout, size) })
        }

        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            unsafe { System.dealloc(pointer, layout) }
        }
    }

    #[global_allocator]
    static ALLOCATOR: Recording = Recording;
}

// The sequential, graphics and 3D sessions. A host that only runs reactive
// programs can build without them: cargo build --no-default-features.
#[cfg(feature = "games")]
pub use crate::web_games::*;

/// A generic event bridge: arithmetic, collisions, and epistemic effects all
/// execute in CAVEAT source. The host supplies only bounded event parameters.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WebReactiveSession {
    inner: ReactiveSession,
    /// The view as `dispatch_view_delta_outcome` last left it; every other
    /// method that changes the session clears it.
    delta_base: ViewDeltaBase,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WebReactiveSession {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(source: &str) -> Result<WebReactiveSession, String> {
        Ok(Self {
            inner: ReactiveSession::from_source_cached(source)?,
            delta_base: ViewDeltaBase::default(),
        })
    }

    pub fn snapshot(&self) -> String {
        to_json_pretty(&self.inner.snapshot()).expect("finite reactive snapshot")
    }

    /// The same snapshot as compact JSON, for a host that parses it rather
    /// than shows the text.
    pub fn snapshot_compact(&self) -> String {
        serde_json::to_string(&self.inner.snapshot()).expect("finite reactive snapshot")
    }

    /// Whether this runtime instance has failed to allocate memory. After a
    /// trap, true means the instance ran out of WebAssembly memory, most
    /// often because sessions were never freed. Reading it is safe after a
    /// trap; nothing else is. Always false outside WebAssembly.
    pub fn out_of_memory() -> bool {
        #[cfg(target_arch = "wasm32")]
        return allocation::FAILED.load(std::sync::atomic::Ordering::Relaxed);
        #[cfg(not(target_arch = "wasm32"))]
        false
    }

    /// Advisory warnings for a program that loads, as JSON
    /// (spec/caveat-check-0.1.md). Errors as `new` does when it does not.
    pub fn check(source: &str) -> Result<String, String> {
        let report = crate::reactive::check_source(source)?;
        to_json_pretty(&report).map_err(|error| error.to_string())
    }

    /// `check`, with the checks asked for as well: `unused` adds C007
    /// `unused-declaration`.
    pub fn check_with(source: &str, unused: bool) -> Result<String, String> {
        let options = crate::reactive::CheckOptions {
            unused,
            ..Default::default()
        };
        let report = crate::reactive::check_source_with(source, options)?;
        to_json_pretty(&report).map_err(|error| error.to_string())
    }

    /// `check`, with the checks `options` asks for, as JSON:
    /// `{"unused": true}` adds C007, `{"records": true}` adds C008, and
    /// `"save": TEXT` with `records` adds C009 for that save. An unknown
    /// option is an error, so a host learns this build does not have it.
    pub fn check_options(source: &str, options: &str) -> Result<String, String> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Options {
            #[serde(default)]
            unused: bool,
            #[serde(default)]
            records: bool,
            save: Option<String>,
        }
        let Options {
            unused,
            records,
            save,
        } = serde_json::from_str(options).map_err(|error| format!("check options: {error}"))?;
        let options = crate::reactive::CheckOptions { unused, records };
        let report = match save {
            Some(save) => crate::reactive::check_save_with(source, &save, options)?,
            None => crate::reactive::check_source_with(source, options)?,
        };
        to_json_pretty(&report).map_err(|error| error.to_string())
    }

    /// The program's events, states, bindings and declared names, as JSON
    /// (spec/caveat-interface-0.1.md). Errors as `new` does when it does not load.
    pub fn interface(source: &str) -> Result<String, String> {
        let interface = crate::reactive::interface_source(source)?;
        to_json_pretty(&interface).map_err(|error| error.to_string())
    }

    pub fn dispatch(&mut self, event: &str, payload_json: &str) -> Result<String, String> {
        self.delta_base.clear();
        self.inner
            .dispatch_json(event, payload_json)
            .and_then(|snapshot| to_json_pretty(&snapshot))
    }

    /// Structured refusals are returned values; every thrown error is fatal.
    /// Uses the same interpreter/transaction as dispatch. See Dispatch 0.1.
    pub fn dispatch_outcome(&mut self, event: &str, payload_json: &str) -> Result<String, String> {
        self.delta_base.clear();
        match self.inner.dispatch_outcome_json(event, payload_json) {
            Ok(outcome) => serde_json::to_string(&outcome).map_err(|error| error.to_string()),
            Err(fatal) => {
                Err(serde_json::to_string(&fatal).expect("serializable fatal dispatch error"))
            }
        }
    }

    /// `dispatch_outcome` with the view after an accepted event, as `view`
    /// returns it, in place of the snapshot, which is not built. A refusal is
    /// the same returned value and a fatal error the same thrown report. See
    /// Dispatch 0.1.
    pub fn dispatch_view_outcome(
        &mut self,
        event: &str,
        payload_json: &str,
    ) -> Result<String, String> {
        self.delta_base.clear();
        match self.inner.dispatch_view_outcome_json(event, payload_json) {
            Ok(outcome) => serde_json::to_string(&outcome).map_err(|error| error.to_string()),
            Err(fatal) => {
                Err(serde_json::to_string(&fatal).expect("serializable fatal dispatch error"))
            }
        }
    }

    /// `dispatch_view_outcome` with only the view fields `fields_json` names,
    /// a JSON array of field names. An unknown or repeated name throws before
    /// the event is applied. See spec/caveat-view-0.1.md.
    pub fn dispatch_view_fields_outcome(
        &mut self,
        event: &str,
        payload_json: &str,
        fields_json: &str,
    ) -> Result<String, String> {
        let fields = crate::reactive::ViewFields::from_json(fields_json)?;
        self.delta_base.clear();
        match self
            .inner
            .dispatch_view_fields_outcome_json(event, payload_json, fields)
        {
            Ok(outcome) => serde_json::to_string(&outcome).map_err(|error| error.to_string()),
            Err(fatal) => {
                Err(serde_json::to_string(&fatal).expect("serializable fatal dispatch error"))
            }
        }
    }

    /// The per-event view, as compact JSON. See spec/caveat-view-0.1.md.
    pub fn view(&self) -> String {
        serde_json::to_string(&self.inner.view()).expect("finite reactive view")
    }

    /// The per-event view in schema 0.2, as compact JSON. See
    /// spec/caveat-view-0.2.md.
    pub fn view_v2(&self) -> String {
        serde_json::to_string(&self.inner.view_v2()).expect("finite reactive view")
    }

    /// `dispatch_outcome` with the View 0.2 delta of an accepted event in
    /// place of the snapshot. A refusal is the same returned value and a fatal
    /// error the same thrown report. See spec/caveat-view-0.2.md.
    pub fn dispatch_view_delta_outcome(
        &mut self,
        event: &str,
        payload_json: &str,
    ) -> Result<String, String> {
        match self.inner.dispatch_view_delta_outcome_json_from(
            &mut self.delta_base,
            event,
            payload_json,
        ) {
            Ok(outcome) => serde_json::to_string(&outcome).map_err(|error| error.to_string()),
            Err(fatal) => {
                Err(serde_json::to_string(&fatal).expect("serializable fatal dispatch error"))
            }
        }
    }

    /// Everything the session knows that an event can change, as JSON a host
    /// can store. See spec/caveat-save-0.1.md.
    pub fn save(&self) -> Result<String, String> {
        self.inner.save_json()
    }

    /// The replay save while the session's accepted events fit in one and it
    /// is smaller, otherwise the state save `save` returns. `restore` takes
    /// either. See spec/caveat-replay-save-0.1.md.
    pub fn save_compact(&self) -> Result<String, String> {
        self.inner.save_compact_json()
    }

    /// `save_compact` with the positional replay save, which holds more
    /// events. See spec/caveat-replay-save-0.2.md.
    pub fn save_compact_positional(&self) -> Result<String, String> {
        self.inner.save_compact_positional_json()
    }

    /// The records departed since the last drain, as a JSON array, and an
    /// empty archive. See spec/caveat-departure-0.1.md, "The archive".
    pub fn drain_archive(&mut self) -> String {
        self.delta_base.clear();
        serde_json::to_string(&self.inner.drain_archive()).expect("finite archive entries")
    }

    /// How many departed records wait in the archive.
    pub fn undrained(&self) -> usize {
        self.inner.undrained()
    }

    /// A session of `source` resumed from a state save, without replaying
    /// events, or from a replay save, by dispatching its events again.
    pub fn restore(source: &str, saved: &str) -> Result<WebReactiveSession, String> {
        Ok(Self {
            inner: ReactiveSession::restore_json(source, saved)?,
            delta_base: ViewDeltaBase::default(),
        })
    }

    /// Dispatch and return the per-event view rather than the full snapshot.
    pub fn dispatch_view(&mut self, event: &str, payload_json: &str) -> Result<String, String> {
        self.delta_base.clear();
        self.inner
            .dispatch_view_json(event, payload_json)
            .map(|view| serde_json::to_string(&view).expect("finite reactive view"))
    }
}
