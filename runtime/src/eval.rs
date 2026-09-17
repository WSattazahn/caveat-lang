use std::collections::HashMap;
use crate::ast::{Program, Statement};
use crate::{EpistemicGraph, NodeId, NodeKind};

#[derive(Debug)]
pub struct Evaluation { pub graph: EpistemicGraph, pub symbols: HashMap<String, NodeId> }

pub fn evaluate(program: &Program) -> Result<Evaluation, String> {
    let mut graph = EpistemicGraph::new();
    let mut symbols = HashMap::new();
    for statement in &program.statements {
        match statement {
            Statement::Claim { name } => { let id=graph.add(NodeKind::Claim{proposition:name.clone()}); define(&mut symbols,name,id)?; }
            Statement::Evidence { name, source } => { let id=graph.add(NodeKind::Evidence{description:name.clone(),source:source.clone()}); define(&mut symbols,name,id)?; }
            Statement::Caveat { name, consequence } => { let id=graph.add_caveat(name,*consequence); define(&mut symbols,name,id)?; }
            Statement::Relate { from, relation, to } => { let a=resolve(&symbols,from)?; let b=resolve(&symbols,to)?; graph.relate(a,*relation,b); }
            Statement::Attention { caveat, state } => { let id=resolve(&symbols,caveat)?; graph.set_attention(id,*state); }
            Statement::Commit { action, reason, retaining } => { let ids=retaining.iter().map(|n|resolve(&symbols,n)).collect::<Result<Vec<_>,_>>()?; let id=graph.commit_because(action,&ids,reason.clone()); define(&mut symbols,action,id)?; }
            Statement::Reopen { commitment, because } => { let c=resolve(&symbols,commitment)?; let b=resolve(&symbols,because)?; graph.reopen(c,b); }
        }
    }
    Ok(Evaluation { graph, symbols })
}
fn define(symbols:&mut HashMap<String,NodeId>,name:&str,id:NodeId)->Result<(),String>{if symbols.insert(name.into(),id).is_some(){Err(format!("duplicate symbol: {name}"))}else{Ok(())}}
fn resolve(symbols:&HashMap<String,NodeId>,name:&str)->Result<NodeId,String>{symbols.get(name).copied().ok_or_else(||format!("unknown symbol: {name}"))}
