import { createHash } from 'node:crypto';
import { appendFileSync, existsSync, readFileSync } from 'node:fs';

// Append-only, hash-chained ledger. Nothing is edited: a correction is a new
// entry, and "does the approval still hold" is derived by replaying the log.
const sha = (s) => createHash('sha256').update(s).digest('hex');

export class Ledger {
  #entries = [];
  #file;

  constructor({ file, now = () => new Date().toISOString() } = {}) {
    this.#file = file;
    this.now = now;
    if (file && existsSync(file)) {
      this.#entries = readFileSync(file, 'utf8').split('\n').filter(Boolean).map((l) => JSON.parse(l));
      this.verify();
    }
  }

  #append(type, body) {
    const prev = this.#entries.at(-1)?.hash ?? null;
    const entry = { seq: this.#entries.length, at: this.now(), type, ...body, prev };
    entry.hash = sha(JSON.stringify(entry));
    this.#entries.push(entry);
    if (this.#file) appendFileSync(this.#file, JSON.stringify(entry) + '\n');
    return entry;
  }

  // evidence: [{ id, source, claim, config?: any }]  (config is hashed, so a later
  // correction can say which configuration the result actually used)
  approve({ id, action, reason, approver, evidence }) {
    if (!evidence?.length) throw new Error('an approval must cite at least one piece of evidence');
    if (this.#entries.some((e) => e.type === 'approval' && e.id === id)) throw new Error(`duplicate approval ${id}`);
    const ev = evidence.map(({ config, ...e }) => ({ ...e, configHash: config === undefined ? null : sha(JSON.stringify(config)) }));
    return this.#append('approval', { id, action, reason, approver, evidence: ev });
  }

  // The correction can arrive any time later, and may refer to evidence by id
  // without knowing which approvals used it.
  retract({ evidenceId, reason, source, correctedClaim }) {
    if (!this.#evidenceIds().has(evidenceId)) throw new Error(`unknown evidence ${evidenceId}`);
    return this.#append('retraction', { evidenceId, reason, source, correctedClaim });
  }

  // Re-verification restores trust in evidence (e.g. test re-run with right config).
  reaffirm({ evidenceId, reason, source }) {
    if (!this.#evidenceIds().has(evidenceId)) throw new Error(`unknown evidence ${evidenceId}`);
    return this.#append('reaffirmation', { evidenceId, reason, source });
  }

  #evidenceIds() {
    return new Set(this.#entries.filter((e) => e.type === 'approval').flatMap((a) => a.evidence.map((v) => v.id)));
  }

  verify() {
    let prev = null;
    for (const [i, { hash, ...rest }] of this.#entries.entries()) {
      if (rest.seq !== i || rest.prev !== prev || sha(JSON.stringify(rest)) !== hash) throw new Error(`ledger tampered at seq ${i}`);
      prev = hash;
    }
    return true;
  }

  // Report on one approval: what it rested on, and whether that still stands.
  // `asOf` (ISO time) replays only entries up to then, showing what was known then.
  report(id, { asOf } = {}) {
    const log = asOf ? this.#entries.filter((e) => e.at <= asOf) : this.#entries;
    const approval = log.find((e) => e.type === 'approval' && e.id === id);
    if (!approval) throw new Error(`no approval ${id}${asOf ? ` as of ${asOf}` : ''}`);
    const evidence = approval.evidence.map((ev) => {
      const history = log.filter((e) => e.seq > approval.seq && e.evidenceId === ev.id);
      const last = history.at(-1);
      const status = last?.type === 'retraction' ? 'retracted' : 'standing';
      return { ...ev, status, history: history.map(({ at, type, reason, source, correctedClaim }) => ({ at, type, reason, source, correctedClaim })) };
    });
    const broken = evidence.filter((e) => e.status === 'retracted');
    return {
      id, action: approval.action, approver: approval.approver, approvedAt: approval.at,
      reason: approval.reason, evidence,
      holds: broken.length === 0,
      verdict: broken.length ? `UNDERMINED by ${broken.map((e) => e.id).join(', ')} – re-review required` : 'holds',
    };
  }

  // Approvals needing re-review because their evidence was retracted.
  undermined() {
    return this.#entries.filter((e) => e.type === 'approval').map((a) => this.report(a.id)).filter((r) => !r.holds);
  }

  get entries() { return structuredClone(this.#entries); }
}
