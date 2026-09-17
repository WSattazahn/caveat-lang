use std::{env,fs,process};
use caveat_runtime::{eval,parser,NodeKind};

fn main(){
 let path=env::args().nth(1).unwrap_or_else(||{eprintln!("usage: caveat <file.cav>");process::exit(2)});
 let source=fs::read_to_string(&path).unwrap_or_else(|e|{eprintln!("caveat: {e}");process::exit(1)});
 let program=parser::parse(&source).unwrap_or_else(|e|{eprintln!("parse error: {e}");process::exit(1)});
 let result=eval::evaluate(&program).unwrap_or_else(|e|{eprintln!("evaluation error: {e}");process::exit(1)});
 println!("CAVEAT: {} statements -> {} graph nodes, {} edges",program.statements.len(),result.graph.nodes.len(),result.graph.edges.len());
 for (name,id) in &result.symbols { if let Some(NodeKind::Commitment{open,stop_reason,..})=result.graph.nodes.get(id){println!("commit {name}: open={open} stop={stop_reason:?}");} }
}
