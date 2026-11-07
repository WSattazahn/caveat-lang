//! One write path for the containers that reads will index: the graph, the
//! names, the observation ledger, the withdrawals and the take-backs. Each is
//! `Sealed`: anything can read it as the `Arc` it holds, and only this module
//! can make one or write into one, so no write can skip what its session
//! methods keep up to date. Each method makes the same change, in the same
//! container, that its callers made before it existed. Outside this module a
//! sealed container can still be copied, so code that holds a session could
//! swap in a copy of one whole, from this session or another; nothing does,
//! and an index kept inside its container, as the names keep theirs by node
//! (`NameTable`), moves with it.
//!
//! The graph and the session's keyed records (`Records`) also log what an
//! event writes to them, from the start of the event until `Changes` has
//! read what it changed, so that check reads only what the event wrote.
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
    pub(super) graph: SealedGraph,
    pub(super) symbols: NameTable,
    pub(super) observations: Sealed<Vec<String>>,
    pub(super) withdrawals: Sealed<Vec<Withdrawal>>,
    pub(super) taken_back: Sealed<Vec<TakeBack>>,
}

/// The graph and names a program declares, sealed for the session it loads.
pub(super) fn seal_declared(graph: EpistemicGraph, symbols: HashMap<String, NodeId>) -> Declared {
    Declared {
        graph: SealedGraph::new(graph),
        symbols: NameTable::new(symbols),
        observations: Sealed::new(Vec::new()),
        withdrawals: Sealed::new(Vec::new()),
        taken_back: Sealed::new(Vec::new()),
    }
}

/// The session's graph, sealed like the other containers, with the nodes an
/// event wrote. It reads as the graph's `Arc`. Only the session methods below
/// write it: each says which node it adds or changes, and until records
/// depart an event only adds edges. So after an event `Changes` compares
/// only the nodes it wrote and the edges past the old count, where it
/// compared every node and edge before.
pub(super) struct SealedGraph {
    graph: Arc<EpistemicGraph>,
    written: Written<NodeId>,
}

impl SealedGraph {
    fn new(graph: EpistemicGraph) -> Self {
        Self {
            graph: Arc::new(graph),
            written: Written::Any,
        }
    }

    /// The graph, to add edges to or to write the nodes the caller notes.
    fn write(&mut self) -> &mut EpistemicGraph {
        Arc::make_mut(&mut self.graph)
    }

    /// The graph, to write anything: the log no longer says what changed.
    fn write_any(&mut self) -> &mut EpistemicGraph {
        self.written = Written::Any;
        Arc::make_mut(&mut self.graph)
    }

    fn note(&mut self, id: NodeId) {
        if let Written::Only(ids) = &mut self.written {
            ids.insert(id);
        }
    }

    /// Whether this graph's nodes and edges are `old`'s, where `old` is the
    /// session this event began from.
    pub(super) fn unchanged_since(&self, old: &Self) -> bool {
        if Arc::ptr_eq(&self.graph, &old.graph) {
            return true;
        }
        let (new, old) = (&*self.graph, &*old.graph);
        let unchanged = match &self.written {
            Written::Only(ids) => {
                new.edges.len() == old.edges.len()
                    && ids.iter().all(|id| new.nodes.get(id) == old.nodes.get(id))
            }
            Written::Any => new.edges == old.edges && new.nodes == old.nodes,
        };
        debug_assert_eq!(
            unchanged,
            new.edges == old.edges && new.nodes == old.nodes,
            "an event wrote to the graph without saying what"
        );
        unchanged
    }

    /// What this event did to the graph, where `old` is the session it began
    /// from: the edges it added, and the nodes it added or changed. `None` if
    /// it removed a node or changed an edge `old` holds.
    pub(super) fn added_since<'a>(&'a self, old: &Self) -> Option<(&'a [Edge], BTreeSet<NodeId>)> {
        let found = added(&self.graph, &old.graph, self.written.only());
        #[cfg(debug_assertions)]
        if self.written.only().is_some() {
            assert_eq!(
                found,
                added(&self.graph, &old.graph, None),
                "an event wrote to the graph without saying what"
            );
        }
        found
    }
}

