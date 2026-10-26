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
        self.chunks.iter().flat_map(|chunk| chunk.iter().map(|value| &**value))
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
            Some(last) if last.len() < CHUNK => {
                Arc::make_mut(last).push(Arc::new(value))
            }
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
                out.extend(Arc::unwrap_or_clone(first).into_iter().map(Arc::unwrap_or_clone));
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

/// A table keyed by the graph's node ids, which count up from 1 and are
/// never reused, kept in id order so a copy shares its chunks. Each slot holds
/// its id too, so it lists as a map does.
#[derive(Debug)]
pub struct IdTable<T> {
    slots: Chunked<Option<(u64, T)>>,
    len: usize,
}

impl<T> Default for IdTable<T> {
    fn default() -> Self {
        Self {
            slots: Chunked::default(),
            len: 0,
        }
    }
}

impl<T> Clone for IdTable<T> {
    fn clone(&self) -> Self {
        Self {
            slots: self.slots.clone(),
            len: self.len,
        }
    }
}

impl<T: Clone> IdTable<T> {
    fn slot(id: u64) -> Option<usize> {
        usize::try_from(id).ok()?.checked_sub(1)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, id: &u64) -> Option<&T> {
        self.slots
            .get(Self::slot(*id)?)?
            .as_ref()
            .map(|(_, value)| value)
    }

    pub fn get_mut(&mut self, id: &u64) -> Option<&mut T> {
        let slot = Self::slot(*id)?;
        self.slots.get(slot)?.as_ref()?;
        self.slots.get_mut(slot)?.as_mut().map(|(_, value)| value)
    }

    pub fn contains_key(&self, id: &u64) -> bool {
        self.get(id).is_some()
    }

    pub fn insert(&mut self, id: u64, value: T) -> Option<T> {
        let slot = Self::slot(id).expect("graph node ids start at 1");
        while self.slots.len() <= slot {
            self.slots.push(None);
        }
        let old = self
            .slots
            .get_mut(slot)
            .expect("slot just made")
            .replace((id, value))
            .map(|(_, value)| value);
        if old.is_none() {
            self.len += 1;
        }
        old
    }

    pub fn remove(&mut self, id: &u64) -> Option<T> {
        let slot = Self::slot(*id)?;
        self.slots.get(slot)?.as_ref()?;
        let old = self.slots.get_mut(slot)?.take().map(|(_, value)| value);
        if old.is_some() {
            self.len -= 1;
        }
        old
    }

    /// Ids and values in id order.
    pub fn iter(&self) -> impl Iterator<Item = (&u64, &T)> + '_ {
        self.slots
            .iter()
            .filter_map(|slot| slot.as_ref().map(|(id, value)| (id, value)))
    }

    pub fn keys(&self) -> impl Iterator<Item = &u64> + '_ {
        self.iter().map(|(id, _)| id)
    }

    pub fn values(&self) -> impl Iterator<Item = &T> + '_ {
        self.iter().map(|(_, value)| value)
    }
}

impl<'a, T: Clone> IntoIterator for &'a IdTable<T> {
    type Item = (&'a u64, &'a T);
    type IntoIter = Box<dyn Iterator<Item = (&'a u64, &'a T)> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

impl<T: Clone> std::ops::Index<&u64> for IdTable<T> {
    type Output = T;

    fn index(&self, id: &u64) -> &T {
        self.get(id).expect("graph node id present")
    }
}

impl<T: Clone + PartialEq> PartialEq for IdTable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && (self.slots == other.slots || self.iter().eq(other.iter()))
    }
}

#[cfg(test)]
mod tests {
    use super::Chunked;

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
