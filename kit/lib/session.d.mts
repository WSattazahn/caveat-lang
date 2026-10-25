// Types for session.mjs. Outcomes follow spec/caveat-dispatch-0.1.md, the
// view spec/caveat-view-0.1.md and the check report spec/caveat-check-0.1.md.
// Snapshot and view fields are the runtime's (runtime/src/reactive.rs);
// kit/test/types.test.mjs compares every name here with the code.

/** A value standard JSON carries unchanged. */
export type JsonValue = null | boolean | number | string | JsonValue[] | JsonObject;
export interface JsonObject {
  [key: string]: JsonValue;
}

export declare const DISPATCH_SCHEMA: 'caveat-dispatch/0.1';
export declare const ORIGINS: readonly ['policy', 'input', 'evaluation', 'limit'];

/** Where a classified refusal came from. `host` is reserved and never returned. */
export type Origin = 'policy' | 'input' | 'evaluation' | 'limit';

/** The codes of the Dispatch 0.1 catalog. A later runtime may add codes. */
export type KnownRejectionCode =
  | 'reject'
  | 'not_permitted'
  | 'unknown_event'
  | 'payload_invalid'
  | 'bound_exceeded'
  | 'decision_in_force'
  | 'empty_caveated_selection'
  | 'attention_limit'
  | 'ungrounded_citation'
  | 'work_limit'
  | 'depth_limit'
  | 'history_limit'
  | 'identifier_limit'
  | 'renewal_limit'
  | 'unobserved_evidence'
  | 'expression'
  | 'requirement_failed'
  | 'scheduled_limit'
  | 'not_committed'
  | 'already_committed'
  | 'foreclosed'
  | 'caveated_irreversible'
  | 'already_resolved'
  | 'not_retained'
  | 'reason_spent'
  | 'wrong_agent';
export type RejectionCode = KnownRejectionCode | (string & {});

/** An accepted event, from `dispatch`: the full snapshot after it. */
export interface DispatchAccepted {
  schema: 'caveat-dispatch/0.1';
  outcome: 'accepted';
  snapshot: Snapshot;
}

/** An accepted event, from `dispatchView`: the view after it. */
export interface DispatchViewAccepted<V = View> {
  schema: 'caveat-dispatch/0.1';
  outcome: 'accepted';
  view: V;
}

/**
 * A classified refusal. The session is unchanged. Only a `policy/reject`
 * message is stable: it is the program's quoted literal.
 */
export interface DispatchRejected {
  schema: 'caveat-dispatch/0.1';
  outcome: 'rejected';
  origin: Origin;
  code: RejectionCode;
  message: string;
}

export type DispatchOutcome = DispatchAccepted | DispatchRejected;
export type DispatchViewOutcome<V = View> = DispatchViewAccepted<V> | DispatchRejected;

/** An accepted event, from `dispatchViewDelta`: how the View 0.2 changed. */
export interface DispatchViewDeltaAccepted {
  schema: 'caveat-dispatch/0.1';
  outcome: 'accepted';
  delta: ViewDelta;
}

export type DispatchViewDeltaOutcome = DispatchViewDeltaAccepted | DispatchRejected;

/** The fatal report the runtime throws, kept as `CaveatError.report`. */
export interface DispatchFatal {
  schema: 'caveat-dispatch/0.1';
  outcome: 'fatal';
  code: string;
  message: string;
}

/**
 * `load`: the source did not load, or the runtime build lacks an entry point.
 * `restore`: the save does not restore with this source. `payload`: the
 * payload cannot be sent unchanged as JSON; the session is untouched.
 * `fatal`: the session, or the whole runtime after a trap, is unusable.
 * `closed`: the session was closed.
 */
export type CaveatErrorKind = 'load' | 'restore' | 'payload' | 'fatal' | 'closed';

export declare class CaveatError extends Error {
  constructor(kind: CaveatErrorKind, message: string, report?: DispatchFatal | null);
  readonly kind: CaveatErrorKind;
  /** The runtime's fatal report, when the error carried one. */
  readonly report: DispatchFatal | null;
  /** For an unrecognised outcome only: what arrived, for diagnostics. */
  readonly received?: unknown;
}

