//! A map from numbers to values that a transaction copies in constant time.
//!
//! A session copies itself before each event so a refusal leaves it untouched.
//! A hash table of the graph's nodes was then copied whole by the first write
//! of every event, so an event cost as much as the graph was large. Here the
//! values sit in a tree of blocks of 32 slots, and each level of the tree
//! reads five more bits of the key. A copy shares every block; a write copies
//! only the blocks on the way down to the slot it changes, and the values in
//! them are shared rather than copied.
//!
//! A key keeps its slot for as long as it is in the map, so keys that are
//! never used again cost nothing once they are removed: a block with no slot
//! in use is dropped.
use std::sync::Arc;

/// Bits of the key that one level of the tree reads.
const BITS: u32 = 5;
/// Slots in one block.
const WIDTH: usize = 1 << BITS;
/// The height at which a tree has a slot for every `u64`.
const TALLEST: u32 = (u64::BITS - 1) / BITS;
/// Keys in a run: the keys that one block of the lowest level holds, which
/// agree in every bit but the lowest five.
pub const RUN: u64 = WIDTH as u64;

enum Block<V> {
    /// Above the lowest level, a slot holds the block one level down.
    Branch([Option<Arc<Block<V>>>; WIDTH]),
    /// At the lowest level, a slot holds a value.
    Leaf([Option<Arc<V>>; WIDTH]),
}

// Not derived: a copy of a block shares its values, so `V` need not be `Clone`.
impl<V> Clone for Block<V> {
    fn clone(&self) -> Self {
        match self {
            Block::Branch(slots) => Block::Branch(slots.clone()),
            Block::Leaf(slots) => Block::Leaf(slots.clone()),
        }
    }
}

impl<V> Block<V> {
    /// An empty block for `level`, where the lowest level is 0.
    fn empty(level: u32) -> Self {
        if level == 0 {
            Block::Leaf(std::array::from_fn(|_| None))
        } else {
            Block::Branch(std::array::from_fn(|_| None))
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Block::Branch(slots) => slots.iter().all(Option::is_none),
            Block::Leaf(slots) => slots.iter().all(Option::is_none),
        }
    }
}

/// The slot of `id` in a block at `level`.
fn slot(id: u64, level: u32) -> usize {
    ((id >> (BITS * level)) & (WIDTH as u64 - 1)) as usize
}

/// Whether a tree of `height` levels below its root has a slot for `id`.
fn covers(height: u32, id: u64) -> bool {
    height >= TALLEST || id >> (BITS * (height + 1)) == 0
}

pub struct IdMap<V> {
    root: Option<Arc<Block<V>>>,
    /// Levels below the root. The root of a tree of height 0 is a leaf.
    height: u32,
    len: usize,
}

impl<V> Default for IdMap<V> {
    fn default() -> Self {
        Self {
            root: None,
            height: 0,
            len: 0,
        }
    }
}

impl<V> Clone for IdMap<V> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            height: self.height,
            len: self.len,
        }
    }
}

