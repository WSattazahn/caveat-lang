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
} from './session.mjs';

export declare const SERVE_SCHEMA: 'caveat-serve/0.1';

export interface ServeOptions {
  runtime: CaveatRuntime;
  source: string;
  /** The program's name, as `ready` reports it. */
  program?: string | null;
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

/** Write `response`; then stop with `exit` as the status when it is a number. */
export interface ServeResult {
  response: ServeResponse;
  exit: 0 | 1 | null;
}

export interface CaveatServer {
  ready: ServeReady;
  handle(line: string): ServeResult;
  close(): void;
}

/** Opening the session throws CaveatError("load") when the program does not load. */
export declare function createServer(options: ServeOptions): CaveatServer;
