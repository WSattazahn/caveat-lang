//! Save a reactive session and resume it without replaying its events. See
//! spec/caveat-save-0.1.md.
//!
//! A save holds everything an event can change, by name. Restoring loads the
//! source again and applies the save to it, so declarations always come from
//! the source, and bindings are computed again rather than trusted. A save is
//! data a host stored and may hand back altered: every name, kind, range and
//! limit is checked, and a malformed save is an error, never a panic.
use super::*;
use serde::Deserialize;

pub const REACTIVE_SAVE_SCHEMA: &str = "caveat-reactive-save/0.1";
const MAX_SAVE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReactiveSave {
    pub schema: String,
    /// The program this save belongs to; restoring checks it.
    pub source_id: String,
    pub sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event: Option<String>,
    #[serde(default)]
    pub elapsed: f64,
    /// States that differ from the value the program gives them when it loads.
    pub states: BTreeMap<String, SavedState>,
    #[serde(default)]
    pub graph: SavedGraph,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub commitment_bases: BTreeMap<String, CommitmentBasis>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub commitment_grounds: BTreeMap<String, Compact>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub reading_streams: BTreeMap<String, ReadingStream>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub decision_series: BTreeMap<String, DecisionSeries>,
    /// Each renewable evidence's occurrences, first to current.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub renewals: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scheduled_qualifications: Vec<ScheduledQualification>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub observation_qualifications: BTreeMap<String, Compact>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub examination_qualifications: BTreeMap<String, Compact>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub reopening_qualifications: BTreeMap<String, Compact>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub predicate_qualifications: BTreeMap<String, BTreeMap<String, Compact>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_journal: Vec<JournalEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<SavedResources>,
    /// The last event's effects and cues, so the view is the same after resuming.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<EffectReport>,
    /// Cue ids; their content comes from the source.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cues: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cue_qualifications: Vec<Compact>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedState {
    pub value: f64,
    #[serde(default, skip_serializing_if = "Compact::is_empty")]
    pub lineage: Compact,
    /// Left out when the grounds are the lineage, as they usually are.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grounds: Option<Compact>,
}

/// A provenance as a save writes it: an empty list is left out.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Compact(pub Provenance);