/// `added_since`, reading only the nodes of `written` and taking the edges
/// past `old`'s as added, or reading every node and edge when `None`.
fn added<'a>(
    new: &'a EpistemicGraph,
    old: &EpistemicGraph,
    written: Option<&BTreeSet<NodeId>>,
) -> Option<(&'a [Edge], BTreeSet<NodeId>)> {
    let edges = match written {
        Some(_) => new.edges.get(old.edges.len()..)?,
        None => new.edges.strip_prefix(old.edges.as_slice())?,
    };
    let mut nodes = BTreeSet::new();
    let mut check = |id: NodeId| -> Option<()> {
        match (old.nodes.get(&id), new.nodes.get(&id)) {
            (Some(_), None) => return None,
            (previous, Some(node)) if previous != Some(node) => {
                nodes.insert(id);
            }
            _ => {}
        }
        Some(())
    };
    match written {
        Some(ids) => {
            for &id in ids {
                check(id)?;
            }
        }
        None => {
            if old.nodes.keys().any(|id| !new.nodes.contains_key(id)) {
                return None;
            }
            for &id in new.nodes.keys() {
                check(id)?;
            }
        }
    }
    Some((edges, nodes))
}

impl Deref for SealedGraph {
    type Target = Arc<EpistemicGraph>;

    fn deref(&self) -> &Arc<EpistemicGraph> {
        &self.graph
    }
}

impl Clone for SealedGraph {
    fn clone(&self) -> Self {
        Self {
            graph: Arc::clone(&self.graph),
            written: self.written.clone(),
        }
    }
}

/// Prints what the `Arc` printed, so a session's debug output is unchanged.
impl fmt::Debug for SealedGraph {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.graph.fmt(formatter)
    }
}

/// The names map, sealed like the other containers, with each named record's
/// name kept by node beside it. It reads as the map's `Arc`, and `name_of`
/// reads a node's name. Only the session methods below write it, and they
/// write both together, so a copy of the table carries both.
#[derive(Clone)]
pub(super) struct NameTable {
    by_name: Arc<HashMap<String, NodeId>>,
    by_node: Arc<NodeNames>,
}

impl Deref for NameTable {
    type Target = Arc<HashMap<String, NodeId>>;

    fn deref(&self) -> &Arc<HashMap<String, NodeId>> {
        &self.by_name
    }
}

/// Prints what the names map's `Arc` printed, so a session's debug output is
/// unchanged.
impl fmt::Debug for NameTable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.by_name.fmt(formatter)
    }
}

impl NameTable {
    fn new(by_name: HashMap<String, NodeId>) -> Self {
        let mut by_node = NodeNames::default();
        for (name, &id) in &by_name {
            debug_assert!(by_node.get(id).is_none(), "a node has one name");
            by_node.set(id, Some(Arc::from(name.as_str())));
        }
        Self {
            by_name: Arc::new(by_name),
            by_node: Arc::new(by_node),
        }
    }

    /// The name of `id`'s record, if it has one.
    pub(super) fn name_of(&self, id: NodeId) -> Option<&str> {
        self.by_node.get(id)
    }

    /// Whether `other` holds every name this table holds, each for the same
    /// node. Chunks of names the two share are not read.
    pub(super) fn kept_in(&self, other: &Self) -> bool {
        self.by_node.kept_in(&other.by_node)
    }

    /// Names `id`. A name given again leaves the node it named without one.
    fn insert(&mut self, name: String, id: NodeId) {
        let by_node = Arc::make_mut(&mut self.by_node);
        debug_assert!(
            by_node.get(id).is_none_or(|held| held == name),
            "a node has one name"
        );
        let shared = Arc::<str>::from(name.as_str());
        if let Some(previous) = Arc::make_mut(&mut self.by_name).insert(name, id) {
            by_node.set(previous, None);
        }
        by_node.set(id, Some(shared));
    }

    /// Removes `name`, and the name of the node it named.
    fn remove(&mut self, name: &str) {
        if let Some(id) = Arc::make_mut(&mut self.by_name).remove(name) {
            Arc::make_mut(&mut self.by_node).set(id, None);
        }
    }

    /// The names by node are the names map turned around, exactly.
    #[cfg(any(debug_assertions, test))]
    pub(super) fn check(&self) {
        for (name, &id) in self.by_name.iter() {
            debug_assert_eq!(
                self.by_node.get(id),
                Some(name.as_str()),
                "the name of node {id}"
            );
        }
        debug_assert_eq!(self.by_node.len(), self.by_name.len(), "names by node");
    }
}

