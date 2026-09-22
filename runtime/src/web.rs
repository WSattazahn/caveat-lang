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

    pub fn dispatch(&mut self, event: &str, payload_json: &str) -> Result<String, String> {
        self.inner
            .dispatch_json(event, payload_json)
            .and_then(|snapshot| to_json_pretty(&snapshot))
    }

    /// The per-event view, as compact JSON. See spec/caveat-view-0.1.md.
    pub fn view(&self) -> String {
        serde_json::to_string(&self.inner.view()).expect("finite reactive view")
    }

    /// Dispatch and return the per-event view rather than the full snapshot.
    pub fn dispatch_view(&mut self, event: &str, payload_json: &str) -> Result<String, String> {
        self.inner
            .dispatch_view_json(event, payload_json)
            .map(|view| serde_json::to_string(&view).expect("finite reactive view"))
    }
}