/** The payload as JSON text. Throws `CaveatError("payload")` for anything JSON would change. */
export declare function payloadText(payload: unknown): string;

export type SessionState = 'open' | 'closed' | 'fatal';

/** Options for `CaveatSession.save`. */
export interface SaveOptions {
  /** The replay save when it is smaller. Default false: the state save. */
  compact?: boolean;
  /**
   * With `compact`, the positional replay save, which holds more events
   * (spec/caveat-replay-save-0.2.md). Default false.
   */
  positional?: boolean;
}

/**
 * Options for sending one event. `agent` says which declared agent sent it
 * (spec/caveat-agents-0.1.md); a program that declares none refuses any agent
 * as `input/wrong_agent`. A runtime build without agents throws
 * `CaveatError("load")` for an event sent with one.
 */
export interface DispatchOptions {
  agent?: string;
}

/** One program's session. Made by `runtime.open` or `runtime.restore`. */
export declare class CaveatSession {
  #private;
  constructor(inner: RuntimeSessionHandle, runtime: CaveatRuntime);
  get state(): SessionState;
  /**
   * Sends one event. Refusals are returned values; anything else throws
   * `CaveatError("fatal")` and the session is unusable from then on. A payload
   * JSON would change throws `CaveatError("payload")` and changes nothing.
   */
  dispatch(event: string, payload?: object, options?: DispatchOptions): DispatchOutcome;
  /** The same transaction as `dispatch`, with the view in place of the snapshot. */
  dispatchView(event: string, payload?: object, options?: DispatchOptions): DispatchViewOutcome;
  /**
   * The same, with only the named view fields (plus `schema`, `sequence` and
   * `last_event`), each exactly as `view()` has it. The runtime builds no
   * other field. An unknown or repeated name throws `TypeError`.
   */
  dispatchView(event: string, payload: object, options: DispatchOptions & { fields: readonly ViewField[] }): DispatchViewOutcome<ViewSelection>;
  /**
   * The same transaction as `dispatch`, with the View 0.2 delta in place of
   * the snapshot. See spec/caveat-view-0.2.md.
   */
  dispatchViewDelta(event: string, payload?: object, options?: DispatchOptions): DispatchViewDeltaOutcome;
  snapshotText(): string;
  snapshot(): Snapshot;
  viewText(options?: { schema?: '0.1' | '0.2' }): string;
  view(options?: { schema?: '0.1' }): View;
  view(options: { schema: '0.2' }): ViewV2;
  /**
   * JSON text. Restore it with the exact source it came from. With
   * `{ compact: true }`, the replay save while the session's accepted events
   * fit in one (at most 4,096 bytes) and it is smaller, otherwise the same
   * state save; `restore` takes either. A runtime build that predates compact
   * saves throws `CaveatError("load")`. See spec/caveat-replay-save-0.1.md.
   */
  save(options?: SaveOptions): string;
  /**
   * Drain departed record entries and provenance nodes accumulated since the
   * last drain. Record entries retain departure order; proof nodes may refer
   * to earlier chunks. The archive is not saved (spec/caveat-departure-0.1.md).
   */
  drainArchive(): ArchiveEntry[];
  /** How many archive items (records plus provenance nodes) wait to be drained. */
  readonly undrained: number;
  close(): void;
}

/** Where a departed record was named when it departed. */
export interface ArchiveHolder {
  kind: 'state' | 'commitment' | 'series' | 'stream' | 'scheduled' | 'cue' | 'qualification';
  name: string;
  in: string;
}

/** A departed record as it was when it departed (spec/caveat-departure-0.1.md). */
export interface ArchiveRecord {
  kind?: never;
  record: string;
  history: string;
  number: number;
  retired_at: number;
  departed_at: number;
  relations?: [string, string, string][];
  qualifications?: Record<string, Provenance>;
  reading?: ReadingOccurrence;
  journal_entry?: DecisionJournalEntry;
  withdrawal?: Withdrawal;
  holders: ArchiveHolder[];
}

