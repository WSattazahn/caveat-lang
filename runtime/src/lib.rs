pub mod ast;
pub mod parser;
pub mod eval;

use std::collections::HashMap;
pub type NodeId=u64;
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord)] pub enum Consequence{Negligible,Low,Material,High,Catastrophic}
#[derive(Debug,Clone,Copy,PartialEq,Eq)] pub enum Attention{Unexamined,Deferred,Examining,Examined}
#[derive(Debug,Clone,PartialEq,Eq)] pub enum StopReason{Enough,BudgetExhausted,Deadline,ExternalDecision(String)}
#[derive(Debug,Clone,PartialEq,Eq)] pub enum NodeKind{Claim{proposition:String},Evidence{description:String,source:String},Caveat{description:String,consequence:Consequence,attention:Attention},Context{name:String},Commitment{action:String,open:bool,stop_reason:StopReason},Observation{description:String}}
#[derive(Debug,Clone,Copy,PartialEq,Eq)] pub enum Relation{Supports,Opposes,Qualifies,InContext,Retains,Reopens}
#[derive(Debug,Clone,PartialEq,Eq)] pub struct Edge{pub from:NodeId,pub to:NodeId,pub relation:Relation}
#[derive(Debug,Default)] pub struct EpistemicGraph{next_id:NodeId,pub nodes:HashMap<NodeId,NodeKind>,pub edges:Vec<Edge>}
impl EpistemicGraph{
pub fn new()->Self{Self{next_id:1,..Self::default()}}
pub fn add(&mut self,node:NodeKind)->NodeId{let id=self.next_id;self.next_id+=1;self.nodes.insert(id,node);id}
pub fn add_caveat(&mut self,d:impl Into<String>,c:Consequence)->NodeId{self.add(NodeKind::Caveat{description:d.into(),consequence:c,attention:Attention::Unexamined})}
pub fn relate(&mut self,f:NodeId,r:Relation,t:NodeId){assert!(self.nodes.contains_key(&f));assert!(self.nodes.contains_key(&t));self.edges.push(Edge{from:f,to:t,relation:r});}
pub fn set_attention(&mut self,c:NodeId,a:Attention){match self.nodes.get_mut(&c){Some(NodeKind::Caveat{attention,..})=>*attention=a,_=>panic!("attention target must be a caveat")}}
pub fn consequence(&self,c:NodeId)->Consequence{match self.nodes.get(&c){Some(NodeKind::Caveat{consequence,..})=>*consequence,_=>panic!("consequence target must be a caveat")}}
pub fn commit(&mut self,a:impl Into<String>,r:&[NodeId])->NodeId{self.commit_because(a,r,StopReason::Enough)}
pub fn commit_because(&mut self,a:impl Into<String>,r:&[NodeId],s:StopReason)->NodeId{let id=self.add(NodeKind::Commitment{action:a.into(),open:false,stop_reason:s});for &n in r{self.relate(id,Relation::Retains,n);}id}
pub fn reopen(&mut self,c:NodeId,b:NodeId){match self.nodes.get_mut(&c){Some(NodeKind::Commitment{open,..})=>*open=true,_=>panic!("reopen target must be a commitment")};self.relate(b,Relation::Reopens,c);}
pub fn relations(&self,r:Relation)->impl Iterator<Item=&Edge>{self.edges.iter().filter(move|e|e.relation==r)}
pub fn qualification_depth_from(&self,root:NodeId)->usize{fn d(g:&EpistemicGraph,n:NodeId,p:&mut Vec<NodeId>)->usize{if p.contains(&n){return 0}p.push(n);let m=g.edges.iter().filter(|e|e.relation==Relation::Qualifies&&e.to==n).map(|e|1+d(g,e.from,p)).max().unwrap_or(0);p.pop();m}d(self,root,&mut vec![])}
}

#[cfg(test)] mod tests{use super::*;#[test]fn source_to_graph(){let s="claim release_ready; caveat e1 consequence low; caveat e2 consequence low; e1 qualifies release_ready; e2 qualifies e1; commit ship_v1 because enough retaining e1, e2;";let p=parser::parse(s).unwrap();let x=eval::evaluate(&p).unwrap();let root=x.symbols["release_ready"];let k=x.symbols["ship_v1"];assert_eq!(x.graph.qualification_depth_from(root),2);assert_eq!(x.graph.relations(Relation::Retains).filter(|e|e.from==k).count(),2);assert!(matches!(x.graph.nodes.get(&k),Some(NodeKind::Commitment{stop_reason:StopReason::Enough,..})));}}
