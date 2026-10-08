//! Draft withdrawal collection. Exact references veto only additional
//! departures; they never become ordinary pins. Owners make record-local
//! metadata conditional, so an unreachable cycle cannot root itself.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CollectorOwner {
    State(String),
    Record(String),
    Stream(String),
    Series(String),
    Graph(String),
    Pending(String),
}

impl CollectorOwner {
    fn record(&self) -> Option<&str> {
        match self {
            Self::Record(name) | Self::Graph(name) => Some(name),
            _ => None,
        }
    }
}

// Keep the common low-degree indexes compact without making high-degree
// mutation linear. All forms have the same sorted, unique iteration order;
// canonical promotion/demotion also makes equality independent of history.
const SMALL_REFERENCE_LIMIT: usize = 8;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum CollectorSet<T> {
    #[default]
    Empty,
    One(T),
    Small(Vec<T>),
    Many(BTreeSet<T>),
}

impl<T: Ord> CollectorSet<T> {
    fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::One(_) => 1,
            Self::Small(values) => values.len(),
            Self::Many(values) => values.len(),
        }
    }

    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        let (small, many): (&[T], Option<&BTreeSet<T>>) = match self {
            Self::Empty => (&[], None),
            Self::One(value) => (std::slice::from_ref(value), None),
            Self::Small(values) => (values, None),
            Self::Many(values) => (&[], Some(values)),
        };
        small.iter().chain(many.into_iter().flatten())
    }

    fn insert(&mut self, value: T) -> bool {
        match self {
            Self::Empty => *self = Self::One(value),
            Self::One(existing) if *existing == value => return false,
            Self::One(_) => {
                let Self::One(existing) = std::mem::take(self) else {
                    unreachable!()
                };
                let pair = if existing < value {
                    [existing, value]
                } else {
                    [value, existing]
                };
                *self = Self::Small(Vec::from(pair));
            }
            Self::Small(values) => match values.binary_search(&value) {
                Ok(_) => return false,
                Err(index) if values.len() < SMALL_REFERENCE_LIMIT => values.insert(index, value),
                Err(_) => {
                    let mut many: BTreeSet<_> = std::mem::take(values).into_iter().collect();
                    many.insert(value);
                    *self = Self::Many(many);
                }
            },
            Self::Many(values) => return values.insert(value),
        }
        true
    }

    fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match self {
            Self::Empty => false,
            Self::One(existing) => {
                if <T as std::borrow::Borrow<Q>>::borrow(existing) != value {
                    return false;
                }
                *self = Self::Empty;
                true
            }
            Self::Small(values) => {
                let Ok(index) = values.binary_search_by(|existing| existing.borrow().cmp(value))
                else {
                    return false;
                };
                values.remove(index);
                if values.len() == 1 {
                    *self = Self::One(values.pop().unwrap());
                }
                true
            }
            Self::Many(values) => {
                if !values.remove(value) {
                    return false;
                }
                if values.len() == SMALL_REFERENCE_LIMIT {
                    *self = Self::Small(std::mem::take(values).into_iter().collect());
                }
                true
            }
        }
    }
}

impl<T: Ord> Extend<T> for CollectorSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, values: I) {
        for value in values {
            self.insert(value);
        }
    }
}

impl<T: Ord> FromIterator<T> for CollectorSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        let mut references = Self::default();
        references.extend(values);
        references
    }
}

#[cfg(test)]
impl<T: Ord, const N: usize> From<[T; N]> for CollectorSet<T> {
    fn from(values: [T; N]) -> Self {
        values.into_iter().collect()
    }
}

