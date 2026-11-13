//! A list that a transaction copies in constant time. A session copies itself
//! before each event so a refusal leaves it untouched; a list it then appends
//! to would otherwise be copied whole. Here the values sit in chunks, and the
//! chunks sit in an [`IdMap`] by their number, so a copy shares every chunk
//! and every block of the map. A write copies only the chunk it lands in,
//! whose values are themselves shared rather than copied, and the blocks of
//! the map on the way down to it.
use crate::idmap::{IdMap, RUN};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

const CHUNK: usize = 16;

pub struct Chunked<T> {
    /// The chunks by number, from `first` on with no number missing. Every
    /// chunk is full but the last, and the first after a removal from the
    /// front, and none is empty.
    chunks: IdMap<Vec<Arc<T>>>,
    /// The number of the first chunk. A removal from the front drops chunks
    /// and moves this on, so the chunks left keep their numbers.
    first: u64,
    len: usize,
}

impl<T> Default for Chunked<T> {
    fn default() -> Self {
        Self {
            chunks: IdMap::new(),
            first: 0,
            len: 0,
        }
    }
}

impl<T> Clone for Chunked<T> {
    fn clone(&self) -> Self {
        Self {
            chunks: self.chunks.clone(),
            first: self.first,
            len: self.len,
        }
    }
}

