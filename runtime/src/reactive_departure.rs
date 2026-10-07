//! Departure: a retired record that nothing pins leaves the session for the
//! archive the host drains. See spec/caveat-departure-0.1.md and
//! spec/caveat-lineage-compaction-0.1.md.
use super::*;

/// Host-drained records and the provenance transfer nodes referring to them.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ArchiveItem {
    Record(Box<ArchiveEntry>),
    Provenance(ArchiveProvenance),
}

impl ArchiveItem {
    pub fn record(&self) -> Option<&ArchiveEntry> {
        match self {
            Self::Record(record) => Some(record),
            Self::Provenance(_) => None,
        }
    }
}

/// A departed record as it was when it left, for the host to drain.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArchiveEntry {
    pub record: String,
    pub history: String,
    pub number: u64,
    pub retired_at: u64,
    pub departed_at: u64,
    /// The relations that left with it, as a save writes them.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<[String; 3]>,
    /// The provenance of each qualification it carried: `observation`,
    /// `examination`, `reopening`, or a predicate's kind.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub qualifications: BTreeMap<String, Provenance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading: Option<ReadingOccurrence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_entry: Option<JournalEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdrawal: Option<Withdrawal>,
    /// Every provenance that named it when it departed.
    pub holders: Vec<Holder>,
}

/// A provenance that named a departed record: its kind, whose it is, and the
/// field it is in.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Holder {
    pub kind: &'static str,
    pub name: String,
    #[serde(rename = "in")]
    pub field: &'static str,
}

/// Compact one provenance for every departing record it names, noting it as
/// a holder of each.
fn compact(
    provenance: &mut Provenance,
    departing: &BTreeMap<String, (String, u64)>,
    sequence: u64,
    source_id: &str,
    holder: impl Fn() -> Holder,
    holders: &mut BTreeMap<String, Vec<Holder>>,
) {
    if provenance.evidence.is_empty() {
        return;
    }
    let named = provenance
        .evidence
        .iter()
        .filter(|name| departing.contains_key(*name))
        .cloned()
        .collect::<Vec<_>>();
    for record in named {
        let (history, number) = &departing[&record];
        provenance.depart(&record, history, *number, sequence, source_id);
        holders.entry(record).or_default().push(holder());
    }
}

fn names_any(provenance: &Provenance, departing: &BTreeMap<String, (String, u64)>) -> bool {
    provenance
        .evidence
        .iter()
        .any(|name| departing.contains_key(name))
}

impl ReactiveSession {
    /// Whether the program declares any window, the journal's included.
    pub(crate) fn windowed(&self) -> bool {
        !self.windows.is_empty() || self.journal_window.is_some()
    }

    /// Merge a selection or reopening qualification into `into`. Only a
    /// program with a window keeps which names were inherited, since only
    /// departure consults it; elsewhere the names merge as any others.
    pub(crate) fn inherit(&self, into: &mut Provenance, from: &Provenance) -> Result<(), String> {
        if self.windowed() {
            into.merge_inherited(from)
        } else {
            into.merge(from)
        }
    }

    /// The name of the `index`th journal entry the session holds, counting
    /// from 0: entries that departed keep their numbers.
    pub(crate) fn journal_name(&self, index: usize) -> String {
        format!("journal@{}", self.journal_departed + index + 1)
    }

    /// The number the next record of a stream or renewable evidence takes:
    /// one past its newest, whatever has departed.
    pub(crate) fn next_stream_ordinal(&self, stream: &str) -> u64 {
        self.reading_streams[stream]
            .occurrences
            .last()
            .map_or(1, |reading| reading.ordinal + 1)
    }

    pub(crate) fn next_renewal_ordinal(&self, evidence: &str) -> usize {
        self.renewals[evidence]
            .occurrences
            .last()
            .and_then(|name| save::occurrence_number(name))
            .map_or(2, |number| number as usize + 1)
    }

