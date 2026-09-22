pub mod action_runtime;
pub mod ast;
pub mod caveat_rs;
pub mod eval;
pub mod game_session;
pub mod graphics;
pub mod link;
pub mod map;
pub mod parser;
pub mod presentation;
pub mod reactive;
mod reactive_expr;
pub mod repeat;
pub mod session;
pub mod source_library;
pub mod web;
#[cfg(feature = "games")]
pub mod web_games;
#[cfg(feature = "games")]
pub mod web_source_library;
pub mod world3d;

use std::collections::{HashMap, HashSet, VecDeque};

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
pub enum StopReason {
    Enough,
    BudgetExhausted,
    Deadline,
    ExternalDecision(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QualificationTopology {
    Finite { max_depth: usize },
    Cyclic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Claim {
        proposition: String,
    },
    Evidence {
        description: String,
        source: String,
    },
    Caveat {
        description: String,
        consequence: Consequence,
        attention: Attention,
    },
    Context {
        name: String,
    },
    Commitment {
        action: String,
        open: bool,
        stop_reason: StopReason,
    },
    Observation {
        description: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    Supports,
    Opposes,
    Qualifies,
    InContext,
    Retains,
    Reopens,
    /// A decision used this evidence as a basis; this does not assert truth.
    ReliesOn,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub relation: Relation,
    /// Which bundle part asserted this edge, when it was asserted while
    /// declarations were being read. `None` for an edge a running effect
    /// revealed: see spec/caveat-authorship-0.1.md.
    pub origin: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationImpact {
    pub caveat: NodeId,
    pub directly_qualifies: NodeId,
    pub affected: Vec<NodeId>,
}

#[derive(Debug, Default, Clone)]
pub struct EpistemicGraph {
    next_id: NodeId,
    pub nodes: HashMap<NodeId, NodeKind>,
    pub edges: Vec<Edge>,
    /// The part whose declarations are being read right now.
    authoring: Option<String>,
    /// Which part declared each node. A program with no `origin` statement
    /// records nothing rather than guessing, so "unattributed" is visible
    /// instead of being filled in with a default.
    pub origins: HashMap<NodeId, String>,
}

impl EpistemicGraph {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            ..Self::default()
        }
    }
    /// Record which part is declaring from here on. `None` means the caller
    /// cannot honestly attribute what follows, and nothing is recorded.
    pub fn set_authoring(&mut self, part: Option<String>) {
        self.authoring = part;
    }

    /// Which part declared this node, if it was declared under an origin.
    pub fn origin(&self, node: NodeId) -> Option<&str> {
        self.origins.get(&node).map(String::as_str)
    }

    pub fn add(&mut self, node: NodeKind) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, node);
        if let Some(part) = &self.authoring {
            self.origins.insert(id, part.clone());
        }
        id
    }
    pub fn add_caveat(
        &mut self,
        description: impl Into<String>,
        consequence: Consequence,
    ) -> NodeId {
        self.add(NodeKind::Caveat {
            description: description.into(),
            consequence,
            attention: Attention::Unexamined,
        })
    }
    pub fn relate(&mut self, from: NodeId, relation: Relation, to: NodeId) {
        assert!(self.nodes.contains_key(&from));
        assert!(self.nodes.contains_key(&to));
        self.edges.push(Edge {
            from,
            to,
            relation,
            origin: self.authoring.clone(),
        });
    }
    pub fn set_attention(&mut self, caveat: NodeId, state: Attention) {
        match self.nodes.get_mut(&caveat) {
            Some(NodeKind::Caveat { attention, .. }) => *attention = state,
            _ => panic!("attention target must be a caveat"),
        }
    }
    pub fn consequence(&self, caveat: NodeId) -> Consequence {
        match self.nodes.get(&caveat) {
            Some(NodeKind::Caveat { consequence, .. }) => *consequence,
            _ => panic!("consequence target must be a caveat"),
        }
    }
    pub fn commit_because(
        &mut self,
        action: impl Into<String>,
        retained: &[NodeId],
        reason: StopReason,
    ) -> NodeId {
        let id = self.add(NodeKind::Commitment {
            action: action.into(),
            open: false,
            stop_reason: reason,
        });
        for &node in retained {
            self.relate(id, Relation::Retains, node);
        }
        id
    }
    pub fn commit(&mut self, action: impl Into<String>, retained: &[NodeId]) -> NodeId {
        self.commit_because(action, retained, StopReason::Enough)
    }
    pub fn reopen(&mut self, commitment: NodeId, because: NodeId) {
        match self.nodes.get_mut(&commitment) {
            Some(NodeKind::Commitment { open, .. }) => *open = true,
            _ => panic!("reopen target must be a commitment"),
        };
        self.relate(because, Relation::Reopens, commitment);
    }
    pub fn relations(&self, relation: Relation) -> impl Iterator<Item = &Edge> {
        self.edges
            .iter()
            .filter(move |edge| edge.relation == relation)
    }

    pub fn qualification_impacts(&self, caveat: NodeId) -> Vec<QualificationImpact> {
        let mut impacts = Vec::new();
        for direct in self
            .edges
            .iter()
            .filter(|edge| edge.from == caveat && edge.relation == Relation::Qualifies)
            .map(|edge| edge.to)
        {
            let mut seen = HashSet::new();
            let mut queue = VecDeque::from([direct]);
            while let Some(node) = queue.pop_front() {
                if !seen.insert(node) {
                    continue;
                }
                for next in self.edges.iter().filter_map(|edge| {
                    if edge.from == node && edge.relation == Relation::Supports {
                        Some(edge.to)
                    } else if edge.to == node && edge.relation == Relation::ReliesOn {
                        Some(edge.from)
                    } else {
                        None
                    }
                }) {
                    queue.push_back(next);
                }
            }
            let mut affected: Vec<_> = seen.into_iter().collect();
            affected.sort_unstable();
            impacts.push(QualificationImpact {
                caveat,
                directly_qualifies: direct,
                affected,
            });
        }
        impacts
    }

    pub fn qualification_topology_from(&self, root: NodeId) -> QualificationTopology {
        fn depth(
            graph: &EpistemicGraph,
            node: NodeId,
            path: &mut Vec<NodeId>,
        ) -> Result<usize, ()> {
            if path.contains(&node) {
                return Err(());
            }
            path.push(node);
            let mut maximum = 0;
            for edge in graph
                .edges
                .iter()
                .filter(|edge| edge.relation == Relation::Qualifies && edge.to == node)
            {
                maximum = maximum.max(1 + depth(graph, edge.from, path)?);
            }
            path.pop();
            Ok(maximum)
        }
        match depth(self, root, &mut Vec::new()) {
            Ok(max_depth) => QualificationTopology::Finite { max_depth },
            Err(()) => QualificationTopology::Cyclic,
        }
    }

    pub fn qualification_depth_from(&self, root: NodeId) -> usize {
        match self.qualification_topology_from(root) {
            QualificationTopology::Finite { max_depth } => max_depth,
            QualificationTopology::Cyclic => panic!("cyclic qualification has no finite depth"),
        }
    }
}