impl<T: Clone> Chunked<T> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// One past the number of the last chunk.
    fn end(&self) -> u64 {
        self.first + self.chunks.len() as u64
    }

    /// The values of the chunk numbered `number`: none past either end.
    fn chunk(&self, number: u64) -> &[Arc<T>] {
        self.chunks.get(&number).map_or(&[], Vec::as_slice)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter(self.arcs_from(0))
    }

    /// The values from `index` on, as the chunks hold them: none when `index`
    /// is past the end.
    fn arcs_from(&self, index: usize) -> Arcs<'_, T> {
        let mut arcs = Arcs {
            chunks: &self.chunks,
            runs: 0..0,
            front_slots: Default::default(),
            front: Default::default(),
            back_slots: Default::default(),
            back: Default::default(),
            rest: 0,
        };
        if index < self.len {
            let (start, at) = self.place(index);
            let slots = self.chunks.run(start);
            let place = (start % RUN) as usize;
            let chunk = slots[place].as_deref().expect("a chunk holds each value");
            arcs.front = chunk[at..].iter();
            arcs.front_slots = slots[place + 1..].iter();
            arcs.runs = start / RUN + 1..(self.end() - 1) / RUN + 1;
            arcs.rest = self.len - index - arcs.front.len();
        }
        arcs
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        let (number, at) = self.place(index);
        self.chunk(number).get(at).map(|value| &**value)
    }

    pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        self.chunk(self.end() - 1).last().map(|value| &**value)
    }

    pub fn push(&mut self, value: T) {
        let value = Arc::new(value);
        let last = self.end().wrapping_sub(1);
        let room = !self.is_empty() && self.chunk(last).len() < CHUNK;
        if room {
            self.chunks
                .get_mut(&last)
                .expect("the last chunk")
                .push(value);
        } else {
            let mut chunk = Vec::with_capacity(CHUNK);
            chunk.push(value);
            self.chunks.insert(self.end(), chunk);
        }
        self.len += 1;
    }

    /// Remove and return the first `count` values, oldest first.
    pub fn drain_front(&mut self, count: usize) -> Vec<T> {
        let count = count.min(self.len);
        let mut out = Vec::with_capacity(count);
        while out.len() < count {
            let need = count - out.len();
            let first = self
                .chunks
                .remove(&self.first)
                .expect("a chunk holds each value");
            if first.len() <= need {
                self.first += 1;
                out.extend(
                    Arc::unwrap_or_clone(first)
                        .into_iter()
                        .map(Arc::unwrap_or_clone),
                );
            } else {
                let mut values = Arc::unwrap_or_clone(first);
                let rest = values.split_off(need);
                out.extend(values.into_iter().map(Arc::unwrap_or_clone));
                self.chunks.insert(self.first, rest);
            }
        }
        self.len -= count;
        out
    }

    /// The value at `index` to change, copying first only its chunk and the
    /// blocks on the way down to it and, when another copy shares it, the
    /// value.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        let (number, at) = self.place(index);
        let chunk = self.chunks.get_mut(&number)?;
        Some(Arc::make_mut(&mut chunk[at]))
    }

    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

    /// The number of the chunk that holds `index`, which is in range, and
    /// the place in it.
    fn place(&self, index: usize) -> (u64, usize) {
        let first = self.chunk(self.first).len();
        if index < first {
            (self.first, index)
        } else {
            let rest = index - first;
            (self.first + 1 + (rest / CHUNK) as u64, rest % CHUNK)
        }
    }

    /// The values from `index` on, in order: none when `index` is past the
    /// end. Reaching the first of them reads no value before it.
    pub fn iter_from(&self, index: usize) -> Iter<'_, T> {
        Iter(self.arcs_from(index))
    }

    /// Whether this list's first values are `prefix`'s, in order. A chunk
    /// both lists share at the same place is not read.
    pub fn starts_with(&self, prefix: &Self) -> bool
    where
        T: PartialEq,
    {
        if prefix.len > self.len {
            return false;
        }
        let aligned = self.first == prefix.first
            && self.chunk(self.first).len() == prefix.chunk(prefix.first).len();
        if !aligned {
            return self
                .iter()
                .zip(prefix.iter())
                .all(|(own, theirs)| own == theirs);
        }
        // Each chunk of `prefix` but its last is full where `self`'s is, so
        // the two lists' chunks of one number hold the same places, and the
        // maps are read only where they do not share a block or a chunk.
        prefix.chunks.within(&self.chunks, |theirs, own| {
            own.len() >= theirs.len()
                && own
                    .iter()
                    .zip(theirs)
                    .all(|(own, theirs)| Arc::ptr_eq(own, theirs) || own == theirs)
        })
    }

    /// Keeps the values `keep` answers true for, in order, asking it once
    /// for each value from first to last, as `Vec::retain` does. The chunks
    /// before the first value it drops stay shared; from there on the values
    /// kept are shared too, in new chunks. A panic in `keep` leaves the list
    /// as it was.
    pub fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        // Every value is asked about before the map changes. The chunks are
        // read a run of numbers at a time, and the map holds no chunk outside
        // `first..end`, so the empty slots of a run are passed over.
        let end = self.end();
        let runs = self.first / RUN..end.div_ceil(RUN);
        let mut dropped = None;
        'find: for run in runs.clone() {
            for (slot, chunk) in self.chunks.run(run * RUN).iter().enumerate() {
                let Some(chunk) = chunk else { continue };
                for (at, value) in chunk.iter().enumerate() {
                    if !keep(value) {
                        dropped = Some((run * RUN + slot as u64, at));
                        break 'find;
                    }
                }
            }
        }
        let Some((number, at)) = dropped else {
            return;
        };
        // Every chunk between the first and chunk `number` is full.
        let before = match number - self.first {
            0 => 0,
            later => self.chunk(self.first).len() + (later as usize - 1) * CHUNK,
        };
        // The values kept from chunk `number` on, in full chunks but the
        // last.
        let head = self.chunk(number);
        let mut filling: Vec<Arc<T>> = Vec::with_capacity(CHUNK);
        filling.extend_from_slice(&head[..at]);
        let mut kept: Vec<Vec<Arc<T>>> = Vec::new();
        let mut take = |value: &Arc<T>| {
            if keep(value) {
                if filling.len() == CHUNK {
                    kept.push(std::mem::replace(&mut filling, Vec::with_capacity(CHUNK)));
                }
                filling.push(Arc::clone(value));
            }
        };
        head[at + 1..].iter().for_each(&mut take);
        let place = (number % RUN) as usize + 1;
        for run in number / RUN..runs.end {
            let skip = if run == number / RUN { place } else { 0 };
            for chunk in self.chunks.run(run * RUN)[skip..].iter().flatten() {
                chunk.iter().for_each(&mut take);
            }
        }
        if !filling.is_empty() {
            kept.push(filling);
        }
        // They are written over the chunks there, and the chunks left past
        // them are removed.
        self.len = before + kept.iter().map(Vec::len).sum::<usize>();
        let mut next = number;
        for chunk in kept {
            self.chunks.insert(next, chunk);
            next += 1;
        }
        for number in next..end {
            self.chunks.remove(&number);
        }
    }
}

