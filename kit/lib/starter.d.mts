// Types for starter.mjs: one program's session behind a host that
// checkpoints every accepted event and keeps the archive beside it.
import type { ArchiveEntry, CaveatErrorKind, CaveatRuntime, Origin, RejectionCode, RuntimeIdentity, Snapshot, View } from './session.mjs';
import type { PayloadArgument, ProgramTypes, TypedView } from './types.mjs';

export declare const CHECKPOINT_SCHEMA: 'caveat-starter-checkpoint/0.1';

/** SHA-256 of the exact source text. It identifies the text; it authenticates nothing. */
export declare function sourceDigest(source: string): string;

/**
 * `store`: the store failed or holds something the host cannot use.
 * `source`: the checkpoint was made under other source text.
 * `unknown`: not a decision of the program. `fatal`: the session is unusable.
 * `closed`: the host was closed.
 */
export type StarterErrorKind = 'store' | 'source' | 'unknown' | 'fatal' | 'closed';

export declare class StarterError extends Error {
  constructor(kind: StarterErrorKind, message: string);
  readonly kind: StarterErrorKind;
}

/** Archive items drained after one accepted event, numbered from 0. */
export interface ArchiveChunk {
  number: number;
  sequence: number;
  /** SHA-256 of the entries' JSON text: a consistency check, not authentication. */
  sha256: string;
  entries: ArchiveEntry[];
}

/** What the store keeps after every accepted event. */
export interface Checkpoint {
  schema: 'caveat-starter-checkpoint/0.1';
  /** SHA-256 of the source text the session runs from. */
  source_sha256: string;
  runtime: Readonly<RuntimeIdentity>;
  sequence: number;
  save: string;
  next_chunk: number;
  /** Chunks not yet appended to the archive when the checkpoint was written. */
  pending: ArchiveChunk[];
}

/** What a host persists through. Each write resolves only once it is as durable as the store can make it. */
export interface StarterStore {
  /** False for a store that keeps nothing past the process. */
  readonly durable?: boolean;
  readCheckpoint(): Promise<string | null>;
  /** Replaces the checkpoint as a whole or not at all. */
  writeCheckpoint(text: string): Promise<void>;
  readChunks(): Promise<ArchiveChunk[]>;
  /** Appends in order. The same chunk may be appended again after a failure. */
  appendChunks(chunks: ArchiveChunk[]): Promise<void>;
  close?(): Promise<void>;
}

/** Persistence of a change: `durable` once its checkpoint is written, `archived` once no chunk waits. */
export interface Persisted {
  durable: boolean;
  archived: boolean;
  /** The store's error, when a write failed. Nothing pending was discarded. */
  storeError?: string;
}

/** The request was not handled: the payload cannot be sent, or the session failed. Nothing changed. */
export interface SendUnhandled {
  handled: false;
  accepted: false;
  durable: boolean;
  error: { kind: CaveatErrorKind; message: string };
}

/** The event was refused. The session is unchanged. */
export interface SendRejected {
  handled: true;
  accepted: false;
  durable: boolean;
  rejection: { origin: Origin; code: RejectionCode; message: string };
  sequence: number;
}

/** The event was accepted. Act on it only when `durable`, and only as the current assessment permits. */
export interface SendAccepted<V = View> extends Persisted {
  handled: true;
  accepted: true;
  view: V;
  sequence: number;
}

export type SendResult<V = View> = SendUnhandled | SendRejected | SendAccepted<V>;

export type ResyncResult<V = View> = { resync: false; sequence: number } | { resync: true; sequence: number; view: V };

/** The decision's last journal change in the current view, with the source it was made under. */
export interface Assessment {
  decision: string;
  status: 'none' | 'in_force' | 'reopened';
  /** The session's sequence when the assessment was read. */
  sequence: number;
  revision?: string | null;
  change?: {
    change: 'committed' | 'reopened';
    sequence: number;
    event: string;
    because: string[];
    caveats: string[];
    value?: number;
    permitted_by?: string;
  };
  /** The source digest the revision was committed under: the session's own, since a save restores only under its source. */
  source_sha256?: string;
}

export interface CaveatHost<V = View> {
  readonly state: 'open' | 'closed' | 'fatal';
  readonly sourceSha256: string;
  readonly sequence: number;
  readonly durable: boolean;
  readonly pendingChunks: number;
  send(event: string, payload?: object): Promise<SendResult<V>>;
  flush(): Promise<Persisted>;
  view(): V;
  snapshot(): Snapshot;
  resync(clientSequence: number | null): ResyncResult<V>;
  assessment(decision: string): Assessment;
  permits(decision: string): boolean;
  archive(): Promise<ArchiveEntry[]>;
  close(): Promise<void>;
}

/** A host whose events, payloads and bindings are the program's, from `caveat types`. */
export interface TypedHost<T extends ProgramTypes> extends Omit<CaveatHost<TypedView<T>>, 'send'> {
  send: <K extends keyof T['events'] & string>(event: K, ...payload: PayloadArgument<T, K>) => Promise<SendResult<TypedView<T>>>;
}

export interface OpenHostOptions {
  runtime: CaveatRuntime;
  source: string;
  store: StarterStore;
}

export declare function openHost<T extends ProgramTypes = ProgramTypes>(options: OpenHostOptions): Promise<TypedHost<T>>;

/** Keeps everything in memory. Not durable. */
export declare function memoryStore(): StarterStore;

/** A browser store in IndexedDB; how durable a completed transaction is depends on the browser. */
export declare function indexedDbStore(name: string, options?: { indexedDB?: unknown }): Promise<StarterStore>;
