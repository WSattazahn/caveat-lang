// Types for scenarios.mjs: Scenarios 0.1 (spec/caveat-scenarios-0.1.md)
// validation, matching and the runner. No Node imports.
import type { CaveatRuntime, JsonValue, Origin, RejectionCode, RuntimeIdentity, Snapshot } from './session.mjs';

export declare const SCENARIO_SCHEMA: 'caveat-scenarios/0.1';
export declare const REPORT_SCHEMA: 'caveat-scenario-report/0.1';

/** An invalid scenario file. `where` names the file, a scenario or a step. */
export declare class ScenarioFileError extends Error {
  constructor(where: string, message: string);
  where: string;
}

// ------------------------------------------------------------- the file

export interface ScenarioFile {
  schema: 'caveat-scenarios/0.1';
  /** The program file, relative to the scenario file. */
  source: string;
  scenarios: Scenario[];
  note?: string;
}

export interface Scenario {
  id: string;
  title: string;
  /** This scenario's program, in place of the file's. */
  source?: string;
  steps: ScenarioStep[];
  note?: string;
}

export type ScenarioStep = SendStep | ExpectStep | SameAsStep | CheckpointStep | ResumeStep | SizeStep;

export interface SendStep {
  send: string;
  payload?: JsonValue;
  /** Left out, the event must be accepted. `true` or a message: a policy rejection. */
  rejected?: true | string | ExpectedRejection;
  repeat?: number;
  note?: string;
}

export interface ExpectedRejection {
  origin: Origin;
  code?: string;
  /** Only a policy rejection has a stable message. */
  message?: string;
}

/** JSON Pointers into the snapshot, each with a value or a matcher. */
export interface ExpectStep {
  expect: Record<string, JsonValue>;
  note?: string;
}

export interface SameAsStep {
  same_as: string;
  paths: string[];
  note?: string;
}

export interface CheckpointStep {
  checkpoint: string;
  note?: string;
}

export interface ResumeStep {
  resume: true;
  note?: string;
}

export interface SizeStep {
  size: SizeBounds;
  note?: string;
}

/** Bounds on the save, the snapshot or both, in bytes of compact JSON. */
export interface SizeBounds {
  save?: SizeMax;
  snapshot?: SizeMax | SizeGrowth;
}

export interface SizeMax {
  max: number;
}

export interface SizeGrowth {
  max_growth: number;
  since: string;
}

/** Validates a parsed scenario file and returns it. Throws ScenarioFileError. */
export declare function validateScenarioFile(doc: unknown): ScenarioFile;

/** Parses and validates scenario file text. Throws ScenarioFileError. */
export declare function parseScenarioFile(text: string): ScenarioFile;

/** Throws ScenarioFileError unless `pointer` is a JSON Pointer. */
export declare function checkPointer(pointer: unknown, where: string): asserts pointer is string;

// ------------------------------------------------------------- matching

/** Marks a missing value in failures. JSON writes it as `{"$absent": true}`. */
export declare const ABSENT: Readonly<{ toJSON: () => { $absent: true } }>;
export type Absent = typeof ABSENT;

/** Where two documents first differ; a missing value is ABSENT. */
export interface Difference {
  path: string;
  expected: unknown;
  actual: unknown;
}

export declare function getPointer(document: unknown, pointer: string): { found: true; value: unknown } | { found: false };

/** JSON text with object keys sorted. */
export declare function canonical(value: unknown): string;

/** Whether two JSON values are equal, whatever the order of their keys. */
export declare function same(a: unknown, b: unknown): boolean;

/** The first JSON Pointer at which two documents differ, or null. */
export declare function firstDifference(a: unknown, b: unknown, at?: string): Difference | null;

/** Null when `actual` matches `expected`, which may use the $exact, $set, $includes and $absent matchers. */
export declare function match(actual: unknown, expected: unknown, at?: string): Difference | null;

/** `match` at a JSON Pointer of `document`. */
export declare function expectAt(document: unknown, pointer: string, expected: unknown): Difference | null;

export interface GroundsViolation {
  path: string;
  item: string;
  message: string;
}

/** A value or commitment grounded on a name outside its lineage, or null. */
export declare function groundsViolation(snapshot: Snapshot): GroundsViolation | null;

// ------------------------------------------------------------- the runner

export interface RunScenarioOptions {
  runtime: CaveatRuntime;
  /** A fresh runtime after a WebAssembly trap. */
  reload?: () => Promise<CaveatRuntime>;
  /** The program text for a path relative to the file. */
  readSource: (path: string) => string | Promise<string>;
  /** The file's name, for the report. */
  file?: string | null;
}

export interface ScenarioFileReport {
  file: string | null;
  /** The program paths read, in order. */
  sources: string[];
  scenarios: ScenarioResult[];
  passed: number;
  failed: number;
}

export interface ScenarioResult {
  id: string;
  title: string;
  pass: boolean;
  events: number;
  rejected: number;
  resumes: number;
  /** Present when the scenario failed. */
  failure?: ScenarioFailure;
}

export type FailureKind =
  | 'load'
  | 'fatal'
  | 'send'
  | 'expect'
  | 'same_as'
  | 'size'
  | 'atomicity'
  | 'agreement'
  | 'grounds'
  | 'resume'
  | 'final-resume';

export type FailureCategory = 'expectation' | 'runtime-invariant' | 'fatal' | 'load';

/** Which fields a failure has depends on its kind; failureText reads them. */
export interface ScenarioFailure {
  /** One-based; 0 when the source could not be read, and one past the last step for the final restore. */
  step: number;
  kind: FailureKind;
  category: FailureCategory;
  /** `primary`, `shadow N` or `final restore`; null for a trap while releasing sessions. */
  session?: string | null;
  repetition?: number;
  path?: string;
  expected?: unknown;
  actual?: unknown;
  outcome?: FailureOutcome;
  message?: string;
  /** For same_as: the stored state compared with. */
  from?: string;
  /** For size: the limit and the measured bytes. */
  limit?: number;
  bytes?: number;
}

export type FailureOutcome =
  | { outcome: 'accepted' }
  | { outcome: 'rejected'; origin: Origin; code: RejectionCode; message: string }
  | { outcome: 'fatal'; code: string | null; message: string };

/** Runs every scenario of a file; a failing one does not stop the rest. */
export declare function runScenarioFile(doc: ScenarioFile, options: RunScenarioOptions): Promise<ScenarioFileReport>;

export interface ScenarioRunReport {
  schema: 'caveat-scenario-report/0.1';
  dispatchSchema: 'caveat-dispatch/0.1';
  runtime: Readonly<RuntimeIdentity>;
  files: ScenarioFileReport[];
  passed: number;
  failed: number;
}

/** The report of a run over several files, as `caveat test --json` prints it. */
export declare function report(files: ScenarioFileReport[], runtimeIdentity?: Readonly<RuntimeIdentity>): ScenarioRunReport;

/** One failure as text. */
export declare function failureText(failure: ScenarioFailure): string;

/** A file's PASS and FAIL lines and its totals. */
export declare function formatFileReport(fileReport: ScenarioFileReport): string;
