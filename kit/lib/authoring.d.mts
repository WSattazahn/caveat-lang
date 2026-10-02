// Internal inline-source operation contract for the local authoring bridge.
import type { RuntimeIdentity, Snapshot } from './session.mjs';

export declare const AUTHORING_SCHEMA: 'caveat-authoring/0.1';
export declare const AUTHORING_LIMITS: Readonly<Record<string, number>>;
export declare class AuthoringError extends Error {
  constructor(kind: 'input' | 'limit' | 'runtime', message: string);
  kind: 'input' | 'limit' | 'runtime';
}
export interface AuthoringResult {
  schema: 'caveat-authoring/0.1';
  operation: string;
  sourceSha256: string;
  runtime: RuntimeIdentity;
  exitCode: number;
  report: Record<string, unknown>;
}
export declare function validateAuthoringArguments(toolName: string, args: unknown): Record<string, unknown>;
export declare function describeValidation(snapshot: Snapshot, program?: string): Record<string, unknown>;
export declare function runAuthoringOperation(toolName: string, args: unknown, options?: { runtimeDirectory?: string }): Promise<AuthoringResult>;
