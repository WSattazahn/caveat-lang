//! A list that a transaction copies in time proportional to its number of
//! chunks rather than its length. A session copies itself before each event so
//! a refusal leaves it untouched; a list it then appends to would otherwise be
//! copied whole. Here the copy shares every chunk, and a write copies only the
//! chunk it lands in, whose values are themselves shared rather than copied.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

const CHUNK: usize = 16;

#[derive(Debug)]
pub struct Chunked<T> {
    chunks: Vec<Arc<Vec<Arc<T>>>>,
    len: usize,
}

impl<T> Default for Chunked<T> {
    fn default() -> Self {
        Self {
            chunks: Vec::new(),
            len: 0,
        }
    }
}

impl<T> Clone for Chunked<T> {
    fn clone(&self) -> Self {
        Self {
            chunks: self.chunks.clone(),
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

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + Clone + '_ {
        self.chunks
            .iter()
            .flat_map(|chunk| chunk.iter().map(|value| &**value))
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        // Every chunk is full but the last, and the first after a removal
        // from the front.
        let first = self.chunks.first().map_or(0, |chunk| chunk.len());
        if index < first {
            return self.chunks[0].get(index).map(|value| &**value);
        }
        let rest = index - first;
        self.chunks
            .get(1 + rest / CHUNK)?
            .get(rest % CHUNK)
            .map(|value| &**value)
    }

    pub fn last(&self) -> Option<&T> {
        self.chunks
            .last()
            .and_then(|chunk| chunk.last())
            .map(|value| &**value)
    }

    pub fn push(&mut self, value: T) {
        match self.chunks.last_mut() {
            Some(last) if last.len() < CHUNK => Arc::make_mut(last).push(Arc::new(value)),
            _ => {
                let mut chunk = Vec::with_capacity(CHUNK);
                chunk.push(Arc::new(value));
                self.chunks.push(Arc::new(chunk));
            }
        }
        self.len += 1;
    }

    /// Remove and return the first `count` values, oldest first.
    pub fn drain_front(&mut self, count: usize) -> Vec<T> {
        let count = count.min(self.len);
        let mut out = Vec::with_capacity(count);
        while out.len() < count {
            let need = count - out.len();
            let first = self.chunks.remove(0);
            if first.len() <= need {
                out.extend(
                    Arc::unwrap_or_clone(first)
                        .into_iter()
                        .map(Arc::unwrap_or_clone),
                );
            } else {
                let mut values = Arc::unwrap_or_clone(first);
                let rest = values.split_off(need);
                out.extend(values.into_iter().map(Arc::unwrap_or_clone));
                self.chunks.insert(0, Arc::new(rest));
            }
        }
        self.len -= count;
        out
    }

    /// The value at `index` to change, copying first only its chunk and,
    /// when another copy shares it, the value.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        let first = self.chunks.first().map_or(0, |chunk| chunk.len());
        let (chunk, at) = if index < first {
            (0, index)
        } else {
            (1 + (index - first) / CHUNK, (index - first) % CHUNK)
        };
        let chunk = Arc::make_mut(&mut self.chunks[chunk]);
        Some(Arc::make_mut(&mut chunk[at]))
    }

    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

    /// The chunk that holds `index`, which is in range, and the place in it.
    fn place(&self, index: usize) -> (usize, usize) {
        let first = self.chunks.first().map_or(0, |chunk| chunk.len());
        if index < first {
            (0, index)
        } else {
            (1 + (index - first) / CHUNK, (index - first) % CHUNK)
        }
    }

