//! An ordered map that a transaction copies in constant time.
//!
//! A session copies itself before each event so a refusal leaves it untouched.
//! An ordered map of every record was then copied whole by the first write of
//! an event, key by key and value by value. Here the entries sit in blocks of
//! at most 32, in the order of their keys, under a tree of blocks that say
//! which keys each block below them holds. A copy shares every block; a write
//! copies only the blocks on the way down to the entry it changes, and the
//! values in them are shared rather than copied.
//!
//! A copied block copies its keys, so a key should be cheap to copy: a number,
//! or text behind an `Arc`.
//!
//! Blocks are split when they fill and dropped when they empty. A split
//! leaves each half at least half full, except that an entry past the last
//! key of the whole map starts a block of its own, so keys that only grow fill
//! each block. Blocks are not merged, and the tree loses a level only when its
//! root is left with one block below it, so a map that has lost most of its
//! entries can keep small blocks, and more levels than its size needs, until
//! new entries fill them.
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::sync::Arc;

/// Most entries of a leaf, and most blocks below a branch.
const WIDTH: usize = 32;

enum Block<K, V> {
    Branch(Branch<K, V>),
    /// Entries in the order of their keys.
    Leaf(Vec<(K, Arc<V>)>),
}

struct Branch<K, V> {
    /// One bound between each two neighbours of `below`: `least[i]` is greater
    /// than every key under `below[i]` and no greater than any key under
    /// `below[i + 1]`.
    least: Vec<K>,
    below: Vec<Arc<Block<K, V>>>,
}

/// A copy with room for one more, since a block is copied to be written to.
fn roomy<T: Clone>(values: &[T]) -> Vec<T> {
    let mut copy = Vec::with_capacity(values.len() + 1);
    copy.extend_from_slice(values);
    copy
}

/// Room in a block for one more, which never takes it past `WIDTH + 1`: a
/// block holds at most `WIDTH` and one more only until it is split. Growing
/// by itself would double a full block's room.
fn make_room<T>(values: &mut Vec<T>) {
    if values.len() == values.capacity() {
        let room = (values.len() * 2).clamp(4, WIDTH + 1);
        values.reserve_exact(room.saturating_sub(values.len()).max(1));
    }
}

// Not derived: a copy of a block shares its values, so `V` need not be `Clone`.
impl<K: Clone, V> Clone for Block<K, V> {
    fn clone(&self) -> Self {
        match self {
            Block::Branch(branch) => Block::Branch(Branch {
                least: roomy(&branch.least),
                below: roomy(&branch.below),
            }),
            Block::Leaf(entries) => Block::Leaf(roomy(entries)),
        }
    }
}

impl<K, V> Block<K, V> {
    fn is_empty(&self) -> bool {
        match self {
            Block::Branch(branch) => branch.below.is_empty(),
            Block::Leaf(entries) => entries.is_empty(),
        }
    }
}

impl<K, V> Branch<K, V> {
    /// Which block below holds a key, where `compare` orders a key of the map
    /// against the key looked for.
    fn toward(&self, compare: &impl Fn(&K) -> Ordering) -> usize {
        self.least
            .partition_point(|bound| compare(bound) != Ordering::Greater)
    }
}

pub struct OrdMap<K, V> {
    root: Option<Arc<Block<K, V>>>,
    len: usize,
}

impl<K, V> Default for OrdMap<K, V> {
    fn default() -> Self {
        Self { root: None, len: 0 }
    }
}

impl<K, V> Clone for OrdMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            len: self.len,
        }
    }
}

