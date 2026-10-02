// CLI-internal diagnostics. This module is not a package export.
import type { RuntimeIdentity } from './session.mjs';

export interface DoctorCheck {
  id: string;
  status: 'pass' | 'warn' | 'fail';
  message: string;
}
export interface DoctorExecutable {
  name: string;
  path: string | null;
  target: string | null;
  owner: { name: string; version: string; directory: string } | null;
  candidates: string[];
  error?: string;
}
export interface DoctorOptions {
  runtimeDirectory?: string;
  packageDirectory?: string;
  nodeVersion?: string;
  env?: Record<string, string | undefined>;
  cwd?: string;
}
export interface DoctorReport {
  schema: 'caveat-doctor/0.1';
  ok: boolean;
  node: { version: string; required: string; executable: string };
  package: { directory: string; name: string | null; version: string | null };
  runtime: {
    directory: string;
    files: Array<{ name: string; bytes: number; sha256: string }>;
    buildInfo: { revision: string; clean: boolean; compiled: boolean; host: string; [key: string]: unknown } | null;
    identity: RuntimeIdentity | null;
    session: { accepted: true; rejectedWithoutChange: true; exactGrounds: true; restored: true } | null;
  };
  resolution: string;
  executables: DoctorExecutable[];
  checks: DoctorCheck[];
}
export declare function inspectExecutables(options?: {
  env?: Record<string, string | undefined>;
  cwd?: string;
  platform?: string;
}): Promise<DoctorExecutable[]>;
export declare function doctor(options?: DoctorOptions): Promise<DoctorReport>;
export declare function formatDoctor(result: DoctorReport): string;
