//! Each record's relations, kept by the record a read asks about: what it
//! supports or opposes, what it retains or relies on, what qualifies it and
//! what reopens it. A read looks the record up here instead of scanning every
//! relation in the graph. The graph (`SealedGraph`) holds these lists beside
//! its relations and, as their only writer, adds each relation it adds and
//! removes each one departure removes, so the lists always hold exactly the
//! graph's relations. `in_context` relations are not kept: no read an event
//! makes asks for them.
use super::*;
use crate::Edge;
use std::ops::Range;

/// One relation, as the record that keeps it sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Link {
    /// Where the relation stands in graph order. Each relation added takes
    /// the next number, and departure leaves the numbers of the relations it
    /// keeps as they were, so the numbers of a record's links, in all its
    /// lists, are in graph order.
    pub(super) serial: u64,
    /// The record at the relation's other end.
    pub(super) other: NodeId,
    pub(super) relation: Relation,
    /// The list that keeps it.
    list: List,
}

/// Which of a record's lists keeps a relation, in the order a record's lists
/// are held.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum List {
    /// `supports` and `opposes`, kept by the record they come from.
    Stance,
    /// `retains` and `relies_on`, kept by the commitment they come from.
    Basis,
    /// `qualifies`, kept by the record qualified.
    Qualifiers,
    /// `reopens`, kept by the commitment reopened.
    Reopeners,
}

const LISTS: [List; 4] = [List::Stance, List::Basis, List::Qualifiers, List::Reopeners];

impl List {
    /// The list that keeps `from relation to`, the record that keeps it and
    /// the record at its other end; `None` for `in_context`.
    pub(super) fn place(
        from: NodeId,
        relation: Relation,
        to: NodeId,
    ) -> Option<(Self, NodeId, NodeId)> {
        Some(match relation {
            Relation::Supports | Relation::Opposes => (Self::Stance, from, to),
            Relation::Retains | Relation::ReliesOn => (Self::Basis, from, to),
            Relation::Qualifies => (Self::Qualifiers, to, from),
            Relation::Reopens => (Self::Reopeners, to, from),
            Relation::InContext => return None,
        })
    }

    fn of(edge: &Edge) -> Option<(Self, NodeId, NodeId)> {
        Self::place(edge.from, edge.relation, edge.to)
    }
}

/// How many records' lists one chunk holds.
const CHUNK: usize = 16;
/// `CHUNK` counted in node ids.
const CHUNK_NODES: NodeId = CHUNK as NodeId;
/// The room for more links a copy of a chunk makes: a chunk is copied to be
/// written.
const ROOM: usize = 4;
/// From this many relations added at once, `add` writes each chunk they go
/// to once, rather than once for each run of them.
const BULK: usize = 32;

