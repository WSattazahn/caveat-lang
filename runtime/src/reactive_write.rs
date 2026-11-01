//! One write path for the containers that reads will index: the graph, the
//! names, the observation ledger, the withdrawals and the take-backs. Each is
//! `Sealed`: anything can read it as the `Arc` it holds, and only this module
//! can make one or write into one, so no write can skip what its session
//! methods come to keep up to date. Each method makes the same change, in the
//! same container, that its callers made before it existed. Outside this
//! module a sealed container can still be copied, so code that holds a
//! session could swap in a copy of one whole, from this session or another;
//! nothing does.
use super::*;
use crate::Edge;
use std::fmt;
use std::ops::Deref;

/// A shared container that reads like the `Arc` it holds. Only this module
/// makes one or writes into one, through the session methods below; there is
/// no empty one to take, and nothing outside can reach the `Arc` to change it.
pub(super) struct Sealed<T>(Arc<T>);

impl<T> Sealed<T> {
    fn new(value: T) -> Self {
        Self(Arc::new(value))
    }
}

impl<T> Deref for Sealed<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Arc<T> {
        &self.0
    }
}

impl<T> Clone for Sealed<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

/// Prints what the `Arc` printed, so a session's debug output is unchanged.
impl<T: fmt::Debug> fmt::Debug for Sealed<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A session's sealed containers as its program declares them: the graph and
/// its names, and no observations, withdrawals or take-backs yet.
pub(super) struct Declared {
    pub(super) graph: Sealed<EpistemicGraph>,
    pub(super) symbols: Sealed<HashMap<String, NodeId>>,
    pub(super) observations: Sealed<Vec<String>>,
    pub(super) withdrawals: Sealed<Vec<Withdrawal>>,
    pub(super) taken_back: Sealed<Vec<TakeBack>>,
}

/// The graph and names a program declares, sealed for the session it loads.
pub(super) fn seal_declared(graph: EpistemicGraph, symbols: HashMap<String, NodeId>) -> Declared {
    Declared {
        graph: Sealed::new(graph),
        symbols: Sealed::new(symbols),
        observations: Sealed::new(Vec::new()),
        withdrawals: Sealed::new(Vec::new()),
        taken_back: Sealed::new(Vec::new()),
    }
}

impl ReactiveSession {
    /// Adds a node and names it.
    pub(super) fn add_named(&mut self, name: String, kind: NodeKind) -> NodeId {
        let id = Arc::make_mut(&mut self.graph.0).add(kind);
        Arc::make_mut(&mut self.symbols.0).insert(name, id);
        id
    }

    /// Adds an unexamined caveat and names it.
    pub(super) fn add_named_caveat(
        &mut self,
        name: String,
        description: &str,
        consequence: Consequence,
    ) -> NodeId {
        let id = Arc::make_mut(&mut self.graph.0).add_caveat(description, consequence);
        Arc::make_mut(&mut self.symbols.0).insert(name, id);
        id
    }

    /// Adds a commitment that retains `retained`, and names it.
    pub(super) fn commit_named(
        &mut self,
        name: String,
        retained: &[NodeId],
        reason: StopReason,
    ) -> NodeId {
        let id = Arc::make_mut(&mut self.graph.0).commit_because(&name, retained, reason);
        Arc::make_mut(&mut self.symbols.0).insert(name, id);
        id
    }

    pub(super) fn relate(&mut self, from: NodeId, relation: Relation, to: NodeId) {
        Arc::make_mut(&mut self.graph.0).relate(from, relation, to);
    }

    pub(super) fn set_node_attention(&mut self, caveat: NodeId, state: Attention) {
        Arc::make_mut(&mut self.graph.0).set_attention(caveat, state);
    }

    /// Opens `commitment` and records that `because` reopened it.
    pub(super) fn reopen_commitment(&mut self, commitment: NodeId, because: NodeId) {
        Arc::make_mut(&mut self.graph.0).reopen(commitment, because);
    }

    /// Opens a restored commitment; false when `id` is not a commitment.
    pub(super) fn open_restored_commitment(&mut self, id: NodeId) -> bool {
        match Arc::make_mut(&mut self.graph.0).nodes.get_mut(&id) {
            Some(NodeKind::Commitment { open, .. }) => {
                *open = true;
                true
            }
            _ => false,
        }
    }

    /// Removes departing records' nodes, names and every edge that touches
    /// one, passing each removed edge to `removed` in graph order.
    pub(super) fn depart_records(
        &mut self,
        ids: &HashMap<NodeId, String>,
        mut removed: impl FnMut(&Edge),
    ) {
        let graph = Arc::make_mut(&mut self.graph.0);
        graph.edges.retain(|edge| {
            if ids.contains_key(&edge.from) || ids.contains_key(&edge.to) {
                removed(edge);
                false
            } else {
                true
            }
        });
        for id in ids.keys() {
            graph.nodes.remove(id);
            graph.origins.remove(id);
        }
        let symbols = Arc::make_mut(&mut self.symbols.0);
        for record in ids.values() {
            symbols.remove(record);
        }
    }

    /// Replaces the observation ledger: its first materialization, or a
    /// restore.
    pub(super) fn set_observations(&mut self, observations: Vec<String>) {
        self.observations = Sealed::new(observations);
    }

    pub(super) fn push_observation(&mut self, evidence: String) {
        Arc::make_mut(&mut self.observations.0).push(evidence);
    }

    pub(super) fn retain_observations(&mut self, keep: impl FnMut(&String) -> bool) {
        Arc::make_mut(&mut self.observations.0).retain(keep);
    }

    pub(super) fn set_withdrawals(&mut self, withdrawals: Vec<Withdrawal>) {
        self.withdrawals = Sealed::new(withdrawals);
    }

    pub(super) fn push_withdrawal(&mut self, withdrawal: Withdrawal) {
        Arc::make_mut(&mut self.withdrawals.0).push(withdrawal);
    }

    /// Removes and returns the withdrawals of departing records.
    pub(super) fn depart_withdrawals(
        &mut self,
        departing: &BTreeMap<String, (String, u64)>,
    ) -> Vec<Withdrawal> {
        departure::extract_departing_withdrawals(&mut self.withdrawals.0, departing)
    }

    pub(super) fn set_taken_back(&mut self, records: Vec<TakeBack>) {
        self.taken_back = Sealed::new(records);
    }

    pub(super) fn push_take_back(&mut self, record: TakeBack) {
        Arc::make_mut(&mut self.taken_back.0).push(record);
    }

    pub(super) fn retain_taken_back(&mut self, keep: impl FnMut(&TakeBack) -> bool) {
        Arc::make_mut(&mut self.taken_back.0).retain(keep);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cloned_container_shares_what_it_holds() {
        let sealed = Sealed::new(vec![1, 2]);
        let clone = sealed.clone();
        assert!(Arc::ptr_eq(&sealed, &clone));
    }

    #[test]
    fn a_sealed_container_prints_what_its_arc_printed() {
        let sealed = Sealed::new(vec![String::from("a")]);
        let arc = Arc::new(vec![String::from("a")]);
        assert_eq!(format!("{sealed:?}"), format!("{arc:?}"));
        assert_eq!(format!("{sealed:#?}"), format!("{arc:#?}"));
    }
}