/// Diagnostics describe the last accepted event, not rejected attempts.
/// `peak_work_items` counts live algorithm work-set entries, not heap bytes.
#[cfg(feature = "collector-metrics")]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize)]
pub struct WithdrawalCollectionMetrics {
    pub dirty_owners: usize,
    pub references_scanned: usize,
    pub vertices_visited: usize,
    pub edges_visited: usize,
    pub candidates_examined: usize,
    pub collected: usize,
    pub peak_work_items: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct WithdrawalCollector {
    owners: Arc<BTreeMap<CollectorOwner, CollectorSet<String>>>,
    incoming: Arc<BTreeMap<String, CollectorSet<CollectorOwner>>>,
    // Only multiplicity is needed to distinguish withdrawal pins from roots.
    // Exact subject -> reason identities remain in withdrawal_reasons and
    // Record owner references, and in the authoritative withdrawal records.
    reasons: Arc<BTreeMap<String, usize>>,
    pending: Arc<BTreeMap<String, BTreeMap<String, usize>>>,
    dirty: BTreeSet<CollectorOwner>,
    queued: BTreeSet<String>,
    #[cfg(feature = "collector-metrics")]
    metrics: WithdrawalCollectionMetrics,
}

macro_rules! metric {
    ($collector:expr, $field:ident, $count:expr) => {
        #[cfg(feature = "collector-metrics")]
        {
            $collector.metrics.$field += $count;
        }
    };
}

impl WithdrawalCollector {
    pub(super) fn queue(&mut self, record: &str) {
        self.queued.insert(record.into());
    }

    fn replace(&mut self, owner: CollectorOwner, references: CollectorSet<String>) {
        // Compare borrowed sets before any copy-on-write, including when this
        // owner has no references. Keep the old set borrowed for the diff too.
        if self
            .owners
            .get(&owner)
            .map_or(references.is_empty(), |previous| previous == &references)
        {
            return;
        }
        if let Some(record) = owner.record() {
            self.queue(record);
        }
        let incoming = Arc::make_mut(&mut self.incoming);
        {
            // Both iterators are ordered. Merge once instead of looking up
            // each old/new member again when an owner has many references.
            let mut previous = self
                .owners
                .get(&owner)
                .into_iter()
                .flat_map(CollectorSet::iter)
                .peekable();
            let mut next = references.iter().peekable();
            loop {
                let order = match (previous.peek(), next.peek()) {
                    (Some(before), Some(after)) => before.cmp(after),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => break,
                };
                let (name, add) = match order {
                    std::cmp::Ordering::Less => (previous.next().unwrap(), false),
                    std::cmp::Ordering::Greater => (next.next().unwrap(), true),
                    std::cmp::Ordering::Equal => {
                        previous.next();
                        next.next();
                        continue;
                    }
                };
                self.queued.insert(name.clone());
                if add {
                    incoming
                        .entry(name.clone())
                        .or_default()
                        .insert(owner.clone());
                } else {
                    let owners = incoming.get_mut(name).expect("indexed reference");
                    owners.remove(&owner);
                    if owners.is_empty() {
                        incoming.remove(name);
                    }
                }
            }
        }
        let owners = Arc::make_mut(&mut self.owners);
        if references.is_empty() {
            owners.remove(&owner);
        } else {
            owners.insert(owner, references);
        }
    }

    pub(super) fn withdrawal(&mut self, subject: &str, reason: &str, add: bool) {
        let reverse = Arc::make_mut(&mut self.reasons);
        // Callers add only the first withdrawal of a subject, and remove only
        // after removing its authoritative subject -> reason entry. Duplicate
        // language effects therefore never increment this derived count.
        if add {
            *reverse.entry(reason.into()).or_default() += 1;
        } else {
            let count = reverse.get_mut(reason).expect("indexed withdrawal reason");
            *count -= 1;
            if *count == 0 {
                reverse.remove(reason);
            }
        }
        self.dirty.insert(CollectorOwner::Record(subject.into()));
        self.queue(subject);
        self.queue(reason);
    }

    fn peak(&mut self, count: usize) {
        #[cfg(feature = "collector-metrics")]
        {
            self.metrics.peak_work_items = self.metrics.peak_work_items.max(count);
        }
        #[cfg(not(feature = "collector-metrics"))]
        let _ = count;
    }

