//! A map from names to node ids that a copy shares. A session copies itself
//! before each event so that a refused event leaves it untouched, and a hash
//! table of every name would be copied whole by the event's first new name.
//! Here the names sit in buckets by the top bits of their hash, in an
//! [`IdMap`]: a copy shares every block, and a write copies the blocks on its
//! way down to one bucket, and that bucket's short list.
//!
//! Names are hashed with the process's seed, as the runtime's hash tables
//! are, so the order the map lists them in is not an order anything may rely
//! on. Two names can share a bucket, and some ordinary names share a whole
//! hash whatever the seed, so a bucket holds a list and a lookup compares
//! names.
use crate::fast_hash::FastState;
use crate::idmap::IdMap;
use crate::NodeId;
use std::hash::BuildHasher;
use std::sync::Arc;

/// How many top bits of a name's hash pick its bucket. Twenty bits keep the
/// tree four levels deep and buckets of one for any session's names.
const BUCKET_BITS: u32 = 20;

type Bucket = Vec<(Arc<str>, NodeId)>;

#[derive(Clone, Default)]
pub struct NameMap {
    buckets: IdMap<Bucket>,
    len: usize,
}

fn bucket_of(name: &str) -> u64 {
    // The hasher's last step multiplies, so its top bits are its best mixed.
    FastState::default().hash_one(name) >> (u64::BITS - BUCKET_BITS)
}

impl NameMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// How many names the map holds.
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The node named `name`.
    pub fn get(&self, name: &str) -> Option<&NodeId> {
        self.buckets
            .get(&bucket_of(name))?
            .iter()
            .find(|(held, _)| &**held == name)
            .map(|(_, id)| id)
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// The name and the node of each entry, in no order to rely on.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &NodeId)> + '_ {
        self.buckets
            .values()
            .flat_map(|bucket| bucket.iter().map(|(name, id)| (&**name, id)))
    }

    /// Each entry's name as the map holds it, shared.
    pub fn entries(&self) -> impl Iterator<Item = (&Arc<str>, NodeId)> + '_ {
        self.buckets
            .values()
            .flat_map(|bucket| bucket.iter().map(|(name, id)| (name, *id)))
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> + '_ {
        self.iter().map(|(name, _)| name)
    }

    pub fn values(&self) -> impl Iterator<Item = &NodeId> + '_ {
        self.iter().map(|(_, id)| id)
    }

    /// Names `id` by `name`, and returns the node `name` named before.
    pub fn insert(&mut self, name: Arc<str>, id: NodeId) -> Option<NodeId> {
        let at = bucket_of(&name);
        if let Some(bucket) = self.buckets.get_mut(&at) {
            if let Some(held) = bucket.iter_mut().find(|(held, _)| *held == name) {
                return Some(std::mem::replace(&mut held.1, id));
            }
            bucket.push((name, id));
        } else {
            self.buckets.insert(at, vec![(name, id)]);
        }
        self.len += 1;
        None
    }

    /// Takes `name` out, and returns the node it named.
    pub fn remove(&mut self, name: &str) -> Option<NodeId> {
        let at = bucket_of(name);
        // A name the map lacks leaves every block shared.
        let place = self
            .buckets
            .get(&at)?
            .iter()
            .position(|(held, _)| &**held == name)?;
        let bucket = self.buckets.get_mut(&at).expect("found above");
        let (_, id) = bucket.swap_remove(place);
        if bucket.is_empty() {
            self.buckets.remove(&at);
        }
        self.len -= 1;
        Some(id)
    }

    /// Whether the two maps share every block, and so hold the same names
    /// without either being read.
    pub fn shares_all(&self, other: &Self) -> bool {
        self.buckets.shares_all(&other.buckets)
    }
}

/// Two maps are equal when they name the same nodes by the same names.
impl PartialEq for NameMap {
    fn eq(&self, other: &Self) -> bool {
        self.shares_all(other)
            || (self.len == other.len && self.iter().all(|(name, id)| other.get(name) == Some(id)))
    }
}

impl Eq for NameMap {}

impl std::ops::Index<&str> for NameMap {
    type Output = NodeId;

    fn index(&self, name: &str) -> &NodeId {
        self.get(name).expect("a name the map holds")
    }
}

impl std::fmt::Debug for NameMap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut entries = self.iter().collect::<Vec<_>>();
        entries.sort_unstable();
        formatter.debug_map().entries(entries).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fast_hash::HashMap;

    #[test]
    fn the_map_reads_as_a_hash_map_of_the_same_writes() {
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = move |below: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % below
        };
        let mut map = NameMap::new();
        let mut plain = HashMap::<String, NodeId>::default();
        let mut copies = Vec::new();
        for step in 0..6_000u64 {
            // Names that differ only late, as `record@N` names do.
            let name = format!("record@{}", next(2_500));
            if next(4) == 0 {
                assert_eq!(map.remove(&name), plain.remove(&name), "remove {name}");
            } else {
                assert_eq!(
                    map.insert(Arc::from(name.as_str()), step),
                    plain.insert(name.clone(), step),
                    "insert {name}"
                );
            }
            assert_eq!(map.len(), plain.len());
            if step % 500 == 0 {
                copies.push((map.clone(), plain.clone()));
            }
        }
        // Each copy kept what it held when it was taken.
        for (copy, plain) in copies.iter().chain([(map.clone(), plain.clone())].iter()) {
            assert_eq!(copy.len(), plain.len());
            for (name, id) in plain {
                assert_eq!(copy.get(name), Some(id));
                assert_eq!(copy[name.as_str()], *id);
            }
            let mut listed = copy
                .iter()
                .map(|(name, id)| (name.to_owned(), *id))
                .collect::<Vec<_>>();
            listed.sort_unstable();
            let mut expected = plain
                .iter()
                .map(|(name, id)| (name.clone(), *id))
                .collect::<Vec<_>>();
            expected.sort_unstable();
            assert_eq!(listed, expected);
            assert!(!copy.contains_key("never@named"));
        }
    }

    #[test]
    fn names_that_share_a_bucket_are_told_apart() {
        // Find two names in one bucket.
        let mut seen = HashMap::<u64, String>::default();
        let (one, other) = (0..)
            .find_map(|n: u64| {
                let name = format!("n{n}");
                let at = bucket_of(&name);
                match seen.get(&at) {
                    Some(earlier) => Some((earlier.clone(), name)),
                    None => {
                        seen.insert(at, name);
                        None
                    }
                }
            })
            .unwrap();
        let mut map = NameMap::new();
        map.insert(Arc::from(one.as_str()), 1);
        map.insert(Arc::from(other.as_str()), 2);
        assert_eq!((map.get(&one), map.get(&other)), (Some(&1), Some(&2)));
        assert_eq!(map.remove(&one), Some(1));
        assert_eq!((map.get(&one), map.get(&other)), (None, Some(&2)));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn a_copy_shares_until_written_and_a_missing_name_writes_nothing() {
        let mut map = NameMap::new();
        for n in 0..300u64 {
            map.insert(Arc::from(format!("name{n}").as_str()), n);
        }
        let mut copy = map.clone();
        assert!(copy.shares_all(&map));
        assert_eq!(copy.remove("absent"), None);
        assert!(copy.shares_all(&map), "a name the map lacks copies nothing");
        copy.insert(Arc::from("name7"), 7);
        assert!(!copy.shares_all(&map));
        assert_eq!(copy, map, "the same names for the same nodes");
        copy.insert(Arc::from("name7"), 8);
        assert_ne!(copy, map);
    }
}
