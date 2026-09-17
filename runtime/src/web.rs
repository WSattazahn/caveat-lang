#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

/// Minimal browser embedding boundary. The browser supplies CAVEAT source;
/// the same parser/evaluator used by the native runner executes it.
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn evaluate_summary(source:&str)->String {
    match crate::parser::parse(source).and_then(|p| crate::eval::evaluate(&p)) {
        Ok(e) => {
            let budget=e.resources.as_ref().map(|b|format!("{}/{}",b.remaining,b.initial)).unwrap_or_else(||"none".into());
            let decisions=e.graph.nodes.values().filter(|n|matches!(n,crate::NodeKind::Commitment{..})).count();
            format!("CAVEAT|nodes={}|edges={}|events={}|budget={}|decisions={}",e.graph.nodes.len(),e.graph.edges.len(),e.history.len(),budget,decisions)
        }
        Err(err)=>format!("CAVEAT_ERROR|{err}")
    }
}
