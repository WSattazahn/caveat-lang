//! One write path for the containers that reads will index: the graph, the
//! names, the observation ledger, the withdrawals and the take-backs. Each is
//! sealed: anything can read it, the ledger through its own methods and the
//! others (`Sealed`) as the `Arc` they hold, and only this module can make
//! one or write into one, so no write can skip what its session methods
//! keep up to date. Each method makes the same change, in the same
//! container, that its callers made before it existed. Outside this module a
//! sealed container can still be copied, so code that holds a session could
//! swap in a copy of one whole, from this session or another; nothing does,
//! and an index kept inside its container, as the names keep theirs by node
//! (`NameTable`), moves with it.
//!
//! The graph and the session's keyed records (`Logged`) also log what an
//! event writes to them, from the start of the event until `Changes` has
//! read what it changed, so that check reads only what the event wrote.
use super::relations::{Departed, Link, List, Relations};
use super::*;
use crate::idmap::IdMap;
use crate::namemap::NameMap;
use crate::ordmap::OrdMap;
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

/// The observation ledger: the evidence observed, in the order it was first
/// observed, once a neutral reveal has started one. It is sealed as the other
/// containers are: anything can read it, and only the session methods below
/// write it.
///
/// A record takes the next number when it is observed and keeps it, and a
/// record that departs leaves its number unused, so the records in the order
/// of their numbers are the ledger in order. A copy shares both maps whole,
/// and a write copies the blocks on the way to one slot of each. Whether the
/// ledger holds a record, where the record stands and whether an event only
/// added to the ledger are each read without walking it.
#[derive(Clone, Default)]
pub(super) struct Observations {
    /// The records by number: the ledger in order.
    order: IdMap<NodeId>,
    /// Each record's number.
    numbers: IdMap<u64>,
    /// The number the next record takes.
    next: u64,
}

impl Observations {
    /// Whether no ledger has been started: a ledger holds a record from its
    /// first neutral reveal on, unless every record it held has departed.
    pub(super) fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Whether the ledger holds `id`.
    pub(super) fn holds(&self, id: NodeId) -> bool {
        self.numbers.contains_key(&id)
    }

    /// Where `id` stands in the ledger: a record observed before another has
    /// the lower number.
    pub(super) fn rank(&self, id: NodeId) -> Option<u64> {
        self.numbers.get(&id).copied()
    }

    /// The records, in the order they were observed.
    pub(super) fn iter(&self) -> impl ExactSizeIterator<Item = NodeId> + '_ {
        self.order.iter().map(|(_, id)| *id)
    }

    /// Whether `later`, this ledger after an event, begins with it: it holds
    /// each record of this ledger at its number, and no other record before
    /// the last of them. The numbers an event gives are past every number
    /// given before it, so this is whether the event only added records.
    /// What the two share is not read.
    pub(super) fn begins(&self, later: &Self) -> bool {
        self.order.begins(&later.order)
    }

    fn push(&mut self, id: NodeId) {
        debug_assert!(!self.holds(id), "a record is observed once");
        self.order.insert(self.next, id);
        self.numbers.insert(id, self.next);
        self.next += 1;
    }

    fn remove(&mut self, id: NodeId) {
        if let Some(number) = self.numbers.remove(&id) {
            self.order.remove(&number);
        }
    }

    /// The two maps are each other turned around, exactly.
    #[cfg(any(debug_assertions, test))]
    fn check(&self) {
        debug_assert_eq!(self.order.len(), self.numbers.len());
        for (number, id) in &self.order {
            debug_assert!(number < self.next, "record {id} has number {number}");
            debug_assert_eq!(self.numbers.get(id), Some(&number), "the number of {id}");
        }
    }
}

/// Two ledgers are equal when they hold the same records in the same order,
/// whatever numbers the records have. A ledger and a copy of it that was not
/// written since are not read.
impl PartialEq for Observations {
    fn eq(&self, other: &Self) -> bool {
        self.order.shares_all(&other.order)
            || (self.order.len() == other.order.len() && self.iter().eq(other.iter()))
    }
}

/// Prints the records in ledger order, each by its node.
impl fmt::Debug for Observations {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_list().entries(self.iter()).finish()
    }
}

/// A session's sealed containers as its program declares them: the graph and
/// its names, and no observations, withdrawals or take-backs yet.
pub(super) struct Declared {
    pub(super) graph: SealedGraph,
    pub(super) symbols: NameTable,
    pub(super) observations: Observations,
    pub(super) withdrawals: Sealed<Vec<Withdrawal>>,
    pub(super) taken_back: TakeBacks,
}

/// The take-backs, in the order they were made, with how many name each
/// occurrence and caveat, so that a read looks the two names up where it
/// read every take-back. The counts follow from the list and are written
/// only where it is; a copy shares both, and a write copies only the blocks
/// of the counts on the way down to its occurrence. It reads like the list's
/// `Arc`.
pub(super) struct TakeBacks {
    list: Arc<Vec<TakeBack>>,
    /// By occurrence name: each caveat a take-back names with it, with how
    /// many do. An occurrence is taken back from a handful of caveats at
    /// most, so a short list.
    counts: Named<Vec<(Arc<str>, usize)>>,
}

impl TakeBacks {
    fn new(list: Vec<TakeBack>) -> Self {
        let mut counts = Named::default();
        for record in &list {
            Self::count_in(&mut counts, record);
        }
        Self {
            list: Arc::new(list),
            counts,
        }
    }

    fn count_in(counts: &mut Named<Vec<(Arc<str>, usize)>>, record: &TakeBack) {
        if let Some(caveats) = counts.0.get_mut(record.evidence.as_str()) {
            match caveats
                .iter_mut()
                .find(|(caveat, _)| **caveat == *record.caveat)
            {
                Some((_, count)) => *count += 1,
                None => caveats.push((Arc::from(record.caveat.as_str()), 1)),
            }
        } else {
            counts.0.insert(
                Arc::from(record.evidence.as_str()),
                vec![(Arc::from(record.caveat.as_str()), 1)],
            );
        }
    }

    fn count_out(counts: &mut Named<Vec<(Arc<str>, usize)>>, record: &TakeBack) {
        let caveats = counts
            .0
            .get_mut(record.evidence.as_str())
            .expect("a take-back's occurrence is counted");
        let at = caveats
            .iter()
            .position(|(caveat, _)| **caveat == *record.caveat)
            .expect("a take-back's caveat is counted");
        caveats[at].1 -= 1;
        if caveats[at].1 == 0 {
            caveats.swap_remove(at);
            if caveats.is_empty() {
                counts.0.remove(record.evidence.as_str());
            }
        }
    }

    fn push(&mut self, record: TakeBack) {
        Self::count_in(&mut self.counts, &record);
        Arc::make_mut(&mut self.list).push(record);
    }

    fn retain(&mut self, mut keep: impl FnMut(&TakeBack) -> bool) {
        let counts = &mut self.counts;
        Arc::make_mut(&mut self.list).retain(|record| {
            let kept = keep(record);
            if !kept {
                Self::count_out(counts, record);
            }
            kept
        });
    }

    /// How many take-backs name `caveat` and `occurrence`. A debug build
    /// counts them in the list too.
    pub(super) fn count(&self, caveat: &str, occurrence: &str) -> usize {
        let count = self
            .counts
            .get(occurrence)
            .and_then(|caveats| caveats.iter().find(|(held, _)| **held == *caveat))
            .map_or(0, |(_, count)| *count);
        debug_assert_eq!(
            count,
            self.list
                .iter()
                .filter(|record| record.caveat == caveat && record.evidence == occurrence)
                .count(),
            "take-back counts by name"
        );
        count
    }
}

impl Deref for TakeBacks {
    type Target = Arc<Vec<TakeBack>>;

    fn deref(&self) -> &Arc<Vec<TakeBack>> {
        &self.list
    }
}

impl Clone for TakeBacks {
    fn clone(&self) -> Self {
        Self {
            list: Arc::clone(&self.list),
            counts: self.counts.clone(),
        }
    }
}

/// Prints what the list's `Arc` printed, so a session's debug output is
/// unchanged.
impl fmt::Debug for TakeBacks {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.list.fmt(formatter)
    }
}

/// The graph and names a program declares, sealed for the session it loads.
pub(super) fn seal_declared(graph: EpistemicGraph, symbols: HashMap<String, NodeId>) -> Declared {
    Declared {
        graph: SealedGraph::new(graph),
        symbols: NameTable::new(symbols),
        observations: Observations::default(),
        withdrawals: Sealed::new(Vec::new()),
        taken_back: TakeBacks::new(Vec::new()),
    }
}

/// The session's graph, sealed like the other containers, with the nodes an
/// event wrote and each record's relations (`Relations`). It reads as the
/// graph's `Arc`. Only the session methods below write it: each says which
/// node it adds or changes, and until records depart an event only adds
/// edges. So after an event `Changes` compares only the nodes it wrote and
/// the edges past the old count, where it compared every node and edge
/// before; and each edge added or removed is added to or removed from the
/// relation lists, which reads look records up in instead of scanning every
/// edge.
pub(super) struct SealedGraph {
    graph: Arc<EpistemicGraph>,
    relations: Arc<Relations>,
    written: Written<NodeId>,
}