/// The values of a list as its chunks hold them, in order, from either end.
/// Each end reads the chunks of a run of numbers from one block of the map,
/// so it walks down the map once for every [`RUN`] chunks rather than once
/// for each.
struct Arcs<'a, T> {
    chunks: &'a IdMap<Vec<Arc<T>>>,
    /// The runs of chunk numbers that neither end has reached.
    runs: std::ops::Range<u64>,
    /// The slots of the run each end is in, past the chunk it is in.
    front_slots: std::slice::Iter<'a, Option<Arc<Vec<Arc<T>>>>>,
    back_slots: std::slice::Iter<'a, Option<Arc<Vec<Arc<T>>>>>,
    /// The values each end has not read of the chunk it is in.
    front: std::slice::Iter<'a, Arc<T>>,
    back: std::slice::Iter<'a, Arc<T>>,
    /// The values in the chunks neither end has been in, so that reading a
    /// value counts nothing.
    rest: usize,
}

// Not derived: a copy of the walk needs no copy of a value.
impl<T> Clone for Arcs<'_, T> {
    fn clone(&self) -> Self {
        Self {
            chunks: self.chunks,
            runs: self.runs.clone(),
            front_slots: self.front_slots.clone(),
            back_slots: self.back_slots.clone(),
            front: self.front.clone(),
            back: self.back.clone(),
            rest: self.rest,
        }
    }
}

impl<'a, T> Arcs<'a, T> {
    /// Moves the front on to the next chunk neither end has been in, and
    /// reads its first value; once there is none, the front reads on in the
    /// chunk the back is in. The chunk is in the front's run, in a run
    /// neither end has reached, or among the slots the back has not read.
    #[inline]
    fn front_chunk(&mut self) -> Option<&'a Arc<T>> {
        if self.rest == 0 {
            return self.back.next();
        }
        loop {
            let slot = match self.front_slots.next() {
                Some(slot) => slot,
                None => match self.runs.next() {
                    Some(run) => {
                        self.front_slots = self.chunks.run(run * RUN).iter();
                        continue;
                    }
                    None => self.back_slots.next()?,
                },
            };
            if let Some(chunk) = slot {
                self.rest -= chunk.len();
                self.front = chunk.iter();
                return self.front.next();
            }
        }
    }

    /// As [`Self::front_chunk`], from the back.
    #[inline]
    fn back_chunk(&mut self) -> Option<&'a Arc<T>> {
        if self.rest == 0 {
            return self.front.next_back();
        }
        loop {
            let slot = match self.back_slots.next_back() {
                Some(slot) => slot,
                None => match self.runs.next_back() {
                    Some(run) => {
                        self.back_slots = self.chunks.run(run * RUN).iter();
                        continue;
                    }
                    None => self.front_slots.next_back()?,
                },
            };
            if let Some(chunk) = slot {
                self.rest -= chunk.len();
                self.back = chunk.iter();
                return self.back.next_back();
            }
        }
    }
}

impl<'a, T> Iterator for Arcs<'a, T> {
    type Item = &'a Arc<T>;

    #[inline]
    fn next(&mut self) -> Option<&'a Arc<T>> {
        match self.front.next() {
            Some(value) => Some(value),
            None => self.front_chunk(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.rest + self.front.len() + self.back.len();
        (left, Some(left))
    }
}

impl<'a, T> DoubleEndedIterator for Arcs<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a Arc<T>> {
        match self.back.next_back() {
            Some(value) => Some(value),
            None => self.back_chunk(),
        }
    }
}

impl<T> ExactSizeIterator for Arcs<'_, T> {}

