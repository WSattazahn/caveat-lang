// Retrospective, read-only audit of frozen author logs. Never imports a runtime
// or executes a candidate. Only the two generated files beside this script write.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../..');
const id = '[A-Za-z_][A-Za-z0-9_]*';
const framePattern = new RegExp(`^event (${id}), rule ([1-9][0-9]*): ([\\s\\S]*)$`);
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const read = file => readFile(path.join(root, file));
const slash = file => file.split(path.sep).join('/');

export function sourceIdentity(source) {
  const bytes = Buffer.from(source);
  let hash = 0xcbf29ce484222325n;
  for (const byte of bytes) hash = BigInt.asUintN(64, (hash ^ BigInt(byte)) * 0x100000001b3n);
  return `fnv1a64:${hash.toString(16).padStart(16, '0')}:${bytes.length}`;
}

// A deliberately narrow source corroborator, not a replacement CAVEAT parser.
// Preserve comments/quotes/procedure braces when finding top-level on rules.
export function inspectSource(source) {
  const statements = [];
  let text = '', line = 1, startLine = null, quoted = false, escaped = false, comment = false, braces = 0;
  let unsupported = null;
  for (let index = 0; index < source.length; index++) {
    const ch = source[index];
    if (comment) {
      text += ch === '\n' ? '\n' : ' ';
      if (ch === '\n') comment = false;
    } else if (quoted) {
      text += ch;
      if (escaped) escaped = false;
      else if (ch === '\\') escaped = true;
      else if (ch === '"') quoted = false;
    } else if (ch === '#' || (ch === '/' && source[index + 1] === '/')) {
      comment = true;
      text += ' ';
    } else if (ch === ';' && braces === 0) {
      if (text.trim()) statements.push({ text: text.trim(), line: startLine });
      text = ''; startLine = null;
    } else {
      if (!/\s/.test(ch) && startLine === null) startLine = line;
      if (ch === '"') quoted = true;
      if (ch === '{') braces++;
      if (ch === '}') braces--;
      if (braces < 0) unsupported = 'unbalanced source braces';
      text += ch;
    }
    if (ch === '\n') line++;
  }
  if (quoted || braces !== 0 || text.trim()) unsupported = 'unterminated or unbalanced source';
  if (statements.some(statement => /^(?:for|module|part|use|bundle)\b/.test(statement.text))) {
    unsupported = 'source expansion/linking is outside this corroborator';
  }
  const rules = statements.filter(statement => /^on\s/.test(statement.text)).map((statement, index) => {
    // String tokens are kept whole, so text that says reject cannot be an effect.
    const tokens = statement.text.match(/"(?:\\.|[^"\\])*"|[^\s"]+/gs) ?? [];
    const rejectIndex = tokens.findIndex(token => token === 'reject');
    let message = null;
    if (rejectIndex >= 0 && rejectIndex === tokens.length - 2 && tokens.at(-1).startsWith('"')) {
      try { message = JSON.parse(tokens.at(-1)); } catch { /* Unsupported literal stays uncorroborated. */ }
    }
    return { number: index + 1, event: tokens[1], line: statement.line, text: statement.text, rejectMessage: message };
  });
  return { rules, unsupported, sourceId: sourceIdentity(source),
    hasProcedures: statements.some(statement => /^proc\b/.test(statement.text)),
    containsRejectionMarker: source.includes('rejected: ') };
}

