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
    pub states: BTreeMap<String, SavedState>,
    #[serde(default)]
    pub graph: SavedGraph,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub commitment_bases: BTreeMap<String, CommitmentBasis>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub commitment_grounds: BTreeMap<String, Provenance>,
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
    pub observation_qualifications: BTreeMap<String, Provenance>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub examination_qualifications: BTreeMap<String, Provenance>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub reopening_qualifications: BTreeMap<String, Provenance>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub predicate_qualifications: BTreeMap<String, BTreeMap<String, Provenance>>,
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
    pub cue_qualifications: Vec<Provenance>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedState {
    pub value: f64,
    #[serde(default, skip_serializing_if = "Provenance::is_empty")]
    pub lineage: Provenance,
    #[serde(default, skip_serializing_if = "Provenance::is_empty")]
    pub grounds: Provenance,
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
                .zip(self.states.cells.iter())
                .map(|(name, cell)| {
                    let state = SavedState {
                        value: cell.value.value,
                        lineage: cell.value.provenance.clone(),
                        grounds: cell.grounds.clone(),
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
            commitment_grounds: (*self.commitment_grounds).clone(),
            reading_streams: (*self.reading_streams).clone(),
            decision_series: (*self.decision_series).clone(),
            renewals: self
                .renewals
                .iter()
                .filter(|(_, renewal)| renewal.occurrences.len() > 1)
                .map(|(name, renewal)| (name.clone(), renewal.occurrences.clone()))
                .collect(),
            scheduled_qualifications: (*self.scheduled).clone(),
            observation_qualifications: (*self.observation_qualifications).clone(),
            examination_qualifications: (*self.examination_qualifications).clone(),
            reopening_qualifications: (*self.reopening_qualifications).clone(),
            predicate_qualifications: (*self.predicate_qualifications).clone(),
            decision_journal: (*self.journal).clone(),
            resources: self.resources.as_ref().map(|ledger| SavedResources {
                remaining: ledger.remaining,
                spent: ledger.spent,
                exhausted: ledger.exhausted,
            }),
            effects: self.effects.clone(),
            cues: self.cues.iter().map(|cue| cue.id().to_string()).collect(),
            cue_qualifications: self.cue_qualifications.clone(),
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
        if !save.elapsed.is_finite() || save.elapsed < 0.0 {
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
        if saved.len() != self.states.len()
            || saved.keys().any(|name| !self.states.contains_key(name))
        {
            return Err("its states are not this program's states".into());
        }
        for (name, state) in saved {
            let range = &self.ranges[name];
            if !state.value.is_finite() {
                return Err(format!("state {name} is not a finite number"));
            }
            check_range(name, state.value, range.min, range.max)?;
            self.check_provenance(&format!("state {name}"), &state.lineage)?;
            self.check_provenance(&format!("state {name} grounds"), &state.grounds)?;
            let slot = self.states.slot(name).expect("checked above");
            self.states.set(
                slot,
                StateCell {
                    value: Tracked::new(state.value, state.lineage.clone())?,
                    grounds: state.grounds.clone(),
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
            self.check_provenance(&format!("commitment {name} grounds"), grounds)?;
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
                self.check_provenance(&format!("{what} of {name}"), provenance)?;
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
                self.check_provenance(&format!("{kind}({name})"), provenance)?;
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
            self.check_provenance(&format!("cue {id}"), qualification)?;
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
        self.commitment_bases = Arc::new(save.commitment_bases.clone());
        self.commitment_grounds = Arc::new(save.commitment_grounds.clone());
        self.reading_streams = Arc::new(save.reading_streams.clone());
        self.decision_series = Arc::new(save.decision_series.clone());
        let renewals = Arc::make_mut(&mut self.renewals);
        for (name, occurrences) in &save.renewals {
            renewals.get_mut(name).expect("checked above").occurrences = occurrences.clone();
        }
        self.scheduled = Arc::new(save.scheduled_qualifications.clone());
        self.observation_qualifications = Arc::new(save.observation_qualifications.clone());
        self.examination_qualifications = Arc::new(save.examination_qualifications.clone());
        self.reopening_qualifications = Arc::new(save.reopening_qualifications.clone());
        self.predicate_qualifications = Arc::new(save.predicate_qualifications.clone());
        self.journal = Arc::new(save.decision_journal.clone());
        self.effects = save.effects.clone();
        self.cues = cues;
        self.cue_qualifications = save.cue_qualifications.clone();
        Ok(())
    }
}
