//! WebAssembly adapter for the same pure source functions used by native code.

use crate::source_library::SourceLibrary;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Enough room for 32 JSON numeric arguments without permitting unbounded
// whitespace/allocation. The native library enforces arity and value bounds.
const MAX_ARGUMENT_JSON_BYTES: usize = 4096;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WebSourceLibrary {
    inner: SourceLibrary,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WebSourceLibrary {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(source: &str) -> Result<Self, String> {
        Ok(Self {
            inner: SourceLibrary::from_source(source)?,
        })
    }

    /// Numeric JSON array input; the output retains `{value, provenance}`.
    pub fn call(&self, name: &str, arguments_json: &str) -> Result<String, String> {
        if arguments_json.len() > MAX_ARGUMENT_JSON_BYTES {
            return Err(format!(
                "source call JSON exceeds limit {MAX_ARGUMENT_JSON_BYTES} bytes"
            ));
        }
        let arguments: Vec<f64> = serde_json::from_str(arguments_json)
            .map_err(|error| format!("invalid source call arguments: {error}"))?;
        let result = self.inner.call_numbers(name, &arguments)?;
        serde_json::to_string(&result).map_err(|error| format!("source result JSON: {error}"))
    }

    pub fn signatures(&self) -> Result<String, String> {
        serde_json::to_string(&self.inner.signatures())
            .map_err(|error| format!("source signatures JSON: {error}"))
    }
}
