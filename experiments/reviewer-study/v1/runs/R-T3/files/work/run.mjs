import { readFileSync } from "node:fs";
import { createPolicy } from "./out/gate.js";
for (const h of ["H1","H2","H3"]) {
  const p = createPolicy();
  console.log(`===== ${h}`);
  readFileSync(new URL(`../histories/${h}.jsonl`, import.meta.url), "utf8").split("\n").filter(l=>l.trim()).forEach((l,i)=>{
    let r="accepted"; try { p.dispatch(JSON.parse(l)); } catch(e){ r="REFUSED: "+e.message; }
    console.log(`line ${i+1}: ${r} view=${JSON.stringify(p.view())}`);
  });
  console.log(p.explain());
  const rec = p.record();
  for (const o of rec.observations) console.log(`obs ${o.id} ${o.kind} ${JSON.stringify(o.payload)} stance=${o.stance} caveats=${JSON.stringify(o.caveats)} withdrawn=${JSON.stringify(o.withdrawn)}`);
}