/** A host-drained membership node. Hashes establish consistency, not authenticity. */
export type ArchiveProvenanceNode = {
  kind: 'provenance';
  id: string;
  history: string;
} & ({
  operation: 'record';
  source_id: string;
  record: string;
  departed_at: number;
} | {
  operation: 'union';
  /** Sorted unique IDs of at least two parent nodes. */
  parents: string[];
});

export type ArchiveEntry = ArchiveRecord | ArchiveProvenanceNode;

/** What `loadRuntimeFromDirectory` records; `loadRuntime` keeps whatever it is given. */
export interface RuntimeIdentity {
  reactiveWasmSha256?: string;
  revision?: string;
  clean?: boolean;
  compiled?: boolean;
  host?: string;
  [key: string]: unknown;
}

/** One loaded runtime instance. A trap through any of its sessions stops all of them. */
export interface CaveatRuntime {
  readonly identity: Readonly<RuntimeIdentity>;
  readonly trapped: boolean;
  markTrapped(): void;
  /**
   * Sessions opened or restored through this instance and neither closed nor
   * garbage-collected. Each holds runtime memory until `close()`.
   */
  readonly openSessions: number;
  /** Called by a session's `close()`; hosts do not call it. */
  released(session: CaveatSession): void;
  /** The message a trap is reported with: names running out of memory when the runtime can tell. */
  trapMessage(error: unknown): string;
  /** Throws `CaveatError("load")` when the source does not load. */
  open(source: string): CaveatSession;
  /** Advisory warnings for a program that loads; throws as `open` does when it does not. */
  check(source: string, options?: CheckOptions): CheckReport;
  /** The program's interface (spec/caveat-interface-0.1.md); throws as `open` does when it does not load. */
  interface(source: string): ProgramInterface;
  /** Throws `CaveatError("restore")` when the save does not restore with this source. */
  restore(source: string, saved: string): CaveatSession;
}

/** A session object of the runtime: `WebReactiveSession`, or a stand-in with its methods. */
export interface RuntimeSessionHandle {
  dispatch_outcome(event: string, payload: string): string;
  dispatch_view_outcome?(event: string, payload: string): string;
  dispatch_view_delta_outcome?(event: string, payload: string): string;
  dispatch_view_fields_outcome?(event: string, payload: string, fields: string): string;
  dispatch_outcome_as?(event: string, payload: string, agent: string): string;
  dispatch_view_outcome_as?(event: string, payload: string, agent: string): string;
  dispatch_view_delta_outcome_as?(event: string, payload: string, agent: string): string;
  dispatch_view_fields_outcome_as?(event: string, payload: string, fields: string, agent: string): string;
  snapshot(): string;
  snapshot_compact?(): string;
  view(): string;
  view_v2?(): string;
  save(): string;
  save_compact?(): string;
  save_compact_positional?(): string;
  drain_archive?(): string;
  undrained?(): number;
  free(): void;
}

/** The runtime's session class: `WebReactiveSession`, or a stand-in. */
export interface RuntimeSessionClass {
  new (source: string): RuntimeSessionHandle;
  restore(source: string, saved: string): RuntimeSessionHandle;
  check?(source: string): string;
  check_with?(source: string, unused: boolean): string;
  /** `options` is JSON: `{"unused"?: boolean, "records"?: boolean, "save"?: string}`. */
  check_options?(source: string, options: string): string;
  interface?(source: string): string;
  /** True once the instance has failed to allocate memory; safe to read after a trap. */
  out_of_memory?(): boolean;
}

/** The imported namespace of the runtime's caveat_runtime.js. */
export interface RuntimeModule {
  default(...options: never[]): Promise<unknown>;
  WebReactiveSession: RuntimeSessionClass;
}

export declare function createRuntime(SessionClass: RuntimeSessionClass, identity?: RuntimeIdentity): CaveatRuntime;