    /// The values from `index` on, in order: none when `index` is past the
    /// end. Reaching the first of them reads no value before it.
    pub fn iter_from(&self, index: usize) -> impl Iterator<Item = &T> + '_ {
        let (chunk, at) = if index < self.len {
            self.place(index)
        } else {
            (self.chunks.len(), 0)
        };
        self.chunks
            .get(chunk..)
            .unwrap_or_default()
            .iter()
            .enumerate()
            .flat_map(move |(step, values)| {
                let skip = if step == 0 { at } else { 0 };
                values[skip..].iter().map(|value| &**value)
            })
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
        let aligned = self.chunks.first().map(|chunk| chunk.len())
            == prefix.chunks.first().map(|chunk| chunk.len());
        if !aligned {
            return self
                .iter()
                .zip(prefix.iter())
                .all(|(own, theirs)| own == theirs);
        }
        // Each chunk of `prefix` but its last is full where `self`'s is, so
        // the two lists' chunks hold the same places.
        self.chunks.iter().zip(&prefix.chunks).all(|(own, theirs)| {
            Arc::ptr_eq(own, theirs)
                || (own.len() >= theirs.len()
                    && own
                        .iter()
                        .zip(theirs.iter())
                        .all(|(own, theirs)| Arc::ptr_eq(own, theirs) || own == theirs))
        })
    }

    /// Keeps the values `keep` answers true for, in order, asking it once
    /// for each value from first to last, as `Vec::retain` does. The chunks
    /// before the first value it drops stay shared; from there on the values
    /// kept are shared too, in new chunks.
    pub fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        let mut dropped = None;
        'chunks: for (chunk, values) in self.chunks.iter().enumerate() {
            for (at, value) in values.iter().enumerate() {
                if !keep(value) {
                    dropped = Some((chunk, at));
                    break 'chunks;
                }
            }
        }
        let Some((chunk, at)) = dropped else {
            return;
        };
        let rest = self.chunks.split_off(chunk);
        self.len = self.chunks.iter().map(|chunk| chunk.len()).sum();
        let mut rest = rest.into_iter();
        let first = rest.next().expect("the chunk of the value dropped");
        let mut kept: Vec<Arc<T>> = first[..at].to_vec();
        for value in &first[at + 1..] {
            if keep(value) {
                kept.push(Arc::clone(value));
            }
        }
        for values in rest {
            for value in values.iter() {
                if keep(value) {
                    kept.push(Arc::clone(value));
                }
            }
        }
        // The chunks kept are full but the first, so every chunk but the
        // first and the last stays full.
        self.len += kept.len();
        for values in kept.chunks(CHUNK) {
            self.chunks.push(Arc::new(values.to_vec()));
        }
    }
}

impl<'a, T: Clone> IntoIterator for &'a Chunked<T> {
    type Item = &'a T;
    type IntoIter = Box<dyn DoubleEndedIterator<Item = &'a T> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
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
        if self.chunks.len() == other.chunks.len()
            && self
                .chunks
                .iter()
                .zip(&other.chunks)
                .all(|(left, right)| Arc::ptr_eq(left, right) || left == right)
        {
            return true;
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
    use std::sync::Arc;

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

    /// Every chunk is full but the first and the last, none is empty, and
    /// the length is theirs.
    fn check<T: Clone>(list: &Chunked<T>) {
        let chunks = &list.chunks;
        for (place, chunk) in chunks.iter().enumerate() {
            assert!(!chunk.is_empty(), "chunk {place} is empty");
            assert!(chunk.len() <= CHUNK);
            if place != 0 && place + 1 != chunks.len() {
                assert_eq!(chunk.len(), CHUNK, "chunk {place} is not full");
            }
        }
        assert_eq!(
            list.len,
            chunks.iter().map(|chunk| chunk.len()).sum::<usize>()
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
            .iter()
            .zip(&copy.chunks)
            .take_while(|(one, other)| Arc::ptr_eq(one, other))
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
        assert!(whole
            .chunks
            .iter()
            .zip(&copy.chunks)
            .all(|(one, other)| Arc::ptr_eq(one, other)));
    }

    #[test]
    fn a_copy_is_unchanged_by_a_later_push() {
        let mut list: Chunked<u32> = (0..40).collect();
        let copy = list.clone();
        list.push(40);
        assert_eq!(copy.len(), 40);
        assert_eq!(list.len(), 41);
        assert_ne!(copy, list);
    }
}