impl Compact {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&Provenance> for Compact {
    fn from(provenance: &Provenance) -> Self {
        Self(provenance.clone())
    }
}

impl Serialize for Compact {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        if !self.0.evidence.is_empty() {
            map.serialize_entry("evidence", &self.0.evidence)?;
        }
        if !self.0.caveats.is_empty() {
            map.serialize_entry("caveats", &self.0.caveats)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Compact {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Provenance::deserialize(deserializer).map(Self)
    }
}

fn compact_map(records: &BTreeMap<String, Provenance>) -> BTreeMap<String, Compact> {
    records
        .iter()
        .map(|(name, provenance)| (name.clone(), provenance.into()))
        .collect()
}

fn full_map(records: &BTreeMap<String, Compact>) -> BTreeMap<String, Provenance> {
    records
        .iter()
        .map(|(name, compact)| (name.clone(), compact.0.clone()))
        .collect()
}

/// What events did to the graph, since the program loaded.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedGraph {
    /// Nodes events created, in order: occurrences of reading streams and
    /// renewable evidence, which are rebuilt from their names, and commitments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<SavedNode>,
    /// Relations events added, in order, as `[from, relation, to]`. Their
    /// order is the order evidence was observed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<[String; 3]>,
    /// Caveats whose attention is not `unexamined`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attention: BTreeMap<String, String>,
    /// Commitments that are open.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SavedNode {
    Occurrence { name: String },
    Commitment { name: String, reason: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedResources {
    pub remaining: u64,
    pub spent: u64,
    pub exhausted: bool,
}

fn relation_from_name(name: &str) -> Result<Relation, String> {
    Ok(match name {
        "supports" => Relation::Supports,
        "opposes" => Relation::Opposes,
        "qualifies" => Relation::Qualifies,
        "in_context" => Relation::InContext,
        "retains" => Relation::Retains,
        "reopens" => Relation::Reopens,
        "relies_on" => Relation::ReliesOn,
        _ => return Err(format!("unknown relation {name}")),
    })
}

fn reason_name(reason: &StopReason) -> Result<&'static str, String> {
    Ok(match reason {
        StopReason::Enough => "enough",
        StopReason::BudgetExhausted => "budget",
        StopReason::Deadline => "deadline",
        StopReason::ExternalDecision(_) => {
            return Err("a reactive session cannot save an external decision".into())
        }
    })
}

fn reason_from_name(name: &str) -> Result<StopReason, String> {
    Ok(match name {
        "enough" => StopReason::Enough,
        "budget" => StopReason::BudgetExhausted,
        "deadline" => StopReason::Deadline,
        _ => return Err(format!("unknown commit reason {name}")),
    })
}

fn attention_name(attention: Attention) -> &'static str {
    match attention {
        Attention::Unexamined => "unexamined",
        Attention::Deferred => "deferred",
        Attention::Examining => "examining",
        Attention::Examined => "examined",
    }
}

fn attention_from_name(name: &str) -> Result<Attention, String> {
    Ok(match name {
        "unexamined" => Attention::Unexamined,
        "deferred" => Attention::Deferred,
        "examining" => Attention::Examining,
        "examined" => Attention::Examined,
        _ => return Err(format!("unknown attention {name}")),
    })
}

/// `NAME@N` split into its parts.
fn occurrence_parts(name: &str) -> Option<(&str, usize)> {
    let (base, ordinal) = name.rsplit_once('@')?;
    let ordinal: usize = ordinal.parse().ok()?;
    (ordinal >= 1 && ordinal.to_string() == name[base.len() + 1..]).then_some((base, ordinal))
}

impl ReactiveSession {
    /// Everything this session knows that an event can change.
    pub fn save(&self) -> Result<ReactiveSave, String> {
        let names = self
            .symbols
            .iter()
            .map(|(name, id)| (*id, name.as_str()))
            .collect::<HashMap<_, _>>();
        let name_of = |id: &NodeId| {
            names
                .get(id)
                .map(|name| name.to_string())
                .ok_or_else(|| format!("graph node {id} has no name"))
        };
        let mut created = self
            .graph
            .nodes
            .keys()
            .filter(|id| **id > self.loaded.last_node)
            .copied()
            .collect::<Vec<_>>();
        created.sort_unstable();
        let mut nodes = Vec::new();
        for id in created {
            let name = name_of(&id)?;
            nodes.push(match &self.graph.nodes[&id] {
                NodeKind::Evidence { .. } => SavedNode::Occurrence { name },
                NodeKind::Commitment { stop_reason, .. } => SavedNode::Commitment {
                    name,
                    reason: reason_name(stop_reason)?.into(),
                },
                _ => return Err(format!("cannot save graph node {name}")),
            });
        }
        let mut relations = Vec::new();
        for edge in &self.graph.edges[self.loaded.edges..] {
            relations.push([
                name_of(&edge.from)?,
                relation_name(edge.relation).into(),
                name_of(&edge.to)?,
            ]);
        }
        let mut attention = BTreeMap::new();
        let mut open = Vec::new();
        for (id, node) in &self.graph.nodes {
            match node {
                NodeKind::Caveat {
                    attention: state, ..
                } if *state != Attention::Unexamined => {
                    attention.insert(name_of(id)?, attention_name(*state).into());
                }
                NodeKind::Commitment { open: true, .. } => open.push(name_of(id)?),
                _ => {}
            }
        }
        open.sort();
        Ok(ReactiveSave {
            schema: REACTIVE_SAVE_SCHEMA.into(),
            source_id: self.source_id.clone(),
            sequence: self.sequence,
            last_event: self.last_event.clone(),
            elapsed: self.elapsed,
            states: self
                .states
                .names()
                .zip(self.states.cells.iter().zip(self.loaded.states.iter()))
                .filter(|(_, (cell, loaded))| !Arc::ptr_eq(cell, loaded) && cell != loaded)
                .map(|(name, (cell, _))| {
                    let state = SavedState {
                        value: cell.value.value,
                        lineage: (&cell.value.provenance).into(),
                        grounds: (cell.grounds != cell.value.provenance)
                            .then(|| (&cell.grounds).into()),
                    };
                    (name.clone(), state)
                })
                .collect(),
            graph: SavedGraph {
                nodes,
                relations,
                attention,
                open,
            },
            commitment_bases: (*self.commitment_bases).clone(),
            commitment_grounds: compact_map(&self.commitment_grounds),
            reading_streams: (*self.reading_streams).clone(),
            decision_series: (*self.decision_series).clone(),
            renewals: self
                .renewals
                .iter()
                .filter(|(_, renewal)| renewal.occurrences.len() > 1)
                .map(|(name, renewal)| (name.clone(), renewal.occurrences.clone()))
                .collect(),
            scheduled_qualifications: (*self.scheduled).clone(),
            observation_qualifications: compact_map(&self.observation_qualifications),
            examination_qualifications: compact_map(&self.examination_qualifications),
            reopening_qualifications: compact_map(&self.reopening_qualifications),
            predicate_qualifications: self
                .predicate_qualifications
                .iter()
                .map(|(kind, targets)| (kind.clone(), compact_map(targets)))
                .collect(),
            decision_journal: (*self.journal).clone(),
            resources: self.resources.as_ref().map(|ledger| SavedResources {
                remaining: ledger.remaining,
                spent: ledger.spent,
                exhausted: ledger.exhausted,
            }),
            effects: self.effects.clone(),
            cues: self.cues.iter().map(|cue| cue.id().to_string()).collect(),
            cue_qualifications: self.cue_qualifications.iter().map(Compact::from).collect(),
        })
    }

