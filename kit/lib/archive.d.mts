// Internal pure helper used by explain/dependents; not a package export.
import type { ArchiveEntry, ArchiveProvenanceNode, DepartureMarker, Snapshot } from './session.mjs';
import type { ArchivedHistory } from './explain.mjs';

/** Synchronous UTF-8 SHA-256 for browser-compatible archive consistency checks. */
export declare function archiveSha256(text: string): string;
/** Canonical content digest of a provenance node, ignoring its id field. */
export declare function archiveNodeId(node: ArchiveProvenanceNode): string;
/** Returns exact names for complete consistent closure, or null conservatively. */
export declare function archiveResolver(snapshot: Snapshot, archive?: ArchiveEntry[]): (marker: DepartureMarker) => string[] | null;
/** Internal draft historical report builder; never mutates or replaces snapshot data. */
export declare function archiveHistory(snapshot: Snapshot, archive?: ArchiveEntry[], required?: (string | DepartureMarker)[],
  query?: {kind: 'evidence' | 'caveat'; names: Set<string>; histories?: Set<string>} | null): ArchivedHistory | null;
