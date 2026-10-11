// Tracks which evidence each approval was granted on, so withdrawing one
// piece of evidence only puts the approvals that relied on it up for review.
export class ApprovalTracker {
  #evidence = new Set(); // currently valid evidence ids
  #approvals = new Map(); // approval id -> { basis: Set<evidence id>, status }

  addEvidence(id) {
    this.#evidence.add(id);
  }

  approve(id, basis) {
    const missing = [...basis].filter((e) => !this.#evidence.has(e));
    if (missing.length) throw new Error(`unknown or withdrawn evidence: ${missing.join(", ")}`);
    this.#approvals.set(id, { basis: new Set(basis), status: "approved" });
  }

  // Withdraw evidence; returns ids of approvals that now need reconsideration.
  withdraw(evidenceId) {
    this.#evidence.delete(evidenceId);
    const affected = [];
    for (const [id, a] of this.#approvals) {
      if (a.status === "approved" && a.basis.has(evidenceId)) {
        a.status = "reconsider";
        affected.push(id);
      }
    }
    return affected;
  }

  status(id) {
    const a = this.#approvals.get(id);
    if (!a) throw new Error(`unknown approval: ${id}`);
    return a.status;
  }
}