export interface LoadRuntimeOptions {
  /**
   * A URL of caveat_runtime.js (a relative one resolves against session.mjs),
   * or its imported namespace. Only a URL can be imported again once the
   * instance has trapped.
   */
  module: string | URL | RuntimeModule;
  /** What the runtime's init accepts: bytes, a URL or a Response. Left out, the runtime finds its own wasm. */
  wasm?: unknown;
  identity?: RuntimeIdentity;
}

export declare function loadRuntime(options: LoadRuntimeOptions): Promise<CaveatRuntime>;

// ------------------------------------------------------------- interface

/** A program's interface (spec/caveat-interface-0.1.md). Lists are in name order. */
export interface ProgramInterface {
  schema: 'caveat-interface/0.1';
  events: InterfaceEvent[];
  states: InterfaceState[];
  bindings: InterfaceBinding[];
  cues: InterfaceCue[];
  decisions: InterfaceDecisions[];
  readings: InterfaceReadings[];
  evidence: string[];
  caveats: string[];
  claims: string[];
  /** From 1.4: the declared agents, in name order; absent when there are none (spec/caveat-agents-0.1.md). */
  agents?: InterfaceAgent[];
}

/** An agent, and the events its declaration says it sends, in declared order. */
export interface InterfaceAgent {
  name: string;
  sends: string[];
}

export interface InterfaceEvent {
  name: string;
  /** The payload's fields, in declared order. */
  parameters: InterfaceParameter[];
}

/** `min` and `max` for `number`; `kind` for `entity`; `members` for `entity` and `member`. */
export interface InterfaceParameter {
  name: string;
  type: InterfaceParameterType;
  min?: number;
  max?: number;
  kind?: string;
  members?: string[];
}

export type InterfaceParameterType = 'number' | 'entity' | 'member' | 'id';

export interface InterfaceState {
  name: string;
  min: number;
  max: number;
}

/** `always`: some declaration has no condition, so every view shows it. */
export interface InterfaceBinding {
  target: string;
  property: string;
  type: InterfaceBindingType;
  always: boolean;
}

export type InterfaceBindingType = 'number' | 'boolean' | 'text';

export interface InterfaceCue {
  name: string;
  kind: 'sound' | 'toast' | 'flash' | 'ring';
}

export interface InterfaceDecisions {
  name: string;
  limit: number;
}

export interface InterfaceReadings {
  name: string;
  evidence: string;
  limit: number;
}

// ------------------------------------------------------------- check report

/** Checks reported only when asked for, so a program that checked clean before still does. */
export interface CheckOptions {
  /** Also report C007 `unused-declaration`: a claim, evidence, caveat or state nothing else names. */
  unused?: boolean;
  /**
   * Also report C008 `record-reads-late-caveat`: a rule, procedure or binding
   * that reads a decision's record in a program that qualifies evidence late
   * (spec/caveat-caveat-sources-0.1.md, D015).
   */
  records?: boolean;
  /**
   * With `records`, a save's text: also report C009 `save-lacks-late-caveat`
   * for each value it holds without a late caveat its evidence carries.
   * Throws `CaveatError("restore")` when it does not restore.
   */
  save?: string;
}

export interface CheckReport {
  schema: 'caveat-check/0.1';
  diagnostics: CheckDiagnostic[];
  /** Warnings an allow comment silenced, in the same form. */
  suppressed: CheckDiagnostic[];
}

export interface CheckDiagnostic {
  code: string;
  name: string;
  severity: 'warning';
  line: number;
  column: number;
  message: string;
  suggestion: string;
  related: CheckRelated[];
}

export interface CheckRelated {
  line: number;
  column: number;
  note: string;
}

// ------------------------------------------------------------- snapshot and view

/** What a value cites or could have been influenced by: evidence and caveat names, sorted. */
export interface Provenance {
  evidence: string[];
  caveats: string[];
  /**
   * Departed records of each windowed history, one marker per history in
   * place of their names (spec/caveat-lineage-compaction-0.1.md). Absent
   * unless a record has departed.
   */
  departed?: DepartureMarker[];
  /**
   * The names of `evidence` inherited through a selection or reopening
   * qualification rather than read. Only a program with a window keeps it;
   * absent when empty.
   */
  inherited?: string[];
  /**
   * The take-back caveats among `caveats` this value holds through a record
   * rather than a live qualification, so no take-back removes them. Only a
   * program that takes back keeps any; absent when empty
   * (spec/caveat-take-back-0.1.md).
   */
  kept?: string[];
}

