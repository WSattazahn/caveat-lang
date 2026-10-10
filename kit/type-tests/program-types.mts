// Compiled by runtime.yml: declarations from `caveat types` (gauge.d.ts, made
// from gauge.cav; kit/test/types-command.test.mjs keeps it current).
// Local: npm exec --yes --package=typescript@5.9.3 -- tsc
// --noEmit --strict --module nodenext --target es2022
// --lib ES2022,ESNext.Disposable,DOM kit/type-tests/program-types.mts
import { typed } from '../lib/types.mjs';
import type { ArchiveEntry, CaveatRuntime } from '../lib/session.mjs';
import { dependents, explain } from '../lib/explain.mjs';
import type { Program, ProgramEvent, Session, View } from './gauge.js';

export function use(runtime: CaveatRuntime, source: string): void {
  const session: Session = typed<Program>(runtime.open(source));
  const inferred: Session = typed(runtime.open(source));

  session.dispatch('read', { celsius: 17 });
  session.dispatch('pick', { bed: 'north', size: 'large', who: 'ana' });
  session.dispatch('decide');
  inferred.dispatchView('decide', {});

  // View 0.2 on an untyped session (spec/caveat-view-0.2.md).
  const plain = runtime.open(source);
  const next = plain.view({ schema: '0.2' });
  const reopened: boolean | undefined = next.commitments[0]?.reopened;
  // @ts-expect-error View 0.2 has no `open`
  next.commitments[0]?.open;
  const delta = plain.dispatchViewDelta('decide');
  if (delta.outcome === 'accepted') {
    const since: number = delta.delta.since;
    const dropped: number[] = delta.delta.relations.removed;
    void [since, dropped];
  }
  void reopened;

  // @ts-expect-error not an event of the program
  session.dispatch('raed', { celsius: 17 });
  // @ts-expect-error not a field of read's payload
  session.dispatch('read', { celcius: 17 });
  // @ts-expect-error read needs its payload
  session.dispatch('read');
  // @ts-expect-error not an entity of kind bed
  session.dispatch('pick', { bed: 'west', size: 'large', who: 'ana' });
  // @ts-expect-error decide takes no fields
  session.dispatch('decide', { now: true });

  const view: View = session.view();
  const value: number = view.bindings.gauge.value;
  const covered: boolean = view.bindings.gauge.covered;
  const label: string | undefined = view.bindings.gauge.label;
  // @ts-expect-error label is shown only while its condition holds
  const always: string = view.bindings.gauge.label;
  // @ts-expect-error not a bound property
  view.bindings.gauge.valeu;
  const cited = view.binding_explanations.gauge?.value?.evidence;
  const pending: number = session.undrained;
  const archive: ArchiveEntry[] = session.drainArchive();
  const explained = explain(session.snapshot(), [], { archive });
  const records: string[] | undefined = explained.displayed[0]?.cites.departed?.[0]?.records;
  if (explained.archive) {
    explained.archive.authenticated satisfies false;
    explained.archive.scope satisfies 'provided records and their referenced closure';
    for (const record of explained.archive.records) {
      record.status satisfies 'complete' | 'unavailable';
      record.dependencies satisfies string[];
      record.entry?.withdrawal?.because satisfies string | undefined;
    }
  }
  dependents(session.snapshot(), 'reads', { archive });
  for (const entry of archive) {
    if (entry.kind === 'provenance') {
      entry.id satisfies string;
      if (entry.operation === 'union') entry.parents satisfies string[];
      else entry.source_id satisfies string;
    } else entry.record satisfies string;
  }
  void [pending, records];

  const outcome = session.dispatchView('read', { celsius: 3 });
  if (outcome.outcome === 'accepted') {
    const shown: number = outcome.view.bindings.gauge.value;
    void shown;
  }

  const queued: ProgramEvent[] = [{ event: 'read', payload: { celsius: 1 } }, { event: 'decide', payload: {} }];
  // @ts-expect-error a payload of another event
  const wrong: ProgramEvent = { event: 'read', payload: { bed: 'north', size: 'small', who: 'x' } };
  void [value, covered, label, always, cited, queued, wrong];
}

// The starter host takes the same declarations.
import { memoryStore, openHost } from '../lib/starter.mjs';
import type { SendResult } from '../lib/starter.mjs';
import { fileStore } from '../lib/starter-node.mjs';

export async function host(runtime: CaveatRuntime, source: string, directory: string): Promise<void> {
  const typedHost = await openHost<Program>({ runtime, source, store: memoryStore() });
  const result: SendResult<View> = await typedHost.send('read', { celsius: 17 });
  if (result.accepted && result.durable) {
    const shown: number = result.view.bindings.gauge.value;
    void shown;
  } else if (result.handled && !result.accepted) {
    const code: string = result.rejection.code;
    void code;
  }
  await typedHost.send('decide');
  // @ts-expect-error not an event of the program
  await typedHost.send('raed', { celsius: 17 });
  // @ts-expect-error read needs its payload
  await typedHost.send('read');
  const status: 'none' | 'in_force' | 'reopened' = typedHost.assessment('cover').status;
  void status;
  const onDisk = await openHost({ runtime, source, store: await fileStore(directory) });
  const entries: ArchiveEntry[] = await onDisk.archive();
  void entries;
}
