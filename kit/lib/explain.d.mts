// Types for explain.mjs: why a session is the way it is, and what rests on a
// name. Both read a snapshot and never run a program.
import type {
  ArchiveEntry,
  ArchiveRecord,
  BindingValue,
  CaveatErrorKind,
  DispatchRejected,
  DepartureMarker,
  Effect,
  Outcome,
  PermissionRecord,
  Provenance,
  Snapshot,
  Withdrawal,
} from './session.mjs';

export declare const EXPLAIN_SCHEMA: 'caveat-explain/0.1';
export declare const DEPENDENTS_SCHEMA: 'caveat-dependents/0.1';

/**
 * An event that led to the snapshot, in order: a dispatch outcome without
 * its snapshot (with or without its schema), or a fatal one.
 */
export interface ExplainEvent {
  event: string;
  payload: unknown;
  /** From 1.4: the agent the host said sent it; absent when it named none (spec/caveat-agents-0.1.md). */
  agent?: string;
  outcome: ExplainEventOutcome;
}

/**
 * An event as `explain` takes it. For an accepted event, the snapshot's
 * `sequence` and `effects` after it may come with it; then each revision
 * reports `observed_by`. The report's `events` leave both out.
 */
export interface ExplainInputEvent extends ExplainEvent {
  sequence?: number;
  effects?: Effect[];
}

export type ExplainEventOutcome =
  | { outcome: 'accepted'; schema?: 'caveat-dispatch/0.1' }
  | (Omit<DispatchRejected, 'schema'> & { schema?: 'caveat-dispatch/0.1' })
  | { outcome: 'fatal'; message: string; kind?: CaveatErrorKind | null };

export interface ExplainReport {
  schema: 'caveat-explain/0.1';
  sequence: number;
  elapsed: number;
  /** The session's source digest, and the last sequence whose decisions have none recorded (0 when all do). */
  source: { sha256: string | null; unrecorded_through: number };
  events: ExplainEvent[];
  decisions: ExplainedSeries[];
  evidence: ExplainedEvidence[];
  displayed: ExplainedDisplay[];
  /** Draft archive-backed history, separate from all snapshot-backed fields. */
  archive?: ArchivedHistory;
}

/**
 * Draft report contract: complete checks only the named dependency closure of
 * supplied records. It never proves archive inventory completeness, payload
 * authenticity, absence of other edges, or unique same-source execution.
 */
export interface ArchivedHistory {
  scope: 'provided records and their referenced closure';
  authenticated: false;
  records: ArchivedHistoryRecord[];
  /** Required records or membership roots not reconstructable from this input. */
  unresolved: string[];
  /** Known archived withdrawals; query-matched in a dependents report. */
  withdrawals: Withdrawal[];
  /** Present for dependents: known archived withdrawals with a selected reason. */
  reasonForWithdrawals?: Withdrawal[];
  /** Typed historical edges, deduplicated across supplied complete records. */
  relations: {from: string; relation: string; to: string}[];
}

export interface ArchivedHistoryRecord {
  record: string;
  /** Completeness is scoped to supplied referenced closure, not global history. */
  status: 'complete' | 'unavailable';
  dependencies: string[];
  unresolved: string[];
  /** Present only for a source-matched entry with consistent referenced closure. */
  entry?: ArchiveRecord;
}

export interface ExplainedSeries {
  /** Present for a plain commitment without a declared revision history. */
  kind?: 'plain';
  name: string;
  limit: number;
  current: string | null;
  revisions: ExplainedRevision[];
  /** Present when the series is declared irreversible (spec/caveat-foreclosure-0.1.md). */
  irreversible?: true;
  /** The actions the series forecloses, in source order; present when it declares any. An irreversible series forecloses them once committed, any other while its current revision is in force (spec/caveat-foreclosure-0.2.md). */
  forecloses?: string[];
  /** The revisions that foreclose this decision now, in commit order: committed irreversible revisions, and current revisions in force of series that foreclose it; present when there are any. */
  foreclosed_by?: string[];
}

export type RevisionStatus = 'in force' | 'reopened' | 'superseded';

export interface ExplainedDepartureMarker extends DepartureMarker {
  departed_between: number | null;
  /** Complete means structurally consistent membership, not authenticated history. */
  archive_status: 'complete' | 'unavailable';
  /** Present only when the archive has complete, consistent membership proof. */
  records?: string[];
}

export interface ExplainedProvenance extends Omit<Provenance, 'departed'> {
  departed?: ExplainedDepartureMarker[];
}

