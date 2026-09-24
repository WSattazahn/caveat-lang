// Plausible wrong programs, one realistic mistake each. Every one must fail.
import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';

const p1 = await readFile(new URL('./v3-sealed/reference-phase1.cav', import.meta.url), 'utf8');
const p2 = await readFile(new URL('./v3-sealed/reference-phase2.cav', import.meta.url), 'utf8');
const mutate = (source, from, to) => {
  if (!source.includes(from)) throw new Error(`mutation target missing: ${from}`);
  return source.replace(from, to);
};

const mutants = [
  ['grounds from a state copy of the minimum', 1, mutate(mutate(p1, 'on decide commit rink because enough using thinnest;',
    'on decide commit rink because enough using low;'), 'state cracked = 0 min 0 max 1;',
    'state cracked = 0 min 0 max 1;\nstate low = 60 min 0 max 60;\non measure set low = min(low, cm);')],
  ['closed rink reopened by thin reading', 1, mutate(p1, 'when cm < 10 and open_rink and latest(rink) >= 10', 'when cm < 10 and open_rink')],
  ['second crack accepted', 1, mutate(p1, 'on crack when cracked == 1 reject "A crack has already been reported.";\n', '')],
  ['no guard before the reading limit (fatal)', 1, mutate(p1, 'on measure when history_count(thickness) >= 8 reject "Eight holes have already been drilled.";\n', '')],
  ['decide with two readings', 1, mutate(p1, 'history_count(thickness) < 3 reject', 'history_count(thickness) < 2 reject')],
  ['value is the latest reading', 1, mutate(p1, 'on decide commit rink because enough using thinnest;', 'on decide commit rink because enough using latest(thickness);')],
  ['crack report without its caveat', 1, mutate(p1, 'secondhand qualifies crack_report;\n', '')],
  ['10 cm counted as thin', 1, mutate(mutate(p1, 'on measure when cm >= 10 sample', 'on measure when cm > 10 sample'), 'on measure when cm < 10 sample', 'on measure when cm <= 10 sample')],
  ['crack also reopens a closed rink', 1, mutate(p1, 'on crack when open_rink and latest(rink) >= 10 reopen', 'on crack when open_rink reopen')],
  ['decide allowed while in force', 1, mutate(p1, 'on decide when open_rink reject "The rink decision is already in force.";\n', '')],
  ['hud shows latest as thinnest', 1, mutate(p1, 'bind hud.thinnest = thinnest when has_sample(thickness);', 'bind hud.thinnest = latest(thickness) when has_sample(thickness);')],
  ['extra hud property', 1, mutate(p1, 'bind hud.revision = history_count(rink);', 'bind hud.revision = history_count(rink);\nbind hud.cracks = cracked;')],
  ['sonar left out of the value', 2, mutate(p2, 'on decide commit rink because enough using thinnest;', 'on decide commit rink because enough using auger_min;')],
  ['readings counts both instruments', 2, mutate(p2, 'bind hud.readings = history_count(thickness);', 'bind hud.readings = history_count(thickness) + history_count(sonar_thickness);')],
  ['thin scan does not reopen', 2, mutate(p2, 'on scan when cm < 10 and open_rink and latest(rink) >= 10\n  reopen rink because latest(sonar_thickness);\n', '')],
  ['decide still needs three auger readings', 2, mutate(p2, 'history_count(thickness) + history_count(sonar_thickness) < 3', 'history_count(thickness) < 3')],
  ['scan cap shared with measurements', 2, mutate(p2, 'on scan when history_count(sonar_thickness) >= 8', 'on scan when history_count(sonar_thickness) + history_count(thickness) >= 8')],
  ['sonar without its caveat', 2, mutate(p2, 'uncalibrated_sonar qualifies sonar;\n', '')],
];
const targets = mutants.map(([, phase, source]) => [source, phase, false]);
const targetFile = new URL('./mutant-targets.json', import.meta.url);
await writeFile(targetFile, JSON.stringify(targets));
const output = execFileSync(process.execPath, ['validate-v3.mjs', 'mutant-targets.json'],
  { encoding: 'utf8', cwd: new URL('./', import.meta.url) });
const lines = output.trim().split('\n').filter(line => /^(ok|BAD) /.test(line));
lines.forEach((line, index) => console.log(`${line.slice(0, 4)}${mutants[index][0].padEnd(44)} ${line.replace(/^.*?(\d+\/\d+|load error.*?\)).*$/, '$1')}`));
console.log(`${lines.filter(line => line.startsWith('ok')).length}/${mutants.length} mutants fail as they should`);
