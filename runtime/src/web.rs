use crate::map::to_json_pretty;
use crate::reactive::ReactiveSession;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// The sequential, graphics and 3D sessions. A host that only runs reactive
// programs can build without them: cargo build --no-default-features.
#[cfg(feature = "games")]
pub use crate::web_games::*;

/// A generic event bridge: arithmetic, collisions, and epistemic effects all
/// execute in CAVEAT source. The host supplies only bounded event parameters.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WebReactiveSession {
    inner: ReactiveSession,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WebReactiveSession {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(source: &str) -> Result<WebReactiveSession, String> {
        Ok(Self {
            inner: ReactiveSession::from_source(source)?,
        })
    }

    pub fn snapshot(&self) -> String {
        to_json_pretty(&self.inner.snapshot()).expect("finite reactive snapshot")
    }

    /// Advisory warnings for a program that loads, as JSON
    /// (spec/caveat-check-0.1.md). Errors as `new` does when it does not.
    pub fn check(source: &str) -> Result<String, String> {
        let report = crate::reactive::check_source(source)?;
        to_json_pretty(&report).map_err(|error| error.to_string())
    }

    /// The program's events, states, bindings and declared names, as JSON
    /// (spec/caveat-interface-0.1.md). Errors as `new` does when it does not load.
    pub fn interface(source: &str) -> Result<String, String> {
        let interface = crate::reactive::interface_source(source)?;
        to_json_pretty(&interface).map_err(|error| error.to_string())
    }

    pub fn dispatch(&mut self, event: &str, payload_json: &str) -> Result<String, String> {
        self.inner
            .dispatch_json(event, payload_json)
            .and_then(|snapshot| to_json_pretty(&snapshot))
    }

    /// Structured refusals are returned values; every thrown error is fatal.
    /// Uses the same interpreter/transaction as dispatch. See Dispatch 0.1.
    pub fn dispatch_outcome(&mut self, event: &str, payload_json: &str) -> Result<String, String> {
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
        match self.inner.dispatch_view_outcome_json(event, payload_json) {
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
        match self
            .inner
            .dispatch_view_delta_outcome_json(event, payload_json)
        {
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

    /// The records departed since the last drain, as a JSON array, and an
    /// empty archive. See spec/caveat-departure-0.1.md, "The archive".
    pub fn drain_archive(&mut self) -> String {
        serde_json::to_string(&self.inner.drain_archive()).expect("finite archive entries")
    }

    /// How many departed records wait in the archive.
    pub fn undrained(&self) -> usize {
        self.inner.undrained()
    }

    /// A session of `source` resumed from a save, without replaying events.
    pub fn restore(source: &str, saved: &str) -> Result<WebReactiveSession, String> {
        Ok(Self {
            inner: ReactiveSession::restore_json(source, saved)?,
        })
    }

    /// Dispatch and return the per-event view rather than the full snapshot.
    pub fn dispatch_view(&mut self, event: &str, payload_json: &str) -> Result<String, String> {
        self.inner
            .dispatch_view_json(event, payload_json)
            .map(|view| serde_json::to_string(&view).expect("finite reactive view"))
    }
}
