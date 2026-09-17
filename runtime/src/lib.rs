pub mod ast;

use std::collections::HashMap;

pub type NodeId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Consequence { Negligible, Low, Material, High, Catastrophic }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attention { Unexamined, Deferred, Examining, Examined }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopReason { Enough, BudgetExhausted, Deadline, ExternalDecision(String) }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Claim { proposition: String }, Evidence { description: String, source: String },
    Caveat { description: String, consequence: Consequence, attention: Attention },
    Context { name: String }, Commitment { action: String, open: bool, stop_reason: StopReason },
    Observation { description: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation { Supports, Opposes, Qualifies, InContext, Retains, Reopens }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge { pub from: NodeId, pub to: NodeId, pub relation: Relation }
#[derive(Debug, Default)]
pub struct EpistemicGraph { next_id: NodeId, pub nodes: HashMap<NodeId, NodeKind>, pub edges: Vec<Edge> }

impl EpistemicGraph {
    pub fn new() -> Self { Self { next_id: 1, ..Self::default() } }
    pub fn add(&mut self, node: NodeKind) -> NodeId { let id=self.next_id; self.next_id+=1; self.nodes.insert(id,node); id }
    pub fn add_caveat(&mut self, description: impl Into<String>, consequence: Consequence) -> NodeId { self.add(NodeKind::Caveat { description: description.into(), consequence, attention: Attention::Unexamined }) }
    pub fn relate(&mut self, from: NodeId, relation: Relation, to: NodeId) { assert!(self.nodes.contains_key(&from)); assert!(self.nodes.contains_key(&to)); self.edges.push(Edge{from,to,relation}); }
    pub fn set_attention(&mut self, caveat: NodeId, attention: Attention) { match self.nodes.get_mut(&caveat) { Some(NodeKind::Caveat{attention:current,..})=>*current=attention,_=>panic!("attention target must be a caveat") } }
    pub fn consequence(&self, caveat: NodeId) -> Consequence { match self.nodes.get(&caveat) { Some(NodeKind::Caveat{consequence,..})=>*consequence,_=>panic!("consequence target must be a caveat") } }
    pub fn commit(&mut self, action: impl Into<String>, retained:&[NodeId])->NodeId { self.commit_because(action,retained,StopReason::Enough) }
    pub fn commit_because(&mut self, action:impl Into<String>, retained:&[NodeId], stop_reason:StopReason)->NodeId { let id=self.add(NodeKind::Commitment{action:action.into(),open:false,stop_reason}); for &n in retained { self.relate(id,Relation::Retains,n); } id }
    pub fn reopen(&mut self, commitment:NodeId,because:NodeId){ match self.nodes.get_mut(&commitment){Some(NodeKind::Commitment{open,..})=>*open=true,_=>panic!("reopen target must be a commitment")}; self.relate(because,Relation::Reopens,commitment); }
    pub fn relations(&self, relation:Relation)->impl Iterator<Item=&Edge>{self.edges.iter().filter(move|e|e.relation==relation)}
    pub fn qualification_depth_from(&self,root:NodeId)->usize{fn d(g:&EpistemicGraph,n:NodeId,p:&mut Vec<NodeId>)->usize{if p.contains(&n){return 0}p.push(n);let best=g.edges.iter().filter(|e|e.relation==Relation::Qualifies&&e.to==n).map(|e|1+d(g,e.from,p)).max().unwrap_or(0);p.pop();best}d(self,root,&mut Vec::new())}
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn canonical_001(){let mut g=EpistemicGraph::new();let a=g.add(NodeKind::Claim{proposition:"cat".into()});let b=g.add(NodeKind::Claim{proposition:"not_cat".into()});let e=g.add(NodeKind::Evidence{description:"photo".into(),source:"camera".into()});let c=g.add_caveat("label_misplaced",Consequence::Material);g.relate(e,Relation::Supports,a);let k=g.commit("treat_as_cat",&[b,c]);g.reopen(k,c);assert!(matches!(g.nodes.get(&k),Some(NodeKind::Commitment{open:true,..})));}
 #[test] fn canonical_002(){let mut g=EpistemicGraph::new();let b=g.add(NodeKind::Claim{proposition:"bridge_safe".into()});let p=g.add_caveat("paint",Consequence::Negligible);let w=g.add_caveat("crosswind",Consequence::Catastrophic);g.relate(p,Relation::Qualifies,b);g.relate(w,Relation::Qualifies,b);g.set_attention(p,Attention::Deferred);g.set_attention(w,Attention::Examining);assert_eq!(g.consequence(w),Consequence::Catastrophic);let k=g.commit("continue_review",&[p,w]);assert_eq!(g.relations(Relation::Retains).filter(|e|e.from==k).count(),2);}
 #[test] fn canonical_003(){let mut g=EpistemicGraph::new();let r=g.add(NodeKind::Claim{proposition:"release_ready".into()});let a=g.add_caveat("edge1",Consequence::Low);let b=g.add_caveat("edge2",Consequence::Low);let c=g.add_caveat("edge3",Consequence::Low);g.relate(a,Relation::Qualifies,r);g.relate(b,Relation::Qualifies,a);g.relate(c,Relation::Qualifies,b);assert_eq!(g.qualification_depth_from(r),3);let k=g.commit_because("ship_v1",&[a,b,c],StopReason::Enough);assert!(matches!(g.nodes.get(&k),Some(NodeKind::Commitment{stop_reason:StopReason::Enough,..})));assert_eq!(g.qualification_depth_from(r),3);}
}