#[cfg(test)]
thread_local! {
    /// How many links the writes to the lists have moved or copied within
    /// their chunks, so that a test can bound the work a write does.
    static MOVED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Counts `links` moved or copied within a chunk.
#[cfg_attr(not(test), allow(unused_variables))]
fn moved(links: usize) {
    #[cfg(test)]
    MOVED.with(|moved| moved.set(moved.get() + links));
}

/// The chunk and slot that hold `id`'s lists.
fn place(id: NodeId) -> (NodeId, usize) {
    (id / CHUNK_NODES, (id % CHUNK_NODES) as usize)
}

/// Moves `ends`, where records end, down over `removed` links removed
/// before them.
fn lowered(ends: &mut [u32], removed: usize) {
    let removed = u32::try_from(removed).expect("a chunk holds fewer than 2^32 links");
    for end in ends {
        *end -= removed;
    }
}

/// The lists of the `CHUNK` records that sit together among node ids, in one
/// run of links: by record, each record's in list order, and each list in
/// graph order. The links of the record in slot `slot` end at `ends[slot]`.
/// One run keeps a write to one of these records to one copy of the run, as
/// a copy of the graph's relations would make, where lists of their own
/// would each be another.
#[derive(Debug, PartialEq, Eq)]
struct Chunk {
    ends: [u32; CHUNK],
    links: Vec<Link>,
}

impl Clone for Chunk {
    fn clone(&self) -> Self {
        let mut links = Vec::with_capacity(self.links.len() + ROOM);
        links.extend_from_slice(&self.links);
        Self {
            ends: self.ends,
            links,
        }
    }
}

impl Chunk {
    fn empty() -> Self {
        Self {
            ends: [0; CHUNK],
            links: Vec::new(),
        }
    }

    /// Where the links of the record in `slot` sit.
    fn record(&self, slot: usize) -> Range<usize> {
        let start = slot
            .checked_sub(1)
            .map_or(0, |before| self.ends[before] as usize);
        start..self.ends[slot] as usize
    }

    /// Where `list` of the record in `slot` sits.
    fn list(&self, slot: usize, list: List) -> Range<usize> {
        let record = self.record(slot);
        let links = &self.links[record.clone()];
        let start = links.partition_point(|link| link.list < list);
        let end = start + links[start..].partition_point(|link| link.list == list);
        record.start + start..record.start + end
    }

    /// Moves where the record in `slot`, and each record after it, ends.
    fn grown(&mut self, slot: usize, added: usize) {
        let added = u32::try_from(added).expect("a chunk holds fewer than 2^32 links");
        for end in &mut self.ends[slot..] {
            *end += added;
        }
    }

    /// This chunk with `added` at the end of the lists that keep them: links
    /// newer than every link the chunk holds, each with the slot of the
    /// record that keeps it, in slot and list order and each list's in graph
    /// order.
    fn merged(&self, added: &[(NodeId, usize, Link)]) -> Self {
        let mut links = Vec::with_capacity(self.links.len() + added.len() + ROOM);
        let mut ends = [0; CHUNK];
        let mut added = added.iter().peekable();
        for (slot, end) in ends.iter_mut().enumerate() {
            let held = &self.links[self.record(slot)];
            let mut start = 0;
            for list in LISTS {
                let kept = held[start..].partition_point(|link| link.list == list);
                links.extend_from_slice(&held[start..start + kept]);
                start += kept;
                while let Some(&(_, _, link)) =
                    added.next_if(|&&(_, by, link)| by == slot && link.list == list)
                {
                    links.push(link);
                }
            }
            *end = u32::try_from(links.len()).expect("a chunk holds fewer than 2^32 links");
        }
        moved(links.len());
        Self { ends, links }
    }
}

/// Every record's lists, in fixed-size chunks keyed by where they sit among
/// node ids, as the names are kept by node: a copy shares every chunk, and a
/// write copies only the chunk it changes. A chunk whose records keep no
/// relation is dropped, so what this holds grows with the relations the
/// graph holds now rather than with every id the session has issued.
#[derive(Clone, Default)]
pub(super) struct Relations {
    chunks: BTreeMap<NodeId, Arc<Chunk>>,
    /// How many of the graph's relations these hold, `in_context` ones
    /// included: all of them, after every write.
    held: usize,
    /// How many of them are `qualifies`.
    qualifies: usize,
    /// The number the next relation added takes.
    next: u64,
}

/// What departure removed from the graph, noted relation by relation as it
/// removes them, for `Relations::depart`.
#[derive(Default)]
pub(super) struct Departed {
    /// How many relations departure removed.
    removed: usize,
    /// How many of them are `qualifies`.
    qualifies: usize,
    /// The records whose lists keep one of them, in the order noted, each
    /// once unless another came between. The first is held apart, so that
    /// a departure whose relations one record keeps allocates nothing.
    keeper: Option<NodeId>,
    keepers: Vec<NodeId>,
    /// Each relation a list keeps, as the link its keeper holds, for debug
    /// builds to check that departure removes exactly these links.
    #[cfg(debug_assertions)]
    links: Vec<(NodeId, List, NodeId, Relation)>,
}

impl Departed {
    /// Notes `edge`, a relation departure removed.
    #[cfg_attr(not(debug_assertions), allow(unused_variables))]
    pub(super) fn note(&mut self, edge: &Edge) {
        self.removed += 1;
        if edge.relation == Relation::Qualifies {
            self.qualifies += 1;
        }
        if let Some((list, keeper, other)) = List::of(edge) {
            if self.keeper.is_none() {
                self.keeper = Some(keeper);
            } else if self.keepers.last().or(self.keeper.as_ref()) != Some(&keeper) {
                self.keepers.push(keeper);
            }
            #[cfg(debug_assertions)]
            self.links.push((keeper, list, other, edge.relation));
        }
    }

    /// Whether departure removed any relation.
    pub(super) fn any(&self) -> bool {
        self.removed > 0
    }
}

impl Relations {
    /// The lists of the relations `edges` holds, numbered in their order.
    pub(super) fn new(edges: &[Edge]) -> Self {
        let mut relations = Self::default();
        relations.add(edges);
        relations
    }

    /// Adds `edges`: the relations added to the graph since these last held
    /// all of it, in graph order.
    pub(super) fn add(&mut self, edges: &[Edge]) {
        if edges.len() >= BULK {
            self.add_all(edges);
            return;
        }
        let mut rest = edges;
        while let Some(edge) = rest.first() {
            let Some((list, keeper, _)) = List::of(edge) else {
                self.number(edge);
                rest = &rest[1..];
                continue;
            };
            // The relations from here on that one record keeps in one list
            // go in together, as a commitment's basis is added at once.
            let run = rest
                .iter()
                .take_while(|edge| {
                    List::of(edge).is_some_and(|(at, by, _)| at == list && by == keeper)
                })
                .count();
            let (adding, after) = rest.split_at(run);
            let first = self.next;
            for edge in adding {
                self.number(edge);
            }
            let links = adding.iter().zip(first..).map(|(edge, serial)| {
                let (list, _, other) = List::of(edge).expect("each relation of the run is kept");
                Link {
                    serial,
                    other,
                    relation: edge.relation,
                    list,
                }
            });
            let (at, slot) = place(keeper);
            let chunk = Arc::make_mut(
                self.chunks
                    .entry(at)
                    .or_insert_with(|| Arc::new(Chunk::empty())),
            );
            // Each is newer than every link the record keeps, so they end
            // their list.
            let end = chunk.list(slot, list).end;
            chunk.links.extend(links);
            moved(chunk.links.len() - end);
            chunk.links[end..].rotate_right(run);
            chunk.grown(slot, run);
            rest = after;
        }
    }

    /// `add` for many relations at once, as a program loads and a restore
    /// adds them: each chunk they go to is written once, each of its lists
    /// followed by the links added to it. Run by run, each run would move
    /// the rest of its chunk, once for each relation when the records that
    /// keep them take turns.
    fn add_all(&mut self, edges: &[Edge]) {
        let mut added = Vec::with_capacity(edges.len());
        for edge in edges {
            let serial = self.next;
            self.number(edge);
            if let Some((list, keeper, other)) = List::of(edge) {
                let (at, slot) = place(keeper);
                let link = Link {
                    serial,
                    other,
                    relation: edge.relation,
                    list,
                };
                added.push((at, slot, link));
            }
        }
        // A stable sort, so each list's links stay in graph order.
        added.sort_by_key(|&(at, slot, link)| (at, slot, link.list));
        for group in added.chunk_by(|one, other| one.0 == other.0) {
            let chunk = self
                .chunks
                .entry(group[0].0)
                .or_insert_with(|| Arc::new(Chunk::empty()));
            let merged = chunk.merged(group);
            *chunk = Arc::new(merged);
        }
    }

    /// Counts and numbers a relation added.
    fn number(&mut self, edge: &Edge) {
        self.next += 1;
        self.held += 1;
        if edge.relation == Relation::Qualifies {
            self.qualifies += 1;
        }
    }

    /// Removes the relations departure removed, as `departed` noted them:
    /// the links of the records that keep them that touch a record
    /// `departing` names, which are all a departing record keeps, and a
    /// staying record's links to departing ones. No other record keeps one.
    /// Each chunk that keeps one is compacted in one pass from its first
    /// record that loses links, which moves each link after that record
    /// once, where removing them one by one would move the rest of the
    /// chunk for each. Debug builds check that the links removed are those
    /// of the relations noted.
    pub(super) fn depart(&mut self, departed: Departed, departing: impl Fn(NodeId) -> bool) {
        self.held -= departed.removed;
        self.qualifies -= departed.qualifies;
        let mut keepers = departed.keepers;
        let keepers = if keepers.is_empty() {
            departed.keeper.as_slice()
        } else {
            keepers.extend(departed.keeper);
            keepers.sort_unstable();
            keepers.dedup();
            &keepers
        };
        // The links removed, record by record, for debug builds' check.
        #[cfg(debug_assertions)]
        let mut dropped = Vec::new();
        for records in keepers.chunk_by(|one, other| place(*one).0 == place(*other).0) {
            let (at, first) = place(records[0]);
            let Some(shared) = self.chunks.get_mut(&at) else {
                continue;
            };
            let Chunk { ends, links } = Arc::make_mut(shared);
            let held = links.len();
            // The links before the first record that loses links stay where
            // they are. Each link after it that stays moves down once, over
            // the links removed before it: those from `span` up to the next
            // record that loses links move together.
            let mut span = first
                .checked_sub(1)
                .map_or(0, |before| ends[before] as usize);
            moved(held - span);
            let (mut removed, mut next) = (0, first);
            for &id in records {
                let (_, slot) = place(id);
                let start = if slot > next {
                    ends[slot - 1] as usize
                } else {
                    span
                };
                lowered(&mut ends[next..slot], removed);
                links.copy_within(span..start, span - removed);
                let end = ends[slot] as usize;
                let mut kept = start - removed;
                // A departing record's links all go, and a staying one loses
                // its links to departing records.
                if !departing(id) {
                    for read in start..end {
                        let link = links[read];
                        if !departing(link.other) {
                            links[kept] = link;
                            kept += 1;
                        } else {
                            #[cfg(debug_assertions)]
                            dropped.push((id, link.list, link.other, link.relation));
                        }
                    }
                } else {
                    #[cfg(debug_assertions)]
                    dropped.extend(
                        links[start..end]
                            .iter()
                            .map(|link| (id, link.list, link.other, link.relation)),
                    );
                }
                removed = end - kept;
                ends[slot] = u32::try_from(kept).expect("a chunk holds fewer than 2^32 links");
                (span, next) = (end, slot + 1);
            }
            lowered(&mut ends[next..], removed);
            links.copy_within(span..held, span - removed);
            links.truncate(held - removed);
            if links.is_empty() {
                self.chunks.remove(&at);
            }
        }
        #[cfg(debug_assertions)]
        {
            // The links removed come record by record, each record's list by
            // list, and each list's in graph order, the order departure
            // noted their relations in.
            let mut noted = departed.links;
            noted.sort_by_key(|&(keeper, list, ..)| (keeper, list));
            assert_eq!(
                dropped, noted,
                "departure removes the link of each relation it removes that a list keeps, and no other"
            );
        }
    }

    /// How many of the graph's relations these hold.
    pub(super) fn held(&self) -> usize {
        self.held
    }

    /// Whether any relation is `qualifies`.
    pub(super) fn qualifies_any(&self) -> bool {
        self.qualifies > 0
    }

    /// `id`'s `list`, in graph order.
    pub(super) fn list(&self, id: NodeId, list: List) -> &[Link] {
        let (at, slot) = place(id);
        self.chunks
            .get(&at)
            .map_or(&[], |chunk| &chunk.links[chunk.list(slot, list)])
    }

    /// These are exactly the lists `edges` makes: each relation kept once,
    /// by the record that keeps it, in graph order, with numbers in graph
    /// order; each record's lists in list order, and no chunk that holds
    /// none.
    #[cfg(any(debug_assertions, test))]
    pub(super) fn check(&self, edges: &[Edge]) {
        assert_eq!(
            self.held,
            edges.len(),
            "the relation lists hold every relation"
        );
        let mut read = HashMap::<(NodeId, List), usize>::default();
        let mut last = None;
        let mut qualifies = 0;
        for edge in edges {
            if edge.relation == Relation::Qualifies {
                qualifies += 1;
            }
            let Some((list, keeper, other)) = List::of(edge) else {
                continue;
            };
            let at = read.entry((keeper, list)).or_insert(0);
            let link = self.list(keeper, list).get(*at).copied();
            assert_eq!(
                link.map(|link| (link.other, link.relation)),
                Some((other, edge.relation)),
                "{edge:?} is kept in graph order"
            );
            let serial = link.expect("checked above").serial;
            assert!(
                last.is_none_or(|last| last < serial) && serial < self.next,
                "{edge:?} is numbered in graph order"
            );
            last = Some(serial);
            *at += 1;
        }
        assert_eq!(self.qualifies, qualifies, "qualifies relations");
        for (&at, chunk) in &self.chunks {
            assert!(
                !chunk.links.is_empty(),
                "a chunk that keeps nothing is dropped"
            );
            assert!(
                chunk.ends.windows(2).all(|pair| pair[0] <= pair[1])
                    && chunk.ends[CHUNK - 1] as usize == chunk.links.len(),
                "chunk {at} ends where its links do"
            );
            for slot in 0..CHUNK {
                let id = at * CHUNK_NODES + slot as NodeId;
                let record = &chunk.links[chunk.record(slot)];
                assert!(
                    record.windows(2).all(|pair| pair[0].list <= pair[1].list)
                        && record.iter().all(|link| link.list
                            == List::place(0, link.relation, 0)
                                .expect("a kept relation has a list")
                                .0),
                    "node {id} keeps its lists in order"
                );
                for list in LISTS {
                    assert_eq!(
                        chunk.list(slot, list).len(),
                        read.get(&(id, list)).copied().unwrap_or(0),
                        "node {id} keeps only the graph's relations"
                    );
                }
            }
        }
    }
}

/// The links of two of one record's lists, together in graph order.
pub(super) fn in_graph_order<'a>(
    first: &'a [Link],
    second: &'a [Link],
) -> impl Iterator<Item = &'a Link> {
    let (mut first, mut second) = (first.iter().peekable(), second.iter().peekable());
    std::iter::from_fn(move || match (first.peek(), second.peek()) {
        (Some(one), Some(other)) if other.serial < one.serial => second.next(),
        (Some(_), _) => first.next(),
        (None, _) => second.next(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(from: NodeId, relation: Relation, to: NodeId) -> Edge {
        Edge {
            from,
            to,
            relation,
            origin: None,
        }
    }

    fn others(links: &[Link]) -> Vec<(NodeId, Relation)> {
        links
            .iter()
            .map(|link| (link.other, link.relation))
            .collect()
    }

    /// Departure's removal: every relation that touches a record `departing`
    /// names goes, in graph order, and the rest stay.
    fn depart(
        relations: &mut Relations,
        edges: &mut Vec<Edge>,
        departing: impl Fn(NodeId) -> bool,
    ) {
        let mut departed = Departed::default();
        edges.retain(|edge| {
            let gone = departing(edge.from) || departing(edge.to);
            if gone {
                departed.note(edge);
            }
            !gone
        });
        relations.depart(departed, departing);
        relations.check(edges);
    }

    #[test]
    fn each_relation_is_kept_by_the_record_a_read_asks_about() {
        let edges = [
            edge(1, Relation::Supports, 2),
            edge(3, Relation::Qualifies, 1),
            edge(1, Relation::Opposes, 4),
            edge(5, Relation::Retains, 3),
            edge(5, Relation::ReliesOn, 1),
            edge(1, Relation::Reopens, 5),
            edge(1, Relation::InContext, 2),
            edge(3, Relation::Qualifies, 1),
        ];
        let relations = Relations::new(&edges);
        relations.check(&edges);
        assert_eq!(
            others(relations.list(1, List::Stance)),
            [(2, Relation::Supports), (4, Relation::Opposes)]
        );
        // Each relation is kept once, so a repeated one is kept twice.
        assert_eq!(
            others(relations.list(1, List::Qualifiers)),
            [(3, Relation::Qualifies), (3, Relation::Qualifies)]
        );
        assert_eq!(
            others(relations.list(5, List::Basis)),
            [(3, Relation::Retains), (1, Relation::ReliesOn)]
        );
        assert_eq!(
            others(relations.list(5, List::Reopeners)),
            [(1, Relation::Reopens)]
        );
        // Nothing keeps a relation from the end no read asks about.
        assert!(relations.list(2, List::Stance).is_empty());
        assert!(relations.list(3, List::Qualifiers).is_empty());
        assert!(relations.list(9, List::Stance).is_empty());
        assert!(relations.qualifies_any());
        assert_eq!(relations.held(), edges.len());
    }

    #[test]
    fn relations_added_together_keep_graph_order_in_each_list() {
        // A commitment's basis comes at once, between other relations of the
        // same records and of their neighbors in the chunk.
        let mut edges = vec![
            edge(2, Relation::Reopens, 5),
            edge(4, Relation::Supports, 1),
        ];
        let mut relations = Relations::new(&edges);
        let added = [
            edge(5, Relation::Retains, 1),
            edge(5, Relation::Retains, 2),
            edge(5, Relation::ReliesOn, 3),
            edge(6, Relation::Opposes, 1),
            edge(5, Relation::Retains, 4),
            edge(3, Relation::Qualifies, 5),
        ];
        relations.add(&added);
        edges.extend(added);
        relations.check(&edges);
        assert_eq!(
            others(relations.list(5, List::Basis)),
            [
                (1, Relation::Retains),
                (2, Relation::Retains),
                (3, Relation::ReliesOn),
                (4, Relation::Retains),
            ]
        );
        assert_eq!(
            others(relations.list(5, List::Qualifiers)),
            [(3, Relation::Qualifies)]
        );
        assert_eq!(
            others(relations.list(5, List::Reopeners)),
            [(2, Relation::Reopens)]
        );
        assert_eq!(
            others(relations.list(4, List::Stance)),
            [(1, Relation::Supports)]
        );
        assert_eq!(
            others(relations.list(6, List::Stance)),
            [(1, Relation::Opposes)]
        );
    }

    #[test]
    fn two_lists_read_together_come_in_graph_order() {
        let edges = [
            edge(5, Relation::Retains, 3),
            edge(1, Relation::Reopens, 5),
            edge(5, Relation::ReliesOn, 1),
            edge(2, Relation::Reopens, 5),
            edge(2, Relation::Reopens, 5),
            edge(5, Relation::Retains, 4),
        ];
        let relations = Relations::new(&edges);
        let order = in_graph_order(
            relations.list(5, List::Basis),
            relations.list(5, List::Reopeners),
        )
        .map(|link| (link.other, link.relation))
        .collect::<Vec<_>>();
        assert_eq!(
            order,
            [
                (3, Relation::Retains),
                (1, Relation::Reopens),
                (1, Relation::ReliesOn),
                (2, Relation::Reopens),
                (2, Relation::Reopens),
                (4, Relation::Retains),
            ]
        );
        assert_eq!(
            in_graph_order(&[], relations.list(5, List::Reopeners)).count(),
            3
        );
        assert_eq!(
            in_graph_order(relations.list(5, List::Basis), &[]).count(),
            3
        );
    }

    #[test]
    fn departure_removes_the_relations_of_departing_records_and_keeps_the_order_of_the_rest() {
        let chunk = CHUNK_NODES;
        let declared = vec![
            edge(1, Relation::Supports, 2),
            edge(chunk + 1, Relation::Qualifies, 1),
            edge(3, Relation::Qualifies, 1),
            edge(chunk + 1, Relation::Supports, 2),
            edge(3, Relation::Qualifies, chunk + 2),
            edge(chunk + 1, Relation::Qualifies, 1),
            edge(4, Relation::Qualifies, 1),
        ];
        let before = Relations::new(&declared);
        let mut relations = before.clone();
        let mut edges = declared.clone();
        // Records `chunk + 1` and `chunk + 2` depart, with every relation
        // that touches one: record 1 loses both relations from `chunk + 1`
        // and keeps the others in order.
        depart(&mut relations, &mut edges, |id| {
            id == chunk + 1 || id == chunk + 2
        });
        assert_eq!(
            others(relations.list(1, List::Qualifiers)),
            [(3, Relation::Qualifies), (4, Relation::Qualifies)]
        );
        // Their chunk kept only what departed, so it is dropped.
        assert!(!relations.chunks.contains_key(&1));
        // A later relation is numbered after every one before.
        let added = edge(5, Relation::Qualifies, 1);
        relations.add(std::slice::from_ref(&added));
        edges.push(added);
        relations.check(&edges);
        assert_eq!(
            others(relations.list(1, List::Qualifiers)),
            [
                (3, Relation::Qualifies),
                (4, Relation::Qualifies),
                (5, Relation::Qualifies)
            ]
        );
        // The copy taken before is untouched.
        before.check(&declared);
        // Every relation into a record departing departs: nothing qualifies.
        let mut emptied = relations.clone();
        depart(&mut emptied, &mut edges, |id| id == 1);
        assert!(edges.is_empty());
        assert!(!emptied.qualifies_any());
        assert!(emptied.chunks.is_empty());
    }

    #[test]
    fn a_copy_shares_every_chunk_it_does_not_write() {
        let chunk = CHUNK_NODES;
        let edges = [
            edge(1, Relation::Supports, 2),
            edge(chunk + 1, Relation::Supports, 2),
        ];
        let before = Relations::new(&edges);
        let mut after = before.clone();
        after.add(&[edge(chunk + 3, Relation::Opposes, 2)]);
        assert!(Arc::ptr_eq(&before.chunks[&0], &after.chunks[&0]));
        assert!(!Arc::ptr_eq(&before.chunks[&1], &after.chunks[&1]));
        assert!(before.list(chunk + 3, List::Stance).is_empty());
        assert_eq!(after.list(chunk + 3, List::Stance).len(), 1);
        assert_eq!(after.list(chunk + 1, List::Stance).len(), 1);
    }

    #[test]
    #[should_panic(expected = "is kept in graph order")]
    fn the_check_finds_a_relation_the_lists_lack() {
        let edges = [
            edge(1, Relation::Supports, 2),
            edge(3, Relation::Qualifies, 1),
        ];
        let mut relations = Relations::new(&edges[..1]);
        relations.held = edges.len();
        relations.check(&edges);
    }

    /// Lists numbered, counted and laid out as `edges` makes them, but with
    /// one thing wrong that `check` looks for.
    fn wrong(edges: &[Edge], spoil: impl FnOnce(&mut Relations)) {
        let mut relations = Relations::new(edges);
        relations.check(edges);
        spoil(&mut relations);
        relations.check(edges);
    }

    fn chunk_of(relations: &mut Relations, id: NodeId) -> &mut Chunk {
        Arc::make_mut(
            relations
                .chunks
                .get_mut(&place(id).0)
                .expect("the record keeps a relation"),
        )
    }

    #[test]
    #[should_panic(expected = "the relation lists hold every relation")]
    fn the_check_finds_lists_that_count_a_relation_the_graph_lacks() {
        wrong(&[edge(1, Relation::Supports, 2)], |relations| {
            relations.held += 1;
        });
    }

    #[test]
    #[should_panic(expected = "is numbered in graph order")]
    fn the_check_finds_two_lists_of_a_record_numbered_out_of_graph_order() {
        // Each list holds the right relation, but read together the
        // reopening would come before the basis it followed.
        wrong(
            &[edge(5, Relation::Retains, 3), edge(1, Relation::Reopens, 5)],
            |relations| {
                for link in &mut chunk_of(relations, 5).links {
                    link.serial = 1 - link.serial;
                }
            },
        );
    }

    #[test]
    #[should_panic(expected = "qualifies relations")]
    fn the_check_finds_a_wrong_count_of_qualifies_relations() {
        wrong(&[edge(3, Relation::Qualifies, 1)], |relations| {
            relations.qualifies = 0;
        });
    }

    #[test]
    #[should_panic(expected = "a chunk that keeps nothing is dropped")]
    fn the_check_finds_a_chunk_that_keeps_nothing() {
        wrong(&[edge(1, Relation::Supports, 2)], |relations| {
            relations.chunks.insert(1, Arc::new(Chunk::empty()));
        });
    }

    #[test]
    #[should_panic(expected = "chunk 0 ends where its links do")]
    fn the_check_finds_a_link_no_record_keeps() {
        wrong(&[edge(1, Relation::Supports, 2)], |relations| {
            let chunk = chunk_of(relations, 1);
            let stray = chunk.links[0];
            chunk.links.push(stray);
        });
    }

    #[test]
    #[should_panic(expected = "node 1 keeps its lists in order")]
    fn the_check_finds_a_link_past_a_later_list_that_no_read_reaches() {
        // A link after a later list is in none of the record's lists: each
        // list reads what the graph holds, and only their order shows it.
        let edges = [
            edge(1, Relation::Supports, 2),
            edge(1, Relation::Retains, 3),
        ];
        wrong(&edges, |relations| {
            let chunk = chunk_of(relations, 1);
            let stray = Link {
                other: 4,
                ..chunk.links[0]
            };
            chunk.links.push(stray);
            chunk.grown(1, 1);
        });
    }

    #[test]
    #[should_panic(expected = "node 1 keeps only the graph's relations")]
    fn the_check_finds_a_link_to_a_relation_the_graph_lacks() {
        wrong(&[edge(1, Relation::Supports, 2)], |relations| {
            let chunk = chunk_of(relations, 1);
            chunk.links.push(Link {
                serial: 1,
                other: 3,
                relation: Relation::Opposes,
                list: List::Stance,
            });
            chunk.grown(1, 1);
        });
    }

    /// Departure of `removed`, relations the lists may not keep, with the
    /// records `departing` names.
    #[cfg(debug_assertions)]
    fn depart_unkept(
        relations: &mut Relations,
        removed: &[Edge],
        departing: impl Fn(NodeId) -> bool,
    ) {
        let mut departed = Departed::default();
        for edge in removed {
            departed.note(edge);
        }
        relations.depart(departed, departing);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "departure removes the link of each relation it removes")]
    fn departing_a_relation_a_staying_record_does_not_keep_is_a_desync() {
        // Record 1 keeps `1 supports 2`, not the `1 supports 3` that
        // departure removes, as an edge written without `added` would be.
        let mut relations = Relations::new(&[edge(1, Relation::Supports, 2)]);
        depart_unkept(&mut relations, &[edge(1, Relation::Supports, 3)], |id| {
            id == 3
        });
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "departure removes the link of each relation it removes")]
    fn departing_a_relation_of_a_staying_record_that_keeps_none_is_a_desync() {
        let mut relations = Relations::new(&[edge(CHUNK_NODES + 1, Relation::Supports, 2)]);
        depart_unkept(&mut relations, &[edge(1, Relation::Supports, 3)], |id| {
            id == 3
        });
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "departure removes the link of each relation it removes")]
    fn departing_a_relation_whose_record_keeps_another_to_the_same_record_is_a_desync() {
        // Record 1 keeps `1 opposes 3`, and departure removes `1 supports 3`:
        // one link goes for one relation, but not that relation's.
        let mut relations = Relations::new(&[edge(1, Relation::Opposes, 3)]);
        depart_unkept(&mut relations, &[edge(1, Relation::Supports, 3)], |id| {
            id == 3
        });
    }

    /// `count` relations of every kind, at random, among the records of
    /// `span` ids.
    fn random_edges(random: &mut impl FnMut(u64) -> u64, count: u64, span: NodeId) -> Vec<Edge> {
        const KINDS: [Relation; 7] = [
            Relation::Supports,
            Relation::Opposes,
            Relation::Retains,
            Relation::ReliesOn,
            Relation::Qualifies,
            Relation::Reopens,
            Relation::InContext,
        ];
        (0..count)
            .map(|_| {
                let relation = KINDS[random(KINDS.len() as u64) as usize];
                edge(random(span), relation, random(span))
            })
            .collect()
    }

    #[test]
    fn relations_added_at_once_leave_the_lists_adding_them_one_at_a_time_leaves() {
        // On fresh lists, on lists a copy shares, and after a departure,
        // which leaves exactly the graph's relations.
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut random = move |below: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % below
        };
        let same = |one: &Relations, other: &Relations| {
            one.chunks == other.chunks
                && (one.held, one.qualifies, one.next) == (other.held, other.qualifies, other.next)
        };
        for round in 0..1000 {
            // Few records, whose lists share a chunk, or more over several.
            let span = [6, 16, 17, 40, 120][round % 5];
            let count = random(90);
            let before = random_edges(&mut random, count, span);
            let count = BULK as u64 + random(300);
            let batch = random_edges(&mut random, count, span);
            let mut one = Relations::default();
            for edge in before.iter().chain(&batch) {
                one.add(std::slice::from_ref(edge));
            }
            let mut all = Relations::new(&before);
            let shared = all.clone();
            all.add(&batch);
            let mut edges = [before.clone(), batch].concat();
            all.check(&edges);
            assert!(same(&one, &all), "round {round}: adding at once");
            shared.check(&before);
            // One or two records depart, or about a quarter of them.
            let mut gone = 0u128;
            if round % 2 == 0 {
                gone |= (1 << random(span)) | (1 << random(span));
            } else {
                for id in 0..span {
                    if random(4) == 0 {
                        gone |= 1 << id;
                    }
                }
            }
            let departing = |id: NodeId| gone & (1 << id) != 0;
            let mut kept = edges.clone();
            depart(&mut one, &mut kept, departing);
            depart(&mut all, &mut edges, departing);
            assert!(same(&one, &all), "round {round}: departing");
            let count = random(70);
            let later = random_edges(&mut random, count, span);
            for edge in &later {
                one.add(std::slice::from_ref(edge));
            }
            all.add(&later);
            edges.extend(later);
            all.check(&edges);
            assert!(same(&one, &all), "round {round}: adding after departing");
        }
    }

    /// The sixteen records of chunk 0, each opposing `per` records of its
    /// own, and record 2 reopened by `per` more: round by round when
    /// `interleaved`, so that each round adds a link to every list, or else
    /// record by record.
    fn full_chunk(per: NodeId, interleaved: bool) -> Vec<Edge> {
        let stance = |keeper: NodeId, n: NodeId| {
            edge(keeper, Relation::Opposes, 1_000_000 + keeper * per + n)
        };
        let reopens = |n: NodeId| edge(2_000_000 + n, Relation::Reopens, 2);
        if interleaved {
            (0..per)
                .flat_map(|n| {
                    (0..CHUNK_NODES)
                        .map(move |keeper| stance(keeper, n))
                        .chain([reopens(n)])
                })
                .collect()
        } else {
            (0..CHUNK_NODES)
                .flat_map(|keeper| (0..per).map(move |n| stance(keeper, n)))
                .chain((0..per).map(reopens))
                .collect()
        }
    }

    #[test]
    fn a_departure_that_empties_most_of_a_full_chunk_keeps_the_rest_in_order() {
        let declared = full_chunk(500, true);
        let numbers = declared
            .iter()
            .zip(0..)
            .map(|(edge, serial)| ((edge.from, edge.to), serial))
            .collect::<HashMap<_, u64>>();
        let mut edges = declared.clone();
        let mut relations = Relations::new(&edges);
        // Record 5 departs, and 450 of each record's 500 others with it.
        let departing = |id: NodeId| {
            id == 5 || ((1_000_000..2_000_000).contains(&id) && (id - 1_000_000) % 500 < 450)
        };
        depart(&mut relations, &mut edges, departing);
        for keeper in 0..CHUNK_NODES {
            let stance = relations.list(keeper, List::Stance);
            if keeper == 5 {
                assert!(stance.is_empty());
                continue;
            }
            let rest = (450..500)
                .map(|n| (1_000_000 + keeper * 500 + n, Relation::Opposes))
                .collect::<Vec<_>>();
            assert_eq!(others(stance), rest, "record {keeper}");
            // Each keeps the number its relation was added with.
            for link in stance {
                assert_eq!(
                    link.serial,
                    numbers[&(keeper, link.other)],
                    "record {keeper}"
                );
            }
        }
        assert_eq!(relations.list(2, List::Reopeners).len(), 500);
        assert_eq!(relations.held(), edges.len());
    }

    /// How many links the writes to the lists in `write` move or copy
    /// within their chunks.
    fn moved_by(write: impl FnOnce()) -> usize {
        MOVED.with(|moved| moved.set(0));
        write();
        MOVED.with(std::cell::Cell::get)
    }

    #[test]
    fn relations_whose_records_take_turns_are_added_in_one_pass() {
        // The sixteen records of chunk 0, each qualified in turn, as a long
        // session that flags them in turn leaves them. Added run by run,
        // each relation would move the rest of the chunk.
        let edges = (0..4_000)
            .map(|n| edge(100, Relation::Qualifies, n % CHUNK_NODES))
            .collect::<Vec<_>>();
        let mut relations = Relations::default();
        let moved = moved_by(|| relations.add(&edges));
        relations.check(&edges);
        assert!(
            moved <= 2 * edges.len(),
            "adding {} relations moved {moved} links",
            edges.len()
        );
    }

    #[test]
    fn an_add_writes_a_chunk_once_from_as_many_relations_as_bulk_and_run_by_run_below() {
        // The sixteen records of chunk 0, each qualified 250 times.
        let turn = |n: NodeId| edge(100, Relation::Qualifies, n % CHUNK_NODES);
        let mut edges = (0..4_000).map(turn).collect::<Vec<_>>();
        let mut relations = Relations::new(&edges);
        let held = edges.len();
        // `BULK` more, the records taking turns: the chunk is written once.
        // Run by run, each would move the links of the records after its
        // own, about 60,000 in all.
        let more = (0..BULK as NodeId).map(turn).collect::<Vec<_>>();
        edges.extend_from_slice(&more);
        let moved = moved_by(|| relations.add(&more));
        relations.check(&edges);
        assert!(
            moved <= 2 * held,
            "adding {BULK} relations to {held} moved {moved} links"
        );
        // One fewer, all of the chunk's last record, which no link follows:
        // only they move. Added as many are, they would write the chunk.
        let few = vec![turn(CHUNK_NODES - 1); BULK - 1];
        edges.extend_from_slice(&few);
        let moved = moved_by(|| relations.add(&few));
        relations.check(&edges);
        assert_eq!(moved, BULK - 1);
    }

    #[test]
    fn a_departure_moves_each_link_of_the_chunks_it_touches_once() {
        // The sixteen records of chunk 0 lose all but the last of their
        // others at once. Removed one by one, each relation would move the
        // rest of the chunk.
        let per = 256;
        let mut edges = full_chunk(per, false);
        let mut relations = Relations::new(&edges);
        let held = relations.held();
        let departing =
            |id: NodeId| (1_000_000..2_000_000).contains(&id) && (id - 1_000_000) % per + 1 < per;
        let moved = moved_by(|| depart(&mut relations, &mut edges, departing));
        assert_eq!(edges.len(), CHUNK + per as usize);
        assert!(
            moved <= 2 * held,
            "a departure from {held} relations moved {moved} links"
        );
    }

    #[test]
    fn a_restore_adds_the_relations_of_records_qualified_in_turn_in_one_pass() {
        // Sixteen evidence records, each qualified and taken back in turn,
        // which leaves a `qualifies` relation each time. Added one by one,
        // each relation would move the rest of its chunk.
        let mut source = String::from("claim seen;\ncaveat doubt consequence low;\n");
        for k in 0..16 {
            source += &format!("evidence e{k} from \"evidence {k}\";\n");
        }
        source += "event start;\nevent flag k min 0 max 15;\nevent unflag k min 0 max 15;\n";
        for k in 0..16 {
            source += &format!("on start reveal e{k} supports seen;\n");
            source += &format!("on flag when k == {k} qualify e{k} with doubt;\n");
            source += &format!("on unflag when k == {k} unqualify e{k} with doubt;\n");
        }
        let mut session = ReactiveSession::from_source(&source).unwrap();
        session.dispatch_json("start", "{}").unwrap();
        for turn in 0..400 {
            let payload = format!("{{\"k\": {}}}", turn % 16);
            session.dispatch_json("flag", &payload).unwrap();
            session.dispatch_json("unflag", &payload).unwrap();
        }
        let save = session.save_json().unwrap();
        let relations = session.graph.edges.len();
        assert_eq!(relations, 16 + 400);
        // A restore starts from the program as it loaded.
        ReactiveSession::from_source_cached(&source).unwrap();
        let mut restored = None;
        let moved = moved_by(|| {
            restored = Some(ReactiveSession::restore_json(&source, &save).unwrap());
        });
        assert_eq!(restored.unwrap().save_json().unwrap(), save);
        assert!(
            moved <= 2 * relations,
            "restoring {relations} relations moved {moved} links"
        );
    }
}
