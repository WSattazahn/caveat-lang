// Types for handoff.mjs: a session limited to one agent's events, and one
// agent's accepted events replayed onto another agent's session.
import type {
  CaveatSession,
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
}

/**
 * A session that sends only its listed events. Any other event is refused as
 * `input/unknown_event` without reaching the session. Reads are not limited.
 */
export declare class ScopedSession {
  #private;
  constructor(session: CaveatSession, events: readonly string[]);
  /** The events this session may send, as listed. */
  readonly events: readonly string[];
  readonly state: SessionState;
  /** A fresh copy each read: the events sent through this scope that were accepted, in order. */
  readonly accepted: AcceptedEvent[];
  dispatch(event: string, payload?: object): DispatchOutcome;
  dispatchView(event: string, payload?: object): DispatchViewOutcome;
  dispatchView(event: string, payload: object, options: { fields: readonly ViewField[] }): DispatchViewOutcome<ViewSelection>;
  dispatchViewDelta(event: string, payload?: object): DispatchViewDeltaOutcome;
  snapshot(): Snapshot;
  snapshotText(): string;
  view(options?: { schema?: '0.1' }): View;
  view(options: { schema: '0.2' }): ViewV2;
  viewText(options?: { schema?: '0.1' | '0.2' }): string;
  save(options?: SaveOptions): string;
}

/** A session for one agent. Throws `TypeError` for a list that is not distinct non-empty names. */
export declare function scopeSession(session: CaveatSession, events: readonly string[]): ScopedSession;

/** An event that did not apply when replayed: its position in the list and the refusal. */
export interface ReplayConflict {
  index: number;
  event: string;
  payload: JsonObject;
  outcome: DispatchRejected;
}

export interface ReplayResult {
  accepted: AcceptedEvent[];
  conflicts: ReplayConflict[];
}

/**
 * Sends each event to `session` in order. A refused event is a conflict and
 * changes nothing; the rest are still sent. A fatal outcome throws, as
 * `dispatch` does.
 */
export declare function replayOnto(session: Pick<CaveatSession, 'dispatch'> | ScopedSession, events: readonly AcceptedEvent[]): ReplayResult;