export interface ExplainedRevision {
  id: string;
  value: number | null;
  status: RevisionStatus;
  grounds: ExplainedProvenance;
  lineage: ExplainedProvenance;
  /** Caveats retained by the commitment; no evidence-carrier is implied. */
  retained: string[];
  /** What permitted it, frozen: not its grounds. */
  permission: PermissionRecord | null;
  /**
   * Each retained caveat that named a reason, with the reason's occurrence;
   * absent when none did (spec/caveat-retention-reasons-0.1.md).
   */
  retention_reasons?: Record<string, string>;
  /** Grounds withdrawn since the decision was made. */
  withdrawn: Withdrawal[];
  /**
   * For each grounds evidence, in grounds order, the accepted event whose
   * `reveal` or `sample` effect first observed it, with its payload. Present
   * when the events given to `explain` carry their effects; all null when
   * none of them observed it.
   */
  observed_by?: GroundsObservation[];
  /** From 1.3: how it turned out, once a `resolve` recorded it (spec/caveat-outcomes-0.1.md). */
  outcome?: Outcome;
  history: RevisionChange[];
}

export type GroundsObservation =
  | { evidence: string; sequence: number; event: string; agent?: string; payload: unknown }
  | { evidence: string; sequence: null; event: null; payload: null };

export interface RevisionChange {
  change: 'committed' | 'reopened' | 'held' | 'deferred' | 'released' | 'resolved';
  sequence: number;
  event: string;
  /** From 1.4: the agent the host said sent it; absent when it named none (spec/caveat-agents-0.1.md). */
  agent?: string;
  because: string[];
  caveats: string[];
  /** For `resolved`, the outcome's value. */
  outcome?: number;
  /** For `resolved`, the retained caveats it says came true. */
  materialized?: string[];
  /**
   * SHA-256 of the source this change was made under, or null when it ran
   * before saves recorded one ("not recorded"). Identifies the source; does
   * not authenticate a save or its history.
   */
  source_sha256: string | null;
}

export interface ExplainedEvidence {
  id: string;
  /** Null together with claim for observed evidence without a stance. */
  relation: 'supports' | 'opposes' | null;
  claim: string | null;
  value: number | null;
  sequence: number | null;
  event: string | null;
  /** From 1.4: the agent the host said sent it; absent when it named none (spec/caveat-agents-0.1.md). */
  agent?: string;
  /** The caveats that qualify it through a live relation; a taken-back one is listed under `taken_back` instead (D029). */
  caveats: string[];
  /** From 1.5: the take-backs recorded against it, in record order; absent when none (spec/caveat-take-back-0.1.md). */
  taken_back?: TakeBackNote[];
  withdrawn: WithdrawalNote | null;
  /** The sequence at which it retired from its window; absent while live. */
  retired_at?: number;
}

export interface TakeBackNote {
  caveat: string;
  sequence: number;
  event: string;
  /** The agent the host said sent it; absent when it named none (spec/caveat-agents-0.1.md). */
  agent?: string;
}

export interface WithdrawalNote {
  because: string;
  sequence: number;
  event: string;
  /** From 1.4: the agent the host said sent it; absent when it named none (spec/caveat-agents-0.1.md). */
  agent?: string;
}

export interface ExplainedDisplay {
  name: string;
  value: BindingValue;
  cites: ExplainedProvenance;
  /** Present when displayed lineage contains departure markers, including uncited reads. */
  lineage?: ExplainedProvenance;
}

/** What `explain` may be given beside the snapshot. */
export interface ExplainOptions {
  /**
   * Entries drained from the session's archive. Exact names require the
   * marker's complete, hash-consistent, source-matched provenance graph and
   * its departed record entries. Incomplete/conflicting graphs stay ranges.
   * Draft historical rendering additionally joins supplied record payloads in
   * a separate archive section. Source-scoped leaves do not authenticate those
   * payloads or prove the supplied archive is exhaustive.
   */
  archive?: ArchiveEntry[];
}

/** A structured explanation of `snapshot`; `events` lists what led to it, in order. */
export declare function explain(snapshot: Snapshot, events?: ExplainInputEvent[], options?: ExplainOptions): ExplainReport;

/** The explanation as text for a person. `title` names the program. */
export declare function formatExplanation(report: ExplainReport, title?: string): string;

export interface DependentsReport {
  schema: 'caveat-dependents/0.1';
  subject: string;
  kind: 'evidence' | 'caveat';
  sequence: number;
  /** True for a departed subject: complete archive membership is exact; unresolved markers report may rest on. */
  departed?: true;
  /** Retired records the subject stands for, with the sequence at which each retired; absent when none. */
  retired?: RetiredRecord[];
  /** Query-matched withdrawn occurrences, in withdrawal order, even without dependents. */
  withdrawals: Withdrawal[];
  /** Withdrawals whose reason is selected by the query. */
  reasonForWithdrawals: Withdrawal[];
  /** From 1.5: take-backs of the subject caveat, or of a caveat on the subject evidence; absent when none (D029). */
  taken_back?: DependentTakeBack[];
  /** Supporting/opposing relations from query-matched evidence. */
  claims: DependentClaim[];
  decisions: DependentDecision[];
  changes: DependentChange[];
  values: DependentValue[];
  displayed: DependentDisplay[];
  /** Draft historical query results; live lists above retain their meaning. */
  archive?: ArchivedHistory;
}

