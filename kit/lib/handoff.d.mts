// Types for handoff.mjs: a session limited to one agent's events, and one
// agent's accepted events replayed onto another agent's session.
import type {
  CaveatSession,
  DispatchOptions,
  DispatchOutcome,
  DispatchRejected,
  DispatchViewDeltaOutcome,
  DispatchViewOutcome,
  JsonObject,
  SaveOptions,
  SessionState,
  Snapshot,
  View,
  ViewField,
  ViewSelection,
  ViewV2,
} from './session.mjs';

/** One event a session accepted, with the payload as it was sent. */
export interface AcceptedEvent {
  event: string;
  payload: JsonObject;
  /** The agent that sent it, for a scope made with one (spec/caveat-agents-0.1.md). */
  agent?: string;
}

/** `agent`: the declared agent every event of the scope is sent as. */
export interface ScopeOptions {
  agent?: string;
}

/**
 * A session that sends only its listed events. Any other event is refused as
 * `input/unknown_event` without reaching the session. Reads are not limited.
 */
export declare class ScopedSession {
  #private;
  constructor(session: CaveatSession, events: readonly string[], options?: ScopeOptions);
  /** The events this session may send, as listed. */
  readonly events: readonly string[];
  /** The agent this scope sends as, or null. */
  readonly agent: string | null;
  readonly state: SessionState;
  /** A fresh copy each read: the events sent through this scope that were accepted, in order. */
  readonly accepted: AcceptedEvent[];
  /** Options may repeat the scope's own agent, never name another: that throws `TypeError`. */
  dispatch(event: string, payload?: object, options?: DispatchOptions): DispatchOutcome;
  dispatchView(event: string, payload?: object, options?: DispatchOptions): DispatchViewOutcome;
  dispatchView(event: string, payload: object, options: DispatchOptions & { fields: readonly ViewField[] }): DispatchViewOutcome<ViewSelection>;
  dispatchViewDelta(event: string, payload?: object, options?: DispatchOptions): DispatchViewDeltaOutcome;
  /** What `dispatchView` would return through this scope, without sending the event. */
  whatIf(event: string, payload?: object, options?: DispatchOptions): DispatchViewOutcome;
  whatIf(event: string, payload: object, options: DispatchOptions & { fields: readonly ViewField[] }): DispatchViewOutcome<ViewSelection>;
  snapshot(): Snapshot;
  snapshotText(): string;
  view(options?: { schema?: '0.1' }): View;
  view(options: { schema: '0.2' }): ViewV2;
  viewText(options?: { schema?: '0.1' | '0.2' }): string;
  save(options?: SaveOptions): string;
}

/** A session for one agent. Throws `TypeError` for a list that is not distinct non-empty names. */
export declare function scopeSession(session: CaveatSession, events: readonly string[], options?: ScopeOptions): ScopedSession;

/** An event that did not apply when replayed: its position in the list and the refusal. */
export interface ReplayConflict {
  index: number;
  event: string;
  payload: JsonObject;
  agent?: string;
  outcome: DispatchRejected;
}

export interface ReplayResult {
  accepted: AcceptedEvent[];
  conflicts: ReplayConflict[];
}

/**
 * Sends each event to `session` in order, an event with an `agent` as that
 * agent. A refused event is a conflict and
 * changes nothing; the rest are still sent. A fatal outcome throws, as
 * `dispatch` does.
 */
export declare function replayOnto(session: Pick<CaveatSession, 'dispatch'> | ScopedSession, events: readonly AcceptedEvent[]): ReplayResult;
