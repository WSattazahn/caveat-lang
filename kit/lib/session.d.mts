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
  | 'scheduled_limit';
export type RejectionCode = KnownRejectionCode | (string & {});

/** An accepted event, from `dispatch`: the full snapshot after it. */
export interface DispatchAccepted {
  schema: 'caveat-dispatch/0.1';
  outcome: 'accepted';
  snapshot: Snapshot;
}

/** An accepted event, from `dispatchView`: the view after it. */
export interface DispatchViewAccepted {
  schema: 'caveat-dispatch/0.1';
  outcome: 'accepted';
  view: View;
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
export type DispatchViewOutcome = DispatchViewAccepted | DispatchRejected;

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
  dispatch(event: string, payload?: object): DispatchOutcome;
  /** The same transaction as `dispatch`, with the view in place of the snapshot. */
  dispatchView(event: string, payload?: object): DispatchViewOutcome;
  snapshotText(): string;
  snapshot(): Snapshot;
  viewText(): string;
  view(): View;
  /** JSON text. Restore it with the exact source it came from. */
  save(): string;
  close(): void;
}

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
  /** Throws `CaveatError("load")` when the source does not load. */
  open(source: string): CaveatSession;
  /** Advisory warnings for a program that loads; throws as `open` does when it does not. */
  check(source: string): CheckReport;
  /** Throws `CaveatError("restore")` when the save does not restore with this source. */
  restore(source: string, saved: string): CaveatSession;
}

/** A session object of the runtime: `WebReactiveSession`, or a stand-in with its methods. */
export interface RuntimeSessionHandle {
  dispatch_outcome(event: string, payload: string): string;
  dispatch_view_outcome?(event: string, payload: string): string;
  snapshot(): string;
  view(): string;
  save(): string;
  free(): void;
}

/** The runtime's session class: `WebReactiveSession`, or a stand-in. */
export interface RuntimeSessionClass {
  new (source: string): RuntimeSessionHandle;
  restore(source: string, saved: string): RuntimeSessionHandle;
  check?(source: string): string;
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

// ------------------------------------------------------------- check report

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
}

export type BindingValue = number | boolean | string;
/** Displayed values, by target and property. */
export type Bindings = Record<string, Record<string, BindingValue>>;

/** Everything the session knows. Large: a host that redraws reads `view()`. */
export interface Snapshot {
  schema: 'caveat-reactive/0.1';
  source_id: string;
  sequence: number;
  elapsed: number;
  renewals: Record<string, Renewal>;
  /** Texts received for `id` parameters, in handle order; absent when there are none. */
  identifiers?: string[];
  /** Withdrawn observations, in order; absent when there are none. */
  withdrawals?: Withdrawal[];
  /** What each permitted commitment was permitted by; absent when there are none. */
  commitment_permissions?: Record<string, PermissionRecord>;
  scheduled_qualifications: ScheduledQualification[];
  last_event: string | null;
  values: Record<string, number>;
  qualified_values: Record<string, QualifiedValue>;
  commitment_bases: Record<string, CommitmentBasis>;
  reading_streams: Record<string, ReadingStream>;
  decision_series: Record<string, DecisionSeries>;
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
  value: number;
  provenance: Provenance;
  relation: string;
  claim: string;
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
}

export interface DecisionSeries {
  limit: number;
  current: string | null;
  revisions: DecisionRevision[];
  selection_qualifications: Provenance;
}

export interface DecisionJournalEntry {
  decision: string;
  commitment: string;
  change: 'committed' | 'reopened';
  sequence: number;
  event: string;
  /** Absent in older saves. */
  elapsed?: number;
  /** The commitment's frozen `using` value, if it had one. */
  value?: number;
  because: string[];
  caveats: string[];
  /** For a commitment made with `permitted by`, its grant. */
  permitted_by?: string;
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
  relation: string;
  target: string;
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

export type Effect =
  | EffectSample
  | EffectReveal
  | EffectExamine
  | EffectCommit
  | EffectReopen
  | EffectQualify
  | EffectRenew
  | EffectWithdraw;

export interface Control {
  event: string;
  reset: boolean;
}

export interface Clock {
  event: string;
  step: number;
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