    /// Enumerate pin sources only while loading/restoring or auditing the index.
    /// Event mutations update the index directly, never by rescanning these sources.
    fn for_each_pin(&self, mut pin: impl FnMut(&String, &'static str)) {
        for cell in self.states.cells.iter() {
            for name in cell.grounds.own_evidence() {
                pin(name, "state grounds");
            }
        }
        // A series revision is in force while it is its series' current one;
        // a commitment outside a series is in force once made.
        let superseded = self
            .decision_series
            .values()
            .flat_map(|series| {
                series
                    .revisions
                    .iter()
                    .filter(move |revision| series.current.as_ref() != Some(&revision.id))
                    .map(|revision| revision.id.as_str())
            })
            .collect::<HashSet<_>>();
        for (commitment, grounds) in self.commitment_grounds.iter() {
            if !superseded.contains(commitment.as_str()) {
                for name in grounds.own_evidence() {
                    pin(name, "commitment in force");
                }
            }
        }
        for (index, entry) in self.journal.iter().enumerate() {
            if !self.retired.contains_key(&self.journal_name(index)) {
                for name in entry.because.iter().chain(&entry.permitted_by) {
                    pin(name, "journal entry");
                }
            }
        }
        for scheduled in self.scheduled.iter() {
            pin(&scheduled.evidence, "scheduled qualification");
        }
        for record in self.commitment_permissions.values() {
            pin(&record.grant, "permission");
        }
        for withdrawal in self.withdrawals.iter() {
            pin(&withdrawal.because, "withdrawal reason");
        }
    }

    /// The records held by each kind of provenance, for restore validation and audits.
    pub(crate) fn pins(&self) -> BTreeMap<String, BTreeSet<&'static str>> {
        let mut pins: BTreeMap<String, BTreeSet<&'static str>> = BTreeMap::new();
        self.for_each_pin(|name, by| {
            pins.entry(name.clone()).or_default().insert(by);
        });
        pins
    }

    pub(crate) fn rebuild_departure_index(&mut self) {
        let mut counts = BTreeMap::new();
        if self.windowed() {
            self.for_each_pin(|name, _| *counts.entry(name.clone()).or_insert(0) += 1);
        }
        self.pin_counts = Arc::new(counts);
        self.withdrawal_reasons = Arc::new(
            self.withdrawals
                .iter()
                .map(|withdrawal| (withdrawal.evidence.clone(), withdrawal.because.clone()))
                .collect(),
        );
        // rc.15 saves may contain unpinned retired records. Restore does not
        // depart them; the next accepted event consumes these candidates.
        self.departure_candidates = Arc::new(self.retired.keys().cloned().collect());
    }

    pub(crate) fn add_pin(&mut self, name: &str) {
        if self.windowed() {
            *Arc::make_mut(&mut self.pin_counts)
                .entry(name.to_string())
                .or_insert(0) += 1;
        }
    }

    pub(crate) fn release_pin(&mut self, name: &str) {
        if !self.windowed() {
            return;
        }
        let counts = Arc::make_mut(&mut self.pin_counts);
        let count = counts
            .get_mut(name)
            .expect("every released source had a pin");
        *count -= 1;
        if *count == 0 {
            counts.remove(name);
            if self.retired.contains_key(name) {
                Arc::make_mut(&mut self.departure_candidates).insert(name.to_string());
            }
        }
    }

