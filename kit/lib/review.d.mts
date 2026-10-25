// Types for review.mjs: which evidence a decision hinged on, found by
// replaying its events without the events that observed its grounds.
import type { CaveatSession, Outcome, Withdrawal } from './session.mjs';
import type { RevisionStatus } from './explain.mjs';

export declare const REVIEW_SCHEMA: 'caveat-review/0.1';
/** Every combination of observing events is replayed up to this many. */
export declare const EXHAUSTIVE_LIMIT: 8;

export interface ReviewEvent {
  event: string;
  payload?: object;
  /** The agent that sent it; replays send it as that agent. */
  agent?: string;
}

/** An event left out of a replay, by its sequence in the full run. */
export interface ReviewDropped {
  sequence: number;
  event: string;
  /** The agent that sent it, when it named one (spec/caveat-agents-0.1.md). */
  agent?: string;
}

/** Why the deciding event committed nothing in a replay. */
export type ReviewFailure =
  | { event: string; outcome: 'accepted'; message: string }
  | { event: string; outcome: 'rejected'; origin: string | null; code: string | null; message: string | null }
  | { event: string | null; outcome: 'fatal' | 'not_sent'; message: string };

export interface ReviewDecisive {
  /** The grounds evidence the dropped events observed. */
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
  /** Grounds evidence that carries the caveat now. */
  carried_by: string[];
  /** True when some carrier is in a decisive set. */
  decisive: boolean;
}

export interface Review {
  schema: 'caveat-review/0.1';
  revision: string;
  value: number | null;
  status: RevisionStatus;
  decided_by: { sequence: number; event: string; agent?: string };
  grounds: { evidence: string; caveats: string[] }[];
  retained: string[];
  outcome: Outcome | null;
  /** Smallest sets of grounds without which the deciding event commits nothing. */
  decisive: ReviewDecisive[];
  /** Smallest sets without which it commits another value, and no decisive set is inside. */
  value_only: ReviewValueOnly[];
  /** Grounds in no decisive or value-only set. */
  not_decisive: string[];
  /** Grounds observed by the deciding event itself; they cannot be dropped apart from it. */
  entangled: string[];
  /** Grounds no event given observed. */
  unobserved: string[];
  /** The outcome's materialized caveats. */
  came_true: ReviewCameTrue[];
  withdrawn: Withdrawal[];
  /** Evidence observed before the deciding event and not in its grounds. */
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