/** At least `read` of `history`'s departed records numbered `from` to `through`, the last departing at `departed_at`. */
export interface DepartureMarker {
  history: string;
  read: number;
  from: number;
  through: number;
  departed_at: number;
  /** Optional immutable root in the host-drained membership archive. */
  archive_ref?: string;
}

export type BindingValue = number | boolean | string;
/** Displayed values, by target and property. */
export type Bindings = Record<string, Record<string, BindingValue>>;

/** Everything the session knows. Large: a host that redraws reads `view()`. */
export interface Snapshot {
  schema: 'caveat-reactive/0.1';
  source_id: string;
  /** SHA-256 of the exact source text, lowercase hex. Identifies the source; does not authenticate a save. */
  source_sha256: string;
  /** Events at or before this sequence ran before the session's saves recorded its digest; absent when none did. */
  source_unrecorded_through?: number;
  sequence: number;
  elapsed: number;
  renewals: Record<string, Renewal>;
  /** Texts received for `id` parameters, in handle order; absent when there are none. */
  identifiers?: string[];
  /** Withdrawn observations, in order; absent when there are none. */
  withdrawals?: Withdrawal[];
  /** Taken-back qualifications, in order; absent when there are none (spec/caveat-take-back-0.1.md). */
  taken_back?: TakeBack[];
  /** What each permitted commitment was permitted by; absent when there are none. */
  commitment_permissions?: Record<string, PermissionRecord>;
  /**
   * By commitment, each retained caveat that named a reason, with the reason's
   * occurrence; absent when there are none (spec/caveat-retention-reasons-0.1.md).
   */
  retention_reasons?: Record<string, Record<string, string>>;
  scheduled_qualifications: ScheduledQualification[];
  last_event: string | null;
  /** The agent the host said sent the last event; absent when it named none. */
  last_agent?: string;
  values: Record<string, number>;
  qualified_values: Record<string, QualifiedValue>;
  commitment_bases: Record<string, CommitmentBasis>;
  reading_streams: Record<string, ReadingStream>;
  decision_series: Record<string, DecisionSeries>;
  /**
   * Irreversible series and the actions each forecloses, in source order;
   * absent when no series is irreversible (spec/caveat-foreclosure-0.1.md).
   */
  irreversible?: Record<string, string[]>;
  /** First-observation order; present after neutral observation, otherwise derived from stance relations. */
  observations?: string[];
  observation_qualifications: Record<string, Provenance>;
  examination_qualifications: Record<string, Provenance>;
  reopening_qualifications: Record<string, Provenance>;
  predicate_qualifications: Record<string, Record<string, Provenance>>;
  events: EventSignature[];
  bindings: Bindings;
  binding_qualifications: Record<string, Record<string, Provenance>>;
  binding_explanations: Record<string, Record<string, Provenance>>;
  value_grounds: Record<string, Provenance>;
  commitment_grounds: Record<string, Provenance>;
  decision_journal: DecisionJournalEntry[];
  /** Each retired record of a windowed history, with the sequence of the event that retired it; journal entries are `journal@K`. Absent when nothing has retired (spec/caveat-windows-0.1.md). */
  retired?: Record<string, number>;
  /** The histories declared with a window, `journal` for `journal window`; their `limit` is the window. Absent when none is. */
  windows?: string[];
  cues: Cue[];
  cue_qualifications: Provenance[];
  controls: Record<string, Control>;
  clock: Clock | null;
  world: World;
  scenes: string[];
  labels: Record<string, string>;
  symbols: SnapshotSymbol[];
  relations: Relation[];
  commitments: Commitment[];
  budget: Budget | null;
  effects: Effect[];
}

