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
use std::borrow::Cow;

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
#[serde(deny_unknown_fields, expecting = "a save object")]
pub struct ReactiveSave {
    pub schema: String,
    /// The program this save belongs to; restoring checks it.
    pub source_id: String,
    /// SHA-256 of the source, in lowercase hex. Saves made before
    /// spec/caveat-save-0.1.md "Source digest" have none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_sha256: Option<String>,
    /// The last sequence whose decisions have no recorded source digest.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub source_unrecorded_through: u64,
    pub sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event: Option<String>,
    /// The agent the host said sent the last event. Saves made before
    /// spec/caveat-agents-0.1.md, or of sessions no agent sent to, have none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_agent: Option<String>,
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
    /// Each commitment's grounds evidence in first-observed order, where that
    /// differs from name order. Absent in older saves, whose order comes from
    /// the retained journal entry. See spec/caveat-view-0.2.md.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub evidence_order: BTreeMap<String, Vec<String>>,
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
    /// Taken-back qualifications. Saves made before
    /// spec/caveat-take-back-0.1.md have none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub taken_back: Vec<TakeBack>,
    /// Each permitted commitment's frozen permission record. Saves made
    /// before spec/caveat-permission-0.1.md have none.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub commitment_permissions: BTreeMap<String, PermissionRecord>,
    /// Each retained caveat's reason, by commitment. Saves made before
    /// spec/caveat-retention-reasons-0.1.md have none.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub retention_reasons: BTreeMap<String, BTreeMap<String, String>>,
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
    /// Retired records of windowed histories, with the sequence of the event
    /// that retired each. See spec/caveat-windows-0.1.md.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub retired: BTreeMap<String, u64>,
    /// How many of the journal's oldest entries have departed. See
    /// spec/caveat-departure-0.1.md.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub journal_departed: u64,
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

pub(super) fn is_zero(count: &u64) -> bool {
    *count == 0
}

fn deserialize_observations<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error> {
    Vec::<String>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    expecting = "a state entry object such as {\"value\": 3}"
)]
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

fn markers<S: serde::Serializer>(
    markers: &&BTreeMap<String, crate::reactive_expr::Marker>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    crate::reactive_expr::departure_markers::serialize(markers, serializer)
}