impl<K, V> OrdMap<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The entry under `block` whose key `compare` calls equal.
    #[inline(always)]
    fn find<'a>(
        mut block: &'a Block<K, V>,
        compare: &impl Fn(&K) -> Ordering,
    ) -> Option<(&'a K, &'a V)> {
        loop {
            match block {
                Block::Branch(branch) => block = &branch.below[branch.toward(compare)],
                Block::Leaf(entries) => {
                    let at = entries.binary_search_by(|(key, _)| compare(key)).ok()?;
                    let (key, value) = &entries[at];
                    return Some((key, value));
                }
            }
        }
    }

    /// The entry whose key `compare` calls equal, where `compare` orders a key
    /// of the map against the key looked for and agrees with the map's order.
    #[inline(always)]
    pub fn get_by(&self, compare: impl Fn(&K) -> Ordering) -> Option<(&K, &V)> {
        Self::find(self.root.as_deref()?, &compare)
    }

    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get_by(|own| own.borrow().cmp(key))
            .map(|(_, value)| value)
    }

    #[inline]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }

    /// The entries in the order of their keys.
    pub fn iter(&self) -> Iter<'_, K, V> {
        let mut entries = Iter {
            leaf: [].iter(),
            path: Vec::new(),
            left: self.len,
        };
        if let Some(root) = self.root.as_deref() {
            entries.descend(root);
        }
        entries
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> + '_ {
        self.iter().map(|(key, _)| key)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.iter().map(|(_, value)| value)
    }

    pub fn first_key_value(&self) -> Option<(&K, &V)> {
        self.iter().next()
    }

    pub fn last_key_value(&self) -> Option<(&K, &V)> {
        let mut block = self.root.as_deref()?;
        loop {
            match block {
                Block::Branch(branch) => block = branch.below.last()?,
                Block::Leaf(entries) => {
                    return entries.last().map(|(key, value)| (key, &**value));
                }
            }
        }
    }

    /// Whether the two maps are one tree, which a copy and its original are
    /// until either is written. Two maps for which this holds have the same
    /// entries.
    pub fn shares_all(&self, other: &Self) -> bool {
        match (&self.root, &other.root) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }
}

/// What an insertion below hands up: the value it replaced, and the upper
/// half of a block that filled, with the least key that half can hold.
type Inserted<K, V> = (Option<Arc<V>>, Option<(K, Arc<Block<K, V>>)>);

impl<K: Ord + Clone, V> OrdMap<K, V> {
    /// `last` says whether `block` is the last of its level, so that an
    /// entry at its end is past every key of the map.
    fn insert_below(
        block: &mut Arc<Block<K, V>>,
        compare: &impl Fn(&K) -> Ordering,
        key: impl FnOnce() -> K,
        value: Arc<V>,
        last: bool,
    ) -> Inserted<K, V> {
        match Arc::make_mut(block) {
            Block::Leaf(entries) => match entries.binary_search_by(|(own, _)| compare(own)) {
                Ok(at) => (Some(std::mem::replace(&mut entries[at].1, value)), None),
                Err(at) => {
                    let key = key();
                    debug_assert!(
                        compare(&key) == Ordering::Equal,
                        "a key made for an insertion is the key looked for"
                    );
                    make_room(entries);
                    entries.insert(at, (key, value));
                    if entries.len() <= WIDTH {
                        return (None, None);
                    }
                    // Keys that only grow fill each leaf: an entry past the
                    // map's last key starts the next leaf alone. Anywhere
                    // else, keys may yet come between, and an entry that
                    // started a leaf alone would leave a leaf of one for
                    // each key that comes falling in after it.
                    let keep = if last && at == WIDTH {
                        WIDTH
                    } else {
                        entries.len() / 2
                    };
                    let upper = entries.split_off(keep);
                    let least = upper[0].0.clone();
                    (None, Some((least, Arc::new(Block::Leaf(upper)))))
                }
            },
            Block::Branch(branch) => {
                let at = branch.toward(compare);
                let below_last = last && at + 1 == branch.below.len();
                let (old, split) =
                    Self::insert_below(&mut branch.below[at], compare, key, value, below_last);
                let Some((least, upper)) = split else {
                    return (old, None);
                };
                make_room(&mut branch.least);
                make_room(&mut branch.below);
                branch.least.insert(at, least);
                branch.below.insert(at + 1, upper);
                if branch.below.len() <= WIDTH {
                    return (old, None);
                }
                let keep = if below_last && at + 1 == WIDTH {
                    WIDTH
                } else {
                    branch.below.len() / 2
                };
                let below = branch.below.split_off(keep);
                let bounds = branch.least.split_off(keep);
                let least = branch.least.pop().expect("a bound between the halves");
                let upper = Block::Branch(Branch {
                    least: bounds,
                    below,
                });
                (old, Some((least, Arc::new(upper))))
            }
        }
    }

    /// Put `value` at the key `compare` calls equal, and return the value
    /// that was there. A map that holds no such key gets `key()`, which
    /// `compare` must call equal; a key the map holds is kept, and `key` is
    /// not called.
    pub fn insert_by(
        &mut self,
        compare: impl Fn(&K) -> Ordering,
        key: impl FnOnce() -> K,
        value: V,
    ) -> Option<Arc<V>> {
        let value = Arc::new(value);
        let Some(root) = &mut self.root else {
            self.root = Some(Arc::new(Block::Leaf(vec![(key(), value)])));
            self.len = 1;
            return None;
        };
        let (old, split) = Self::insert_below(root, &compare, key, value, true);
        if let Some((least, upper)) = split {
            let lower = Arc::clone(root);
            *root = Arc::new(Block::Branch(Branch {
                least: vec![least],
                below: vec![lower, upper],
            }));
        }
        if old.is_none() {
            self.len += 1;
        }
        old
    }

