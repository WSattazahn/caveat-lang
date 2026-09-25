// Text for a caveat-check/0.1 report (spec/caveat-check-0.1.md): each warning
// with where it is, what was found and what to consider, then any warnings an
// allow comment silenced, so none is hidden.

export const CHECK_SCHEMA = 'caveat-check/0.1';

export function formatCheck(report, name) {
  const lines = [];
  for (const warning of report.diagnostics) {
    lines.push(`${name}:${warning.line}:${warning.column}: warning ${warning.code} ${warning.name}: ${warning.message}`);
    lines.push(`  ${warning.suggestion}`);
    for (const related of warning.related) lines.push(`  see line ${related.line}: ${related.note}`);
  }
  for (const warning of report.suppressed) {
    lines.push(`${name}:${warning.line}:${warning.column}: allowed ${warning.code} ${warning.name} (an allow comment silences it)`);
  }
  const count = report.diagnostics.length;
  lines.push(count
    ? `${count} warning${count === 1 ? '' : 's'}. A warning points at a pattern worth a second look; it is not an error.`
    : `${name}: no warnings.`);
  return lines.join('\n');
}
