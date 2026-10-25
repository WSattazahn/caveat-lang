// Types for review.mjs: which evidence a decision hinged on, found by
// replaying its events without the events that observed its lineage.
import type { CaveatSession, Outcome, Withdrawal } from './session.mjs';
import type { RevisionStatus } from './explain.mjs';

export declare const REVIEW_SCHEMA: 'caveat-review/0.1';
/** Every combination of observing events is replayed up to this many. */
export declare const EXHAUSTIVE_LIMIT: 8;

export interface ReviewEvent {
  event: string;
  payload?: object;
}

/** An event left out of a replay, by its sequence in the full run. */
export interface ReviewDropped {
  sequence: number;
  event: string;
}

/** Why the deciding event committed nothing in a replay. */
export type ReviewFailure =
  | { event: string; outcome: 'accepted'; message: string }
  | { event: string; outcome: 'rejected'; origin: string | null; code: string | null; message: string | null }
  | { event: string | null; outcome: 'fatal' | 'not_sent'; message: string };

export interface ReviewDecisive {
  /** The grounds or control evidence the dropped events observed. */
  evidence: string[];
  without: ReviewDropped[];
  because: ReviewFailure;
  /** The caveats that evidence carries now. */
  caveats: string[];
}

export interface ReviewValueOnly {
  evidence: string[];
  without: ReviewDropped[];
  /** The value the deciding event commits without them. */
  value: number | null;
  caveats: string[];
}

export interface ReviewCameTrue {
  caveat: string;
  /** Grounds or control evidence that carries the caveat now. */
  carried_by: string[];
  /** True when some carrier is in a decisive set. */
  decisive: boolean;
}

export interface Review {
  schema: 'caveat-review/0.1';
  revision: string;
  value: number | null;
  status: RevisionStatus;
  decided_by: { sequence: number; event: string };
  grounds: { evidence: string; caveats: string[] }[];
  /**
   * Lineage evidence outside the grounds: read by the deciding rule's `when`
   * guard, a permission grant or a predecessor's basis. Replayed like grounds.
   */
  control: string[];
  retained: string[];
  outcome: Outcome | null;
  /** Smallest sets of grounds or control without which the deciding event commits nothing. */
  decisive: ReviewDecisive[];
  /** Smallest sets without which it commits another value, and no decisive set is inside. */
  value_only: ReviewValueOnly[];
  /** Grounds or control in no decisive or value-only set. */
  not_decisive: string[];
  /** Grounds or control observed by the deciding event itself; they cannot be dropped apart from it. */
  entangled: string[];
  /** Grounds or control no event given observed. */
  unobserved: string[];
  /** The outcome's materialized caveats. */
  came_true: ReviewCameTrue[];
  withdrawn: Withdrawal[];
  /** Evidence observed before the deciding event and in neither its grounds nor its control. */
  outside_grounds: string[];
  /** The message of the fatal event that ended the run, or null. */
  fatal: string | null;
  search: { mode: 'exhaustive' | 'single'; units: number; replays: number; limit: number };
}

/**
 * Reviews `revision` of the session made by sending `events` to `open()`.
 * `open` must return a fresh session of the same program on each call.
 * Throws when there is no such revision of a decision series.
 */
export declare function review(
  open: () => CaveatSession,
  events: readonly ReviewEvent[],
  revision: string,
  options?: { limit?: number },
): Review;

/** The review as text for a person. `title` names the program. */
export declare function formatReview(report: Review, title?: string): string;
