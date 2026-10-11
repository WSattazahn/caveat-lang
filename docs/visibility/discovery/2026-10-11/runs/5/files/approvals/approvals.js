// Tracks which evidence each approval rests on, so withdrawing one piece of
// evidence only puts the approvals that relied on it up for reconsideration.
// An approval may have several alternative justifications (sets of evidence);
// it stays valid while at least one justification is fully intact.
export class ApprovalLedger {
  #evidence = new Set(); // evidence currently standing
  #approvals = new Map(); // id -> { justifications: Set<string>[] }

  addEvidence(id) {
    this.#evidence.add(id);
  }

  approve(id, ...justifications) {
    if (justifications.length === 0) throw new Error(`approval ${id} needs a justification`);
    const sets = justifications.map((j) => new Set(j));
    for (const s of sets) for (const e of s) {
      if (!this.#evidence.has(e)) throw new Error(`unknown or withdrawn evidence: ${e}`);
    }
    this.#approvals.set(id, { justifications: sets });
  }

  // Withdraws evidence; returns ids of approvals that just lost all support.
  withdraw(evidenceId) {
    const before = new Set(this.needsReconsideration());
    this.#evidence.delete(evidenceId);
    return this.needsReconsideration().filter((id) => !before.has(id));
  }

  // Restoring evidence revives approvals whose justification is whole again.
  restore(evidenceId) {
    this.#evidence.add(evidenceId);
  }

  isValid(id) {
    const a = this.#approvals.get(id);
    if (!a) throw new Error(`unknown approval: ${id}`);
    return a.justifications.some((s) => [...s].every((e) => this.#evidence.has(e)));
  }

  needsReconsideration() {
    return [...this.#approvals.keys()].filter((id) => !this.isValid(id));
  }
}