/** What an event can change; each field equals the snapshot's. */
export interface View {
  schema: 'caveat-reactive-view/0.1';
  sequence: number;
  last_event: string | null;
  bindings: Bindings;
  binding_explanations: Record<string, Record<string, Provenance>>;
  cues: Cue[];
  effects: Effect[];
  commitments: Commitment[];
  commitment_grounds: Record<string, Provenance>;
  decision_series: Record<string, DecisionSeries>;
  decision_journal: DecisionJournalEntry[];
  relations: Relation[];
}

/** A field `dispatchView` can select. */
export type ViewField = Exclude<keyof View, 'schema' | 'sequence' | 'last_event'>;

/** The view fields `VIEW_FIELDS` lists, in the view's order. */
export declare const VIEW_FIELDS: readonly ViewField[];

/** A view with only the selected fields: each one present is exactly the full view's. */
export type ViewSelection = Pick<View, 'schema' | 'sequence' | 'last_event'> & Partial<Pick<View, ViewField>>;

/**
 * View 0.2: View 0.1 with grounds evidence in first-observed order and
 * `reopened` in place of `open`. See spec/caveat-view-0.2.md.
 */
export interface ViewV2 extends Omit<View, 'schema' | 'commitments'> {
  schema: 'caveat-reactive-view/0.2';
  commitments: ViewCommitment[];
  /** The snapshot's take-back records; present only in a program with an `unqualify`. */
  taken_back?: TakeBack[];
}

/** A commitment as View 0.2 shows it. */
export interface ViewCommitment extends Omit<Commitment, 'open'> {
  /** True when the commitment was reopened and awaits a new decision. */
  reopened: boolean;
}

/** How the full View 0.2 changed across one accepted event. */
export interface ViewDelta {
  schema: 'caveat-reactive-view-delta/0.2';
  /** The sequence of the view this delta applies to. */
  since: number;
  sequence: number;
  last_event: string | null;
  bindings: { set: Bindings; removed: [string, string][] };
  binding_explanations: { set: Record<string, Record<string, Provenance>>; removed: [string, string][] };
  cues: Cue[];
  effects: Effect[];
  commitments: { set: ViewCommitment[]; removed: string[] };
  commitment_grounds: { set: Record<string, Provenance>; removed: string[] };
  decision_series: { set: Record<string, DecisionSeries>; removed: string[] };
  /** Positions in the old list, ascending, then entries added at the end. */
  decision_journal: { removed: number[]; appended: DecisionJournalEntry[] };
  relations: { removed: number[]; appended: Relation[] };
  /** Present only in a program with an `unqualify`, as the view's member. */
  taken_back?: { removed: number[]; appended: TakeBack[] };
}

export interface QualifiedValue {
  value: number;
  provenance: Provenance;
}

export interface CommitmentBasis {
  value: number | null;
  provenance: Provenance;
}

export interface ReadingOccurrence {
  id: string;
  ordinal: number;
  sequence: number;
  event: string;
  /** The agent the host said sent the event; absent when it named none (spec/caveat-agents-0.1.md). */
  agent?: string;
  value: number;
  provenance: Provenance;
  /** Relation and claim are both absent for a neutral reading, otherwise both present. */
  relation?: 'supports' | 'opposes';
  claim?: string;
  /** The text a `sample … noting` kept with the reading; absent otherwise. */
  note?: string;
}

export interface ReadingStream {
  template: string;
  limit: number;
  current: string | null;
  occurrences: ReadingOccurrence[];
  selection_qualifications: Provenance;
}

export interface DecisionRevision {
  id: string;
  previous: string | null;
  ordinal: number;
  sequence: number;
  event: string;
  /** From 1.3 (spec/caveat-outcomes-0.1.md): how it turned out. Absent until a `resolve` records it. */
  outcome?: Outcome;
}

/** What `resolve` recorded on a revision (spec/caveat-outcomes-0.1.md). */
export interface Outcome {
  value: number;
  /** The evidence it was read from, in observation order. */
  because: string[];
  /** The revision's retained caveats that came true. Absent when none. */
  materialized?: string[];
  sequence: number;
  event: string;
  /** The value's, the evidence's and the guard's lineage, frozen. */
  lineage: Provenance;
}

