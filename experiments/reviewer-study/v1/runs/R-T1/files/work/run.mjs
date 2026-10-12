import { readFileSync } from "node:fs";
import { createPolicy } from "./gate.mjs";
for (const h of ["H1","H2","H3"]) {
  const p = createPolicy();
  const lines = readFileSync(`../histories/${h}.jsonl`,"utf8").split("\n").filter(Boolean);
  console.log(`===== ${h}`);
  lines.forEach((l,i)=>{ let r="accepted"; try{p.dispatch(JSON.parse(l));}catch(e){r="REFUSED: "+e.message;} console.log(`line ${i+1} ${l} -> ${r} view=${JSON.stringify(p.view())}`);});
  console.log(p.explain());
  const rec = p.record();
  for (const o of rec.observations) console.log(`obs ${o.id} ${o.kind} ${JSON.stringify(o.payload)} caveats=${JSON.stringify(o.caveats)} withdrawn=${JSON.stringify(o.withdrawn)}`);
}
