//! Hash maps and sets keyed by the runtime's own names and node ids.
//!
//! The standard hasher, SipHash, was about a tenth of an event's work. These
//! tables use a multiply-rotate hash instead, seeded once per process so a
//! host cannot choose colliding keys ahead of time. So the order a table
//! walks its keys in changes from one process to the next, and between native
//! and WebAssembly builds. Nothing written may follow it: an outcome, a
//! message, a save or a listing walks a sorted list, an ordered map or set,
//! or the program's statements, never a table's own order.

use std::hash::{BuildHasher, Hasher};
use std::sync::OnceLock;

pub type HashMap<K, V> = std::collections::HashMap<K, V, FastState>;
pub type HashSet<T> = std::collections::HashSet<T, FastState>;

const K: u64 = 0xf135_7aea_2e62_a9c5;

#[derive(Clone, Copy, Debug)]
pub struct FastState(u64);

impl Default for FastState {
    fn default() -> Self {
        static SEED: OnceLock<u64> = OnceLock::new();
        Self(
            *SEED.get_or_init(|| {
                std::collections::hash_map::RandomState::new().hash_one(0x5eed_u64)
            }),
        )
    }
}

impl BuildHasher for FastState {
    type Hasher = FastHasher;
    fn build_hasher(&self) -> FastHasher {
        FastHasher(self.0)
    }
}

pub struct FastHasher(u64);

impl FastHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(K);
    }
}

impl Hasher for FastHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let (chunks, rest) = bytes.as_chunks::<8>();
        for chunk in chunks {
            self.add(u64::from_le_bytes(*chunk));
        }
        if !rest.is_empty() {
            let mut last = [0u8; 8];
            last[..rest.len()].copy_from_slice(rest);
            self.add(u64::from_le_bytes(last));
        }
    }
    #[inline]
    fn write_u8(&mut self, value: u8) {
        self.add(value.into());
    }
    #[inline]
    fn write_u32(&mut self, value: u32) {
        self.add(value.into());
    }
    #[inline]
    fn write_u64(&mut self, value: u64) {
        self.add(value);
    }
    #[inline]
    fn write_usize(&mut self, value: usize) {
        self.add(value as u64);
    }
    #[inline]
    fn finish(&self) -> u64 {
        // Fold the high bits down: hashbrown takes its bucket from the low
        // bits and its tag from the top seven.
        self.0 ^ (self.0 >> 29)
    }
}
