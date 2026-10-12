// Usage: node run.ts <history.jsonl>...  (Node >= 22.18 strips types; or compile with tsc)
import { readFileSync } from "node:fs";
import { createPolicy } from "./gate.ts";

for (const file of process.argv.slice(2)) {
  console.log(`== ${file}`);
  const p = createPolicy();
  const lines = readFileSync(file, "utf8").split("\n").filter((l) => l.trim() !== "");
  lines.forEach((text, i) => {
    let outcome = "accepted";
    try { p.dispatch(JSON.parse(text)); } catch (e) { outcome = `REFUSED (${(e as Error).message})`; }
    console.log(`${String(i + 1).padStart(2)} ${text}\n   -> ${outcome}; view ${JSON.stringify(p.view())}`);
  });
  console.log(p.explain());
}
