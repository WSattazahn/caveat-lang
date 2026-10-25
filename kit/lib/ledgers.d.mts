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

/** Text lines for what a review consulted. */
export declare function formatConsulted(consulted: readonly Consulted[]): string[];