export function classifyRejected(entry, source) {
  const unknown = reason => ({ origin: 'unknown', reason });
  if (entry.kind !== 'rejected' || typeof entry.error !== 'string' || typeof entry.event?.event !== 'string') {
    return unknown('missing rejected-event string fields');
  }
  const message = entry.error;
  const event = entry.event.event;
  const frame = framePattern.exec(message);
  if (frame && frame[1] === event && frame[3].startsWith('rejected: ')) {
    if (source.unsupported) return unknown(source.unsupported);
    if (entry.snapshot?.source_id !== source.sourceId) return unknown('snapshot does not match saved source identity');
    const rule = source.rules[Number(frame[2]) - 1];
    const policyMessage = frame[3].slice('rejected: '.length);
    if (rule?.event !== event || rule.rejectMessage !== policyMessage) {
      return unknown('context prefix lacks matching saved on-rule/reject literal');
    }
    return { origin: 'authored_policy', reason: 'exact direct-rule rejection prefix and saved reject literal match',
      rule: rule.number, sourceLine: rule.line, sourceStatement: rule.text, policyMessage };
  }
  // Fixed complete templates where practical; never count arbitrary substring
  // matches as policy. Nested procedure frames are deliberately not inferred.
  if (message === `undeclared event ${event}`) return { origin: 'other_error', reason: 'undeclared_event_text' };
  if (message === `event ${event} requires exactly its declared parameters`) {
    return { origin: 'other_error', reason: 'parameter_set_text' };
  }
  const number = new RegExp(`^event (${id}) parameter (${id}) expects a number$`).exec(message);
  if (number?.[1] === event) return { origin: 'other_error', reason: 'parameter_type_text' };
  if (message.startsWith('invalid event payload: ')) return { origin: 'other_error', reason: 'payload_decode_text' };
  if (new RegExp(`^${id} must be finite and in -?[0-9.e+]+\\.\\.-?[0-9.e+]+$`).test(message)) {
    return { origin: 'other_error', reason: 'numeric_range_text' };
  }
  if (frame?.[1] === event) {
    if (new RegExp(`^decision series ${id} reached its history limit [0-9]+$`).test(frame[3])) {
      return { origin: 'other_error', reason: 'history_limit_text' };
    }
    if (new RegExp(`^current decision in ${id} must be explicitly reopened before revision$`).test(frame[3])) {
      return { origin: 'other_error', reason: 'closed_revision_text' };
    }
  }
  return unknown('no corroborated policy rejection or recognized non-policy template');
}

function emptyCounts() {
  return { checks: 0, checkErrors: 0, acceptedEvents: 0, rejectedEvents: 0, authoredPolicy: 0, otherErrors: 0, unknown: 0, resumed: 0 };
}

function addCounts(into, from) {
  for (const key of Object.keys(into)) into[key] += from[key];
}

async function evidence(file, expected) {
  const bytes = await read(file);
  const hash = sha256(bytes);
  if (expected !== undefined) assert.equal(hash, expected, `Saved SHA-256 mismatch: ${file}`);
  return { file, bytes: bytes.length, sha256: hash, ...(expected === undefined ? {} : { recordedSha256Matches: true }) };
}

function historicalRuntime(commit) {
  const file = 'runtime/src/reactive.rs';
  const bytes = execFileSync('git', ['show', `${commit}:${file}`], { cwd: root, maxBuffer: 8 * 1024 * 1024 });
  const lines = bytes.toString('utf8').split('\n');
  const needles = ['Effect::Reject { message } => return Err(format!("rejected: {message}"))',
    'format!("event {event}, rule {}: {error}"', 'format!("procedure {name}, step {}: {error}"'];
  const constructors = needles.map(needle => {
    const index = lines.findIndex(line => line.includes(needle));
    assert(index >= 0, `Historical constructor missing: ${commit} ${needle}`);
    return { line: index + 1, text: lines[index].trim() };
  });
  return { commit, file, bytes: bytes.length, sha256: sha256(bytes), constructors };
}

