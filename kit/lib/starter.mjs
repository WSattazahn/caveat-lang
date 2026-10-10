// The integration starter: one program's session behind a host that keeps
// what an application needs apart. A send is handled or not, an event is
// accepted or not, the checkpoint holding it is durable or not, and whether
// a decision permits anything is read from the current view, never from the
// send. External actions stay with the application.
//
// Persistence goes through a store. Every accepted event is checkpointed: the
// save, the source digest and any archive items not yet appended. An archive
// chunk is appended only after a checkpoint that holds it is durable, so the
// archive never runs ahead of the checkpoint, and a chunk is retried, never
// dropped, until an append succeeds. A store is only as durable as its own
// writes; memoryStore is not durable at all.
import { archiveSha256 } from './archive.mjs';
import { CaveatError } from './session.mjs';

export const CHECKPOINT_SCHEMA = 'caveat-starter-checkpoint/0.1';

/** SHA-256 of the exact source text, as the checkpoint records it. */
export function sourceDigest(source) {
  if (typeof source !== 'string') throw new TypeError('source must be text');
  return archiveSha256(source);
}

export class StarterError extends Error {
  constructor(kind, message) {
    super(message);
    this.name = 'StarterError';
    this.kind = kind;
  }
}

const chunkDigest = entries => archiveSha256(JSON.stringify(entries));
const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);

function validChunk(chunk) {
  return object(chunk) && Number.isSafeInteger(chunk.number) && chunk.number >= 0
    && Number.isSafeInteger(chunk.sequence) && Array.isArray(chunk.entries)
    && typeof chunk.sha256 === 'string' && chunk.sha256 === chunkDigest(chunk.entries);
}

function parseCheckpoint(text) {
  let checkpoint;
  try { checkpoint = JSON.parse(text); } catch {
    throw new StarterError('store', 'the stored checkpoint is not JSON');
  }
  if (!object(checkpoint) || checkpoint.schema !== CHECKPOINT_SCHEMA) {
    throw new StarterError('store', `the stored checkpoint is not ${CHECKPOINT_SCHEMA}`);
  }
  if (typeof checkpoint.save !== 'string' || typeof checkpoint.source_sha256 !== 'string'
    || !Number.isSafeInteger(checkpoint.next_chunk) || !Array.isArray(checkpoint.pending)
    || !checkpoint.pending.every(validChunk)) {
    throw new StarterError('store', 'the stored checkpoint is malformed');
  }
  return checkpoint;
}

// Chunks as stored, one per number. A repeated number must carry the same
// entries: a retried append may land twice, a different chunk may not.
function settleChunks(chunks) {
  const byNumber = new Map();
  for (const chunk of chunks) {
    if (!validChunk(chunk)) throw new StarterError('store', 'the stored archive holds a malformed chunk');
    const earlier = byNumber.get(chunk.number);
    if (earlier && earlier.sha256 !== chunk.sha256) {
      throw new StarterError('store', `the stored archive holds two different chunks numbered ${chunk.number}`);
    }
    byNumber.set(chunk.number, chunk);
  }
  return [...byNumber.values()].sort((a, b) => a.number - b.number);
}

class Host {
  #runtime;
  #store;
  #session;
  #digest;
  #nextChunk;
  #pending;
  #durableSequence;
  #sequence;
  #queue = Promise.resolve();
  #state = 'open';

  constructor({ runtime, store, session, digest, nextChunk, pending, durableSequence }) {
    this.#runtime = runtime;
    this.#store = store;
    this.#session = session;
    this.#digest = digest;
    this.#nextChunk = nextChunk;
    this.#pending = pending;
    this.#durableSequence = durableSequence;
    this.#sequence = session.view().sequence;
  }

