#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;
use crate::ast::{Program,Statement};

fn esc(s:&str)->String{s.replace('\\',"\\\\").replace('"',"\\\"").replace('\n',"\\n")}
fn arr(xs:&[String])->String{format!("[{}]",xs.iter().map(|x|format!("\"{}\"",esc(x))).collect::<Vec<_>>().join(","))}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn evaluate_summary(source:&str)->String{match crate::parser::parse(source).and_then(|p|crate::eval::evaluate(&p)){Ok(e)=>{let b=e.resources.as_ref().map(|x|format!("{}/{}",x.remaining,x.initial)).unwrap_or_else(||"none".into());format!("CAVEAT|nodes={}|edges={}|events={}|budget={}",e.graph.nodes.len(),e.graph.edges.len(),e.history.len(),b)},Err(e)=>format!("CAVEAT_ERROR|{e}")}}

/// Returns the first unresolved interactive operation as JSON. Browser clients
/// can render this without understanding the internal graph representation.
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn pending_interaction(source:&str)->String{
 let p=match crate::parser::parse(source){Ok(p)=>p,Err(e)=>return format!("{{\"kind\":\"error\",\"message\":\"{}\"}}",esc(&e))};
 for(i,s)in p.statements.iter().enumerate(){match s{
  Statement::Inspect{investigation,caveat,cost}=>{let pre=match crate::eval::evaluate(&Program::new(p.statements[..i].to_vec())){Ok(x)=>x,Err(e)=>return format!("{{\"kind\":\"error\",\"message\":\"{}\"}}",esc(&e))};if let Some(inv)=pre.investigations.get(investigation){let budget=pre.resources.as_ref().map(|b|b.remaining).unwrap_or(0);return format!("{{\"kind\":\"investigate\",\"name\":\"{}\",\"options\":{},\"default\":\"{}\",\"cost\":{},\"budget\":{}}}",esc(investigation),arr(&inv.options),esc(caveat),cost,budget)}}
  Statement::Select{choice,option}=>{let pre=match crate::eval::evaluate(&Program::new(p.statements[..i].to_vec())){Ok(x)=>x,Err(e)=>return format!("{{\"kind\":\"error\",\"message\":\"{}\"}}",esc(&e))};if let Some(cp)=pre.choices.get(choice){let budget=pre.resources.as_ref().map(|b|b.remaining).unwrap_or(0);return format!("{{\"kind\":\"choice\",\"name\":\"{}\",\"options\":{},\"default\":\"{}\",\"budget\":{}}}",esc(choice),arr(&cp.options),esc(option),budget)}}
  _=>{}
 }}"{\"kind\":\"complete\"}".into()
}

/// Rewrites the first pending operation with the player's selection. Returning
/// source keeps the browser stateless while CAVEAT remains authoritative.
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
pub fn apply_first_action(source:&str,selection:&str)->String{let mut p=match crate::parser::parse(source){Ok(p)=>p,Err(e)=>return format!("CAVEAT_ERROR|{e}")};for s in&mut p.statements{match s{Statement::Inspect{caveat,..}=>{*caveat=selection.into();break},Statement::Select{option,..}=>{*option=selection.into();break},_=>{}}}serialize(&p)}

fn serialize(p:&Program)->String{format!("{:?}",p)}
