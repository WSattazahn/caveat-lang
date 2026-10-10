// Builds round 8's blind phase texts from REQUESTS.md (the writer's revised
// requests): each "# Phase crN" section verbatim, under the heading round 7's
// phase texts use.
//
//   node experiments/glowcap/round8/build-phases.mjs   (writes phases/cr17-cr20.md)
import { mkdir, readFile, writeFile } from 'node:fs/promises';

const text = await readFile(new URL('./REQUESTS.md', import.meta.url), 'utf8');
await mkdir(new URL('./phases/', import.meta.url), { recursive: true });
for (const n of [17, 18, 19, 20]) {
  const start = text.indexOf(`# Phase cr${n}\n`);
  if (start < 0) throw new Error(`REQUESTS.md has no cr${n}`);
  const next = text.slice(start + 1).search(/^#{1,2} /m);
  const body = text.slice(start, next < 0 ? undefined : start + 1 + next).split('\n').slice(1).join('\n').trim();
  await writeFile(new URL(`./phases/cr${n}.md`, import.meta.url), `# Phase cr${n}\n\nApplied after every earlier phase; cumulative.\n\n${body}\n`);
}