/// How many names one chunk of `NodeNames` holds.
const NAME_CHUNK: usize = 64;
/// `NAME_CHUNK` counted in node ids.
const CHUNK_NODES: NodeId = NAME_CHUNK as NodeId;

type NameChunk = [Option<Arc<str>>; NAME_CHUNK];

/// Each named record's name, by node, in fixed-size chunks keyed by where
/// they sit among node ids. Ids count up and are never reused, and departure
/// removes old records, so a chunk that names nothing is dropped: what the
/// names hold, and what a copy or a comparison of them reads, grows with the
/// records named now rather than with every id the session has issued. A copy
/// shares every chunk, and a write copies only the chunk it changes.
#[derive(Clone, Default)]
struct NodeNames {
    chunks: BTreeMap<NodeId, Arc<NameChunk>>,
}

impl NodeNames {
    /// The chunk and slot that hold `id`'s name.
    fn place(id: NodeId) -> (NodeId, usize) {
        (id / CHUNK_NODES, (id % CHUNK_NODES) as usize)
    }

    fn get(&self, id: NodeId) -> Option<&str> {
        let (chunk, slot) = Self::place(id);
        self.chunks.get(&chunk)?[slot].as_deref()
    }

    fn set(&mut self, id: NodeId, name: Option<Arc<str>>) {
        let (at, slot) = Self::place(id);
        if let Some(name) = name {
            let chunk = self
                .chunks
                .entry(at)
                .or_insert_with(|| Arc::new(std::array::from_fn(|_| None)));
            Arc::make_mut(chunk)[slot] = Some(name);
            return;
        }
        let Some(chunk) = self.chunks.get_mut(&at) else {
            return;
        };
        if chunk[slot].is_none() {
            return;
        }
        // The last name a chunk holds takes the chunk with it, uncopied.
        if chunk
            .iter()
            .enumerate()
            .all(|(held, name)| held == slot || name.is_none())
        {
            self.chunks.remove(&at);
        } else {
            Arc::make_mut(chunk)[slot] = None;
        }
    }

    /// Whether every node named here has the same name in `other`. Each
    /// chunk held names a node, so one `other` lacks is not kept.
    fn kept_in(&self, other: &Self) -> bool {
        self.chunks.iter().all(|(at, chunk)| {
            other.chunks.get(at).is_some_and(|kept| {
                Arc::ptr_eq(chunk, kept)
                    || chunk.iter().zip(kept.iter()).all(|(name, kept)| {
                        name.as_ref().is_none_or(|name| {
                            kept.as_ref()
                                .is_some_and(|kept| Arc::ptr_eq(name, kept) || name == kept)
                        })
                    })
            })
        })
    }

    /// How many nodes are named. No chunk held names nothing.
    #[cfg(any(debug_assertions, test))]
    fn len(&self) -> usize {
        self.chunks
            .values()
            .map(|chunk| {
                let named = chunk.iter().filter(|name| name.is_some()).count();
                debug_assert!(named > 0, "a chunk that names nothing is dropped");
                named
            })
            .sum()
    }
}

/// A session's records of one kind, by name, with the names written since
/// the event began. It reads like the map's `Arc`; only the methods here
/// write it, and each says which record it writes, so after an event
/// `Changes` compares only the records the event wrote, where it compared
/// every record of the kind before.
pub(super) struct Records<V> {
    map: Arc<BTreeMap<String, V>>,
    written: Written<String>,
}

/// What an event wrote to one container.
#[derive(Clone, Debug)]
enum Written<K> {
    /// These, and no other: everything else is as the event found it.
    Only(BTreeSet<K>),
    /// Anything: written outside an event, or by a write that does not say
    /// what.
    Any,
}

impl<K> Written<K> {
    fn only(&self) -> Option<&BTreeSet<K>> {
        match self {
            Self::Only(keys) => Some(keys),
            Self::Any => None,
        }
    }
}

impl<V> Records<V> {
    pub(super) fn new(map: BTreeMap<String, V>) -> Self {
        Self {
            map: Arc::new(map),
            written: Written::Any,
        }
    }

    /// Starts the log of an event's writes: none yet.
    fn begin(&mut self) {
        self.written = Written::Only(BTreeSet::new());
    }

