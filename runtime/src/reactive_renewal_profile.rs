//! Native-only attribution outside transactional state. No recorder or clocks
//! exist unless renewal-removal-profile is enabled on a native target.
use super::{ReactiveSession, ReadingOccurrence};
use serde::Serialize;
use std::cell::Cell;
use std::time::Instant;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RemovalProfile {
    pub blocks: u64,
    pub searches: u64,
    pub removes: u64,
    pub matches: u64,
    pub misses: u64,
    /// Exact position predicate calls (index + 1, or original length on miss),
    /// not character comparisons or all internal map comparisons.
    pub probes: u64,
    pub shifted_elements: u64,
    /// Vec header movement estimate only: not allocated/copied payload bytes,
    /// allocator traffic, or COW copies of nested data.
    pub estimated_shifted_header_bytes: u64,
    /// Detaches at this removal boundary, not earlier event/compaction detaches.
    pub shared_cow_detaches: u64,
    /// All entries and occurrence slots in the cloned map, not just this history.
    pub cow_cloned_histories: u64,
    pub cow_cloned_occurrences: u64,
    /// Starts after branch classification and history lookup, immediately before
    /// position; includes search, guarded COW, get_mut, remove and renewal String
    /// drop. ReadingOccurrence moves into the later archive instead. Nested
    /// clocks perturb this time; do not add nested times to the block time.
    pub block_ns: u64,
    pub search_ns: u64,
    /// Only Arc::make_mut calls with an actual shared strong-count detach.
    pub cow_ns: u64,
    /// get_mut + Vec::remove, including the removed renewal String's drop.
    pub remove_ns: u64,
}

impl RemovalProfile {
    const ZERO: Self = Self {
        blocks: 0,
        searches: 0,
        removes: 0,
        matches: 0,
        misses: 0,
        probes: 0,
        shifted_elements: 0,
        estimated_shifted_header_bytes: 0,
        shared_cow_detaches: 0,
        cow_cloned_histories: 0,
        cow_cloned_occurrences: 0,
        block_ns: 0,
        search_ns: 0,
        cow_ns: 0,
        remove_ns: 0,
    };