    /// Put `value` at `key`, and return the value that was there, as the map
    /// held it: a copy of the map may share it, and it is not copied to be
    /// handed back.
    pub fn insert(&mut self, key: K, value: V) -> Option<Arc<V>> {
        self.insert_by(|own| own.cmp(&key), || key.clone(), value)
    }

    /// Where `block` is on the way down to the key `compare` calls equal: the
    /// place in it to go on from, or of the key in a leaf, and whether the key
    /// is now known to be under it. `None` if it is not.
    ///
    /// A key that is not in the map must leave every block shared, so a write
    /// looks before it copies: at the first block on its way that another map
    /// shares it reads on down to the key, and only then copies. The blocks
    /// above that one are this map's own, and it goes down them once.
    fn toward_held(
        block: &Arc<Block<K, V>>,
        compare: &impl Fn(&K) -> Ordering,
        known: bool,
    ) -> Option<(usize, bool)> {
        match &**block {
            Block::Leaf(entries) => entries
                .binary_search_by(|(key, _)| compare(key))
                .ok()
                .map(|at| (at, true)),
            Block::Branch(branch) => {
                let at = branch.toward(compare);
                if !known && Arc::strong_count(block) > 1 {
                    Self::find(&branch.below[at], compare)?;
                    return Some((at, true));
                }
                Some((at, known))
            }
        }
    }

    /// The place of the value at the key `compare` calls equal, to write
    /// there. This copies the blocks on the way down that another map shares,
    /// and none where the map holds no such key.
    fn slot<'a>(
        block: &'a mut Arc<Block<K, V>>,
        compare: &impl Fn(&K) -> Ordering,
        known: bool,
    ) -> Option<&'a mut Arc<V>> {
        let (at, known) = Self::toward_held(block, compare, known)?;
        match Arc::make_mut(block) {
            Block::Branch(branch) => Self::slot(&mut branch.below[at], compare, known),
            Block::Leaf(entries) => Some(&mut entries[at].1),
        }
    }

    /// The value to change at the key `compare` calls equal. This copies
    /// first the blocks on the way down that another copy shares and then,
    /// when one shares it, the value.
    pub fn get_mut_by(&mut self, compare: impl Fn(&K) -> Ordering) -> Option<&mut V>
    where
        V: Clone,
    {
        Self::slot(self.root.as_mut()?, &compare, false).map(Arc::make_mut)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        V: Clone,
    {
        self.get_mut_by(|own| own.borrow().cmp(key))
    }

    /// Put `value` over the value at the key `compare` calls equal, and say
    /// whether the map holds such a key. The value that was there is not
    /// read: a copy that shares it keeps it, and it is not copied to be
    /// written over, as `get_mut_by` would copy it.
    pub fn replace_by(&mut self, compare: impl Fn(&K) -> Ordering, value: V) -> bool {
        let Some(root) = self.root.as_mut() else {
            return false;
        };
        match Self::slot(root, &compare, false) {
            Some(slot) => {
                *slot = Arc::new(value);
                true
            }
            None => false,
        }
    }

    pub fn replace<Q>(&mut self, key: &Q, value: V) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.replace_by(|own| own.borrow().cmp(key), value)
    }

    /// Take the entry whose key `compare` calls equal out of the map. A block
    /// it leaves empty is dropped, and so is each level of the tree that has
    /// one block left. The value is handed back as the map held it.
    pub fn remove_by(&mut self, compare: impl Fn(&K) -> Ordering) -> Option<Arc<V>> {
        fn take<K: Ord + Clone, V>(
            block: &mut Arc<Block<K, V>>,
            compare: &impl Fn(&K) -> Ordering,
            known: bool,
        ) -> Option<Arc<V>> {
            let (at, known) = OrdMap::toward_held(block, compare, known)?;
            match Arc::make_mut(block) {
                Block::Branch(branch) => {
                    let value = take(&mut branch.below[at], compare, known)?;
                    if branch.below[at].is_empty() {
                        branch.below.remove(at);
                        // Its bound goes with it. The first block has none:
                        // the bound after it goes, and the block that was
                        // second holds whatever is less than the next bound.
                        if !branch.least.is_empty() {
                            branch.least.remove(at.saturating_sub(1));
                        }
                    }
                    Some(value)
                }
                Block::Leaf(entries) => Some(entries.remove(at).1),
            }
        }
        let value = take(self.root.as_mut()?, &compare, false)?;
        self.len -= 1;
        loop {
            let lower = match self.root.as_deref() {
                Some(root) if root.is_empty() => None,
                Some(Block::Branch(branch)) if branch.below.len() == 1 => {
                    branch.below.first().cloned()
                }
                _ => break,
            };
            self.root = lower;
        }
        Some(value)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<Arc<V>>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.remove_by(|own| own.borrow().cmp(key))
    }
}