    /// Ends the log: past the event's check of what it changed, any record
    /// may change without saying which.
    fn end(&mut self) {
        self.written = Written::Any;
    }

    /// The names written since the event began, if the writes said which;
    /// `None` if any record may have changed.
    pub(super) fn written(&self) -> Option<&BTreeSet<String>> {
        self.written.only()
    }

    fn note(&mut self, name: &str) {
        if let Written::Only(names) = &mut self.written {
            if !names.contains(name) {
                names.insert(name.to_string());
            }
        }
    }
}

impl<V: Clone> Records<V> {
    pub(super) fn insert(&mut self, name: String, value: V) -> Option<V> {
        self.note(&name);
        Arc::make_mut(&mut self.map).insert(name, value)
    }

    /// The record `name`, to write, if there is one. Copies the map first
    /// when another session shares it.
    pub(super) fn get_mut(&mut self, name: &str) -> Option<&mut V> {
        if !self.map.contains_key(name) {
            return None;
        }
        self.note(name);
        Arc::make_mut(&mut self.map).get_mut(name)
    }

    pub(super) fn entry(
        &mut self,
        name: String,
    ) -> std::collections::btree_map::Entry<'_, String, V> {
        self.note(&name);
        Arc::make_mut(&mut self.map).entry(name)
    }

    pub(super) fn remove(&mut self, name: &str) -> Option<V> {
        self.note(name);
        Arc::make_mut(&mut self.map).remove(name)
    }

    /// The whole map, to write any of it: the log no longer says which
    /// records changed.
    pub(super) fn make_mut(&mut self) -> &mut BTreeMap<String, V> {
        self.written = Written::Any;
        Arc::make_mut(&mut self.map)
    }
}

impl<V: PartialEq> Records<V> {
    /// Whether these are the records `old` holds, where `old` is the session
    /// this event began from: only the records it wrote can differ, when its
    /// log says which.
    pub(super) fn unchanged_since(&self, old: &Self) -> bool {
        if Arc::ptr_eq(&self.map, &old.map) {
            return true;
        }
        let unchanged = match &self.written {
            Written::Only(names) => names
                .iter()
                .all(|name| self.map.get(name) == old.map.get(name)),
            Written::Any => self.map == old.map,
        };
        debug_assert_eq!(
            unchanged,
            self.map == old.map,
            "an event wrote a record its log does not name"
        );
        unchanged
    }
}

impl<V> Deref for Records<V> {
    type Target = Arc<BTreeMap<String, V>>;

    fn deref(&self) -> &Arc<BTreeMap<String, V>> {
        &self.map
    }
}

impl<V> Clone for Records<V> {
    fn clone(&self) -> Self {
        Self {
            map: Arc::clone(&self.map),
            written: self.written.clone(),
        }
    }
}

impl<V> Default for Records<V> {
    fn default() -> Self {
        Self::new(BTreeMap::new())
    }
}

/// Prints what the map's `Arc` printed, so a session's debug output is
/// unchanged.
impl<V: fmt::Debug> fmt::Debug for Records<V> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.map.fmt(formatter)
    }
}

impl ReactiveSession {
    /// Starts the log of the records this event writes.
    pub(super) fn begin_write_log(&mut self) {
        self.graph.written = Written::Only(BTreeSet::new());
        self.reading_streams.begin();
        self.decision_series.begin();
        self.commitment_bases.begin();
        self.commitment_grounds.begin();
        self.observation_qualifications.begin();
        self.examination_qualifications.begin();
        self.reopening_qualifications.begin();
    }

    /// Ends the log once the event has read what it changed.
    pub(super) fn end_write_log(&mut self) {
        self.graph.written = Written::Any;
        self.reading_streams.end();
        self.decision_series.end();
        self.commitment_bases.end();
        self.commitment_grounds.end();
        self.observation_qualifications.end();
        self.examination_qualifications.end();
        self.reopening_qualifications.end();
    }

    /// Adds a node and names it.
    pub(super) fn add_named(&mut self, name: String, kind: NodeKind) -> NodeId {
        let id = self.graph.write().add(kind);
        self.graph.note(id);
        self.symbols.insert(name, id);
        id
    }

