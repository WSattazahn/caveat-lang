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
/// The largest sequence restore accepts: 2^53 - 1, the largest integer a JSON
/// host reads exactly. The sequence counts accepted events, and at ten
/// million a second 2^53 of them take over 28 years, so a save past it was
/// edited. Nothing else in a save bounds the sequence from above, since an
/// event can leave no record. A session restored at or near the bound numbers
/// its events past it like any others, up to u64::MAX, and its saves from
/// then on are refused.
const MAX_SAVED_SEQUENCE: u64 = (1 << 53) - 1;

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
    /// Complete first-observation order once neutral reveal is used. Absence
    /// keeps the legacy edge-derived representation; an explicit empty list is invalid.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_observations"
    )]
    pub observations: Option<Vec<String>>,
    /// Texts received for `id` parameters, in handle order. Saves made before
    /// spec/caveat-identifiers-0.1.md have none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identifiers: Vec<String>,
    /// Withdrawn observations. Saves made before
    /// spec/caveat-withdrawal-0.1.md have none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub withdrawals: Vec<Withdrawal>,
    /// Each permitted commitment's frozen permission record. Saves made
    /// before spec/caveat-permission-0.1.md have none.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub commitment_permissions: BTreeMap<String, PermissionRecord>,
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

fn deserialize_observations<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error> {
    Vec::<String>::deserialize(deserializer).map(Some)
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

/// The kinds of node a relation an event adds connects, from and to, as
/// `apply_effect` and `apply_qualification` add them: a reading or revealed
/// evidence supports or opposes a claim; a caveat qualifies evidence, late,
/// by a withdrawal, inherited by a reading or carried by a renewal; a
/// commitment retains caveats and relies on evidence; evidence reopens a
/// commitment. No event adds `in_context`.
fn created_relation_ends(relation: Relation) -> Option<(&'static str, &'static str)> {
    match relation {
        Relation::Supports | Relation::Opposes => Some(("evidence", "a claim")),
        Relation::Qualifies => Some(("a caveat", "evidence")),
        Relation::Retains => Some(("a commitment", "a caveat")),
        Relation::ReliesOn => Some(("a commitment", "evidence")),
        Relation::Reopens => Some(("evidence", "a commitment")),
        Relation::InContext => None,
    }
}

/// A node's kind, as `created_relation_ends` names it.
fn node_kind(node: Option<&NodeKind>) -> &'static str {
    match node {
        Some(NodeKind::Claim { .. }) => "a claim",
        Some(NodeKind::Evidence { .. }) => "evidence",
        Some(NodeKind::Caveat { .. }) => "a caveat",
        Some(NodeKind::Commitment { .. }) => "a commitment",
        Some(NodeKind::Context { .. }) => "a context",
        Some(NodeKind::Observation { .. }) => "an observation",
        None => "no node",
    }
}

/// A commitment retains exactly its basis's caveats and relies on exactly its
/// evidence. Reading `committed(...)` or `reopened(...)` adds what it retains
/// and relies on to that basis, so anything more would change what the next
/// revision is made on, and enough of it would take that past the provenance
/// limits a basis keeps to.
fn check_relations_within_bases(save: &ReactiveSave) -> Result<(), String> {
    for [from, name, to] in &save.graph.relations {
        let basis = save
            .commitment_bases
            .get(from)
            .map(|basis| &basis.provenance);
        let within = match name.as_str() {
            "retains" => basis.is_some_and(|basis| basis.caveats.contains(to)),
            "relies_on" => basis.is_some_and(|basis| basis.evidence.contains(to)),
            _ => continue,
        };
        if !within {
            return Err(format!(
                "relation {from} {name} {to}: {to} is not in {from}'s basis"
            ));
        }
    }
    Ok(())
}

