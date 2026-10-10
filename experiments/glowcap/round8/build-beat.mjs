// Builds round 8's consolidated beat: round 7's BEAT.md (base to CR12) and
// round 7's registered phase texts for CR13-CR16, verbatim, so the
// change-request writer sees every inherited phase (PROTOCOL.md row 3).
//
//   node experiments/glowcap/round8/build-beat.mjs   (writes BEAT.md)
import { readFile, writeFile } from 'node:fs/promises';

const round7 = (file) => readFile(new URL(`../round7/${file}`, import.meta.url), 'utf8');
const head = '# Glowcap: the beat and change requests CR1-CR12\n';
let beat = await round7('BEAT.md');
if (!beat.startsWith(head)) throw new Error('round 7 BEAT.md heading changed');
beat = `# Glowcap: the beat and change requests CR1-CR16\n${beat.slice(head.length)}`;
const later = await Promise.all([13, 14, 15, 16].map((n) => round7(`phases/cr${n}.md`)));
await writeFile(new URL('./BEAT.md', import.meta.url), `${[beat.trimEnd(), ...later.map((t) => t.trimEnd())].join('\n\n')}\n`);
