import {readFileSync,writeFileSync} from 'node:fs';
const doc=JSON.parse(readFileSync('tracker.scenarios.json'));
doc.scenarios.push(
{id:'T07',title:'Reading limit rolls back neutral observation and leaves full history unchanged',steps:[
{send:'observe',payload:{value:100},repeat:16},
{checkpoint:'full'},
{send:'observe',payload:{value:0},rejected:{origin:'limit',code:'history_limit'}},
{same_as:'full',paths:['/']},
{resume:true},
{send:'assess'},
{expect:{'/commitment_grounds/answer@1/evidence':['checks@16']}}]},
{id:'T08',title:'Decision limit refuses atomically after every-reading reopening',steps:[
...Array.from({length:8},()=>[{send:'observe',payload:{value:70}},{send:'assess'}]).flat(),
{send:'observe',payload:{value:100}},
{checkpoint:'full'},
{send:'assess',rejected:{origin:'limit',code:'history_limit'}},
{same_as:'full',paths:['/commitment_bases','/qualified_values','/decision_journal']},
{resume:true}
]});
/* Whole snapshot pointer is the empty string, not slash. */
doc.scenarios.find(s=>s.id==='T07').steps.find(s=>s.same_as).paths=[''];
writeFileSync('tracker.scenarios.json',JSON.stringify(doc,null,2));