    /// Adds an unexamined caveat and names it.
    pub(super) fn add_named_caveat(
        &mut self,
        name: String,
        description: &str,
        consequence: Consequence,
    ) -> NodeId {
        let id = self.graph.write().add_caveat(description, consequence);
        self.graph.note(id);
        self.symbols.insert(name, id);
        id
    }

    /// Adds a commitment that retains `retained`, and names it.
    pub(super) fn commit_named(
        &mut self,
        name: String,
        retained: &[NodeId],
        reason: StopReason,
    ) -> NodeId {
        let id = self.graph.write().commit_because(&name, retained, reason);
        self.graph.note(id);
        self.symbols.insert(name, id);
        id
    }

    pub(super) fn relate(&mut self, from: NodeId, relation: Relation, to: NodeId) {
        self.graph.write().relate(from, relation, to);
    }

    pub(super) fn set_node_attention(&mut self, caveat: NodeId, state: Attention) {
        self.graph.write().set_attention(caveat, state);
        self.graph.note(caveat);
    }

    /// Opens `commitment` and records that `because` reopened it.
    pub(super) fn reopen_commitment(&mut self, commitment: NodeId, because: NodeId) {
        self.graph.write().reopen(commitment, because);
        self.graph.note(commitment);
    }

    /// Opens a restored commitment; false when `id` is not a commitment.
    pub(super) fn open_restored_commitment(&mut self, id: NodeId) -> bool {
        self.graph.note(id);
        match self.graph.write().nodes.get_mut(&id) {
            Some(NodeKind::Commitment { open, .. }) => {
                *open = true;
                true
            }
            _ => false,
        }
    }