    fn forget(&mut self, departing: &BTreeMap<String, (String, u64)>) {
        // Compaction removes exactly these names from all surviving holders.
        // Visit their inverse references instead of scanning the session.
        for record in departing.keys() {
            let incoming = self.incoming.get(record).cloned().unwrap_or_default();
            for owner in incoming.iter() {
                metric!(self, references_scanned, 1);
                if let CollectorOwner::Pending(target) = owner {
                    let pending = Arc::make_mut(&mut self.pending);
                    if let Some(refs) = pending.get_mut(target) {
                        refs.remove(record);
                        if refs.is_empty() {
                            pending.remove(target);
                        }
                    }
                }
                if let Some(refs) = Arc::make_mut(&mut self.owners).get_mut(owner) {
                    refs.remove(record);
                    if refs.is_empty() {
                        Arc::make_mut(&mut self.owners).remove(owner);
                    }
                }
            }
            // Unrelated departures must not copy the shared reverse index
            // merely to remove a key that is absent from it.
            if self.incoming.contains_key(record) {
                Arc::make_mut(&mut self.incoming).remove(record);
            }
        }
        for record in departing.keys() {
            self.replace(
                CollectorOwner::Record(record.clone()),
                CollectorSet::default(),
            );
            self.replace(
                CollectorOwner::Graph(record.clone()),
                CollectorSet::default(),
            );
        }
        self.dirty.clear();
        // Every affected component was considered against the full prospective
        // batch already; no established work is deferred to a later event.
        self.queued.clear();
    }
}

impl ReactiveSession {
    #[cfg(feature = "collector-metrics")]
    pub fn withdrawal_collection_metrics(&self) -> WithdrawalCollectionMetrics {
        self.collector.metrics
    }

    pub(super) fn collector_begin(&mut self) {
        #[cfg(feature = "collector-metrics")]
        {
            self.collector.metrics = WithdrawalCollectionMetrics::default();
        }
    }

    pub(super) fn collector_dirty(&mut self, owner: CollectorOwner) {
        if self.windowed() {
            self.collector.dirty.insert(owner);
        }
    }

    pub(super) fn collector_effect(&mut self, effect: &Effect) {
        if !self.windowed() {
            return;
        }
        let owner = match effect {
            Effect::Set { name, .. } => CollectorOwner::State(name.clone()),
            Effect::Sample { stream, .. } => {
                self.collector_dirty(CollectorOwner::Stream(stream.clone()));
                let current = self.reading_streams[stream].current.clone();
                if let Some(current) = current {
                    self.collector_dirty(CollectorOwner::Record(current));
                }
                return;
            }
            Effect::Reveal { evidence, .. }
            | Effect::Observe { evidence }
            | Effect::Renew { evidence }
            | Effect::Qualify { evidence, .. } => {
                CollectorOwner::Record(self.occurrence(evidence).into())
            }
            Effect::Examine { caveat, .. } => CollectorOwner::Record(caveat.clone()),
            Effect::Commit { action, .. } | Effect::Reopen { action, .. } => {
                self.collector_dirty(CollectorOwner::Series(action.clone()));
                let current = self
                    .decision_series
                    .get(action)
                    .and_then(|series| series.current.clone())
                    .unwrap_or_else(|| action.clone());
                CollectorOwner::Record(current)
            }
            Effect::Withdraw { target, .. } => {
                let Some(name) = self.withdrawal_target(target) else {
                    return;
                };
                CollectorOwner::Record(name)
            }
            _ => return,
        };
        self.collector_dirty(owner);
    }

    pub(super) fn collector_schedule(&mut self, scheduled: &ScheduledQualification, add: bool) {
        if !self.windowed() {
            return;
        }
        metric!(
            self.collector,
            references_scanned,
            scheduled.guard.evidence.len()
        );
        let pending = Arc::make_mut(&mut self.collector.pending);
        let references = pending.entry(scheduled.evidence.clone()).or_default();
        for name in &scheduled.guard.evidence {
            if add {
                *references.entry(name.clone()).or_default() += 1;
            } else {
                let count = references.get_mut(name).expect("scheduled dependency");
                *count -= 1;
                if *count == 0 {
                    references.remove(name);
                }
            }
        }
        let refs = references.keys().cloned().collect();
        if references.is_empty() {
            pending.remove(&scheduled.evidence);
        }
        self.collector
            .replace(CollectorOwner::Pending(scheduled.evidence.clone()), refs);
    }