/// The entries of an [`OrdMap`] in the order of their keys.
pub struct Iter<'a, K, V> {
    /// What is left to read of the leaf being read.
    leaf: std::slice::Iter<'a, (K, Arc<V>)>,
    /// The branches on the way down to that leaf, each with the place in it
    /// of the block to read next. A map of one leaf has none, so reading it
    /// allocates nothing.
    path: Vec<(&'a Branch<K, V>, usize)>,
    left: usize,
}

impl<'a, K, V> Iter<'a, K, V> {
    /// Goes down from `block` to the first leaf under it.
    fn descend(&mut self, mut block: &'a Block<K, V>) {
        loop {
            match block {
                Block::Branch(branch) => {
                    // No block is empty, so a branch has a first block.
                    let Some(first) = branch.below.first() else {
                        return;
                    };
                    self.path.push((branch, 1));
                    block = first;
                }
                Block::Leaf(entries) => {
                    self.leaf = entries.iter();
                    return;
                }
            }
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((key, value)) = self.leaf.next() {
                self.left -= 1;
                return Some((key, value));
            }
            // The leaf is read: on to the next block of the nearest branch
            // that has one.
            let below = loop {
                let (branch, next) = self.path.last_mut()?;
                match branch.below.get(*next) {
                    Some(below) => {
                        *next += 1;
                        break below;
                    }
                    None => {
                        self.path.pop();
                    }
                }
            };
            self.descend(below);
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.left, Some(self.left))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {}

impl<'a, K, V> IntoIterator for &'a OrdMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K, Q, V> std::ops::Index<&Q> for OrdMap<K, V>
where
    K: Borrow<Q>,
    Q: Ord + ?Sized,
{
    type Output = V;

    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no value at this key")
    }
}

impl<K: PartialEq, V: PartialEq> PartialEq for OrdMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && (self.shares_all(other) || self.iter().eq(other.iter()))
    }
}

impl<K: Eq, V: Eq> Eq for OrdMap<K, V> {}

impl<K: std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for OrdMap<K, V> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Ord + Clone, V: Clone> Extend<(K, V)> for OrdMap<K, V> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, entries: I) {
        for (key, value) in entries {
            self.insert(key, value);
        }
    }
}

impl<K: Ord + Clone, V: Clone> FromIterator<(K, V)> for OrdMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(entries: I) -> Self {
        let mut map = Self::default();
        map.extend(entries);
        map
    }
}

#[cfg(test)]
mod tests {
    use super::{Block, OrdMap, WIDTH};
    use std::cell::Cell;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    /// A small generator with a fixed seed, so a failure can be run again.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }

        fn below(&mut self, bound: u64) -> u64 {
            self.next() % bound
        }
    }

    /// Every block of the map, by address.
    fn blocks<K, V>(map: &OrdMap<K, V>) -> Vec<*const Block<K, V>> {
        fn walk<K, V>(block: &Arc<Block<K, V>>, out: &mut Vec<*const Block<K, V>>) {
            out.push(Arc::as_ptr(block));
            if let Block::Branch(branch) = &**block {
                for below in &branch.below {
                    walk(below, out);
                }
            }
        }
        let mut out = Vec::new();
        if let Some(root) = &map.root {
            walk(root, &mut out);
        }
        out
    }

    /// The levels of the tree: 0 for no entries, 1 for a single leaf.
    fn depth<K, V>(map: &OrdMap<K, V>) -> usize {
        let mut levels = 0;
        let mut block = map.root.as_deref();
        while let Some(here) = block {
            levels += 1;
            block = match here {
                Block::Branch(branch) => branch.below.first().map(|below| &**below),
                Block::Leaf(_) => None,
            };
        }
        levels
    }

    /// What every map must hold whatever was done to it: its count is the
    /// number of its entries, no block is empty or over-full, every leaf is at
    /// the same depth, the keys rise from first to last, each bound parts the
    /// keys of its two neighbours, and a root that is a branch has two blocks
    /// or more.
    fn check_shape<K: Ord + Clone + std::fmt::Debug, V>(map: &OrdMap<K, V>) {
        // Returns the entries below and the depth, given the bounds the keys must lie in.
        fn walk<K: Ord + Clone + std::fmt::Debug, V>(
            block: &Block<K, V>,
            least: Option<&K>,
            under: Option<&K>,
        ) -> (usize, usize) {
            assert!(!block.is_empty(), "an empty block is kept");
            match block {
                Block::Leaf(entries) => {
                    assert!(entries.len() <= WIDTH);
                    assert!(
                        entries.capacity() <= WIDTH + 1,
                        "a leaf with room for {}",
                        entries.capacity()
                    );
                    for pair in entries.windows(2) {
                        assert!(pair[0].0 < pair[1].0, "keys out of order");
                    }
                    if let Some(least) = least {
                        assert!(*least <= entries[0].0, "{least:?} over {:?}", entries[0].0);
                    }
                    if let Some(under) = under {
                        let last = &entries[entries.len() - 1].0;
                        assert!(last < under, "{last:?} not under {under:?}");
                    }
                    (entries.len(), 1)
                }
                Block::Branch(branch) => {
                    assert!(branch.below.len() <= WIDTH);
                    assert!(branch.below.capacity() <= WIDTH + 1);
                    assert!(branch.least.capacity() <= WIDTH + 1);
                    assert_eq!(branch.least.len(), branch.below.len() - 1);
                    for pair in branch.least.windows(2) {
                        assert!(pair[0] < pair[1], "bounds out of order");
                    }
                    let mut count = 0;
                    let mut depth = None;
                    for (at, below) in branch.below.iter().enumerate() {
                        let least = if at == 0 {
                            least
                        } else {
                            Some(&branch.least[at - 1])
                        };
                        let under = branch.least.get(at).or(under);
                        let (entries, levels) = walk(below, least, under);
                        count += entries;
                        assert_eq!(*depth.get_or_insert(levels), levels, "leaves at two depths");
                    }
                    (count, 1 + depth.unwrap())
                }
            }
        }
        match map.root.as_deref() {
            None => assert_eq!(map.len, 0),
            Some(root) => {
                assert_eq!(walk(root, None, None).0, map.len);
                if let Block::Branch(branch) = root {
                    assert!(branch.below.len() > 1, "a root with one block below");
                }
            }
        }
    }

    fn same<K, V>(map: &OrdMap<K, V>, model: &BTreeMap<K, V>)
    where
        K: Ord + Clone + std::fmt::Debug,
        V: PartialEq + Clone + std::fmt::Debug,
    {
        check_shape(map);
        assert_eq!(map.len(), model.len());
        assert_eq!(map.is_empty(), model.is_empty());
        assert_eq!(map.iter().len(), model.len());
        assert!(map.iter().eq(model.iter()));
        assert!(map.keys().eq(model.keys()));
        assert!(map.values().eq(model.values()));
        assert_eq!(map.first_key_value(), model.first_key_value());
        assert_eq!(map.last_key_value(), model.last_key_value());
        for (key, value) in model {
            assert_eq!(map.get(key), Some(value));
            assert_eq!(&map[key], value);
        }
        // A map built from the model alone shares nothing with this one.
        let rebuilt: OrdMap<K, V> = model
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert!(map.is_empty() || !map.shares_all(&rebuilt));
        assert!(*map == rebuilt);
    }

    /// Names as a session makes them: a few stems, each with a number.
    fn name(rng: &mut Rng, spread: u64) -> String {
        const STEMS: [&str; 5] = ["obs", "reading@", "d@", "k", "evidence_of_"];
        format!("{}{}", STEMS[rng.below(5) as usize], rng.below(spread))
    }

    #[test]
    fn it_answers_as_an_ordered_map_does_and_a_copy_keeps_what_it_had() {
        for (seed, spread, steps) in [
            (1, 12, 3_000),
            (2, 300, 5_000),
            (3, 6_000, 6_000),
            (4, 60, 4_000),
        ] {
            let mut rng = Rng(0x9e37_79b9_7f4a_7c15 ^ seed);
            let mut map: OrdMap<String, String> = OrdMap::new();
            let mut model = BTreeMap::new();
            // Copies taken on the way, each with the model of that moment.
            let mut copies: Vec<(OrdMap<String, String>, BTreeMap<String, String>)> = Vec::new();
            for step in 0..steps {
                let key = name(&mut rng, spread);
                match rng.below(10) {
                    0..=2 => {
                        let value = format!("v{step}");
                        assert_eq!(
                            map.insert(key.clone(), value.clone()).as_deref(),
                            model.insert(key, value).as_ref()
                        );
                    }
                    7 => {
                        let value = format!("r{step}");
                        let held = model
                            .get_mut(&key)
                            .map(|held| *held = value.clone())
                            .is_some();
                        assert_eq!(map.replace(key.as_str(), value), held);
                    }
                    8 => {
                        // A key the map holds is kept, and none is made for it.
                        let held = model.contains_key(&key);
                        let made = Cell::new(false);
                        let value = format!("b{step}");
                        let old = map.insert_by(
                            |own| own.as_str().cmp(key.as_str()),
                            || {
                                made.set(true);
                                key.clone()
                            },
                            value.clone(),
                        );
                        assert_eq!(made.get(), !held);
                        assert_eq!(old.as_deref(), model.insert(key, value).as_ref());
                    }
                    3 | 4 => assert_eq!(
                        map.remove(key.as_str()).as_deref(),
                        model.remove(&key).as_ref()
                    ),
                    5 => {
                        let changed = map.get_mut(key.as_str()).map(|value| {
                            value.push('+');
                            value.clone()
                        });
                        let expected = model.get_mut(&key).map(|value| {
                            value.push('+');
                            value.clone()
                        });
                        assert_eq!(changed, expected);
                    }
                    6 => {
                        assert_eq!(map.get(key.as_str()), model.get(&key));
                        assert_eq!(map.contains_key(key.as_str()), model.contains_key(&key));
                    }
                    _ => {
                        if copies.len() == 6 {
                            copies.remove(rng.below(6) as usize);
                        }
                        let copy = map.clone();
                        assert!(copy.shares_all(&map));
                        assert!(copy == map);
                        copies.push((copy, model.clone()));
                    }
                }
                // A copy equals the map exactly when nothing that was written
                // since it was taken still shows.
                for (copy, copied) in &copies {
                    assert_eq!(*copy == map, *copied == model);
                }
                if step % 211 == 0 {
                    same(&map, &model);
                    for (copy, model) in &copies {
                        same(copy, model);
                    }
                }
            }
            same(&map, &model);
            for (copy, model) in &copies {
                same(copy, model);
            }
            // Emptied in an order of its own, it gives every block back.
            let mut keys: Vec<String> = model.keys().cloned().collect();
            while !keys.is_empty() {
                let key = keys.swap_remove(rng.below(keys.len() as u64) as usize);
                assert_eq!(map.remove(key.as_str()), model.remove(&key).map(Arc::new));
                check_shape(&map);
            }
            assert!(map.root.is_none());
            assert!(map == OrdMap::new());
            for (copy, model) in &copies {
                same(copy, model);
            }
        }
    }

    #[test]
    fn a_write_copies_only_the_blocks_on_its_way_down() {
        let mut map: OrdMap<u64, u64> = (0..30_000).map(|key| (key, key)).collect();
        // Keys that only grow fill every leaf and every branch.
        assert_eq!(depth(&map), 3);
        let all = blocks(&map).len();
        assert_eq!(all, 1 + 30 + 938);
        let shared_with = |map: &OrdMap<u64, u64>, copy: &OrdMap<u64, u64>| {
            let theirs = blocks(copy);
            blocks(map)
                .iter()
                .filter(|block| theirs.contains(block))
                .count()
        };

        let copy = map.clone();
        assert!(map.shares_all(&copy));
        assert_eq!(shared_with(&map, &copy), all);

        // A key that is not there changes nothing and copies nothing.
        assert_eq!(map.get_mut(&50_000), None);
        assert_eq!(map.remove(&50_000), None);
        assert!(map.shares_all(&copy));

        // One block for each level, and no more.
        *map.get_mut(&12_345).unwrap() += 1;
        assert!(!map.shares_all(&copy));
        assert_eq!(blocks(&map).len(), all);
        assert_eq!(shared_with(&map, &copy), all - 3);
        // The blocks are now this map's own, so a second write copies none.
        let before = blocks(&map);
        assert_eq!(map.insert(12_346, 0), Some(Arc::new(12_346)));
        assert_eq!(map.remove(&12_347), Some(Arc::new(12_347)));
        assert_eq!(blocks(&map), before);
        // Another part of the tree shares the root only.
        assert_eq!(map.insert(29_999, 0), Some(Arc::new(29_999)));
        assert_eq!(shared_with(&map, &copy), all - 5);

        assert_eq!(copy.get(&12_345), Some(&12_345));
        assert_eq!(copy.get(&12_346), Some(&12_346));
        assert_eq!(copy.get(&12_347), Some(&12_347));
        assert_eq!(copy.get(&29_999), Some(&29_999));
        assert_eq!(copy.len(), 30_000);
        assert_eq!(map.len(), 29_999);
        assert!(map != copy);
        check_shape(&map);
        check_shape(&copy);
    }

    #[test]
    fn a_key_that_is_not_there_copies_no_block_wherever_it_would_be() {
        // Even keys only, so every leaf could hold a key it does not.
        let mut map: OrdMap<u64, u64> = (0..30_000).map(|key| (2 * key, key)).collect();
        assert_eq!(depth(&map), 3);
        let copy = map.clone();
        let absent = |map: &mut OrdMap<u64, u64>, key: u64| {
            let before = blocks(map);
            assert_eq!(map.get_mut(&key), None);
            assert_eq!(map.remove(&key), None);
            assert!(!map.replace(&key, 0));
            assert_eq!(blocks(map), before);
        };
        absent(&mut map, 24_691);
        assert!(map.shares_all(&copy));
        // The first write makes one block of each level the map's own: the
        // root, a branch under it and a leaf under that.
        *map.get_mut(&24_690).unwrap() += 1;
        // In that leaf; in a leaf the copy shares, under that branch; under
        // branches the copy shares, at the start and at the end.
        for key in [24_691, 24_691 + 2 * 64, 1, 59_999] {
            absent(&mut map, key);
        }
        assert_eq!(map.len(), copy.len());
        check_shape(&map);
    }

    #[test]
    fn a_write_to_blocks_of_its_own_looks_for_its_key_once() {
        let mut map: OrdMap<u64, u64> = (0..30_000).map(|key| (key, key)).collect();
        let looks = Cell::new(0);
        // Orders a key of the map against `key`, and counts the keys shown.
        let against = |key: u64| {
            let looks = &looks;
            move |own: &u64| {
                looks.set(looks.get() + 1);
                own.cmp(&key)
            }
        };
        let copy = map.clone();
        assert_eq!(map.get_by(against(12_345)), Some((&12_345, &12_345)));
        let read = looks.replace(0);
        // Under blocks a copy shares, a write reads down to its key before
        // it copies them: twice below the root, once at it.
        *map.get_mut_by(against(12_345)).unwrap() += 1;
        let shared = looks.replace(0);
        assert!(read < shared && shared < 2 * read, "{read} then {shared}");
        // The blocks on the way are now the map's own, and each write goes
        // down them as a read does.
        *map.get_mut_by(against(12_345)).unwrap() += 1;
        assert_eq!(looks.replace(0), read);
        assert!(map.replace_by(against(12_345), 0));
        assert_eq!(looks.replace(0), read);
        assert_eq!(
            map.insert_by(against(12_345), || 12_345, 1),
            Some(Arc::new(0))
        );
        assert_eq!(looks.replace(0), read);
        assert_eq!(map.remove_by(against(12_345)), Some(Arc::new(1)));
        assert_eq!(looks.replace(0), read);
        // And no more to find that a key is not there, or to put it back.
        assert_eq!(map.get_by(against(12_345)), None);
        let absent = looks.replace(0);
        assert_eq!(map.remove_by(against(12_345)), None);
        assert_eq!(looks.replace(0), absent);
        assert_eq!(map.insert_by(against(12_345), || 12_345, 2), None);
        // A debug build also checks the key made against the one looked for.
        let check = usize::from(cfg!(debug_assertions));
        assert_eq!(looks.replace(0), absent + check);
        assert_eq!(map.get(&12_345), Some(&2));
        assert_eq!(copy.get(&12_345), Some(&12_345));
        check_shape(&map);
    }

    /// A value that counts the copies made of it.
    #[derive(Debug)]
    struct Counted<'a>(u64, &'a Cell<usize>);

    impl Clone for Counted<'_> {
        fn clone(&self) -> Self {
            self.1.set(self.1.get() + 1);
            Self(self.0, self.1)
        }
    }

    #[test]
    fn a_value_written_over_is_not_copied() {
        let copies = Cell::new(0);
        let value = |number| Counted(number, &copies);
        let mut map: OrdMap<u64, Counted> = (0..100).map(|key| (key, value(key))).collect();
        let copy = map.clone();
        // Written over, the value the copy shares is neither copied nor
        // changed.
        assert!(map.replace(&7, value(700)));
        assert_eq!(copies.get(), 0);
        assert_eq!((map[&7].0, copy[&7].0), (700, 7));
        // Changed in place, it is copied once, for the copy that shares it.
        map.get_mut(&8).unwrap().0 += 1;
        assert_eq!(copies.get(), 1);
        map.get_mut(&8).unwrap().0 += 1;
        assert_eq!(copies.get(), 1);
        assert_eq!((map[&8].0, copy[&8].0), (10, 8));
        // Handed back, it is handed back as the map held it: not copied,
        // whether the copy still holds it or the map alone does.
        assert_eq!(map.insert(9, value(900)).map(|old| old.0), Some(9));
        assert_eq!(copies.get(), 1);
        assert_eq!(map.insert(9, value(901)).map(|old| old.0), Some(900));
        assert_eq!(map.remove(&7).map(|old| old.0), Some(700));
        assert_eq!(copies.get(), 1);
        assert_eq!((map.len(), copy.len()), (99, 100));
        assert_eq!(copy.values().map(|held| held.0).sum::<u64>(), 4_950);
    }

    #[test]
    fn a_window_of_keys_moving_up_keeps_few_blocks() {
        let mut map = OrdMap::new();
        let mut most = 0;
        for key in 0..100_000u64 {
            map.insert(key, key);
            if key >= 40 {
                assert_eq!(map.remove(&(key - 40)), Some(Arc::new(key - 40)));
            }
            if key % 1_009 == 0 {
                check_shape(&map);
                most = most.max(blocks(&map).len());
            }
        }
        assert_eq!(map.len(), 40);
        assert!(most <= 2 * 4, "{most} blocks");
        assert!(depth(&map) <= 3);
        assert!(map.keys().copied().eq(99_960..100_000));
    }

    #[test]
    fn keys_falling_in_above_a_full_leaf_fill_their_leaves() {
        // A full leaf, then keys past it that arrive largest first: only the
        // first of them is past the map's last key.
        let mut map: OrdMap<u64, u64> = (0..WIDTH as u64).map(|key| (key, key)).collect();
        for key in (1_000..5_000).rev() {
            map.insert(key, key);
        }
        check_shape(&map);
        let mut small = 0;
        let mut leaves = 0;
        fn walk(block: &Block<u64, u64>, small: &mut usize, leaves: &mut usize) {
            match block {
                Block::Leaf(entries) => {
                    *leaves += 1;
                    if entries.len() < WIDTH / 2 {
                        *small += 1;
                    }
                }
                Block::Branch(branch) => {
                    for below in &branch.below {
                        walk(below, small, leaves);
                    }
                }
            }
        }
        walk(map.root.as_deref().unwrap(), &mut small, &mut leaves);
        assert!(small <= 1, "{small} of {leaves} leaves under half full");
        assert!(leaves <= map.len() / (WIDTH / 2) + 1, "{leaves} leaves");
        assert!(map
            .keys()
            .copied()
            .eq((0..WIDTH as u64).chain(1_000..5_000)));
    }

    #[test]
    fn keys_are_found_by_what_they_borrow_as_and_by_an_order_of_the_callers() {
        let mut map: OrdMap<String, u64> = OrdMap::new();
        for number in 0..500u64 {
            map.insert(format!("name{number:03}"), number);
        }
        assert_eq!(map.get("name123"), Some(&123));
        assert_eq!(map["name007"], 7);
        assert_eq!(map.get("name"), None);
        assert_eq!(map.get("name500"), None);
        *map.get_mut("name123").unwrap() += 1_000;
        assert_eq!(map.remove("name124"), Some(Arc::new(124)));
        assert_eq!(map.remove("name124"), None);
        assert_eq!(map.len(), 499);
        // A pair is found without building one.
        let mut pairs: OrdMap<(u64, String), u64> = OrdMap::new();
        for number in 0..500u64 {
            pairs.insert((number % 7, format!("n{number}")), number);
        }
        let find = |hash: u64, name: &str| {
            pairs
                .get_by(|(own_hash, own_name)| {
                    own_hash
                        .cmp(&hash)
                        .then_with(|| own_name.as_str().cmp(name))
                })
                .map(|(_, number)| *number)
        };
        assert_eq!(find(3, "n3"), Some(3));
        assert_eq!(find(3, "n4"), None);
        assert_eq!(find(495 % 7, "n495"), Some(495));
        assert_eq!(
            pairs.remove_by(|(hash, name)| hash.cmp(&3).then_with(|| name.as_str().cmp("n3"))),
            Some(Arc::new(3))
        );
        assert_eq!(pairs.len(), 499);
        check_shape(&pairs);
        let small: OrdMap<&str, u64> = [("b", 2), ("a", 1)].into_iter().collect();
        assert_eq!(format!("{small:?}"), r#"{"a": 1, "b": 2}"#);
    }
}
