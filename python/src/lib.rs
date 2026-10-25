//! The native half of the `caveat_lang` Python package: the runtime's
//! `WebReactiveSession`, the type the npm kit reaches through WebAssembly,
//! exposed method for method. Every method takes and returns the runtime's
//! own text, so the Python layer (caveat_lang/__init__.py) reads exactly the
//! bytes the JavaScript kit reads.
//!
//! A runtime error (`Err(String)`) raises `RuntimeFailure` with its text. A
//! panic is the native counterpart of a WebAssembly trap: it raises `Trap`,
//! and the session that panicked refuses every later call.

use std::panic::{catch_unwind, AssertUnwindSafe};

use caveat_runtime::web::WebReactiveSession;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(_runtime, RuntimeFailure, PyException);
create_exception!(_runtime, Trap, PyException);

fn panic_text(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "the runtime panicked".to_string()
    }
}

/// Runs `call`, turning a runtime error into `RuntimeFailure` and a panic into
/// `Trap`.
fn guarded<T>(call: impl FnOnce() -> Result<T, String>) -> PyResult<T> {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(message)) => Err(RuntimeFailure::new_err(message)),
        Err(payload) => Err(Trap::new_err(panic_text(payload))),
    }
}

#[pyclass(module = "caveat_lang._runtime", unsendable)]
struct NativeSession {
    inner: Option<WebReactiveSession>,
}

impl NativeSession {
    /// Runs `call` on the session. A panic leaves the session unusable.
    fn with<T>(
        &mut self,
        call: impl FnOnce(&mut WebReactiveSession) -> Result<T, String>,
    ) -> PyResult<T> {
        let inner = self
            .inner
            .as_mut()
            .ok_or_else(|| Trap::new_err("this session panicked earlier and cannot be used"))?;
        match catch_unwind(AssertUnwindSafe(|| call(inner))) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(message)) => Err(RuntimeFailure::new_err(message)),
            Err(payload) => {
                self.inner = None;
                Err(Trap::new_err(panic_text(payload)))
            }
        }
    }
}

#[pymethods]
impl NativeSession {
    #[new]
    fn new(source: &str) -> PyResult<Self> {
        guarded(|| WebReactiveSession::new(source)).map(|inner| Self { inner: Some(inner) })
    }

    #[staticmethod]
    fn restore(source: &str, saved: &str) -> PyResult<Self> {
        guarded(|| WebReactiveSession::restore(source, saved)).map(|inner| Self { inner: Some(inner) })
    }

    #[staticmethod]
    fn check(source: &str) -> PyResult<String> {
        guarded(|| WebReactiveSession::check(source))
    }

    #[staticmethod]
    fn check_with(source: &str, unused: bool) -> PyResult<String> {
        guarded(|| WebReactiveSession::check_with(source, unused))
    }

    #[staticmethod]
    fn check_options(source: &str, options: &str) -> PyResult<String> {
        guarded(|| WebReactiveSession::check_options(source, options))
    }

    #[staticmethod]
    fn interface(source: &str) -> PyResult<String> {
        guarded(|| WebReactiveSession::interface(source))
    }

    fn dispatch_outcome(&mut self, event: &str, payload_json: &str) -> PyResult<String> {
        self.with(|inner| inner.dispatch_outcome(event, payload_json))
    }

    fn dispatch_view_outcome(&mut self, event: &str, payload_json: &str) -> PyResult<String> {
        self.with(|inner| inner.dispatch_view_outcome(event, payload_json))
    }

    fn dispatch_view_fields_outcome(
        &mut self,
        event: &str,
        payload_json: &str,
        fields_json: &str,
    ) -> PyResult<String> {
        self.with(|inner| inner.dispatch_view_fields_outcome(event, payload_json, fields_json))
    }

    fn dispatch_view_delta_outcome(&mut self, event: &str, payload_json: &str) -> PyResult<String> {
        self.with(|inner| inner.dispatch_view_delta_outcome(event, payload_json))
    }

    fn dispatch_outcome_as(
        &mut self,
        event: &str,
        payload_json: &str,
        agent: &str,
    ) -> PyResult<String> {
        self.with(|inner| inner.dispatch_outcome_as(event, payload_json, agent))
    }

    fn dispatch_view_outcome_as(
        &mut self,
        event: &str,
        payload_json: &str,
        agent: &str,
    ) -> PyResult<String> {
        self.with(|inner| inner.dispatch_view_outcome_as(event, payload_json, agent))
    }

    fn dispatch_view_fields_outcome_as(
        &mut self,
        event: &str,
        payload_json: &str,
        fields_json: &str,
        agent: &str,
    ) -> PyResult<String> {
        self.with(|inner| {
            inner.dispatch_view_fields_outcome_as(event, payload_json, fields_json, agent)
        })
    }

    fn dispatch_view_delta_outcome_as(
        &mut self,
        event: &str,
        payload_json: &str,
        agent: &str,
    ) -> PyResult<String> {
        self.with(|inner| inner.dispatch_view_delta_outcome_as(event, payload_json, agent))
    }

    fn snapshot(&mut self) -> PyResult<String> {
        self.with(|inner| Ok(inner.snapshot()))
    }

    fn snapshot_compact(&mut self) -> PyResult<String> {
        self.with(|inner| Ok(inner.snapshot_compact()))
    }

    fn view(&mut self) -> PyResult<String> {
        self.with(|inner| Ok(inner.view()))
    }

    fn view_v2(&mut self) -> PyResult<String> {
        self.with(|inner| Ok(inner.view_v2()))
    }

    fn save(&mut self) -> PyResult<String> {
        self.with(|inner| inner.save())
    }

    fn save_compact(&mut self) -> PyResult<String> {
        self.with(|inner| inner.save_compact())
    }

    fn save_compact_positional(&mut self) -> PyResult<String> {
        self.with(|inner| inner.save_compact_positional())
    }

    fn drain_archive(&mut self) -> PyResult<String> {
        self.with(|inner| Ok(inner.drain_archive()))
    }

    fn undrained(&mut self) -> PyResult<usize> {
        self.with(|inner| Ok(inner.undrained()))
    }
}

// The sessions hold no locks of their own, so the module needs the GIL; on a
// free-threaded Python it is re-enabled while the module is loaded.
#[pymodule(gil_used = true)]
fn _runtime(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();
    module.add_class::<NativeSession>()?;
    module.add("RuntimeFailure", py.get_type::<RuntimeFailure>())?;
    module.add("Trap", py.get_type::<Trap>())?;
    Ok(())
}
