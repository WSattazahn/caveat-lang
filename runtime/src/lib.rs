pub mod ast; pub mod parser; pub mod eval;
use std::collections::HashMap; pub type NodeId=u64;
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord)]pub enum Consequence{Negligible,Low,Material,High,Catastrophic}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum Attention{Unexamined,Deferred,Examining,Examined}
#[derive(Debug,Clone,PartialEq,Eq)]pub enum StopReason{Enough,BudgetExhausted,Deadline,ExternalDecision(String)}
#[derive(Debug,Clone,PartialEq,Eq)]pub enum QualificationTopology{Finite{max_depth:usize},Cyclic}
#[derive(Debug,Clone,PartialEq,Eq)]pub enum NodeKind{Claim{proposition:String},Evidence{description:String,source:String},Caveat{description:String,consequence:Consequence,attention:Attention},Context{name:String},Commitment{action:String,open:bool,stop_reason:StopReason},Observation{description:String}}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum Relation{Supports,Opposes,Qualifies,InContext,Retains,Reopens}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct Edge{pub from:NodeId,pub to:NodeId,pub relation:Relation}
#[derive(Debug,Default)]pub struct EpistemicGraph{next_id:NodeId,pub nodes:HashMap<NodeId,NodeKind>,pub edges:Vec<Edge>}
impl EpistemicGraph{
pub fn new()->Self{Self{next_id:1,..Self::default()}}
pub fn add(&mut self,n:NodeKind)->NodeId{let i=self.next_id;self.next_id+=1;self.nodes.insert(i,n);i}
pub fn add_caveat(&mut self,d:impl Into<String>,c:Consequence)->NodeId{self.add(NodeKind::Caveat{description:d.into(),consequence:c,attention:Attention::Unexamined})}
pub fn relate(&mut self,f:NodeId,r:Relation,t:NodeId){assert!(self.nodes.contains_key(&f));assert!(self.nodes.contains_key(&t));self.edges.push(Edge{from:f,to:t,relation:r})}
pub fn set_attention(&mut self,c:NodeId,a:Attention){match self.nodes.get_mut(&c){Some(NodeKind::Caveat{attention,..})=>*attention=a,_=>panic!("attention target must be a caveat")}}
pub fn consequence(&self,c:NodeId)->Consequence{match self.nodes.get(&c){Some(NodeKind::Caveat{consequence,..})=>*consequence,_=>panic!("consequence target must be a caveat")}}
pub fn commit(&mut self,a:impl Into<String>,r:&[NodeId])->NodeId{self.commit_because(a,r,StopReason::Enough)}
pub fn commit_because(&mut self,a:impl Into<String>,r:&[NodeId],s:StopReason)->NodeId{let i=self.add(NodeKind::Commitment{action:a.into(),open:false,stop_reason:s});for &n in r{self.relate(i,Relation::Retains,n)}i}
pub fn reopen(&mut self,c:NodeId,b:NodeId){match self.nodes.get_mut(&c){Some(NodeKind::Commitment{open,..})=>*open=true,_=>panic!("reopen target must be a commitment")};self.relate(b,Relation::Reopens,c)}
pub fn relations(&self,r:Relation)->impl Iterator<Item=&Edge>{self.edges.iter().filter(move|e|e.relation==r)}
pub fn qualification_topology_from(&self,root:NodeId)->QualificationTopology{fn walk(g:&EpistemicGraph,n:NodeId,path:&mut Vec<NodeId>)->Result<usize,()>{if path.contains(&n){return Err(())}path.push(n);let mut m=0;for e in g.edges.iter().filter(|e|e.relation==Relation::Qualifies&&e.to==n){m=m.max(1+walk(g,e.from,path)?)}path.pop();Ok(m)}match walk(self,root,&mut vec![]){Ok(max_depth)=>QualificationTopology::Finite{max_depth},Err(())=>QualificationTopology::Cyclic}}
pub fn qualification_depth_from(&self,root:NodeId)->usize{match self.qualification_topology_from(root){QualificationTopology::Finite{max_depth}=>max_depth,QualificationTopology::Cyclic=>panic!("cyclic qualification has no finite depth")}}
}
#[cfg(test)]mod tests{use super::*;
#[test]fn finite_topology(){let p=parser::parse("claim r; caveat a consequence low; caveat b consequence low; a qualifies r; b qualifies a;").unwrap();let x=eval::evaluate(&p).unwrap();assert_eq!(x.graph.qualification_topology_from(x.symbols["r"]),QualificationTopology::Finite{max_depth:2});}
#[test]fn cycle_is_not_reported_as_depth(){let p=parser::parse("caveat a consequence material; caveat b consequence material; a qualifies b; b qualifies a;").unwrap();let x=eval::evaluate(&p).unwrap();assert_eq!(x.graph.qualification_topology_from(x.symbols["a"]),QualificationTopology::Cyclic);}
#[test]fn contradictory_evidence_coexists(){let p=parser::parse("claim door; evidence camera from cam; evidence latch from sensor; camera supports door; latch opposes door;").unwrap();let x=eval::evaluate(&p).unwrap();let d=x.symbols["door"];assert_eq!(x.graph.edges.iter().filter(|e|e.to==d&&matches!(e.relation,Relation::Supports|Relation::Opposes)).count(),2);}
#[test]fn duplicate_symbol_fails(){let p=parser::parse("claim x; claim x;").unwrap();assert!(eval::evaluate(&p).unwrap_err().contains("duplicate symbol"));}
#[test]fn unknown_symbol_fails(){let p=parser::parse("claim x; ghost supports x;").unwrap();assert!(eval::evaluate(&p).unwrap_err().contains("unknown symbol"));}
#[test]fn repeated_reopen_is_visible_history(){let p=parser::parse("claim x; caveat c consequence material; commit go because enough retaining c; reopen go because c; reopen go because c;").unwrap();let x=eval::evaluate(&p).unwrap();let go=x.symbols["go"];assert_eq!(x.graph.relations(Relation::Reopens).filter(|e|e.to==go).count(),2);}
}