    /// Removes departing records' nodes, names and every edge that touches
    /// one, passing each removed edge, with the names of its two ends, to
    /// `removed` in graph order.
    pub(super) fn depart_records(
        &mut self,
        ids: &HashMap<NodeId, String>,
        mut removed: impl FnMut(&Edge, &str, &str),
    ) {
        let graph = self.graph.write_any();
        let names = &self.symbols;
        graph.edges.retain(|edge| {
            if ids.contains_key(&edge.from) || ids.contains_key(&edge.to) {
                let name = |id| {
                    names
                        .name_of(id)
                        .expect("a removed edge relates named records")
                };
                removed(edge, name(edge.from), name(edge.to));
                false
            } else {
                true
            }
        });
        for id in ids.keys() {
            graph.nodes.remove(id);
            graph.origins.remove(id);
        }
        for record in ids.values() {
            self.symbols.remove(record);
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

    fn names(entries: &[(NodeId, &str)]) -> NodeNames {
        let mut names = NodeNames::default();
        for &(id, name) in entries {
            names.set(id, Some(Arc::from(name)));
        }
        names
    }

    fn chunk() -> NodeId {
        NodeId::try_from(NAME_CHUNK).unwrap()
    }

    #[test]
    fn names_by_node_read_back_across_chunks() {
        let last = chunk();
        let mut names = names(&[(1, "a"), (last - 1, "b"), (last, "c")]);
        assert_eq!(names.get(1), Some("a"));
        assert_eq!(names.get(last - 1), Some("b"));
        assert_eq!(names.get(last), Some("c"));
        assert_eq!(names.get(2), None);
        assert_eq!(names.get(last * 9), None);
        names.set(last, None);
        assert_eq!(names.get(last), None);
        // The chunk that named only `last` is dropped, and clearing where no
        // chunk is adds none.
        assert_eq!(names.chunks.len(), 1);
        names.set(last * 9, None);
        assert_eq!(names.chunks.len(), 1);
        assert_eq!(names.get(1), Some("a"));
        assert_eq!(names.get(last - 1), Some("b"));
    }

    #[test]
    fn names_by_node_hold_only_chunks_that_name_a_record() {
        // A windowed session: a few declared names, then a stream of records
        // that each depart a little later. Ids keep counting up; the chunks
        // held stay the few that name a live record.
        let mut names = names(&[(1, "declared"), (2, "also declared")]);
        let window = 8;
        let issued = chunk() * 100;
        for id in 3..issued {
            names.set(id, Some(Arc::from(format!("r{id}").as_str())));
            if id >= 3 + window {
                names.set(id - window, None);
            }
            assert!(
                names.chunks.len() <= 3,
                "{} chunks at id {id}",
                names.chunks.len()
            );
        }
        assert_eq!(names.len(), 2 + usize::try_from(window).unwrap());
        assert_eq!(names.get(1), Some("declared"));
        assert_eq!(
            names.get(issued - 1).map(str::to_string),
            Some(format!("r{}", issued - 1))
        );
        assert_eq!(names.get(issued - 1 - window), None);
    }

    #[test]
    fn clearing_the_last_name_in_a_shared_chunk_drops_it_without_a_copy() {
        let last = chunk();
        let before = names(&[(1, "a"), (last, "b")]);
        let mut after = before.clone();
        after.set(last, None);
        assert!(!after.chunks.contains_key(&1));
        assert_eq!(before.get(last), Some("b"));
        assert!(Arc::ptr_eq(&before.chunks[&0], &after.chunks[&0]));
        assert!(after.kept_in(&before));
        assert!(!before.kept_in(&after));
    }

    #[test]
    fn a_copy_of_the_names_by_node_shares_every_chunk_it_does_not_write() {
        let last = chunk();
        let before = names(&[(1, "a"), (last, "b")]);
        let mut after = before.clone();
        after.set(last + 1, Some(Arc::from("c")));
        assert!(Arc::ptr_eq(&before.chunks[&0], &after.chunks[&0]));
        assert!(!Arc::ptr_eq(&before.chunks[&1], &after.chunks[&1]));
        assert_eq!(before.get(last + 1), None);
        assert_eq!(after.get(last + 1), Some("c"));
    }

    #[test]
    fn names_are_kept_when_only_added_to() {
        let last = chunk();
        let before = names(&[(1, "a"), (last, "b")]);
        let mut added = before.clone();
        added.set(last + 1, Some(Arc::from("c")));
        added.set(last * 3, Some(Arc::from("d")));
        assert!(before.kept_in(&before.clone()));
        assert!(before.kept_in(&added));
        assert!(!added.kept_in(&before));
        // The same names, held apart, are still kept.
        assert!(before.kept_in(&names(&[(1, "a"), (last, "b")])));

        let mut removed = added.clone();
        removed.set(1, None);
        assert!(!before.kept_in(&removed));
        let mut renamed = added.clone();
        renamed.set(last, Some(Arc::from("e")));
        assert!(!before.kept_in(&renamed));
        assert!(!before.kept_in(&NodeNames::default()));
        assert!(NodeNames::default().kept_in(&before));
    }

    #[test]
    fn a_name_table_keeps_both_ways_in_step() {
        let mut table = NameTable::new(HashMap::from_iter([
            (String::from("a"), 1),
            (String::from("b"), 2),
        ]));
        let before = table.clone();
        table.insert(String::from("c"), 3);
        // A name given again leaves the node it named without one.
        table.insert(String::from("a"), 4);
        table.remove("b");
        table.remove("unknown");
        table.check();
        assert_eq!(table.get("a"), Some(&4));
        assert_eq!(table.name_of(1), None);
        assert_eq!(table.name_of(2), None);
        assert_eq!(table.name_of(3), Some("c"));
        assert_eq!(table.name_of(4), Some("a"));
        // The copy taken before is untouched, and the names it held are gone.
        before.check();
        assert_eq!(before.name_of(1), Some("a"));
        assert!(!before.kept_in(&table));
        let mut grown = before.clone();
        grown.insert(String::from("d"), 5);
        assert!(before.kept_in(&grown));
        assert!(format!("{table:?}") == format!("{:?}", table.by_name));
    }

    fn records(entries: &[(&str, u32)]) -> Records<u32> {
        Records::new(
            entries
                .iter()
                .map(|&(name, value)| (name.to_string(), value))
                .collect(),
        )
    }

    fn written(records: &Records<u32>) -> Option<Vec<&str>> {
        records
            .written()
            .map(|names| names.iter().map(String::as_str).collect())
    }

    #[test]
    fn records_log_the_names_an_event_writes() {
        let before = records(&[("a", 1), ("b", 2), ("c", 3)]);
        let mut records = before.clone();
        assert!(Arc::ptr_eq(&records, &before));
        // Outside an event the log does not say which records changed.
        assert_eq!(written(&records), None);
        records.begin();
        assert_eq!(written(&records), Some(vec![]));
        assert!(records.unchanged_since(&before));
        // Looking for a record that is not there writes nothing.
        assert_eq!(records.get_mut("x"), None);
        assert!(Arc::ptr_eq(&records, &before));
        *records.get_mut("a").unwrap() = 10;
        records.insert(String::from("d"), 4);
        records.remove("b");
        *records.entry(String::from("e")).or_default() += 5;
        assert_eq!(written(&records), Some(vec!["a", "b", "d", "e"]));
        assert!(!records.unchanged_since(&before));
        // The copy taken before the event is untouched.
        assert_eq!(before.get("a"), Some(&1));
        assert_eq!(before.get("b"), Some(&2));
        assert_eq!(before.get("d"), None);
        // A write that does not say which record ends the log.
        records.make_mut().insert(String::from("f"), 6);
        assert_eq!(written(&records), None);
        assert!(!records.unchanged_since(&before));
    }

    #[test]
    fn records_written_back_to_what_they_were_are_unchanged() {
        let before = records(&[("a", 1)]);
        let mut records = before.clone();
        records.begin();
        *records.get_mut("a").unwrap() = 2;
        *records.get_mut("a").unwrap() = 1;
        records.insert(String::from("b"), 2);
        records.remove("b");
        assert!(!Arc::ptr_eq(&records, &before));
        assert_eq!(written(&records), Some(vec!["a", "b"]));
        assert!(records.unchanged_since(&before));
        records.end();
        assert_eq!(written(&records), None);
        assert!(records.unchanged_since(&before));
    }

    #[test]
    fn an_event_ends_its_log_once_it_has_read_what_it_changed() {
        let mut session = ReactiveSession::from_source(
            "claim ok;\nevidence e from \"tool\";\nevent see;\non see reveal e supports ok;\n",
        )
        .unwrap();
        session.dispatch_json("see", "{}").unwrap();
        assert_eq!(session.graph.written.only(), None);
        assert_eq!(session.reading_streams.written(), None);
        assert_eq!(session.decision_series.written(), None);
        assert_eq!(session.commitment_bases.written(), None);
        assert_eq!(session.commitment_grounds.written(), None);
        assert_eq!(session.observation_qualifications.written(), None);
        assert_eq!(session.examination_qualifications.written(), None);
        assert_eq!(session.reopening_qualifications.written(), None);
    }

    #[test]
    fn the_graph_logs_the_nodes_an_event_writes_and_the_edges_it_adds() {
        let before = ReactiveSession::from_source(
            "claim ok;\ncaveat c consequence low;\nevidence e from \"tool\";\nevent idle;\n",
        )
        .unwrap();
        let (ok, c, e) = (
            before.symbols["ok"],
            before.symbols["c"],
            before.symbols["e"],
        );
        let mut session = before.clone();
        assert_eq!(session.graph.written.only(), None);
        session.begin_write_log();
        assert!(session.graph.unchanged_since(&before.graph));
        assert_eq!(
            session.graph.added_since(&before.graph),
            Some((&[][..], BTreeSet::new()))
        );
        session.set_node_attention(c, Attention::Examined);
        session.relate(e, Relation::Supports, ok);
        assert_eq!(session.graph.written.only(), Some(&BTreeSet::from([c])));
        assert!(!session.graph.unchanged_since(&before.graph));
        let supports = Edge {
            from: e,
            to: ok,
            relation: Relation::Supports,
            origin: None,
        };
        assert_eq!(
            session.graph.added_since(&before.graph),
            Some((std::slice::from_ref(&supports), BTreeSet::from([c])))
        );
        // The copy taken before the event is untouched.
        assert!(!before.graph.edges.contains(&supports));
        // A record departing removes its node and edges: the log no longer
        // says what changed, and the graph was not only added to.
        session.depart_records(&HashMap::from_iter([(e, String::from("e"))]), |_, _, _| {});
        assert_eq!(session.graph.written.only(), None);
        assert_eq!(session.graph.added_since(&before.graph), None);
        assert!(!session.graph.unchanged_since(&before.graph));
        session.end_write_log();
        assert_eq!(session.graph.written.only(), None);
    }

    #[test]
    fn records_print_what_their_arc_printed() {
        let records = records(&[("a", 1)]);
        let arc = Arc::new(BTreeMap::from([(String::from("a"), 1)]));
        assert_eq!(format!("{records:?}"), format!("{arc:?}"));
        assert_eq!(format!("{records:#?}"), format!("{arc:#?}"));
    }

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
