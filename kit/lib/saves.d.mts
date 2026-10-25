// Types for saves.mjs: a replay save's events read back by name, and a save
// carried onto a changed program by replaying them.
import type { CaveatRuntime, CaveatSession } from './session.mjs';
import type { AcceptedEvent, ReplayConflict } from './handoff.mjs';

/**
 * The events a replay save holds, in order, each with a payload that names
 * its members, read with the source that wrote the save. Throws
 * `CaveatError("restore")` when the save does not restore with that source,
 * and when it is a state save, which holds no events.
 */
export declare function savedEvents(runtime: CaveatRuntime, source: string, saved: string): AcceptedEvent[];

export interface CarriedSave {
  /** A session of the new source, made by the save's events. Close it when done. */
  session: CaveatSession;
  /** The events the new program accepted, in order. */
  accepted: AcceptedEvent[];
  /** The events it refused; each changed nothing. */
  conflicts: ReplayConflict[];
}

/**
 * A session of `newSource` made by sending it the events of a save written
 * with `oldSource`. The save is not changed.
 */
export declare function carrySave(runtime: CaveatRuntime, oldSource: string, saved: string, newSource: string): CarriedSave;