    fn add(&mut self, other: Self) {
        self.blocks += other.blocks;
        self.searches += other.searches;
        self.removes += other.removes;
        self.matches += other.matches;
        self.misses += other.misses;
        self.probes += other.probes;
        self.shifted_elements += other.shifted_elements;
        self.estimated_shifted_header_bytes += other.estimated_shifted_header_bytes;
        self.shared_cow_detaches += other.shared_cow_detaches;
        self.cow_cloned_histories += other.cow_cloned_histories;
        self.cow_cloned_occurrences += other.cow_cloned_occurrences;
        self.block_ns += other.block_ns;
        self.search_ns += other.search_ns;
        self.cow_ns += other.cow_ns;
        self.remove_ns += other.remove_ns;
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RenewalRemovalProfile {
    pub renewal_removal: RemovalProfile,
    pub readings: RemovalProfile,
    /// Entire depart_unpinned call, including early no-op return, recording,
    /// length-only COW inventory and archive construction. Excludes final
    /// bindings, archive provenance settlement and transaction publication.
    pub departure_ns: u64,
}

impl RenewalRemovalProfile {
    const ZERO: Self = Self {
        renewal_removal: RemovalProfile::ZERO,
        readings: RemovalProfile::ZERO,
        departure_ns: 0,
    };
}

thread_local! {
    static PROFILE: Cell<RenewalRemovalProfile> = const {
        Cell::new(RenewalRemovalProfile::ZERO)
    };
}

impl ReactiveSession {
    /// Reset native thread-local attribution before an apply call.
    pub fn reset_renewal_removal_profile() {
        PROFILE.set(RenewalRemovalProfile::ZERO);
    }

    /// Read and reset attempted work, including subsequently rejected events.
    pub fn take_renewal_removal_profile() -> RenewalRemovalProfile {
        PROFILE.replace(RenewalRemovalProfile::ZERO)
    }
}

pub(super) struct Block {
    reading: bool,
    len: usize,
    started: Instant,
    cow_started: Option<Instant>,
    remove_started: Option<Instant>,
    result: RemovalProfile,
}

impl Block {
    pub(super) fn readings(len: usize) -> Self {
        Self::new(true, len)
    }

    pub(super) fn renewals(len: usize) -> Self {
        Self::new(false, len)
    }

    fn new(reading: bool, len: usize) -> Self {
        Self {
            reading,
            len,
            started: Instant::now(),
            cow_started: None,
            remove_started: None,
            result: RemovalProfile::ZERO,
        }
    }

    pub(super) fn searched(&mut self, index: Option<usize>) {
        self.result.search_ns = self.started.elapsed().as_nanos() as u64;
        self.result.blocks = 1;
        self.result.searches = 1;
        self.result.probes = index.map_or(self.len, |index| index + 1) as u64;
        if let Some(index) = index {
            self.result.matches = 1;
            self.result.shifted_elements = (self.len - index - 1) as u64;
            let header = if self.reading {
                std::mem::size_of::<ReadingOccurrence>()
            } else {
                std::mem::size_of::<String>()
            };
            self.result.estimated_shifted_header_bytes =
                self.result.shifted_elements * header as u64;
        } else {
            self.result.misses = 1;
        }
    }

    pub(super) fn start_cow(&mut self, shared: bool) {
        if shared {
            self.result.shared_cow_detaches = 1;
            self.cow_started = Some(Instant::now());
        }
    }

    pub(super) fn finish_cow(&mut self) {
        if let Some(started) = self.cow_started {
            self.result.cow_ns = started.elapsed().as_nanos() as u64;
        }
        self.remove_started = Some(Instant::now());
    }

    pub(super) fn removed(&mut self) {
        self.result.remove_ns = self.remove_started.unwrap().elapsed().as_nanos() as u64;
        self.result.removes = 1;
    }

    pub(super) fn finish(
        mut self,
        histories: usize,
        remaining_occurrences: impl FnOnce() -> usize,
    ) {
        // Stop before length-only whole-map inventory and the single TLS update.
        // Those diagnostic costs still contribute to departure/apply timing.
        self.result.block_ns = self.started.elapsed().as_nanos() as u64;
        if self.result.shared_cow_detaches != 0 {
            self.result.cow_cloned_histories = histories as u64;
            // A successful branch removes exactly one slot and no history key.
            self.result.cow_cloned_occurrences = remaining_occurrences() as u64 + 1;
        }
        let mut profile = PROFILE.get();
        if self.reading {
            profile.readings.add(self.result);
        } else {
            profile.renewal_removal.add(self.result);
        }
        PROFILE.set(profile);
    }
}

pub(super) struct Departure(Instant);

impl Departure {
    pub(super) fn start() -> Self {
        Self(Instant::now())
    }
}

impl Drop for Departure {
    fn drop(&mut self) {
        let ns = self.0.elapsed().as_nanos() as u64;
        let mut profile = PROFILE.get();
        profile.departure_ns += ns;
        PROFILE.set(profile);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn counters(mut value: RemovalProfile) -> RemovalProfile {
        value.block_ns = 0;
        value.search_ns = 0;
        value.cow_ns = 0;
        value.remove_ns = 0;
        value
    }

    #[test]
    fn renewal_profile_infers_misses_and_reset_without_cow_inventory() {
        ReactiveSession::reset_renewal_removal_profile();
        for names in [vec!["a", "b", "c"], vec![]] {
            let mut block = Block::renewals(names.len());
            block.searched(names.iter().position(|name| *name == "absent"));
            block.finish(2, || panic!("a miss must not inventory or detach the map"));
        }
        let value = ReactiveSession::take_renewal_removal_profile();
        assert_eq!(
            counters(value.renewal_removal),
            RemovalProfile {
                blocks: 2,
                searches: 2,
                misses: 2,
                probes: 3,
                ..RemovalProfile::default()
            }
        );
        assert_eq!(value.readings, RemovalProfile::default());
        assert_eq!(
            ReactiveSession::take_renewal_removal_profile(),
            RenewalRemovalProfile::default()
        );
        let mut block = Block::renewals(0);
        block.searched(None);
        block.finish(0, || unreachable!());
        ReactiveSession::reset_renewal_removal_profile();
        assert_eq!(
            ReactiveSession::take_renewal_removal_profile(),
            RenewalRemovalProfile::default()
        );
    }

    #[test]
    fn renewal_profile_counts_shared_map_and_unshared_removal_separately() {
        ReactiveSession::reset_renewal_removal_profile();
        let mut histories = Arc::new(BTreeMap::from([
            (
                "a",
                vec!["a".to_owned(), "a@2".to_owned(), "a@3".to_owned()],
            ),
            ("b", vec!["b".to_owned(), "b@2".to_owned()]),
        ]));
        let original = histories.clone();
        for record in ["a@2", "a@3"] {
            let names = &histories["a"];
            let mut block = Block::renewals(names.len());
            let index = names.iter().position(|name| name == record);
            block.searched(index);
            block.start_cow(Arc::strong_count(&histories) > 1);
            let map = Arc::make_mut(&mut histories);
            block.finish_cow();
            map.get_mut("a").unwrap().remove(index.unwrap());
            block.removed();
            block.finish(histories.len(), || histories.values().map(Vec::len).sum());
        }
        let value = ReactiveSession::take_renewal_removal_profile().renewal_removal;
        assert_eq!(
            counters(value),
            RemovalProfile {
                blocks: 2,
                searches: 2,
                removes: 2,
                matches: 2,
                probes: 4,
                shifted_elements: 1,
                estimated_shifted_header_bytes: std::mem::size_of::<String>() as u64,
                shared_cow_detaches: 1,
                cow_cloned_histories: 2,
                cow_cloned_occurrences: 5,
                ..RemovalProfile::default()
            }
        );
        assert_eq!(original["a"].len(), 3);
        assert_eq!(histories["a"], ["a"]);
        // A solely unshared pass must report no COW time or inventory.
        let mut block = Block::renewals(histories["b"].len());
        block.searched(Some(1));
        block.start_cow(Arc::strong_count(&histories) > 1);
        let map = Arc::make_mut(&mut histories);
        block.finish_cow();
        map.get_mut("b").unwrap().remove(1);
        block.removed();
        block.finish(histories.len(), || {
            panic!("unshared removal must not inventory")
        });
        let value = ReactiveSession::take_renewal_removal_profile().renewal_removal;
        assert_eq!(value.shared_cow_detaches, 0);
        assert_eq!(value.cow_ns, 0);
        assert_eq!(value.cow_cloned_occurrences, 0);
        assert_eq!(value.shifted_elements, 0);
    }

    #[test]
    fn renewal_profile_retains_rejected_work_and_exact_session_rollback() {
        let source =
            include_str!("../../experiments/departure-gate/collector-fixtures/release-mutual.cav");
        let mut session = ReactiveSession::from_source(source).unwrap();
        for _ in 0..4 {
            session.apply("cycle", &BTreeMap::new()).unwrap();
            session.drain_archive();
        }
        let saved = session.save_json().unwrap();
        ReactiveSession::reset_renewal_removal_profile();
        assert!(session.apply("fail", &BTreeMap::new()).is_err());
        let rejected = ReactiveSession::take_renewal_removal_profile();
        assert_eq!(
            counters(rejected.renewal_removal),
            RemovalProfile {
                blocks: 6,
                searches: 6,
                removes: 6,
                matches: 6,
                probes: 12,
                shifted_elements: 12,
                estimated_shifted_header_bytes: 12 * std::mem::size_of::<String>() as u64,
                shared_cow_detaches: 1,
                cow_cloned_histories: 2,
                cow_cloned_occurrences: 10,
                ..RemovalProfile::default()
            }
        );
        assert_eq!(rejected.readings, RemovalProfile::default());
        assert_eq!(session.save_json().unwrap(), saved);
        assert_eq!(session.undrained(), 0);
        assert_eq!(
            ReactiveSession::take_renewal_removal_profile(),
            RenewalRemovalProfile::default()
        );
        session.apply("release", &BTreeMap::new()).unwrap();
        let accepted = ReactiveSession::take_renewal_removal_profile();
        assert_eq!(
            counters(accepted.renewal_removal),
            counters(rejected.renewal_removal)
        );
        assert_eq!(
            session
                .drain_archive()
                .iter()
                .filter(|item| item.record().is_some())
                .count(),
            6
        );
        session.apply("release", &BTreeMap::new()).unwrap();
        let idle = ReactiveSession::take_renewal_removal_profile();
        assert_eq!(idle.renewal_removal, RemovalProfile::default());
        assert_eq!(idle.readings, RemovalProfile::default());
    }

    #[test]
    fn renewal_profile_shared_reading_map_inventory_counts_all_streams() {
        let source = r#"
claim seen; evidence sensor from "sensor";
readings r from sensor limit 3; readings s from sensor limit 3;
event read;
on read sample r = 7 supports seen;
on read sample s = 8 supports seen;
"#;
        let mut session = ReactiveSession::from_source(source).unwrap();
        for _ in 0..2 {
            session.apply("read", &BTreeMap::new()).unwrap();
        }
        let mut streams = session.reading_streams.clone();
        ReactiveSession::reset_renewal_removal_profile();
        let occurrences = &streams["r"].occurrences;
        let mut block = Block::readings(occurrences.len());
        let index = occurrences.iter().position(|reading| reading.id == "r@1");
        block.searched(index);
        block.start_cow(Arc::strong_count(&streams) > 1);
        let map = Arc::make_mut(&mut streams);
        block.finish_cow();
        let reading = map.get_mut("r").unwrap().occurrences.remove(index.unwrap());
        block.removed();
        block.finish(streams.len(), || {
            streams.values().map(|s| s.occurrences.len()).sum()
        });
        let profile = ReactiveSession::take_renewal_removal_profile();
        assert_eq!(
            counters(profile.readings),
            RemovalProfile {
                blocks: 1,
                searches: 1,
                removes: 1,
                matches: 1,
                probes: 1,
                shifted_elements: 1,
                estimated_shifted_header_bytes: std::mem::size_of::<ReadingOccurrence>() as u64,
                shared_cow_detaches: 1,
                cow_cloned_histories: 2,
                cow_cloned_occurrences: 4,
                ..RemovalProfile::default()
            }
        );
        assert_eq!(profile.renewal_removal, RemovalProfile::default());
        assert_eq!(session.reading_streams["r"].occurrences.len(), 2);
        assert_eq!(streams["r"].occurrences.len(), 1);
        assert_eq!(reading.id, "r@1");
        assert_eq!(reading.value, 7.0);
    }

    #[test]
    fn renewal_profile_reading_move_uses_reading_headers_and_rolls_back() {
        let source = r#"
claim seen;
evidence sensor from "sensor";
readings r from sensor window 1;
state held = 0; state divisor = 1;
event sample; event hold; event release; event fail;
on sample sample r = 7 supports seen;
on hold set held = latest(r);
on release set held = 0;
on fail set held = 0;
on fail set divisor = 0;
bind hud.guard = 1 / divisor;
"#;
        let mut session = ReactiveSession::from_source(source).unwrap();
        for event in ["sample", "hold", "sample"] {
            session.apply(event, &BTreeMap::new()).unwrap();
            session.drain_archive();
        }
        let saved = session.save_json().unwrap();
        ReactiveSession::reset_renewal_removal_profile();
        assert!(session.apply("fail", &BTreeMap::new()).is_err());
        let rejected = ReactiveSession::take_renewal_removal_profile();
        assert_eq!(
            counters(rejected.readings),
            RemovalProfile {
                blocks: 1,
                searches: 1,
                removes: 1,
                matches: 1,
                probes: 1,
                shifted_elements: 1,
                estimated_shifted_header_bytes: std::mem::size_of::<ReadingOccurrence>() as u64,
                // Earlier provenance compaction detached this reading map.
                // Removal-boundary attribution must not count that work again.
                ..RemovalProfile::default()
            }
        );
        assert_eq!(rejected.renewal_removal, RemovalProfile::default());
        assert_eq!(rejected.readings.cow_ns, 0);
        assert_eq!(session.save_json().unwrap(), saved);
        assert_eq!(session.undrained(), 0);
        session.apply("release", &BTreeMap::new()).unwrap();
        let accepted = ReactiveSession::take_renewal_removal_profile();
        assert_eq!(counters(accepted.readings), counters(rejected.readings));
        let archive = session.drain_archive();
        let record = archive.iter().find_map(|item| item.record()).unwrap();
        assert_eq!(record.record, "r@1");
        let reading = record.reading.as_ref().unwrap();
        assert_eq!(reading.id, "r@1");
        assert_eq!(reading.value, 7.0);
    }
}
