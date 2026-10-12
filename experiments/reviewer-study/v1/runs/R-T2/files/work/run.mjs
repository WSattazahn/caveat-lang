import { readFileSync } from "node:fs";
import { createPolicy } from "./build/gate.js";
const probes = JSON.parse(readFileSync(new URL("../probes.json", import.meta.url)));
for (const h of ["H1","H2","H3"]) {
  console.log(`===== ${h}`);
  const p = createPolicy();
  const lines = readFileSync(new URL(`../histories/${h}.jsonl`, import.meta.url), "utf8").split("\n").filter(Boolean);
  lines.forEach((l, i) => {
    let res = "accepted";
    try { p.dispatch(JSON.parse(l)); } catch (e) { res = "REFUSED: " + e.message; }
    console.log(`line ${i+1} ${l} -> ${res} | view ${JSON.stringify(p.view())}`);
  });
  console.log("--- explain()"); console.log(p.explain());
  const rec = p.record();
  console.log("--- observations");
  for (const o of rec.observations) console.log(`${o.id} ${o.kind} ${JSON.stringify(o.payload)} stance=${o.stance} caveats=${JSON.stringify(o.caveats)} withdrawn=${JSON.stringify(o.withdrawn)}`);
  console.log("--- decisions (record)"); for (const d of rec.decisions) console.log(JSON.stringify(d));
  // Q2: hypothetical withdrawal against the record, using the program's own rule (gate.ts:252/259: reopen iff in force and restsOn includes id)
  const d = rec.decisions[rec.decisions.length-1];
  const inForce = d && d.reopened === null ? d : null;
  console.log("--- probes");
  for (const pr of probes[h]) {
    const o = rec.observations.find(x => x.line === pr);
    let ans;
    if (!inForce) ans = "none_in_force";
    else ans = [inForce.restsOn.check, inForce.restsOn.review, inForce.restsOn.scan].includes(`#${pr}`) ? "yes" : "no";
    console.log(`probe #${pr} (${o ? o.kind : "no observation"}): ${ans}; decision in force: ${inForce ? inForce.number : "none"}`);
  }
}
