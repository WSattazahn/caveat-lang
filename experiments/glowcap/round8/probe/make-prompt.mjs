// Builds the learnability probe's prompt (round 8 protocol, "The learnability
// probe"). The probe model sees the beat and what Caveat is for, never its
// syntax: every code block and inline code span is removed, and every link
// keeps its text but loses its target, so nothing points at a specification.
//
//   node make-prompt.mjs            writes prompt.txt and prints its SHA256
//   node make-prompt.mjs --check    exits 1 if prompt.txt differs from a rebuild
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, '../../../..');
const read = (relative) => readFileSync(path.join(repo, relative), 'utf8');

function withoutSyntax(markdown) {
  return markdown
    .replace(/^```[^\n]*\n[\s\S]*?^```[^\n]*\n?/gm, '')
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/!\[[^\]]*\]\([^)]*\)/g, '')
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/`[^`\n]+`/g, '(…)')
    .replace(/https?:\/\/\S+/g, '')
    .replace(/\n{3,}/g, '\n\n')
    .trim();
}

// The package README's description: the paragraph that opens with the headline.
function description() {
  const text = read('kit/README.md');
  const start = text.indexOf('**A decision ledger for agents.**');
  if (start < 0) throw new Error('kit/README.md has no headline paragraph');
  return text.slice(start, text.indexOf('\n\n', start));
}

function buildPrompt() {
  const beat = read('experiments/glowcap/round7/BEAT.md');
  const essence = read('docs/CAVEAT_ESSENCE.md');
  return `${read('experiments/glowcap/round8/probe/instructions.md').trim()}

=== What Caveat is (its documentation, with all code removed) ===

${withoutSyntax(description())}

${withoutSyntax(essence)}

=== The feature to implement ===

${beat.trim()}
`;
}

const target = path.join(here, 'prompt.txt');
const prompt = buildPrompt();
const digest = createHash('sha256').update(prompt).digest('hex');
if (process.argv.includes('--check')) {
  const current = readFileSync(target, 'utf8');
  if (current !== prompt) {
    console.error('prompt.txt differs from a rebuild; run node make-prompt.mjs');
    process.exit(1);
  }
  console.log(`prompt.txt matches (${digest})`);
} else {
  writeFileSync(target, prompt);
  console.log(`prompt.txt ${prompt.length} characters, sha256 ${digest}`);
}
