import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
const A='/home/claude/glowcap-r7/authors/';
const h=JSON.parse(readFileSync('results/fuzz-seed7.json')).find(x=>x.entry.includes('/C2/')).example.history;
for (const n of ['T1','T2','T3','C1','C2','C3','C4']) {
  const m=await import(pathToFileURL(A+n+'/impl/'+(n[0]==='T'?'glowcap.ts':'adapter.mjs')).href); if(m.ready) await m.ready;
  let p=m.createPolicy(); let i=0; let res=[];
  for (const e of [...h, {type:'tick',dt:0.0625}]) {
    if (e.type==='resume') { if(!process.env.NORESUME) p=m.createPolicy(JSON.parse(JSON.stringify(p.save()))); continue; }
    try{p.dispatch(e)}catch{}; if (i>=1783) res.push(p.view().belief.caveats.includes('taste_faded')); i++;
  }
  console.log(n, res.join(' '));
}
