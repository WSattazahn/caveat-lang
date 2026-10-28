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
