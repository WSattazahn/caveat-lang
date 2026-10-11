'use strict';
// Append-only approval ledger. Records are never edited: an approval stores the
// evidence it rested on as it was understood at the time, and later corrections
// are separate records that point at that evidence. Status is derived on read.
const fs = require('node:fs');

class Ledger {
  // file: optional JSONL path; omit for in-memory use.
  constructor(file) {
    this.file = file;
    this.records = [];
    if (file && fs.existsSync(file)) {
      this.records = fs.readFileSync(file, 'utf8').split('\n').filter(Boolean).map((l) => JSON.parse(l));
    }
  }

  _append(rec) {
    rec.seq = this.records.length + 1;
    rec.at = rec.at || new Date().toISOString();
    Object.freeze(rec);
    this.records.push(rec);
    if (this.file) fs.appendFileSync(this.file, JSON.stringify(rec) + '\n');
    return rec;
  }

  // evidence: [{ id, source, claim, config? }] -- what the approver relied on.
  approve({ id, action, agent, reason, evidence }) {
    if (!id || !reason || !Array.isArray(evidence) || !evidence.length) {
      throw new Error('approval needs id, reason and at least one piece of evidence');
    }
    if (this.records.some((r) => r.type === 'approval' && r.id === id)) throw new Error(`duplicate approval ${id}`);
    const ids = evidence.map((e) => e.id);
    if (ids.some((x) => !x) || new Set(ids).size !== ids.length) throw new Error('evidence needs unique ids');
    // Deep-copy so later mutation by the caller cannot rewrite history.
    return this._append({ type: 'approval', id, action, agent, reason, evidence: JSON.parse(JSON.stringify(evidence)) });
  }

  // Correction arrives later. effect: 'invalidates' (evidence cannot support the
  // approval) or 'qualifies' (still usable, with caveat). replacedBy optionally
  // names a corrected evidence item (re-run under the right config).
  correct({ approvalId, evidenceId, effect, finding, replacedBy }) {
    const a = this.records.find((r) => r.type === 'approval' && r.id === approvalId);
    if (!a) throw new Error(`unknown approval ${approvalId}`);
    if (!a.evidence.some((e) => e.id === evidenceId)) throw new Error(`approval ${approvalId} did not rely on ${evidenceId}`);
    if (!['invalidates', 'qualifies'].includes(effect)) throw new Error('effect must be invalidates|qualifies');
    if (!finding) throw new Error('correction needs a finding');
    return this._append({ type: 'correction', approvalId, evidenceId, effect, finding, replacedBy: replacedBy || null });
  }

  // Corrected evidence, e.g. the test re-run with the right config.
  // result: 'supports' | 'contradicts'
  reverify({ approvalId, evidenceId, result, evidence }) {
    if (!['supports', 'contradicts'].includes(result)) throw new Error('result must be supports|contradicts');
    const corr = this.records.find((r) => r.type === 'correction' && r.approvalId === approvalId && r.evidenceId === evidenceId && r.effect === 'invalidates');
    if (!corr) throw new Error('nothing to re-verify: no invalidating correction for that evidence');
    return this._append({ type: 'reverification', approvalId, evidenceId, result, evidence: evidence || null });
  }

  // Derived view: what it rested on, what has since been said about it, and
  // whether it still holds. Status values:
  //   holds                 - no correction touches the evidence
  //   holds-with-caveats    - only 'qualifies' corrections
  //   unsupported           - some evidence invalidated and not re-verified
  //                           (the approval may be right, but nothing now shows it)
  //   reaffirmed            - all invalidated evidence re-verified as supporting
  //   contradicted          - re-verification went against the approval
  report(approvalId) {
    const a = this.records.find((r) => r.type === 'approval' && r.id === approvalId);
    if (!a) throw new Error(`unknown approval ${approvalId}`);
    const mine = this.records.filter((r) => r.approvalId === approvalId);
    const evidence = a.evidence.map((e) => {
      const corrections = mine.filter((r) => r.type === 'correction' && r.evidenceId === e.id);
      const reverifications = mine.filter((r) => r.type === 'reverification' && r.evidenceId === e.id);
      const invalidated = corrections.some((c) => c.effect === 'invalidates');
      const last = reverifications[reverifications.length - 1];
      let state = 'intact';
      if (invalidated) state = last ? (last.result === 'supports' ? 'reaffirmed' : 'contradicted') : 'invalidated';
      else if (corrections.length) state = 'qualified';
      return { ...e, state, corrections, reverifications };
    });
    const states = evidence.map((e) => e.state);
    const status = states.includes('contradicted') ? 'contradicted'
      : states.includes('invalidated') ? 'unsupported'
      : states.includes('reaffirmed') ? 'reaffirmed'
      : states.includes('qualified') ? 'holds-with-caveats' : 'holds';
    return { approval: a, status, evidence };
  }

  format(approvalId) {
    const { approval: a, status, evidence } = this.report(approvalId);
    const out = [`Approval ${a.id}: ${a.action || ''} by ${a.agent || 'unknown'} at ${a.at}`,
      `Reason given: ${a.reason}`, `STATUS NOW: ${status}`, 'Rested on:'];
    for (const e of evidence) {
      out.push(`  - [${e.state}] ${e.id} (${e.source}): ${e.claim}${e.config ? ` | config: ${JSON.stringify(e.config)}` : ''}`);
      for (const c of e.corrections) out.push(`      correction @${c.at} (${c.effect}): ${c.finding}`);
      for (const r of e.reverifications) out.push(`      re-verified @${r.at}: ${r.result}`);
    }
    return out.join('\n');
  }
}

module.exports = { Ledger };