impl<V> IdMap<V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, id: &u64) -> Option<&V> {
        self.leaf(*id)?[slot(*id, 0)].as_deref()
    }

    /// The slots of the block of the lowest level that has the slot of `id`,
    /// when the tree has that block.
    fn leaf(&self, id: u64) -> Option<&[Option<Arc<V>>; WIDTH]> {
        let mut block = self.root.as_deref()?;
        if !covers(self.height, id) {
            return None;
        }
        let mut level = self.height;
        loop {
            match block {
                Block::Branch(slots) => {
                    block = slots[slot(id, level)].as_deref()?;
                    level -= 1;
                }
                Block::Leaf(slots) => return Some(slots),
            }
        }
    }

    /// The slots of the keys of the run that holds `id`, from the multiple of
    /// [`RUN`] at or below `id` to the next, in the order of the keys, each
    /// with the value at its key if the map holds one; none when it holds no
    /// key of the run. One block holds them all, so they take one walk down
    /// the tree, where getting the values one by one takes a walk for each.
    pub fn run(&self, id: u64) -> &[Option<Arc<V>>] {
        self.leaf(id).map_or(&[], |slots| slots.as_slice())
    }

    pub fn contains_key(&self, id: &u64) -> bool {
        self.get(id).is_some()
    }

    /// The keys and values, in the order of the keys.
    pub fn iter(&self) -> Iter<'_, V> {
        Iter {
            path: self
                .root
                .as_deref()
                .map(|root| (root, 0))
                .into_iter()
                .collect(),
            left: self.len,
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = u64> + '_ {
        self.iter().map(|(id, _)| id)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.iter().map(|(_, value)| value)
    }

    /// Whether the two maps are one tree, which a copy and its original are
    /// until either is written. Two maps for which this holds have the same
    /// keys and values.
    pub fn shares_all(&self, other: &Self) -> bool {
        match (&self.root, &other.root) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }

    /// Whether `later`, read in the order of its keys, begins with this map:
    /// it holds every key of this map with an equal value, and none of its
    /// other keys comes before a key of this map. A block or a value that
    /// both maps share is not read, so a map and a copy that keys were added
    /// to are compared in time proportional to what was added.
    pub fn begins(&self, later: &Self) -> bool
    where
        V: PartialEq,
    {
        /// Reads two blocks of one level slot by slot, in key order. `past`
        /// says that `later` has shown a key this map lacks: a key of this
        /// map after that one is out of place.
        fn walk<V: PartialEq>(
            here: &Arc<Block<V>>,
            later: &Arc<Block<V>>,
            past: &mut bool,
        ) -> bool {
            if Arc::ptr_eq(here, later) {
                // No block is empty, so this one holds a key of this map.
                return !*past;
            }
            match (&**here, &**later) {
                (Block::Branch(here), Block::Branch(later)) => {
                    here.iter().zip(later).all(|slots| match slots {
                        (Some(here), Some(later)) => walk(here, later, past),
                        (Some(_), None) => false,
                        (None, later) => {
                            *past |= later.is_some();
                            true
                        }
                    })
                }
                (Block::Leaf(here), Block::Leaf(later)) => {
                    here.iter().zip(later).all(|slots| match slots {
                        (Some(here), Some(later)) => {
                            !*past && (Arc::ptr_eq(here, later) || here == later)
                        }
                        (Some(_), None) => false,
                        (None, later) => {
                            *past |= later.is_some();
                            true
                        }
                    })
                }
                _ => false,
            }
        }
        let Some(here) = &self.root else {
            return true;
        };
        // A taller tree holds a key past every key `later` has room for.
        if self.height > later.height {
            return false;
        }
        let Some(mut there) = later.root.as_ref() else {
            return false;
        };
        // The keys this tree has room for are under the first slot of each
        // level `later` has above it, and the keys under the other slots of
        // those levels all come after them.
        for _ in self.height..later.height {
            let Block::Branch(slots) = &**there else {
                return false;
            };
            let Some(below) = &slots[0] else {
                return false;
            };
            there = below;
        }
        walk(here, there, &mut false)
    }

    /// Whether `later` holds every key of this map, each with a value that
    /// `kept` answers true for, given this map's value and `later`'s. A block
    /// or a value both maps share is not read, so `kept` must answer true
    /// for a value and itself: a map and a copy an event wrote to are
    /// compared in time proportional to what the event wrote.
    pub fn within(&self, later: &Self, mut kept: impl FnMut(&V, &V) -> bool) -> bool {
        fn walk<V>(
            here: &Arc<Block<V>>,
            later: &Arc<Block<V>>,
            kept: &mut impl FnMut(&V, &V) -> bool,
        ) -> bool {
            if Arc::ptr_eq(here, later) {
                return true;
            }
            match (&**here, &**later) {
                (Block::Branch(here), Block::Branch(later)) => {
                    here.iter().zip(later).all(|slots| match slots {
                        (Some(here), Some(later)) => walk(here, later, kept),
                        (Some(_), None) => false,
                        (None, _) => true,
                    })
                }
                (Block::Leaf(here), Block::Leaf(later)) => {
                    here.iter().zip(later).all(|slots| match slots {
                        (Some(here), Some(later)) => Arc::ptr_eq(here, later) || kept(here, later),
                        (Some(_), None) => false,
                        (None, _) => true,
                    })
                }
                _ => false,
            }
        }
        let Some(here) = &self.root else {
            return true;
        };
        // A taller tree holds a key past every key `later` has room for.
        if self.height > later.height {
            return false;
        }
        let Some(mut there) = later.root.as_ref() else {
            return false;
        };
        // The keys this tree has room for are under the first slot of each
        // level `later` has above it.
        for _ in self.height..later.height {
            let Block::Branch(slots) = &**there else {
                return false;
            };
            let Some(below) = &slots[0] else {
                return false;
            };
            there = below;
        }
        walk(here, there, &mut kept)
    }

    /// Put `value` at `id`, and return the value that was there, as the map
    /// held it: a copy of the map may share it, and it is not copied to be
    /// handed back.
    pub fn insert(&mut self, id: u64, value: V) -> Option<Arc<V>> {
        let root = match &mut self.root {
            Some(root) => {
                // A taller tree keeps the one it replaces as its first block.
                while !covers(self.height, id) {
                    self.height += 1;
                    let mut taller = Block::empty(self.height);
                    if let Block::Branch(slots) = &mut taller {
                        slots[0] = Some(Arc::clone(root));
                    }
                    *root = Arc::new(taller);
                }
                root
            }
            None => {
                self.height = (0..=TALLEST)
                    .find(|height| covers(*height, id))
                    .unwrap_or(TALLEST);
                self.root.insert(Arc::new(Block::empty(self.height)))
            }
        };
        let mut block = Arc::make_mut(root);
        let mut level = self.height;
        loop {
            match block {
                Block::Branch(slots) => {
                    level -= 1;
                    let below = slots[slot(id, level + 1)]
                        .get_or_insert_with(|| Arc::new(Block::empty(level)));
                    block = Arc::make_mut(below);
                }
                Block::Leaf(slots) => {
                    let old = slots[slot(id, level)].replace(Arc::new(value));
                    if old.is_none() {
                        self.len += 1;
                    }
                    return old;
                }
            }
        }
    }

    /// The value at `id` to change. This copies first the blocks on the way
    /// down that another copy shares and then, when one shares it, the value.
    pub fn get_mut(&mut self, id: &u64) -> Option<&mut V>
    where
        V: Clone,
    {
        // A key that is not here must leave every block shared.
        if !self.contains_key(id) {
            return None;
        }
        let id = *id;
        let mut block = Arc::make_mut(self.root.as_mut()?);
        let mut level = self.height;
        loop {
            match block {
                Block::Branch(slots) => {
                    block = Arc::make_mut(slots[slot(id, level)].as_mut()?);
                    level -= 1;
                }
                Block::Leaf(slots) => return slots[slot(id, level)].as_mut().map(Arc::make_mut),
            }
        }
    }

    /// Take the value at `id` out of the map. The blocks it leaves empty are
    /// dropped, and so is each level of the tree that the keys left do not
    /// need. The value is handed back as the map held it.
    pub fn remove(&mut self, id: &u64) -> Option<Arc<V>> {
        fn take<V>(block: &mut Arc<Block<V>>, id: u64, level: u32) -> Option<Arc<V>> {
            match Arc::make_mut(block) {
                Block::Branch(slots) => {
                    let below = &mut slots[slot(id, level)];
                    let value = take(below.as_mut()?, id, level - 1);
                    if below.as_deref().is_some_and(Block::is_empty) {
                        *below = None;
                    }
                    value
                }
                Block::Leaf(slots) => slots[slot(id, level)].take(),
            }
        }
        if !self.contains_key(id) {
            return None;
        }
        let value = take(self.root.as_mut()?, *id, self.height)?;
        self.len -= 1;
        loop {
            let lower = match self.root.as_deref() {
                Some(root) if root.is_empty() => None,
                // The keys left are all below the first block of the root.
                Some(Block::Branch(slots)) if slots[1..].iter().all(Option::is_none) => {
                    slots[0].clone()
                }
                _ => break,
            };
            self.height = if lower.is_some() { self.height - 1 } else { 0 };
            self.root = lower;
        }
        Some(value)
    }
}