impl SealedGraph {
    fn new(graph: EpistemicGraph) -> Self {
        Self {
            relations: Arc::new(Relations::new(&graph.edges.iter().collect::<Vec<_>>())),
            graph: Arc::new(graph),
            written: Written::Any,
        }
    }

    /// The graph, to add edges to or to write the nodes the caller notes.
    /// The caller keeps the edges it adds with `added`.
    fn write(&mut self) -> &mut EpistemicGraph {
        Arc::make_mut(&mut self.graph)
    }

    /// Keeps, in the relation lists, the edges added since they last held
    /// every edge.
    fn added(&mut self) {
        let held = self.relations.held();
        let edges = self.graph.edges.iter_from(held).collect::<Vec<_>>();
        if !edges.is_empty() {
            Arc::make_mut(&mut self.relations).add(&edges);
        }
    }

    /// Removes the nodes of `ids` and every edge that touches one, passing
    /// each removed edge to `removed` in graph order. The log no longer says
    /// what changed.
    fn depart(&mut self, ids: &HashMap<NodeId, String>, mut removed: impl FnMut(&Edge)) {
        self.written = Written::Any;
        let graph = Arc::make_mut(&mut self.graph);
        let departing = |id: NodeId| ids.contains_key(&id);
        let mut departed = Departed::default();
        graph.edges.retain(|edge| {
            if departing(edge.from) || departing(edge.to) {
                removed(edge);
                departed.note(edge);
                false
            } else {
                true
            }
        });
        if departed.any() {
            Arc::make_mut(&mut self.relations).depart(departed, departing);
        }
        for id in ids.keys() {
            graph.nodes.remove(id);
            graph.origins.remove(id);
        }
    }

    /// `id`'s `list` of relations, in graph order.
    fn list(&self, id: NodeId, list: List) -> &[Link] {
        debug_assert_eq!(
            self.relations.held(),
            self.graph.edges.len(),
            "the relation lists hold every edge"
        );
        self.relations.list(id, list)
    }

    /// What `id` supports or opposes, in graph order.
    pub(super) fn stance(&self, id: NodeId) -> &[Link] {
        self.list(id, List::Stance)
    }

    /// What commitment `id` retains or relies on, in graph order.
    pub(super) fn basis(&self, id: NodeId) -> &[Link] {
        self.list(id, List::Basis)
    }

    /// What qualifies `id`, in graph order.
    pub(super) fn qualifiers(&self, id: NodeId) -> &[Link] {
        self.list(id, List::Qualifiers)
    }

    /// What reopens commitment `id`, in graph order.
    pub(super) fn reopeners(&self, id: NodeId) -> &[Link] {
        self.list(id, List::Reopeners)
    }

    /// Whether any edge is `qualifies`.
    pub(super) fn qualifies_any(&self) -> bool {
        self.relations.qualifies_any()
    }

    /// Whether an edge `from relation to` is in the graph.
    pub(super) fn related(&self, from: NodeId, relation: Relation, to: NodeId) -> bool {
        match List::place(from, relation, to) {
            Some((list, keeper, other)) => self
                .list(keeper, list)
                .iter()
                .any(|link| link.other == other && link.relation == relation),
            None => self
                .graph
                .edges
                .iter()
                .any(|edge| edge.from == from && edge.to == to && edge.relation == relation),
        }
    }

    /// The relation lists hold exactly this graph's edges.
    #[cfg(any(debug_assertions, test))]
    pub(super) fn check(&self) {
        self.relations
            .check(&self.graph.edges.iter().collect::<Vec<_>>());
    }

    fn note(&mut self, id: NodeId) {
        let size = self.graph.nodes.len() + self.graph.edges.len();
        self.written.note(&id, size);
    }

    /// Whether this graph's nodes and edges are `old`'s, where `old` is the
    /// session this event began from.
    #[inline]
    pub(super) fn unchanged_since(&self, old: &Self) -> bool {
        Arc::ptr_eq(&self.graph, &old.graph) || self.unchanged_written(old)
    }

