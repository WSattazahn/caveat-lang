import fs from 'node:fs';
const src = fs.readFileSync('pond.cav', 'utf8');
const scen = fs.readFileSync('pond.scenarios.json', 'utf8');
const mutants = {
  m1_no_scan_reopen: ['on scan when cm < 10 and rink_open reopen rink because latest(sonar_thickness);', ''],
  m2_count_auger_only: ['define reading_total = history_count(thickness) + history_count(sonar_thickness);', 'define reading_total = history_count(thickness);'],
  m3_scan_limit_missing: ['on scan when history_count(sonar_thickness) >= 8\n  reject "Eight sonar scans have already been taken; no more scans are accepted.";', ''],
  m4_thinnest_auger_only: ['fold_history(sonar_thickness, fold_history(thickness, latest(thickness), min), min)', 'fold_history(thickness, latest(thickness), min)'],
  m5_scans_reopen_closed: ['on scan when cm < 10 and rink_open reopen', 'on scan when cm < 10 and in_force reopen'],
  m6_shared_limit: ['on scan when history_count(sonar_thickness) >= 8', 'on scan when reading_total >= 8'],
};
for (const [name, [from, to]] of Object.entries(mutants)) {
  if (!src.includes(from)) throw new Error(`${name}: pattern not found`);
  fs.mkdirSync(`mutants/${name}`, { recursive: true });
  fs.writeFileSync(`mutants/${name}/pond.cav`, src.replace(from, to));
  fs.writeFileSync(`mutants/${name}/pond.scenarios.json`, scen);
}