impl Serialize for Compact {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Written<'a> {
            #[serde(skip_serializing_if = "NameSet::is_empty")]
            evidence: &'a NameSet,
            #[serde(skip_serializing_if = "NameSet::is_empty")]
            caveats: &'a NameSet,
            #[serde(skip_serializing_if = "BTreeMap::is_empty", serialize_with = "markers")]
            departed: &'a BTreeMap<String, crate::reactive_expr::Marker>,
            #[serde(skip_serializing_if = "NameSet::is_empty")]
            inherited: &'a NameSet,
            #[serde(skip_serializing_if = "NameSet::is_empty")]
            kept: &'a NameSet,
        }
        Written {
            evidence: &self.0.evidence,
            caveats: &self.0.caveats,
            departed: &self.0.departed,
            inherited: &self.0.inherited,
            kept: &self.0.kept,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Compact {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            #[serde(default)]
            evidence: BTreeSet<String>,
            #[serde(default)]
            caveats: BTreeSet<String>,
            #[serde(default, with = "crate::reactive_expr::departure_markers")]
            departed: BTreeMap<String, crate::reactive_expr::Marker>,
            #[serde(default)]
            inherited: BTreeSet<String>,
            #[serde(default)]
            kept: BTreeSet<String>,
        }
        let saved = Saved::deserialize(deserializer)?;
        Ok(Self(Provenance {
            evidence: saved.evidence.into(),
            caveats: saved.caveats.into(),
            departed: saved.departed,
            inherited: saved.inherited.into(),
            kept: saved.kept.into(),
        }))
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

/// What events did to the graph since the program loaded, and the attention
/// and open commitments it holds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, expecting = "a graph object")]
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
#[serde(deny_unknown_fields, expecting = "a resources object")]
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

/// A decision journal entry's witnesses as names: its `because` evidence and
/// its `caveats`, each sorted and without repeats. They are not a lineage. A
/// reopening that ends a hold, or breaks through one, cites every deferred
/// cause, and their caveats together can pass the limits one lineage keeps
/// to, so no limit applies here.
fn journal_witnesses(entry: &JournalEntry) -> Provenance {
    Provenance {
        evidence: entry.because.iter().cloned().collect(),
        caveats: entry.caveats.iter().cloned().collect(),
        ..Provenance::default()
    }
}

/// What `check_saved_journal` reads of the graph and the program for each
/// entry, read out of them once for the whole journal rather than once for
/// each entry (D039). The check holds the session shared, so neither changes
/// while it runs, and each answer here is the one the session's own read
/// gives.
struct JournalReads<'a> {
    /// The caveats with a `qualifies` relation to each node, in graph order.
    qualified_by: HashMap<NodeId, Vec<NodeId>>,
    /// The claims each node supports or opposes, which `qualify_roots`
    /// searches with evidence.
    bears_on: HashMap<NodeId, Vec<NodeId>>,
    /// Each node with a `supports` or `opposes` relation, and the observation
    /// ledger: what `predicate("observed")` reads.
    stance: HashSet<NodeId>,
    ledger: HashSet<&'a str>,
    ranks: ObservationRanks<'a>,
    /// The `reopens` relations events added, by commitment, in graph order.
    reopens: HashMap<NodeId, Vec<NodeId>>,
    /// The caveats reaching each root an entry has needed.
    reaching: Reaching,
    /// `event_can_change_decision`, by its arguments.
    changes: HashMap<(&'a str, &'a str, &'a str), bool>,
    /// The first revision of each name in each series.
    revisions: HashMap<(&'a str, &'a str), &'a DecisionRevision>,
}

/// The caveats reaching each root, kept for every entry that needs them.
/// Caveats can qualify caveats, so what one root reaches can be most of the
/// graph's caveats, and many roots can reach the same ones: what is kept is
/// held to the size of the graph, and a root past that is walked where it is
/// needed.
struct Reaching {
    by_root: HashMap<NodeId, HashSet<NodeId>>,
    /// How many more caveats `by_root` may hold.
    room: usize,
}

/// The caveats reaching `root` through `qualifies` relations, a taken-back
/// relation still counted, as `caveats_reaching` counts it with
/// `honor_take_back` false: every caveat an entry made on it may cite.
fn caveats_reaching_root<'m>(
    qualified_by: &HashMap<NodeId, Vec<NodeId>>,
    reaching: &'m mut Reaching,
    root: NodeId,
) -> Cow<'m, HashSet<NodeId>> {
    if reaching.by_root.contains_key(&root) {
        return Cow::Borrowed(&reaching.by_root[&root]);
    }
    let mut frontier = vec![root];
    let mut visited = HashSet::default();
    let mut caveats = HashSet::default();
    while let Some(id) = frontier.pop() {
        if visited.insert(id) {
            for &caveat in qualified_by.get(&id).into_iter().flatten() {
                caveats.insert(caveat);
                frontier.push(caveat);
            }
        }
    }
    match reaching.room.checked_sub(caveats.len()) {
        Some(room) => {
            reaching.room = room;
            Cow::Borrowed(reaching.by_root.entry(root).or_insert(caveats))
        }
        None => Cow::Owned(caveats),
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
    // A grounds marker lies within the lineage's for its history
    // (spec/caveat-lineage-compaction-0.1.md, "Meaning").
    for (history, marker) in &grounds.departed {
        if !lineage
            .departed
            .get(history)
            .is_some_and(|outer| outer.contains(marker))
        {
            return Err(format!(
                "{what} grounds hold departed {history} records outside its lineage"
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

/// What the program's declarations leave as it loads, before any event: the
/// evidence they relate to a claim, with the stances they give it, and the
/// attention they leave caveats in. See `declared_memberships`.
struct Declared {
    /// Each piece of evidence a declaration relates to a claim, which is
    /// observed from the start, with every (relation, claim) declared for it.
    stances: HashMap<String, Vec<(Relation, String)>>,
    /// Each caveat whose attention, once every declaration has run, is
    /// `examined`, `examining` or `deferred`, with that attention.
    attention: HashMap<String, Attention>,
}

impl Declared {
    /// Whether a declaration relates `evidence` to a claim.
    fn observes(&self, evidence: &str) -> bool {
        self.stances.contains_key(evidence)
    }

    /// Whether a declaration relates `evidence` to `claim` by `relation`.
    fn gives(&self, evidence: &str, relation: Relation, claim: &str) -> bool {
        self.stances.get(evidence).is_some_and(|stances| {
            stances
                .iter()
                .any(|(given, to)| *given == relation && to == claim)
        })
    }
}

/// D052: the names the `reveal`, `sample` and `renew` effects rules reach
/// leave observation records on, read from the source. See
/// `observation_sources`.
#[derive(Default)]
struct ObservationSources<'a> {
    /// Evidence a `reveal` names, with a stance or neutral. Its record goes
    /// on the evidence's current occurrence.
    revealed: HashSet<&'a str>,
    /// The evidence of those that a `reveal` can leave a record on under its
    /// own name: one with a stance no declaration gives it, or a neutral one
    /// of evidence no declaration relates to a claim.
    own: HashSet<&'a str>,
    /// Streams a `sample` reads. Its record goes on the reading it takes.
    sampled: HashSet<&'a str>,
    /// Evidence a `renew` names, and whether a condition or call argument on
    /// the way to one of them reads a state, the graph or a history, the
    /// only reads that put anything in its guard. A renewal leaves a record,
    /// on the occurrence it makes, only when its guard holds something.
    renewed: HashMap<&'a str, bool>,
}

/// The evidence a save may name, read once for every record restore checks.
/// See `restore_graph`.
struct Observed {
    /// Evidence observed explicitly or by a stance-bearing relation. A record
    /// that retires keeps both, so it stays here: it was observed when it was
    /// cited. What an event names itself must be here: a withdrawal's
    /// evidence and reason, a hold, a deferral, an outcome's `because`, the
    /// evidence of a scheduled qualification or of an observation record, a
    /// permission's grant, a retention reason, the cause of a reopening a
    /// rule made, a witness of a journal entry other than a commit's, and
    /// what the last event's effects report.
    evidence: HashSet<NodeId>,
    /// Retired evidence a declaration's `reopen` names that no event
    /// observed, with the sequence the save says it retired at (D051). The
    /// declaration cites it as the program loads, and once it retires
    /// `reopened(...)` reads it observed or not, so what that read makes may
    /// cite it: a lineage or grounds, a basis and its `relies_on` relations,
    /// a commit's journal entry, a reading, an outcome, and the
    /// qualifications a record keeps. Before it retires the read is refused
    /// (D045), so a record the save dates cites it only from then on.
    declared_causes: HashMap<NodeId, u64>,
}

/// The caveats a value can carry, as the loaded source can attach them:
/// those a value resting on each evidence can carry, and those a value can
/// carry with no evidence that bears them. See `caveat_sources`.
struct CaveatSources {
    /// By evidence as the source names it.
    attachable: HashMap<String, HashSet<String>>,
    /// The other way round: by caveat, the evidence that can carry it.
    carriers: HashMap<String, Vec<String>>,
    unpaired: HashSet<String>,
}

/// An effect's kind, as a save writes it.
fn effect_kind(effect: &EffectReport) -> &'static str {
    match effect {
        EffectReport::Sample { .. } => "sample",
        EffectReport::Reveal { .. } => "reveal",
        EffectReport::Examine { .. } => "examine",
        EffectReport::Commit { .. } => "commit",
        EffectReport::Reopen { .. } => "reopen",
        EffectReport::Qualify { .. } => "qualify",
        EffectReport::Unqualify { .. } => "unqualify",
        EffectReport::Renew { .. } => "renew",
        EffectReport::Withdraw { .. } => "withdraw",
        EffectReport::Retire { .. } => "retire",
        EffectReport::Depart { .. } => "depart",
    }
}

/// The records of a windowed history a save holds, by number, and the number
/// of its oldest live record. A number below that one that the save does not
/// hold has departed. See spec/caveat-departure-0.1.md.
struct Held {
    numbers: BTreeSet<u64>,
    oldest_live: u64,
}

impl Held {
    fn of(
        save: &ReactiveSave,
        windows: &BTreeSet<String>,
        journal: bool,
        history: &str,
    ) -> Option<Self> {
        let is_journal = journal && history == "journal";
        if !is_journal && !windows.contains(history) {
            return None;
        }
        let names: Vec<String> = if is_journal {
            (1..=save.decision_journal.len() as u64)
                .map(|index| format!("journal@{}", save.journal_departed + index))
                .collect()
        } else if let Some(stream) = save.reading_streams.get(history) {
            stream.occurrences.iter().map(|o| o.id.clone()).collect()
        } else {
            save.renewals
                .get(history)
                .cloned()
                .unwrap_or_else(|| vec![history.to_string()])
        };
        let number = |name: &str| occurrence_number(name).unwrap_or(0);
        let numbers = names
            .iter()
            .map(|name| number(name))
            .collect::<BTreeSet<_>>();
        let oldest_live = names
            .iter()
            .find(|name| !save.retired.contains_key(*name))
            .map_or_else(|| numbers.last().map_or(1, |n| n + 1), |name| number(name));
        Some(Self {
            numbers,
            oldest_live,
        })
    }

    fn departed(&self, number: u64) -> bool {
        number >= 1 && number < self.oldest_live && !self.numbers.contains(&number)
    }

    /// How many records numbered `from` to `through` have departed.
    fn departed_between(&self, from: u64, through: u64) -> u64 {
        let held = self.numbers.range(from..=through).count() as u64;
        (through - from + 1).saturating_sub(held)
    }

    /// Whether any record has departed.
    fn any_departed(&self) -> bool {
        self.oldest_live > 1 && self.departed_between(1, self.oldest_live - 1) > 0
    }
}

/// The history of `record` if it is a departed record of a windowed history.
fn departed_record(
    save: &ReactiveSave,
    windows: &BTreeSet<String>,
    journal: bool,
    record: &str,
) -> Option<String> {
    let (history, number) = match record.strip_prefix("journal@").filter(|_| journal) {
        Some(number) => ("journal", number.parse::<u64>().ok()?),
        None => {
            let (history, number) = occurrence_parts(record)?;
            (history, number as u64)
        }
    };
    let held = Held::of(save, windows, journal, history)?;
    held.departed(number).then(|| history.to_string())
}

/// A renewal occurrence's number: the declared name is 1, `NAME@N` is N.
pub(super) fn occurrence_number(name: &str) -> Option<u64> {
    Some(occurrence_parts(name).map_or(1, |(_, number)| number as u64))
}

pub(super) fn occurrence_parts(name: &str) -> Option<(&str, usize)> {
    let (base, ordinal) = name.rsplit_once('@')?;
    let ordinal: usize = ordinal.parse().ok()?;
    (ordinal >= 1 && ordinal.to_string() == name[base.len() + 1..]).then_some((base, ordinal))
}

impl ReactiveSession {
    /// The frozen order of each commitment's grounds evidence: from the save
    /// where it records one, otherwise from the commitment's retained
    /// `committed` journal entry, otherwise name order. Never invented.
    fn restore_evidence_order(&mut self, save: &ReactiveSave) -> Result<(), String> {
        let mut orders = BTreeMap::new();
        for (name, names) in &save.evidence_order {
            let grounds = self
                .commitment_grounds
                .get(name)
                .ok_or_else(|| format!("evidence_order names {name}, which has no grounds"))?;
            if names.len() != grounds.evidence.len()
                || names.iter().collect::<BTreeSet<_>>().len() != names.len()
                || names
                    .iter()
                    .any(|evidence| !grounds.evidence.contains(evidence))
                || names
                    .iter()
                    .map(String::as_str)
                    .eq(grounds.evidence.iter().map(Name::as_str))
            {
                return Err(format!(
                    "evidence_order for {name} is not a reordering of its grounds' evidence"
                ));
            }
            orders.insert(name.clone(), names.clone());
        }
        for entry in self.journal.iter() {
            if entry.change != "committed" || orders.contains_key(&entry.commitment) {
                continue;
            }
            let Some(grounds) = self.commitment_grounds.get(&entry.commitment) else {
                continue;
            };
            let names = super::view2::ordered(&grounds.evidence, Some(&entry.because));
            if !names
                .iter()
                .copied()
                .eq(grounds.evidence.iter().map(Name::as_str))
            {
                let names = names.into_iter().map(String::from).collect();
                orders.insert(entry.commitment.clone(), names);
            }
        }
        self.evidence_order = Arc::new(orders);
        Ok(())
    }

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
            source_sha256: Some(self.source_sha256.clone()),
            source_unrecorded_through: self.source_unrecorded_through,
            sequence: self.sequence,
            last_event: self.last_event.clone(),
            last_agent: self.last_agent.clone(),
            elapsed: self.elapsed,
            states: self
                .states
                .names()
                .zip(self.states.cells.iter().zip(self.loaded.states.iter()))
                .filter(|(_, (cell, loaded))| {
                    !Arc::ptr_eq(cell, loaded) && !Self::saved_same(cell, loaded)
                })
                .map(|(name, (cell, _))| {
                    let lineage = Compact::from(&cell.value.provenance);
                    let grounds = Compact::from(&cell.grounds);
                    let state = SavedState {
                        value: cell.value.value,
                        grounds: (grounds != lineage).then_some(grounds),
                        lineage,
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
            commitment_bases: (**self.commitment_bases).clone(),
            commitment_grounds: self
                .commitment_grounds
                .iter()
                .map(|(name, grounds)| (name.clone(), Compact::from(grounds)))
                .collect(),
            evidence_order: self
                .commitment_grounds
                .iter()
                .filter_map(|(name, grounds)| {
                    let order = self.evidence_order.get(name)?;
                    let names = super::view2::ordered(&grounds.evidence, Some(order));
                    (!names
                        .iter()
                        .copied()
                        .eq(grounds.evidence.iter().map(Name::as_str)))
                    .then(|| (name.clone(), names.into_iter().map(String::from).collect()))
                })
                .collect(),
            reading_streams: self
                .reading_streams
                .iter()
                .map(|(name, stream)| (name.clone(), (**stream).clone()))
                .collect(),
            decision_series: (**self.decision_series).clone(),
            renewals: self
                .renewals
                .iter()
                .filter(|(_, renewal)| renewal.occurrences.len() > 1)
                .map(|(name, renewal)| (name.clone(), renewal.occurrences.clone()))
                .collect(),
            observations: (!self.observations.is_empty()).then(|| (**self.observations).clone()),
            identifiers: self.identifiers.texts().to_vec(),
            withdrawals: (**self.withdrawals).clone(),
            taken_back: (**self.taken_back).clone(),
            commitment_permissions: (*self.commitment_permissions).clone(),
            retention_reasons: (*self.retention_reasons).clone(),
            scheduled_qualifications: (*self.scheduled).clone(),
            observation_qualifications: compact_map(&self.observation_qualifications),
            examination_qualifications: compact_map(&self.examination_qualifications),
            reopening_qualifications: compact_map(&self.reopening_qualifications),
            predicate_qualifications: self
                .predicate_qualifications
                .iter()
                .map(|(kind, targets)| (kind.clone(), compact_map(targets)))
                .collect(),
            decision_journal: self.journal.to_vec(),
            retired: (*self.retired).clone(),
            journal_departed: self.journal_departed as u64,
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

    /// Whether a save may leave a state out as the program loaded it: the
    /// same value and provenances.
    fn saved_same(cell: &StateCell, loaded: &StateCell) -> bool {
        cell.value.value.to_bits() == loaded.value.value.to_bits()
            && cell.value.provenance == loaded.value.provenance
            && cell.grounds == loaded.grounds
            && cell.unset == loaded.unset
    }

    pub fn save_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.save()?).map_err(|error| error.to_string())
    }

    /// A session of `source` resumed from `save`: the same state, and the
    /// same outcome for every later event, as the session that was saved.
    pub fn restore(source: &str, save: &ReactiveSave) -> Result<Self, String> {
        let mut session = Self::from_source_cached(source)?;
        session.replay = None;
        session
            .apply_save(save)
            .map_err(|error| format!("cannot restore save: {error}"))?;
        Ok(session)
    }

    /// A state save, or a replay save (spec/caveat-replay-save-0.1.md).
    pub fn restore_json(source: &str, save: &str) -> Result<Self, String> {
        if Self::is_replay_save(save) {
            return Self::restore_replay_json(source, save);
        }
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
        // A save without a digest predates it: the decisions it holds were
        // made under this source (its `source_id` matched) but did not record
        // its digest, so they stay "not recorded". The digest is never
        // supplied for them here.
        self.source_unrecorded_through = match &save.source_sha256 {
            Some(digest) if *digest != self.source_sha256 => {
                return Err("its source digest is not this source's".into());
            }
            Some(_) if save.source_unrecorded_through > save.sequence => {
                return Err("source_unrecorded_through is past its sequence".into());
            }
            Some(_) => save.source_unrecorded_through,
            None if save.source_unrecorded_through != 0 => {
                return Err("source_unrecorded_through without source_sha256".into());
            }
            None => save.sequence,
        };
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
        if self.integer_clock() && !whole_clock_reading(save.elapsed) {
            return Err(format!(
                "elapsed time on an integer clock must be a whole number within -{MAX_INTEGER_CLOCK}..{MAX_INTEGER_CLOCK}"
            ));
        }
        if save.sequence > MAX_SAVED_SEQUENCE {
            return Err(format!("sequence must be at most {MAX_SAVED_SEQUENCE}"));
        }
        self.sequence = save.sequence;
        self.check_saved_agents(save)?;
        self.last_event = save.last_event.clone();
        self.last_agent = save.last_agent.clone();
        self.elapsed = save.elapsed;
        self.identifiers = Arc::new(Identifiers::restore(
            self.identifiers.limit(),
            save.identifiers.clone(),
        )?);
        // Taken from the program as loaded, before the save changes it.
        let caveats = self.caveat_sources();
        let declared = self.declared_memberships();
        // What the declarations spent as the program loaded, before the
        // save replaces it.
        let declared_spent = self.resources.as_ref().map_or(0, |ledger| ledger.spent);
        let observed = self.restore_graph(save, &declared)?;
        self.restore_withdrawals(save, &observed.evidence)?;
        self.restore_taken_back(save)?;
        self.restore_states(&save.states, &observed, &caveats)?;
        self.restore_records(save, &observed, &caveats, &declared, declared_spent)?;
        self.restore_retired(save)?;
        self.restore_permissions(save, &observed.evidence)?;
        self.restore_retention_reasons(save, &observed.evidence)?;
        self.restore_departures(save)?;
        self.evaluate_bindings(None)
            .map_err(|error| error.to_string())?;
        // After every other check, so that a save refused before for
        // anything else keeps its message (D053).
        self.check_last_event_spending(save, declared_spent)?;
        self.settle_archive_provenance(None, false);
        Ok(())
    }

    /// Each agent a save records must be one the program declares and one
    /// that may send the event it is recorded on, and the records of one
    /// event name one agent: a host can send nothing else. See
    /// spec/caveat-agents-0.1.md. Like the rest of restore, this checks that
    /// the save is possible, not who really sent what.
    fn check_saved_agents(&self, save: &ReactiveSave) -> Result<(), String> {
        let mut records: Vec<(u64, &str, Option<&str>)> = Vec::new();
        records.extend(
            save.decision_journal
                .iter()
                .map(|entry| (entry.sequence, entry.event.as_str(), entry.agent.as_deref())),
        );
        for stream in save.reading_streams.values() {
            records.extend(stream.occurrences.iter().map(|reading| {
                (
                    reading.sequence,
                    reading.event.as_str(),
                    reading.agent.as_deref(),
                )
            }));
        }
        records.extend(save.withdrawals.iter().map(|withdrawal| {
            (
                withdrawal.sequence,
                withdrawal.event.as_str(),
                withdrawal.agent.as_deref(),
            )
        }));
        records.extend(save.taken_back.iter().map(|record| {
            (
                record.sequence,
                record.event.as_str(),
                record.agent.as_deref(),
            )
        }));
        if let Some(event) = &save.last_event {
            records.push((save.sequence, event.as_str(), save.last_agent.as_deref()));
        } else if save.last_agent.is_some() {
            return Err("agents: a last agent without a last event".into());
        }
        let mut by_sequence: HashMap<u64, Option<&str>> = HashMap::default();
        for (sequence, event, agent) in records {
            if let Some(agent) = agent {
                if !self.agents.contains_key(agent) {
                    return Err(format!("agents: undeclared agent {agent}"));
                }
            }
            if let Some(senders) = self.senders.get(event) {
                if !agent.is_some_and(|agent| senders.contains(agent)) {
                    return Err(format!(
                        "agents: event {event} at sequence {sequence} is not recorded with an agent that sends it"
                    ));
                }
            }
            if *by_sequence.entry(sequence).or_insert(agent) != agent {
                return Err(format!(
                    "agents: the records of sequence {sequence} name different agents"
                ));
            }
        }
        Ok(())
    }

    /// Departure markers, inherited grounds and the departure rule: see
    /// spec/caveat-lineage-compaction-0.1.md and spec/caveat-departure-0.1.md,
    /// "Save and restore".
    /// No foreclosed action is committed after the irreversible commitment
    /// that forecloses it, or while a reversible revision that forecloses it
    /// is in force: after its commit and before its reopening. A commit with
    /// no journal entry is placed by its revision's sequence alone, and a
    /// reopening the journal no longer holds ends nothing that can be
    /// placed, so it is not checked. See spec/caveat-foreclosure-0.1.md and
    /// spec/caveat-foreclosure-0.2.md.
    fn check_foreclosures_held(&self, save: &ReactiveSave) -> Result<(), String> {
        let journaled = |commitment: &str| {
            save.decision_journal
                .iter()
                .position(|entry| entry.commitment == commitment && entry.change == "committed")
        };
        let commits = |action: &str| -> Vec<(String, u64, Option<usize>)> {
            match save.decision_series.get(action) {
                Some(series) => series
                    .revisions
                    .iter()
                    .map(|revision| {
                        (
                            revision.id.clone(),
                            revision.sequence,
                            journaled(&revision.id),
                        )
                    })
                    .collect(),
                None => journaled(action)
                    .map(|index| {
                        let entry = &save.decision_journal[index];
                        vec![(entry.commitment.clone(), entry.sequence, Some(index))]
                    })
                    .unwrap_or_default(),
            }
        };
        for (series, forecloses) in self.irreversible.iter() {
            let Some((by, sequence, index)) = commits(series).into_iter().next() else {
                continue;
            };
            for action in forecloses {
                for (commitment, later, later_index) in commits(action) {
                    let after = later > sequence
                        || (later == sequence
                            && matches!((index, later_index), (Some(a), Some(b)) if b > a));
                    if after {
                        return Err(format!(
                            "{commitment} is committed after {by}, which forecloses it"
                        ));
                    }
                }
            }
        }
        let reopening = |commitment: &str| {
            save.decision_journal
                .iter()
                .position(|entry| entry.commitment == commitment && entry.change == "reopened")
                .map(|index| (save.decision_journal[index].sequence, Some(index)))
        };
        // Whether `(sequence, index)` comes after, or before, a point.
        let after = |(later, later_index): (u64, Option<usize>),
                     (at, index): (u64, Option<usize>)| {
            later > at
                || (later == at && matches!((index, later_index), (Some(a), Some(b)) if b > a))
        };
        for (series, forecloses) in self.foreclosures.iter() {
            for (by, sequence, index) in commits(series) {
                let current = save
                    .decision_series
                    .get(series)
                    .is_some_and(|saved| saved.current.as_deref() == Some(by.as_str()));
                let end = if save.graph.open.contains(&by) {
                    match reopening(&by) {
                        Some(end) => Some(end),
                        None => continue,
                    }
                } else if current {
                    None
                } else {
                    continue;
                };
                for action in forecloses {
                    for (commitment, later, later_index) in commits(action) {
                        let inside = after((later, later_index), (sequence, index))
                            && end.is_none_or(|end| after(end, (later, later_index)));
                        if inside {
                            return Err(format!(
                                "{commitment} is committed while {by}, which forecloses it, is in force"
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn restore_departures(&mut self, save: &ReactiveSave) -> Result<(), String> {
        let journal = self.journal_window.is_some();
        let windowed = self.windowed();
        // Every provenance the save holds.
        let mut provenances: Vec<(String, &Provenance)> = Vec::new();
        for (name, state) in &save.states {
            provenances.push((format!("state {name}"), &state.lineage.0));
            if let Some(grounds) = &state.grounds {
                provenances.push((format!("state {name} grounds"), &grounds.0));
            }
        }
        for (name, basis) in &save.commitment_bases {
            provenances.push((format!("commitment {name}"), &basis.provenance));
        }
        for (name, grounds) in &save.commitment_grounds {
            provenances.push((format!("commitment {name} grounds"), &grounds.0));
        }
        for (name, stream) in &save.reading_streams {
            provenances.push((
                format!("reading stream {name}"),
                &stream.selection_qualifications,
            ));
            for reading in &stream.occurrences {
                provenances.push((format!("reading {}", reading.id), &reading.provenance));
            }
        }
        for (name, series) in &save.decision_series {
            provenances.push((
                format!("decision series {name}"),
                &series.selection_qualifications,
            ));
            for revision in &series.revisions {
                if let Some(outcome) = &revision.outcome {
                    provenances.push((format!("outcome of {}", revision.id), &outcome.lineage));
                }
            }
        }
        for scheduled in &save.scheduled_qualifications {
            provenances.push(("scheduled qualification".into(), &scheduled.guard));
        }
        for (what, records) in [
            ("observation", &save.observation_qualifications),
            ("examination", &save.examination_qualifications),
            ("reopening", &save.reopening_qualifications),
        ] {
            for (name, provenance) in records {
                provenances.push((format!("{what} of {name}"), &provenance.0));
            }
        }
        for (kind, targets) in &save.predicate_qualifications {
            for (name, provenance) in targets {
                provenances.push((format!("{kind}({name})"), &provenance.0));
            }
        }
        for (id, provenance) in save.cues.iter().zip(&save.cue_qualifications) {
            provenances.push((format!("cue {id}"), &provenance.0));
        }
        // Kept caveats: see spec/caveat-take-back-0.1.md, "Snapshot and save".
        for (what, provenance) in &provenances {
            if let Some(caveat) = provenance.kept.iter().find(|caveat| {
                !self.take_back_caveats.contains(caveat.as_str())
                    || !provenance.caveats.contains(*caveat)
            }) {
                return Err(format!(
                    "{what} keeps {caveat}, which is not a take-back caveat it carries"
                ));
            }
        }
        for (name, state) in &save.states {
            if let Some(grounds) = &state.grounds {
                if !grounds.0.kept.is_subset(&state.lineage.0.kept) {
                    return Err(format!(
                        "state {name} grounds keep a caveat its lineage does not keep"
                    ));
                }
            }
        }
        for (name, grounds) in &save.commitment_grounds {
            if save
                .commitment_bases
                .get(name)
                .is_some_and(|basis| !grounds.0.kept.is_subset(&basis.provenance.kept))
            {
                return Err(format!(
                    "commitment {name} grounds keep a caveat its basis does not keep"
                ));
            }
        }
        let mut any_marker = false;
        for (what, provenance) in &provenances {
            if !provenance.inherited.is_empty() {
                if !windowed {
                    return Err(format!("{what} cannot record inherited names"));
                }
                if let Some(name) = provenance.inherited.difference(&provenance.evidence).next() {
                    return Err(format!("{what} inherited {name}, which it does not hold"));
                }
            }
            for (history, marker) in &provenance.departed {
                any_marker = true;
                let held = Held::of(save, &self.windows, journal, history).ok_or_else(|| {
                    format!("{what} holds departed records of {history}, which has no window")
                })?;
                if marker.from < 1
                    || marker.from > marker.through
                    || !held.departed(marker.from)
                    || !held.departed(marker.through)
                {
                    return Err(format!(
                        "{what}: {history}'s departed range is not of departed records"
                    ));
                }
                if marker.read < 1
                    || marker.read > held.departed_between(marker.from, marker.through)
                {
                    return Err(format!(
                        "{what}: {history}'s departed range counts none, or more records than departed"
                    ));
                }
                if marker.departed_at == 0 || marker.departed_at > save.sequence {
                    return Err(format!("{what}: {history} departed out of sequence"));
                }
            }
        }
        // Whatever departed, departure took every retired record nothing
        // pinned at the end of the event the save follows. A renewable
        // evidence's declared first occurrence never departs.
        let departed = any_marker
            || save.journal_departed > 0
            || self.windows.iter().any(|history| {
                Held::of(save, &self.windows, journal, history)
                    .is_some_and(|held| held.any_departed())
            });
        self.journal_departed = save.journal_departed as usize;
        if departed {
            let pins = self.pins();
            if let Some(record) = self.retired.keys().find(|record| {
                !pins.contains_key(*record) && !self.renewals.contains_key(record.as_str())
            }) {
                return Err(format!(
                    "retired {record} is pinned by nothing, so it would have departed"
                ));
            }
        }
        // An older save may hold retired records nothing pins: they depart
        // at the next accepted event.
        self.rebuild_departure_index();
        Ok(())
    }

    /// The windowed history a retired record belongs to, as the save holds
    /// it, or None.
    fn history_of_record(&self, save: &ReactiveSave, record: &str) -> Option<String> {
        // A stream or renewable may be named `journal` when the program has
        // no journal window; load refuses a windowed one beside it.
        if let Some(index) = record
            .strip_prefix("journal@")
            .filter(|_| self.journal_window.is_some())
        {
            let index: u64 = index.parse().ok()?;
            let departed = save.journal_departed;
            return (index > departed
                && index.to_string() == record["journal@".len()..]
                && index - departed <= save.decision_journal.len() as u64)
                .then(|| "journal".to_string());
        }
        let base = occurrence_parts(record).map_or(record, |(base, _)| base);
        if !self.windows.contains(base) {
            return None;
        }
        let held = if let Some(stream) = save.reading_streams.get(base) {
            stream
                .occurrences
                .iter()
                .any(|occurrence| occurrence.id == record)
        } else if self.renewals.contains_key(base) {
            record == base
                || save
                    .renewals
                    .get(base)
                    .is_some_and(|occurrences| occurrences.iter().any(|name| name == record))
        } else {
            false
        };
        held.then(|| base.to_string())
    }

    /// The `retired` map: see spec/caveat-windows-0.1.md, "Save and restore".
    fn restore_retired(&mut self, save: &ReactiveSave) -> Result<(), String> {
        let added = |record: &str| -> u64 {
            if let Some(index) = record
                .strip_prefix("journal@")
                .filter(|_| self.journal_window.is_some())
            {
                let index: u64 = index.parse().unwrap_or(0);
                return save.decision_journal[(index - save.journal_departed - 1) as usize]
                    .sequence;
            }
            let base = occurrence_parts(record).map_or(record, |(base, _)| base);
            if let Some(stream) = save.reading_streams.get(base) {
                if let Some(occurrence) = stream.occurrences.iter().find(|o| o.id == record) {
                    return occurrence.sequence;
                }
            }
            0
        };
        for (record, at) in &save.retired {
            if self.history_of_record(save, record).is_none() {
                return Err(format!(
                    "retired {record} is no record of a windowed history it holds"
                ));
            }
            if *at == 0 || *at > save.sequence || *at < added(record) {
                return Err(format!("retired {record} is out of sequence"));
            }
        }
        // Each windowed history's live records are exactly its newest
        // min(N, newest): the retired ones before them, retired oldest first.
        // Departed records leave gaps only below the oldest live one
        // (spec/caveat-departure-0.1.md).
        let check = |history: &str, records: Vec<String>, window: usize| -> Result<(), String> {
            let retired = records
                .iter()
                .take_while(|record| save.retired.contains_key(*record))
                .count();
            let live = records.len() - retired;
            let number = |record: &str| -> u64 {
                record
                    .rsplit_once('@')
                    .and_then(|(_, number)| number.parse().ok())
                    .unwrap_or(1)
            };
            let newest = records.last().map_or(0, |record| number(record));
            let live_numbers = records[retired..].iter().map(|record| number(record));
            let contiguous = live_numbers
                .clone()
                .zip(live_numbers.skip(1))
                .all(|(older, newer)| newer == older + 1);
            if records[retired..]
                .iter()
                .any(|record| save.retired.contains_key(record))
                || live as u64 != newest.min(window as u64)
                || !contiguous
            {
                return Err(format!(
                    "{history} does not keep exactly its newest {window} records live"
                ));
            }
            if records[..retired]
                .windows(2)
                .any(|pair| save.retired[&pair[0]] > save.retired[&pair[1]])
            {
                return Err(format!(
                    "{history} retired a newer record before an older one"
                ));
            }
            Ok(())
        };
        for name in self.windows.iter() {
            if let Some(stream) = save.reading_streams.get(name) {
                let records = stream.occurrences.iter().map(|o| o.id.clone()).collect();
                check(name, records, stream.limit)?;
            } else if let Some(renewal) = self.renewals.get(name) {
                let records = save
                    .renewals
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| vec![name.clone()]);
                check(name, records, renewal.limit)?;
            }
        }
        if let Some(window) = self.journal_window {
            let departed = save.journal_departed;
            let records = (1..=save.decision_journal.len() as u64)
                .map(|index| format!("journal@{}", departed + index))
                .collect();
            check("journal", records, window)?;
        } else if save.journal_departed != 0 {
            return Err("journal_departed needs journal window".into());
        }
        // No reopening after a record retired names it as its reason. A later
        // basis may still hold it through one that already cites it, such as
        // a predecessor's (Reactive 0.5).
        let mut holdable = None;
        for entry in save
            .decision_journal
            .iter()
            .filter(|entry| entry.change == "reopened")
        {
            for (record, at) in &save.retired {
                if entry.sequence <= *at || !entry.because.iter().any(|name| name == record) {
                    continue;
                }
                // A reopening that ends a hold, at a release or when it
                // breaks through for a withdrawal, reopens for what the hold
                // deferred, which may have retired since; the hold kept it
                // from departing (spec/caveat-holds-0.1.md, D050). Only a
                // program that withdraws can break through.
                let released =
                    self.event_can_change_decision(&entry.event, &entry.decision, "released");
                if !released
                    && !holdable
                        .get_or_insert_with(|| {
                            if self.withdraws() {
                                self.holdable_series()
                            } else {
                                HashSet::default()
                            }
                        })
                        .contains(entry.decision.as_str())
                {
                    return Err(format!(
                        "{} {} after {record} retired reads it",
                        entry.commitment, entry.change
                    ));
                }
            }
        }
        self.retired = Arc::new(save.retired.clone());
        Ok(())
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
        let holdable = self.holdable_series();
        for withdrawal in &save.withdrawals {
            // Findings 86, 89: its event reaches a `withdraw` of that
            // evidence for that reason. Guards are not evaluated.
            let report = EffectReport::Withdraw {
                evidence: withdrawal.evidence.clone(),
                because: withdrawal.because.clone(),
            };
            let reached = self.reached_effects(Some(&withdrawal.event));
            if !self.last_event_can_make(&report, &reached, None, &holdable) {
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
        self.set_withdrawals(save.withdrawals.clone());
        Ok(())
    }

    /// Take-back records must each name an occurrence and a caveat some
    /// `unqualify` of its event can reach, in sequence order, and agree with
    /// the graph's relations: see spec/caveat-take-back-0.1.md, "Snapshot and
    /// save". Conditions are not evaluated, so a record shows only what the
    /// event could have taken back.
    fn restore_taken_back(&mut self, save: &ReactiveSave) -> Result<(), String> {
        if save.taken_back.is_empty() {
            return Ok(());
        }
        if self.take_back_caveats.is_empty() {
            return Err("save holds take-backs but the program does not take back".into());
        }
        let mut last = 0;
        let mut records = HashMap::<(NodeId, NodeId), usize>::default();
        for record in &save.taken_back {
            let what = format!(
                "the take-back of {} with {}",
                record.evidence, record.caveat
            );
            self.require_kind(&record.evidence, "evidence")
                .map_err(|_| format!("{what} names unknown evidence {}", record.evidence))?;
            let source = self.source_evidence(&record.evidence);
            if source != record.evidence
                && !self.renewals.contains_key(source)
                && !self.reading_streams.contains_key(source)
            {
                return Err(format!("{what} names no occurrence of renewable evidence"));
            }
            if !self
                .take_back_effects()
                .any(|(evidence, caveat)| evidence == source && caveat == record.caveat)
            {
                return Err(format!(
                    "{what}: no unqualify of this program takes back {} with {}",
                    source, record.caveat
                ));
            }
            if record.sequence == 0 || record.sequence > save.sequence || record.sequence < last {
                return Err(format!("{what} is out of sequence"));
            }
            last = record.sequence;
            if !self.events.contains_key(&record.event)
                || !self
                    .reached_effects(Some(&record.event))
                    .iter()
                    .any(|effect| {
                        matches!(effect, Effect::Unqualify { evidence, caveat, .. }
                            if evidence == source && *caveat == record.caveat)
                    })
            {
                return Err(format!(
                    "{what} by {}: no unqualify of that event takes back {} with {}",
                    record.event, source, record.caveat
                ));
            }
            *records
                .entry((self.symbols[&record.caveat], self.symbols[&record.evidence]))
                .or_insert(0) += 1;
        }
        // Each relation but a pair's last needs its record; the last may have
        // one. A take-back caveat has no declared relations.
        let mut relations = HashMap::<(NodeId, NodeId), usize>::default();
        for caveat in self.take_back_caveats.iter() {
            let id = self.symbols[caveat];
            for edge in self
                .graph
                .edges
                .iter()
                .filter(|edge| edge.from == id && edge.relation == Relation::Qualifies)
            {
                *relations.entry((id, edge.to)).or_insert(0) += 1;
            }
        }
        for (pair, count) in &records {
            if relations
                .get(pair)
                .is_none_or(|relations| count > relations)
            {
                return Err("a take-back record has no relation to mark".into());
            }
        }
        for (pair, count) in &relations {
            if records.get(pair).copied().unwrap_or(0) + 1 < *count {
                return Err("a qualification has two live relations".into());
            }
        }
        self.set_taken_back(save.taken_back.clone());
        Ok(())
    }

    /// Retention reasons must name restored commitments, caveats their grounds
    /// hold, and observed evidence in those grounds. See
    /// spec/caveat-retention-reasons-0.1.md.
    fn restore_retention_reasons(
        &mut self,
        save: &ReactiveSave,
        observed: &HashSet<NodeId>,
    ) -> Result<(), String> {
        for (commitment, reasons) in &save.retention_reasons {
            let Some(grounds) = self.commitment_grounds.get(commitment) else {
                return Err(format!(
                    "a retention reason names unknown commitment {commitment}"
                ));
            };
            if reasons.is_empty() {
                return Err(format!("the retention reasons of {commitment} are empty"));
            }
            for (caveat, evidence) in reasons {
                if !grounds.caveats.contains(caveat) {
                    return Err(format!(
                        "{commitment} gives a reason for {caveat}, which its grounds do not retain"
                    ));
                }
                self.require_kind(evidence, "evidence").map_err(|_| {
                    format!(
                        "the reason for {caveat} on {commitment} names unknown evidence {evidence}"
                    )
                })?;
                if self.require_observed(evidence, observed).is_err()
                    || !grounds.evidence.contains(evidence)
                {
                    return Err(format!(
                        "the reason for {caveat} on {commitment}, {evidence}, is not observed evidence in its grounds"
                    ));
                }
            }
        }
        self.retention_reasons = Arc::new(save.retention_reasons.clone());
        Ok(())
    }

    /// Permission records must name restored commitments, known evidence,
    /// finite scope values and declared caveats, and agree with the journal's
    /// `permitted_by` in both directions.
    fn restore_permissions(
        &mut self,
        save: &ReactiveSave,
        observed: &HashSet<NodeId>,
    ) -> Result<(), String> {
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
            // Retirement changes the live observed predicate, not whether
            // this exact grant was observed when permission was recorded.
            if self.require_observed(&record.grant, observed).is_err() {
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
    /// a reading inheriting its template's caveats, `sample ... with` putting
    /// caveats on its reading, and a renewal carrying its evidence's declared
    /// caveats. Guards are not evaluated: a pair is possible, not proof that
    /// an event made it.
    fn qualification_sources(&self) -> (QualificationPairs, QualificationPairs) {
        let names: HashMap<NodeId, &str> = self
            .symbols
            .iter()
            .map(|(name, id)| (*id, name.as_str()))
            .collect();
        let mut possible = HashSet::default();
        let mut scheduled = HashSet::default();
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
                    ..
                } => {
                    let pair = (caveat.clone(), evidence.clone());
                    if after.is_some() {
                        scheduled.insert(pair.clone());
                    }
                    possible.insert(pair);
                }
                Effect::Withdraw {
                    target:
                        EvidenceSelector::Named(evidence)
                        | EvidenceSelector::Latest(evidence)
                        | EvidenceSelector::Occurrence { evidence, .. }
                        | EvidenceSelector::Every(evidence),
                    ..
                } => {
                    possible.insert((WITHDRAWN.to_string(), evidence.clone()));
                }
                // `sample ... with`: spec/caveat-reading-caveats-0.1.md.
                Effect::Sample {
                    stream,
                    with: Some(with),
                    ..
                } => {
                    for caveat in &with.caveats {
                        possible.insert((caveat.clone(), stream.clone()));
                    }
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
        let mut costs = HashMap::default();
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

    /// D052: the names the `reveal`, `sample` and `renew` effects in a rule,
    /// or in a procedure a rule reaches, leave observation records on. A
    /// `reveal` leaves one on its evidence's current occurrence when it adds
    /// a stance that occurrence lacks, or, neutral, when the occurrence is
    /// not observed; on the evidence's own name, then, only with a stance no
    /// declaration gives it, or neutrally when no declaration relates it to
    /// a claim. A `sample` leaves one on the reading it takes, and a `renew`
    /// on the occurrence it makes when its guard holds anything. That guard
    /// is the lineage of the conditions on its rule, on the steps on the way
    /// and on its own step, with that of each argument a call on the way
    /// passes. Only a read of a state, the graph or a history carries
    /// lineage: a payload field, a constant, the clock and a literal carry
    /// none, and a procedure's parameter carries its argument's. Conditions
    /// are not evaluated: a `reveal` or `sample` counts whatever its
    /// conditions hold, and a `renew` counts as guarded when one of those
    /// conditions or arguments reads a state, the graph or a history,
    /// whatever that holds. Restore does not establish that an event left a
    /// record these allow, or could have.
    fn observation_sources(&self, declared: &Declared) -> ObservationSources<'_> {
        // Whether `expression` reads a state, the graph or a history, or
        // calls a function not yet expanded, which could read anything (a
        // loaded program has none). A procedure's parameter does not count:
        // on a path where no argument reads one of those, every parameter is
        // plain, and on any other path the walk already counts the renewal
        // as guarded.
        let lineage = |expression: &Expr| {
            let mut reads = Reads::default();
            expression.collect_reads(&mut reads);
            reads.graph
                || reads.anything
                || reads
                    .names
                    .iter()
                    .any(|name| self.states.contains_key(name))
        };
        let mut sources = ObservationSources::default();
        let mut pending = self
            .rules
            .iter()
            .map(|rule| (&rule.effect, lineage(&rule.condition)))
            .collect::<Vec<_>>();
        let mut visited = HashSet::default();
        while let Some((effect, guarded)) = pending.pop() {
            match effect {
                // Each argument's lineage joins the guard of every step.
                Effect::Call { name, arguments } => {
                    let guarded = guarded || arguments.iter().any(lineage);
                    if visited.insert((name.as_str(), guarded)) {
                        if let Some(procedure) = self.procedures.get(name) {
                            pending.extend(
                                procedure.body.iter().map(|step| {
                                    (&step.effect, guarded || lineage(&step.condition))
                                }),
                            );
                        }
                    }
                }
                Effect::Observe { evidence } => {
                    sources.revealed.insert(evidence.as_str());
                    if !declared.observes(evidence) {
                        sources.own.insert(evidence.as_str());
                    }
                }
                Effect::Reveal {
                    evidence,
                    relation,
                    claim,
                } => {
                    sources.revealed.insert(evidence.as_str());
                    if !declared.gives(evidence, *relation, claim) {
                        sources.own.insert(evidence.as_str());
                    }
                }
                Effect::Sample { stream, .. } => {
                    sources.sampled.insert(stream.as_str());
                }
                Effect::Renew { evidence } => {
                    *sources.renewed.entry(evidence.as_str()).or_default() |= guarded;
                }
                _ => {}
            }
        }
        sources
    }

    /// Findings F330, F333, F335-F337: the caveats some mechanism of the
    /// loaded source can attach to a value. A value resting on evidence
    /// carries what qualifies that evidence (a pair `qualification_sources`
    /// allows, or an extra caveat a `qualified(...)` names), what qualifies a
    /// claim the evidence can bear on, and what qualifies those caveats in
    /// turn. With no evidence it carries only what `examined(...)` reads (the
    /// caveat and what qualifies it) and what a commitment retains by name.
    /// Guards are not evaluated: a caveat listed here could have been
    /// attached, not shown to have been.
    fn caveat_sources(&self) -> CaveatSources {
        let names: HashMap<NodeId, &str> = self
            .symbols
            .iter()
            .map(|(name, id)| (*id, name.as_str()))
            .collect();
        let mut qualifiers: HashMap<String, HashSet<String>> = HashMap::default();
        let (possible, _) = self.qualification_sources();
        for (caveat, target) in possible {
            qualifiers.entry(target).or_default().insert(caveat);
        }
        let mut bears: HashMap<String, HashSet<String>> = HashMap::default();
        let mut unpaired = HashSet::default();
        for edge in self.graph.edges.iter().take(self.loaded.edges) {
            let (Some(from), Some(to)) = (names.get(&edge.from), names.get(&edge.to)) else {
                continue;
            };
            match edge.relation {
                Relation::Supports | Relation::Opposes => {
                    bears
                        .entry(from.to_string())
                        .or_default()
                        .insert(to.to_string());
                }
                Relation::Retains => {
                    unpaired.insert(to.to_string());
                }
                _ => {}
            }
        }
        let mut examined = BTreeSet::new();
        let mut extras = BTreeSet::new();
        let mut read =
            |expression: &Expr| expression.collect_caveat_reads(&mut examined, &mut extras);
        for rule in self.rules.iter() {
            read(&rule.condition);
        }
        for procedure in self.procedures.values() {
            for step in &procedure.body {
                read(&step.condition);
            }
        }
        for (_, _, condition) in self.reopening_triggers.iter() {
            read(condition);
        }
        for (_, value) in self.define_rules.iter() {
            read(value);
        }
        for binding in self.binding_rules.iter() {
            read(&binding.condition);
            if let BindingExpression::Expression(value) = &binding.value {
                read(value);
            }
            for citation in binding.because.iter().flatten() {
                read(citation);
            }
        }
        for effect in self.reached_effects(None) {
            match effect {
                Effect::Call { arguments, .. } => arguments.iter().for_each(&mut read),
                Effect::Sample {
                    stream,
                    value,
                    stance,
                    with,
                    ..
                } => {
                    read(value);
                    if let Some(condition) = with.as_ref().and_then(|with| with.condition.as_ref())
                    {
                        read(condition);
                    }
                    let claims = bears.entry(stream.clone()).or_default();
                    claims.extend(stance.iter().map(|(_, claim)| claim.clone()));
                }
                Effect::Reveal {
                    evidence, claim, ..
                } => {
                    bears
                        .entry(evidence.clone())
                        .or_default()
                        .insert(claim.clone());
                }
                Effect::Qualify { after, index, .. } => {
                    after.iter().chain(index.as_deref()).for_each(&mut read)
                }
                // An occurrence's index joins the effect's lineage. See
                // spec/caveat-earlier-occurrence-0.1.md.
                Effect::Unqualify {
                    index: Some(index), ..
                }
                | Effect::Withdraw {
                    target: EvidenceSelector::Occurrence { index, .. },
                    ..
                }
                | Effect::Reopen {
                    because: EvidenceSelector::Occurrence { index, .. },
                    ..
                } => read(index),
                Effect::Set { value, because, .. } => {
                    read(value);
                    because.iter().flatten().for_each(&mut read);
                }
                Effect::Commit {
                    using,
                    citing,
                    permission,
                    retaining,
                    ..
                } => {
                    using.iter().chain(citing).for_each(&mut read);
                    if let Some(scope) =
                        permission.as_ref().and_then(|clause| clause.scope.as_ref())
                    {
                        read(scope);
                    }
                    unpaired.extend(retaining.iter().cloned());
                }
                _ => {}
            }
        }
        for (caveat, evidence) in extras {
            qualifiers.entry(evidence).or_default().insert(caveat);
        }
        // What a state's initializer attached when the program loaded.
        for name in self.states.names() {
            let lineage = &self.states.get(name).expect("named").value.provenance;
            for evidence in &lineage.evidence {
                qualifiers
                    .entry(self.source_evidence(evidence).to_string())
                    .or_default()
                    .extend(lineage.caveats.iter().map(|caveat| caveat.to_string()));
            }
        }
        // A reading bears on what its stream's samples name, and, at most,
        // on what its template does.
        for (stream, readings) in self.reading_streams.iter() {
            if let Some(claims) = bears.get(&readings.template).cloned() {
                bears.entry(stream.clone()).or_default().extend(claims);
            }
        }
        let incoming = |roots: Vec<String>| {
            let mut frontier = roots;
            let mut visited = HashSet::default();
            let mut caveats = HashSet::default();
            while let Some(name) = frontier.pop() {
                if !visited.insert(name.clone()) {
                    continue;
                }
                for caveat in qualifiers.get(&name).into_iter().flatten() {
                    caveats.insert(caveat.clone());
                    frontier.push(caveat.clone());
                }
            }
            caveats
        };
        let evidence = self
            .symbols
            .iter()
            .filter(|(_, id)| matches!(self.graph.nodes.get(id), Some(NodeKind::Evidence { .. })))
            .map(|(name, _)| name.clone())
            .chain(self.reading_streams.keys().cloned())
            .collect::<HashSet<_>>();
        let attachable: HashMap<String, HashSet<String>> = evidence
            .into_iter()
            .map(|name| {
                let mut roots = vec![name.clone()];
                roots.extend(bears.get(&name).into_iter().flatten().cloned());
                let caveats = incoming(roots);
                (name, caveats)
            })
            .collect();
        unpaired.extend(incoming(examined.iter().cloned().collect()));
        unpaired.extend(examined);
        let mut carriers: HashMap<String, Vec<String>> = HashMap::default();
        for (evidence, caveats) in &attachable {
            for caveat in caveats {
                carriers
                    .entry(caveat.clone())
                    .or_default()
                    .push(evidence.clone());
            }
        }
        CaveatSources {
            attachable,
            carriers,
            unpaired,
        }
    }

    /// The memberships the program declares as it loads: evidence its
    /// declarations relate to a claim, which is observed from the start, with
    /// the stances they give it, and the attention they leave caveats in,
    /// examined ones among them. No event made these, so they have no records.
    fn declared_memberships(&self) -> Declared {
        let names: HashMap<NodeId, &str> = self
            .symbols
            .iter()
            .map(|(name, id)| (*id, name.as_str()))
            .collect();
        let mut stances = HashMap::<String, Vec<(Relation, String)>>::default();
        for edge in self.graph.edges.iter().take(self.loaded.edges) {
            if !matches!(edge.relation, Relation::Supports | Relation::Opposes) {
                continue;
            }
            if let Some(evidence) = names.get(&edge.from) {
                let given = stances.entry(evidence.to_string()).or_default();
                if let Some(claim) = names.get(&edge.to) {
                    given.push((edge.relation, claim.to_string()));
                }
            }
        }
        let attention = self
            .symbols
            .iter()
            .filter_map(|(name, id)| match self.graph.nodes.get(id) {
                Some(NodeKind::Caveat { attention, .. }) if *attention != Attention::Unexamined => {
                    Some((name.clone(), *attention))
                }
                _ => None,
            })
            .collect();
        Declared { stances, attention }
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

    /// The graph as the save leaves it, and the evidence its records may name.
    fn restore_graph(
        &mut self,
        save: &ReactiveSave,
        declared: &Declared,
    ) -> Result<Observed, String> {
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
            self.add_named(name.clone(), kind);
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
            self.relate(from_id, relation, to_id);
        }
        // Evidence a commitment relies on, or that reopens one, was observed
        // when the event added the relation, and stays observed: the next
        // commitment and qualification that read it require it. Checked
        // against the whole graph, which holds the observation somewhere.
        // A basis may also rely on a retired cause a declaration's `reopen`
        // names, which no event need have observed (D051); a reopening a
        // rule made names its cause itself.
        let mut declared_causes = self.retired_declared_causes(save);
        self.restore_observations(save, &declared_causes)?;
        let evidence = self.observed_evidence();
        // A declared cause an event observed, with a stance or in the
        // ledger, is cited as other observed evidence is, and no record is
        // dated by its retirement.
        declared_causes.retain(|id, _| !evidence.contains(id));
        let observed = Observed {
            evidence,
            declared_causes,
        };
        for [from, name, to] in &saved.relations {
            match name.as_str() {
                "relies_on" => self.require_cited(to, &observed),
                "reopens" => self.require_observed(from, &observed.evidence),
                _ => continue,
            }
            .map_err(|error| format!("relation {from} {name} {to}: {error}"))?;
        }
        // Findings 123, 188-191: attention no examination of the source can
        // leave. A reactive event only ever leaves a caveat examined, by an
        // `examine` the budget pays for. D043: the attention a declaration
        // leaves as the program loads claims nothing an event did.
        let costs = self.examination_costs();
        for (name, attention) in &saved.attention {
            self.require_kind(name, "caveat")?;
            let state = attention_from_name(attention)?;
            match state {
                _ if declared.attention.get(name) == Some(&state) => {}
                // D046: no event leaves a caveat unexamined once the
                // declarations leave it otherwise.
                Attention::Unexamined => {
                    if let Some(left) = declared.attention.get(name) {
                        return Err(format!(
                            "caveat {name} is unexamined, but the declarations leave it {}, \
                             and no reactive event leaves a caveat unexamined",
                            attention_name(*left)
                        ));
                    }
                }
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
            self.set_node_attention(id, state);
        }
        for name in &saved.open {
            let id = self.symbols.get(name).copied();
            if !id.is_some_and(|id| self.open_restored_commitment(id)) {
                return Err(format!("{name} is not a commitment"));
            }
        }
        Ok(observed)
    }

    /// Source-reachable neutral effects, including symbol-instantiated
    /// procedures: neutral reveals' evidence and neutral samples' streams.
    fn neutral_targets(&self) -> HashSet<&str> {
        let mut pending = self
            .rules
            .iter()
            .map(|rule| &rule.effect)
            .collect::<Vec<_>>();
        let mut visited = HashSet::default();
        let mut targets = HashSet::default();
        while let Some(effect) = pending.pop() {
            match effect {
                Effect::Observe { evidence } => {
                    targets.insert(evidence.as_str());
                }
                Effect::Sample {
                    stream,
                    stance: None,
                    ..
                } => {
                    targets.insert(stream.as_str());
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

    fn restore_observations(
        &mut self,
        save: &ReactiveSave,
        declared_causes: &HashMap<NodeId, u64>,
    ) -> Result<(), String> {
        let Some(observations) = &save.observations else {
            return Ok(());
        };
        if observations.is_empty() {
            return Err("observations must be omitted or nonempty".into());
        }
        let targets = self.neutral_targets();
        if targets.is_empty() || save.sequence == 0 {
            return Err(
                "observations require a source-reachable neutral reveal or sample in an accepted event"
                    .into(),
            );
        }
        let neutral_target = |name: &str| {
            targets.contains(name)
                || occurrence_parts(name).is_some_and(|(base, _)| {
                    (self.renewals.contains_key(base) || self.reading_streams.contains_key(base))
                        && targets.contains(base)
                })
        };
        let mut seen = HashSet::default();
        for name in observations {
            self.require_kind(name, "evidence")?;
            if !seen.insert(name.as_str()) {
                return Err(format!("observations repeats {name}"));
            }
            if !self.predicate("observed", name)? && !neutral_target(name) {
                return Err(format!(
                    "observations names {name}, which no reachable neutral reveal or sample observes"
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
        // The least name it omits, not the first the set below walks to.
        let omitted = self
            .observed_evidence()
            .into_iter()
            .map(|id| {
                self.symbols
                    .iter()
                    .find_map(|(name, candidate)| (*candidate == id).then_some(name))
                    .unwrap()
            })
            .filter(|name| !seen.contains(name.as_str()))
            .min();
        if let Some(name) = omitted {
            return Err(format!("observations omits stance-bearing evidence {name}"));
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
                // A retired cause a declaration's `reopen` names that the
                // ledger does not hold was never observed, and a reading may
                // cite it once it has retired (D051). One the ledger holds was
                // observed before it retired: a reading cites it only after
                // the ledger observes it, as other evidence.
                let declared_cause = !positions.contains_key(evidence.as_str())
                    && self
                        .symbols
                        .get(evidence.as_str())
                        .and_then(|id| declared_causes.get(id))
                        .is_some_and(|retired| *retired <= reading.sequence);
                if !declared_cause
                    && positions
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
        self.set_observations(observations.clone());
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

    /// The evidence a declaration's `reopen` names that the save says has
    /// retired, with the sequence it says it retired at: what a record may
    /// cite though no event observed it (D051). Only a renewal retires
    /// declared evidence, and `restore_retired` checks each retirement the
    /// save lists.
    fn retired_declared_causes(&self, save: &ReactiveSave) -> HashMap<NodeId, u64> {
        self.graph.edges[..self.loaded.edges]
            .iter()
            .filter(|edge| edge.relation == Relation::Reopens)
            .filter_map(|edge| {
                let retired = save.retired.get(self.symbols.name_of(edge.from)?)?;
                Some((edge.from, *retired))
            })
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

    /// What a record cites: observed evidence, or a retired cause a
    /// declaration's `reopen` names (D051). Refused as `require_observed`
    /// refuses.
    fn require_cited(&self, name: &str, observed: &Observed) -> Result<(), String> {
        if self
            .symbols
            .get(name)
            .is_some_and(|id| observed.declared_causes.contains_key(id))
        {
            return Ok(());
        }
        self.require_observed(name, &observed.evidence)
    }

    /// What a record the save dates cites, made by the event at `made`: a
    /// retired declared cause no event observed only once it had retired,
    /// since reading `reopened(...)` before then is refused (D045, D051).
    /// Refused as `require_observed` refuses; `check_provenance` checks the
    /// rest.
    fn check_cited_once_retired(
        &self,
        what: &str,
        provenance: &Provenance,
        observed: &Observed,
        made: u64,
    ) -> Result<(), String> {
        for name in &provenance.evidence {
            if self
                .symbols
                .get(name.as_str())
                .and_then(|id| observed.declared_causes.get(id))
                .is_some_and(|retired| *retired > made)
            {
                return Err(format!("{what}: {name} is not observed"));
            }
        }
        Ok(())
    }

    /// A provenance whose names are all declared or created evidence and
    /// caveats, and whose evidence is observed: a commitment made on it, or a
    /// qualification of it, requires that. It is what a record cites, so its
    /// evidence may also be a retired declared cause (D051). With `caveats`,
    /// each caveat is also one the source can attach to it.
    fn check_provenance(
        &self,
        what: &str,
        provenance: &Provenance,
        observed: &Observed,
        caveats: Option<&CaveatSources>,
    ) -> Result<(), String> {
        provenance
            .validate()
            .map_err(|error| format!("{what}: {error}"))?;
        self.check_names(what, provenance, observed, true, caveats)
    }

    /// `check_provenance` without the limits on one lineage's size, for names
    /// that are not a lineage: a decision journal entry's witnesses. They are
    /// what a record `cited`, or else what an event named itself, which a
    /// retired declared cause cannot be (D051).
    fn check_names(
        &self,
        what: &str,
        provenance: &Provenance,
        observed: &Observed,
        cited: bool,
        caveats: Option<&CaveatSources>,
    ) -> Result<(), String> {
        for name in &provenance.evidence {
            self.require_kind(name, "evidence")
                .and_then(|()| {
                    if cited {
                        self.require_cited(name, observed)
                    } else {
                        self.require_observed(name, &observed.evidence)
                    }
                })
                .map_err(|error| format!("{what}: {error}"))?;
        }
        for name in &provenance.caveats {
            self.require_kind(name, "caveat")
                .map_err(|error| format!("{what}: {error}"))?;
        }
        match caveats {
            Some(caveats) => self.check_attachable(what, provenance, caveats),
            None => Ok(()),
        }
    }

    /// F330, F333, F335-F337: each caveat is one the source can attach with
    /// no evidence, or to some evidence of the provenance. A caveat that
    /// could attach is accepted without proof that it did.
    fn check_attachable(
        &self,
        what: &str,
        provenance: &Provenance,
        caveats: &CaveatSources,
    ) -> Result<(), String> {
        // The sources of its evidence that can attach a caveat, each once. A
        // decision journal entry can cite many occurrences of one stream, and
        // many caveats of many sources, and no limit keeps its witnesses
        // few: each caveat is looked for among the sources that can carry it
        // or among these, whichever are fewer.
        let mut sources: Option<HashSet<&str>> = None;
        for name in &provenance.caveats {
            if caveats.unpaired.contains(name.as_str()) {
                continue;
            }
            let sources = sources.get_or_insert_with(|| {
                // A departed record's qualifications stay with what carried
                // it, so its history's marker can account for one.
                provenance
                    .evidence
                    .iter()
                    .map(|evidence| self.source_evidence(evidence))
                    .chain(provenance.departed.keys().map(String::as_str))
                    .filter(|source| caveats.attachable.contains_key(*source))
                    .collect()
            });
            let carried = match caveats.carriers.get(name.as_str()) {
                Some(carriers) if carriers.len() <= sources.len() => carriers
                    .iter()
                    .any(|source| sources.contains(source.as_str())),
                Some(_) => sources.iter().any(|source| {
                    caveats
                        .attachable
                        .get(*source)
                        .is_some_and(|attachable| attachable.contains(name.as_str()))
                }),
                None => false,
            };
            if !carried {
                return Err(format!("{what}: {name} cannot qualify any of its evidence"));
            }
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
        observed: &Observed,
        caveats: &CaveatSources,
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
            self.check_provenance(&format!("state {name}"), lineage, observed, Some(caveats))?;
            // Grounds' caveats lie within the lineage just checked, or are
            // refused below as outside it.
            self.check_provenance(&format!("state {name} grounds"), grounds, observed, None)?;
            check_grounds_within_lineage(&format!("state {name}"), grounds, lineage)?;
            let slot = self.states.slot(name).expect("checked above");
            self.states.set(
                slot,
                StateCell {
                    value: Tracked::new(state.value, lineage.clone())?,
                    grounds: grounds.clone(),
                    unset: false,
                },
            );
        }
        Ok(())
    }

    fn restore_records(
        &mut self,
        save: &ReactiveSave,
        observed: &Observed,
        caveats: &CaveatSources,
        declared: &Declared,
        declared_spent: u64,
    ) -> Result<(), String> {
        let holdable = self.holdable_series();
        // When the save says an event made each commitment: a revision's
        // sequence, or that of the commitment's `committed` journal entry
        // while the journal holds it. A basis is frozen then.
        let made = save
            .decision_series
            .values()
            .flat_map(|series| &series.revisions)
            .map(|revision| (revision.id.as_str(), revision.sequence))
            .chain(
                save.decision_journal
                    .iter()
                    .filter(|entry| entry.change == "committed")
                    .map(|entry| (entry.commitment.as_str(), entry.sequence)),
            )
            .collect::<HashMap<_, _>>();
        for (name, basis) in &save.commitment_bases {
            self.require_commitment(name)?;
            if basis.value.is_some_and(|value| !value.is_finite()) {
                return Err(format!("commitment {name} basis is not a finite number"));
            }
            // A listed threshold is one the source declares, at its value:
            // spec/caveat-thresholds-0.1.md.
            for (threshold, value) in &basis.thresholds {
                let declared = self.define_rules.iter().find_map(|(_, expression)| {
                    expression
                        .as_threshold()
                        .filter(|(declared, _)| declared == threshold)
                });
                if declared.is_none_or(|(_, declared)| declared != *value) {
                    return Err(format!(
                        "commitment {name} lists threshold {threshold} = {value}, which the source does not declare"
                    ));
                }
            }
            let what = format!("commitment {name}");
            self.check_provenance(&what, &basis.provenance, observed, Some(caveats))?;
            if let Some(made) = made.get(name.as_str()) {
                self.check_cited_once_retired(&what, &basis.provenance, observed, *made)?;
            }
        }
        for (name, grounds) in &save.commitment_grounds {
            self.require_commitment(name)?;
            // Grounds' caveats lie within the basis checked above, or are
            // refused below as outside it.
            self.check_provenance(
                &format!("commitment {name} grounds"),
                &grounds.0,
                observed,
                None,
            )?;
        }
        if save.reading_streams.keys().ne(self.reading_streams.keys()) {
            return Err("its reading streams are not this program's".into());
        }
        for (name, stream) in &save.reading_streams {
            let declared = &self.reading_streams[name];
            let capacity = if self.windows.contains(name) {
                MAX_WINDOWED_RECORDS
            } else {
                stream.limit
            };
            if stream.template != declared.template
                || stream.limit != declared.limit
                || stream.occurrences.len() > capacity
            {
                return Err(format!("reading stream {name} does not match the program"));
            }
            // Numbered from 1 in order; a windowed stream's departed readings
            // leave gaps (spec/caveat-departure-0.1.md).
            let windowed = self.windows.contains(name);
            let mut last = 0;
            for (index, occurrence) in stream.occurrences.iter().enumerate() {
                let expected = if windowed {
                    occurrence.ordinal
                } else {
                    index as u64 + 1
                };
                // Without a window this is the check rc.15 made, by name alone.
                let in_order =
                    !windowed || (occurrence.ordinal == expected && occurrence.ordinal > last);
                last = occurrence.ordinal;
                if !in_order || occurrence.id != format!("{name}@{expected}") {
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
                let what = format!("reading {}", occurrence.id);
                self.check_provenance(&what, &occurrence.provenance, observed, Some(caveats))?;
                self.check_cited_once_retired(
                    &what,
                    &occurrence.provenance,
                    observed,
                    occurrence.sequence,
                )?;
                self.check_reading_stance(save, name, occurrence)?;
                self.check_reading_note(save, name, occurrence)?;
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
                Some(caveats),
            )?;
        }
        if save.decision_series.keys().ne(self.decision_series.keys()) {
            return Err("its decision series are not this program's".into());
        }
        for (name, series) in &save.decision_series {
            let capacity = if self.windows.contains(name) {
                MAX_WINDOWED_RECORDS
            } else {
                series.limit
            };
            if series.limit != self.decision_series[name].limit || series.revisions.len() > capacity
            {
                return Err(format!("decision series {name} does not match the program"));
            }
            for revision in &series.revisions {
                self.require_commitment(&revision.id)?;
                if !save.commitment_bases.contains_key(&revision.id) {
                    return Err(format!("revision {} has no basis", revision.id));
                }
                if let Some(outcome) = &revision.outcome {
                    self.check_outcome(save, name, revision, outcome, observed, caveats)?;
                }
            }
            if series.current.as_ref() != series.revisions.last().map(|revision| &revision.id) {
                return Err(format!(
                    "decision series {name} current is not its latest revision"
                ));
            }
            // A hold is of a series the source can hold, with a current
            // revision; it defers only while some reason holds it, each name
            // observed evidence, once (spec/caveat-holds-0.1.md).
            if (!series.on_hold.is_empty()
                && (series.current.is_none() || !holdable.contains(name.as_str())))
                || (series.on_hold.is_empty() && !series.deferred.is_empty())
            {
                return Err(format!(
                    "decision series {name} has a hold no source can make"
                ));
            }
            for (index, evidence) in series.on_hold.iter().enumerate() {
                self.require_kind(evidence, "evidence")?;
                self.require_observed(evidence, &observed.evidence)?;
                if series.on_hold[..index].contains(evidence) {
                    return Err(format!("decision series {name} holds {evidence} twice"));
                }
            }
            for (index, evidence) in series.deferred.iter().enumerate() {
                self.require_kind(evidence, "evidence")?;
                self.require_observed(evidence, &observed.evidence)?;
                if series.deferred[..index].contains(evidence) {
                    return Err(format!("decision series {name} defers {evidence} twice"));
                }
            }
            self.check_provenance(
                &format!("decision series {name}"),
                &series.selection_qualifications,
                observed,
                Some(caveats),
            )?;
        }
        self.check_foreclosures_held(save)?;
        for (name, occurrences) in &save.renewals {
            let declared = self
                .renewals
                .get(name)
                .ok_or_else(|| format!("{name} is not renewable"))?;
            let capacity = if self.windows.contains(name) {
                MAX_WINDOWED_RECORDS
            } else {
                declared.limit
            };
            if occurrences.first() != Some(name) || occurrences.len() > capacity {
                return Err(format!(
                    "renewable {name} occurrences do not match the program"
                ));
            }
            let windowed = self.windows.contains(name);
            let mut last = 1;
            for (index, occurrence) in occurrences.iter().enumerate().skip(1) {
                let number = occurrence_parts(occurrence)
                    .filter(|(base, _)| base == name)
                    .map_or(0, |(_, number)| number);
                let expected = if windowed { number } else { index + 1 };
                let in_order = !windowed || number > last;
                last = number;
                if !in_order || *occurrence != format!("{name}@{expected}") {
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
                if !stream.occurrences.iter().any(|occurrence| {
                    occurrence.id == *name && occurrence.ordinal == ordinal as u64
                }) {
                    return Err(format!(
                        "reading stream {base} occurrence {name} is not in its occurrences"
                    ));
                }
            } else if self.renewals.contains_key(base)
                && !save
                    .renewals
                    .get(base)
                    .is_some_and(|occurrences| occurrences.contains(name))
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
            self.require_observed(&scheduled.evidence, &observed.evidence)?;
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
            if self.integer_clock()
                && !(whole_clock_reading(scheduled.scheduled_at)
                    && whole_clock_delay(scheduled.after))
            {
                return Err(
                    "a scheduled qualification's times on an integer clock must be whole numbers in range"
                        .into(),
                );
            }
            self.check_provenance(
                "scheduled qualification",
                &scheduled.guard,
                observed,
                Some(caveats),
            )?;
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
        let costs = self.examination_costs();
        let sources = self.observation_sources(declared);
        // When the save says an event last joined each reopening's record:
        // the latest `reopened`, `held`, `deferred` or `released` entry of its
        // commitment while the journal holds one. Each reopening, hold,
        // deferral and release that joins the record journals one of these
        // for that commitment in the same event, and the journal departs
        // oldest first (D051).
        let joined = save
            .decision_journal
            .iter()
            .filter(|entry| {
                matches!(
                    entry.change.as_str(),
                    "reopened" | "held" | "deferred" | "released"
                )
            })
            .map(|entry| (entry.commitment.as_str(), entry.sequence))
            .collect::<HashMap<_, _>>();
        for (what, records) in [
            ("observation", &save.observation_qualifications),
            ("examination", &save.examination_qualifications),
            ("reopening", &save.reopening_qualifications),
        ] {
            for (name, provenance) in records {
                let id = self.symbols.get(name);
                let fits = match what {
                    "observation" => {
                        id.is_some_and(|id| observed.evidence.contains(id))
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
                    // A cause that departed took its `reopens` relation with
                    // it, and the qualification holds the cause as a marker
                    // (spec/caveat-departure-0.1.md).
                    // A hold, a deferred reopening and a release join the
                    // record of a revision they do not reopen
                    // (spec/caveat-holds-0.1.md).
                    _ => id.is_some_and(|id| {
                        reopened.contains(id)
                            || !provenance.0.departed.is_empty()
                                && matches!(
                                    self.graph.nodes.get(id),
                                    Some(NodeKind::Commitment { open: true, .. })
                                )
                            || matches!(self.graph.nodes.get(id), Some(NodeKind::Commitment { .. }))
                                && save.decision_series.iter().any(|(series, record)| {
                                    holdable.contains(series.as_str())
                                        && record
                                            .revisions
                                            .iter()
                                            .any(|revision| &revision.id == name)
                                })
                    }),
                };
                if !fits {
                    let kind = match what {
                        "observation" => "observed evidence or a renewed occurrence",
                        "examination" => "an examined caveat",
                        _ => "a commitment a reopens relation names",
                    };
                    return Err(format!("{what} of {name}: {name} is not {kind}"));
                }
                // D052: only an event's reveal, sample or renewal leaves an
                // observation record. A reading's record is its sample's,
                // and an occurrence's a reveal's of its renewable evidence or
                // a renewal's whose guard can hold something. The evidence's
                // own name takes one only from a reveal that adds to what the
                // declarations leave: a stance they do not give it, or a
                // neutral reveal of evidence they do not observe.
                if what == "observation" {
                    let source = self.source_evidence(name);
                    let revealed = sources.revealed.contains(source);
                    let missing = if self.reading_streams.contains_key(source) {
                        (!sources.sampled.contains(source)).then(|| format!("samples {source}"))
                    } else if source != name {
                        match (revealed, sources.renewed.get(source)) {
                            (true, _) | (_, Some(true)) => None,
                            (false, Some(false)) => {
                                Some(format!("reveals {source}, or renews it with a guard"))
                            }
                            (false, None) => Some(format!("reveals or renews {source}")),
                        }
                    } else if sources.own.contains(source) {
                        None
                    } else if revealed {
                        Some(format!(
                            "reveals {source} with a stance no declaration gives it"
                        ))
                    } else {
                        Some(format!("reveals {source}"))
                    };
                    if let Some(missing) = missing {
                        return Err(format!(
                            "observation of {name}: no rule or procedure of this program {missing}"
                        ));
                    }
                }
                // D046: only an event's examination leaves a record, and a
                // caveat only the declarations examine has none.
                if what == "examination" && !costs.contains_key(name) {
                    return Err(format!(
                        "examination of {name}: no rule or procedure of this program examines \
                         {name}"
                    ));
                }
                let record = format!("{what} of {name}");
                self.check_provenance(&record, &provenance.0, observed, Some(caveats))?;
                if let Some(joined) = joined.get(name.as_str()).filter(|_| what == "reopening") {
                    self.check_cited_once_retired(&record, &provenance.0, observed, *joined)?;
                }
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
                self.check_provenance(
                    &format!("{kind}({name})"),
                    &provenance.0,
                    observed,
                    Some(caveats),
                )?;
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
        // charges, and the budget spent all of it on top of what the
        // declarations spent. A caveat the declarations leave examined is
        // counted only when its examination record shows an event examined
        // it again: no event need have examined it (D043, D046).
        if let Some(ledger) = &self.resources {
            let least: u128 = save
                .graph
                .attention
                .iter()
                .filter(|(name, attention)| {
                    attention.as_str() == "examined"
                        && (declared.attention.get(*name) != Some(&Attention::Examined)
                            || save.examination_qualifications.contains_key(*name))
                })
                .filter_map(|(name, _)| costs.get(name))
                .map(|cost| u128::from(*cost))
                .sum();
            if least + u128::from(declared_spent) > u128::from(ledger.spent) {
                return Err(if declared_spent == 0 {
                    format!(
                        "examined caveats cost at least {least} attention to examine, more than \
                         the {} the budget spent",
                        ledger.spent
                    )
                } else {
                    format!(
                        "examined caveats cost at least {least} attention to examine and the \
                         declarations spent {declared_spent}, {} in all, more than the {} the \
                         budget spent",
                        least + u128::from(declared_spent),
                        ledger.spent
                    )
                });
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
            self.check_provenance(
                &format!("cue {id}"),
                &qualification.0,
                observed,
                Some(caveats),
            )?;
        }
        for effect in &save.effects {
            let names: Vec<&str> = match effect {
                EffectReport::Sample { id, target, .. } => std::iter::once(id)
                    .chain(target)
                    .map(String::as_str)
                    .collect(),
                EffectReport::Reveal {
                    evidence,
                    relation,
                    target,
                } => {
                    let departed = self.departed_in_last_event(save, evidence);
                    if !departed {
                        self.require_kind(evidence, "evidence")?;
                        self.require_observed(evidence, &observed.evidence)?;
                    }
                    match (relation, target) {
                        (None, None) => {
                            if !departed && !self.observations.iter().any(|name| name == evidence) {
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
                            if !departed
                                && !self.graph.edges.iter().any(|edge| {
                                    edge.from == self.symbols[evidence]
                                        && edge.to == self.symbols[target]
                                        && edge.relation == stance
                                })
                            {
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
                // `unqualify latest(STREAM)` reports the stream, which is no
                // symbol: spec/caveat-latest-0.1.md.
                EffectReport::Unqualify {
                    evidence, caveat, ..
                } if self.reading_streams.contains_key(evidence) => vec![caveat],
                EffectReport::Unqualify {
                    evidence, caveat, ..
                } => vec![evidence, caveat],
                EffectReport::Renew { occurrence, .. } => vec![occurrence],
                EffectReport::Withdraw { evidence, because } => vec![evidence, because],
                // A journal entry is no symbol, and a departed record no
                // longer one; checked against the save below.
                EffectReport::Retire { .. } | EffectReport::Depart { .. } => Vec::new(),
            };
            if let Some(name) = names.into_iter().find(|name| {
                !self.symbols.contains_key(*name) && !self.departed_in_last_event(save, name)
            }) {
                return Err(format!("an effect names unknown {name}"));
            }
            self.check_effect_against_save(effect, save, &observed.evidence, declared_spent)?;
            if !self.last_event_can_make(
                effect,
                &last_effects,
                save.last_event.as_deref(),
                &holdable,
            ) {
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
        // After the journal's own diagnostics: what each entry records. A
        // commit's entry records its grounds' evidence and caveats, which
        // `check_saved_journal` has matched to them, but not their markers.
        // A caveat the grounds carry through a record that departed before
        // the commit is accounted for by that history's marker, here as in
        // the commitment's basis (D049).
        for entry in &save.decision_journal {
            let mut witnesses = journal_witnesses(entry);
            if entry.change == "committed" {
                if let Some(grounds) = save.commitment_grounds.get(&entry.commitment) {
                    witnesses.departed = grounds.0.departed.clone();
                }
            }
            self.check_attachable("decision journal", &witnesses, caveats)?;
        }
        // F317-F322: and the other way, every membership the graph records
        // has its record, as every event that adds one leaves it. Evidence
        // an event observed has an observation record, a caveat an event
        // examined an examination record, and a commitment a `reopens`
        // relation names a reopening record. Declarations add the
        // first two without records.
        let observation = save
            .graph
            .relations
            .iter()
            .filter(|[_, relation, _]| relation == "supports" || relation == "opposes")
            .map(|[evidence, _, _]| evidence)
            .chain(save.observations.iter().flatten());
        for name in observation.filter(|name| !declared.observes(name)) {
            if !save.observation_qualifications.contains_key(name) {
                return Err(format!(
                    "observed evidence {name} has no observation record"
                ));
            }
        }
        for (name, attention) in &save.graph.attention {
            if attention == "examined"
                && declared.attention.get(name) != Some(&Attention::Examined)
                && !save.examination_qualifications.contains_key(name)
            {
                return Err(format!("examined caveat {name} has no examination record"));
            }
        }
        for [_, relation, commitment] in &save.graph.relations {
            if relation == "reopens" && !save.reopening_qualifications.contains_key(commitment) {
                return Err(format!(
                    "reopened commitment {commitment} has no reopening record"
                ));
            }
        }
        self.commitment_bases = Records::new(save.commitment_bases.clone());
        self.commitment_grounds = Records::new(full_map(&save.commitment_grounds));
        self.reading_streams = Records::new(
            save.reading_streams
                .iter()
                .map(|(name, stream)| (name.clone(), Arc::new(stream.clone())))
                .collect(),
        );
        self.decision_series = Records::new(save.decision_series.clone());
        let renewals = Arc::make_mut(&mut self.renewals);
        for (name, occurrences) in &save.renewals {
            renewals.get_mut(name).expect("checked above").occurrences = occurrences.clone();
        }
        self.scheduled = Arc::new(save.scheduled_qualifications.clone());
        self.observation_qualifications = Records::new(full_map(&save.observation_qualifications));
        self.examination_qualifications = Records::new(full_map(&save.examination_qualifications));
        self.reopening_qualifications = Records::new(full_map(&save.reopening_qualifications));
        self.predicate_qualifications = Arc::new(
            save.predicate_qualifications
                .iter()
                .map(|(kind, targets)| (kind.clone(), full_map(targets)))
                .collect(),
        );
        self.journal = save.decision_journal.clone().into();
        self.restore_evidence_order(save)?;
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

    /// What `check_saved_journal` reads for each entry, read once: see
    /// `JournalReads`.
    fn journal_reads<'a>(&'a self, save: &'a ReactiveSave) -> JournalReads<'a> {
        let mut reads = JournalReads {
            qualified_by: HashMap::default(),
            bears_on: HashMap::default(),
            stance: HashSet::default(),
            ledger: self.observations.iter().map(String::as_str).collect(),
            ranks: self.observation_ranks(),
            reopens: HashMap::default(),
            reaching: Reaching {
                by_root: HashMap::default(),
                room: self.graph.nodes.len() + self.graph.edges.len(),
            },
            changes: HashMap::default(),
            revisions: HashMap::default(),
        };
        for (position, edge) in self.graph.edges.iter().enumerate() {
            let kind = |id| self.graph.nodes.get(id);
            match edge.relation {
                Relation::Qualifies
                    if matches!(kind(&edge.from), Some(NodeKind::Caveat { .. })) =>
                {
                    reads
                        .qualified_by
                        .entry(edge.to)
                        .or_default()
                        .push(edge.from);
                }
                Relation::Supports | Relation::Opposes => {
                    reads.stance.insert(edge.from);
                    if matches!(kind(&edge.to), Some(NodeKind::Claim { .. })) {
                        reads.bears_on.entry(edge.from).or_default().push(edge.to);
                    }
                }
                Relation::Reopens if position >= self.loaded.edges => {
                    reads.reopens.entry(edge.to).or_default().push(edge.from);
                }
                _ => {}
            }
        }
        for (name, series) in save.decision_series.iter() {
            for revision in &series.revisions {
                reads
                    .revisions
                    .entry((name.as_str(), revision.id.as_str()))
                    .or_insert(revision);
            }
        }
        reads
    }

    /// Check historical records against the source and the independently saved
    /// graph/bases. This establishes consistency, not authenticity: without an
    /// event log or signature an internally consistent edited save is still data.
    fn check_saved_journal(&self, save: &ReactiveSave, observed: &Observed) -> Result<(), String> {
        let fail = |message: &str| format!("decision journal: {message}");
        if (save.sequence == 0) != save.last_event.is_none() {
            return Err(fail("sequence and last event disagree"));
        }
        let mut reads = self.journal_reads(save);
        let monotonic_time = self.source_clock_is_monotonic();
        let mut committed = HashSet::default();
        // Each commitment's reopening causes in journal order, and as a set.
        let mut reopened: HashMap<&str, (Vec<&str>, HashSet<&str>)> = HashMap::default();
        let mut current: HashMap<&str, &str> = HashMap::default();
        let mut previous: Option<&JournalEntry> = None;
        let mut latest_elapsed = None;
        let mut last_committed_node = None;
        // With departed entries, the journal begins partway: a commitment
        // with no held `committed` entry was made in one that departed, and
        // its series' history before the window is no longer held
        // (spec/caveat-departure-0.1.md).
        let partial = save.journal_departed > 0;
        let held_commits = save
            .decision_journal
            .iter()
            .filter(|entry| entry.change == "committed")
            .map(|entry| entry.commitment.as_str())
            .collect::<HashSet<_>>();
        let before_window = |name: &str| {
            partial
                && !held_commits.contains(name)
                && self
                    .symbols
                    .get(name)
                    .is_some_and(|id| *id > self.loaded.last_node)
        };
        for entry in &save.decision_journal {
            self.require_commitment(&entry.commitment)?;
            if !matches!(
                entry.change.as_str(),
                "committed" | "reopened" | "held" | "deferred" | "released" | "resolved"
            ) {
                return Err(fail("unknown change"));
            }
            // Only an outcome's entry records one (spec/caveat-outcomes-0.1.md).
            if (entry.change == "resolved") != entry.outcome.is_some_and(f64::is_finite)
                || (entry.change != "resolved" && !entry.materialized.is_empty())
            {
                return Err(fail("entry outcome does not match its change"));
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
            let key = (
                entry.event.as_str(),
                entry.decision.as_str(),
                entry.change.as_str(),
            );
            if !*reads.changes.entry(key).or_insert_with(|| {
                self.event_can_change_decision(&entry.event, &entry.decision, &entry.change)
            }) {
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
                    || (self.integer_clock() && !whole_clock_reading(elapsed))
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
            let provenance = journal_witnesses(entry);
            // A commit's entry records its grounds' evidence, which a record
            // cites and which may be a retired declared cause (D051). Any
            // other entry's witnesses are causes an event named itself.
            let cited = entry.change == "committed";
            self.check_names("decision journal", &provenance, observed, cited, None)?;
            // `in_observation_order` and `predicate("observed")`, on the
            // ranks and observations read once.
            let mut ranked = provenance
                .evidence
                .iter()
                .map(|name| (reads.ranks.rank(name), name.as_str()))
                .collect::<Vec<_>>();
            ranked.sort();
            let observed_now = |name: &str| {
                self.symbols.get(name).is_some_and(|id| {
                    matches!(self.graph.nodes.get(id), Some(NodeKind::Evidence { .. }))
                        && ((!self.is_retired(name)
                            && (reads.ledger.contains(name) || reads.stance.contains(id)))
                            || (cited && observed.declared_causes.contains_key(id)))
                })
            };
            if provenance.evidence.len() != entry.because.len()
                || entry
                    .because
                    .iter()
                    .map(String::as_str)
                    .ne(ranked.iter().map(|(_, name)| *name))
                || entry
                    .caveats
                    .iter()
                    .map(String::as_str)
                    .ne(provenance.caveats.iter().map(|caveat| caveat.as_str()))
                || entry.because.iter().any(|name| !observed_now(name))
            {
                return Err(fail(
                    "entry witnesses are unobserved, repeated, or out of order",
                ));
            }
            if save.decision_series.contains_key(&entry.decision) {
                let revision = reads
                    .revisions
                    .get(&(entry.decision.as_str(), entry.commitment.as_str()))
                    .copied()
                    .ok_or_else(|| fail("commitment does not belong to its decision series"))?;
                if entry.change == "committed" {
                    if revision.sequence != entry.sequence || revision.event != entry.event {
                        return Err(fail("commitment disagrees with its revision record"));
                    }
                    let tracked = current.get(entry.decision.as_str()).copied();
                    let follows = match (revision.previous.as_deref(), tracked) {
                        (previous, Some(_)) => previous == tracked,
                        (Some(previous), None) => before_window(previous),
                        (None, None) => true,
                    };
                    if !follows
                        || revision.previous.as_deref().is_some_and(|name| {
                            !reopened.contains_key(name) && !before_window(name)
                        })
                    {
                        return Err(fail("revision has no preceding reopened commitment"));
                    }
                    current.insert(&entry.decision, &entry.commitment);
                } else if entry.change == "resolved" {
                    // An outcome may be of any revision made so far.
                    if !committed.contains(entry.commitment.as_str())
                        && !before_window(&entry.commitment)
                    {
                        return Err(fail("outcome precedes its revision"));
                    }
                } else if current
                    .get(entry.decision.as_str())
                    .map_or(!before_window(&entry.commitment), |current| {
                        *current != entry.commitment.as_str()
                    })
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
                // The journal represents the exact evidence and caveats of
                // grounds, not their departure markers or inherited-name
                // bookkeeping. Keep that full provenance in the grounds:
                // its basis containment and marker/inherited validity are
                // checked separately by restore_records/restore_departures.
                if basis.is_none()
                    || grounds.0.evidence != provenance.evidence
                    || grounds.0.caveats != provenance.caveats
                {
                    return Err(fail("commitment witnesses differ from its frozen grounds"));
                }
            } else {
                if entry.because.is_empty()
                    || (!committed.contains(entry.commitment.as_str())
                        && !before_window(&entry.commitment)
                        && self.symbols[&entry.commitment] > self.loaded.last_node)
                {
                    return Err(fail("reopening has no cause or preceding commitment"));
                }
                // A hold, a deferred reopening and a release change no
                // reopening cause (spec/caveat-holds-0.1.md).
                let (causes, seen) = reopened.entry(&entry.commitment).or_default();
                if entry.change == "reopened" {
                    for evidence in &entry.because {
                        if !seen.insert(evidence) {
                            return Err(fail("reopening repeats an existing cause"));
                        }
                        causes.push(evidence);
                    }
                }
                // Qualification only adds relations, and a take-back marks one
                // without removing it (Take-back 0.1), so a reopening's
                // caveats are a subset of what its witnesses' relations,
                // marked or not, carry today; equality would rewrite time.
                // Each cause passed `predicate("observed")` above, so its
                // qualification roots are itself and the claims it bears on.
                // What reaches any root is what reaches one of them, so a
                // root's caveats are walked once for the journal while
                // `Reaching` has room for them, and once for each entry that
                // needs them past that; an entry citing no caveat walks none.
                let mut unrelated = HashSet::default();
                for caveat in &provenance.caveats {
                    // `check_names` found each a declared caveat.
                    let Some(&id) = self.symbols.get(caveat.as_str()) else {
                        return Err(fail("reopening cites caveats unrelated to its witnesses"));
                    };
                    unrelated.insert(id);
                }
                let mut roots = HashSet::default();
                'causes: for evidence in &entry.because {
                    // And each cause declared evidence.
                    let Some(&id) = self.symbols.get(evidence) else {
                        return Err(format!("{evidence} must name a declared evidence"));
                    };
                    let claims = reads.bears_on.get(&id).map_or(&[][..], Vec::as_slice);
                    for &root in std::iter::once(&id).chain(claims) {
                        if unrelated.is_empty() {
                            break 'causes;
                        }
                        if !roots.insert(root) {
                            continue;
                        }
                        let reach =
                            caveats_reaching_root(&reads.qualified_by, &mut reads.reaching, root);
                        if unrelated.len() <= reach.len() {
                            unrelated.retain(|caveat| !reach.contains(caveat));
                        } else {
                            for caveat in reach.iter() {
                                unrelated.remove(caveat);
                            }
                        }
                    }
                }
                if !unrelated.is_empty() {
                    return Err(fail("reopening cites caveats unrelated to its witnesses"));
                }
            }
            previous = Some(entry);
        }
        for (name, series) in &save.decision_series {
            for (index, revision) in series.revisions.iter().enumerate() {
                if revision.ordinal != index as u64 + 1
                    || revision.id != format!("{name}@{}", index + 1)
                    || !(committed.contains(revision.id.as_str()) || before_window(&revision.id))
                {
                    return Err(fail("revision identity/order has no matching commitment"));
                }
            }
        }
        // In name order: with more than one defect, the first commitment by
        // name decides the refusal, not the symbol table's hash order.
        let mut by_name = self.symbols.iter().collect::<Vec<_>>();
        by_name.sort_unstable();
        for (name, id) in by_name {
            let Some(NodeKind::Commitment { open, .. }) = self.graph.nodes.get(id) else {
                continue;
            };
            if *id > self.loaded.last_node
                && !committed.contains(name.as_str())
                && !before_window(name)
            {
                return Err(fail("created commitment has no journal entry"));
            }
            let causes = reopened
                .get(name.as_str())
                .map_or(&[][..], |(causes, _)| causes.as_slice());
            let edges = reads.reopens.get(id).map_or(&[][..], Vec::as_slice);
            let recorded = causes
                .iter()
                .map(|name| self.symbols[*name])
                .collect::<Vec<_>>();
            // Reopenings recorded in departed entries leave their relations
            // ahead of the held ones, or leave with a departed cause.
            let agrees = if partial {
                edges.ends_with(&recorded)
                    && (*id <= self.loaded.last_node || *open || causes.is_empty())
            } else {
                edges == recorded && (*id <= self.loaded.last_node || *open != causes.is_empty())
            };
            if !agrees {
                return Err(fail("reopening history differs from the graph"));
            }
        }
        Ok(())
    }

    /// A reading has both `relation` and `claim`, or, taken by a neutral
    /// sample, neither. A neutral reading has no stance relation and is in
    /// the observation ledger; `restore_observations` checks that a neutral
    /// `sample` of its stream could have observed it. See
    /// spec/caveat-neutral-sample-0.1.md. Either way the event the reading
    /// names reaches a `sample` of its stream with that stance, in a rule or
    /// a procedure a rule calls; conditions are not evaluated, so the event
    /// could have taken the reading, not shown to have.
    fn check_reading_stance(
        &self,
        save: &ReactiveSave,
        stream: &str,
        reading: &ReadingOccurrence,
    ) -> Result<(), String> {
        let id = &reading.id;
        let stance = match (&reading.relation, &reading.claim) {
            (Some(relation), Some(claim)) => Some((relation.as_str(), claim.as_str())),
            (None, None) => {
                if save.graph.relations.iter().any(|[from, relation, _]| {
                    from == id && (relation == "supports" || relation == "opposes")
                }) {
                    return Err(format!("neutral reading {id} has a stance relation"));
                }
                if !save
                    .observations
                    .as_ref()
                    .is_some_and(|observations| observations.contains(id))
                {
                    return Err(format!("neutral reading {id} is not in observations"));
                }
                None
            }
            _ => return Err(format!("reading {id} needs relation and claim together")),
        };
        let sampled = self
            .reached_effects(Some(&reading.event))
            .into_iter()
            .any(|effect| {
                matches!(effect, Effect::Sample { stream: source, stance: taken, .. }
                    if source == stream
                        && taken.as_ref().map(|(relation, claim)| (relation_name(*relation), claim.as_str())) == stance)
            });
        if !sampled {
            return Err(format!(
                "reading {id} names a sample no rule of event {} makes",
                reading.event
            ));
        }
        Ok(())
    }

    /// A reading's note is the text of an identifier the save holds, taken by
    /// a `sample … noting` of its stream. See spec/caveat-reading-notes-0.1.md.
    fn check_reading_note(
        &self,
        save: &ReactiveSave,
        stream: &str,
        reading: &ReadingOccurrence,
    ) -> Result<(), String> {
        let Some(note) = &reading.note else {
            return Ok(());
        };
        let noted = self.reached_effects(None).into_iter().any(|effect| {
            matches!(effect, Effect::Sample { stream: source, note: Some(_), .. } if source == stream)
        });
        if !noted || !save.identifiers.contains(note) {
            return Err(format!(
                "reading {} has a note no sample of this program could take",
                reading.id
            ));
        }
        Ok(())
    }

    /// Whether a rule or procedure can put `series` on hold
    /// (spec/caveat-holds-0.1.md).
    /// An outcome restore can trust: one a `resolve` of the series in the
    /// source could have recorded on this revision, on observed evidence,
    /// naming caveats the revision retains. Its value is the save's: a save
    /// binds its source, not its history. See spec/caveat-outcomes-0.1.md.
    fn check_outcome(
        &self,
        save: &ReactiveSave,
        series: &str,
        revision: &DecisionRevision,
        outcome: &Outcome,
        observed: &Observed,
        caveats: &CaveatSources,
    ) -> Result<(), String> {
        let fail = |reason: &str| format!("revision {} has an outcome {reason}", revision.id);
        if !outcome.value.is_finite() {
            return Err(fail("that is not a finite number"));
        }
        if outcome.sequence < revision.sequence
            || outcome.sequence > save.sequence
            || !self.events.contains_key(&outcome.event)
            || !self.event_can_change_decision(&outcome.event, series, "resolved")
        {
            return Err(fail("no event of the source can record"));
        }
        if outcome.because.is_empty() {
            return Err(fail("with no evidence"));
        }
        for (index, evidence) in outcome.because.iter().enumerate() {
            self.require_kind(evidence, "evidence")?;
            self.require_observed(evidence, &observed.evidence)?;
            if outcome.because[..index].contains(evidence) {
                return Err(fail(&format!("that cites {evidence} twice")));
            }
        }
        let id = self.symbols[&revision.id];
        for (index, caveat) in outcome.materialized.iter().enumerate() {
            self.require_kind(caveat, "caveat")?;
            let retained = self.symbols.get(caveat).is_some_and(|caveat| {
                self.graph.edges.iter().any(|edge| {
                    edge.from == id && edge.to == *caveat && edge.relation == Relation::Retains
                })
            });
            if !retained || outcome.materialized[..index].contains(caveat) {
                return Err(fail(&format!("whose materialized {caveat} it cannot hold")));
            }
        }
        let what = format!("outcome of {}", revision.id);
        self.check_provenance(&what, &outcome.lineage, observed, Some(caveats))?;
        self.check_cited_once_retired(&what, &outcome.lineage, observed, outcome.sequence)
    }

    /// Each series a rule or procedure can put on hold, read once for every
    /// record that asks (spec/caveat-holds-0.1.md).
    fn holdable_series(&self) -> HashSet<String> {
        self.reached_effects(None)
            .into_iter()
            .filter_map(|effect| match effect {
                Effect::Reopen {
                    action,
                    form: DecisionChange::Hold,
                    ..
                } => Some(action.clone()),
                _ => None,
            })
            .collect()
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
        let mut visited = HashSet::default();
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

    /// An effect can name a record removed later in that same event only
    /// when a departure report agrees with the save's actual history gap.
    /// A missing symbol or an arbitrary departure report alone is insufficient.
    fn departed_in_last_event(&self, save: &ReactiveSave, record: &str) -> bool {
        let Some(history) =
            departed_record(save, &self.windows, self.journal_window.is_some(), record)
        else {
            return false;
        };
        // These callers validate evidence fields. A departed journal entry
        // is a reportable record, but cannot stand in for an evidence node.
        if !self.reading_streams.contains_key(&history) && !self.renewals.contains_key(&history) {
            return false;
        }
        save.effects.iter().any(|effect| {
            matches!(effect,
            EffectReport::Depart { history: reported, record: gone }
                if reported == &history && gone == record)
        })
    }

    /// A saved effect must agree with what the rest of the save restored: its
    /// names have the kinds its place needs, and the relation, record or
    /// spending it reports is there, unless a validated same-event departure
    /// removed that record. The source must still be able to emit the effect.
    fn check_effect_against_save(
        &self,
        effect: &EffectReport,
        save: &ReactiveSave,
        observed: &HashSet<NodeId>,
        declared_spent: u64,
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
                if let Some(target) = target {
                    named(self.require_kind(target, "claim"))?;
                }
                let listed = save.reading_streams.get(stream).is_some_and(|readings| {
                    readings
                        .occurrences
                        .iter()
                        .any(|occurrence| occurrence.id == *id)
                });
                let departed = self.departed_in_last_event(save, id);
                if !listed && !departed {
                    return Err(fail("names no reading of its stream"));
                }
                if !departed {
                    named(self.require_observed(id, observed))?;
                }
                // A neutral reading has neither: spec/caveat-neutral-sample-0.1.md.
                match (relation, target) {
                    (None, None) => {}
                    (Some(relation), Some(target)) => {
                        let stance = match relation.as_str() {
                            "supports" => Relation::Supports,
                            "opposes" => Relation::Opposes,
                            _ => return Err(fail("relation must be supports or opposes")),
                        };
                        if !departed && !related(id, stance, target) {
                            return Err(fail("has no matching graph relation"));
                        }
                    }
                    _ => return Err(fail("requires relation and target together")),
                }
            }
            // Checked with the other reveals' semantics in restore_records.
            EffectReport::Reveal { .. } => {}
            EffectReport::Examine { caveat, cost } => {
                named(self.require_kind(caveat, "caveat"))?;
                let Some(resources) = self.resources.as_ref() else {
                    return Err(fail("spends more attention than the budget spent"));
                };
                if *cost > resources.spent {
                    return Err(fail("spends more attention than the budget spent"));
                }
                // The declarations spent theirs as the program loaded, before
                // any event (D046).
                if *cost > resources.spent.saturating_sub(declared_spent) {
                    return Err(fail(&format!(
                        "spends more attention than the budget spent after the \
                         {declared_spent} the declarations spent"
                    )));
                }
                // All of the last event's examines together are checked
                // last, by `check_last_event_spending` (D053).
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
                let departed = self.departed_in_last_event(save, because);
                if !departed {
                    named(self.require_observed(because, observed))?;
                }
                if !departed && !related(because, Relation::Reopens, action) {
                    return Err(fail("has no matching graph relation"));
                }
            }
            EffectReport::Qualify { evidence, caveat } => {
                let departed = self.departed_in_last_event(save, evidence);
                if !departed {
                    named(self.require_kind(evidence, "evidence"))?;
                }
                named(self.require_kind(caveat, "caveat"))?;
                if !departed && !related(caveat, Relation::Qualifies, evidence) {
                    return Err(fail("has no matching graph relation"));
                }
            }
            EffectReport::Unqualify {
                evidence,
                caveat,
                occurrences,
            } => {
                if !self.reading_streams.contains_key(evidence) {
                    named(self.require_kind(evidence, "evidence"))?;
                }
                named(self.require_kind(caveat, "caveat"))?;
                if occurrences.is_empty() {
                    return Err(fail("marks no occurrence"));
                }
                // Each occurrence the last event marked, in occurrence order;
                // one that departed since takes its record with it.
                let order = |occurrence: &str| {
                    if occurrence == evidence {
                        Some(1)
                    } else {
                        occurrence_parts(occurrence)
                            .filter(|(base, _)| base == evidence)
                            .map(|(_, ordinal)| ordinal)
                    }
                };
                let mut last = 0;
                for occurrence in occurrences {
                    let ordinal = order(occurrence)
                        .ok_or_else(|| fail("names an occurrence of other evidence"))?;
                    if ordinal <= last {
                        return Err(fail("lists its occurrences out of order"));
                    }
                    last = ordinal;
                    let recorded = save.taken_back.iter().any(|record| {
                        record.evidence == *occurrence
                            && record.caveat == *caveat
                            && record.sequence == save.sequence
                    });
                    if !recorded && !self.departed_in_last_event(save, occurrence) {
                        return Err(fail("names an occurrence the last event did not mark"));
                    }
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
                if !listed && !self.departed_in_last_event(save, occurrence) {
                    return Err(fail("names no occurrence of its renewals"));
                }
            }
            EffectReport::Withdraw { evidence, because } => {
                let departed = self.departed_in_last_event(save, evidence);
                if !departed {
                    named(self.require_observed(evidence, observed))?;
                }
                if !self.departed_in_last_event(save, because) {
                    named(self.require_observed(because, observed))?;
                }
                if !departed
                    && !self.withdrawals.iter().any(|withdrawal| {
                        withdrawal.evidence == *evidence && withdrawal.because == *because
                    })
                {
                    return Err(fail("has no matching withdrawal record"));
                }
            }
            EffectReport::Retire { history, record } => {
                // A record can retire and depart in the same event.
                let departed_now = save.effects.iter().any(|effect| {
                    matches!(effect, EffectReport::Depart { record: gone, .. } if gone == record)
                });
                let retired_now = save.retired.get(record) == Some(&save.sequence)
                    && self.history_of_record(save, record).as_deref() == Some(history.as_str());
                if !retired_now && !departed_now {
                    return Err(fail("names no record the last event retired"));
                }
            }
            EffectReport::Depart { history, record } => {
                if departed_record(save, &self.windows, self.journal_window.is_some(), record)
                    .as_deref()
                    != Some(history.as_str())
                {
                    return Err(fail("names no departed record of its history"));
                }
            }
        }
        Ok(())
    }

    /// D053: the last event paid for every `examine` it reports, each in
    /// full, after what the declarations spent as the program loaded and
    /// what earlier events spent, so the budget spent at least what the
    /// declarations spent plus all those costs together.
    /// `check_effect_against_save` checks each cost on its own, which a
    /// save can meet while the sum is more. By now `self`'s budget is the
    /// save's and every effect is one the last event can make; with no
    /// budget, no `examine` effect got this far. The least costs of the
    /// caveats examined before the last event, which `restore_records`
    /// checks, are not added to the sum: the two are checked apart
    /// (docs/RESTORE_TRUST_BOUNDARY.md, "Attention").
    fn check_last_event_spending(
        &self,
        save: &ReactiveSave,
        declared_spent: u64,
    ) -> Result<(), String> {
        let Some(ledger) = &self.resources else {
            return Ok(());
        };
        // Summed wide, as the least costs are: a save may list an effect
        // any number of times, and no sum of the `u64` costs a save can
        // hold overflows a `u128`.
        let costs: u128 = save
            .effects
            .iter()
            .filter_map(|effect| match effect {
                EffectReport::Examine { cost, .. } => Some(u128::from(*cost)),
                _ => None,
            })
            .sum();
        let total = costs + u128::from(declared_spent);
        if total <= u128::from(ledger.spent) {
            return Ok(());
        }
        Err(if declared_spent == 0 {
            format!(
                "examine effects spend {costs} attention together, more than the {} the \
                 budget spent",
                ledger.spent
            )
        } else {
            format!(
                "examine effects spend {costs} attention together and the declarations spent \
                 {declared_spent}, {total} in all, more than the {} the budget spent",
                ledger.spent
            )
        })
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
        holdable: &HashSet<String>,
    ) -> bool {
        let named = |reported: &str, source: &str| {
            reported == source || occurrence_parts(reported).is_some_and(|(base, _)| base == source)
        };
        let selects = |reported: &str, selector: &EvidenceSelector| match selector {
            EvidenceSelector::Named(source)
            | EvidenceSelector::Latest(source)
            | EvidenceSelector::Occurrence {
                evidence: source, ..
            } => named(reported, source),
            EvidenceSelector::Caveated { .. } => true,
            // A reading of a stream that reads from the named evidence counts
            // too (spec/caveat-untrusted-sources-0.1.md).
            EvidenceSelector::Every(source) => {
                named(reported, source)
                    || occurrence_parts(reported).is_some_and(|(stream, _)| {
                        self.reading_streams
                            .get(stream)
                            .is_some_and(|readings| readings.template == *source)
                    })
            }
        };
        // A reopening of a series the program can hold may also name causes
        // deferred by earlier events, at a release or when a withdrawal
        // reaches a revision on hold (spec/caveat-holds-0.1.md).
        let holdable = |series: &str| holdable.contains(series);
        reached.iter().any(|rule| match (effect, rule) {
            (
                EffectReport::Sample {
                    stream,
                    id,
                    relation,
                    target,
                    note,
                    ..
                },
                Effect::Sample {
                    stream: source,
                    stance,
                    note: noting,
                    ..
                },
            ) => {
                stream == source
                    && named(id, source)
                    && (note.is_none() || noting.is_some())
                    && match (relation, target, stance) {
                        (None, None, None) => true,
                        (Some(relation), Some(target), Some((stance, claim))) => {
                            relation == relation_name(*stance) && target == claim
                        }
                        _ => false,
                    }
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
                    form,
                },
            ) => {
                named(action, source)
                    && match form {
                        DecisionChange::Reopen => selects(because, selector) || holdable(source),
                        DecisionChange::Release => true,
                        DecisionChange::Hold => false,
                    }
            }
            (EffectReport::Reopen { action, because }, Effect::Sample { stream, stance, .. }) => {
                (named(because, stream)
                    || self
                        .reopening_triggers
                        .iter()
                        .any(|(series, _, _)| named(action, series) && holdable(series)))
                    && self.reopening_triggers.iter().any(|(series, triggers, _)| {
                        named(action, series)
                            && triggers
                                .iter()
                                .any(|trigger| trigger.matches(stream, stance.as_ref()))
                    })
            }
            (
                EffectReport::Qualify { evidence, caveat },
                Effect::Qualify {
                    evidence: source,
                    caveat: added,
                    after: None,
                    ..
                },
            ) => named(evidence, source) && caveat == added,
            (
                EffectReport::Unqualify {
                    evidence, caveat, ..
                },
                Effect::Unqualify {
                    evidence: source,
                    caveat: taken,
                    ..
                },
            ) => evidence == source && caveat == taken,
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
            // A journal entry retires when a commit, reopen or resolve adds
            // one past `journal window N`, and a sample when its reading
            // reopens a series; a record when its history adds one.
            (EffectReport::Retire { history, .. }, _) if history == "journal" => {
                self.journal_window.is_some()
                    && matches!(
                        rule,
                        Effect::Commit { .. }
                            | Effect::Reopen { .. }
                            | Effect::Resolve(_)
                            | Effect::Sample { .. }
                    )
            }
            (EffectReport::Retire { history, .. }, Effect::Sample { stream, .. }) => {
                history == stream
            }
            (EffectReport::Retire { history, .. }, Effect::Renew { evidence }) => {
                history == evidence
            }
            _ => false,
        }) || matches!(effect, EffectReport::Depart { .. })
            // Any accepted event can release a pin, and a restored session's
            // first event departs what an older save kept.
            && self.windowed()
            || matches!(effect, EffectReport::Qualify { evidence, caveat }
            if last_event.is_some() && last_event == self.time_event.as_deref()
                && self.reached_effects(None).iter().any(|reached| {
                    matches!(reached, Effect::Qualify {
                        evidence: source,
                        caveat: added,
                        after: Some(_),
                        ..
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
        let mut visited = HashSet::default();
        while let Some(effect) = pending.pop() {
            match effect {
                Effect::Commit { action, .. } if change == "committed" && action == decision => {
                    return true
                }
                Effect::Resolve(resolution)
                    if change == "resolved" && resolution.action == decision =>
                {
                    return true
                }
                Effect::Reopen { action, form, .. }
                    if action == decision
                        && matches!(
                            (form, change),
                            (DecisionChange::Reopen, "reopened" | "deferred")
                                | (DecisionChange::Hold, "held")
                                | (DecisionChange::Release, "released" | "reopened")
                        ) =>
                {
                    return true
                }
                // A declared trigger reopens it when this sample's stream,
                // relation and claim match (spec/caveat-reopening-triggers-0.1.md).
                Effect::Sample { stream, stance, .. }
                    if matches!(change, "reopened" | "deferred")
                        && self.reopening_triggers.iter().any(|(series, triggers, _)| {
                            series == decision
                                && triggers
                                    .iter()
                                    .any(|trigger| trigger.matches(stream, stance.as_ref()))
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

#[cfg(test)]
mod journal_order_tests {
    use super::*;

    /// Finding: the last loop of `check_saved_journal` walked the symbol table
    /// in its own (hash) order, so a save in which one commitment has no
    /// journal entry and another disagrees with the graph's reopenings was
    /// refused with either message from process to process. Here the table
    /// walks the second commitment first, so only name order passes.
    #[test]
    fn a_journal_with_two_defects_is_refused_for_the_commitment_first_in_name_order_not_symbol_table_order(
    ) {
        for salt in 0..64 {
            let (plain, reopened) = (format!("a{salt}"), format!("b{salt}"));
            let source = format!(
                "claim c;\nevidence e from \"x\";\nevent go;\nevent pa;\nevent pb;\nevent re;\n\
                 on go reveal e supports c;\non pa commit {plain} because enough;\n\
                 on pb commit {reopened} because enough;\non re reopen {reopened} because e;\n"
            );
            let mut session = ReactiveSession::from_source(&source).unwrap();
            for event in ["go", "pa", "pb", "re"] {
                session.dispatch_json(event, "{}").unwrap();
            }
            let save = session.save().unwrap();
            let restored = ReactiveSession::restore(&source, &save).unwrap();
            let first = restored
                .symbols
                .keys()
                .find(|name| **name == plain || **name == reopened)
                .unwrap();
            if *first == plain {
                // This table walks them in name order already: other names.
                continue;
            }
            // `plain` loses its only entry, and `reopened` its reopening,
            // which the graph still holds.
            let mut edited = save.clone();
            edited
                .decision_journal
                .retain(|entry| entry.commitment != plain && entry.change != "reopened");
            let observed = Observed {
                evidence: restored.observed_evidence(),
                declared_causes: HashMap::default(),
            };
            assert_eq!(
                restored.check_saved_journal(&edited, &observed),
                Err("decision journal: created commitment has no journal entry".to_string())
            );
            assert_eq!(
                ReactiveSession::restore(&source, &edited).map(|_| ()),
                Err(
                    "cannot restore save: decision journal: created commitment has no journal entry"
                        .to_string()
                )
            );
            return;
        }
        panic!("no names put the symbol table's order against name order");
    }
}

#[cfg(test)]
mod journal_reach_tests {
    use super::*;

    /// Evidence 1 to 4, each qualified by caveat 10, which caveats 11, 12 and
    /// 13 qualify in turn, and 10 qualifies 13: each evidence is reached by
    /// 10 to 13.
    fn hub() -> HashMap<NodeId, Vec<NodeId>> {
        let mut qualified_by: HashMap<NodeId, Vec<NodeId>> = HashMap::default();
        for evidence in 1..=4 {
            qualified_by.entry(evidence).or_default().push(10);
        }
        qualified_by.entry(10).or_default().extend([11, 12, 13]);
        qualified_by.entry(13).or_default().push(10);
        qualified_by
    }

    /// What reaches a root is kept while it fits the room left, and walked
    /// again where it is asked for once the room is spent: the same answer
    /// either way.
    #[test]
    fn caveats_reaching_a_root_past_the_room_are_walked_again_not_kept() {
        let expected = [10, 11, 12, 13].into_iter().collect::<HashSet<NodeId>>();
        let qualified_by = hub();
        let mut reaching = Reaching {
            by_root: HashMap::default(),
            room: 9,
        };
        for round in 0..2 {
            for root in 1..=4 {
                let reach = caveats_reaching_root(&qualified_by, &mut reaching, root);
                assert_eq!(*reach, expected, "root {root}, round {round}");
                assert_eq!(
                    matches!(reach, Cow::Borrowed(_)),
                    root <= 2,
                    "root {root}, round {round}"
                );
            }
        }
        assert_eq!(reaching.room, 1);
        assert_eq!(reaching.by_root.len(), 2);
        // Nothing reaches a caveat no caveat qualifies, which takes no room.
        assert!(caveats_reaching_root(&qualified_by, &mut reaching, 11).is_empty());
        assert_eq!(reaching.room, 1);
    }
}