export async function auditAuthorLogs() {
  const report = { schema: 1, method: 'retrospective_source_corroborated_text_inference',
    scope: 'Only recorded author checks in v1/v2 runs/*/checks/[0-9]+.jsonl; SMOKE excluded. No candidate reruns.',
    caveat: 'Plain historical errors do not provide structured runtime origin. Other errors are not necessarily bugs; many are intentional invalid-input probes.',
    counts: emptyCounts(), studies: [] };
  for (const study of ['v1', 'v2']) {
    const base = `experiments/agent-authoring/${study}`;
    const manifestFile = `${base}/packet/manifest.json`;
    const manifest = JSON.parse(await read(manifestFile));
    const result = { study, counts: emptyCounts(), manifest: await evidence(manifestFile),
      runtimeSource: historicalRuntime(manifest.sourceCommit), runtimeArtifacts: [],
      runner: await evidence(`${base}/packet/runner.mjs`), excludedRuns: ['SMOKE'], runs: [] };
    for (const [file, expected] of Object.entries(manifest.runtime)) {
      result.runtimeArtifacts.push(await evidence(`${base}/packet/runtime/${file}`, expected));
    }
    const directories = (await readdir(path.join(root, base, 'runs'), { withFileTypes: true }))
      .filter(item => item.isDirectory() && item.name !== 'SMOKE').map(item => item.name).sort();
    for (const run of directories) {
      const runBase = `${base}/runs/${run}`;
      const recordFile = `${runBase}/record.json`;
      const record = JSON.parse(await read(recordFile));
      const row = { run, counts: emptyCounts(), record: await evidence(recordFile), sources: [], checks: [], rejected: [], checkErrors: [] };
      const sources = new Map();
      for (const version of record.versions) {
        const file = `${runBase}/${version.file}`;
        const source = inspectSource((await read(file)).toString('utf8'));
        sources.set(version.number, source);
        row.sources.push({ version: version.number, ...await evidence(file, version.sha256), sourceId: source.sourceId,
          corroboratorLimitation: source.unsupported, hasProcedures: source.hasProcedures,
          containsRejectionMarker: source.containsRejectionMarker });
      }
      const logFiles = (await readdir(path.join(root, runBase, 'checks'))).filter(file => /^[0-9]+\.jsonl$/.test(file)).sort();
      assert.deepEqual(logFiles, record.checks.map(check => path.basename(check.log)).sort(), `${runBase}: recorded checks differ from log files`);
      for (const check of record.checks) {
        const file = `${runBase}/${check.log}`;
        const source = sources.get(check.version);
        assert(source, `${file}: unknown saved version ${check.version}`);
        const checkInfo = { index: check.index, command: check.command, version: check.version, status: check.status,
          ...await evidence(file), input: null };
        const inputFile = file.replace(/\.jsonl$/, '.input.jsonl');
        try { checkInfo.input = await evidence(inputFile); } catch (error) { if (error.code !== 'ENOENT') throw error; }
        row.checks.push(checkInfo); row.counts.checks++;
        let errorCount = 0;
        const logLines = (await read(file)).toString('utf8').split(/\r?\n/);
        for (let index = 0; index < logLines.length; index++) {
          if (!logLines[index].trim()) continue;
          const entry = JSON.parse(logLines[index]);
          if (entry.kind === 'accepted') row.counts.acceptedEvents++;
          else if (entry.kind === 'resumed') row.counts.resumed++;
          else if (entry.kind === 'error') {
            row.counts.checkErrors++; errorCount++;
            row.checkErrors.push({ file, jsonlLine: index + 1, check: check.index, version: check.version, message: entry.message,
              classification: /^line [0-9]+, column [0-9]+: /.test(entry.message) ? 'construction_or_parse_error_text' : 'unknown_check_error' });
            assert.equal(entry.message, check.error, `${file}: check error differs from record`);
          } else if (entry.kind === 'rejected') {
            row.counts.rejectedEvents++;
            const classification = classifyRejected(entry, source);
            row.counts[{ authored_policy: 'authoredPolicy', other_error: 'otherErrors', unknown: 'unknown' }[classification.origin]]++;
            row.rejected.push({ file, jsonlLine: index + 1, inputLine: entry.line, check: check.index, version: check.version,
              sourceFile: row.sources.find(item => item.version === check.version).file,
              event: entry.event, error: entry.error, snapshotSourceId: entry.snapshot?.source_id ?? null, ...classification });
          } else assert.equal(entry.kind, 'validated', `${file}: unrecognized log kind`);
        }
        assert.equal(errorCount, check.status === 'error' ? 1 : 0, `${file}: check-level error count differs from record`);
      }
      assert.equal(row.counts.rejectedEvents, row.counts.authoredPolicy + row.counts.otherErrors + row.counts.unknown);
      addCounts(result.counts, row.counts); result.runs.push(row);
    }
    addCounts(report.counts, result.counts); report.studies.push(result);
  }
  return report;
}

