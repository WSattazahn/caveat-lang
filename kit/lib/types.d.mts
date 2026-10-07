// Types for types.mjs: declarations for one program from its interface
// (spec/caveat-interface-0.1.md), and the typed session they describe.
import type {
  ArchiveEntry, BindingValue, CaveatSession, DispatchOutcome, DispatchRejected, DispatchViewAccepted, ProgramInterface, Provenance, SessionState, Snapshot, View,
} from './session.mjs';

export declare const INTERFACE_SCHEMA: 'caveat-interface/0.1';

/** What `caveat types` declares as `Program`: each event's payload, and the bindings. */
export interface ProgramTypes {
  events: { [event: string]: object };
  bindings: { [target: string]: { [property: string]: BindingValue | undefined } };
}

/** An event's payload argument, optional when the event takes no fields. */
export type PayloadArgument<T extends ProgramTypes, K extends keyof T['events']> =
  {} extends T['events'][K] ? [payload?: T['events'][K]] : [payload: T['events'][K]];

/** The view, with the program's bindings and what each one cites. */
export type TypedView<T extends ProgramTypes> = Omit<View, 'bindings' | 'binding_explanations'> & {
  bindings: T['bindings'];
  binding_explanations: { [K in keyof T['bindings']]?: { [P in keyof T['bindings'][K]]?: Provenance } };
};

export type TypedDispatchViewOutcome<T extends ProgramTypes> =
  (Omit<DispatchViewAccepted, 'view'> & { view: TypedView<T> }) | DispatchRejected;

/** A `CaveatSession` whose events, payloads and bindings are the program's. */
export interface TypedSession<T extends ProgramTypes> {
  readonly state: SessionState;
  dispatch: <K extends keyof T['events'] & string>(event: K, ...payload: PayloadArgument<T, K>) => DispatchOutcome;
  dispatchView: <K extends keyof T['events'] & string>(event: K, ...payload: PayloadArgument<T, K>) => TypedDispatchViewOutcome<T>;
  snapshotText: () => string;
  snapshot: () => Snapshot;
  viewText: () => string;
  view: () => TypedView<T>;
  save: () => string;
  drainArchive: () => ArchiveEntry[];
  readonly undrained: number;
  close: () => void;
}

export interface DeclarationOptions {
  /** Names the program in the header. */
  program?: string;
  /** The module the declarations import `TypedSession` from; `caveat-lang/types` by default. */
  from?: string;
}

/** The text of a .d.ts module for the program. Throws for anything but a caveat-interface/0.1 interface. */
export declare function declarations(programInterface: ProgramInterface, options?: DeclarationOptions): string;

/** The session itself; its type is the program's when the declarations give `T`. */
export declare function typed<T extends ProgramTypes>(session: CaveatSession): TypedSession<T>;