export interface DecisionSeries {
  limit: number;
  current: string | null;
  revisions: DecisionRevision[];
  selection_qualifications: Provenance;
  /** From 1.2 (spec/caveat-holds-0.1.md): the reasons the current revision is on hold. Absent when it is not. */
  on_hold?: string[];
  /** From 1.2: the reopenings a hold defers, applied on release. Absent when there are none. */
  deferred?: string[];
}

export interface DecisionJournalEntry {
  decision: string;
  commitment: string;
  /** `held`, `deferred` and `released` from 1.2 (spec/caveat-holds-0.1.md); `resolved` from 1.3 (spec/caveat-outcomes-0.1.md). */
  change: 'committed' | 'reopened' | 'held' | 'deferred' | 'released' | 'resolved';
  sequence: number;
  event: string;
  /** The agent the host said sent the event; absent when it named none (spec/caveat-agents-0.1.md). */
  agent?: string;
  /** Absent in older saves. */
  elapsed?: number;
  /** The commitment's frozen `using` value, if it had one. */
  value?: number;
  because: string[];
  caveats: string[];
  /** For a commitment made with `permitted by`, its grant. */
  permitted_by?: string;
  /** For `resolved`, the outcome's value. */
  outcome?: number;
  /** For `resolved`, the retained caveats it says came true. Absent when none. */
  materialized?: string[];
}

export interface Renewal {
  limit: number;
  occurrences: string[];
}

export interface ScheduledQualification {
  evidence: string;
  caveat: string;
  scheduled_at: number;
  after: number;
  guard: Provenance;
}

export interface Withdrawal {
  evidence: string;
  because: string;
  sequence: number;
  event: string;
  /** The agent the host said sent the event; absent when it named none (spec/caveat-agents-0.1.md). */
  agent?: string;
}

/** One taken-back qualification: the occurrence whose live relation by `caveat` the event marked. */
export interface TakeBack {
  evidence: string;
  caveat: string;
  sequence: number;
  event: string;
  /** The agent the host said sent the event; absent when it named none (spec/caveat-agents-0.1.md). */
  agent?: string;
}

export interface PermissionRecord {
  grant: string;
  scope?: PermissionScope;
  caveats?: string[];
}

export interface PermissionScope {
  granted: number;
  required: number;
}

export interface EventSignature {
  name: string;
  parameters: EventParameter[];
}

export interface EventParameter {
  name: string;
  min: number;
  max: number;
  /** Absent for a numeric parameter. */
  domain?:
    | { entity: { kind: string; members: string[] } }
    | { member: { members: string[] } }
    | { identifier: { limit: number } };
}

export interface CueSound {
  kind: 'sound';
  id: string;
  frequency: number;
  duration: number;
  gain: number;
}

export interface CueToast {
  kind: 'toast';
  id: string;
  text: string;
  duration: number;
}

export interface CueFlash {
  kind: 'flash';
  id: string;
  target: string;
  duration: number;
}

export interface CueRing {
  kind: 'ring';
  id: string;
  target: string;
  color: number | string;
  duration: number;
}

export type Cue = CueSound | CueToast | CueFlash | CueRing;

export interface EffectSample {
  kind: 'sample';
  stream: string;
  id: string;
  value: number;
  /** Relation and target are both absent for a neutral sample, otherwise both present. */
  relation?: 'supports' | 'opposes';
  target?: string;
  /** The text a `sample … noting` kept with the reading; absent otherwise. */
  note?: string;
}

export interface EffectReveal {
  kind: 'reveal';
  evidence: string;
  /** Relation and target are both absent for neutral reveal, otherwise both present. */
  relation?: 'supports' | 'opposes';
  target?: string;
}

export interface EffectExamine {
  kind: 'examine';
  caveat: string;
  cost: number;
}

export interface EffectCommit {
  kind: 'commit';
  action: string;
  retained: string[];
}

export interface EffectReopen {
  kind: 'reopen';
  action: string;
  because: string;
}

