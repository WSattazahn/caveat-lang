// Compiled by runtime.yml: TypeScript formatter compatibility.
// Local: npm exec --yes --package=typescript@5.9.3 -- tsc
// --noEmit --strict --module nodenext --target es2022
// --lib ES2022,ESNext.Disposable,DOM kit/type-tests/dependents-format.mts
import {
  dependents, formatDependents,
  type DependentsReport, type DependentDecision, type DependentChange,
  type DependentValue, type DependentDisplay,
} from '../lib/explain.mjs';
import type { Snapshot, Withdrawal } from '../lib/session.mjs';

type LegacyReport = Omit<DependentsReport, 'withdrawals' | 'reasonForWithdrawals' | 'claims' | 'decisions' | 'changes' | 'values' | 'displayed'> & {
  decisions: Omit<DependentDecision, 'withdrawn'>[];
  changes: Omit<DependentChange, 'withdrawn'>[];
  values: Omit<DependentValue, 'withdrawn'>[];
  displayed: Omit<DependentDisplay, 'withdrawn'>[];
};

const legacy: LegacyReport = {
  schema: 'caveat-dependents/0.1', subject: 'checks', kind: 'evidence', sequence: 4,
  decisions: [{ id: 'merge@1', value: 1, status: 'in force', basis: 'grounds', via: ['checks@1'] }],
  changes: [{ sequence: 3, event: 'decide', commitment: 'merge@1', change: 'committed', via: ['checks@1'] }],
  values: [{ name: 'estimate', value: 1, basis: 'grounds', via: ['checks@1'] }],
  displayed: [{ name: 'hud.value', value: 1, basis: 'cites', via: ['checks@1'] }],
};
const withdrawal: Withdrawal = { evidence: 'checks@1', because: 'recheck', sequence: 4, event: 'misread' };

// A saved legacy report and partially upgraded metadata are accepted directly.
formatDependents(legacy, 'legacy.cav', 4) satisfies string;
formatDependents({ ...legacy, withdrawals: [withdrawal],
  decisions: [{ ...legacy.decisions[0]!, withdrawn: [withdrawal] }],
  values: [{ ...legacy.values[0]!, withdrawn: [] }],
}) satisfies string;

// The producer always supplies its new fields, without optional-array checks.
declare const snapshot: Snapshot;
const current = dependents(snapshot, 'checks');
formatDependents(current) satisfies string;
current.withdrawals satisfies Withdrawal[];
current.reasonForWithdrawals satisfies Withdrawal[];
current.claims[0]!.relation satisfies 'supports' | 'opposes';
current.decisions[0]!.withdrawn satisfies Withdrawal[];
current.changes[0]!.withdrawn satisfies Withdrawal[];
current.values[0]!.withdrawn satisfies Withdrawal[];
current.displayed[0]!.withdrawn satisfies Withdrawal[];

// @ts-expect-error A legacy formatter input is not a complete new output report.
const legacyOutput: DependentsReport = legacy;
// @ts-expect-error New decisions must always have their withdrawal arrays.
const missingDecision: DependentsReport = { ...current, decisions: legacy.decisions };
// @ts-expect-error New changes must always have their withdrawal arrays.
const missingChange: DependentsReport = { ...current, changes: legacy.changes };
// @ts-expect-error New values must always have their withdrawal arrays.
const missingValue: DependentsReport = { ...current, values: legacy.values };
// @ts-expect-error New displays must always have their withdrawal arrays.
const missingDisplay: DependentsReport = { ...current, displayed: legacy.displayed };
// @ts-expect-error A present withdrawal still needs a numeric sequence.
formatDependents({ ...legacy, withdrawals: [{ ...withdrawal, sequence: '4' }] });
// @ts-expect-error Withdrawal is an annotation, never an authored revision status.
formatDependents({ ...legacy, decisions: [{ ...legacy.decisions[0]!, status: 'withdrawn' }] });
