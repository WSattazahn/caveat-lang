// Types for explain.mjs: why a session is the way it is, and what rests on a
// name. Both read a snapshot and never run a program.
import type {
  BindingValue,
  CaveatErrorKind,
  DispatchRejected,
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
  outcome: ExplainEventOutcome;
}

export type ExplainEventOutcome =
  | { outcome: 'accepted'; schema?: 'caveat-dispatch/0.1' }
  | (Omit<DispatchRejected, 'schema'> & { schema?: 'caveat-dispatch/0.1' })
  | { outcome: 'fatal'; message: string; kind?: CaveatErrorKind | null };

export interface ExplainReport {
  schema: 'caveat-explain/0.1';
  sequence: number;
  elapsed: number;
  events: ExplainEvent[];
  decisions: ExplainedSeries[];
  evidence: ExplainedEvidence[];
  displayed: ExplainedDisplay[];
}

export interface ExplainedSeries {
  name: string;
  limit: number;
  current: string | null;
  revisions: ExplainedRevision[];
}

export type RevisionStatus = 'in force' | 'reopened' | 'superseded';

export interface ExplainedRevision {
  id: string;
  value: number | null;
  status: RevisionStatus;
  grounds: Provenance;
  lineage: Provenance;
  /** What permitted it, frozen: not its grounds. */
  permission: PermissionRecord | null;
  /** Grounds withdrawn since the decision was made. */
  withdrawn: Withdrawal[];
  history: RevisionChange[];
}

export interface RevisionChange {
  change: 'committed' | 'reopened';
  sequence: number;
  event: string;
  because: string[];
  caveats: string[];
}

export interface ExplainedEvidence {
  id: string;
  relation: 'supports' | 'opposes';
  claim: string;
  value: number | null;
  sequence: number | null;
  event: string | null;
  caveats: string[];
  withdrawn: WithdrawalNote | null;
}

export interface WithdrawalNote {
  because: string;
  sequence: number;
  event: string;
}

export interface ExplainedDisplay {
  name: string;
  value: BindingValue;
  cites: Provenance;
}

/** A structured explanation of `snapshot`; `events` lists what led to it, in order. */
export declare function explain(snapshot: Snapshot, events?: ExplainEvent[]): ExplainReport;

/** The explanation as text for a person. `title` names the program. */
export declare function formatExplanation(report: ExplainReport, title?: string): string;

export interface DependentsReport {
  schema: 'caveat-dependents/0.1';
  subject: string;
  kind: 'evidence' | 'caveat';
  sequence: number;
  decisions: DependentDecision[];
  changes: DependentChange[];
  values: DependentValue[];
  displayed: DependentDisplay[];
}

export interface DependentDecision {
  id: string;
  value: number | null;
  status: RevisionStatus;
  basis: 'grounds' | 'permission' | 'lineage';
  via: string[];
}

export interface DependentChange {
  sequence: number;
  event: string;
  commitment: string;
  change: 'committed' | 'reopened';
  via: string[];
}

export interface DependentValue {
  name: string;
  value: number;
  basis: 'grounds' | 'lineage';
  via: string[];
}

export interface DependentDisplay {
  name: string;
  value: BindingValue;
  basis: 'cites' | 'lineage';
  via: string[];
}

/**
 * Everything in `snapshot` that rests on `subject`: evidence, a reading
 * stream or a caveat. Throws an Error when the program declares no such name.
 */
export declare function dependents(snapshot: Snapshot, subject: string): DependentsReport;

/** The dependents report as text for a person. `events` is how many led to the snapshot. */
export declare function formatDependents(report: DependentsReport, title?: string, events?: number): string;

/** An events file: one `{"event": NAME}` per line, with an optional `"payload"`. */
export interface EventLine {
  event: string;
  payload: unknown;
}

/** Parses an events file. Blank lines are skipped; throws an Error that names the line. */
export declare function parseEvents(text: string): EventLine[];