    pub(crate) fn pin_journal_entry(&mut self, entry: &JournalEntry) {
        for name in entry.because.iter().chain(&entry.permitted_by) {
            self.add_pin(name);
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn check_departure_index(&self) {
        if self.windowed() {
            let mut expected = BTreeMap::new();
            self.for_each_pin(|name, _| *expected.entry(name.clone()).or_insert(0) += 1);
            debug_assert_eq!(*self.pin_counts, expected, "incremental pin counts");
            let reasons = self
                .withdrawals
                .iter()
                .map(|withdrawal| (withdrawal.evidence.clone(), withdrawal.because.clone()))
                .collect::<BTreeMap<_, _>>();
            debug_assert_eq!(*self.withdrawal_reasons, reasons, "withdrawal pin sources");
        }
    }

    /// The history and number of a retired record that may depart: not a
    /// renewable evidence's declared first occurrence.
    fn departable(&self, record: &str) -> Option<(String, u64)> {
        if let Some(number) = record
            .strip_prefix("journal@")
            .filter(|_| self.journal_window.is_some())
        {
            return Some(("journal".into(), number.parse().ok()?));
        }
        let (history, number) = save::occurrence_parts(record)?;
        Some((history.to_string(), number as u64))
    }

    /// At the end of an event: every retired record that nothing pins
    /// departs. Reported after every other effect, by history and number.
    pub(crate) fn depart_unpinned(&mut self) -> Result<(), String> {
        #[cfg(debug_assertions)]
        self.check_departure_index();
        let mut departing = BTreeMap::new();
        // Only retirements and released pins queue candidates. Resolve the
        // complete withdrawal cascade before compaction so effects and archive
        // entries are globally ordered, not ordered by cascade depth.
        while let Some(record) = Arc::make_mut(&mut self.departure_candidates).pop_first() {
            if !self.retired.contains_key(&record) || self.pin_counts.contains_key(&record) {
                continue;
            }
            if let Some(place) = self.departable(&record) {
                if let Some(reason) = Arc::make_mut(&mut self.withdrawal_reasons).remove(&record) {
                    self.release_pin(&reason);
                }
                departing.insert(record, place);
            }
        }
        if departing.is_empty() {
            return Ok(());
        }
        let sequence = self.sequence;
        let mut holders: BTreeMap<String, Vec<Holder>> = BTreeMap::new();
        let holder = |kind: &'static str, name: &str, field: &'static str| {
            let name = name.to_string();
            move || Holder {
                kind,
                name: name.clone(),
                field,
            }
        };

        // 1. Every provenance that names a departing record compacts.
        for slot in 0..self.states.len() {
            let cell = &self.states.cells[slot];
            if !names_any(&cell.value.provenance, &departing)
                && !names_any(&cell.grounds, &departing)
            {
                continue;
            }
            let name = self.states.names[slot].clone();
            let cell = self.states.cell_mut(slot);
            compact(
                &mut cell.value.provenance,
                &departing,
                sequence,
                &self.source_id,
                holder("state", &name, "lineage"),
                &mut holders,
            );
            compact(
                &mut cell.grounds,
                &departing,
                sequence,
                &self.source_id,
                holder("state", &name, "grounds"),
                &mut holders,
            );
        }
        if self
            .commitment_bases
            .values()
            .any(|basis| names_any(&basis.provenance, &departing))
        {
            for (name, basis) in Arc::make_mut(&mut self.commitment_bases).iter_mut() {
                compact(
                    &mut basis.provenance,
                    &departing,
                    sequence,
                    &self.source_id,
                    holder("commitment", name, "basis"),
                    &mut holders,
                );
            }
        }
        if self
            .commitment_grounds
            .values()
            .any(|grounds| names_any(grounds, &departing))
        {
            for (name, grounds) in Arc::make_mut(&mut self.commitment_grounds).iter_mut() {
                compact(
                    grounds,
                    &departing,
                    sequence,
                    &self.source_id,
                    holder("commitment", name, "grounds"),
                    &mut holders,
                );
            }
        }
        if self.reading_streams.values().any(|stream| {
            names_any(&stream.selection_qualifications, &departing)
                || stream
                    .occurrences
                    .iter()
                    .any(|reading| names_any(&reading.provenance, &departing))
        }) {
            for (name, stream) in Arc::make_mut(&mut self.reading_streams).iter_mut() {
                compact(
                    &mut stream.selection_qualifications,
                    &departing,
                    sequence,
                    &self.source_id,
                    holder("stream", name, "selection_qualifications"),
                    &mut holders,
                );
                for reading in &mut stream.occurrences {
                    if departing.contains_key(&reading.id) {
                        continue;
                    }
                    compact(
                        &mut reading.provenance,
                        &departing,
                        sequence,
                        &self.source_id,
                        holder("stream", &reading.id, "provenance"),
                        &mut holders,
                    );
                }
            }
        }
        if self
            .decision_series
            .values()
            .any(|series| names_any(&series.selection_qualifications, &departing))
        {
            for (name, series) in Arc::make_mut(&mut self.decision_series).iter_mut() {
                compact(
                    &mut series.selection_qualifications,
                    &departing,
                    sequence,
                    &self.source_id,
                    holder("series", name, "selection_qualifications"),
                    &mut holders,
                );
            }
        }
        for (field, map) in [
            (
                "observation_qualifications",
                &mut self.observation_qualifications,
            ),
            (
                "examination_qualifications",
                &mut self.examination_qualifications,
            ),
            (
                "reopening_qualifications",
                &mut self.reopening_qualifications,
            ),
        ] {
            if map
                .values()
                .any(|provenance| names_any(provenance, &departing))
            {
                for (name, provenance) in Arc::make_mut(map).iter_mut() {
                    if departing.contains_key(name) {
                        continue;
                    }
                    compact(
                        provenance,
                        &departing,
                        sequence,
                        &self.source_id,
                        holder("qualification", name, field),
                        &mut holders,
                    );
                }
            }
        }
        if self.predicate_qualifications.values().any(|targets| {
            targets
                .values()
                .any(|provenance| names_any(provenance, &departing))
        }) {
            for targets in Arc::make_mut(&mut self.predicate_qualifications).values_mut() {
                for (name, provenance) in targets.iter_mut() {
                    if departing.contains_key(name) {
                        continue;
                    }
                    compact(
                        provenance,
                        &departing,
                        sequence,
                        &self.source_id,
                        holder("qualification", name, "predicate_qualifications"),
                        &mut holders,
                    );
                }
            }
        }
        if self
            .scheduled
            .iter()
            .any(|scheduled| names_any(&scheduled.guard, &departing))
        {
            for scheduled in Arc::make_mut(&mut self.scheduled).iter_mut() {
                compact(
                    &mut scheduled.guard,
                    &departing,
                    sequence,
                    &self.source_id,
                    holder("scheduled", &scheduled.evidence, "guard"),
                    &mut holders,
                );
            }
        }
        for (index, provenance) in self.cue_qualifications.iter_mut().enumerate() {
            compact(
                provenance,
                &departing,
                sequence,
                &self.source_id,
                holder("cue", &index.to_string(), "qualifications"),
                &mut holders,
            );
        }

        // 2. Each record leaves the session, into its archive entry.
        let names = self
            .symbols
            .iter()
            .map(|(name, id)| (*id, name.clone()))
            .collect::<HashMap<_, _>>();
        let ids = departing
            .keys()
            .filter_map(|record| self.symbols.get(record).map(|id| (*id, record.clone())))
            .collect::<HashMap<_, _>>();
        let mut relations: BTreeMap<String, Vec<[String; 3]>> = BTreeMap::new();
        if !ids.is_empty() {
            let graph = Arc::make_mut(&mut self.graph);
            graph.edges.retain(|edge| {
                let record = ids.get(&edge.from).or_else(|| ids.get(&edge.to));
                let Some(record) = record else {
                    return true;
                };
                relations.entry(record.clone()).or_default().push([
                    names[&edge.from].clone(),
                    relation_name(edge.relation).into(),
                    names[&edge.to].clone(),
                ]);
                false
            });
            for id in ids.keys() {
                graph.nodes.remove(id);
                graph.origins.remove(id);
            }
            let symbols = Arc::make_mut(&mut self.symbols);
            for record in ids.values() {
                symbols.remove(record);
            }
        }
        if self
            .observations
            .iter()
            .any(|name| departing.contains_key(name))
        {
            Arc::make_mut(&mut self.observations).retain(|name| !departing.contains_key(name));
        }
        let mut entries = BTreeMap::new();
        for (record, (history, number)) in &departing {
            let mut qualifications = BTreeMap::new();
            for (kind, map) in [
                ("observation", &mut self.observation_qualifications),
                ("examination", &mut self.examination_qualifications),
                ("reopening", &mut self.reopening_qualifications),
            ] {
                if map.contains_key(record) {
                    qualifications
                        .insert(kind.to_string(), Arc::make_mut(map).remove(record).unwrap());
                }
            }
            if self
                .predicate_qualifications
                .values()
                .any(|targets| targets.contains_key(record))
            {
                let predicates = Arc::make_mut(&mut self.predicate_qualifications);
                for (kind, targets) in predicates.iter_mut() {
                    if let Some(provenance) = targets.remove(record) {
                        qualifications.insert(kind.clone(), provenance);
                    }
                }
                predicates.retain(|_, targets| !targets.is_empty());
            }
            let withdrawal = self
                .withdrawals
                .iter()
                .position(|withdrawal| &withdrawal.evidence == record)
                .map(|index| Arc::make_mut(&mut self.withdrawals).remove(index));
            let mut reading = None;
            let mut journal_entry = None;
            if history == "journal"
                && self.journal_window.is_some()
                && record.starts_with("journal@")
            {
                journal_entry = Some(());
            } else if let Some(stream) = self.reading_streams.get(history) {
                if let Some(index) = stream.occurrences.iter().position(|r| &r.id == record) {
                    reading = Some(
                        Arc::make_mut(&mut self.reading_streams)
                            .get_mut(history)
                            .unwrap()
                            .occurrences
                            .remove(index),
                    );
                }
            } else if let Some(renewal) = self.renewals.get(history) {
                if let Some(index) = renewal.occurrences.iter().position(|name| name == record) {
                    Arc::make_mut(&mut self.renewals)
                        .get_mut(history)
                        .unwrap()
                        .occurrences
                        .remove(index);
                }
            }
            let retired_at = Arc::make_mut(&mut self.retired)
                .remove(record)
                .expect("a departing record is retired");
            let mut entry_holders = holders.remove(record).unwrap_or_default();
            entry_holders.sort();
            entries.insert(
                record.clone(),
                (
                    journal_entry.is_some(),
                    ArchiveEntry {
                        record: record.clone(),
                        history: history.clone(),
                        number: *number,
                        retired_at,
                        departed_at: sequence,
                        relations: relations.remove(record).unwrap_or_default(),
                        qualifications,
                        reading,
                        journal_entry: None,
                        withdrawal,
                        holders: entry_holders,
                    },
                ),
            );
        }
        // Departing journal entries are the oldest held: nothing pins one.
        let journal_departing = entries.values().filter(|(journal, _)| *journal).count();
        if journal_departing > 0 {
            let journal = Arc::make_mut(&mut self.journal);
            for (index, entry) in journal.drain(..journal_departing).enumerate() {
                let name = format!("journal@{}", self.journal_departed + index + 1);
                if let Some((_, archived)) = entries.get_mut(&name) {
                    archived.journal_entry = Some(entry);
                } else {
                    return Err(format!("journal entry {name} departed out of order"));
                }
            }
            self.journal_departed += journal_departing;
        }

        // 3. Reported after every other effect, by history and number.
        let mut order = entries
            .into_values()
            .map(|(_, entry)| entry)
            .collect::<Vec<_>>();
        order.sort_by(|a, b| a.history.cmp(&b.history).then(a.number.cmp(&b.number)));
        for entry in order {
            self.effects.push(EffectReport::Depart {
                history: entry.history.clone(),
                record: entry.record.clone(),
            });
            self.archive.push(ArchiveItem::Record(Box::new(entry)));
        }
        #[cfg(debug_assertions)]
        self.check_departure_index();
        Ok(())
    }

    /// The departed records since the last drain, in the order they left.
    /// See spec/caveat-departure-0.1.md, "The archive".
    pub fn drain_archive(&mut self) -> Vec<ArchiveItem> {
        std::mem::take(&mut self.archive)
    }

    /// How many departed records wait in the archive for the host to drain.
    pub fn undrained(&self) -> usize {
        self.archive.len()
    }
}

fn settle_provenance(
    provenance: &mut Provenance,
    nodes: &mut Vec<ArchiveProvenance>,
    seen: &mut BTreeSet<String>,
) {
    for marker in provenance.departed.values_mut() {
        if let Some(root) = &mut marker.archive_ref {
            root.settle(nodes, seen);
            #[cfg(debug_assertions)]
            debug_assert!(root.is_settled(), "live provenance retains only its root");
        }
    }
}

impl ReactiveSession {
    /// Publish temporary pure-expression DAGs only after every event effect and
    /// binding succeeds. Unchanged transaction maps already contain opaque roots.
    /// Restore recomputes the same deterministic binding roots, then settles them
    /// without publishing: its archive starts empty.
    pub(crate) fn settle_archive_provenance(&mut self, old: Option<&Self>, publish: bool) {
        if !self.windowed() {
            return;
        }
        let mut nodes = Vec::new();
        let mut seen = BTreeSet::new();
        if old.is_none_or(|old| !Arc::ptr_eq(&old.states.cells, &self.states.cells)) {
            for cell in Arc::make_mut(&mut self.states.cells) {
                if cell
                    .value
                    .provenance
                    .departed
                    .values()
                    .chain(cell.grounds.departed.values())
                    .all(|marker| {
                        marker
                            .archive_ref
                            .as_ref()
                            .is_none_or(|root| root.is_settled())
                    })
                {
                    continue;
                }
                let cell = Arc::make_mut(cell);
                settle_provenance(&mut cell.value.provenance, &mut nodes, &mut seen);
                settle_provenance(&mut cell.grounds, &mut nodes, &mut seen);
            }
        }
        if old.is_none_or(|old| !Arc::ptr_eq(&old.commitment_bases, &self.commitment_bases)) {
            for basis in Arc::make_mut(&mut self.commitment_bases).values_mut() {
                settle_provenance(&mut basis.provenance, &mut nodes, &mut seen);
            }
        }
        if old.is_none_or(|old| !Arc::ptr_eq(&old.commitment_grounds, &self.commitment_grounds)) {
            for grounds in Arc::make_mut(&mut self.commitment_grounds).values_mut() {
                settle_provenance(grounds, &mut nodes, &mut seen);
            }
        }
        if old.is_none_or(|old| !Arc::ptr_eq(&old.reading_streams, &self.reading_streams)) {
            for stream in Arc::make_mut(&mut self.reading_streams).values_mut() {
                settle_provenance(&mut stream.selection_qualifications, &mut nodes, &mut seen);
                for reading in &mut stream.occurrences {
                    settle_provenance(&mut reading.provenance, &mut nodes, &mut seen);
                }
            }
        }
        if old.is_none_or(|old| !Arc::ptr_eq(&old.decision_series, &self.decision_series)) {
            for series in Arc::make_mut(&mut self.decision_series).values_mut() {
                settle_provenance(&mut series.selection_qualifications, &mut nodes, &mut seen);
            }
        }
        for (map, unchanged) in [
            (
                &mut self.observation_qualifications,
                old.map(|old| &old.observation_qualifications),
            ),
            (
                &mut self.examination_qualifications,
                old.map(|old| &old.examination_qualifications),
            ),
            (
                &mut self.reopening_qualifications,
                old.map(|old| &old.reopening_qualifications),
            ),
        ] {
            if unchanged.is_none_or(|old| !Arc::ptr_eq(old, map)) {
                for provenance in Arc::make_mut(map).values_mut() {
                    settle_provenance(provenance, &mut nodes, &mut seen);
                }
            }
        }
        if old.is_none_or(|old| {
            !Arc::ptr_eq(
                &old.predicate_qualifications,
                &self.predicate_qualifications,
            )
        }) {
            for targets in Arc::make_mut(&mut self.predicate_qualifications).values_mut() {
                for provenance in targets.values_mut() {
                    settle_provenance(provenance, &mut nodes, &mut seen);
                }
            }
        }
        if old.is_none_or(|old| !Arc::ptr_eq(&old.scheduled, &self.scheduled)) {
            for scheduled in Arc::make_mut(&mut self.scheduled) {
                settle_provenance(&mut scheduled.guard, &mut nodes, &mut seen);
            }
        }
        for provenance in &mut self.cue_qualifications {
            settle_provenance(provenance, &mut nodes, &mut seen);
        }
        for map in [
            &mut self.binding_qualifications,
            &mut self.binding_explanations,
        ] {
            for targets in map.values_mut() {
                for provenance in targets.values_mut() {
                    settle_provenance(provenance, &mut nodes, &mut seen);
                }
            }
        }
        for item in &mut self.archive {
            if let ArchiveItem::Record(record) = item {
                for provenance in record.qualifications.values_mut() {
                    settle_provenance(provenance, &mut nodes, &mut seen);
                }
                if let Some(reading) = &mut record.reading {
                    settle_provenance(&mut reading.provenance, &mut nodes, &mut seen);
                }
            }
        }
        if publish {
            self.archive
                .extend(nodes.into_iter().map(ArchiveItem::Provenance));
        }
    }
}