export function authorMarkdown(report) {
  const rows = report.studies.flatMap(study => study.runs.map(run => `| ${study.study} | ${run.run} | ${run.counts.checks} | ${run.counts.checkErrors} | ${run.counts.acceptedEvents} | ${run.counts.rejectedEvents} | ${run.counts.authoredPolicy} | ${run.counts.otherErrors} | ${run.counts.unknown} |`));
  const totals = report.studies.map(study => `| ${study.study} | **total** | ${study.counts.checks} | ${study.counts.checkErrors} | ${study.counts.acceptedEvents} | ${study.counts.rejectedEvents} | ${study.counts.authoredPolicy} | ${study.counts.otherErrors} | ${study.counts.unknown} |`);
  const errorRows = report.studies.flatMap(study => study.runs.flatMap(run => run.checkErrors.map(error =>
    `| ${study.study}/${run.run} | ${error.check} | ${error.version} | ${error.message.replaceAll('|', '\\|')} |`)));
  const groups = new Map();
  for (const study of report.studies) for (const run of study.runs) for (const entry of run.rejected) {
    if (entry.origin === 'authored_policy') continue;
    const key = `${study.study}|${entry.reason}`; groups.set(key, (groups.get(key) ?? 0) + 1);
  }
  return `# Frozen author-log rejection-origin audit\n\n` +
    `This is retrospective, source-corroborated **text inference**, not structured runtime attribution. No author candidate was rerun. SMOKE is excluded. Only recorded author checks in \`runs/*/checks/[0-9]+.jsonl\` are counted; inputs, console transcripts, verifier/scoring runs, and final-result verdicts are not additional observations.\n\n` +
    `Run \`node experiments/dispatch-audit/author-logs.mjs\` from the repository root to regenerate this report and \`results-author-logs.json\`. Run \`node --test experiments/dispatch-audit/author-logs.test.mjs\` for the classifier fixture. The script reads frozen inputs and writes only these two audit outputs.\n\n` +
    `## Counts\n\nCheck errors are failed construction/validation/check operations, not rejected dispatch events. Rejected events below are partitioned into authored policy, recognized other error text, and unknown. Other errors include deliberately invalid input probes and bounded-history protections; this audit does not call them implementation bugs.\n\n` +
    `| Study | Run | Checks | Check errors | Accepted events | Rejected events | Authored policy | Other errors | Unknown |\n|---|---|---:|---:|---:|---:|---:|---:|---:|\n${[...rows, ...totals].join('\n')}\n\n` +
    `## Attribution rule and evidence\n\nA policy classification requires the entire prefix \`event IDENTIFIER, rule POSITIVE_INTEGER: rejected: \`, exact agreement with the logged event name, the snapshot source identity matching the preserved version, and the corresponding top-level \`on\` rule having the identical \`reject\` literal. A \`rejected: \` substring alone is never sufficient. The corroborator respects quoted strings, comments, and procedure braces; it declines expansion/linking and nested-procedure attribution instead of guessing.\n\n` +
    `Both recorded source commits have the unique \`Effect::Reject\` constructor and the event/rule and procedure/step wrappers. The JSON preserves each historical source commit, source SHA-256, constructor line/text, frozen runtime manifest/artifact hashes, runner hash, run-record hash, saved-version hash, every check-log/input hash, and every rejected observation's JSONL line, input line, source version, error, and classification. Historical runtime source is read with \`git show\` at each packet's recorded commit; current core edits do not alter this evidence. Packet WASM hashes match their manifests. This verifies file consistency, not a cryptographic proof that the historical WASM was compiled from those source bytes.\n\n` +
    `## Check-level errors\n\n| Run | Check | Version | Saved message |\n|---|---:|---:|---|\n${errorRows.join('\n')}\n\n` +
    `## Other rejected-event templates\n\n| Study | Recognized template | Count |\n|---|---|---:|\n${[...groups].sort().map(([key, count]) => `| ${key.replace('|', ' | ')} | ${count} |`).join('\n')}\n\n` +
    `## Counterfeit text and limits\n\nAuthored text or payload strings can contain \`rejected: \`, including apparently complete contextual prefixes. Unknown-event/payload/parser diagnostics can repeat supplied text; quoted source literals and repetition templates can carry arbitrary text. Such embedded text retains an outer error template and is not a policy rejection. The fixture checks counterfeit markers in payload diagnostics, quoted source text, event-name spoofing, and a source-rule/message mismatch. Saved versions contain no \`rejected: \` literal and use no repetition/linking expansion; procedures occur, but no observed policy rejection has a nested procedure frame.\n\n` +
    `The anchored prefix plus preserved direct-rule corroboration makes the observed policy cases strong historical evidence, but does not turn untyped strings into a runtime guarantee. Attribution assumes trustworthy recorded logs and runtime provenance. A fabricated entry or novel diagnostic reproducing the complete matching frame, source identity, rule, and reject literal cannot be detected by this retrospective classifier; an unrecorded runtime change can likewise invalidate the inference. Unsupported source rewriting and nested procedure attribution remain unknown. The classifier deliberately recognizes only the other-error templates observed here; unrecognized formats remain unknown. Authored policy means the reject effect is inferred to have caused the recorded failure, not that the author's admission policy was correct.\n`;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const report = await auditAuthorLogs();
  await writeFile(path.join(here, 'results-author-logs.json'), `${JSON.stringify(report, null, 2)}\n`);
  await writeFile(path.join(here, 'AUTHOR_LOGS.md'), authorMarkdown(report));
  console.log(JSON.stringify({ counts: report.counts, studies: report.studies.map(study => ({ study: study.study, counts: study.counts })) }, null, 2));
}
