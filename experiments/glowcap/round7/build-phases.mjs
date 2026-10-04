// Builds the round-7 phase texts from ../PROTOCOL.md: the beat (base) and
// CR1-CR12, verbatim, with round narratives, predictions and results left
// out. The one change is CR10's "no limit", read as a declared bound for this
// round (PROTOCOL.md here, "Bounded capacity").
//
//   node experiments/glowcap/round7/build-phases.mjs   (writes phases/ and BEAT.md)
import { mkdir, readFile, writeFile } from 'node:fs/promises';

const source = await readFile(new URL('../PROTOCOL.md', import.meta.url), 'utf8');
const lines = source.split('\n');
const at = (text) => {
  const index = lines.findIndex((line) => line.startsWith(text));
  if (index < 0) throw new Error(`missing ${text}`);
  return index;
};

const base = lines.slice(at('## The beat'), at('## Change requests')).join('\n').trimEnd();

function request(n) {
  const start = at(`- **CR${n} — `);
  let end = start + 1;
  while (end < lines.length && lines[end].startsWith('  ')) end += 1;
  // CR11's table is indented; keep blank-separated indented continuation.
  while (end < lines.length && lines[end] === '' && lines[end + 1]?.startsWith('  ')) {
    end += 1;
    while (end < lines.length && lines[end].startsWith('  ')) end += 1;
  }
  return lines.slice(start, end).join('\n');
}

const UNBOUNDED = 'and so on, with no limit.';
const BOUNDED = 'and so on, up to a capacity the implementation declares, of at least\n  64 lives per mushroom. A mushroom that has used its last life stays consumed\n  and does not regrow; no tick is rejected for this, and an event on that\n  mushroom is rejected as on any consumed mushroom.';

await mkdir(new URL('./phases/', import.meta.url), { recursive: true });
const parts = [`# Phase base\n\n${base}\n`];
await writeFile(new URL('./phases/base.md', import.meta.url), parts[0]);
for (let n = 1; n <= 12; n += 1) {
  let text = request(n);
  if (n === 10) {
    if (!text.includes(UNBOUNDED)) throw new Error('CR10 wording changed');
    text = text.replace(UNBOUNDED, BOUNDED);
  }
  const page = `# Phase cr${n}\n\nApplied after every earlier phase; cumulative.\n\n${text}\n`;
  parts.push(page);
  await writeFile(new URL(`./phases/cr${n}.md`, import.meta.url), page);
}
await writeFile(new URL('./BEAT.md', import.meta.url), `# Glowcap: the beat and change requests CR1-CR12\n\n${parts.join('\n')}`);