/// Grounds can narrow lineage, but cannot introduce another dependency. Use
/// the saved state or frozen commitment basis, never today's qualifications.
fn check_grounds_within_lineage(
    what: &str,
    grounds: &Provenance,
    lineage: &Provenance,
) -> Result<(), String> {
    for (kind, names, allowed) in [
        ("evidence", &grounds.evidence, &lineage.evidence),
        ("caveat", &grounds.caveats, &lineage.caveats),
    ] {
        if let Some(name) = names.difference(allowed).next() {
            return Err(format!(
                "{what} grounds include {kind} {name} outside its lineage"
            ));
        }
    }
    Ok(())
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
/// (caveat, evidence) pairs, evidence named as the source names it.
type QualificationPairs = HashSet<(String, String)>;

/// An effect's kind, as a save writes it.
fn effect_kind(effect: &EffectReport) -> &'static str {
    match effect {
        EffectReport::Sample { .. } => "sample",
        EffectReport::Reveal { .. } => "reveal",
        EffectReport::Examine { .. } => "examine",
        EffectReport::Commit { .. } => "commit",
        EffectReport::Reopen { .. } => "reopen",
        EffectReport::Qualify { .. } => "qualify",
        EffectReport::Renew { .. } => "renew",
        EffectReport::Withdraw { .. } => "withdraw",
    }
}

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
                .filter(|(_, (cell, loaded))| !Arc::ptr_eq(cell, loaded) && !cell.same(loaded))
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
            observations: (!self.observations.is_empty()).then(|| (*self.observations).clone()),
            identifiers: self.identifiers.texts().to_vec(),
            withdrawals: (*self.withdrawals).clone(),
            commitment_permissions: (*self.commitment_permissions).clone(),
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
        if save.sequence > MAX_SAVED_SEQUENCE {
            return Err(format!("sequence must be at most {MAX_SAVED_SEQUENCE}"));
        }
        self.sequence = save.sequence;
        self.last_event = save.last_event.clone();
        self.elapsed = save.elapsed;
        self.identifiers = Arc::new(Identifiers::restore(
            self.identifiers.limit(),
            save.identifiers.clone(),
        )?);
        let observed = self.restore_graph(save)?;
        self.restore_withdrawals(save, &observed)?;
        self.restore_states(&save.states, &observed)?;
        self.restore_records(save, &observed)?;
        self.restore_permissions(save)?;
        self.evaluate_bindings(None)
            .map_err(|error| error.to_string())
    }

    /// Withdrawals must agree with the program and with the restored graph:
    /// each needs its `withdrawn qualifies E` relation, and each such relation
    /// its withdrawal.
    fn restore_withdrawals(
        &mut self,
        save: &ReactiveSave,
        observed: &HashSet<NodeId>,
    ) -> Result<(), String> {
        if save.withdrawals.is_empty() && !self.withdraws() {
            return Ok(());
        }
        if !self.withdraws() {
            return Err("save holds withdrawals but the program does not withdraw".into());
        }
        let mut withdrawn = BTreeSet::new();
        for withdrawal in &save.withdrawals {
            for name in [&withdrawal.evidence, &withdrawal.because] {
                self.require_kind(name, "evidence")
                    .map_err(|_| format!("a withdrawal names unknown evidence {name}"))?;
            }
            // Its reason was observed when it happened, and `withdrawn(...)`
            // qualifies it again.
            self.require_observed(&withdrawal.because, observed)
                .map_err(|error| format!("the withdrawal of {}: {error}", withdrawal.evidence))?;
            if !withdrawn.insert(withdrawal.evidence.as_str()) {
                return Err(format!("{} is withdrawn twice", withdrawal.evidence));
            }
            if withdrawal.sequence == 0 || withdrawal.sequence > save.sequence {
                return Err(format!(
                    "the withdrawal of {} is out of sequence",
                    withdrawal.evidence
                ));
            }
            if !self.events.contains_key(&withdrawal.event) {
                return Err(format!(
                    "the withdrawal of {} names unknown event {}",
                    withdrawal.evidence, withdrawal.event
                ));
            }
        }
        let caveat = self.symbols[WITHDRAWN];
        let names = self
            .symbols
            .iter()
            .map(|(name, id)| (*id, name.as_str()))
            .collect::<HashMap<_, _>>();
        let qualified = self
            .graph
            .edges
            .iter()
            .filter(|edge| edge.from == caveat && edge.relation == Relation::Qualifies)
            .map(|edge| names[&edge.to])
            .collect::<BTreeSet<_>>();
        if qualified != withdrawn {
            return Err("withdrawals disagree with the graph's withdrawn relations".into());
        }
        for withdrawal in &save.withdrawals {
            // Findings 86, 89: its event reaches a `withdraw` of that
            // evidence for that reason. Guards are not evaluated.
            let report = EffectReport::Withdraw {
                evidence: withdrawal.evidence.clone(),
                because: withdrawal.because.clone(),
            };
            let reached = self.reached_effects(Some(&withdrawal.event));
            if !self.last_event_can_make(&report, &reached, None) {
                return Err(format!(
                    "the withdrawal of {} by {}: no withdraw of that event withdraws {} because {}",
                    withdrawal.evidence, withdrawal.event, withdrawal.evidence, withdrawal.because
                ));
            }
        }
        // Withdrawn evidence was observed when it was withdrawn, and a later
        // `withdraw` of it requires that again.
        for withdrawal in &save.withdrawals {
            self.require_observed(&withdrawal.evidence, observed)
                .map_err(|error| format!("the withdrawal of {}: {error}", withdrawal.evidence))?;
        }
        self.withdrawals = Arc::new(save.withdrawals.clone());
        Ok(())
    }

    /// Permission records must name restored commitments, known evidence,
    /// finite scope values and declared caveats, and agree with the journal's
    /// `permitted_by` in both directions.
    fn restore_permissions(&mut self, save: &ReactiveSave) -> Result<(), String> {
        for (commitment, record) in &save.commitment_permissions {
            if !self.commitment_bases.contains_key(commitment) {
                return Err(format!(
                    "a permission names unknown commitment {commitment}"
                ));
            }
            self.require_kind(&record.grant, "evidence").map_err(|_| {
                format!(
                    "the permission of {commitment} names unknown evidence {}",
                    record.grant
                )
            })?;
            if let Some(scope) = &record.scope {
                if !scope.granted.is_finite() || !scope.required.is_finite() {
                    return Err(format!(
                        "the permission of {commitment} has a scope that is not finite"
                    ));
                }
                // The commitment happened only because the two were equal.
                if scope.granted != scope.required {
                    return Err(format!(
                        "the permission of {commitment} records a scope that was not met"
                    ));
                }
            }
            // The record must describe the decision it belongs to: the grant
            // was observed and is in the commitment's frozen lineage. Whether
            // it is still unwithdrawn, or still matches today's head, is not
            // checked: later changes do not rewrite the record.
            if !self.predicate("observed", &record.grant)? {
                return Err(format!(
                    "the permission of {commitment} names a grant that was never observed, {}",
                    record.grant
                ));
            }
            if !self.commitment_bases[commitment]
                .provenance
                .evidence
                .contains(&record.grant)
            {
                return Err(format!(
                    "the permission of {commitment} names {}, which is not in its lineage",
                    record.grant
                ));
            }
            for caveat in &record.caveats {
                self.require_kind(caveat, "caveat")?;
            }
        }
        for entry in &save.decision_journal {
            let recorded = save
                .commitment_permissions
                .get(&entry.commitment)
                .map(|record| &record.grant);
            let expected = if entry.change == "committed" {
                recorded
            } else {
                None
            };
            if entry.permitted_by.as_ref() != expected {
                return Err(format!(
                    "the journal's permission for {} disagrees with its record",
                    entry.commitment
                ));
            }
        }
        self.commitment_permissions = Arc::new(save.commitment_permissions.clone());
        Ok(())
    }

    /// The (caveat, evidence) pairs some mechanism of the loaded source can
    /// relate by `qualifies`, and those a `qualify ... after` can schedule.
    /// Evidence is named as the source names it: a reading stream for its
    /// readings, renewable evidence for its occurrences. The mechanisms are a
    /// declared qualification, `qualify` in a rule or a procedure it reaches
    /// (specialized procedures included), `withdraw`'s generated `withdrawn`,
    /// a reading inheriting its template's caveats, and a renewal carrying its
    /// evidence's declared caveats. Guards are not evaluated: a pair is
    /// possible, not proof that an event made it.
    fn qualification_sources(&self) -> (QualificationPairs, QualificationPairs) {
        let names: HashMap<NodeId, &str> = self
            .symbols
            .iter()
            .map(|(name, id)| (*id, name.as_str()))
            .collect();
        let mut possible = HashSet::new();
        let mut scheduled = HashSet::new();
        for edge in self.graph.edges.iter().take(self.loaded.edges) {
            if edge.relation == Relation::Qualifies {
                if let (Some(caveat), Some(evidence)) = (names.get(&edge.from), names.get(&edge.to))
                {
                    possible.insert((caveat.to_string(), evidence.to_string()));
                }
            }
        }
        for effect in self.reached_effects(None) {
            match effect {
                Effect::Qualify {
                    evidence,
                    caveat,
                    after,
                } => {
                    let pair = (caveat.clone(), evidence.clone());
                    if after.is_some() {
                        scheduled.insert(pair.clone());
                    }
                    possible.insert(pair);
                }
                Effect::Withdraw {
                    target: EvidenceSelector::Named(evidence) | EvidenceSelector::Latest(evidence),
                    ..
                } => {
                    possible.insert((WITHDRAWN.to_string(), evidence.clone()));
                }
                _ => {}
            }
        }
        // A reading inherits whatever qualifies its template when it is taken.
        for (stream, readings) in self.reading_streams.iter() {
            let inherited = possible
                .iter()
                .filter(|(_, evidence)| *evidence == readings.template)
                .map(|(caveat, _)| (caveat.clone(), stream.clone()))
                .collect::<Vec<_>>();
            possible.extend(inherited);
        }
        (possible, scheduled)
    }

    /// The least cost at which some rule, or a procedure a rule reaches,
    /// examines each caveat. Guards are not evaluated: a caveat listed here
    /// could have been examined, not shown to have been.
    fn examination_costs(&self) -> HashMap<String, u64> {
        let mut costs = HashMap::new();
        for effect in self.reached_effects(None) {
            if let Effect::Examine { caveat, cost } = effect {
                costs
                    .entry(caveat.clone())
                    .and_modify(|least: &mut u64| *least = (*least).min(*cost))
                    .or_insert(*cost);
            }
        }
        costs
    }

    /// The name the source gives `evidence`: its stream or renewable evidence
    /// for an occurrence, and itself otherwise.
    fn source_evidence<'a>(&self, evidence: &'a str) -> &'a str {
        match occurrence_parts(evidence) {
            Some((base, _))
                if self.reading_streams.contains_key(base) || self.renewals.contains_key(base) =>
            {
                base
            }
            _ => evidence,
        }
    }

    /// The graph as the save leaves it, and the evidence it observes.
    fn restore_graph(&mut self, save: &ReactiveSave) -> Result<HashSet<NodeId>, String> {
        let saved = &save.graph;
        let committable = self.committable_actions();
        let (possible, _) = self.qualification_sources();
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
        for [from, name, to] in &saved.relations {
            let lookup = |name: &str| {
                self.symbols
                    .get(name)
                    .copied()
                    .ok_or_else(|| format!("relation names unknown {name}"))
            };
            let (from_id, to_id) = (lookup(from)?, lookup(to)?);
            let relation = relation_from_name(name)?;
            // Later events read each end back as the kind an event wrote: a
            // retained caveat, evidence relied on, a reopening's cause. An
            // end of another kind made the next event that read it fatal.
            let (from_kind, to_kind) = created_relation_ends(relation)
                .ok_or_else(|| format!("relation {from} {name} {to} is not one an event adds"))?;
            for (end, id, kind) in [(from, from_id, from_kind), (to, to_id, to_kind)] {
                let actual = node_kind(self.graph.nodes.get(&id));
                if actual != kind {
                    return Err(format!(
                        "relation {from} {name} {to}: {end} is {actual}, not {kind}"
                    ));
                }
            }
            // F95, F260: a qualification no mechanism of the source can make.
            if relation == Relation::Qualifies
                && !possible.contains(&(from.clone(), self.source_evidence(to).to_string()))
            {
                return Err(format!(
                    "relation {from} {name} {to}: no rule, declaration, reading, renewal or \
                     withdrawal of this program qualifies {} with {from}",
                    self.source_evidence(to)
                ));
            }
            Arc::make_mut(&mut self.graph).relate(from_id, relation, to_id);
        }
        // Evidence a commitment relies on, or that reopens one, was observed
        // when the event added the relation, and stays observed: the next
        // commitment and qualification that read it require it. Checked
        // against the whole graph, which holds the observation somewhere.
        self.restore_observations(save)?;
        let observed = self.observed_evidence();
        for [from, name, to] in &saved.relations {
            let evidence = match name.as_str() {
                "relies_on" => to,
                "reopens" => from,
                _ => continue,
            };
            self.require_observed(evidence, &observed)
                .map_err(|error| format!("relation {from} {name} {to}: {error}"))?;
        }
        // Findings 123, 188-191: attention no examination of the source can
        // leave. A reactive event only ever leaves a caveat examined, by an
        // `examine` the budget pays for.
        let costs = self.examination_costs();
        for (name, attention) in &saved.attention {
            self.require_kind(name, "caveat")?;
            let state = attention_from_name(attention)?;
            match state {
                Attention::Unexamined => {}
                Attention::Deferred | Attention::Examining => {
                    return Err(format!(
                        "caveat {name} is {attention}, which no reactive event leaves"
                    ));
                }
                Attention::Examined => {
                    if self.resources.is_none() {
                        return Err(format!(
                            "caveat {name} is examined, but this program has no attention budget"
                        ));
                    }
                    if !costs.contains_key(name) {
                        return Err(format!(
                            "caveat {name} is examined, but no rule or procedure of this \
                             program examines it"
                        ));
                    }
                }
            }
            let id = self.symbols[name];
            Arc::make_mut(&mut self.graph).set_attention(id, state);
        }
        for name in &saved.open {
            let id = self.symbols.get(name).copied();
            match id.and_then(|id| Arc::make_mut(&mut self.graph).nodes.get_mut(&id)) {
                Some(NodeKind::Commitment { open, .. }) => *open = true,
                _ => return Err(format!("{name} is not a commitment")),
            }
        }
        Ok(observed)
    }

    /// Source-reachable neutral effects, including symbol-instantiated procedures.
    fn neutral_targets(&self) -> HashSet<&str> {
        let mut pending = self
            .rules
            .iter()
            .map(|rule| &rule.effect)
            .collect::<Vec<_>>();
        let mut visited = HashSet::new();
        let mut targets = HashSet::new();
        while let Some(effect) = pending.pop() {
            match effect {
                Effect::Observe { evidence } => {
                    targets.insert(evidence.as_str());
                }
                Effect::Call { name, .. } if visited.insert(name) => {
                    if let Some(procedure) = self.procedures.get(name) {
                        pending.extend(procedure.body.iter().map(|step| &step.effect));
                    }
                }
                _ => {}
            }
        }
        targets
    }

    fn restore_observations(&mut self, save: &ReactiveSave) -> Result<(), String> {
        let Some(observations) = &save.observations else {
            return Ok(());
        };
        if observations.is_empty() {
            return Err("observations must be omitted or nonempty".into());
        }
        let targets = self.neutral_targets();
        if targets.is_empty() || save.sequence == 0 {
            return Err(
                "observations require a source-reachable neutral reveal in an accepted event"
                    .into(),
            );
        }
        let neutral_target = |name: &str| {
            targets.contains(name)
                || occurrence_parts(name).is_some_and(|(base, _)| {
                    self.renewals.contains_key(base) && targets.contains(base)
                })
        };
        let mut seen = HashSet::new();
        for name in observations {
            self.require_kind(name, "evidence")?;
            if !seen.insert(name.as_str()) {
                return Err(format!("observations repeats {name}"));
            }
            if !self.predicate("observed", name)? && !neutral_target(name) {
                return Err(format!(
                    "observations names {name}, which no reachable neutral reveal observes"
                ));
            }
        }
        // Source-declared observations existed before any event. Later first
        // stances need not have ledger order: a neutral reveal may precede them.
        let mut loaded = Vec::new();
        for edge in &self.graph.edges[..self.loaded.edges] {
            if matches!(edge.relation, Relation::Supports | Relation::Opposes)
                && matches!(
                    self.graph.nodes.get(&edge.from),
                    Some(NodeKind::Evidence { .. })
                )
            {
                let name = self
                    .symbols
                    .iter()
                    .find_map(|(name, id)| (*id == edge.from).then_some(name))
                    .unwrap();
                if !loaded.contains(name) {
                    loaded.push(name.clone());
                }
            }
        }
        if !observations.starts_with(&loaded) {
            return Err("observations changes source-declared observation order".into());
        }
        for id in self.observed_evidence() {
            let name = self
                .symbols
                .iter()
                .find_map(|(name, candidate)| (*candidate == id).then_some(name))
                .unwrap();
            if !seen.contains(name.as_str()) {
                return Err(format!("observations omits stance-bearing evidence {name}"));
            }
        }
        let positions = observations
            .iter()
            .enumerate()
            .map(|(index, name)| (name.as_str(), index))
            .collect::<HashMap<_, _>>();
        let mut readings = save
            .reading_streams
            .values()
            .flat_map(|stream| &stream.occurrences)
            .collect::<Vec<_>>();
        readings.sort_by_key(|reading| {
            positions
                .get(reading.id.as_str())
                .copied()
                .unwrap_or(usize::MAX)
        });
        let mut last_reading = None;
        for reading in readings {
            let rank = positions
                .get(reading.id.as_str())
                .ok_or_else(|| format!("observations omits reading {}", reading.id))?;
            let node = self
                .symbols
                .get(&reading.id)
                .ok_or_else(|| format!("observations names unknown reading {}", reading.id))?;
            if last_reading.is_some_and(|(sequence, id)| reading.sequence < sequence || *node <= id)
            {
                return Err("observations changes reading chronology".into());
            }
            last_reading = Some((reading.sequence, *node));
            for evidence in &reading.provenance.evidence {
                if positions
                    .get(evidence.as_str())
                    .is_none_or(|position| position > rank)
                {
                    return Err(format!(
                        "reading {} cites evidence not yet observed: {evidence}",
                        reading.id
                    ));
                }
            }
        }
        self.observations = Arc::new(observations.clone());
        Ok(())
    }

    /// Every action a rule or procedure can commit.
    /// Every name a `committed(...)` or `reopened(...)` guard of a skipped
    /// effect can be kept under: an action a rule or procedure commits or
    /// reopens, a decision series, and each restored commitment.
    fn decision_names(&self) -> HashSet<String> {
        let mut names = self.committable_actions();
        for effect in self.reached_effects(None) {
            if let Effect::Reopen { action, .. } = effect {
                names.insert(action.clone());
            }
        }
        names.extend(self.decision_series.keys().cloned());
        names.extend(
            self.symbols
                .iter()
                .filter(|(_, id)| {
                    matches!(self.graph.nodes.get(id), Some(NodeKind::Commitment { .. }))
                })
                .map(|(name, _)| name.clone()),
        );
        names
    }

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

    /// Every evidence node observed explicitly or by a stance-bearing edge.
    /// Collected once, since a save may
    /// cite the same evidence in many relations and records.
    fn observed_evidence(&self) -> HashSet<NodeId> {
        self.graph
            .edges
            .iter()
            .filter(|edge| {
                matches!(edge.relation, Relation::Supports | Relation::Opposes)
                    && matches!(
                        self.graph.nodes.get(&edge.from),
                        Some(NodeKind::Evidence { .. })
                    )
            })
            .map(|edge| edge.from)
            .chain(self.observations.iter().map(|name| self.symbols[name]))
            .collect()
    }

    /// Evidence enters a lineage or a record only once observed, and stays
    /// observed.
    fn require_observed(&self, name: &str, observed: &HashSet<NodeId>) -> Result<(), String> {
        if self
            .symbols
            .get(name)
            .is_some_and(|id| observed.contains(id))
        {
            Ok(())
        } else {
            Err(format!("{name} is not observed"))
        }
    }

    /// A provenance whose names are all declared or created evidence and
    /// caveats, and whose evidence is observed: a commitment made on it, or a
    /// qualification of it, requires that.
    fn check_provenance(
        &self,
        what: &str,
        provenance: &Provenance,
        observed: &HashSet<NodeId>,
    ) -> Result<(), String> {
        provenance
            .validate()
            .map_err(|error| format!("{what}: {error}"))?;
        for name in &provenance.evidence {
            self.require_kind(name, "evidence")
                .and_then(|()| self.require_observed(name, observed))
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

    fn restore_states(
        &mut self,
        saved: &BTreeMap<String, SavedState>,
        observed: &HashSet<NodeId>,
    ) -> Result<(), String> {
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
            self.check_provenance(&format!("state {name}"), lineage, observed)?;
            self.check_provenance(&format!("state {name} grounds"), grounds, observed)?;
            check_grounds_within_lineage(&format!("state {name}"), grounds, lineage)?;
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

    fn restore_records(
        &mut self,
        save: &ReactiveSave,
        observed: &HashSet<NodeId>,
    ) -> Result<(), String> {
        for (name, basis) in &save.commitment_bases {
            self.require_commitment(name)?;
            if basis.value.is_some_and(|value| !value.is_finite()) {
                return Err(format!("commitment {name} basis is not a finite number"));
            }
            self.check_provenance(&format!("commitment {name}"), &basis.provenance, observed)?;
        }
        for (name, grounds) in &save.commitment_grounds {
            self.require_commitment(name)?;
            self.check_provenance(&format!("commitment {name} grounds"), &grounds.0, observed)?;
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
            for (index, occurrence) in stream.occurrences.iter().enumerate() {
                if occurrence.id != format!("{name}@{}", index + 1) {
                    return Err(format!(
                        "reading stream {name} occurrence {} is out of order",
                        occurrence.id
                    ));
                }
                self.require_kind(&occurrence.id, "evidence")?;
                if !occurrence.value.is_finite() {
                    return Err(format!("reading {} is not a finite number", occurrence.id));
                }
                // Taken by an accepted event, so within the save's sequence.
                if occurrence.sequence == 0 || occurrence.sequence > save.sequence {
                    return Err(format!("reading {} is out of sequence", occurrence.id));
                }
                self.check_provenance(
                    &format!("reading {}", occurrence.id),
                    &occurrence.provenance,
                    observed,
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
                observed,
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
                observed,
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
        // And the other way: every reading and renewal occurrence the graph
        // holds is in its stream's occurrences or its evidence's renewals at
        // its ordinal, so the limits above bound it too. Unlisted, the list
        // would stop short of it, and the next sample or renew would generate
        // a name the graph already holds.
        for node in &save.graph.nodes {
            let SavedNode::Occurrence { name } = node else {
                continue;
            };
            let Some((base, ordinal)) = occurrence_parts(name) else {
                continue;
            };
            if let Some(stream) = save.reading_streams.get(base) {
                if stream
                    .occurrences
                    .get(ordinal - 1)
                    .map(|occurrence| &occurrence.id)
                    != Some(name)
                {
                    return Err(format!(
                        "reading stream {base} occurrence {name} is not in its occurrences"
                    ));
                }
            } else if self.renewals.contains_key(base)
                && save
                    .renewals
                    .get(base)
                    .and_then(|occurrences| occurrences.get(ordinal - 1))
                    != Some(name)
            {
                return Err(format!(
                    "renewable {base} occurrence {name} is not in its renewals"
                ));
            }
        }
        if save.scheduled_qualifications.len() > MAX_SCHEDULED_QUALIFICATIONS {
            return Err("too many scheduled qualifications".into());
        }
        let (_, schedulable) = self.qualification_sources();
        for scheduled in &save.scheduled_qualifications {
            self.require_kind(&scheduled.evidence, "evidence")?;
            self.require_observed(&scheduled.evidence, observed)?;
            self.require_kind(&scheduled.caveat, "caveat")?;
            let evidence = self.source_evidence(&scheduled.evidence);
            if !schedulable.contains(&(scheduled.caveat.clone(), evidence.to_string())) {
                return Err(format!(
                    "scheduled qualification of {} with {}: no qualify ... after of this \
                     program schedules it",
                    scheduled.evidence, scheduled.caveat
                ));
            }
            if !(scheduled.after.is_finite()
                && scheduled.after >= 0.0
                && scheduled.scheduled_at.is_finite())
            {
                return Err("a scheduled qualification's times are not valid".into());
            }
            self.check_provenance("scheduled qualification", &scheduled.guard, observed)?;
        }
        // Findings 86, 89, 96, 118-120: each table is keyed by what made its
        // record. An observation is of observed evidence, or of an occurrence
        // a renewal made; an examination of an examined caveat; a reopening
        // of a commitment a `reopens` relation names.
        let reopened = self
            .graph
            .edges
            .iter()
            .filter(|edge| edge.relation == Relation::Reopens)
            .map(|edge| edge.to)
            .collect::<HashSet<_>>();
        for (what, records) in [
            ("observation", &save.observation_qualifications),
            ("examination", &save.examination_qualifications),
            ("reopening", &save.reopening_qualifications),
        ] {
            for (name, provenance) in records {
                let id = self.symbols.get(name);
                let fits = match what {
                    "observation" => {
                        id.is_some_and(|id| observed.contains(id))
                            || occurrence_parts(name)
                                .is_some_and(|(base, _)| self.renewals.contains_key(base))
                                && self.require_kind(name, "evidence").is_ok()
                    }
                    "examination" => matches!(
                        id.and_then(|id| self.graph.nodes.get(id)),
                        Some(NodeKind::Caveat {
                            attention: Attention::Examined,
                            ..
                        })
                    ),
                    _ => id.is_some_and(|id| reopened.contains(id)),
                };
                if !fits {
                    let kind = match what {
                        "observation" => "observed evidence or a renewed occurrence",
                        "examination" => "an examined caveat",
                        _ => "a commitment a reopens relation names",
                    };
                    return Err(format!("{what} of {name}: {name} is not {kind}"));
                }
                self.check_provenance(&format!("{what} of {name}"), &provenance.0, observed)?;
            }
        }
        let decisions = self.decision_names();
        for (kind, targets) in &save.predicate_qualifications {
            // A skipped withdrawal keeps its guard under "withdrawn" with no
            // withdrawal record, since none happened; its target is evidence.
            let withdrawal = kind == WITHDRAWN && self.withdraws();
            if !withdrawal
                && !matches!(
                    kind.as_str(),
                    "observed" | "examined" | "committed" | "reopened"
                )
            {
                return Err(format!("unknown predicate {kind}"));
            }
            for (name, provenance) in targets {
                if withdrawal {
                    self.require_kind(name, "evidence")
                        .map_err(|_| format!("withdrawn({name}) names unknown evidence"))?;
                }
                // The target of the skipped effect the guard was kept for.
                let expected = match kind.as_str() {
                    "observed" if self.require_kind(name, "evidence").is_err() => Some("evidence"),
                    "examined" if self.require_kind(name, "caveat").is_err() => Some("a caveat"),
                    "committed" | "reopened" if !decisions.contains(name.as_str()) => {
                        Some("a decision this program makes")
                    }
                    _ => None,
                };
                if let Some(expected) = expected {
                    return Err(format!("{kind}({name}): {name} is not {expected}"));
                }
                self.check_provenance(&format!("{kind}({name})"), &provenance.0, observed)?;
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
        // Each examined caveat cost at least the least any rule examining it
        // charges, and the budget spent all of it.
        if let Some(ledger) = &self.resources {
            let costs = self.examination_costs();
            let least: u128 = save
                .graph
                .attention
                .iter()
                .filter(|(_, attention)| attention.as_str() == "examined")
                .filter_map(|(name, _)| costs.get(name))
                .map(|cost| u128::from(*cost))
                .sum();
            if least > u128::from(ledger.spent) {
                return Err(format!(
                    "examined caveats cost at least {least} attention to examine, more than the \
                     {} the budget spent",
                    ledger.spent
                ));
            }
        }
        if save.cue_qualifications.len() != save.cues.len() {
            return Err("its cues and their qualifications do not match".into());
        }
        let last_effects = save
            .last_event
            .as_deref()
            .map(|event| self.reached_effects(Some(event)))
            .unwrap_or_default();
        let mut cues = Vec::new();
        for (id, qualification) in save.cues.iter().zip(&save.cue_qualifications) {
            cues.push(
                self.cue_definitions
                    .get(id)
                    .cloned()
                    .ok_or_else(|| format!("unknown cue {id}"))?,
            );
            // Only an `emit` the last event's rules reach shows a cue.
            if !last_effects
                .iter()
                .any(|effect| matches!(effect, Effect::Emit { name } if name == id))
            {
                return Err(format!("cue {id} is not one the last event can emit"));
            }
            self.check_provenance(&format!("cue {id}"), &qualification.0, observed)?;
        }
        for effect in &save.effects {
            let names: Vec<&str> = match effect {
                EffectReport::Sample { id, target, .. } => vec![id, target],
                EffectReport::Reveal {
                    evidence,
                    relation,
                    target,
                } => {
                    self.require_kind(evidence, "evidence")?;
                    self.require_observed(evidence, observed)?;
                    match (relation, target) {
                        (None, None) => {
                            if !self.observations.iter().any(|name| name == evidence) {
                                return Err(
                                    "neutral reveal effect is missing its observation record"
                                        .into(),
                                );
                            }
                            let targets = self.neutral_targets();
                            let base = occurrence_parts(evidence)
                                .map_or(evidence.as_str(), |(base, _)| base);
                            if !targets.contains(base) {
                                return Err(format!(
                                    "neutral reveal effect names unreachable evidence {evidence}"
                                ));
                            }
                        }
                        (Some(relation), Some(target)) => {
                            self.require_kind(target, "claim")?;
                            let stance = match relation.as_str() {
                                "supports" => Relation::Supports,
                                "opposes" => Relation::Opposes,
                                _ => {
                                    return Err(
                                        "reveal effect relation must be supports or opposes".into(),
                                    )
                                }
                            };
                            if !self.graph.edges.iter().any(|edge| {
                                edge.from == self.symbols[evidence]
                                    && edge.to == self.symbols[target]
                                    && edge.relation == stance
                            }) {
                                return Err("reveal effect has no matching graph relation".into());
                            }
                        }
                        _ => {
                            return Err("reveal effect requires relation and target together".into())
                        }
                    }
                    vec![evidence]
                }
                EffectReport::Examine { caveat, .. } => vec![caveat],
                EffectReport::Commit { action, retained } => std::iter::once(action)
                    .chain(retained)
                    .map(String::as_str)
                    .collect(),
                EffectReport::Reopen { action, because } => vec![action, because],
                EffectReport::Qualify { evidence, caveat } => vec![evidence, caveat],
                EffectReport::Renew { occurrence, .. } => vec![occurrence],
                EffectReport::Withdraw { evidence, because } => vec![evidence, because],
            };
            if let Some(name) = names
                .into_iter()
                .find(|name| !self.symbols.contains_key(*name))
            {
                return Err(format!("an effect names unknown {name}"));
            }
            self.check_effect_against_save(effect, save, observed)?;
            if !self.last_event_can_make(effect, &last_effects, save.last_event.as_deref()) {
                return Err(format!(
                    "{} effect is not one the last event can make",
                    effect_kind(effect)
                ));
            }
        }
        self.check_saved_journal(save, observed)?;
        check_relations_within_bases(save)?;
        // Keep the existing journal and relation diagnostics first. A matching
        // journal alone cannot establish that grounds belong to the basis.
        for (name, grounds) in &save.commitment_grounds {
            let basis = save
                .commitment_bases
                .get(name)
                .ok_or_else(|| format!("commitment {name} has no basis"))?;
            check_grounds_within_lineage(
                &format!("commitment {name}"),
                &grounds.0,
                &basis.provenance,
            )?;
        }
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
    fn check_saved_journal(
        &self,
        save: &ReactiveSave,
        observed: &HashSet<NodeId>,
    ) -> Result<(), String> {
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
            self.check_provenance("decision journal", &provenance, observed)?;
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

    /// Every effect a rule of `event` reaches, or with `None` of any event,
    /// directly or through the procedures it calls, as the source compiled
    /// them. Conditions are ignored: their historical values are unavailable,
    /// so this is what the event could have done, not what it did.
    fn reached_effects(&self, event: Option<&str>) -> Vec<&Effect> {
        let mut pending = self
            .rules
            .iter()
            .filter(|rule| event.is_none_or(|event| rule.event == event))
            .map(|rule| &rule.effect)
            .collect::<Vec<_>>();
        let mut visited = HashSet::new();
        let mut reached = Vec::new();
        while let Some(effect) = pending.pop() {
            if let Effect::Call { name, .. } = effect {
                if visited.insert(name) {
                    if let Some(procedure) = self.procedures.get(name) {
                        pending.extend(procedure.body.iter().map(|step| &step.effect));
                    }
                }
            }
            reached.push(effect);
        }
        reached
    }

    /// A saved effect must agree with what the rest of the save restored: its
    /// names have the kinds its place needs, and the relation, record or
    /// spending it reports is there. Every effect leaves those behind, and
    /// nothing later in the same event removes them.
    fn check_effect_against_save(
        &self,
        effect: &EffectReport,
        save: &ReactiveSave,
        observed: &HashSet<NodeId>,
    ) -> Result<(), String> {
        let kind = effect_kind(effect);
        let fail = |message: &str| format!("{kind} effect {message}");
        let named = |result: Result<(), String>| result.map_err(|error| fail(&error));
        let related = |from: &str, relation: Relation, to: &str| {
            let (from, to) = (self.symbols[from], self.symbols[to]);
            self.graph
                .edges
                .iter()
                .any(|edge| edge.from == from && edge.to == to && edge.relation == relation)
        };
        match effect {
            EffectReport::Sample {
                stream,
                id,
                relation,
                target,
                ..
            } => {
                named(self.require_kind(target, "claim"))?;
                let listed = save.reading_streams.get(stream).is_some_and(|readings| {
                    readings
                        .occurrences
                        .iter()
                        .any(|occurrence| occurrence.id == *id)
                });
                if !listed {
                    return Err(fail("names no reading of its stream"));
                }
                named(self.require_observed(id, observed))?;
                let stance = match relation.as_str() {
                    "supports" => Relation::Supports,
                    "opposes" => Relation::Opposes,
                    _ => return Err(fail("relation must be supports or opposes")),
                };
                if !related(id, stance, target) {
                    return Err(fail("has no matching graph relation"));
                }
            }
            // Checked with the other reveals' semantics in restore_records.
            EffectReport::Reveal { .. } => {}
            EffectReport::Examine { caveat, cost } => {
                named(self.require_kind(caveat, "caveat"))?;
                if self
                    .resources
                    .as_ref()
                    .is_none_or(|resources| *cost > resources.spent)
                {
                    return Err(fail("spends more attention than the budget spent"));
                }
            }
            EffectReport::Commit { action, retained } => {
                named(self.require_commitment(action))?;
                let basis = save
                    .commitment_bases
                    .get(action)
                    .ok_or_else(|| fail("names a commitment with no basis"))?;
                for caveat in retained {
                    named(self.require_kind(caveat, "caveat"))?;
                    if !basis.provenance.caveats.contains(caveat) {
                        return Err(fail("retains a caveat outside its basis"));
                    }
                }
            }
            EffectReport::Reopen { action, because } => {
                named(self.require_commitment(action))?;
                named(self.require_observed(because, observed))?;
                if !related(because, Relation::Reopens, action) {
                    return Err(fail("has no matching graph relation"));
                }
            }
            EffectReport::Qualify { evidence, caveat } => {
                named(self.require_kind(evidence, "evidence"))?;
                named(self.require_kind(caveat, "caveat"))?;
                if !related(caveat, Relation::Qualifies, evidence) {
                    return Err(fail("has no matching graph relation"));
                }
            }
            EffectReport::Renew {
                evidence,
                occurrence,
            } => {
                let listed = self.renewals.contains_key(evidence)
                    && save
                        .renewals
                        .get(evidence)
                        .is_some_and(|occurrences| occurrences.contains(occurrence));
                if !listed {
                    return Err(fail("names no occurrence of its renewals"));
                }
            }
            EffectReport::Withdraw { evidence, because } => {
                named(self.require_observed(evidence, observed))?;
                named(self.require_observed(because, observed))?;
                if !self.withdrawals.iter().any(|withdrawal| {
                    withdrawal.evidence == *evidence && withdrawal.because == *because
                }) {
                    return Err(fail("has no matching withdrawal record"));
                }
            }
        }
        Ok(())
    }

    /// Whether a rule effect the last event reaches, `reached`, could report
    /// `effect`: the same kind of effect, on the names its place takes. A
    /// renewable evidence or reading stream the source names reports one of
    /// its occurrences. Two reports come from no rule effect of their own: a
    /// reading's reopening trigger reopens its series, and the clock's event
    /// applies a scheduled `qualify ... after` from any rule.
    fn last_event_can_make(
        &self,
        effect: &EffectReport,
        reached: &[&Effect],
        last_event: Option<&str>,
    ) -> bool {
        let named = |reported: &str, source: &str| {
            reported == source || occurrence_parts(reported).is_some_and(|(base, _)| base == source)
        };
        let selects = |reported: &str, selector: &EvidenceSelector| match selector {
            EvidenceSelector::Named(source) | EvidenceSelector::Latest(source) => {
                named(reported, source)
            }
            EvidenceSelector::Caveated { .. } => true,
        };
        reached.iter().any(|rule| match (effect, rule) {
            (
                EffectReport::Sample {
                    stream,
                    id,
                    relation,
                    target,
                    ..
                },
                Effect::Sample {
                    stream: source,
                    relation: stance,
                    claim,
                    ..
                },
            ) => {
                stream == source
                    && named(id, source)
                    && relation == relation_name(*stance)
                    && target == claim
            }
            (
                EffectReport::Reveal {
                    evidence,
                    relation: None,
                    target: None,
                },
                Effect::Observe { evidence: source },
            ) => named(evidence, source),
            (
                EffectReport::Reveal {
                    evidence,
                    relation: Some(relation),
                    target: Some(target),
                },
                Effect::Reveal {
                    evidence: source,
                    relation: stance,
                    claim,
                },
            ) => named(evidence, source) && relation == relation_name(*stance) && target == claim,
            (
                EffectReport::Examine { caveat, cost },
                Effect::Examine {
                    caveat: source,
                    cost: spent,
                },
            ) => caveat == source && cost == spent,
            (EffectReport::Commit { action, .. }, Effect::Commit { action: source, .. }) => {
                named(action, source)
            }
            (
                EffectReport::Reopen { action, because },
                Effect::Reopen {
                    action: source,
                    because: selector,
                },
            ) => named(action, source) && selects(because, selector),
            (
                EffectReport::Reopen { action, because },
                Effect::Sample {
                    stream,
                    relation,
                    claim,
                    ..
                },
            ) => {
                named(because, stream)
                    && self.reopening_triggers.iter().any(|(series, triggers, _)| {
                        named(action, series)
                            && triggers
                                .iter()
                                .any(|trigger| trigger.matches(stream, *relation, claim))
                    })
            }
            (
                EffectReport::Qualify { evidence, caveat },
                Effect::Qualify {
                    evidence: source,
                    caveat: added,
                    after: None,
                },
            ) => named(evidence, source) && caveat == added,
            (
                EffectReport::Renew {
                    evidence,
                    occurrence,
                },
                Effect::Renew { evidence: source },
            ) => evidence == source && named(occurrence, source),
            (
                EffectReport::Withdraw { evidence, because },
                Effect::Withdraw {
                    target,
                    because: reason,
                },
            ) => selects(evidence, target) && named(because, reason),
            _ => false,
        }) || matches!(effect, EffectReport::Qualify { evidence, caveat }
        if last_event.is_some() && last_event == self.time_event.as_deref()
            && self.reached_effects(None).iter().any(|reached| {
                matches!(reached, Effect::Qualify {
                    evidence: source,
                    caveat: added,
                    after: Some(_),
                } if named(evidence, source) && caveat == added)
            }))
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
                // A declared trigger reopens it when this sample's stream,
                // relation and claim match (spec/caveat-reopening-triggers-0.1.md).
                Effect::Sample {
                    stream,
                    relation,
                    claim,
                    ..
                } if change == "reopened"
                    && self.reopening_triggers.iter().any(|(series, triggers, _)| {
                        series == decision
                            && triggers
                                .iter()
                                .any(|trigger| trigger.matches(stream, *relation, claim))
                    }) =>
                {
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
