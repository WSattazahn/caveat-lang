import {spawnSync} from 'node:child_process';
import {writeFileSync,appendFileSync,readdirSync} from 'node:fs';
const args=process.argv.slice(2); const n=readdirSync('receipts').filter(x=>x.endsWith('.stdout.txt')).length+1; const stem=`receipts/${String(n).padStart(3,'0')}`;
const command=[process.execPath,...args]; const r=spawnSync(process.execPath,args,{encoding:'utf8',maxBuffer:32*1024*1024});
writeFileSync(stem+'.stdout.txt',r.stdout??'');writeFileSync(stem+'.stderr.txt',r.stderr??'');
const record={command,exit_status:r.status,signal:r.signal,error:r.error?.message,stdout:stem+'.stdout.txt',stderr:stem+'.stderr.txt'};
appendFileSync('receipts/commands.jsonl',JSON.stringify(record)+'\n'); console.log(JSON.stringify(record)); console.log(r.stdout); console.error(r.stderr); process.exitCode=r.status??1;