/// The keys and values of an [`IdMap`] in the order of the keys.
pub struct Iter<'a, V> {
    /// The blocks on the way down to the next value, each with the next of
    /// its slots to read.
    path: Vec<(&'a Block<V>, usize)>,
    left: usize,
}

impl<'a, V> Iterator for Iter<'a, V> {
    type Item = (u64, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (block, next) = self.path.last_mut()?;
            if *next == WIDTH {
                self.path.pop();
                continue;
            }
            let (block, at) = (*block, *next);
            *next += 1;
            match block {
                Block::Branch(slots) => {
                    if let Some(below) = &slots[at] {
                        self.path.push((below, 0));
                    }
                }
                Block::Leaf(slots) => {
                    if let Some(value) = &slots[at] {
                        // Each block on the way down was left one slot past
                        // the one taken, and those slots spell the key.
                        let id = self
                            .path
                            .iter()
                            .fold(0, |id, (_, next)| id << BITS | (*next as u64 - 1));
                        self.left -= 1;
                        return Some((id, value));
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.left, Some(self.left))
    }
}

impl<V> ExactSizeIterator for Iter<'_, V> {}

impl<'a, V> IntoIterator for &'a IdMap<V> {
    type Item = (u64, &'a V);
    type IntoIter = Iter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<V> std::ops::Index<&u64> for IdMap<V> {
    type Output = V;

    fn index(&self, id: &u64) -> &V {
        self.get(id).expect("no value at this key")
    }
}

impl<V: PartialEq> PartialEq for IdMap<V> {
    /// Two maps with the same keys are trees of one shape, because a tree has
    /// no empty block and is no taller than its keys need. So the blocks are
    /// compared slot by slot, and a block or a value that both maps share is
    /// not read at all: a map and the copy an event wrote to are compared in
    /// time proportional to what the event wrote.
    fn eq(&self, other: &Self) -> bool {
        fn same<V: PartialEq>(left: &Arc<Block<V>>, right: &Arc<Block<V>>) -> bool {
            if Arc::ptr_eq(left, right) {
                return true;
            }
            match (&**left, &**right) {
                (Block::Branch(left), Block::Branch(right)) => {
                    left.iter().zip(right).all(|slots| match slots {
                        (Some(left), Some(right)) => same(left, right),
                        (left, right) => left.is_none() && right.is_none(),
                    })
                }
                (Block::Leaf(left), Block::Leaf(right)) => {
                    left.iter().zip(right).all(|slots| match slots {
                        (Some(left), Some(right)) => Arc::ptr_eq(left, right) || left == right,
                        (left, right) => left.is_none() && right.is_none(),
                    })
                }
                _ => false,
            }
        }
        self.len == other.len
            && self.height == other.height
            && match (&self.root, &other.root) {
                (Some(left), Some(right)) => same(left, right),
                (left, right) => left.is_none() && right.is_none(),
            }
    }
}

impl<V: Eq> Eq for IdMap<V> {}

impl<V: std::fmt::Debug> std::fmt::Debug for IdMap<V> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_map().entries(self.iter()).finish()
    }
}

impl<V: Clone> Extend<(u64, V)> for IdMap<V> {
    fn extend<I: IntoIterator<Item = (u64, V)>>(&mut self, values: I) {
        for (id, value) in values {
            self.insert(id, value);
        }
    }
}

impl<V: Clone> FromIterator<(u64, V)> for IdMap<V> {
    fn from_iter<I: IntoIterator<Item = (u64, V)>>(values: I) -> Self {
        let mut map = Self::default();
        map.extend(values);
        map
    }
}

#[cfg(test)]
mod tests {
    use super::{Block, IdMap, TALLEST, WIDTH};
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicIsize, Ordering};
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
    fn blocks<V>(map: &IdMap<V>) -> Vec<*const Block<V>> {
        fn walk<V>(block: &Arc<Block<V>>, out: &mut Vec<*const Block<V>>) {
            out.push(Arc::as_ptr(block));
            if let Block::Branch(slots) = &**block {
                for below in slots.iter().flatten() {
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

    /// What every map must hold whatever was done to it: its count is the
    /// number of its values, no block is empty, the root is a leaf exactly at
    /// height 0, and the tree is no taller than its keys need.
    fn check_shape<V>(map: &IdMap<V>) {
        fn walk<V>(block: &Block<V>, level: u32) -> usize {
            assert!(!block.is_empty(), "an empty block is kept");
            match block {
                Block::Leaf(slots) => {
                    assert_eq!(level, 0, "a leaf above the lowest level");
                    slots.iter().flatten().count()
                }
                Block::Branch(slots) => {
                    assert!(level > 0, "a branch at the lowest level");
                    slots
                        .iter()
                        .flatten()
                        .map(|below| walk(below, level - 1))
                        .sum()
                }
            }
        }
        match map.root.as_deref() {
            None => {
                assert_eq!(map.len, 0);
                assert_eq!(map.height, 0);
            }
            Some(root) => {
                assert_eq!(walk(root, map.height), map.len);
                assert!(map.height <= TALLEST);
                if let Block::Branch(slots) = root {
                    assert!(
                        slots[1..].iter().any(Option::is_some),
                        "a root with only its first block in use"
                    );
                }
            }
        }
    }

    fn same<V: PartialEq + std::fmt::Debug + Clone>(map: &IdMap<V>, model: &BTreeMap<u64, V>) {
        check_shape(map);
        // A map built from the model alone shares nothing with this one.
        let rebuilt: IdMap<V> = model
            .iter()
            .map(|(id, value)| (*id, value.clone()))
            .collect();
        assert!(map.is_empty() || !map.shares_all(&rebuilt));
        assert_eq!(map, &rebuilt);
        assert_eq!(map.height, rebuilt.height);
        assert_eq!(map.len(), model.len());
        assert_eq!(map.is_empty(), model.is_empty());
        assert_eq!(map.iter().len(), model.len());
        let listed: Vec<(u64, V)> = map.iter().map(|(id, value)| (id, value.clone())).collect();
        let expected: Vec<(u64, V)> = model
            .iter()
            .map(|(id, value)| (*id, value.clone()))
            .collect();
        assert_eq!(listed, expected);
        assert!(map.keys().eq(model.keys().copied()));
        assert!(map.values().eq(model.values()));
        for (id, value) in model {
            assert_eq!(map.get(id), Some(value));
            assert_eq!(&map[id], value);
            assert!(map.contains_key(id));
        }
    }

    /// A key drawn so that every level of the tree, the ends of the range and
    /// the neighbours of keys already used all come up.
    fn key(rng: &mut Rng, spread: u64, used: &BTreeMap<u64, String>) -> u64 {
        match rng.below(10) {
            0 => u64::MAX - rng.below(3),
            1 => 1u64 << rng.below(64),
            2 => (1u64 << rng.below(64)).wrapping_sub(1),
            3 | 4 if !used.is_empty() => {
                let nth = rng.below(used.len() as u64) as usize;
                let near = *used.keys().nth(nth).unwrap();
                match rng.below(3) {
                    0 => near,
                    1 => near.wrapping_add(1),
                    _ => near.wrapping_sub(1),
                }
            }
            _ => rng.below(spread),
        }
    }

    #[test]
    fn it_answers_as_an_ordered_map_does_and_a_copy_keeps_what_it_had() {
        for (seed, spread, steps) in [
            (1, 40, 3_000),
            (2, 1_100, 4_000),
            (3, 40_000, 4_000),
            (4, u64::MAX, 4_000),
            (5, 70, 3_000),
        ] {
            let mut rng = Rng(0x9e37_79b9_7f4a_7c15 ^ seed);
            let mut map = IdMap::new();
            let mut model = BTreeMap::new();
            // Copies taken on the way, each with the model of that moment.
            let mut copies: Vec<(IdMap<String>, BTreeMap<u64, String>)> = Vec::new();
            for step in 0..steps {
                let id = key(&mut rng, spread, &model);
                match rng.below(8) {
                    0..=2 => {
                        let value = format!("v{step}");
                        assert_eq!(
                            map.insert(id, value.clone()).as_deref(),
                            model.insert(id, value).as_ref()
                        );
                    }
                    3 | 4 => assert_eq!(map.remove(&id).as_deref(), model.remove(&id).as_ref()),
                    5 => {
                        let changed = map.get_mut(&id).map(|value| {
                            value.push('+');
                            value.clone()
                        });
                        let expected = model.get_mut(&id).map(|value| {
                            value.push('+');
                            value.clone()
                        });
                        assert_eq!(changed, expected);
                    }
                    6 => {
                        assert_eq!(map.get(&id), model.get(&id));
                        assert_eq!(map.contains_key(&id), model.contains_key(&id));
                    }
                    _ => {
                        if copies.len() == 6 {
                            copies.remove(rng.below(6) as usize);
                        }
                        let copy = map.clone();
                        assert!(copy.shares_all(&map));
                        assert_eq!(copy, map);
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
            let mut keys: Vec<u64> = model.keys().copied().collect();
            while !keys.is_empty() {
                let id = keys.swap_remove(rng.below(keys.len() as u64) as usize);
                assert_eq!(map.remove(&id).as_deref(), model.remove(&id).as_ref());
                check_shape(&map);
            }
            assert!(map.root.is_none());
            assert_eq!(map, IdMap::new());
            for (copy, model) in &copies {
                same(copy, model);
            }
        }
    }

    #[test]
    fn a_write_copies_only_the_blocks_on_its_way_down() {
        let mut map: IdMap<u64> = (0..40_000).map(|id| (id, id)).collect();
        assert_eq!(map.height, 3);
        let all = blocks(&map).len();
        let shared_with = |map: &IdMap<u64>, copy: &IdMap<u64>| {
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
        assert_eq!(map.remove(&u64::MAX), None);
        assert!(map.shares_all(&copy));

        // One block for each level, and no more.
        *map.get_mut(&12_345).unwrap() += 1;
        assert!(!map.shares_all(&copy));
        assert_eq!(blocks(&map).len(), all);
        assert_eq!(shared_with(&map, &copy), all - 4);
        // The blocks are now this map's own, so a second write copies none.
        let before = blocks(&map);
        map.insert(12_346, 0);
        assert_eq!(map.remove(&12_347).as_deref(), Some(&12_347));
        assert_eq!(blocks(&map), before);
        // Another part of the tree shares the root only.
        map.insert(39_999, 0);
        assert_eq!(shared_with(&map, &copy), all - 7);

        assert_eq!(copy.get(&12_345), Some(&12_345));
        assert_eq!(copy.get(&12_346), Some(&12_346));
        assert_eq!(copy.get(&12_347), Some(&12_347));
        assert_eq!(copy.get(&39_999), Some(&39_999));
        assert_eq!(copy.len(), 40_000);
        assert_eq!(map.len(), 39_999);
        assert_ne!(map, copy);
    }

    #[test]
    fn a_copied_block_shares_its_values() {
        static LIVE: AtomicIsize = AtomicIsize::new(0);
        static COPIES: AtomicIsize = AtomicIsize::new(0);
        struct Counted(u64);
        impl Counted {
            fn new(value: u64) -> Self {
                LIVE.fetch_add(1, Ordering::Relaxed);
                Counted(value)
            }
        }
        impl Clone for Counted {
            fn clone(&self) -> Self {
                COPIES.fetch_add(1, Ordering::Relaxed);
                Counted::new(self.0)
            }
        }
        impl Drop for Counted {
            fn drop(&mut self) {
                LIVE.fetch_sub(1, Ordering::Relaxed);
            }
        }

        let mut map: IdMap<Counted> = (0..2_000).map(|id| (id, Counted::new(id))).collect();
        let live = || LIVE.load(Ordering::Relaxed);
        let copies = || COPIES.load(Ordering::Relaxed);
        let number = |value: Option<&Counted>| value.map(|value| value.0);
        assert_eq!(live(), 2_000);
        let copy = map.clone();
        // Adding, replacing and removing beside shared values copies none.
        map.insert(2_000, Counted::new(2_000));
        assert_eq!(number(map.insert(7, Counted::new(70)).as_deref()), Some(7));
        assert_eq!(number(map.remove(&8).as_deref()), Some(8));
        assert_eq!(copies(), 0, "what is handed back is the shared value");
        // Changing a shared value copies that one, and the copy keeps its own.
        map.get_mut(&9).unwrap().0 = 90;
        assert_eq!(copies(), 1);
        map.get_mut(&9).unwrap().0 = 91;
        assert_eq!(copies(), 1);
        assert_eq!(number(map.get(&9)), Some(91));
        assert_eq!(number(copy.get(&9)), Some(9));
        assert_eq!(number(copy.get(&7)), Some(7));
        assert_eq!(number(copy.get(&8)), Some(8));
        assert_eq!(number(copy.get(&2_000)), None);
        // The copy's 2,000, and the map's own 2,000, 7 and 9.
        assert_eq!(live(), 2_003);
        drop(copy);
        assert_eq!(live(), 2_000);
        // Nor is a value taken out of the only map that holds it.
        assert_eq!(number(map.remove(&10).as_deref()), Some(10));
        assert_eq!(
            number(map.insert(11, Counted::new(110)).as_deref()),
            Some(11)
        );
        assert_eq!(copies(), 1);
        assert_eq!(live(), 1_999);
        drop(map);
        assert_eq!(live(), 0);
    }

    #[test]
    fn keys_never_used_again_leave_no_blocks() {
        // A window of four live keys moving up, as nodes that retire do.
        let mut map = IdMap::new();
        let mut most = 0;
        for id in 0..200_000u64 {
            map.insert(id, id);
            if id >= 4 {
                assert_eq!(map.remove(&(id - 4)).as_deref(), Some(&(id - 4)));
            }
            if id % 1_009 == 0 {
                check_shape(&map);
                most = most.max(blocks(&map).len());
            }
        }
        assert_eq!(map.len(), 4);
        assert_eq!(map.height, 3);
        // At most two leaves and the blocks above them.
        assert!(most <= 2 * (map.height as usize + 1), "{most} blocks");
        assert!(map.keys().eq(199_996..200_000));
    }

    #[test]
    fn a_key_the_tree_is_too_low_for_is_not_there() {
        // A key past what the tree covers reads the same low bits as one it
        // holds, so only the height tells them apart.
        let mut map = IdMap::new();
        map.insert(1, "one");
        assert_eq!(map.height, 0);
        for id in [
            1 + WIDTH as u64,
            1 + (WIDTH as u64) * (WIDTH as u64),
            1 | 1 << 63,
        ] {
            assert_eq!(map.get(&id), None, "{id}");
            assert!(!map.contains_key(&id));
            assert!(map.get_mut(&id).is_none());
            assert_eq!(map.remove(&id), None);
        }
        map.insert(40, "forty");
        assert_eq!(map.height, 1);
        let past = 40 + (WIDTH as u64) * (WIDTH as u64);
        assert_eq!(map.get(&past), None);
        assert_eq!(map.remove(&past), None);
        assert_eq!(map.len(), 2);
        check_shape(&map);
    }

    #[test]
    fn the_tree_is_as_tall_as_its_keys_need() {
        let mut map = IdMap::new();
        // The first key decides the height, however large it is.
        map.insert(u64::MAX, "last");
        assert_eq!(map.height, TALLEST);
        assert_eq!(blocks(&map).len(), TALLEST as usize + 1);
        map.insert(0, "first");
        assert_eq!(
            map.iter().collect::<Vec<_>>(),
            [(0, &"first"), (u64::MAX, &"last")]
        );
        // Without it the tree is one leaf again.
        assert_eq!(map.remove(&u64::MAX).as_deref(), Some(&"last"));
        assert_eq!(map.height, 0);
        assert_eq!(blocks(&map).len(), 1);
        // Each power of the width is the first key of a taller tree.
        let mut bound = WIDTH as u64;
        for height in 1..=TALLEST {
            map.insert(bound - 1, "below");
            assert_eq!(map.height, height - 1);
            map.insert(bound, "at");
            assert_eq!(map.height, height);
            assert_eq!(map.get(&bound), Some(&"at"));
            assert_eq!(map.get(&(bound - 1)), Some(&"below"));
            assert_eq!(map.get(&(bound + 1)), None);
            check_shape(&map);
            bound = bound.wrapping_mul(WIDTH as u64);
        }
        assert_eq!(map.len(), 1 + 2 * TALLEST as usize);
        // Taking the keys out from the top lowers it again, level by level.
        let keys: Vec<u64> = map.keys().collect();
        for id in keys.into_iter().rev() {
            map.remove(&id);
            check_shape(&map);
        }
        assert!(map.is_empty());
        assert_eq!(map.get(&0), None);
        assert_eq!(map.iter().next(), None);
    }

    #[test]
    fn two_maps_with_the_same_values_are_equal_however_they_were_built() {
        let up: IdMap<u64> = (0..3_000).map(|id| (id * 7, id)).collect();
        let mut down: IdMap<u64> = (0..3_000).rev().map(|id| (id * 7, id)).collect();
        // A key far up and gone again leaves the same tree behind.
        down.insert(1 << 40, 0);
        down.remove(&(1 << 40));
        assert!(!up.shares_all(&down));
        assert_eq!(up, down);
        assert_eq!(format!("{up:?}"), format!("{down:?}"));
        *down.get_mut(&700).unwrap() += 1;
        assert_ne!(up, down);
        *down.get_mut(&700).unwrap() -= 1;
        assert_eq!(up, down);
        down.remove(&700);
        assert_ne!(up, down);
        down.insert(701, 100);
        assert_ne!(up, down);
        let small: IdMap<&str> = [(3, "c"), (1, "a")].into_iter().collect();
        assert_eq!(format!("{small:?}"), r#"{1: "a", 3: "c"}"#);
        assert_eq!(small[&3], "c");
    }

    #[test]
    fn a_map_begins_another_when_its_keys_and_values_come_first_in_it() {
        fn listed(map: &IdMap<u64>) -> Vec<(u64, u64)> {
            map.iter().map(|(id, value)| (id, *value)).collect()
        }
        // Each way round, against the lists of keys and values.
        fn check(one: &IdMap<u64>, other: &IdMap<u64>) {
            for (earlier, later) in [(one, other), (other, one)] {
                assert_eq!(
                    earlier.begins(later),
                    listed(later).starts_with(&listed(earlier)),
                    "{earlier:?} in {later:?}"
                );
            }
        }
        let mut rng = Rng(0x2545_f491_4f6c_dd1d);
        let mut held = 0;
        for spread in [3, 40, 1_100, 40_000, u64::MAX] {
            let mut map: IdMap<u64> = IdMap::new();
            let mut copies = vec![map.clone()];
            for step in 0..800u64 {
                let id = if spread == u64::MAX {
                    rng.next() >> rng.below(64)
                } else {
                    rng.below(spread)
                };
                match rng.below(10) {
                    // Most often a key past the last, as a ledger numbers
                    // what it is given.
                    0..=4 => {
                        let past = map
                            .keys()
                            .last()
                            .map_or(0, |last| last.saturating_add(1 + rng.below(3)));
                        map.insert(past, step);
                    }
                    5 => {
                        map.insert(id, step);
                    }
                    6 => {
                        map.remove(&id);
                    }
                    7 if !map.is_empty() => {
                        let nth = rng.below(map.len() as u64) as usize;
                        let id = map.keys().nth(nth).unwrap();
                        map.remove(&id);
                    }
                    _ => {
                        if let Some(value) = map.get_mut(&id) {
                            *value += 1;
                        }
                    }
                }
                check_shape(&map);
                if step % 5 == 0 {
                    if copies.len() == 8 {
                        copies.remove(rng.below(8) as usize);
                    }
                    copies.push(map.clone());
                }
                for copy in &copies {
                    check(copy, &map);
                    held += usize::from(copy.begins(&map) && copy.len() < map.len());
                }
                // A map built again shares nothing, and answers the same.
                if step % 89 == 0 {
                    let rebuilt: IdMap<u64> = listed(&map).into_iter().collect();
                    assert!(map.begins(&rebuilt) && rebuilt.begins(&map));
                    for copy in &copies {
                        check(copy, &rebuilt);
                    }
                }
            }
        }
        // Both answers came up, and often.
        assert!(held > 1_000, "{held}");
    }

    #[test]
    fn a_map_is_within_another_that_holds_its_keys_with_values_kept() {
        // Against a sorted map: every key held, with a value no smaller.
        fn check(one: &IdMap<u64>, other: &IdMap<u64>) {
            for (here, later) in [(one, other), (other, one)] {
                let expected = here
                    .iter()
                    .all(|(id, value)| later.get(&id).is_some_and(|kept| kept >= value));
                assert_eq!(
                    here.within(later, |value, kept| kept >= value),
                    expected,
                    "{here:?} in {later:?}"
                );
            }
        }
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        let mut held = 0;
        for spread in [3, 40, 1_100, 40_000, u64::MAX] {
            let mut map: IdMap<u64> = IdMap::new();
            let mut copies = vec![map.clone()];
            for step in 0..800u64 {
                let id = if spread == u64::MAX {
                    rng.next() >> rng.below(64)
                } else {
                    rng.below(spread)
                };
                match rng.below(8) {
                    0..=3 => {
                        map.insert(id, step);
                    }
                    4 => {
                        map.remove(&id);
                    }
                    5 => {
                        if let Some(value) = map.get_mut(&id) {
                            *value = value.saturating_sub(1);
                        }
                    }
                    _ => {
                        if let Some(value) = map.get_mut(&id) {
                            *value += 1;
                        }
                    }
                }
                if step % 5 == 0 {
                    if copies.len() == 8 {
                        copies.remove(rng.below(8) as usize);
                    }
                    copies.push(map.clone());
                }
                for copy in &copies {
                    check(copy, &map);
                    held += usize::from(copy.within(&map, |value, kept| kept >= value));
                }
                if step % 89 == 0 {
                    let rebuilt: IdMap<u64> = map.iter().map(|(id, v)| (id, *v)).collect();
                    for copy in &copies {
                        check(copy, &rebuilt);
                    }
                }
            }
        }
        assert!(held > 1_000, "{held}");
    }

    #[test]
    fn what_two_maps_share_is_not_read_to_tell_that_one_is_within_the_other() {
        let mut asked = 0;
        let earlier: IdMap<u64> = (0..40_010).map(|id| (id, id)).collect();
        let mut later = earlier.clone();
        assert!(earlier.within(&later, |_, _| {
            asked += 1;
            false
        }));
        later.insert(40_010, 0);
        later.insert(1 << 30, 0);
        *later.get_mut(&7).unwrap() += 1;
        assert!(earlier.within(&later, |value, kept| {
            asked += 1;
            kept >= value
        }));
        assert_eq!(asked, 1, "only the value written is read");
        assert!(!later.within(&earlier, |_, _| true));
    }

    #[test]
    fn what_two_maps_share_is_not_read_to_tell_that_one_begins_the_other() {
        static COMPARED: AtomicIsize = AtomicIsize::new(0);
        #[derive(Clone)]
        struct Counted(u64);
        impl PartialEq for Counted {
            fn eq(&self, other: &Self) -> bool {
                COMPARED.fetch_add(1, Ordering::Relaxed);
                self.0 == other.0
            }
        }
        let compared = || COMPARED.swap(0, Ordering::Relaxed);

        let earlier: IdMap<Counted> = (0..40_010).map(|id| (id, Counted(id))).collect();
        let mut later = earlier.clone();
        assert!(earlier.begins(&later) && later.begins(&earlier));
        // Keys added past the last: into the last leaf, a copy of it that
        // shares its values, and into leaves of their own.
        for id in 40_010..40_100 {
            later.insert(id, Counted(id));
        }
        assert!(earlier.begins(&later));
        assert!(!later.begins(&earlier));
        // And past what the tree had room for, so that it grows a level.
        later.insert(1 << 30, Counted(0));
        assert!(later.height > earlier.height);
        assert!(earlier.begins(&later));
        assert!(!later.begins(&earlier));
        assert_eq!(compared(), 0, "a value both maps share is not compared");

        // A value written is the map's own, and the one that is compared.
        later.get_mut(&12_345).unwrap().0 = 12_345;
        assert!(earlier.begins(&later));
        assert_eq!(compared(), 1);
        later.get_mut(&12_345).unwrap().0 = 0;
        assert!(!earlier.begins(&later));
        assert_eq!(compared(), 1);
        later.get_mut(&12_345).unwrap().0 = 12_345;
        // A key taken out, and one put in before the last.
        let mut without = later.clone();
        without.remove(&7);
        assert!(!earlier.begins(&without));
        let mut shorter = earlier.clone();
        shorter.remove(&40_000);
        assert!(!shorter.begins(&later), "40,000 comes before 40,009");
        shorter.remove(&40_009);
        assert!(!shorter.begins(&later));
        for id in 40_001..40_009 {
            shorter.remove(&id);
        }
        assert!(shorter.begins(&later) && shorter.begins(&earlier));
        // An empty map begins every map, and only an empty map begins it.
        let empty = IdMap::new();
        assert!(empty.begins(&later) && empty.begins(&empty));
        assert!(!later.begins(&empty));

        // Maps that share nothing are compared value by value.
        compared();
        let rebuilt: IdMap<Counted> = (0..40_050).map(|id| (id, Counted(id))).collect();
        assert!(earlier.begins(&rebuilt));
        assert_eq!(compared(), 40_010);
    }
}