/// The values of a [`Chunked`] list in order, from either end.
pub struct Iter<'a, T>(Arcs<'a, T>);

impl<T> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.0.next().map(|value| &**value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a T> {
        self.0.next_back().map(|value| &**value)
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}

impl<'a, T: Clone> IntoIterator for &'a Chunked<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Prints the values as a list, as a `Vec` of them prints.
impl<T: Clone + std::fmt::Debug> std::fmt::Debug for Chunked<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Clone> std::ops::Index<usize> for Chunked<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.get(index).expect("chunked index in range")
    }
}

impl<T: Clone + PartialEq> PartialEq for Chunked<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        // Two lists numbered alike, whose first chunks are as long, hold
        // each place in a chunk of the same number, and the maps compare
        // only the blocks and the chunks the two do not share.
        if self.first == other.first
            && self.chunk(self.first).len() == other.chunk(other.first).len()
        {
            return self.chunks == other.chunks;
        }
        self.iter().eq(other.iter())
    }
}

impl<T: Clone> From<Vec<T>> for Chunked<T> {
    fn from(values: Vec<T>) -> Self {
        let mut list = Self::default();
        for value in values {
            list.push(value);
        }
        list
    }
}

impl<T: Clone> FromIterator<T> for Chunked<T> {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        let mut list = Self::default();
        for value in values {
            list.push(value);
        }
        list
    }
}

impl<T: Clone + Serialize> Serialize for Chunked<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}

impl<'de, T: Clone + Deserialize<'de>> Deserialize<'de> for Chunked<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<T>::deserialize(deserializer).map(Self::from)
    }
}

#[cfg(test)]
mod tests {
    use super::{Chunked, CHUNK};

    #[test]
    fn indexes_pushes_and_front_removal_agree_with_a_vec() {
        let mut list = Chunked::default();
        let mut plain = Vec::new();
        for value in 0..200 {
            list.push(value);
            plain.push(value);
            let copy = list.clone();
            assert_eq!(copy.to_vec(), plain);
        }
        for count in [0, 5, 27, 32, 1, 40] {
            let drained = list.drain_front(count);
            let expected: Vec<i32> = plain.drain(..count).collect();
            assert_eq!(drained, expected);
            assert_eq!(list.len(), plain.len());
            for (index, value) in plain.iter().enumerate() {
                assert_eq!(list[index], *value);
            }
            list.push(1000 + count as i32);
            plain.push(1000 + count as i32);
            assert_eq!(list.to_vec(), plain);
        }
    }

    /// The chunks are numbered from `first` on with none missing, every
    /// chunk is full but the first and the last, none is empty, and the
    /// length is theirs.
    fn check<T: Clone>(list: &Chunked<T>) {
        let chunks = list.chunks.iter().collect::<Vec<_>>();
        for (place, (number, chunk)) in chunks.iter().enumerate() {
            assert_eq!(*number, list.first + place as u64, "chunk numbers");
            assert!(!chunk.is_empty(), "chunk {place} is empty");
            assert!(chunk.len() <= CHUNK);
            if place != 0 && place + 1 != chunks.len() {
                assert_eq!(chunk.len(), CHUNK, "chunk {place} is not full");
            }
        }
        assert_eq!(
            list.len,
            chunks.iter().map(|(_, chunk)| chunk.len()).sum::<usize>()
        );
    }