export interface EffectQualify {
  kind: 'qualify';
  evidence: string;
  caveat: string;
}

/** The occurrences whose qualification a take-back marked, in occurrence order (spec/caveat-take-back-0.1.md). */
export interface EffectUnqualify {
  kind: 'unqualify';
  evidence: string;
  caveat: string;
  occurrences: string[];
}

export interface EffectRenew {
  kind: 'renew';
  evidence: string;
  occurrence: string;
}

export interface EffectWithdraw {
  kind: 'withdraw';
  evidence: string;
  because: string;
}

/** The oldest live record of a windowed history, retired before the effect that adds a record (spec/caveat-windows-0.1.md). */
export interface EffectRetire {
  kind: 'retire';
  history: string;
  record: string;
}

/** A retired record that nothing pins leaving the session, reported after every other effect (spec/caveat-departure-0.1.md). */
export interface EffectDepart {
  kind: 'depart';
  history: string;
  record: string;
}

export type Effect =
  | EffectSample
  | EffectReveal
  | EffectExamine
  | EffectCommit
  | EffectReopen
  | EffectQualify
  | EffectUnqualify
  | EffectRenew
  | EffectWithdraw
  | EffectRetire
  | EffectDepart;

export interface Control {
  event: string;
  reset: boolean;
}

export interface Clock {
  event: string;
  step: number;
  /** Present, and true, only for a clock declared `integer` (spec/caveat-elapsed-0.1.md#integer-clocks). */
  integer?: true;
}

export interface World {
  start_at: string | null;
  places: Place[];
  entities: Entity[];
  connections: Connection[];
  action_plans: JsonObject[];
  /** Absent when the program declares no presentation. */
  presentation?: JsonObject;
}

export interface Place {
  id: string;
  kind: string;
}

export interface Entity {
  id: string;
  kind: string;
  at: string;
}

export interface Connection {
  from: string;
  to: string;
  via: string | null;
}

export interface SnapshotSymbol {
  name: string;
  kind: string;
  source: string | null;
  written_by: string | null;
  consequence: string | null;
  display: string | null;
  attention: string | null;
}

export interface Relation {
  from: string;
  relation: string;
  to: string;
  origin: string;
}

export interface Commitment {
  action: string;
  /** True when the commitment was reopened and awaits a new decision; it does not mean active. */
  open: boolean;
  retained: string[];
  retained_authorship: RetainedCaveat[];
  reopened_by: string[];
}

export interface RetainedCaveat {
  caveat: string;
  written_by: string | null;
  written_elsewhere: boolean | null;
}

export interface Budget {
  initial: number;
  spent: number;
  remaining: number;
  exhausted: boolean;
}

/** One journal change to a decision revision. */
export interface DecisionChange {
  sequence: number;
  event: string;
  /** For a commit, the evidence it rests on; for a reopening, what reopened it. */
  because: string[];
  caveats: string[];
  /** The commitment's frozen `using` value, if it had one. */
  value?: number;
  permitted_by?: string;
}

/** One revision of a decision series, as `decisionHistory` reads it. */
export interface DecisionHistoryEntry {
  revision: string;
  /** Only the current revision is `in_force` or `reopened`; every earlier one is `superseded`. */
  status: 'in_force' | 'reopened' | 'superseded';
  /** True when a windowed journal retired the revision's commit entry; `committed` is then null. */
  retired: boolean;
  /** What it was committed on; null only when `retired`. */
  committed: DecisionChange | null;
  /** What reopened it, or null while it has not been reopened or once the journal retired that entry. */
  reopened: DecisionChange | null;
}

/**
 * Every revision of one decision series, oldest first, from a view or a
 * snapshot's journal. A revision whose commit entry a windowed journal retired
 * is still listed, with `retired: true`. Throws a `TypeError` for a name that
 * is not a decision series.
 */
export declare function decisionHistory(
  view: { decision_series: Record<string, DecisionSeries>; decision_journal: DecisionJournalEntry[]; commitments: Array<Commitment | ViewCommitment> },
  decision: string,
): DecisionHistoryEntry[];