    pub(super) fn collector_reopens(&mut self, action: &str, reasons: &[String]) {
        // ReliesOn sources and Reopens targets are commitments, which do not
        // depart. Graph owners therefore only supply external roots; candidate
        // dependency traversal needs Record owners alone.
        debug_assert!(!self.retired.contains_key(action));
        if !self.windowed() {
            return;
        }
        let owner = CollectorOwner::Graph(action.into());
        let mut refs = self
            .collector
            .owners
            .get(&owner)
            .cloned()
            .unwrap_or_default();
        refs.extend(reasons.iter().cloned());
        metric!(self.collector, references_scanned, reasons.len());
        self.collector.replace(owner, refs);
    }

    fn collector_references(&self, owner: &CollectorOwner) -> (CollectorSet<String>, usize) {
        let mut references = CollectorSet::default();
        let mut scanned = 0;
        let mut include = |provenance: &Provenance| {
            scanned += provenance.evidence.len();
            references.extend(provenance.evidence.iter().cloned());
        };
        match owner {
            CollectorOwner::State(name) => {
                if let Some(cell) = self.states.get(name) {
                    include(&cell.value.provenance);
                    include(&cell.grounds);
                }
            }
            CollectorOwner::Stream(name) => {
                if let Some(stream) = self.reading_streams.get(name) {
                    include(&stream.selection_qualifications);
                }
            }
            CollectorOwner::Series(name) => {
                if let Some(series) = self.decision_series.get(name) {
                    include(&series.selection_qualifications);
                }
            }
            CollectorOwner::Record(name) => {
                for map in [
                    &self.observation_qualifications,
                    &self.examination_qualifications,
                    &self.reopening_qualifications,
                ] {
                    if let Some(provenance) = map.get(name) {
                        include(provenance);
                    }
                }
                for targets in self.predicate_qualifications.values() {
                    if let Some(provenance) = targets.get(name) {
                        include(provenance);
                    }
                }
                if let Some(basis) = self.commitment_bases.get(name) {
                    include(&basis.provenance);
                }
                if let Some(grounds) = self.commitment_grounds.get(name) {
                    include(grounds);
                }
                if let Some((history, number)) = save::occurrence_parts(name) {
                    if let Some(stream) = self.reading_streams.get(history) {
                        if let Ok(index) = stream
                            .occurrences
                            .binary_search_by_key(&(number as u64), |reading| reading.ordinal)
                        {
                            include(&stream.occurrences[index].provenance);
                        }
                    }
                }
                if let Some(reason) = self.withdrawal_reasons.get(name) {
                    scanned += 1;
                    references.insert(reason.clone());
                }
            }
            CollectorOwner::Graph(_) | CollectorOwner::Pending(_) => {
                references = self
                    .collector
                    .owners
                    .get(owner)
                    .cloned()
                    .unwrap_or_default();
                scanned += references.len();
            }
        }
        (references, scanned)
    }