    #[test]
    fn retain_reads_from_and_starts_with_agree_with_a_vec() {
        // A small generator, so the test needs no crate.
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = move |below: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % below
        };
        for round in 0..300u64 {
            let mut list = Chunked::default();
            let mut plain = Vec::new();
            for value in 0..next(90) {
                list.push(value);
                plain.push(value);
            }
            if next(3) == 0 {
                let count = next(20) as usize;
                list.drain_front(count);
                plain.drain(..count.min(plain.len()));
            }
            let before = list.clone();
            let earlier = plain.clone();
            let mut asked = Vec::new();
            let mut asked_plain = Vec::new();
            let cut = next(5);
            list.retain(|value| {
                asked.push(*value);
                (value + round) % 5 != cut || next(4) == 0
            });
            let mut keep = list
                .iter()
                .copied()
                .collect::<Vec<_>>()
                .into_iter()
                .peekable();
            plain.retain(|value| {
                asked_plain.push(*value);
                keep.next_if_eq(value).is_some()
            });
            assert_eq!(
                asked, asked_plain,
                "retain asks once for each value, in order"
            );
            assert_eq!(list.to_vec(), plain);
            check(&list);
            for (index, value) in plain.iter().enumerate() {
                assert_eq!(list[index], *value);
            }
            for index in 0..plain.len() + 2 {
                let from = list.iter_from(index).copied().collect::<Vec<_>>();
                assert_eq!(from, plain.get(index..).unwrap_or_default());
            }
            // A copy the list grew from begins it; one it lost values of
            // does not, unless they were none.
            assert_eq!(list.starts_with(&before), plain.starts_with(&earlier));
            assert_eq!(before.starts_with(&list), earlier.starts_with(&plain));
            let mut grown = list.clone();
            for value in 0..next(40) {
                grown.push(1_000 + value);
            }
            check(&grown);
            assert!(grown.starts_with(&list));
            assert_eq!(list.starts_with(&grown), grown.len() == list.len());
            if !list.is_empty() {
                let mut changed = grown.clone();
                *changed.get_mut(next(list.len() as u64) as usize).unwrap() += 7;
                assert!(!changed.starts_with(&list));
            }
        }
    }

    #[test]
    fn retain_keeps_the_chunks_before_the_first_value_it_drops() {
        let mut list: Chunked<u32> = (0..100).collect();
        let copy = list.clone();
        list.retain(|value| *value != 70);
        let shared = list
            .chunks
            .values()
            .zip(copy.chunks.values())
            .take_while(|(one, other)| std::ptr::eq(*one, *other))
            .count();
        assert_eq!(shared, 70 / CHUNK);
        // The values kept after it are the copy's, not copies of them.
        let kept = copy.iter().filter(|value| **value != 70);
        assert!(list
            .iter()
            .zip(kept)
            .all(|(one, other)| std::ptr::eq(one, other)));
        let mut whole = copy.clone();
        whole.retain(|_| true);
        assert!(whole.chunks.shares_all(&copy.chunks));
    }

    #[test]
    fn a_panic_in_keep_leaves_the_list_as_it_was() {
        let mut list: Chunked<u32> = (0..48).collect();
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            list.retain(|value| {
                assert_ne!(*value, 20, "keep panics");
                *value != 5
            })
        }));
        assert!(caught.is_err());
        check(&list);
        assert_eq!(list.to_vec(), (0..48).collect::<Vec<_>>());
        list.push(48);
        assert_eq!(list.last(), Some(&48));
    }

    #[test]
    fn a_copy_shares_all_and_a_write_copies_one_chunk() {
        // Long enough that the map of chunks has more than one level, with
        // room in the last chunk.
        let mut list: Chunked<u32> = (0..1_999).collect();
        let copy = list.clone();
        assert!(list.chunks.shares_all(&copy.chunks));
        list.push(1_999);
        *list.get_mut(37).unwrap() += 1;
        check(&list);
        let unshared = list
            .chunks
            .values()
            .zip(copy.chunks.values())
            .filter(|(one, other)| !std::ptr::eq(*one, *other))
            .count();
        assert_eq!(unshared, 2, "the chunk written and the last");
        assert_eq!(copy.to_vec(), (0..1_999).collect::<Vec<_>>());
        assert_eq!(list[37], 38);
        assert_eq!(list.last(), Some(&1_999));
        // The values the write did not reach are still the copy's.
        assert!(std::ptr::eq(&list[36], &copy[36]));
        let mut drained = list.clone();
        drained.drain_front(40);
        check(&drained);
        assert_eq!(drained[0], 40);
        assert!(drained.starts_with(&drained.clone()));
        assert!(!drained.starts_with(&list));
    }

    #[test]
    fn a_copy_is_unchanged_by_a_later_push() {
        let mut list: Chunked<u32> = (0..40).collect();
        let copy = list.clone();
        list.push(40);
        assert_eq!(copy.len(), 40);
        assert_eq!(list.len(), 41);
        assert_ne!(copy, list);
        // It prints as the list of its values does.
        let values = (0..41).collect::<Vec<u32>>();
        assert_eq!(format!("{list:?}"), format!("{values:?}"));
    }

    #[test]
    fn lists_numbered_apart_compare_by_their_values() {
        for total in [0usize, 1, 15, 16, 17, 33, 100, 1_000, 2_000] {
            for drained in [0usize, 1, 5, 16, 17, 40] {
                let mut list: Chunked<u32> = (0..total as u32).collect();
                list.drain_front(drained);
                let values = list.to_vec();
                // Rebuilt, as a restore builds a list, its chunks are
                // numbered from 0 and the first is full.
                let rebuilt = Chunked::from(values.clone());
                assert!(list == rebuilt && rebuilt == list, "{total} {drained}");
                assert!(list.starts_with(&rebuilt) && rebuilt.starts_with(&list));
                for cut in [0, 1, 7, 16, 17, values.len().saturating_sub(1)] {
                    let cut = cut.min(values.len());
                    let prefix = Chunked::from(values[..cut].to_vec());
                    assert!(list.starts_with(&prefix), "{total} {drained} {cut}");
                    assert_eq!(prefix.starts_with(&list), cut == values.len());
                    assert_eq!(prefix == list, cut == values.len());
                }
                let mut grown = list.clone();
                grown.push(9_999);
                assert!(grown.starts_with(&list) && !list.starts_with(&grown));
                assert!(grown != list && list != grown);
                // Numbered alike: a value written apart, then written back.
                if !values.is_empty() {
                    let mut changed = list.clone();
                    *changed.get_mut(values.len() / 2).unwrap() += 1;
                    assert!(changed != list && list != changed);
                    assert!(!changed.starts_with(&list) && !list.starts_with(&changed));
                    *changed.get_mut(values.len() / 2).unwrap() -= 1;
                    assert!(changed == list && list.starts_with(&changed));
                }
            }
        }
    }

    #[test]
    fn lists_read_alike_forward_backward_and_from_any_place() {
        // Up to more than a run of runs of chunks, so the map has three
        // levels, and drained to start inside a run.
        for total in [0usize, 1, 16, 511, 512, 513, 1_000, 16_400] {
            for drained in [0usize, 1, 17, 500, 512, 600] {
                let mut list: Chunked<u32> = (0..total as u32).collect();
                list.drain_front(drained);
                check(&list);
                let plain: Vec<u32> = (drained.min(total) as u32..total as u32).collect();
                assert_eq!(list.iter().len(), plain.len());
                assert!(list.iter().eq(plain.iter()));
                assert!(list.iter().rev().eq(plain.iter().rev()));
                assert_eq!(list.iter().sum::<u32>(), plain.iter().sum::<u32>());
                // From both ends in turn, with a copy of the walk on the way.
                let (mut ours, mut theirs) = (list.iter(), plain.iter());
                loop {
                    let (one, other) = if ours.len() % 3 == 0 {
                        (ours.next_back(), theirs.next_back())
                    } else {
                        (ours.next(), theirs.next())
                    };
                    assert_eq!(one, other);
                    assert_eq!(ours.len(), theirs.len());
                    if ours.len() == plain.len() / 2 {
                        assert!(ours.clone().rev().eq(theirs.clone().rev()));
                    }
                    if other.is_none() {
                        break;
                    }
                }
                let step = 1 + plain.len() / 64;
                let ends = [plain.len().saturating_sub(1), plain.len(), plain.len() + 1];
                for index in (0..plain.len()).step_by(step).chain(ends) {
                    let from = list.iter_from(index);
                    assert_eq!(from.len(), plain.len().saturating_sub(index));
                    assert!(
                        from.eq(plain.iter().skip(index)),
                        "{total} {drained} {index}"
                    );
                }
            }
        }
    }
}
