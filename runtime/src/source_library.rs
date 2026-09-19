//! Pure Caveat functions shared by native and browser adapters.
//!
//! `from_source` parses a complete source file but validates only its function
//! declarations (including unused definitions), together with the prelude.
//! It does not validate a game's events, world, graph or procedures. Callers
//! must validate those separately with the corresponding program runtime.
//! Arguments may retain dependency metadata; this library checks its bounds,
//! not whether the named evidence was observed. Only the graph runtime can
//! authorize observations or construct authoritative graph qualifications.

use crate::ast::Statement;
use crate::reactive::{Directive, Provenance, Tracked};
use crate::reactive_expr::{self, Expr, ValueType};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

pub use crate::reactive_expr::Value as FunctionValue;

const MAX_LIBRARY_SOURCE_BYTES: usize = 1_048_576;
const MAX_LIBRARY_PARAMETERS: usize = 32;
const MAX_LIBRARY_NODES: usize = 32_768;
const MAX_LIBRARY_STRING_BYTES: usize = 1_048_576;
const NUMBER_LIMIT: f64 = 1_000_000_000_000.0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<String>,
    pub result_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledFunction {
    signature: FunctionSignature,
    body: Expr,
}

/// An immutable, cheaply cloned collection of compiled pure functions.
///
/// In addition to the evaluator's existing function, expression and work
/// limits, a library permits 1 MiB of input, 32 parameters per function,
/// 32,768 retained expanded nodes and 1 MiB of retained literal/name bytes.
/// Numeric arguments and numeric results must be finite and within +/-1e12.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLibrary {
    functions: Arc<BTreeMap<String, CompiledFunction>>,
}

impl SourceLibrary {
    /// Extract and compile pure functions from a syntactically valid program.
    /// Other declarations are not semantically validated by this API.
    pub fn from_source(source: &str) -> Result<Self, String> {
        Self::compile(source, false)
    }

    /// Compile a function-only file, rejecting other declaration kinds.
    pub fn from_module_source(source: &str) -> Result<Self, String> {
        Self::compile(source, true)
    }

    fn compile(source: &str, functions_only: bool) -> Result<Self, String> {
        if source.len() > MAX_LIBRARY_SOURCE_BYTES {
            return Err(format!(
                "source library exceeds source limit {MAX_LIBRARY_SOURCE_BYTES} bytes"
            ));
        }
        let mut functions = crate::reactive::prelude_functions()?;
        for statement in crate::parser::parse(source)?.statements {
            match statement {
                Statement::Reactive(Directive::Function(function)) => {
                    let name = function.name.clone();
                    if functions.insert(name.clone(), function).is_some() {
                        return Err(format!("duplicate source function {name} (standard library names are reserved)"));
                    }
                }
                _ if functions_only => {
                    return Err("source module may contain only pure function definitions".into())
                }
                _ => {}
            }
        }
        for function in functions.values() {
            if function.parameters.len() > MAX_LIBRARY_PARAMETERS {
                return Err(format!(
                    "source function {} exceeds parameter limit {MAX_LIBRARY_PARAMETERS}",
                    function.name
                ));
            }
        }
        reactive_expr::validate_functions(&functions)?;
        let mut compiled = BTreeMap::new();
        let mut nodes = 0;
        let mut bytes = 0;
        for (name, function) in &functions {
            let body = reactive_expr::expand(&function.body, &functions)?;
            let numeric: HashSet<String> = function.parameters.iter().cloned().collect();
            let result_type = body.validate(&numeric, &|_, _| {
                Err("pure library cannot read the graph".into())
            })?;
            let result_type = match result_type {
                ValueType::Number => "number",
                ValueType::Bool => "boolean",
                ValueType::Text => "text",
            }
            .to_owned();
            let (body_nodes, body_bytes) = body.storage_usage();
            nodes += body_nodes;
            bytes += body_bytes
                + 2 * name.len()
                + result_type.len()
                + function.parameters.iter().map(String::len).sum::<usize>();
            if nodes > MAX_LIBRARY_NODES {
                return Err(format!(
                    "source library exceeds compiled node limit {MAX_LIBRARY_NODES}"
                ));
            }
            if bytes > MAX_LIBRARY_STRING_BYTES {
                return Err(format!(
                    "source library exceeds compiled string limit {MAX_LIBRARY_STRING_BYTES} bytes"
                ));
            }
            compiled.insert(
                name.clone(),
                CompiledFunction {
                    signature: FunctionSignature {
                        name: name.clone(),
                        parameters: function.parameters.clone(),
                        result_type,
                    },
                    body,
                },
            );
        }
        Ok(Self {
            functions: Arc::new(compiled),
        })
    }

    /// Sorted signatures include the prelude and all authored functions.
    pub fn signatures(&self) -> Vec<FunctionSignature> {
        self.functions
            .values()
            .map(|function| function.signature.clone())
            .collect()
    }

    /// Call with frozen numeric arguments. Every supplied argument is checked
    /// eagerly and its provenance retained, including unused parameters.
    pub fn call(
        &self,
        name: &str,
        arguments: &[Tracked<f64>],
    ) -> Result<Tracked<FunctionValue>, String> {
        let function = self
            .functions
            .get(name)
            .ok_or_else(|| format!("unknown source function {name}"))?;
        if arguments.len() != function.signature.parameters.len() {
            return Err(format!(
                "source function {name} requires {} arguments, found {}",
                function.signature.parameters.len(),
                arguments.len()
            ));
        }
        let mut provenance = Provenance::default();
        for argument in arguments {
            check_number(argument.value)?;
            provenance.merge(&argument.provenance)?;
        }
        let bindings: BTreeMap<&str, &Tracked<f64>> = function
            .signature
            .parameters
            .iter()
            .map(String::as_str)
            .zip(arguments)
            .collect();
        let mut result = function.body.evaluate_tracked(
            &|name| Ok(bindings.get(name).map(|value| (*value).clone())),
            &|_, _| Err("pure library cannot read the graph".into()),
            &|_, _| Err("pure library cannot construct graph qualifications".into()),
        )?;
        if let FunctionValue::Number(value) = result.value {
            check_number(value)?;
        }
        result.provenance.merge(&provenance)?;
        Ok(result)
    }

    /// Adapter convenience for ordinary, unqualified numeric input facts.
    /// The canonical result still includes its dependency metadata.
    pub fn call_numbers(
        &self,
        name: &str,
        arguments: &[f64],
    ) -> Result<Tracked<FunctionValue>, String> {
        if arguments.len() > MAX_LIBRARY_PARAMETERS {
            return Err(format!(
                "source call exceeds argument limit {MAX_LIBRARY_PARAMETERS}"
            ));
        }
        self.call(
            name,
            &arguments
                .iter()
                .copied()
                .map(Tracked::plain)
                .collect::<Vec<_>>(),
        )
    }
}

fn check_number(value: f64) -> Result<(), String> {
    if !value.is_finite() || value.abs() > NUMBER_LIMIT {
        return Err("source library number must be finite and within +/-1e12".into());
    }
    Ok(())
}