    /// `unchanged_since` for a graph the event copied: only the nodes it
    /// wrote, and the edges, can differ, when its log says which nodes.
    #[inline(never)]
    fn unchanged_written(&self, old: &Self) -> bool {
        let (new, old) = (&*self.graph, &*old.graph);
        let unchanged = match self.written.only() {
            Some(ids) => {
                new.edges.len() == old.edges.len()
                    && ids.iter().all(|id| new.nodes.get(id) == old.nodes.get(id))
            }
            None => new.edges == old.edges && new.nodes == old.nodes,
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
    pub(super) fn added_since<'a>(
        &'a self,
        old: &Self,
    ) -> Option<(Vec<&'a Edge>, BTreeSet<NodeId>)> {
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
) -> Option<(Vec<&'a Edge>, BTreeSet<NodeId>)> {
    let edges = match written {
        Some(_) if new.edges.len() >= old.edges.len() => {
            new.edges.iter_from(old.edges.len()).collect()
        }
        Some(_) => return None,
        None if new.edges.starts_with(&old.edges) => new.edges.iter_from(old.edges.len()).collect(),
        None => return None,
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
            if old.nodes.keys().any(|id| !new.nodes.contains_key(&id)) {
                return None;
            }
            for id in new.nodes.keys() {
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

/// A copy shares the graph and its relation lists but not the log, as a
/// copy of `Logged` does.
impl Clone for SealedGraph {
    fn clone(&self) -> Self {
        Self {
            graph: Arc::clone(&self.graph),
            relations: Arc::clone(&self.relations),
            written: Written::Any,
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
    by_name: NameMap,
    by_node: NodeNames,
}

/// Two tables are equal when they name the same nodes by the same names; the
/// names by node follow from those.
impl PartialEq for NameTable {
    fn eq(&self, other: &Self) -> bool {
        self.by_name == other.by_name
    }
}

/// Prints the names in name order.
impl fmt::Debug for NameTable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.by_name.fmt(formatter)
    }
}

impl std::ops::Index<&str> for NameTable {
    type Output = NodeId;

    fn index(&self, name: &str) -> &NodeId {
        &self.by_name[name]
    }
}

impl NameTable {
    fn new(by_name: HashMap<String, NodeId>) -> Self {
        let mut names = NameMap::new();
        let mut by_node = NodeNames::default();
        for (name, id) in by_name {
            debug_assert!(by_node.get(id).is_none(), "a node has one name");
            let shared = Arc::<str>::from(name);
            by_node.set(id, Some(Arc::clone(&shared)));
            names.insert(shared, id);
        }
        Self {
            by_name: names,
            by_node,
        }
    }

    /// The node named `name`.
    pub(super) fn get(&self, name: &str) -> Option<&NodeId> {
        self.by_name.get(name)
    }

    pub(super) fn contains_key(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    /// Each name and the node it names, in no order to rely on.
    pub(super) fn iter(&self) -> impl Iterator<Item = (&str, &NodeId)> + '_ {
        self.by_name.iter()
    }

    pub(super) fn keys(&self) -> impl Iterator<Item = &str> + '_ {
        self.by_name.keys()
    }

    /// Whether the two tables share all they hold, so are the same table
    /// without either being read.
    pub(super) fn shares_all(&self, other: &Self) -> bool {
        self.by_name.shares_all(&other.by_name) && self.by_node.shares_all(&other.by_node)
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
        let by_node = &mut self.by_node;
        debug_assert!(
            by_node.get(id).is_none_or(|held| held == name),
            "a node has one name"
        );
        let shared = Arc::<str>::from(name);
        if let Some(previous) = self.by_name.insert(Arc::clone(&shared), id) {
            by_node.set(previous, None);
        }
        by_node.set(id, Some(shared));
    }

    /// Removes `name`, and the name of the node it named.
    fn remove(&mut self, name: &str) {
        if let Some(id) = self.by_name.remove(name) {
            self.by_node.set(id, None);
        }
    }

    /// The names by node are the names map turned around, exactly.
    #[cfg(any(debug_assertions, test))]
    pub(super) fn check(&self) {
        for (name, &id) in self.by_name.iter() {
            debug_assert_eq!(self.by_node.get(id), Some(name), "the name of node {id}");
        }
        debug_assert_eq!(self.by_node.len(), self.by_name.len(), "names by node");
    }
}

/// How many names one chunk of `NodeNames` holds. A write copies its chunk
/// whole: 32 names take 512 bytes, below the 1,024 at which the system
/// allocator tidies its free lists first, which a chunk of 64 cost each event
/// that named a record.
const NAME_CHUNK: usize = 32;
/// `NAME_CHUNK` counted in node ids.
const CHUNK_NODES: NodeId = NAME_CHUNK as NodeId;

type NameChunk = [Option<Arc<str>>; NAME_CHUNK];

/// Each named record's name, by node, in fixed-size chunks keyed by where
/// they sit among node ids. Ids count up and are never reused, and departure
/// removes old records, so a chunk that names nothing is dropped: what the
/// names hold, and what a comparison of them reads, grows with the records
/// named now rather than with every id the session has issued. A copy shares
/// every chunk and every block of the map they sit in, and a write copies
/// only the chunk it changes and the blocks on the way down to it.
#[derive(Clone, Default)]
struct NodeNames {
    chunks: IdMap<NameChunk>,
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
            match self.chunks.get_mut(&at) {
                Some(chunk) => chunk[slot] = Some(name),
                None => {
                    let mut chunk: NameChunk = std::array::from_fn(|_| None);
                    chunk[slot] = Some(name);
                    self.chunks.insert(at, chunk);
                }
            }
            return;
        }
        let Some(chunk) = self.chunks.get(&at) else {
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
        } else if let Some(chunk) = self.chunks.get_mut(&at) {
            chunk[slot] = None;
        }
    }

    /// Whether the two are one tree, so hold the same names without either
    /// being read.
    fn shares_all(&self, other: &Self) -> bool {
        self.chunks.shares_all(&other.chunks)
    }

    /// Whether every node named here has the same name in `other`. Each
    /// chunk held names a node, so one `other` lacks is not kept. The chunks
    /// and blocks the two share are not read.
    fn kept_in(&self, other: &Self) -> bool {
        self.chunks.within(&other.chunks, |chunk, kept| {
            chunk.iter().zip(kept.iter()).all(|(name, kept)| {
                name.as_ref().is_none_or(|name| {
                    kept.as_ref()
                        .is_some_and(|kept| Arc::ptr_eq(name, kept) || name == kept)
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
pub(super) struct Logged<M> {
    map: Arc<M>,
    written: Written<String>,
}

/// Records in the standard ordered map, which the first write of an event
/// copies whole. The views read these maps too.
pub(super) type Records<V> = Logged<BTreeMap<String, V>>;

/// Records in a map that the first write of an event copies only on the way
/// down to its record (`Named`).
pub(super) type NamedRecords<V> = Logged<Named<V>>;

/// What `Logged` asks of the map it keeps its records in.
pub(super) trait RecordMap: Clone + Default {
    type Value;

    /// How many records the map holds.
    fn count(&self) -> usize;
    fn record(&self, name: &str) -> Option<&Self::Value>;
    fn record_mut(&mut self, name: &str) -> Option<&mut Self::Value>;
    /// Puts `value` at `name`, and says whether a record was there.
    fn put(&mut self, name: String, value: Self::Value) -> bool;
    /// Takes the record `name` out, and says whether there was one.
    fn take(&mut self, name: &str) -> bool;
    /// Every record with its name, in the order of the names.
    fn records(&self) -> impl Iterator<Item = (&str, &Self::Value)>;
}

impl<V: Clone> RecordMap for BTreeMap<String, V> {
    type Value = V;

    fn count(&self) -> usize {
        self.len()
    }

    fn record(&self, name: &str) -> Option<&V> {
        self.get(name)
    }

    fn record_mut(&mut self, name: &str) -> Option<&mut V> {
        self.get_mut(name)
    }

    fn put(&mut self, name: String, value: V) -> bool {
        self.insert(name, value).is_some()
    }

    fn take(&mut self, name: &str) -> bool {
        self.remove(name).is_some()
    }

    fn records(&self) -> impl Iterator<Item = (&str, &V)> {
        self.iter().map(|(name, value)| (name.as_str(), value))
    }
}

/// Records by name in a map that a copy shares whole and a write copies only
/// on the way down to its record (`OrdMap`), where the standard ordered map
/// is copied name by name and record by record. A name is kept as shared
/// text, so a copied block copies none. It reads as the ordered map it
/// stands in for, with each name as a `&str`.
pub(super) struct Named<V>(OrdMap<Arc<str>, V>);

impl<V> Clone for Named<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V> Default for Named<V> {
    fn default() -> Self {
        Self(OrdMap::new())
    }
}

impl<V> Named<V> {
    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.0.len()
    }

    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(super) fn get(&self, name: &str) -> Option<&V> {
        self.0.get(name)
    }

    /// Every record with its name, in the order of the names.
    pub(super) fn iter(&self) -> impl Iterator<Item = (&str, &V)> {
        self.0.iter().map(|(name, value)| (&**name, value))
    }

    pub(super) fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(|name| &**name)
    }

    pub(super) fn values(&self) -> impl Iterator<Item = &V> {
        self.0.values()
    }
}

impl<V: Clone> Named<V> {
    /// The records in a standard ordered map, for what is handed out whole.
    pub(super) fn to_map(&self) -> BTreeMap<String, V> {
        self.iter()
            .map(|(name, value)| (name.to_string(), value.clone()))
            .collect()
    }
}

impl<V: Clone> FromIterator<(String, V)> for Named<V> {
    fn from_iter<I: IntoIterator<Item = (String, V)>>(records: I) -> Self {
        Self(
            records
                .into_iter()
                .map(|(name, value)| (Arc::from(name), value))
                .collect(),
        )
    }
}

impl<V> std::ops::Index<&str> for Named<V> {
    type Output = V;

    fn index(&self, name: &str) -> &V {
        self.get(name).expect("no record of this name")
    }
}

/// Prints what the ordered map printed, so a session's debug output is
/// unchanged.
impl<V: fmt::Debug> fmt::Debug for Named<V> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<V: Clone> RecordMap for Named<V> {
    type Value = V;

    fn count(&self) -> usize {
        self.0.len()
    }

    fn record(&self, name: &str) -> Option<&V> {
        self.0.get(name)
    }

    fn record_mut(&mut self, name: &str) -> Option<&mut V> {
        self.0.get_mut(name)
    }

    /// A name the map holds already keeps its text. The record replaced is
    /// dropped as the map held it, so one a copy shares is not copied.
    fn put(&mut self, name: String, value: V) -> bool {
        self.0
            .insert_by(
                |held| (**held).cmp(name.as_str()),
                || Arc::from(name.as_str()),
                value,
            )
            .is_some()
    }

    fn take(&mut self, name: &str) -> bool {
        self.0.remove(name).is_some()
    }

    fn records(&self) -> impl Iterator<Item = (&str, &V)> {
        self.iter()
    }
}

/// What an event wrote to one container.
#[derive(Debug)]
enum Written<K> {
    /// Nothing yet. Holds no set, so an event that writes nothing here makes
    /// and drops none.
    Nothing,
    /// These, and no other: everything else is as the event found it.
    Only(BTreeSet<K>),
    /// Anything: written outside an event, by a write that does not say
    /// what, or past the most a log keeps.
    Any,
}

/// The most keys a log keeps: this many, or an eighth of the container's
/// records if that is more. Past it, comparing every record once costs less
/// than logging each write, so the log says any record may have changed.
const LOG_MAX: usize = 32;

/// The fewest records, or nodes and edges, a container holds for an event to
/// log which of them it writes. Comparing a few records costs less than
/// logging the writes to them, so in a smaller container the first write
/// gives up the log.
const LOG_MIN: usize = 8;

/// The fewest this build logs from. Debug builds log every container, so
/// that their check of each log against a full comparison reads every
/// write, whatever the size.
const LOG_FROM: usize = if cfg!(debug_assertions) { 0 } else { LOG_MIN };

/// How `Changes` tells whether a record an event may have written is the
/// one the event began with.
pub(super) trait Same {
    fn same(&self, other: &Self) -> bool;
}

/// A record both sessions share is the same without reading it. Records
/// hold only finite numbers, so each is equal to itself.
impl<T: PartialEq> Same for Arc<T> {
    #[inline]
    fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(self, other) || **self == **other
    }
}

macro_rules! same_by_value {
    ($($kind:ty),*) => {$(
        impl Same for $kind {
            #[inline]
            fn same(&self, other: &Self) -> bool {
                self == other
            }
        }
    )*};
}
same_by_value!(Provenance, CommitmentBasis, DecisionSeries);
#[cfg(test)]
same_by_value!(u32);

/// Whether `new` and `old` hold the same record, or neither holds one.
fn same_record<V: Same>(new: Option<&V>, old: Option<&V>) -> bool {
    match (new, old) {
        (Some(new), Some(old)) => new.same(old),
        (new, old) => new.is_none() && old.is_none(),
    }
}

/// Whether two maps hold the same records by the same names.
fn same_records<M: RecordMap>(new: &M, old: &M) -> bool
where
    M::Value: Same,
{
    new.count() == old.count()
        && new
            .records()
            .zip(old.records())
            .all(|((name, new), (other, old))| name == other && new.same(old))
}

impl<K: Ord> Written<K> {
    /// Notes `key`, written in a container of `size` records, or nodes and
    /// edges.
    #[inline]
    fn note<Q>(&mut self, key: &Q, size: usize)
    where
        K: std::borrow::Borrow<Q>,
        Q: Ord + ToOwned<Owned = K> + ?Sized,
    {
        if !matches!(self, Self::Any) {
            self.log(key, size);
        }
    }

    /// `note` while the log says which keys were written: starts it, adds
    /// `key`, or gives up in a small container or once the log holds the
    /// most it keeps.
    fn log<Q>(&mut self, key: &Q, size: usize)
    where
        K: std::borrow::Borrow<Q>,
        Q: Ord + ToOwned<Owned = K> + ?Sized,
    {
        match self {
            Self::Nothing => *self = Self::start(key, size, LOG_FROM),
            Self::Only(keys) => {
                if !keys.contains(key) {
                    if keys.len() >= LOG_MAX.max(size / 8) {
                        *self = Self::Any;
                    } else {
                        keys.insert(key.to_owned());
                    }
                }
            }
            Self::Any => {}
        }
    }

    /// The log after the first write, to `key` in a container of `size`
    /// records: it names `key` from `from` records, and below that gives up.
    #[inline]
    fn start<Q>(key: &Q, size: usize, from: usize) -> Self
    where
        K: std::borrow::Borrow<Q>,
        Q: Ord + ToOwned<Owned = K> + ?Sized,
    {
        if size < from {
            Self::Any
        } else {
            Self::Only(BTreeSet::from([key.to_owned()]))
        }
    }

    /// The keys written, if the log says which, with `none` for a log that
    /// has none yet; `None` if any may have changed.
    fn keys<'a>(&'a self, none: &'a BTreeSet<K>) -> Option<&'a BTreeSet<K>> {
        match self {
            Self::Nothing => Some(none),
            Self::Only(keys) => Some(keys),
            Self::Any => None,
        }
    }
}

impl Written<NodeId> {
    fn only(&self) -> Option<&BTreeSet<NodeId>> {
        static NONE: BTreeSet<NodeId> = BTreeSet::new();
        self.keys(&NONE)
    }
}

impl Written<String> {
    fn only(&self) -> Option<&BTreeSet<String>> {
        static NONE: BTreeSet<String> = BTreeSet::new();
        self.keys(&NONE)
    }
}

impl<M: RecordMap> Logged<M> {
    pub(super) fn new(map: M) -> Self {
        Self {
            map: Arc::new(map),
            written: Written::Any,
        }
    }

    /// Starts the log of an event's writes: none yet.
    fn begin(&mut self) {
        self.written = Written::Nothing;
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
        self.written.note(name, self.map.count());
    }

    /// Puts `value` at `name`, and says whether a record was there.
    pub(super) fn insert(&mut self, name: String, value: M::Value) -> bool {
        self.note(&name);
        Arc::make_mut(&mut self.map).put(name, value)
    }

    /// The record `name`, to write, if there is one. Copies first what the
    /// map copies to be written, when another session shares it.
    pub(super) fn get_mut(&mut self, name: &str) -> Option<&mut M::Value> {
        let records = self.map.count();
        let record = Arc::make_mut(&mut self.map).record_mut(name)?;
        self.written.note(name, records);
        Some(record)
    }

    /// Takes the record `name` out, and says whether there was one.
    pub(super) fn remove(&mut self, name: &str) -> bool {
        self.note(name);
        Arc::make_mut(&mut self.map).take(name)
    }
}

impl<V: Clone> NamedRecords<V> {
    /// Writes `value` over the record `name`, if there is one, and says
    /// whether there is. The record it replaces is not read, so a map that
    /// shares it with another session copies none of it, where `get_mut`
    /// copies the record to hand it out.
    pub(super) fn replace(&mut self, name: &str, value: V) -> bool {
        let records = self.map.count();
        let replaced = Arc::make_mut(&mut self.map).0.replace(name, value);
        if replaced {
            self.written.note(name, records);
        }
        replaced
    }
}

impl<V: Clone> Records<V> {
    /// The whole map, to write any of it: the log no longer says which
    /// records changed.
    pub(super) fn make_mut(&mut self) -> &mut BTreeMap<String, V> {
        self.written = Written::Any;
        Arc::make_mut(&mut self.map)
    }
}

impl<M: RecordMap> Logged<M>
where
    M::Value: Same,
{
    /// Whether these are the records `old` holds, where `old` is the session
    /// this event began from: only the records it wrote can differ, when its
    /// log says which.
    #[inline]
    pub(super) fn unchanged_since(&self, old: &Self) -> bool {
        Arc::ptr_eq(&self.map, &old.map) || self.unchanged_written(old)
    }

    /// `unchanged_since` for records the event copied.
    #[inline(never)]
    fn unchanged_written(&self, old: &Self) -> bool {
        let unchanged = match self.written.only() {
            Some(names) => names
                .iter()
                .all(|name| same_record(self.map.record(name), old.map.record(name))),
            None => same_records(&*self.map, &*old.map),
        };
        debug_assert_eq!(
            unchanged,
            same_records(&*self.map, &*old.map),
            "an event wrote a record its log does not name"
        );
        unchanged
    }
}

impl<M> Deref for Logged<M> {
    type Target = Arc<M>;

    fn deref(&self) -> &Arc<M> {
        &self.map
    }
}

/// A copy shares the records but not the log: nothing says what was
/// written to it, so any record may have changed.
impl<M> Clone for Logged<M> {
    fn clone(&self) -> Self {
        Self {
            map: Arc::clone(&self.map),
            written: Written::Any,
        }
    }
}

impl<M: RecordMap> Default for Logged<M> {
    fn default() -> Self {
        Self::new(M::default())
    }
}

/// Prints what the map's `Arc` printed, so a session's debug output is
/// unchanged.
impl<M: fmt::Debug> fmt::Debug for Logged<M> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.map.fmt(formatter)
    }
}

impl ReactiveSession {
    /// Starts the log of the records this event writes.
    pub(super) fn begin_write_log(&mut self) {
        self.graph.written = Written::Nothing;
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
        self.graph.added();
        self.graph.note(id);
        self.symbols.insert(name, id);
        id
    }

    pub(super) fn relate(&mut self, from: NodeId, relation: Relation, to: NodeId) {
        self.graph.write().relate(from, relation, to);
        self.graph.added();
    }

    /// `relate` for each of `relations` in order, with the lists keeping
    /// them all at once, as a restore adds every relation its save holds.
    pub(super) fn relate_all(&mut self, relations: &[(NodeId, Relation, NodeId)]) {
        // A save that adds no relation leaves the graph the loaded one.
        if relations.is_empty() {
            return;
        }
        let graph = self.graph.write();
        for &(from, relation, to) in relations {
            graph.relate(from, relation, to);
        }
        self.graph.added();
    }

    pub(super) fn set_node_attention(&mut self, caveat: NodeId, state: Attention) {
        self.graph.write().set_attention(caveat, state);
        self.graph.note(caveat);
    }

    /// Opens `commitment` and records that `because` reopened it.
    pub(super) fn reopen_commitment(&mut self, commitment: NodeId, because: NodeId) {
        self.graph.write().reopen(commitment, because);
        self.graph.added();
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
        let names = &self.symbols;
        self.graph.depart(ids, |edge| {
            let name = |id| {
                names
                    .name_of(id)
                    .expect("a removed edge relates named records")
            };
            removed(edge, name(edge.from), name(edge.to));
        });
        for record in ids.values() {
            self.symbols.remove(record);
        }
    }

    /// Replaces the observation ledger with `records` in their order, each
    /// given once: its first materialization, or a restore.
    pub(super) fn set_observations(&mut self, records: impl IntoIterator<Item = NodeId>) {
        let mut ledger = Observations::default();
        for id in records {
            ledger.push(id);
        }
        self.observations = ledger;
    }

    /// Adds `evidence`, which the ledger does not hold, at its end.
    pub(super) fn push_observation(&mut self, evidence: NodeId) {
        self.observations.push(evidence);
    }

    /// Takes each of `records` that the ledger holds out of it.
    pub(super) fn depart_observations(&mut self, records: impl IntoIterator<Item = NodeId>) {
        for id in records {
            self.observations.remove(id);
        }
    }

    /// The ledger's records by name, in ledger order.
    pub(super) fn observation_names(&self) -> Vec<String> {
        self.observations
            .iter()
            .map(|id| {
                self.symbols
                    .name_of(id)
                    .expect("an observed record is named")
                    .to_string()
            })
            .collect()
    }

    /// Whether the ledger holds the record named `name`.
    pub(super) fn in_ledger(&self, name: &str) -> bool {
        self.symbols
            .get(name)
            .is_some_and(|id| self.observations.holds(*id))
    }

    /// The ledger's two maps agree, and each of its records is named
    /// evidence: what lets it keep records by node where saves, and the
    /// rules that read it, name them.
    #[cfg(any(debug_assertions, test))]
    pub(super) fn check_observations(&self) {
        self.observations.check();
        for id in self.observations.iter() {
            debug_assert!(
                self.symbols.name_of(id).is_some(),
                "observed record {id} has no name"
            );
            debug_assert!(
                matches!(self.graph.nodes.get(&id), Some(NodeKind::Evidence { .. })),
                "observed record {id} is not evidence"
            );
        }
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
        self.taken_back = TakeBacks::new(records);
    }

    pub(super) fn push_take_back(&mut self, record: TakeBack) {
        self.taken_back.push(record);
    }

    pub(super) fn retain_taken_back(&mut self, keep: impl FnMut(&TakeBack) -> bool) {
        self.taken_back.retain(keep);
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
        assert!(std::ptr::eq(&before.chunks[&0], &after.chunks[&0]));
        assert!(after.kept_in(&before));
        assert!(!before.kept_in(&after));
    }

    #[test]
    fn a_copy_of_the_names_by_node_shares_every_chunk_it_does_not_write() {
        let last = chunk();
        let before = names(&[(1, "a"), (last, "b")]);
        let mut after = before.clone();
        after.set(last + 1, Some(Arc::from("c")));
        assert!(std::ptr::eq(&before.chunks[&0], &after.chunks[&0]));
        assert!(!std::ptr::eq(&before.chunks[&1], &after.chunks[&1]));
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

    /// `records` and `LOG_MIN` more, named `pad0`, `pad1` and so on, so
    /// that a release build logs the writes to them too.
    fn logged(entries: &[(&str, u32)]) -> Records<u32> {
        let mut map: BTreeMap<String, u32> = entries
            .iter()
            .map(|&(name, value)| (name.to_string(), value))
            .collect();
        map.extend((0..LOG_MIN).map(|n| (format!("pad{n}"), 0)));
        Records::new(map)
    }

    /// The session of `source` with `LOG_MIN` more claims, so that a
    /// release build logs the writes to its graph too.
    fn logged_program(source: &str) -> ReactiveSession {
        let claims: String = (0..LOG_MIN).map(|n| format!("claim pad{n};\n")).collect();
        ReactiveSession::from_source(&format!("{claims}{source}")).unwrap()
    }

    fn written(records: &Records<u32>) -> Option<Vec<&str>> {
        records
            .written()
            .map(|names| names.iter().map(String::as_str).collect())
    }

    #[test]
    fn records_log_the_names_an_event_writes() {
        let before = logged(&[("a", 1), ("b", 2), ("c", 3)]);
        let mut records = before.clone();
        assert!(Arc::ptr_eq(&records, &before));
        // Outside an event the log does not say which records changed.
        assert_eq!(written(&records), None);
        records.begin();
        assert_eq!(written(&records), Some(vec![]));
        assert!(records.unchanged_since(&before));
        // Looking for a record that is not there writes nothing.
        assert_eq!(records.get_mut("x"), None);
        assert_eq!(written(&records), Some(vec![]));
        assert!(records.unchanged_since(&before));
        *records.get_mut("a").unwrap() = 10;
        records.insert(String::from("d"), 4);
        records.remove("b");
        records.insert(String::from("e"), 5);
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
        let before = logged(&[("a", 1)]);
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
    fn a_record_an_event_adds_alone_is_a_change() {
        // A release build compares a few records whole, and more by the
        // names the log holds.
        let few = Records::new(BTreeMap::from([(String::from("a"), 1)]));
        for before in [few, logged(&[("a", 1)])] {
            let mut records = before.clone();
            records.begin();
            records.insert(String::from("b"), 2);
            assert!(!records.unchanged_since(&before));
        }
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
        let before = logged_program(
            "claim ok;\ncaveat c consequence low;\nevidence e from \"tool\";\nevent idle;\n",
        );
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
            Some((Vec::new(), BTreeSet::new()))
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
            Some((vec![&supports], BTreeSet::from([c])))
        );
        // The copy taken before the event is untouched.
        assert!(!before.graph.edges.iter().any(|edge| *edge == supports));
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
    fn a_log_with_nothing_written_holds_no_set() {
        let before = logged(&[("a", 1)]);
        let mut records = before.clone();
        records.begin();
        assert!(matches!(records.written, Written::Nothing));
        assert_eq!(written(&records), Some(vec![]));
        assert!(records.unchanged_since(&before));
        // Looking for a record that is not there logs nothing.
        assert_eq!(records.get_mut("x"), None);
        assert!(matches!(records.written, Written::Nothing));
        *records.get_mut("a").unwrap() = 2;
        assert_eq!(written(&records), Some(vec!["a"]));
        records.end();
        assert!(matches!(records.written, Written::Any));
    }

    #[test]
    fn a_log_past_32_names_says_any_record_may_have_changed() {
        let before = Records::new(
            (0..100u32)
                .map(|n| (format!("r{n:03}"), n))
                .collect::<BTreeMap<_, _>>(),
        );
        let mut records = before.clone();
        records.begin();
        let first: Vec<String> = (0..32).map(|n| format!("r{n:03}")).collect();
        for name in &first {
            *records.get_mut(name).unwrap() += 100;
        }
        assert_eq!(
            records
                .written()
                .map(|names| names.iter().cloned().collect::<Vec<_>>()),
            Some(first)
        );
        // A name logged already, or one the records do not hold, adds nothing.
        *records.get_mut("r000").unwrap() += 1;
        assert_eq!(records.get_mut("absent"), None);
        assert_eq!(records.written().map(BTreeSet::len), Some(32));
        // One name more and the log gives up: every record is compared.
        *records.get_mut("r032").unwrap() += 1;
        assert_eq!(records.written(), None);
        assert!(!records.unchanged_since(&before));
        assert_eq!(before.get("r032"), Some(&32));
        assert_eq!(records.get("r032"), Some(&33));
    }

    #[test]
    fn a_log_of_many_records_holds_an_eighth_of_them() {
        let mut records = Records::new(
            (0..1000u32)
                .map(|n| (format!("r{n:04}"), n))
                .collect::<BTreeMap<_, _>>(),
        );
        records.begin();
        for n in 0..125 {
            *records.get_mut(&format!("r{n:04}")).unwrap() += 1;
        }
        assert_eq!(records.written().map(BTreeSet::len), Some(125));
        *records.get_mut("r0125").unwrap() += 1;
        assert_eq!(records.written(), None);
    }

    /// Readings that list `report`, more than a log keeps, and readings that
    /// list `memo`, fewer: qualifying and unqualifying each reaches the
    /// bindings that read them, through a log that gave up and one that did not.
    const LOGGED_AND_NOT: &str = r#"
claim seen;
evidence glimpse from "a glimpse";
evidence report from "the report";
evidence memo from "the memo";
caveat doubt consequence low;
readings many from glimpse limit 64;
readings few from glimpse limit 16;
event start;
event look v min 0 max 10;
event jot v min 0 max 10;
event flag;
event unflag;
event flagmemo;
event unflagmemo;
on start reveal report supports seen;
on start reveal memo supports seen;
on look when observed(report) sample many = v supports seen;
on jot when observed(memo) sample few = v supports seen;
on flag qualify report with doubt;
on unflag unqualify report with doubt;
on flagmemo qualify memo with doubt;
on unflagmemo unqualify memo with doubt;
bind h.many = latest(many) when has_sample(many);
bind h.few = latest(few) when has_sample(few);
"#;

    #[test]
    fn qualifying_many_records_or_few_reaches_the_bindings_that_read_them() {
        let mut session = ReactiveSession::from_source(LOGGED_AND_NOT).unwrap();
        let none = BTreeMap::new();
        let v = BTreeMap::from([(String::from("v"), 1.0)]);
        session.apply("start", &none).unwrap();
        for _ in 0..40 {
            session.apply("look", &v).unwrap();
        }
        for _ in 0..10 {
            session.apply("jot", &v).unwrap();
        }
        let listing = |session: &ReactiveSession, evidence: &str| {
            session
                .observation_qualifications
                .values()
                .filter(|lineage| lineage.evidence.contains(evidence))
                .count()
        };
        assert_eq!(
            (listing(&session, "report"), listing(&session, "memo")),
            (40, 10)
        );
        let doubted = |session: &ReactiveSession| {
            ["many", "few"].map(|property| {
                session.binding_qualifications["h"][property]
                    .caveats
                    .contains("doubt")
            })
        };
        // Qualifying report writes its 40 records, more than a log keeps, so
        // the check compares every record; qualifying memo writes 10, each named.
        let logged = |evidence: &str| {
            let mut probe = session.clone();
            probe.begin_write_log();
            probe
                .apply_qualification(evidence, "doubt", &Provenance::default())
                .unwrap();
            probe
                .observation_qualifications
                .written()
                .map(BTreeSet::len)
        };
        assert_eq!((logged("report"), logged("memo")), (None, Some(10)));
        assert_eq!(doubted(&session), [false, false]);
        for (event, expected) in [
            ("flag", [true, false]),
            ("flagmemo", [true, true]),
            ("unflag", [false, true]),
            ("unflagmemo", [false, false]),
            ("flagmemo", [false, true]),
            ("flag", [true, true]),
        ] {
            session.apply(event, &none).unwrap();
            assert_eq!(doubted(&session), expected, "after {event}");
        }
    }

    #[test]
    fn every_graph_write_names_its_node_until_the_log_is_full() {
        let before = logged_program(
            "claim ok;\ncaveat c consequence low;\nevidence e from \"tool\";\n\
             decisions d limit 2;\nevent decide;\non decide commit d because enough using 1;\n",
        );
        let mut session = before.clone();
        session.begin_write_log();
        let c = session.symbols["c"];
        let claim = session.add_named(
            String::from("more"),
            NodeKind::Claim {
                proposition: String::from("more"),
            },
        );
        let caveat = session.add_named_caveat(String::from("late"), "late", Consequence::Low);
        let commitment = session.commit_named(String::from("d@1"), &[], StopReason::Enough);
        session.set_node_attention(c, Attention::Examined);
        session.reopen_commitment(commitment, claim);
        assert!(session.open_restored_commitment(commitment));
        assert!(!session.open_restored_commitment(claim));
        assert_eq!(
            session.graph.written.only(),
            Some(&BTreeSet::from([c, claim, caveat, commitment]))
        );
        assert_eq!(
            session
                .graph
                .added_since(&before.graph)
                .map(|(_, nodes)| nodes),
            Some(BTreeSet::from([c, claim, caveat, commitment]))
        );
        // Past 32 nodes the log gives up, and the check reads every node.
        for n in 0..28 {
            session.add_named_caveat(format!("k{n}"), "k", Consequence::Low);
        }
        assert_eq!(session.graph.written.only().map(BTreeSet::len), Some(32));
        let last = session.add_named_caveat(String::from("last"), "k", Consequence::Low);
        assert_eq!(session.graph.written.only(), None);
        let (_, nodes) = session.graph.added_since(&before.graph).unwrap();
        assert_eq!(nodes.len(), 33);
        assert!(nodes.contains(&last));
        assert!(!session.graph.unchanged_since(&before.graph));
    }

    #[test]
    fn a_graph_write_copies_the_node_it_writes_and_no_other() {
        let before = logged_program(
            "claim ok;\ncaveat c consequence low;\nevidence e from \"tool\";\n\
             decisions d limit 2;\nevent decide;\non decide commit d because enough using 1;\n",
        );
        let mut session = before.clone();
        let c = session.symbols["c"];
        session.set_node_attention(c, Attention::Examined);
        let added = session.add_named_caveat(String::from("late"), "late", Consequence::Low);
        let commitment = session.commit_named(String::from("d@1"), &[], StopReason::Enough);
        assert!(!Arc::ptr_eq(&before.graph, &session.graph));
        assert_eq!(session.graph.nodes.len(), before.graph.nodes.len() + 2);
        // Every node the session began with is still the one `before` holds,
        // but for the caveat written; the two added are the session's alone.
        for (id, node) in &before.graph.nodes {
            assert_eq!(
                std::ptr::eq(node, &session.graph.nodes[&id]),
                id != c,
                "node {id}"
            );
        }
        assert!(!before.graph.nodes.contains_key(&added));
        assert!(!before.graph.nodes.contains_key(&commitment));
        assert!(matches!(
            before.graph.nodes[&c],
            NodeKind::Caveat {
                attention: Attention::Unexamined,
                ..
            }
        ));
        // Departure takes nodes out of the session's graph and leaves them
        // in the copy.
        let departing = HashMap::from_iter([(added, String::from("late"))]);
        session.depart_records(&departing, |_, _, _| {});
        assert!(!session.graph.nodes.contains_key(&added));
        assert_eq!(session.graph.nodes.len(), before.graph.nodes.len() + 1);
        for (id, node) in &before.graph.nodes {
            assert_eq!(std::ptr::eq(node, &session.graph.nodes[&id]), id != c);
        }
    }

    #[test]
    fn the_ledger_reads_as_the_list_of_records_it_stands_in_for() {
        // Records are added at the end and taken out anywhere, as observing
        // and departing do, and a node's number is never given again.
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut below = |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        let mut ledger = Observations::default();
        let mut list: Vec<NodeId> = Vec::new();
        let mut node = 0;
        // Copies taken on the way, each with the list of that moment.
        let mut earlier: Vec<(Observations, Vec<NodeId>)> = Vec::new();
        let (mut began, mut equal) = (0, 0);
        for step in 0..3_000 {
            match below(8) {
                0..=4 => {
                    node += 1 + below(40);
                    ledger.push(node);
                    list.push(node);
                }
                5 | 6 if !list.is_empty() => {
                    let id = list.remove(below(list.len() as u64) as usize);
                    ledger.remove(id);
                }
                // A record the ledger does not hold leaves it as it is.
                _ => ledger.remove(node + 1),
            }
            ledger.check();
            assert!(ledger.iter().eq(list.iter().copied()), "step {step}");
            assert_eq!(ledger.iter().len(), list.len());
            assert_eq!(ledger.is_empty(), list.is_empty());
            for (before, listed) in &earlier {
                assert_eq!(before.begins(&ledger), list.starts_with(listed));
                assert_eq!(*before == ledger, *listed == list);
                began += usize::from(before.begins(&ledger));
                equal += usize::from(*before == ledger);
            }
            if step % 40 == 0 {
                for pair in list.windows(2) {
                    assert!(ledger.rank(pair[0]).unwrap() < ledger.rank(pair[1]).unwrap());
                }
                assert!(list.iter().all(|id| ledger.holds(*id)));
                assert!(!ledger.holds(node + 1));
                assert_eq!(ledger.rank(node + 1), None);
                // Built again from the list it numbers its records afresh,
                // and is the same ledger.
                let mut rebuilt = Observations::default();
                for &id in &list {
                    rebuilt.push(id);
                }
                assert_eq!(ledger, rebuilt);
                assert_eq!(format!("{ledger:?}"), format!("{list:?}"));
                if earlier.len() == 6 {
                    earlier.remove(below(6) as usize);
                }
                earlier.push((ledger.clone(), list.clone()));
            }
        }
        // Both answers came up, each way.
        assert!(began > 200 && equal > 10, "{began} {equal}");
    }

    #[test]
    fn ranks_order_names_as_the_ledger_and_the_relations_do() {
        // A stream whose readings take a stance, so that its window takes
        // relations out of the graph from the front, beside declared evidence.
        let source = "claim seen;\nevidence e from \"e\";\nevidence f from \"f\";\n\
             evidence g from \"g\";\nevidence h from \"h\";\nevidence feed from \"feed\";\n\
             readings r from feed window 2;\nevent seee;\nevent seef;\nevent seeg;\n\
             event look;\nevent read v min 0 max 9;\nevent note v min 0 max 9;\n\
             on seee reveal e supports seen;\non seef reveal f opposes seen;\n\
             on seeg reveal g supports seen;\non look reveal h;\n\
             on read sample r = v supports seen;\non note sample r = v;\n";
        let mut session = ReactiveSession::from_source(source).unwrap();
        // The order of every pair of names, as `rank` gives it and as the
        // lists it stands in for give it: the place of a name in the
        // ledger, or without one the place of its first `supports` or
        // `opposes` relation in the graph.
        fn check(session: &ReactiveSession, ledger: bool) {
            assert_eq!(!session.observations.is_empty(), ledger);
            let listed = session.observation_names();
            let place = |name: &str| {
                let found = if ledger {
                    listed.iter().position(|held| held == name)
                } else {
                    session.symbols.get(name).and_then(|id| {
                        session.graph.edges.iter().position(|edge| {
                            edge.from == *id
                                && matches!(edge.relation, Relation::Supports | Relation::Opposes)
                        })
                    })
                };
                found.unwrap_or(usize::MAX)
            };
            let ranks = session.observation_ranks();
            let mut names = session.symbols.keys().collect::<Vec<_>>();
            names.push("nowhere");
            names.sort_unstable();
            for one in &names {
                for other in &names {
                    assert_eq!(
                        ranks.rank(one).cmp(&ranks.rank(other)),
                        place(one).cmp(&place(other)),
                        "{one} and {other}"
                    );
                }
            }
            session.check_observations();
        }
        let send = |session: &mut ReactiveSession, event: &str, ledger: bool| {
            let payload = match event {
                "read" | "note" => "{\"v\":1}",
                _ => "{}",
            };
            let outcome = session
                .dispatch_outcome_json(event, payload)
                .unwrap_or_else(|fatal| panic!("{event}: {}", fatal.message));
            let outcome = serde_json::to_string(&outcome).unwrap();
            assert!(outcome.contains("accepted"), "{event}: {outcome}");
            check(session, ledger);
        };
        check(&session, false);
        for event in ["seef", "read", "read", "seee", "read", "read", "read"] {
            send(&mut session, event, false);
        }
        // The window has taken readings and their relations out.
        assert!(!session.symbols.contains_key("r@1"));
        assert!(session.symbols.contains_key("r@5"));
        let before = session.clone();
        // The first neutral observation starts the ledger from the relations.
        send(&mut session, "look", true);
        assert_eq!(session.observation_names(), ["f", "e", "r@4", "r@5", "h"]);
        assert!(before.observations.is_empty());
        for event in ["note", "seeg", "read", "note", "look", "note"] {
            send(&mut session, event, true);
        }
        assert_eq!(
            session.observation_names(),
            ["f", "e", "h", "g", "r@8", "r@9"]
        );
        // A save names the ledger's records, and a restore reads them back.
        let save = session.save_json().unwrap();
        assert!(save.contains(r#""observations":["f","e","h","g","r@8","r@9"]"#));
        let restored = ReactiveSession::restore_json(source, &save).unwrap();
        check(&restored, true);
        // It gives the records it adds numbers of its own, so the two
        // ledgers hold the same names and not the same nodes.
        assert_eq!(restored.observation_names(), session.observation_names());
        assert_ne!(restored.observations, session.observations);
        assert_eq!(restored.save_json().unwrap(), save);
    }

    #[test]
    fn an_event_logs_every_container_from_before_its_first_rule() {
        let mut session = ReactiveSession::from_source(
            "claim ok;\nevidence e from \"tool\";\nstate a = 0;\nevent see v min 0 max 1;\n\
             on see reveal e supports ok;\non see set a = 1 / v;\n",
        )
        .unwrap();
        let old = session.clone();
        // The second rule divides by zero: the event stops after the first
        // rule's writes and before it reads what it changed, so every log is
        // still open.
        assert!(session
            .run_event(&old, "see", &BTreeMap::from([(String::from("v"), 0.0)]))
            .is_err());
        assert_eq!(session.graph.written.only(), Some(&BTreeSet::new()));
        // The observation records were empty: a release build gave up their
        // log at the first write (`LOG_MIN`), where a debug build named it.
        let observed = (LOG_FROM == 0).then(|| BTreeSet::from([String::from("e")]));
        assert_eq!(
            session.observation_qualifications.written(),
            observed.as_ref()
        );
        let nothing = BTreeSet::new();
        assert_eq!(session.reading_streams.written(), Some(&nothing));
        assert_eq!(session.decision_series.written(), Some(&nothing));
        assert_eq!(session.commitment_bases.written(), Some(&nothing));
        assert_eq!(session.commitment_grounds.written(), Some(&nothing));
        assert_eq!(session.examination_qualifications.written(), Some(&nothing));
        assert_eq!(session.reopening_qualifications.written(), Some(&nothing));
    }

    #[test]
    fn a_log_starts_only_in_a_container_of_log_min_records_or_more() {
        let a = BTreeSet::from([String::from("a")]);
        assert_eq!(
            Written::<String>::start("a", LOG_MIN, LOG_MIN).only(),
            Some(&a)
        );
        // In a smaller one, comparing every record costs less than logging
        // the writes, and the first write gives up the log.
        assert_eq!(
            Written::<String>::start("a", LOG_MIN - 1, LOG_MIN).only(),
            None
        );
        // This build starts from `LOG_FROM`: a debug build logs every
        // container, so none is smaller there.
        let mut log = Written::<String>::Nothing;
        log.note("a", LOG_FROM);
        assert_eq!(log.only(), Some(&a));
        if let Some(fewer) = LOG_FROM.checked_sub(1) {
            let mut log = Written::<String>::Nothing;
            log.note("a", fewer);
            assert_eq!(log.only(), None);
        }
    }

    #[test]
    fn a_copy_shares_what_it_holds_but_not_the_log() {
        let mut records = logged(&[("a", 1)]);
        records.begin();
        *records.get_mut("a").unwrap() = 2;
        assert_eq!(written(&records), Some(vec!["a"]));
        let copy = records.clone();
        assert!(Arc::ptr_eq(&copy, &records));
        assert_eq!(written(&copy), None);
        let mut session = logged_program("claim ok;\ncaveat c consequence low;\nevent idle;\n");
        session.begin_write_log();
        let c = session.symbols["c"];
        session.set_node_attention(c, Attention::Examined);
        assert_eq!(session.graph.written.only(), Some(&BTreeSet::from([c])));
        let copy = session.clone();
        assert!(Arc::ptr_eq(&copy.graph, &session.graph));
        assert_eq!(copy.graph.written.only(), None);
    }

    #[test]
    fn a_record_both_sessions_share_is_the_same_without_reading_it() {
        #[derive(Debug)]
        struct Unread;
        impl PartialEq for Unread {
            fn eq(&self, _: &Self) -> bool {
                panic!("read a record both sessions share")
            }
        }
        let before = Records::new(BTreeMap::from([
            (String::from("a"), Arc::new(Unread)),
            (String::from("b"), Arc::new(Unread)),
        ]));
        let mut records = before.clone();
        records.begin();
        // A write that does not say which record: the check compares every
        // record, each the one `before` holds.
        records
            .make_mut()
            .insert(String::from("c"), Arc::new(Unread));
        records.make_mut().remove("c");
        assert!(!Arc::ptr_eq(&records, &before));
        assert!(records.unchanged_since(&before));
    }

    #[test]
    fn records_print_what_their_arc_printed() {
        let records = records(&[("a", 1)]);
        let arc = Arc::new(BTreeMap::from([(String::from("a"), 1)]));
        assert_eq!(format!("{records:?}"), format!("{arc:?}"));
        assert_eq!(format!("{records:#?}"), format!("{arc:#?}"));
        let named: NamedRecords<u32> =
            NamedRecords::new([(String::from("a"), 1)].into_iter().collect());
        assert_eq!(format!("{named:?}"), format!("{arc:?}"));
        assert_eq!(format!("{named:#?}"), format!("{arc:#?}"));
    }

    /// The records of `map`, as `(name, value)` pairs in the order held.
    fn held<M: RecordMap<Value = u32>>(records: &Logged<M>) -> Vec<(String, u32)> {
        records
            .records()
            .map(|(name, value)| (name.to_string(), *value))
            .collect()
    }

    #[test]
    fn named_records_answer_and_log_as_records_do() {
        // The same writes to both kinds, event after event, each beside the
        // copy its event began from.
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut below = |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        for names in [3, 40, 400] {
            let start: Vec<(String, u32)> = (0..names / 2).map(|n| (format!("r{n}"), n)).collect();
            let mut plain: Records<u32> = Records::new(start.iter().cloned().collect());
            let mut named: NamedRecords<u32> = NamedRecords::new(start.into_iter().collect());
            let (mut plain_before, mut named_before) = (plain.clone(), named.clone());
            for step in 0..4_000 {
                let name = format!("r{}", below(names as u64));
                match below(16) {
                    0 => {
                        plain.end();
                        named.end();
                    }
                    1 => {
                        (plain_before, named_before) = (plain.clone(), named.clone());
                        plain.begin();
                        named.begin();
                    }
                    2..=6 => {
                        assert_eq!(
                            plain.insert(name.clone(), step),
                            named.insert(name.clone(), step)
                        );
                    }
                    7..=9 => assert_eq!(plain.remove(&name), named.remove(&name)),
                    10..=12 => {
                        let wrote = |record: Option<&mut u32>| {
                            record.map(|value| {
                                *value += 1;
                                *value
                            })
                        };
                        assert_eq!(wrote(plain.get_mut(&name)), wrote(named.get_mut(&name)));
                    }
                    13 => {
                        let wrote = plain.get_mut(&name).map(|value| *value = step).is_some();
                        assert_eq!(named.replace(&name, step), wrote);
                    }
                    _ => {
                        assert_eq!(plain.get(&name), named.get(&name));
                        assert_eq!(plain.contains_key(&name), named.get(&name).is_some());
                    }
                }
                assert_eq!(plain.written(), named.written());
                assert_eq!(
                    plain.unchanged_since(&plain_before),
                    named.unchanged_since(&named_before)
                );
                assert_eq!(plain.len(), named.len());
                assert_eq!(plain.is_empty(), named.is_empty());
                if step % 97 == 0 {
                    assert_eq!(held(&plain), held(&named));
                    assert_eq!(held(&plain_before), held(&named_before));
                    assert!(named.keys().eq(plain.keys().map(String::as_str)));
                    assert!(named.values().eq(plain.values()));
                    assert_eq!(named.to_map(), **plain);
                }
            }
            assert_eq!(held(&plain), held(&named));
            assert_eq!(held(&plain_before), held(&named_before));
        }
    }

    #[test]
    fn a_write_to_named_records_copies_the_record_it_writes_and_no_other() {
        static COPIES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        #[derive(Debug, PartialEq)]
        struct Counted(u32);
        impl Clone for Counted {
            fn clone(&self) -> Self {
                COPIES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Counted(self.0)
            }
        }
        impl Same for Counted {
            fn same(&self, other: &Self) -> bool {
                self == other
            }
        }
        let copies = || COPIES.load(std::sync::atomic::Ordering::Relaxed);
        let before: NamedRecords<Counted> = NamedRecords::new(
            (0..1_000)
                .map(|n| (format!("r{n:04}"), Counted(n)))
                .collect(),
        );
        let made = copies();
        let mut records = before.clone();
        records.begin();
        // A record that is not there copies nothing.
        assert!(records.get_mut("absent").is_none());
        assert!(!records.remove("absent"));
        assert!(!records.replace("absent", Counted(0)));
        assert_eq!(copies(), made);
        assert!(records.unchanged_since(&before));
        // Writing one record copies that record alone.
        records.get_mut("r0500").unwrap().0 = 0;
        assert_eq!(copies(), made + 1);
        records.get_mut("r0500").unwrap().0 = 1;
        assert_eq!(copies(), made + 1);
        // Writing over one copies none, and says which it wrote.
        assert!(records.replace("r0600", Counted(9)));
        assert_eq!(copies(), made + 1);
        assert_eq!(before["r0600"], Counted(600));
        assert_eq!(records["r0600"], Counted(9));
        let written = records.written().expect("so few writes are logged");
        assert!(written.contains("r0600"));
        // Adding, replacing and removing beside it copies none.
        assert!(!records.insert(String::from("r0500a"), Counted(7)));
        assert!(records.insert(String::from("r0501"), Counted(8)));
        assert!(records.remove("r0499"));
        assert_eq!(
            copies(),
            made + 1,
            "what is replaced or removed is not copied"
        );
        assert!(!records.unchanged_since(&before));
        assert_eq!(before.len(), 1_000);
        assert_eq!(before["r0500"], Counted(500));
        assert_eq!(before["r0501"], Counted(501));
        assert_eq!(before["r0499"], Counted(499));
        assert_eq!(records["r0500"], Counted(1));
        assert_eq!(records.len(), 1_000);
    }

    #[test]
    fn take_back_counts_follow_the_list() {
        fn record(caveat: u64, evidence: u64) -> TakeBack {
            TakeBack {
                evidence: format!("e@{evidence}"),
                caveat: format!("k{caveat}"),
                sequence: 0,
                event: String::from("t"),
                agent: None,
            }
        }
        fn check(take_backs: &TakeBacks) {
            for caveat in 0..4 {
                for evidence in 0..40 {
                    let (caveat, evidence) = (format!("k{caveat}"), format!("e@{evidence}"));
                    // `count` checks itself against the list in a debug build;
                    // this test checks it in any build.
                    let listed = take_backs
                        .iter()
                        .filter(|record| record.caveat == caveat && record.evidence == evidence)
                        .count();
                    assert_eq!(take_backs.count(&caveat, &evidence), listed);
                }
            }
            let mut counted = 0;
            for (_, caveats) in take_backs.counts.iter() {
                assert!(!caveats.is_empty(), "an occurrence counted with no caveat");
                for (_, count) in caveats {
                    assert!(*count > 0, "a caveat counted none");
                    counted += count;
                }
            }
            assert_eq!(counted, take_backs.len());
        }
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        let mut take_backs = TakeBacks::new(vec![record(0, 0), record(0, 0), record(1, 0)]);
        check(&take_backs);
        let mut copies = Vec::new();
        for step in 0..2_000 {
            match next(8) {
                0..=4 => take_backs.push(record(next(4), next(40))),
                5 => {
                    let gone = next(40);
                    take_backs.retain(|record| record.evidence != format!("e@{gone}"));
                }
                6 => {
                    let caveat = format!("k{}", next(4));
                    let mut first = true;
                    take_backs.retain(|record| {
                        let taken = first && record.caveat == caveat;
                        first &= !taken;
                        !taken
                    });
                }
                _ => {
                    let list = (**take_backs).clone();
                    take_backs = TakeBacks::new(list);
                }
            }
            check(&take_backs);
            if step % 250 == 0 {
                copies.push(((**take_backs).clone(), take_backs.clone()));
            }
        }
        // A copy keeps the counts of its list, whatever was written after it.
        for (list, copy) in &copies {
            assert_eq!(copy.as_slice(), list.as_slice());
            check(copy);
        }
    }

    #[test]
    fn a_record_under_another_name_is_a_change() {
        // Copied, a map's log says nothing of what was written, so it is
        // compared whole: the same records under other names differ.
        fn moved<M: RecordMap<Value = u32>>(before: Logged<M>) {
            let mut records = before.clone();
            assert!(records.remove("a"));
            assert!(!records.insert(String::from("b"), 1));
            assert_eq!(records.written(), None);
            assert!(!records.unchanged_since(&before));
        }
        moved(Records::new(BTreeMap::from([(String::from("a"), 1)])));
        moved(NamedRecords::new(
            [(String::from("a"), 1)].into_iter().collect(),
        ));
    }

    #[test]
    fn a_name_named_records_hold_keeps_its_text() {
        let mut records: NamedRecords<u32> = NamedRecords::default();
        records.insert(String::from("a"), 1);
        let text = |records: &NamedRecords<u32>| records.keys().next().unwrap().as_ptr();
        let first = text(&records);
        // A copy keeps the first text alive, so text made anew could not be
        // put where it was: the same address is the same text.
        let copy = records.clone();
        assert!(records.insert(String::from("a"), 2));
        assert_eq!(text(&records), first);
        assert_eq!(text(&copy), first);
        assert_eq!(records.get("a"), Some(&2));
        assert_eq!(copy.get("a"), Some(&1));
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

    #[cfg(debug_assertions)]
    const TWO_STANCES: &str = "claim x;\nclaim y;\nevidence a from \"tool\";\nevidence b from \"tool\";\na supports x;\nb supports y;\nevent idle;\n";

    /// Numbers `session`'s relation lists as if its graph's relations had
    /// come the other way round. Each record still keeps what it keeps and
    /// the count is right, so no read can tell; only the check can.
    #[test]
    fn a_sample_and_a_renewal_copy_their_lists_and_nothing_the_lists_hold() {
        let source = "claim seen;\nevidence feed from \"feed\";\nevidence e from \"e\";\n\
             readings r from feed limit 16;\nrenewable e limit 16;\n\
             event read v min 0 max 9;\nevent again;\n\
             on read sample r = v supports seen;\non again renew e;\n";
        let mut session = ReactiveSession::from_source(source).unwrap();
        for v in 0..3 {
            let payload = format!("{{\"v\": {v}}}");
            session.dispatch_json("read", &payload).unwrap();
            session.dispatch_json("again", "{}").unwrap();
        }
        let before = session.clone();
        session.dispatch_json("read", "{\"v\": 7}").unwrap();
        session.dispatch_json("again", "{}").unwrap();
        // Each reading the stream held is the one the session before holds.
        let old = &before.reading_streams["r"].occurrences;
        let new = &session.reading_streams["r"].occurrences;
        assert_eq!((old.len(), new.len()), (3, 4));
        assert!(old.iter().zip(new).all(|(old, new)| Arc::ptr_eq(old, new)));
        // And each name the evidence had is the text the session before has.
        let old = &before.renewals["e"].occurrences;
        let new = &session.renewals["e"].occurrences;
        assert_eq!((old.len(), new.len()), (4, 5));
        assert!(old
            .iter()
            .zip(new)
            .all(|(old, new)| std::ptr::eq(old.as_str(), new.as_str())));
    }

    #[test]
    fn a_departure_rewrites_only_the_readings_that_name_what_departs() {
        // Each reading of `b` is taken from the current reading of `a`, whose
        // window holds one: a new reading of `a` takes the one before out of
        // the session, and the readings of `b` that name it keep a marker.
        let source = "claim seen;\nevidence feed from \"feed\";\nevidence src from \"src\";\n\
             readings a from src window 1;\nreadings b from feed limit 16;\n\
             event ina v min 0 max 9;\nevent inb;\n\
             on ina sample a = v;\non inb sample b = latest(a);\n";
        let mut session = ReactiveSession::from_source(source).unwrap();
        let mut rewritten = 0;
        let mut shared = 0;
        for (round, event) in [
            "ina", "inb", "ina", "inb", "inb", "ina", "inb", "ina", "ina",
        ]
        .into_iter()
        .enumerate()
        {
            let before = session.clone();
            let saved = before.save_json().unwrap();
            let payload = format!("{{\"v\": {round}}}");
            session
                .dispatch_json(event, if event == "ina" { &payload } else { "{}" })
                .unwrap();
            let held = |session: &ReactiveSession| {
                let readings = session.reading_streams["a"].occurrences.iter();
                readings
                    .map(|reading| reading.id.clone())
                    .collect::<Vec<_>>()
            };
            let now = held(&session);
            let departed = held(&before)
                .into_iter()
                .filter(|name| !now.contains(name))
                .collect::<Vec<_>>();
            let old = &before.reading_streams["b"].occurrences;
            let new = &session.reading_streams["b"].occurrences;
            for (old, new) in old.iter().zip(new) {
                let named = departed
                    .iter()
                    .any(|name| old.provenance.evidence.contains(name));
                assert_eq!(Arc::ptr_eq(old, new), !named, "round {round}: {}", old.id);
                if named {
                    rewritten += 1;
                    assert!(new.provenance.departed.contains_key("a"), "round {round}");
                    assert!(departed
                        .iter()
                        .all(|name| !new.provenance.evidence.contains(name)));
                } else {
                    shared += 1;
                }
            }
            // The session before the event still holds what it held.
            assert_eq!(before.save_json().unwrap(), saved, "round {round}");
        }
        // The four readings of `b` are each rewritten once, when the
        // reading of `a` they were taken from departs.
        assert_eq!((rewritten, shared), (4, 14));
    }

    #[cfg(debug_assertions)]
    fn misnumber(session: &mut ReactiveSession) {
        let reversed: Vec<Edge> = session.graph.edges.iter().rev().cloned().collect();
        session.graph.relations = Arc::new(Relations::new(&reversed));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "is numbered in graph order")]
    fn an_event_checks_the_relation_lists() {
        let mut session = ReactiveSession::from_source(TWO_STANCES).unwrap();
        misnumber(&mut session);
        let _ = session.dispatch_json("idle", "{}");
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "is numbered in graph order")]
    fn a_restore_checks_the_relation_lists() {
        let save = ReactiveSession::from_source_cached(TWO_STANCES)
            .unwrap()
            .save()
            .unwrap();
        // A restore starts from the session this thread loaded last.
        LOADED.with_borrow_mut(|loaded| {
            let (_, session) = loaded
                .iter_mut()
                .find(|(source, _)| source == TWO_STANCES)
                .expect("the program loaded");
            misnumber(session);
        });
        let _ = ReactiveSession::restore(TWO_STANCES, &save);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "the relation lists hold every edge")]
    fn a_read_checks_that_the_lists_hold_every_edge() {
        let mut session = ReactiveSession::from_source(TWO_STANCES).unwrap();
        let (a, y) = (session.symbols["a"], session.symbols["y"]);
        // An edge written without `added`: the lists do not hold it.
        session.graph.write().relate(a, Relation::Supports, y);
        let _ = session.graph.stance(a);
    }

    #[test]
    fn a_restore_that_adds_no_relation_shares_the_loaded_graph() {
        let save = ReactiveSession::from_source_cached(TWO_STANCES)
            .unwrap()
            .save()
            .unwrap();
        let one = ReactiveSession::restore(TWO_STANCES, &save).unwrap();
        let two = ReactiveSession::restore(TWO_STANCES, &save).unwrap();
        // Neither restore wrote the graph, so both hold the loaded one.
        assert!(Arc::ptr_eq(&one.graph.graph, &two.graph.graph));
    }
}
