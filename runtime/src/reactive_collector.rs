//! Draft withdrawal collection. Exact references veto only additional
//! departures; they never become ordinary pins. Owners make record-local
//! metadata conditional, so an unreachable cycle cannot root itself.
//!
//! An event decides collection from what it changed. Only an eligible record
//! whose every pin is a withdrawal pin, and whose every holder is an eligible
//! record or departs in `D0`, can be collected: it is *unsupported*. Every
//! other eligible record is a root. Such a record is a key of `reasons`, so an
//! event with no standing withdrawal collects nothing and traces nothing. A
//! record that becomes unreachable lost support in this event, at the last
//! break of every root path it had, and the record just past that break is
//! unsupported: a reason that lost a pin, lost a holder, or retired, or one
//! that a record departing in `D0` held. Those reasons are the seeds. A seed
//! that a supported record holds is reached, with all it reaches, so only the
//! other seeds are traced, and the search stays among unsupported records. A
//! holder that retires breaks no path unless it is unsupported itself, and
//! then it is a reason that retired; a record becomes a reason only while it
//! is current, so becoming one is never a loss.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CollectorOwner {
    State(Name),
    Record(Name),
    Stream(Name),
    Series(Name),
    Graph(Name),
    Pending(Name),
}

impl CollectorOwner {
    fn record(&self) -> Option<&str> {
        match self {
            Self::Record(name) | Self::Graph(name) => Some(name.as_str()),
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

    fn contains<Q>(&self, value: &Q) -> bool
    where
        T: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match self {
            Self::Empty => false,
            Self::One(existing) => existing.borrow() == value,
            Self::Small(values) => values
                .binary_search_by(|existing| existing.borrow().cmp(value))
                .is_ok(),
            Self::Many(values) => values.contains(value),
        }
    }

    /// The set of `values`, given in any order and with repeats. Sorted once,
    /// so a large set is built in one pass, not by one insertion per value;
    /// the form is the one inserting them one by one gives.
    fn from_unsorted(mut values: Vec<T>) -> Self {
        values.sort();
        values.dedup();
        match values.len() {
            0 => Self::Empty,
            1 => Self::One(values.pop().expect("one value")),
            len if len <= SMALL_REFERENCE_LIMIT => Self::Small(values),
            _ => Self::Many(values.into_iter().collect()),
        }
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
#[cfg(feature = "collector-metrics")]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize)]
pub struct WithdrawalCollectionMetrics {
    /// Owners whose references the event changed and that were read again.
    pub dirty_owners: usize,
    /// Names read to keep the indexes: those owners' references, scheduled
    /// guards, relations, and the holders of departing records.
    pub references_scanned: usize,
    /// Unsupported records the search visited: the seeds it traced and the
    /// unsupported records connected to them. A seed a supported record
    /// holds is not visited.
    pub vertices_visited: usize,
    /// Holder and reference edges the search followed, including those of
    /// the pass that finds what a root reaches.
    pub edges_visited: usize,
    /// Seeds checked: the reasons that lost a pin or a holder or retired,
    /// and those a record departing in `D0` held.
    pub candidates_examined: usize,
    /// Records collection added to the batch.
    pub collected: usize,
    /// Live work-set entries at most, not heap bytes.
    pub peak_work_items: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct WithdrawalCollector {
    owners: Arc<BTreeMap<CollectorOwner, CollectorSet<Name>>>,
    incoming: Arc<BTreeMap<Name, CollectorSet<CollectorOwner>>>,
    // Only multiplicity is needed to distinguish withdrawal pins from roots.
    // Exact subject -> reason identities remain in withdrawal_reasons and
    // Record owner references, and in the authoritative withdrawal records.
    reasons: Arc<BTreeMap<Name, usize>>,
    pending: Arc<BTreeMap<String, BTreeMap<Name, usize>>>,
    dirty: BTreeSet<CollectorOwner>,
    /// Withdrawal reasons that lost a pin or a holder, or retired, since the
    /// last collection: each may be unsupported now. Only a reason can be
    /// unsupported, so a loss of any other record is not kept.
    queued: BTreeSet<Name>,
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
    /// A pin `record` had was released. Only a withdrawal reason can be left
    /// with withdrawal pins alone, so only a reason is queued.
    pub(super) fn released(&mut self, record: &str) {
        self.queue_reason(record);
    }

    /// `record` retired, so it is eligible now. Only a reason can be
    /// unsupported, and a retired record never becomes one, so only a reason
    /// is queued. What it holds lost nothing unless it is unsupported itself:
    /// a supported holder still roots what it references, and one departing
    /// in `D0` is a seed of its own.
    pub(super) fn retired(&mut self, record: &str) {
        self.queue_reason(record);
    }

    fn queue_reason(&mut self, record: &str) {
        if let Some((name, _)) = self.reasons.get_key_value(record) {
            self.queued.insert(name.clone());
        }
    }

    /// The name already indexed for `name`, so copies share its text, or a
    /// new one.
    fn name(&self, name: &str) -> Name {
        self.incoming
            .get_key_value(name)
            .map(|(name, _)| name.clone())
            .or_else(|| {
                self.reasons
                    .get_key_value(name)
                    .map(|(name, _)| name.clone())
            })
            .unwrap_or_else(|| name.into())
    }

    fn replace(&mut self, owner: CollectorOwner, references: CollectorSet<Name>) {
        // Compare borrowed sets before any write, including when this owner
        // has no references. Keep the old set borrowed for the diff too.
        let previous = self.owners.get(&owner);
        if previous.map_or(references.is_empty(), |previous| previous == &references) {
            return;
        }
        let mut gained = Vec::new();
        let mut lost = Vec::new();
        {
            // Both iterators are ordered. Merge once instead of looking up
            // each old/new member again when an owner has many references.
            let mut before = previous.into_iter().flat_map(CollectorSet::iter).peekable();
            let mut after = references.iter().peekable();
            loop {
                let order = match (before.peek(), after.peek()) {
                    (Some(old), Some(new)) => old.cmp(new),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => break,
                };
                match order {
                    std::cmp::Ordering::Less => lost.push(before.next().unwrap().clone()),
                    std::cmp::Ordering::Greater => gained.push(after.next().unwrap().clone()),
                    std::cmp::Ordering::Equal => {
                        before.next();
                        after.next();
                    }
                }
            }
        }
        let incoming = Arc::make_mut(&mut self.incoming);
        for name in gained {
            incoming.entry(name).or_default().insert(owner.clone());
        }
        for name in lost {
            let holders = incoming.get_mut(&name).expect("indexed reference");
            holders.remove(&owner);
            if holders.is_empty() {
                incoming.remove(&name);
            }
            // A reason that loses a holder may be left unsupported.
            if self.reasons.contains_key(&name) {
                self.queued.insert(name);
            }
        }
        let owners = Arc::make_mut(&mut self.owners);
        if references.is_empty() {
            owners.remove(&owner);
        } else {
            owners.insert(owner, references);
        }
    }

    /// Add `names` to what `owner` references. Only a name it does not hold
    /// yet is written, so the work is what changes, not what the owner holds.
    fn extend<'a>(&mut self, owner: &CollectorOwner, names: impl IntoIterator<Item = &'a str>) {
        for name in names {
            if self
                .owners
                .get(owner)
                .is_some_and(|references| references.contains(name))
            {
                continue;
            }
            let name = self.name(name);
            Arc::make_mut(&mut self.owners)
                .entry(owner.clone())
                .or_default()
                .insert(name.clone());
            Arc::make_mut(&mut self.incoming)
                .entry(name)
                .or_default()
                .insert(owner.clone());
        }
    }

    pub(super) fn withdrawal(&mut self, subject: &str, reason: &str, add: bool) {
        let name = self
            .reasons
            .get_key_value(reason)
            .map_or_else(|| reason.into(), |(name, _)| name.clone());
        let reasons = Arc::make_mut(&mut self.reasons);
        let previous = reasons.get(reason).copied();
        // Callers add only the first withdrawal of a subject, and remove only
        // after removing its authoritative subject -> reason entry. Duplicate
        // language effects therefore never increment this derived count.
        if add {
            reasons.insert(name.clone(), previous.unwrap_or(0) + 1);
        } else {
            let count = previous.expect("indexed withdrawal reason");
            if count == 1 {
                reasons.remove(reason);
            } else {
                reasons.insert(name.clone(), count - 1);
            }
        }
        self.dirty.insert(CollectorOwner::Record(subject.into()));
        // An added reason is not queued: a withdrawal names its reason's
        // current occurrence, which is not eligible, so nothing it gains can
        // leave it unsupported. If it retires later in this event, `retired`
        // queues it then.
    }

    fn peak(&mut self, count: usize) {
        #[cfg(feature = "collector-metrics")]
        {
            self.metrics.peak_work_items = self.metrics.peak_work_items.max(count);
        }
        #[cfg(not(feature = "collector-metrics"))]
        let _ = count;
    }

    /// The references of `record`'s own owner that are withdrawal reasons:
    /// the only records its retirement or departure can leave unsupported.
    /// Read from whichever side is smaller.
    fn reason_references(&self, record: &Name) -> Vec<Name> {
        let owner = CollectorOwner::Record(record.clone());
        let Some(references) = self.owners.get(&owner) else {
            return Vec::new();
        };
        if references.len() <= self.reasons.len() {
            references
                .iter()
                .filter(|name| self.reasons.contains_key(*name))
                .cloned()
                .collect()
        } else {
            self.reasons
                .keys()
                .filter(|reason| {
                    self.incoming
                        .get(*reason)
                        .is_some_and(|holders| holders.contains(&owner))
                })
                .cloned()
                .collect()
        }
    }

    fn forget(&mut self, departing: &BTreeMap<String, (String, u64)>) {
        // Compaction removes exactly these names from all surviving holders.
        // Visit their inverse references instead of scanning the session.
        for record in departing.keys() {
            // Unrelated departures must not copy the shared reverse index
            // merely to remove a key that is absent from it.
            if !self.incoming.contains_key(record.as_str()) {
                continue;
            }
            let holders = Arc::make_mut(&mut self.incoming)
                .remove(record.as_str())
                .expect("checked above");
            for owner in holders.iter() {
                metric!(self, references_scanned, 1);
                if let CollectorOwner::Pending(target) = owner {
                    if self
                        .pending
                        .get(target.as_str())
                        .is_some_and(|references| references.contains_key(record.as_str()))
                    {
                        let pending = Arc::make_mut(&mut self.pending);
                        let references = pending.get_mut(target.as_str()).expect("checked above");
                        references.remove(record.as_str());
                        if references.is_empty() {
                            pending.remove(target.as_str());
                        }
                    }
                }
                let Some(references) = self.owners.get(owner) else {
                    continue;
                };
                if !references.contains(record.as_str()) {
                    continue;
                }
                let owners = Arc::make_mut(&mut self.owners);
                if references_len(owners, owner) == 1 {
                    owners.remove(owner);
                } else {
                    owners
                        .get_mut(owner)
                        .expect("checked above")
                        .remove(record.as_str());
                }
            }
        }
        for record in departing.keys() {
            self.replace(
                CollectorOwner::Record(record.as_str().into()),
                CollectorSet::default(),
            );
            self.replace(
                CollectorOwner::Graph(record.as_str().into()),
                CollectorSet::default(),
            );
        }
        self.dirty.clear();
        // Every affected component was considered against the full prospective
        // batch already; no established work is deferred to a later event.
        self.queued.clear();
    }
}

fn references_len(
    owners: &BTreeMap<CollectorOwner, CollectorSet<Name>>,
    owner: &CollectorOwner,
) -> usize {
    owners.get(owner).map_or(0, CollectorSet::len)
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

    /// Queues an owner whose references may have changed for a recount at
    /// the next collection. Only a program with a window keeps a collector,
    /// so elsewhere the owner is not even built.
    pub(super) fn collector_dirty(&mut self, owner: impl FnOnce(&Self) -> CollectorOwner) {
        if self.windowed() {
            let owner = owner(self);
            self.collector.dirty.insert(owner);
        }
    }

    pub(super) fn collector_effect(&mut self, effect: &Effect) {
        if !self.windowed() {
            return;
        }
        let owner = match effect {
            Effect::Set { name, .. } | Effect::Unset { name } => {
                CollectorOwner::State(name.as_str().into())
            }
            Effect::Sample { stream, .. } => {
                let dirty = &mut self.collector.dirty;
                dirty.insert(CollectorOwner::Stream(stream.as_str().into()));
                if let Some(current) = self.reading_streams[stream].current.as_deref() {
                    dirty.insert(CollectorOwner::Record(current.into()));
                }
                return;
            }
            Effect::Qualify {
                evidence,
                latest: true,
                ..
            } => match self.latest_target(evidence) {
                Some(name) => CollectorOwner::Record(name.into()),
                None => return,
            },
            Effect::Reveal { evidence, .. }
            | Effect::Observe { evidence }
            | Effect::Renew { evidence }
            | Effect::Qualify { evidence, .. } => {
                CollectorOwner::Record(self.occurrence(evidence).into())
            }
            Effect::Examine { caveat, .. } => CollectorOwner::Record(caveat.as_str().into()),
            // An outcome may be of any revision (spec/caveat-outcomes-0.1.md).
            Effect::Resolve(resolution) => {
                CollectorOwner::Series(resolution.action.as_str().into())
            }
            Effect::Commit { action, .. } | Effect::Reopen { action, .. } => {
                self.collector
                    .dirty
                    .insert(CollectorOwner::Series(action.as_str().into()));
                let current = self
                    .decision_series
                    .get(action)
                    .and_then(|series| series.current.as_deref())
                    .unwrap_or(action);
                CollectorOwner::Record(current.into())
            }
            Effect::Withdraw {
                target: EvidenceSelector::Every(name),
                ..
            } => {
                for occurrence in self.every_occurrence(name).unwrap_or_default() {
                    self.collector
                        .dirty
                        .insert(CollectorOwner::Record(occurrence.into()));
                }
                return;
            }
            Effect::Withdraw { target, .. } => {
                let Some(name) = self.withdrawal_target(target) else {
                    return;
                };
                CollectorOwner::Record(name.into())
            }
            _ => return,
        };
        self.collector.dirty.insert(owner);
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
        let collector = &mut self.collector;
        let pending = Arc::make_mut(&mut collector.pending);
        let references = pending.entry(scheduled.evidence.clone()).or_default();
        for name in scheduled.guard.evidence.iter() {
            if add {
                *references.entry(name.clone()).or_default() += 1;
            } else {
                let count = references
                    .get_mut(name.as_str())
                    .expect("scheduled dependency");
                *count -= 1;
                if *count == 0 {
                    references.remove(name.as_str());
                }
            }
        }
        let refs = CollectorSet::from_unsorted(references.keys().cloned().collect());
        if references.is_empty() {
            pending.remove(&scheduled.evidence);
        }
        collector.replace(
            CollectorOwner::Pending(scheduled.evidence.as_str().into()),
            refs,
        );
    }

    pub(super) fn collector_reopens<'a>(
        &mut self,
        action: &str,
        reasons: impl ExactSizeIterator<Item = &'a str>,
    ) {
        // ReliesOn sources and Reopens targets are commitments, which do not
        // depart. Graph owners therefore only supply external roots; candidate
        // dependency traversal needs Record owners alone.
        debug_assert!(!self.retired.contains_key(action));
        if !self.windowed() {
            return;
        }
        metric!(self.collector, references_scanned, reasons.len());
        self.collector
            .extend(&CollectorOwner::Graph(action.into()), reasons);
    }

    fn collector_references(&self, owner: &CollectorOwner) -> (CollectorSet<Name>, usize) {
        let mut references = Vec::new();
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
                if let Some(stream) = self.reading_streams.get(name.as_str()) {
                    include(&stream.selection_qualifications);
                }
            }
            CollectorOwner::Series(name) => {
                if let Some(series) = self.decision_series.get(name.as_str()) {
                    include(&series.selection_qualifications);
                    // An outcome's lineage holds what its value read, as a
                    // basis does, so it vetoes collection (rule 4 of
                    // spec/caveat-withdrawal-collection-0.1.md).
                    for outcome in series
                        .revisions
                        .iter()
                        .filter_map(|revision| revision.outcome.as_ref())
                    {
                        include(&outcome.lineage);
                    }
                }
            }
            CollectorOwner::Record(name) => {
                for map in [
                    &self.observation_qualifications,
                    &self.examination_qualifications,
                    &self.reopening_qualifications,
                ] {
                    if let Some(provenance) = map.get(name.as_str()) {
                        include(provenance);
                    }
                }
                for targets in self.predicate_qualifications.values() {
                    if let Some(provenance) = targets.get(name.as_str()) {
                        include(provenance);
                    }
                }
                if let Some(basis) = self.commitment_bases.get(name.as_str()) {
                    include(&basis.provenance);
                }
                if let Some(grounds) = self.commitment_grounds.get(name.as_str()) {
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
                if let Some(reason) = self.withdrawal_reasons.get(name.as_str()) {
                    scanned += 1;
                    references.push(self.collector.name(reason));
                }
            }
            CollectorOwner::Graph(_) | CollectorOwner::Pending(_) => {
                let held = self
                    .collector
                    .owners
                    .get(owner)
                    .cloned()
                    .unwrap_or_default();
                scanned += held.len();
                return (held, scanned);
            }
        }
        (CollectorSet::from_unsorted(references), scanned)
    }

    pub(super) fn rebuild_withdrawal_collector(&mut self) {
        self.collector = WithdrawalCollector::default();
        if !self.windowed() {
            return;
        }
        let mut owners = BTreeSet::new();
        owners.extend(
            self.states
                .names()
                .map(|name| CollectorOwner::State(name.as_str().into())),
        );
        owners.extend(
            self.reading_streams
                .keys()
                .map(|name| CollectorOwner::Stream(name.as_str().into())),
        );
        owners.extend(
            self.decision_series
                .keys()
                .map(|name| CollectorOwner::Series(name.as_str().into())),
        );
        owners.extend(
            self.symbols
                .keys()
                .map(|name| CollectorOwner::Record(name.as_str().into())),
        );
        // A skipped one-off commitment has predicate metadata before its
        // symbol exists. Rebuild from actual holder keys as well as symbols.
        for map in [
            &self.observation_qualifications,
            &self.examination_qualifications,
            &self.reopening_qualifications,
        ] {
            owners.extend(
                map.keys()
                    .map(|name| CollectorOwner::Record(name.as_str().into())),
            );
        }
        for targets in self.predicate_qualifications.values() {
            owners.extend(
                targets
                    .keys()
                    .map(|name| CollectorOwner::Record(name.as_str().into())),
            );
        }
        owners.extend(
            self.commitment_bases
                .keys()
                .map(|name| CollectorOwner::Record(name.as_str().into())),
        );
        owners.extend(
            self.commitment_grounds
                .keys()
                .map(|name| CollectorOwner::Record(name.as_str().into())),
        );
        for reason in self.withdrawal_reasons.values() {
            let name = self.collector.name(reason);
            *Arc::make_mut(&mut self.collector.reasons)
                .entry(name)
                .or_default() += 1;
        }
        for owner in owners {
            let references = self.collector_references(&owner).0;
            self.collector.replace(owner, references);
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
                self.collector_reopens(&names[&edge.to], [names[&edge.from].as_str()].into_iter());
            } else if edge.relation == Relation::ReliesOn {
                self.collector_reopens(&names[&edge.from], [names[&edge.to].as_str()].into_iter());
            }
        }
        // The first accepted event considers every retired record that can
        // be unsupported: a withdrawal reason, as every pinned record is
        // otherwise a root and an unpinned one departs ordinarily.
        let seeds = self
            .collector
            .reasons
            .keys()
            .filter(|name| self.retired.contains_key(name.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        self.collector.queued.extend(seeds);
        self.collector.dirty.clear();
        self.collector_begin();
    }

    fn collector_eligible(&self, name: &str, ordinary: &BTreeMap<String, (String, u64)>) -> bool {
        self.retired.contains_key(name)
            && !ordinary.contains_key(name)
            && self.departable_name(name)
    }

    /// Whether `record` is eligible and nothing roots it directly: every pin
    /// it has is a withdrawal pin, and every holder departs ordinarily or is
    /// an eligible record. Only such a record can be collected. Decided
    /// against this event's `D0`, once per record per collection.
    fn collector_unsupported(
        &self,
        record: &Name,
        ordinary: &BTreeMap<String, (String, u64)>,
        known: &mut BTreeMap<Name, bool>,
    ) -> bool {
        if let Some(unsupported) = known.get(record) {
            return *unsupported;
        }
        let unsupported = self.collector_eligible(record, ordinary)
            && self.pin_counts.get(record.as_str()).copied().unwrap_or(0)
                <= self.collector.reasons.get(record).copied().unwrap_or(0)
            && self.collector.incoming.get(record).is_none_or(|holders| {
                holders.iter().all(|owner| {
                    owner.record().is_some_and(|holder| {
                        ordinary.contains_key(holder) || self.collector_eligible(holder, ordinary)
                    })
                })
            });
        known.insert(record.clone(), unsupported);
        unsupported
    }

    /// The unsupported records this event's losses reach: each queued reason,
    /// and each reason a record departing in `D0` held, that is unsupported
    /// and that no supported record holds. One a supported record holds is
    /// reached, with all it reaches, so it cannot lead to a collectible
    /// record: what became unreachable lies past a seed that is unreached
    /// itself.
    fn collector_candidates(
        &self,
        queued: BTreeSet<Name>,
        ordinary: &BTreeMap<String, (String, u64)>,
        known: &mut BTreeMap<Name, bool>,
    ) -> (BTreeSet<Name>, usize) {
        let mut examined = 0;
        let mut candidates = BTreeSet::new();
        let held = ordinary.keys().flat_map(|record| {
            self.collector
                .reason_references(&self.collector.name(record))
        });
        for seed in queued.into_iter().chain(held) {
            examined += 1;
            if self.collector_unsupported(&seed, ordinary, known)
                && !self.collector_entered(&seed, ordinary, known)
            {
                candidates.insert(seed);
            }
        }
        (candidates, examined)
    }

    /// Whether a supported record holds unsupported `record`. Every holder of
    /// an unsupported record departs in `D0` or is an eligible record, and a
    /// supported eligible record is a root.
    fn collector_entered(
        &self,
        record: &Name,
        ordinary: &BTreeMap<String, (String, u64)>,
        known: &mut BTreeMap<Name, bool>,
    ) -> bool {
        self.collector.incoming.get(record).is_some_and(|holders| {
            holders.iter().any(|owner| match owner {
                CollectorOwner::Record(holder) => {
                    !ordinary.contains_key(holder.as_str())
                        && !self.collector_unsupported(holder, ordinary, known)
                }
                _ => false,
            })
        })
    }

    /// Every unsupported record connected to `candidates` through unsupported
    /// records, and those of them that a root holds. A root path into this
    /// region enters it at an unsupported record a root holds, so nothing
    /// outside it decides what inside it is reachable.
    fn collector_region(
        &self,
        candidates: BTreeSet<Name>,
        ordinary: &BTreeMap<String, (String, u64)>,
        known: &mut BTreeMap<Name, bool>,
    ) -> (BTreeSet<Name>, BTreeSet<Name>, usize, usize) {
        let mut stack = candidates.iter().cloned().collect::<Vec<_>>();
        let mut region = candidates;
        let mut entered = BTreeSet::new();
        let (mut vertices, mut edges) = (0, 0);
        while let Some(record) = stack.pop() {
            vertices += 1;
            if let Some(holders) = self.collector.incoming.get(&record) {
                for owner in holders.iter() {
                    edges += 1;
                    // Every holder of an unsupported record departs ordinarily
                    // or is an eligible record; only a record's own references
                    // are dependencies.
                    let CollectorOwner::Record(holder) = owner else {
                        continue;
                    };
                    if ordinary.contains_key(holder.as_str()) {
                        continue;
                    }
                    if self.collector_unsupported(holder, ordinary, known) {
                        if region.insert(holder.clone()) {
                            stack.push(holder.clone());
                        }
                    } else {
                        entered.insert(record.clone());
                    }
                }
            }
            if let Some(references) = self
                .collector
                .owners
                .get(&CollectorOwner::Record(record.clone()))
            {
                for name in references.iter() {
                    edges += 1;
                    if !region.contains(name) && self.collector_unsupported(name, ordinary, known) {
                        region.insert(name.clone());
                        stack.push(name.clone());
                    }
                }
            }
        }
        (region, entered, vertices, edges)
    }

    /// The records of `region` that no record of `entered` reaches through
    /// records' own references.
    fn collector_unreached(
        &self,
        region: &BTreeSet<Name>,
        entered: BTreeSet<Name>,
    ) -> (BTreeSet<Name>, usize) {
        let mut reached = BTreeSet::new();
        let mut stack = entered.into_iter().collect::<Vec<_>>();
        let mut edges = 0;
        while let Some(record) = stack.pop() {
            if !reached.insert(record.clone()) {
                continue;
            }
            if let Some(references) = self.collector.owners.get(&CollectorOwner::Record(record)) {
                for name in references.iter() {
                    edges += 1;
                    if region.contains(name) && !reached.contains(name) {
                        stack.push(name.clone());
                    }
                }
            }
        }
        (region.difference(&reached).cloned().collect(), edges)
    }

    /// Collect what this event left unreachable. Only unsupported records are
    /// traced, starting from what lost support; a root release can expose an
    /// entire old region. No work cap changes the event's acceptance or
    /// defers established departures. Only a program with a window has a
    /// collector, so the caller asks first and an event elsewhere does not
    /// pay for the call.
    pub(super) fn collect_withdrawals(&mut self, departing: &mut BTreeMap<String, (String, u64)>) {
        debug_assert!(self.windowed(), "only a windowed program collects");
        for owner in std::mem::take(&mut self.collector.dirty) {
            let (refs, _scanned) = self.collector_references(&owner);
            metric!(self.collector, dirty_owners, 1);
            metric!(self.collector, references_scanned, _scanned);
            self.collector.replace(owner, refs);
        }
        let queued = std::mem::take(&mut self.collector.queued);
        // Every eligible record has a pin, or it would depart in D0, so an
        // unsupported record has withdrawal pins and is a reason.
        if self.collector.reasons.is_empty() {
            #[cfg(debug_assertions)]
            self.check_collection(departing, &BTreeSet::new());
            return;
        }
        let mut known = BTreeMap::new();
        let (candidates, _examined) = self.collector_candidates(queued, departing, &mut known);
        metric!(self.collector, candidates_examined, _examined);
        let (region, entered, _vertices, _edges) =
            self.collector_region(candidates, departing, &mut known);
        metric!(self.collector, vertices_visited, _vertices);
        metric!(self.collector, edges_visited, _edges);
        let work = region.len() + entered.len() + known.len();
        let (additional, _reaching) = self.collector_unreached(&region, entered);
        metric!(self.collector, edges_visited, _reaching);
        self.collector.peak(work + additional.len());
        #[cfg(debug_assertions)]
        self.check_collection(departing, &additional);
        metric!(self.collector, collected, additional.len());
        for record in additional {
            let record = record.to_string();
            if let Some(reason) = Arc::make_mut(&mut self.withdrawal_reasons).remove(&record) {
                self.collector.withdrawal(&record, &reason, false);
                self.release_pin(&reason);
            }
            let place = self.departable(&record).expect("eligible candidate");
            departing.insert(record, place);
        }
    }

    pub(super) fn collector_departed(&mut self, departing: &BTreeMap<String, (String, u64)>) {
        self.collector.forget(departing);
        Arc::make_mut(&mut self.departure_candidates)
            .retain(|record| !departing.contains_key(record));
    }

    /// Debug builds: the records collection takes are exactly the eligible
    /// records that no root reaches, found from every eligible record rather
    /// than from what the event changed.
    #[cfg(debug_assertions)]
    fn check_collection(
        &self,
        ordinary: &BTreeMap<String, (String, u64)>,
        collected: &BTreeSet<Name>,
    ) {
        let eligible = self
            .retired
            .keys()
            .map(String::as_str)
            .filter(|name| self.collector_eligible(name, ordinary))
            .collect::<BTreeSet<_>>();
        let mut stack = eligible
            .iter()
            .copied()
            .filter(|name| {
                self.pin_counts.get(*name).copied().unwrap_or(0)
                    > self.collector.reasons.get(*name).copied().unwrap_or(0)
                    || self.collector.incoming.get(*name).is_some_and(|holders| {
                        holders.iter().any(|owner| {
                            owner.record().is_none_or(|holder| {
                                !ordinary.contains_key(holder) && !eligible.contains(holder)
                            })
                        })
                    })
            })
            .collect::<Vec<_>>();
        let mut reached = BTreeSet::new();
        while let Some(name) = stack.pop() {
            if !reached.insert(name) {
                continue;
            }
            if let Some(references) = self
                .collector
                .owners
                .get(&CollectorOwner::Record(name.into()))
            {
                stack.extend(
                    references
                        .iter()
                        .map(Name::as_str)
                        .filter(|reference| eligible.contains(reference)),
                );
            }
        }
        let unreached = eligible.difference(&reached).copied().collect::<Vec<_>>();
        let collected = collected.iter().map(Name::as_str).collect::<Vec<_>>();
        debug_assert_eq!(
            collected, unreached,
            "collection takes exactly the eligible records no root reaches"
        );
    }

    #[cfg(debug_assertions)]
    pub(super) fn check_withdrawal_collector(&self) {
        if !self.windowed() {
            return;
        }
        debug_assert!(
            self.collector.dirty.is_empty() && self.collector.queued.is_empty(),
            "collection leaves no work for a later event"
        );
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

    fn name(text: &str) -> Name {
        text.into()
    }

    fn assert_same_references(actual: &CollectorSet<String>, expected: &BTreeSet<String>) {
        assert_eq!(
            actual.iter().collect::<Vec<_>>(),
            expected.iter().collect::<Vec<_>>()
        );
        assert_eq!(actual.len(), expected.len());
        assert_eq!(actual.is_empty(), expected.is_empty());
        assert_eq!(actual, &expected.iter().cloned().collect());
        assert_eq!(
            actual,
            &CollectorSet::from_unsorted(expected.iter().rev().cloned().collect()),
            "a set built at once has the form inserting one by one gives"
        );
        for value in expected {
            assert!(actual.contains(value.as_str()));
        }
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
                assert!(!actual.contains(name.as_str()));
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
            let owner = CollectorOwner::Record(name("holder@2"));
            let references: CollectorSet<_> =
                (0..count).map(|n| name(&format!("record@{n}"))).collect();
            let mut original = WithdrawalCollector::default();
            original.replace(owner.clone(), references.clone());
            original.queued.clear();
            let mut transaction = original.clone();

            transaction.replace(owner, references);
            transaction.replace(
                CollectorOwner::Graph(name("absent")),
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
        let owner = CollectorOwner::State(name("kept"));
        let mut original = WithdrawalCollector::default();
        original.replace(owner, CollectorSet::from([name("held@2")]));
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
        let owner = CollectorOwner::State(name("kept"));
        let references = CollectorSet::from([name("held@2"), name("other@2")]);
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
            Some(&CollectorSet::from([name("other@2")]))
        );
        assert!(!transaction.incoming.contains_key("held@2"));
        assert_eq!(
            transaction.incoming.get("other@2"),
            Some(&CollectorSet::from([owner]))
        );
        assert!(!Arc::ptr_eq(&transaction.incoming, &original.incoming));
        assert!(!Arc::ptr_eq(&transaction.owners, &original.owners));
    }

    /// A collector holding a little of everything an event can write.
    fn populated() -> WithdrawalCollector {
        let mut collector = WithdrawalCollector::default();
        let many = (0..12)
            .map(|n| name(&format!("held@{n}")))
            .collect::<CollectorSet<_>>();
        collector.replace(CollectorOwner::State(name("state")), many);
        collector.replace(
            CollectorOwner::Record(name("holder@3")),
            CollectorSet::from([name("held@1"), name("held@2")]),
        );
        collector.extend(&CollectorOwner::Graph(name("decision@1")), ["held@4"]);
        collector.withdrawal("held@1", "held@2", true);
        collector.withdrawal("held@3", "held@2", true);
        Arc::make_mut(&mut collector.pending).insert(
            "target@1".into(),
            BTreeMap::from([(name("held@5"), 2), (name("held@6"), 1)]),
        );
        collector.replace(
            CollectorOwner::Pending(name("target@1")),
            CollectorSet::from([name("held@5"), name("held@6")]),
        );
        collector.dirty.clear();
        collector.queued.clear();
        collector
    }

    #[test]
    fn only_withdrawal_reasons_are_queued_by_losses() {
        let mut collector = populated();
        collector.released("held@1");
        collector.replace(
            CollectorOwner::Record(name("holder@3")),
            CollectorSet::from([name("held@2")]),
        );
        assert!(collector.queued.is_empty(), "held@1 names no reason");
        collector.replace(
            CollectorOwner::Record(name("holder@3")),
            CollectorSet::default(),
        );
        assert_eq!(
            collector.queued,
            BTreeSet::from([name("held@2")]),
            "a lost holder"
        );
        collector.queued.clear();
        collector.released("held@2");
        assert_eq!(
            collector.queued,
            BTreeSet::from([name("held@2")]),
            "a lost pin"
        );
        collector.queued.clear();
        // Becoming a reason is no loss; retiring is one only for a reason.
        collector.withdrawal("held@7", "held@7", true);
        collector.retired("held@8");
        assert!(collector.queued.is_empty());
        collector.retired("held@7");
        assert_eq!(collector.queued, BTreeSet::from([name("held@7")]));
    }
}
