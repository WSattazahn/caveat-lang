// Types for ledgers.mjs: reviewing a set of past ledgers, and the past
// reviews a program says it consulted.
import type { CaveatSession } from './session.mjs';
import type { Review, ReviewEvent } from './review.mjs';

export declare const LEDGER_REVIEW_SCHEMA: 'caveat-ledger-review/0.1';
/** `caveat-review:<program sha256>#<revision>`, as an evidence source. */
export declare const CONSULTED_SOURCE: RegExp;

/** Evidence whose source names a past review. */
export interface ConsultedSource {
  evidence: string;
  /** The reviewed program's source SHA-256, lowercase hex. */
  program_sha256: string;
  /** A revision such as refund@1. */
  revision: string;
}

/** A past review observed before the deciding event. */
export interface Consulted extends ConsultedSource {
  /** True when the evidence is in the revision's lineage. */
  in_lineage: boolean;
  /** Only from reviewLedgers: the ledger with that source, or null. */
  found?: { program: string; events: string } | null;
  /** Only from reviewLedgers: what that revision's review says came true, or null when the set holds no review of it. */
  came_true?: { caveat: string; decisive: boolean }[] | null;
}

export interface ReviewWithConsulted extends Review {
  consulted: Consulted[];
}

export interface Ledger {
  program: string;
  events_file: string;
  source: string;
  events: readonly ReviewEvent[];
}

export type LedgerReview = Omit<Review, 'schema'> & {
  schema: 'caveat-ledger-review/0.1';
  program: string;
  events: string;
  program_sha256: string;
  consulted: Consulted[];
};

export interface LedgerError {
  schema: 'caveat-ledger-review/0.1';
  program: string;
  events: string;
  error: string;
}

/** The evidence in the snapshot's program whose source names a past review. */
export declare function consultedSources(snapshot: { symbols?: readonly { name: string; kind: string; source?: string | null }[] }): ConsultedSource[];

/** Reviews `revision` as review() does, adding the past reviews it consulted. */
export declare function reviewWithConsulted(
  open: () => CaveatSession,
  events: readonly ReviewEvent[],
  revision: string,
  options?: { limit?: number },
): ReviewWithConsulted;

/**
 * Reviews every revision that has an outcome or consulted a past review, in
 * each ledger, resolving consulted reviews against the set.
 */
export declare function reviewLedgers(
  open: (source: string) => CaveatSession,
  ledgers: readonly Ledger[],
  options?: { limit?: number },
): (LedgerReview | LedgerError)[];

export interface CaveatCount {
  retained: number;
  came_true: number;
}

export interface LedgerSummary {
  schema: 'caveat-ledger-summary/0.1';
  revisions: number;
  caveats: (CaveatCount & { caveat: string; by_evidence: (CaveatCount & { evidence: string })[] })[];
}

/** How often each caveat came true across the reviewed revisions with an outcome. */
export declare function summarizeLedgers(entries: readonly (LedgerReview | LedgerError)[]): LedgerSummary;

/** Text lines for a summary; none when no caveat was retained. */
export declare function formatSummary(summary: LedgerSummary): string[];

/** Text lines for what a review consulted. */
export declare function formatConsulted(consulted: readonly Consulted[]): string[];
