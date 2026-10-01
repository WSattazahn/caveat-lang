// Types for check.mjs: text for a caveat-check/0.1 report (spec/caveat-check-0.1.md).
import type { CheckReport } from './session.mjs';

export type { CheckDiagnostic, CheckRelated, CheckReport } from './session.mjs';

export declare const CHECK_SCHEMA: 'caveat-check/0.1';

/** Each warning with where it is, what was found and what to consider; `name` names the program. */
export declare function formatCheck(report: CheckReport, name: string): string;