    pub(super) fn rebuild_withdrawal_collector(&mut self) {
        self.collector = WithdrawalCollector::default();
        if !self.windowed() {
            return;
        }
        let mut owners = BTreeSet::new();
        owners.extend(self.states.names().cloned().map(CollectorOwner::State));
        owners.extend(
            self.reading_streams
                .keys()
                .cloned()
                .map(CollectorOwner::Stream),
        );
        owners.extend(
            self.decision_series
                .keys()
                .cloned()
                .map(CollectorOwner::Series),
        );
        owners.extend(self.symbols.keys().cloned().map(CollectorOwner::Record));
        // A skipped one-off commitment has predicate metadata before its
        // symbol exists. Rebuild from actual holder keys as well as symbols.
        for map in [
            &self.observation_qualifications,
            &self.examination_qualifications,
            &self.reopening_qualifications,
        ] {
            owners.extend(map.keys().cloned().map(CollectorOwner::Record));
        }
        for targets in self.predicate_qualifications.values() {
            owners.extend(targets.keys().cloned().map(CollectorOwner::Record));
        }
        owners.extend(
            self.commitment_bases
                .keys()
                .cloned()
                .map(CollectorOwner::Record),
        );
        owners.extend(
            self.commitment_grounds
                .keys()
                .cloned()
                .map(CollectorOwner::Record),
        );
        for reason in self.withdrawal_reasons.values() {
            *Arc::make_mut(&mut self.collector.reasons)
                .entry(reason.clone())
                .or_default() += 1;
        }
        for owner in owners {
            self.collector
                .replace(owner.clone(), self.collector_references(&owner).0);
        }
        for scheduled in self.scheduled.as_ref().clone() {
            self.collector_schedule(&scheduled, true);
        }
        let names = self
            .symbols
            .iter()
            .map(|(name, id)| (*id, name.clone()))
            .collect::<HashMap<_, _>>();
        for edge in self.graph.edges.clone() {
            if edge.relation == Relation::Reopens {
                self.collector_reopens(&names[&edge.to], &[names[&edge.from].clone()]);
            } else if edge.relation == Relation::ReliesOn {
                self.collector_reopens(&names[&edge.from], &[names[&edge.to].clone()]);
            }
        }
        self.collector.queued.extend(self.retired.keys().cloned());
        self.collector.dirty.clear();
        self.collector_begin();
    }

    fn collector_eligible(&self, name: &str, ordinary: &BTreeMap<String, (String, u64)>) -> bool {
        self.retired.contains_key(name)
            && !ordinary.contains_key(name)
            && self.departable(name).is_some()
    }

    /// Trace only changed connected candidate regions. Traversal is undirected
    /// to discover external roots, then directed to retain their dependencies.
    /// A root release can expose an entire old region; no work cap changes the
    /// event's acceptance or defers established departures.
    pub(super) fn collect_withdrawals(&mut self, departing: &mut BTreeMap<String, (String, u64)>) {
        if !self.windowed() {
            return;
        }
        for owner in std::mem::take(&mut self.collector.dirty) {
            let (refs, _scanned) = self.collector_references(&owner);
            metric!(self.collector, dirty_owners, 1);
            metric!(self.collector, references_scanned, _scanned);
            self.collector.replace(owner, refs);
        }
        for record in departing.keys() {
            if let Some(refs) = self
                .collector
                .owners
                .get(&CollectorOwner::Record(record.clone()))
            {
                self.collector.queued.extend(refs.iter().cloned());
            }
        }
        let mut queued = std::mem::take(&mut self.collector.queued);
        let mut considered = BTreeSet::new();
        let mut additional = BTreeSet::new();
        self.collector.peak(queued.len());
        while let Some(seed) = queued.pop_first() {
            metric!(self.collector, candidates_examined, 1);
            if considered.contains(&seed) || !self.collector_eligible(&seed, departing) {
                continue;
            }
            let mut pending = BTreeSet::from([seed]);
            let mut region = BTreeSet::new();
            let mut roots = BTreeSet::new();
            while let Some(record) = pending.pop_first() {
                if !region.insert(record.clone()) {
                    continue;
                }
                metric!(self.collector, vertices_visited, 1);
                let reason_pins = self.collector.reasons.get(&record).copied().unwrap_or(0);
                if self.pin_counts.get(&record).copied().unwrap_or(0) > reason_pins {
                    roots.insert(record.clone());
                }
                if let Some(incoming) = self.collector.incoming.get(&record) {
                    for owner in incoming.iter() {
                        metric!(self.collector, edges_visited, 1);
                        match owner.record() {
                            Some(name) if departing.contains_key(name) => {}
                            Some(name) if self.collector_eligible(name, departing) => {
                                if !region.contains(name) {
                                    pending.insert(name.to_string());
                                }
                            }
                            _ => {
                                roots.insert(record.clone());
                            }
                        }
                    }
                }
                if let Some(outgoing) = self
                    .collector
                    .owners
                    .get(&CollectorOwner::Record(record.clone()))
                {
                    for name in outgoing.iter() {
                        metric!(self.collector, edges_visited, 1);
                        if self.collector_eligible(name, departing) && !region.contains(name) {
                            pending.insert(name.clone());
                        }
                    }
                }
                self.collector.peak(
                    queued.len()
                        + pending.len()
                        + region.len()
                        + roots.len()
                        + considered.len()
                        + additional.len(),
                );
            }
            let mut reachable = BTreeSet::new();
            while let Some(record) = roots.pop_first() {
                if !reachable.insert(record.clone()) {
                    continue;
                }
                if let Some(outgoing) = self.collector.owners.get(&CollectorOwner::Record(record)) {
                    for name in outgoing.iter() {
                        metric!(self.collector, edges_visited, 1);
                        if region.contains(name) && !reachable.contains(name) {
                            roots.insert(name.clone());
                        }
                    }
                }
                self.collector.peak(
                    queued.len()
                        + region.len()
                        + roots.len()
                        + reachable.len()
                        + considered.len()
                        + additional.len(),
                );
            }
            additional.extend(region.difference(&reachable).cloned());
            self.collector.peak(
                queued.len() + region.len() + reachable.len() + considered.len() + additional.len(),
            );
            considered.extend(region);
        }
        metric!(self.collector, collected, additional.len());
        for record in additional {
            if let Some(reason) = Arc::make_mut(&mut self.withdrawal_reasons).remove(&record) {
                self.collector.withdrawal(&record, &reason, false);
                self.release_pin(&reason);
            }
            departing.insert(
                record.clone(),
                self.departable(&record).expect("eligible candidate"),
            );
        }
    }