  get state() { return this.#state === 'open' ? this.#session.state : this.#state; }
  get sourceSha256() { return this.#digest; }
  get sequence() { return this.#sequence; }
  /** Whether the store holds a checkpoint at the session's current sequence. */
  get durable() { return this.#durableSequence === this.sequence; }
  /** Archive chunks drained from the session and not yet appended to the store. */
  get pendingChunks() { return this.#pending.length; }

  // One operation at a time: a send, a flush or a close waits for the one before.
  #exclusive(operation) {
    const run = this.#queue.then(operation);
    this.#queue = run.catch(() => {});
    return run;
  }

  #usable() {
    if (this.#state === 'closed') throw new StarterError('closed', 'the host is closed');
    if (this.#session.state !== 'open') throw new StarterError('fatal', 'the session is unusable; open the host again from its store');
  }

  send(event, payload = {}) {
    return this.#exclusive(async () => {
      this.#usable();
      let outcome;
      try {
        outcome = this.#session.dispatchView(event, payload);
      } catch (error) {
        if (error instanceof CaveatError) {
          return { handled: false, accepted: false, durable: this.durable, error: { kind: error.kind, message: error.message } };
        }
        throw error;
      }
      if (outcome.outcome === 'rejected') {
        const { origin, code, message } = outcome;
        return { handled: true, accepted: false, durable: this.durable, rejection: { origin, code, message }, sequence: this.sequence };
      }
      this.#sequence = outcome.view.sequence;
      const persisted = await this.#checkpoint();
      return { handled: true, accepted: true, ...persisted, view: outcome.view, sequence: outcome.view.sequence };
    });
  }

  /** Retries a checkpoint or archive append that failed. */
  flush() {
    return this.#exclusive(async () => {
      this.#usable();
      return this.#checkpoint();
    });
  }

  async #checkpoint() {
    const entries = this.#session.drainArchive();
    if (entries.length) {
      this.#pending.push({ number: this.#nextChunk, sequence: this.sequence, sha256: chunkDigest(entries), entries });
      this.#nextChunk += 1;
    }
    const sequence = this.sequence;
    if (this.#durableSequence !== sequence) {
      const checkpoint = {
        schema: CHECKPOINT_SCHEMA,
        source_sha256: this.#digest,
        runtime: this.#runtime.identity,
        sequence,
        save: this.#session.save(),
        next_chunk: this.#nextChunk,
        pending: this.#pending,
      };
      try {
        await this.#store.writeCheckpoint(JSON.stringify(checkpoint));
      } catch (error) {
        return { durable: false, archived: false, storeError: String(error?.message ?? error) };
      }
      this.#durableSequence = sequence;
    }
    if (!this.#pending.length) return { durable: true, archived: true };
    const appending = [...this.#pending];
    try {
      await this.#store.appendChunks(appending);
    } catch (error) {
      return { durable: true, archived: false, storeError: String(error?.message ?? error) };
    }
    this.#pending = this.#pending.filter(chunk => !appending.includes(chunk));
    return { durable: true, archived: true };
  }

  view() { this.#usable(); return this.#session.view(); }
  snapshot() { this.#usable(); return this.#session.snapshot(); }

  /** The full view when the client's sequence is not the session's; View 0.1 has no delta. */
  resync(clientSequence) {
    const view = this.view();
    return clientSequence === view.sequence ? { resync: false, sequence: view.sequence } : { resync: true, sequence: view.sequence, view };
  }

  /**
   * The current assessment of one decision, from the current view: its
   * current revision, whether that revision is open again, the source it
   * was committed under, and the journal's last change to it. A windowed journal
   * may have retired that change; `change` is then absent.
   */
  assessment(decision) {
    const view = this.view();
    const series = view.decision_series[decision];
    if (!series) throw new StarterError('unknown', `${decision} is not a decision of the program`);
    if (series.current === null) return { decision, status: 'none', sequence: view.sequence, revision: null };
    // A revision without its commitment is never read as in force.
    const commitment = view.commitments.find(item => item.action === series.current);
    const result = { decision, status: commitment?.open === false ? 'in_force' : 'reopened', sequence: view.sequence, revision: series.current };
    // The runtime restores a save only under the source it was made under,
    // so every revision of this session was committed under this digest.
    result.source_sha256 = this.#digest;
    const entry = view.decision_journal.findLast(item => item.commitment === series.current);
    if (entry) {
      result.change = { change: entry.change, sequence: entry.sequence, event: entry.event, because: [...entry.because], caveats: [...entry.caveats] };
      if (entry.value !== undefined) result.change.value = entry.value;
      if (entry.permitted_by !== undefined) result.change.permitted_by = entry.permitted_by;
    }
    return result;
  }

  /** True only while the decision is in force in the current view. */
  permits(decision) {
    return this.assessment(decision).status === 'in_force';
  }

  /** Every archive item the store holds, in chunk order, then those still pending. */
  archive() {
    return this.#exclusive(async () => {
      const stored = settleChunks(await this.#store.readChunks());
      const numbers = new Set(stored.map(chunk => chunk.number));
      const chunks = [...stored, ...this.#pending.filter(chunk => !numbers.has(chunk.number))];
      return chunks.flatMap(chunk => chunk.entries);
    });
  }

  close() {
    return this.#exclusive(async () => {
      if (this.#state === 'closed') return;
      this.#state = 'closed';
      this.#session.close();
      await this.#store.close?.();
    });
  }
}

/**
 * Opens the program from the store's checkpoint, or fresh when the store is
 * empty. A checkpoint made under other source text is refused: the runtime
 * restores a save only under the exact source it was made under.
 */
export async function openHost({ runtime, source, store } = {}) {
  if (!runtime || typeof runtime.open !== 'function') throw new TypeError('runtime must be a loaded Caveat runtime');
  if (!store) throw new TypeError('store is required');
  const digest = sourceDigest(source);
  const text = await store.readCheckpoint();
  if (text === null || text === undefined) {
    const session = runtime.open(source);
    const host = new Host({ runtime, store, session, digest, nextChunk: 0, pending: [], durableSequence: null });
    const first = await host.flush();
    if (!first.durable) {
      session.close();
      throw new StarterError('store', `the first checkpoint was not written: ${first.storeError}`);
    }
    return host;
  }
  const checkpoint = parseCheckpoint(text);
  if (checkpoint.source_sha256 !== digest) {
    throw new StarterError('source', `the checkpoint was made under source ${checkpoint.source_sha256}, not this source ${digest}`);
  }
  const stored = settleChunks(await store.readChunks());
  const ahead = stored.find(chunk => chunk.number >= checkpoint.next_chunk);
  if (ahead) {
    throw new StarterError('store', `the archive holds chunk ${ahead.number}, which the checkpoint does not know; the store mixes two histories`);
  }
  for (const chunk of checkpoint.pending) {
    const same = stored.find(item => item.number === chunk.number);
    if (same && same.sha256 !== chunk.sha256) throw new StarterError('store', `archive chunk ${chunk.number} differs from the checkpoint's`);
  }
  const session = runtime.restore(source, checkpoint.save);
  const appended = new Set(stored.map(chunk => chunk.number));
  const host = new Host({
    runtime, store, session, digest,
    nextChunk: checkpoint.next_chunk,
    pending: checkpoint.pending.filter(chunk => !appended.has(chunk.number)),
    durableSequence: checkpoint.sequence,
  });
  await host.flush();
  return host;
}

/** A store in memory, for tests and demonstrations. Nothing survives the process. */
export function memoryStore() {
  let checkpoint = null;
  const chunks = [];
  return {
    durable: false,
    async readCheckpoint() { return checkpoint; },
    async writeCheckpoint(text) { checkpoint = text; },
    async readChunks() { return chunks.map(chunk => structuredClone(chunk)); },
    async appendChunks(list) { for (const chunk of list) chunks.push(structuredClone(chunk)); },
  };
}

const request = action => new Promise((resolve, reject) => {
  action.onsuccess = () => resolve(action.result);
  action.onerror = () => reject(action.error);
});

/**
 * A browser store in IndexedDB. The checkpoint is replaced and chunks are put
 * by number, each in one transaction that resolves when it completes. How
 * durable a completed transaction is depends on the browser.
 */
export async function indexedDbStore(name, { indexedDB = globalThis.indexedDB } = {}) {
  if (!indexedDB) throw new StarterError('store', 'IndexedDB is not available here');
  const opening = indexedDB.open(name, 1);
  opening.onupgradeneeded = () => {
    opening.result.createObjectStore('checkpoint');
    opening.result.createObjectStore('chunks', { keyPath: 'number' });
  };
  const db = await request(opening);
  const transact = (stores, mode, work) => new Promise((resolve, reject) => {
    const transaction = db.transaction(stores, mode, { durability: 'strict' });
    let result;
    transaction.oncomplete = () => resolve(result);
    transaction.onerror = () => reject(transaction.error);
    transaction.onabort = () => reject(transaction.error ?? new Error('IndexedDB transaction aborted'));
    Promise.resolve(work(transaction)).then(value => { result = value; }, reject);
  });
  return {
    durable: true,
    readCheckpoint: () => transact(['checkpoint'], 'readonly', tx => request(tx.objectStore('checkpoint').get('current')).then(value => value ?? null)),
    writeCheckpoint: text => transact(['checkpoint'], 'readwrite', tx => { tx.objectStore('checkpoint').put(text, 'current'); }),
    readChunks: () => transact(['chunks'], 'readonly', tx => request(tx.objectStore('chunks').getAll())),
    appendChunks: list => transact(['chunks'], 'readwrite', tx => { for (const chunk of list) tx.objectStore('chunks').put(chunk); }),
    close: async () => db.close(),
  };
}