    pub fn save_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.save()?).map_err(|error| error.to_string())
    }

    /// A session of `source` resumed from `save`: the same state, and the
    /// same outcome for every later event, as the session that was saved.
    pub fn restore(source: &str, save: &ReactiveSave) -> Result<Self, String> {
        let mut session = Self::from_source(source)?;
        session
            .apply_save(save)
            .map_err(|error| format!("cannot restore save: {error}"))?;
        Ok(session)
    }

    pub fn restore_json(source: &str, save: &str) -> Result<Self, String> {
        if save.len() > MAX_SAVE_BYTES {
            return Err(format!(
                "cannot restore save: larger than {MAX_SAVE_BYTES} bytes"
            ));
        }
        let save: ReactiveSave =
            serde_json::from_str(save).map_err(|error| format!("cannot restore save: {error}"))?;
        Self::restore(source, &save)
    }

    fn apply_save(&mut self, save: &ReactiveSave) -> Result<(), String> {
        if save.schema != REACTIVE_SAVE_SCHEMA {
            return Err(format!(
                "schema {} is not {REACTIVE_SAVE_SCHEMA}",
                save.schema
            ));
        }
        if save.source_id != self.source_id {
            return Err("it belongs to a different program".into());
        }
        if let Some(event) = &save.last_event {
            if !self.events.contains_key(event) {
                return Err(format!("unknown event {event}"));
            }
        }
        if !save.elapsed.is_finite() {
            return Err("elapsed time must be a finite number".into());
        }
        if self.source_clock_is_monotonic() && save.elapsed < 0.0 {
            return Err("elapsed time must be a nonnegative number".into());
        }
        self.sequence = save.sequence;
        self.last_event = save.last_event.clone();
        self.elapsed = save.elapsed;
        self.restore_graph(&save.graph)?;
        self.restore_states(&save.states)?;
        self.restore_records(save)?;
        self.evaluate_bindings(None)
    }

    fn restore_graph(&mut self, saved: &SavedGraph) -> Result<(), String> {
        let committable = self.committable_actions();
        for node in &saved.nodes {
            let (name, kind) = match node {
                SavedNode::Occurrence { name } => {
                    let (base, ordinal) = occurrence_parts(name)
                        .ok_or_else(|| format!("{name} is not an occurrence name"))?;
                    let (template, description) =
                        if let Some(stream) = self.reading_streams.get(base) {
                            (
                                stream.template.clone(),
                                format!("({base} reading {ordinal})"),
                            )
                        } else if self.renewals.contains_key(base) && ordinal >= 2 {
                            (base.to_string(), format!("(occurrence {ordinal})"))
                        } else {
                            return Err(format!(
                            "{name} is not an occurrence of a reading stream or renewable evidence"
                        ));
                        };
                    let Some(NodeKind::Evidence {
                        description: base_description,
                        source,
                    }) = self
                        .symbols
                        .get(&template)
                        .and_then(|id| self.graph.nodes.get(id))
                    else {
                        return Err(format!("{name}: {template} is not evidence"));
                    };
                    let kind = NodeKind::Evidence {
                        description: format!("{base_description} {description}"),
                        source: source.clone(),
                    };
                    (name, kind)
                }
                SavedNode::Commitment { name, reason } => {
                    let series = occurrence_parts(name)
                        .is_some_and(|(base, _)| self.decision_series.contains_key(base));
                    if !series && !committable.contains(name) {
                        return Err(format!("{name} is not a commitment this program makes"));
                    }
                    let kind = NodeKind::Commitment {
                        action: name.clone(),
                        open: false,
                        stop_reason: reason_from_name(reason)?,
                    };
                    (name, kind)
                }
            };
            if self.symbols.contains_key(name) {
                return Err(format!("{name} already exists"));
            }
            let id = Arc::make_mut(&mut self.graph).add(kind);
            Arc::make_mut(&mut self.symbols).insert(name.clone(), id);
        }
        for [from, relation, to] in &saved.relations {
            let lookup = |name: &str| {
                self.symbols
                    .get(name)
                    .copied()
                    .ok_or_else(|| format!("relation names unknown {name}"))
            };
            let (from, to) = (lookup(from)?, lookup(to)?);
            let relation = relation_from_name(relation)?;
            Arc::make_mut(&mut self.graph).relate(from, relation, to);
        }
        for (name, attention) in &saved.attention {
            self.require_kind(name, "caveat")?;
            let id = self.symbols[name];
            Arc::make_mut(&mut self.graph).set_attention(id, attention_from_name(attention)?);
        }
        for name in &saved.open {
            let id = self.symbols.get(name).copied();
            match id.and_then(|id| Arc::make_mut(&mut self.graph).nodes.get_mut(&id)) {
                Some(NodeKind::Commitment { open, .. }) => *open = true,
                _ => return Err(format!("{name} is not a commitment")),
            }
        }
        Ok(())
    }

    /// Every action a rule or procedure can commit.
    fn committable_actions(&self) -> HashSet<String> {
        self.rules
            .iter()
            .map(|rule| &rule.effect)
            .chain(
                self.procedures
                    .values()
                    .flat_map(|procedure| procedure.body.iter().map(|step| &step.effect)),
            )
            .filter_map(|effect| match effect {
                Effect::Commit { action, .. } => Some(action.clone()),
                _ => None,
            })
            .collect()
    }

    /// A provenance whose names are all declared or created evidence and caveats.
    fn check_provenance(&self, what: &str, provenance: &Provenance) -> Result<(), String> {
        provenance
            .validate()
            .map_err(|error| format!("{what}: {error}"))?;
        for name in &provenance.evidence {
            self.require_kind(name, "evidence")
                .map_err(|error| format!("{what}: {error}"))?;
        }
        for name in &provenance.caveats {
            self.require_kind(name, "caveat")
                .map_err(|error| format!("{what}: {error}"))?;
        }
        Ok(())
    }

    fn require_commitment(&self, name: &str) -> Result<(), String> {
        match self
            .symbols
            .get(name)
            .and_then(|id| self.graph.nodes.get(id))
        {
            Some(NodeKind::Commitment { .. }) => Ok(()),
            _ => Err(format!("{name} is not a commitment")),
        }
    }

    fn restore_states(&mut self, saved: &BTreeMap<String, SavedState>) -> Result<(), String> {
        if let Some(name) = saved.keys().find(|name| !self.states.contains_key(name)) {
            return Err(format!("its states are not this program's: {name}"));
        }
        // A state the save leaves out keeps the value the program just gave it.
        for (name, state) in saved {
            let lineage = &state.lineage.0;
            let grounds = state.grounds.as_ref().map_or(lineage, |grounds| &grounds.0);
            let range = &self.ranges[name];
            if !state.value.is_finite() {
                return Err(format!("state {name} is not a finite number"));
            }
            check_range(name, state.value, range.min, range.max)?;
            self.check_provenance(&format!("state {name}"), lineage)?;
            self.check_provenance(&format!("state {name} grounds"), grounds)?;
            let slot = self.states.slot(name).expect("checked above");
            self.states.set(
                slot,
                StateCell {
                    value: Tracked::new(state.value, lineage.clone())?,
                    grounds: grounds.clone(),
                },
            );
        }
        Ok(())
    }

    fn restore_records(&mut self, save: &ReactiveSave) -> Result<(), String> {
        for (name, basis) in &save.commitment_bases {
            self.require_commitment(name)?;
            if basis.value.is_some_and(|value| !value.is_finite()) {
                return Err(format!("commitment {name} basis is not a finite number"));
            }
            self.check_provenance(&format!("commitment {name}"), &basis.provenance)?;
        }
        for (name, grounds) in &save.commitment_grounds {
            self.require_commitment(name)?;
            self.check_provenance(&format!("commitment {name} grounds"), &grounds.0)?;
        }
        if save.reading_streams.keys().ne(self.reading_streams.keys()) {
            return Err("its reading streams are not this program's".into());
        }
        for (name, stream) in &save.reading_streams {
            let declared = &self.reading_streams[name];
            if stream.template != declared.template
                || stream.limit != declared.limit
                || stream.occurrences.len() > stream.limit
            {
                return Err(format!("reading stream {name} does not match the program"));
            }
            for occurrence in &stream.occurrences {
                self.require_kind(&occurrence.id, "evidence")?;
                if !occurrence.value.is_finite() {
                    return Err(format!("reading {} is not a finite number", occurrence.id));
                }
                self.check_provenance(
                    &format!("reading {}", occurrence.id),
                    &occurrence.provenance,
                )?;
            }
            if stream.current.as_ref() != stream.occurrences.last().map(|occurrence| &occurrence.id)
            {
                return Err(format!(
                    "reading stream {name} current is not its latest reading"
                ));
            }
            self.check_provenance(
                &format!("reading stream {name}"),
                &stream.selection_qualifications,
            )?;
        }
        if save.decision_series.keys().ne(self.decision_series.keys()) {
            return Err("its decision series are not this program's".into());
        }
        for (name, series) in &save.decision_series {
            if series.limit != self.decision_series[name].limit
                || series.revisions.len() > series.limit
            {
                return Err(format!("decision series {name} does not match the program"));
            }
            for revision in &series.revisions {
                self.require_commitment(&revision.id)?;
                if !save.commitment_bases.contains_key(&revision.id) {
                    return Err(format!("revision {} has no basis", revision.id));
                }
            }
            if series.current.as_ref() != series.revisions.last().map(|revision| &revision.id) {
                return Err(format!(
                    "decision series {name} current is not its latest revision"
                ));
            }
            self.check_provenance(
                &format!("decision series {name}"),
                &series.selection_qualifications,
            )?;
        }
        for (name, occurrences) in &save.renewals {
            let declared = self
                .renewals
                .get(name)
                .ok_or_else(|| format!("{name} is not renewable"))?;
            if occurrences.first() != Some(name) || occurrences.len() > declared.limit {
                return Err(format!(
                    "renewable {name} occurrences do not match the program"
                ));
            }
            for (ordinal, occurrence) in occurrences.iter().enumerate().skip(1) {
                if *occurrence != format!("{name}@{}", ordinal + 1) {
                    return Err(format!(
                        "renewable {name} occurrence {occurrence} is out of order"
                    ));
                }
                self.require_kind(occurrence, "evidence")?;
            }
        }
        if save.scheduled_qualifications.len() > MAX_SCHEDULED_QUALIFICATIONS {
            return Err("too many scheduled qualifications".into());
        }
        for scheduled in &save.scheduled_qualifications {
            self.require_kind(&scheduled.evidence, "evidence")?;
            self.require_kind(&scheduled.caveat, "caveat")?;
            if !(scheduled.after.is_finite()
                && scheduled.after >= 0.0
                && scheduled.scheduled_at.is_finite())
            {
                return Err("a scheduled qualification's times are not valid".into());
            }
            self.check_provenance("scheduled qualification", &scheduled.guard)?;
        }
        for (what, records) in [
            ("observation", &save.observation_qualifications),
            ("examination", &save.examination_qualifications),
            ("reopening", &save.reopening_qualifications),
        ] {
            for (name, provenance) in records {
                self.check_provenance(&format!("{what} of {name}"), &provenance.0)?;
            }
        }
        for (kind, targets) in &save.predicate_qualifications {
            if !matches!(
                kind.as_str(),
                "observed" | "examined" | "committed" | "reopened"
            ) {
                return Err(format!("unknown predicate {kind}"));
            }
            for (name, provenance) in targets {
                self.check_provenance(&format!("{kind}({name})"), &provenance.0)?;
            }
        }
        match (&mut self.resources, &save.resources) {
            (None, None) => {}
            (Some(ledger), Some(saved))
                if saved.remaining.checked_add(saved.spent) == Some(ledger.initial)
                    && saved.exhausted == (saved.remaining == 0) =>
            {
                ledger.remaining = saved.remaining;
                ledger.spent = saved.spent;
                ledger.exhausted = saved.exhausted;
            }
            _ => return Err("its attention budget does not match the program".into()),
        }
        if save.cue_qualifications.len() != save.cues.len() {
            return Err("its cues and their qualifications do not match".into());
        }
        let mut cues = Vec::new();
        for (id, qualification) in save.cues.iter().zip(&save.cue_qualifications) {
            cues.push(
                self.cue_definitions
                    .get(id)
                    .cloned()
                    .ok_or_else(|| format!("unknown cue {id}"))?,
            );
            self.check_provenance(&format!("cue {id}"), &qualification.0)?;
        }
        for effect in &save.effects {
            let names: Vec<&str> = match effect {
                EffectReport::Sample { id, target, .. } => vec![id, target],
                EffectReport::Reveal {
                    evidence, target, ..
                } => vec![evidence, target],
                EffectReport::Examine { caveat, .. } => vec![caveat],
                EffectReport::Commit { action, retained } => std::iter::once(action)
                    .chain(retained)
                    .map(String::as_str)
                    .collect(),
                EffectReport::Reopen { action, because } => vec![action, because],
                EffectReport::Qualify { evidence, caveat } => vec![evidence, caveat],
                EffectReport::Renew { occurrence, .. } => vec![occurrence],
            };
            if let Some(name) = names
                .into_iter()
                .find(|name| !self.symbols.contains_key(*name))
            {
                return Err(format!("an effect names unknown {name}"));
            }
        }
        self.check_saved_journal(save)?;
        self.commitment_bases = Arc::new(save.commitment_bases.clone());
        self.commitment_grounds = Arc::new(full_map(&save.commitment_grounds));
        self.reading_streams = Arc::new(save.reading_streams.clone());
        self.decision_series = Arc::new(save.decision_series.clone());
        let renewals = Arc::make_mut(&mut self.renewals);
        for (name, occurrences) in &save.renewals {
            renewals.get_mut(name).expect("checked above").occurrences = occurrences.clone();
        }
        self.scheduled = Arc::new(save.scheduled_qualifications.clone());
        self.observation_qualifications = Arc::new(full_map(&save.observation_qualifications));
        self.examination_qualifications = Arc::new(full_map(&save.examination_qualifications));
        self.reopening_qualifications = Arc::new(full_map(&save.reopening_qualifications));
        self.predicate_qualifications = Arc::new(
            save.predicate_qualifications
                .iter()
                .map(|(kind, targets)| (kind.clone(), full_map(targets)))
                .collect(),
        );
        self.journal = Arc::new(save.decision_journal.clone());
        self.effects = save.effects.clone();
        self.cues = cues;
        self.cue_qualifications = save
            .cue_qualifications
            .iter()
            .map(|compact| compact.0.clone())
            .collect();
        Ok(())
    }

    /// Custom clocks may admit negative dt. A save must retain that source's
    /// time semantics, including pending qualifications scheduled below zero.
    fn source_clock_is_monotonic(&self) -> bool {
        self.time_event
            .as_deref()
            .and_then(|event| self.events.get(event))
            .and_then(|parameters| parameters.iter().find(|parameter| parameter.name == "dt"))
            .is_none_or(|parameter| parameter.min.value() >= 0.0)
    }

    /// Check historical records against the source and the independently saved
    /// graph/bases. This establishes consistency, not authenticity: without an
    /// event log or signature an internally consistent edited save is still data.
    fn check_saved_journal(&self, save: &ReactiveSave) -> Result<(), String> {
        let fail = |message: &str| format!("decision journal: {message}");
        if (save.sequence == 0) != save.last_event.is_none() {
            return Err(fail("sequence and last event disagree"));
        }
        let monotonic_time = self.source_clock_is_monotonic();
        let mut committed = HashSet::new();
        let mut reopened: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut current: HashMap<&str, &str> = HashMap::new();
        let mut previous: Option<&JournalEntry> = None;
        let mut latest_elapsed = None;
        let mut last_committed_node = None;
        for entry in &save.decision_journal {
            self.require_commitment(&entry.commitment)?;
            if !matches!(entry.change.as_str(), "committed" | "reopened") {
                return Err(fail("unknown change"));
            }
            if entry.sequence == 0 || entry.sequence > save.sequence {
                return Err(fail("entry sequence is outside the session"));
            }
            if !self.events.contains_key(&entry.event)
                || (entry.sequence == save.sequence
                    && save.last_event.as_ref() != Some(&entry.event))
                || previous.is_some_and(|previous| {
                    entry.sequence < previous.sequence
                        || (entry.sequence == previous.sequence && entry.event != previous.event)
                })
            {
                return Err(fail("entry event or sequence order is inconsistent"));
            }
            if !self.event_can_change_decision(&entry.event, &entry.decision, &entry.change) {
                return Err(fail("event cannot make this decision change"));
            }
            if let Some(elapsed) = entry.elapsed {
                if !elapsed.is_finite()
                    || (monotonic_time
                        && (elapsed < 0.0
                            || elapsed > save.elapsed
                            || latest_elapsed.is_some_and(|previous| elapsed < previous)))
                    || previous.is_some_and(|previous| {
                        previous.sequence == entry.sequence
                            && previous.elapsed.is_some_and(|previous| previous != elapsed)
                    })
                    || (entry.sequence == save.sequence && elapsed != save.elapsed)
                {
                    return Err(fail("entry elapsed time is inconsistent"));
                }
                latest_elapsed = Some(elapsed);
            }
            let basis = save.commitment_bases.get(&entry.commitment);
            if entry.value.is_some_and(|value| {
                !value.is_finite() || basis.and_then(|basis| basis.value) != Some(value)
            }) {
                return Err(fail("entry value differs from its frozen commitment"));
            }
            let provenance = Provenance::from_names(
                entry.because.iter().cloned(),
                entry.caveats.iter().cloned(),
            )?;
            self.check_provenance("decision journal", &provenance)?;
            if provenance.evidence.len() != entry.because.len()
                || entry.because != self.in_observation_order(provenance.evidence.iter())
                || entry.caveats != provenance.caveats.iter().cloned().collect::<Vec<_>>()
                || entry
                    .because
                    .iter()
                    .any(|name| !self.predicate("observed", name).unwrap_or(false))
            {
                return Err(fail(
                    "entry witnesses are unobserved, repeated, or out of order",
                ));
            }
            if let Some(series) = save.decision_series.get(&entry.decision) {
                let revision = series
                    .revisions
                    .iter()
                    .find(|revision| revision.id == entry.commitment)
                    .ok_or_else(|| fail("commitment does not belong to its decision series"))?;
                if entry.change == "committed" {
                    if revision.sequence != entry.sequence || revision.event != entry.event {
                        return Err(fail("commitment disagrees with its revision record"));
                    }
                    if revision.previous.as_deref() != current.get(entry.decision.as_str()).copied()
                        || revision
                            .previous
                            .as_deref()
                            .is_some_and(|name| !reopened.contains_key(name))
                    {
                        return Err(fail("revision has no preceding reopened commitment"));
                    }
                    current.insert(&entry.decision, &entry.commitment);
                } else if current.get(entry.decision.as_str()).copied()
                    != Some(entry.commitment.as_str())
                {
                    return Err(fail("reopening is not of the current revision"));
                }
            } else if entry.decision != entry.commitment {
                return Err(fail("decision and commitment names disagree"));
            }
            if entry.change == "committed" {
                let node = self.symbols[&entry.commitment];
                if last_committed_node.is_some_and(|previous| node <= previous) {
                    return Err(fail("commitment order differs from graph creation order"));
                }
                last_committed_node = Some(node);
                if !committed.insert(entry.commitment.as_str()) {
                    return Err(fail("commitment appears more than once"));
                }
                let grounds = save
                    .commitment_grounds
                    .get(&entry.commitment)
                    .ok_or_else(|| fail("commitment has no frozen grounds"))?;
                if basis.is_none() || grounds.0 != provenance {
                    return Err(fail("commitment witnesses differ from its frozen grounds"));
                }
            } else {
                if entry.because.is_empty()
                    || (!committed.contains(entry.commitment.as_str())
                        && self.symbols[&entry.commitment] > self.loaded.last_node)
                {
                    return Err(fail("reopening has no cause or preceding commitment"));
                }
                let seen = reopened.entry(&entry.commitment).or_default();
                let mut live_caveats = BTreeSet::new();
                for evidence in &entry.because {
                    if seen.contains(&evidence.as_str()) {
                        return Err(fail("reopening repeats an existing cause"));
                    }
                    seen.push(evidence);
                    live_caveats.extend(self.qualify_core(evidence, &[])?.caveats);
                }
                // Qualification only adds caveats. Historical caveats may be a
                // strict subset of today's caveats; equality would rewrite time.
                if !provenance.caveats.is_subset(&live_caveats) {
                    return Err(fail("reopening cites caveats unrelated to its witnesses"));
                }
            }
            previous = Some(entry);
        }
        for (name, series) in &save.decision_series {
            for (index, revision) in series.revisions.iter().enumerate() {
                if revision.ordinal != index as u64 + 1
                    || revision.id != format!("{name}@{}", index + 1)
                    || !committed.contains(revision.id.as_str())
                {
                    return Err(fail("revision identity/order has no matching commitment"));
                }
            }
        }
        for (name, id) in self.symbols.iter() {
            let Some(NodeKind::Commitment { open, .. }) = self.graph.nodes.get(id) else {
                continue;
            };
            if *id > self.loaded.last_node && !committed.contains(name.as_str()) {
                return Err(fail("created commitment has no journal entry"));
            }
            let causes = reopened.get(name.as_str()).cloned().unwrap_or_default();
            let edges = self
                .graph
                .edges
                .iter()
                .skip(self.loaded.edges)
                .filter(|edge| edge.to == *id && edge.relation == Relation::Reopens)
                .map(|edge| edge.from)
                .collect::<Vec<_>>();
            let recorded = causes
                .iter()
                .map(|name| self.symbols[*name])
                .collect::<Vec<_>>();
            if edges != recorded || (*id > self.loaded.last_node && *open != !causes.is_empty()) {
                return Err(fail("reopening history differs from the graph"));
            }
        }
        Ok(())
    }

    /// Ignore conditions (their historical values are unavailable), but require
    /// that this event can reach the named effect through source procedures.
    fn event_can_change_decision(&self, event: &str, decision: &str, change: &str) -> bool {
        let mut pending = self
            .rules
            .iter()
            .filter(|rule| rule.event == event)
            .map(|rule| &rule.effect)
            .collect::<Vec<_>>();
        let mut visited = HashSet::new();
        while let Some(effect) = pending.pop() {
            match effect {
                Effect::Commit { action, .. } if change == "committed" && action == decision => {
                    return true
                }
                Effect::Reopen { action, .. } if change == "reopened" && action == decision => {
                    return true
                }
                Effect::Call { name, .. } if visited.insert(name) => {
                    if let Some(procedure) = self.procedures.get(name) {
                        pending.extend(procedure.body.iter().map(|step| &step.effect));
                    }
                }
                _ => {}
            }
        }
        false
    }
}
