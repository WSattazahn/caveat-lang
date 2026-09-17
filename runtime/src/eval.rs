use std::collections::HashMap; use crate::ast::{Program,Statement}; use crate::{Attention,EpistemicGraph,NodeId,NodeKind,StopReason};
#[derive(Debug,Clone,PartialEq,Eq)]pub struct ResourceLedger{pub initial:u64,pub remaining:u64,pub spent:u64,pub exhausted:bool}
#[derive(Debug)]pub struct Evaluation{pub graph:EpistemicGraph,pub symbols:HashMap<String,NodeId>,pub resources:Option<ResourceLedger>}
pub fn evaluate(program:&Program)->Result<Evaluation,String>{let mut g=EpistemicGraph::new();let mut sy=HashMap::new();let mut res:Option<ResourceLedger>=None;for st in &program.statements{match st{
Statement::Budget{units}=>{if res.is_some(){return Err("budget already defined".into())}res=Some(ResourceLedger{initial:*units,remaining:*units,spent:0,exhausted:*units==0});}
Statement::Claim{name}=>{let i=g.add(NodeKind::Claim{proposition:name.clone()});define(&mut sy,name,i)?}
Statement::Evidence{name,source}=>{let i=g.add(NodeKind::Evidence{description:name.clone(),source:source.clone()});define(&mut sy,name,i)?}
Statement::Caveat{name,consequence}=>{let i=g.add_caveat(name,*consequence);define(&mut sy,name,i)?}
Statement::Relate{from,relation,to}=>{g.relate(resolve(&sy,from)?,*relation,resolve(&sy,to)?)}
Statement::Attention{caveat,state}=>{g.set_attention(resolve(&sy,caveat)?,*state)}
Statement::Examine{caveat,cost}=>{let i=resolve(&sy,caveat)?;let r=res.as_mut().ok_or("examine cost requires a budget")?;if *cost>r.remaining{r.exhausted=true;return Err(format!("budget exhausted: examination of {caveat} costs {cost}, remaining {}",r.remaining))}r.remaining-=*cost;r.spent+=*cost;r.exhausted=r.remaining==0;g.set_attention(i,Attention::Examined);}
Statement::Commit{action,reason,retaining}=>{if matches!(reason,StopReason::BudgetExhausted)&&!res.as_ref().map(|r|r.exhausted).unwrap_or(false){return Err("budget commitment requires exhausted budget".into())}let ids=retaining.iter().map(|n|resolve(&sy,n)).collect::<Result<Vec<_>,_>>()?;let i=g.commit_because(action,&ids,reason.clone());define(&mut sy,action,i)?}
Statement::Reopen{commitment,because}=>{g.reopen(resolve(&sy,commitment)?,resolve(&sy,because)?)}
}}Ok(Evaluation{graph:g,symbols:sy,resources:res})}
fn define(s:&mut HashMap<String,NodeId>,n:&str,i:NodeId)->Result<(),String>{if s.insert(n.into(),i).is_some(){Err(format!("duplicate symbol: {n}"))}else{Ok(())}}
fn resolve(s:&HashMap<String,NodeId>,n:&str)->Result<NodeId,String>{s.get(n).copied().ok_or_else(||format!("unknown symbol: {n}"))}
