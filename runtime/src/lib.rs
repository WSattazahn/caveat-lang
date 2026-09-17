use std::collections::HashMap;

pub type NodeId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Consequence {
    Negligible,
    Low,
    Material,
    High,
    Catastrophic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attention {
    Unexamined,
    Deferred,
    Examining,
    Examined,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Claim { proposition: String },
    Evidence { description: String, source: String },
    Caveat { description: String, consequence: Consequence, attention: Attention },
    Context { name: String },
    Commitment { action: String, open: bool },
    Observation { description: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    Supports,
    Opposes,
    Qualifies,
    InContext,
    Retains,
    Reopens,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub relation: Relation,
}

#[derive(Debug, Default)]
pub struct EpistemicGraph {
    next_id: NodeId,
    pub nodes: HashMap<NodeId, NodeKind>,
    pub edges: Vec<Edge>,
}

impl EpistemicGraph {
    pub fn new() -> Self { Self { next_id: 1, ..Self::default() } }

    pub fn add(&mut self, node: NodeKind) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, node);
        id
    }

    pub fn add_caveat(&mut self, description: impl Into<String>, consequence: Consequence) -> NodeId {
        self.add(NodeKind::Caveat {
            description: description.into(),
            consequence,
            attention: Attention::Unexamined,
        })
    }

    pub fn relate(&mut self, from: NodeId, relation: Relation, to: NodeId) {
        assert!(self.nodes.contains_key(&from), "source node does not exist");
        assert!(self.nodes.contains_key(&to), "target node does not exist");
        self.edges.push(Edge { from, to, relation });
    }

    pub fn set_attention(&mut self, caveat: NodeId, attention: Attention) {
        match self.nodes.get_mut(&caveat) {
            Some(NodeKind::Caveat { attention: current, .. }) => *current = attention,
            _ => panic!("attention target must be a caveat"),
        }
    }

    pub fn consequence(&self, caveat: NodeId) -> Consequence {
        match self.nodes.get(&caveat) {
            Some(NodeKind::Caveat { consequence, .. }) => *consequence,
            _ => panic!("consequence target must be a caveat"),
        }
    }

    pub fn commit(&mut self, action: impl Into<String>, retained: &[NodeId]) -> NodeId {
        let id = self.add(NodeKind::Commitment { action: action.into(), open: false });
        for &node in retained { self.relate(id, Relation::Retains, node); }
        id
    }

    pub fn reopen(&mut self, commitment: NodeId, because: NodeId) {
        match self.nodes.get_mut(&commitment) {
            Some(NodeKind::Commitment { open, .. }) => *open = true,
            _ => panic!("reopen target must be a commitment"),
        }
        self.relate(because, Relation::Reopens, commitment);
    }

    pub fn relations(&self, relation: Relation) -> impl Iterator<Item=&Edge> {
        self.edges.iter().filter(move |edge| edge.relation == relation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_001_preserves_contradiction_commitment_and_reopening() {
        let mut g = EpistemicGraph::new();
        let cat = g.add(NodeKind::Claim { proposition: "creature_is_cat".into() });
        let not_cat = g.add(NodeKind::Claim { proposition: "creature_is_not_cat".into() });
        let photo = g.add(NodeKind::Evidence { description: "photo_1".into(), source: "camera".into() });
        let label = g.add(NodeKind::Evidence { description: "label_1".into(), source: "display_label".into() });
        let misplaced = g.add_caveat("label_may_be_misplaced", Consequence::Material);

        g.relate(photo, Relation::Supports, cat);
        g.relate(label, Relation::Supports, not_cat);
        g.relate(misplaced, Relation::Qualifies, label);

        let commitment = g.commit("treat_as_cat", &[not_cat, misplaced]);
        assert!(g.nodes.contains_key(&cat) && g.nodes.contains_key(&not_cat));

        let record = g.add(NodeKind::Evidence { description: "curator_record_7".into(), source: "archive".into() });
        g.relate(record, Relation::Opposes, label);
        g.reopen(commitment, misplaced);

        assert!(matches!(g.nodes.get(&commitment), Some(NodeKind::Commitment { open: true, .. })));
        assert_eq!(g.relations(Relation::Reopens).count(), 1);
    }

    #[test]
    fn canonical_002_preserves_both_caveats_while_attention_differs() {
        let mut g = EpistemicGraph::new();
        let bridge_safe = g.add(NodeKind::Claim { proposition: "bridge_safe".into() });
        let load_test = g.add(NodeKind::Evidence { description: "static_load_test".into(), source: "lab".into() });
        let paint = g.add_caveat("paint_color_variation", Consequence::Negligible);
        let crosswind = g.add_caveat("untested_crosswind", Consequence::Catastrophic);

        g.relate(load_test, Relation::Supports, bridge_safe);
        g.relate(paint, Relation::Qualifies, bridge_safe);
        g.relate(crosswind, Relation::Qualifies, bridge_safe);

        g.set_attention(paint, Attention::Deferred);
        g.set_attention(crosswind, Attention::Examining);

        assert_eq!(g.consequence(paint), Consequence::Negligible);
        assert_eq!(g.consequence(crosswind), Consequence::Catastrophic);
        assert!(matches!(g.nodes.get(&paint), Some(NodeKind::Caveat { attention: Attention::Deferred, .. })));
        assert!(matches!(g.nodes.get(&crosswind), Some(NodeKind::Caveat { attention: Attention::Examining, .. })));

        let commitment = g.commit("continue_review", &[paint, crosswind]);
        assert_eq!(g.relations(Relation::Retains).filter(|e| e.from == commitment).count(), 2);
    }
}