export interface DependentClaim {
  evidence: string;
  relation: 'supports' | 'opposes';
  claim: string;
}

export interface RetiredRecord {
  record: string;
  sequence: number;
}

export interface DependentDecision {
  id: string;
  value: number | null;
  status: RevisionStatus;
  /** From 1.5, `kept`: a caveat the revision retains that its grounds do not carry, kept through its lineage (D029). */
  basis: 'grounds' | 'permission' | 'retained' | 'kept' | 'lineage' | 'may rest on';
  via: string[];
  /** Withdrawals affecting the query-matched evidence in this reported basis. */
  withdrawn: Withdrawal[];
}

export interface DependentTakeBack extends TakeBackNote {
  evidence: string;
}

export interface DependentChange {
  sequence: number;
  event: string;
  /** From 1.4: the agent the host said sent it; absent when it named none (spec/caveat-agents-0.1.md). */
  agent?: string;
  commitment: string;
  change: 'committed' | 'reopened' | 'held' | 'deferred' | 'released' | 'resolved';
  via: string[];
  /** Withdrawals affecting the query-matched evidence in this reported basis. */
  withdrawn: Withdrawal[];
  /**
   * SHA-256 of the source this change was made under, or null when it ran
   * before saves recorded one ("not recorded"). Identifies the source; does
   * not authenticate a save or its history.
   */
  source_sha256: string | null;
}

export interface DependentValue {
  name: string;
  value: number;
  basis: 'grounds' | 'lineage' | 'may rest on';
  via: string[];
  /** Withdrawals affecting the query-matched evidence in this reported basis. */
  withdrawn: Withdrawal[];
}

export interface DependentDisplay {
  name: string;
  value: BindingValue;
  basis: 'cites' | 'lineage' | 'may rest on';
  via: string[];
  /** Withdrawals affecting the query-matched evidence in this reported basis. */
  withdrawn: Withdrawal[];
}

/**
 * Everything in `snapshot` that rests on `subject`: evidence, a reading
 * stream or a caveat. Throws an Error when the program declares no such name.
 */
export declare function dependents(snapshot: Snapshot, subject: string, options?: ExplainOptions): DependentsReport;

type FormattableDependent<T> = Omit<T, 'withdrawn'> & { withdrawn?: Withdrawal[] };

/** Formatter input also accepts reports written before withdrawal annotations were added. */
export type DependentsFormatInput = Omit<DependentsReport, 'withdrawals' | 'reasonForWithdrawals' | 'claims' | 'decisions' | 'changes' | 'values' | 'displayed'> & {
  withdrawals?: Withdrawal[];
  reasonForWithdrawals?: Withdrawal[];
  claims?: DependentClaim[];
  decisions: FormattableDependent<DependentDecision>[];
  /** Reports written before rc.17 have no `source_sha256`. */
  changes: (FormattableDependent<Omit<DependentChange, 'source_sha256'>> & { source_sha256?: string | null })[];
  values: FormattableDependent<DependentValue>[];
  displayed: FormattableDependent<DependentDisplay>[];
};

/** The dependents report as text for a person. Events is how many led to the snapshot. */
export declare function formatDependents(report: DependentsFormatInput, title?: string, events?: number): string;

export declare const UNDO_SCHEMA: 'caveat-undo/0.1';

/**
 * The undo list (spec/caveat-untrusted-sources-0.1.md, draft for 1.4): every
 * decision revision whose frozen grounds hold withdrawn evidence.
 */
export interface UndoReport {
  schema: 'caveat-undo/0.1';
  sequence: number;
  /** The snapshot's withdrawal ledger, in its order. */
  withdrawals: Withdrawal[];
  /** Revisions resting on withdrawn evidence, in the order `explain` lists them. */
  decisions: UndoDecision[];
}

export interface UndoDecision {
  series: string;
  id: string;
  value: number | null;
  status: RevisionStatus;
  /** Present for an irreversible series: the revision cannot be called back. */
  irreversible?: true;
  /** Present when an outcome was recorded (spec/caveat-outcomes-0.1.md). */
  outcome?: ExplainedRevision['outcome'];
  /** The withdrawals of evidence in the revision's grounds, in ledger order. */
  withdrawn: Withdrawal[];
}

/** Every revision, current or past, resting on withdrawn evidence. Reads the snapshot only. */
export declare function undoList(snapshot: Snapshot): UndoReport;

/** The undo list as text for a person. Events is how many led to the snapshot. */
export declare function formatUndoList(report: UndoReport, title?: string, events?: number): string;

/** An events file: one `{"event": NAME}` per line, with an optional `"payload"`. */
export interface EventLine {
  event: string;
  payload: unknown;
  /** The agent that sent it, when the line names one (spec/caveat-agents-0.1.md). */
  agent?: string;
}

/** Parses an events file. Blank lines are skipped; throws an Error that names the line. */
export declare function parseEvents(text: string): EventLine[];
