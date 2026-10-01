// Types for node.mjs: the session library's runtime, loaded from a directory.
import type { CaveatRuntime } from './session.mjs';

export type {
  CaveatError,
  CaveatErrorKind,
  CaveatRuntime,
  CaveatSession,
  CheckReport,
  DispatchAccepted,
  DispatchOutcome,
  DispatchRejected,
  DispatchViewAccepted,
  DispatchViewOutcome,
  RuntimeIdentity,
  Snapshot,
  View,
} from './session.mjs';

/** The package's own runtime/, or in a repository checkout dist/pkg-reactive. */
export declare function defaultRuntimeDirectory(): string;

/**
 * Loads caveat_runtime.js and caveat_runtime_bg.wasm from `directory`. Every
 * call gives a fresh instance, so a runtime that trapped can be replaced.
 */
export declare function loadRuntimeFromDirectory(directory?: string): Promise<CaveatRuntime>;
