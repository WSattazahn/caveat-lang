// Types for serve.mjs: one program's session behind the line protocol of
// spec/caveat-serve-0.1.md. The module does no I/O.
import type { DependentsReport, ExplainReport } from './explain.mjs';
import type {
  ArchiveEntry,
  CaveatErrorKind,
  CaveatRuntime,
  JsonValue,
  Origin,
  RejectionCode,
  RuntimeIdentity,
  Snapshot,
  View,
} from './session.mjs';

export declare const SERVE_SCHEMA: 'caveat-serve/0.1';

export interface ServeOptions {
  runtime: CaveatRuntime;
  source: string;
  /** The program's name, as `ready` reports it. */
  program?: string | null;
  /**
   * The text of an earlier save to start from instead of opening the program
   * fresh. A save the runtime cannot use throws CaveatError("restore").
   */
  restore?: string | null;
}

/** The first line to write. */
export interface ServeReady {
  schema: 'caveat-serve/0.1';
  ready: true;
  program: string | null;
  runtime: Readonly<RuntimeIdentity>;
}

/**
 * One response. `ok` says which fields it has: the operation's results, or
 * `error`. `id` echoes the request's, or is null.
 */
export interface ServeResponse {
  id: JsonValue;
  ok: boolean;
  outcome?: 'accepted' | 'rejected';
  origin?: Origin;
  code?: RejectionCode;
  message?: string;
  sequence?: number;
  snapshot?: Snapshot;
  /** From `view`, or from `dispatch` with `view: true`: the View 0.1 of the session. */
  view?: View;
  report?: ExplainReport | DependentsReport;
  save?: string;
  /** Host-owned archive items removed from the runtime by drainArchive. */
  archive?: ArchiveEntry[];
  /** Number of pending record entries plus provenance nodes. */
  undrained?: number;
  error?: ServeError;
}

export interface ServeError {
  /** `request` for a malformed request, which changes nothing; otherwise the CaveatError's kind. */
  kind: 'request' | CaveatErrorKind;
  message: string;
}

/**
 * Write `response`; then stop with `exit` as the status when it is a number.
 * `changed` is true when the request changed the session (an accepted event or
 * a restore): a host that keeps a state file rewrites it then.
 */
export interface ServeResult {
  response: ServeResponse;
  exit: 0 | 1 | null;
  changed: boolean;
}

/** A state file's text: the compact save where the runtime has one, else the full save. */
export interface ServeSave {
  text: string;
  compact: boolean;
}

export interface CaveatServer {
  ready: ServeReady;
  handle(line: string): ServeResult;
  save(): ServeSave;
  close(): void;
}

/** Opening the session throws CaveatError("load") when the program does not load. */
export declare function createServer(options: ServeOptions): CaveatServer;