    pub(super) fn collector_departed(&mut self, departing: &BTreeMap<String, (String, u64)>) {
        self.collector.forget(departing);
        Arc::make_mut(&mut self.departure_candidates)
            .retain(|record| !departing.contains_key(record));
    }

    #[cfg(debug_assertions)]
    pub(super) fn check_withdrawal_collector(&self) {
        if !self.windowed() {
            return;
        }
        let mut rebuilt = self.clone();
        rebuilt.rebuild_withdrawal_collector();
        debug_assert_eq!(
            self.collector.owners, rebuilt.collector.owners,
            "collector owner references"
        );
        debug_assert_eq!(
            self.collector.incoming, rebuilt.collector.incoming,
            "collector inverse references"
        );
        debug_assert_eq!(
            self.collector.reasons, rebuilt.collector.reasons,
            "collector withdrawal sources"
        );
        debug_assert_eq!(
            self.collector.pending, rebuilt.collector.pending,
            "collector scheduled sources"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_same_references(actual: &CollectorSet<String>, expected: &BTreeSet<String>) {
        assert_eq!(
            actual.iter().collect::<Vec<_>>(),
            expected.iter().collect::<Vec<_>>()
        );
        assert_eq!(actual.len(), expected.len());
        assert_eq!(actual.is_empty(), expected.is_empty());
        assert_eq!(actual, &expected.iter().cloned().collect());
    }

    #[test]
    fn compact_reference_transitions_match_an_ordered_set() {
        let mut actual = CollectorSet::default();
        let mut expected = BTreeSet::new();
        // Grow and shrink through every representation boundary, in different
        // orders, with duplicate additions and absent removals at each step.
        for _ in 0..3 {
            for number in (0..32).rev() {
                let name = format!("record@{number:02}");
                assert_eq!(actual.insert(name.clone()), expected.insert(name.clone()));
                assert!(!actual.insert(name));
                assert_same_references(&actual, &expected);
            }
            let original = actual.clone();
            for step in 0..32 {
                let name = format!("record@{:02}", (step * 17) % 32);
                assert_eq!(actual.remove(name.as_str()), expected.remove(name.as_str()));
                assert!(!actual.remove(name.as_str()));
                assert_same_references(&actual, &expected);
                assert_eq!(original.len(), 32, "mutating a clone preserves its source");
            }
        }

        // Mixed membership changes exercise promotion after demotion, rather
        // than only monotonically growing or emptying a set.
        let mut seed = 73_u32;
        for _ in 0..1024 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let name = format!("record@{:02}", (seed >> 16) % 32);
            if seed & 0x100 != 0 {
                assert_eq!(actual.insert(name.clone()), expected.insert(name));
            } else {
                assert_eq!(actual.remove(name.as_str()), expected.remove(name.as_str()));
            }
            assert_same_references(&actual, &expected);
        }
    }

    #[test]
    fn unchanged_references_preserve_shared_indexes_and_do_not_queue_work() {
        for count in [1, 2, SMALL_REFERENCE_LIMIT, SMALL_REFERENCE_LIMIT + 1, 32] {
            let owner = CollectorOwner::Record("holder@2".into());
            let references: CollectorSet<_> = (0..count).map(|n| format!("record@{n}")).collect();
            let mut original = WithdrawalCollector::default();
            original.replace(owner.clone(), references.clone());
            original.queued.clear();
            let mut transaction = original.clone();

            transaction.replace(owner, references);
            transaction.replace(
                CollectorOwner::Graph("absent".into()),
                CollectorSet::default(),
            );

            assert!(transaction.queued.is_empty());
            assert!(transaction.dirty.is_empty());
            assert!(Arc::ptr_eq(&transaction.owners, &original.owners));
            assert!(Arc::ptr_eq(&transaction.incoming, &original.incoming));
            assert_eq!(transaction, original);
        }
    }

    #[test]
    fn withdrawal_counts_remove_one_subject_at_a_time_and_rollback_with_clone() {
        let mut original = WithdrawalCollector::default();
        original.withdrawal("subject@2", "reason@2", true);
        original.withdrawal("subject@3", "reason@2", true);
        original.withdrawal("reason@2", "reason@2", true);
        assert_eq!(original.reasons.get("reason@2"), Some(&3));

        let mut transaction = original.clone();
        transaction.withdrawal("subject@2", "reason@2", false);
        assert_eq!(transaction.reasons.get("reason@2"), Some(&2));
        transaction.withdrawal("subject@3", "reason@2", false);
        assert_eq!(transaction.reasons.get("reason@2"), Some(&1));
        transaction.withdrawal("reason@2", "reason@2", false);
        assert!(!transaction.reasons.contains_key("reason@2"));
        assert_eq!(original.reasons.get("reason@2"), Some(&3));
        assert!(!Arc::ptr_eq(&transaction.reasons, &original.reasons));
    }

    #[test]
    fn forget_absent_record_preserves_shared_indexes() {
        let owner = CollectorOwner::State("kept".into());
        let mut original = WithdrawalCollector::default();
        original.replace(owner, CollectorSet::from(["held@2".into()]));
        let mut transaction = original.clone();

        transaction.forget(&BTreeMap::from([(
            "unrelated@2".into(),
            ("unrelated".into(), 2),
        )]));

        assert_eq!(transaction.incoming, original.incoming);
        assert_eq!(transaction.owners, original.owners);
        assert!(Arc::ptr_eq(&transaction.incoming, &original.incoming));
        assert!(Arc::ptr_eq(&transaction.owners, &original.owners));
    }

    #[test]
    fn forget_present_record_preserves_original_shared_indexes() {
        let owner = CollectorOwner::State("kept".into());
        let references = CollectorSet::from(["held@2".into(), "other@2".into()]);
        let mut original = WithdrawalCollector::default();
        original.replace(owner.clone(), references.clone());
        let mut transaction = original.clone();

        transaction.forget(&BTreeMap::from([("held@2".into(), ("held".into(), 2))]));

        assert_eq!(original.owners.get(&owner), Some(&references));
        assert_eq!(
            original.incoming.get("held@2"),
            Some(&CollectorSet::from([owner.clone()]))
        );
        assert_eq!(
            transaction.owners.get(&owner),
            Some(&CollectorSet::from(["other@2".into()]))
        );
        assert!(!transaction.incoming.contains_key("held@2"));
        assert_eq!(
            transaction.incoming.get("other@2"),
            Some(&CollectorSet::from([owner]))
        );
        assert!(!Arc::ptr_eq(&transaction.incoming, &original.incoming));
        assert!(!Arc::ptr_eq(&transaction.owners, &original.owners));
    }
}
