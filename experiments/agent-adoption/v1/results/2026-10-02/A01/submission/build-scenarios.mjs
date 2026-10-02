import {writeFileSync,copyFileSync} from 'node:fs';
const send=(name,value)=>({send:name,...(value===undefined?{}:{payload:{value}})});
const reject=(name)=>({send:name,rejected:{origin:'policy',code:'reject'}});
const expect=x=>({expect:x});
const scenarios=[
{id:'T01',title:'Observation is neutral and does not assess; 70 supports',steps:[
send('observe',70),expect({'/decision_journal':[],'/observations':['tool','checks@1'],'/reading_streams/checks/occurrences/0':{id:'checks@1',value:70,relation:'supports',claim:'ready'},'/bindings/answer/in_force':false}),
send('assess'),expect({'/commitment_grounds/answer@1':{evidence:['checks@1'],caveats:['uncertain']},'/bindings/answer/in_force':true})]},
{id:'T02',title:'Authored refusal and input refusal preserve the session',steps:[
reject('assess'),reject('correct'),reject('learn_stale'),
{send:'observe',payload:{value:101},rejected:{origin:'input',code:'bound_exceeded'}},
send('observe',69),reject('assess'),expect({'/decision_journal':[],'/reading_streams/checks/occurrences/0/relation':'opposes'})]},
{id:'T03',title:'Correction preserves archive and frozen grounds, then revision',steps:[
send('observe',85),send('assess'),{checkpoint:'old'},send('correct'),
expect({'/withdrawals/0':{evidence:'checks@1',because:'correction'},'/bindings/answer/reopened':true}),
{same_as:'old',paths:['/reading_streams/checks/occurrences','/commitment_grounds/answer@1','/commitment_bases/answer@1']},
reject('correct'),reject('assess'),{resume:true},send('observe',92),send('assess'),
expect({'/decision_series/answer/current':'answer@2','/commitment_grounds/answer@2':{evidence:['checks@2'],caveats:['uncertain']}}),
{same_as:'old',paths:['/commitment_grounds/answer@1','/commitment_bases/answer@1']}]},
{id:'T04',title:'Duplicate assess refused; equal and supporting readings reopen',steps:[
send('observe',85),send('assess'),reject('assess'),send('observe',85),
expect({'/bindings/answer/reopened':true}),send('assess'),send('observe',100),send('assess'),
expect({'/decision_series/answer/current':'answer@3','/commitment_grounds/answer@3/evidence':['checks@3']})]},
{id:'T05',title:'Late qualification does not reopen or mutate earlier records; future stale refused',steps:[
send('observe',85),send('assess'),{checkpoint:'clean'},send('learn_stale'),
expect({'/bindings/answer/in_force':true}),
{same_as:'clean',paths:['/reading_streams/checks/occurrences','/commitment_grounds/answer@1','/commitment_bases/answer@1','/decision_journal']},
reject('assess'),{resume:true},send('observe',90),reject('assess'),
expect({'/reading_streams/checks/occurrences/1/provenance/caveats':{$set:['uncertain','stale']},'/decision_series/answer/current':'answer@1','/bindings/answer/reopened':true})]},
{id:'T06',title:'Earlier clean reading eligible after learning stale, across restore',steps:[
send('observe',70),{checkpoint:'reading'},send('learn_stale'),{resume:true},send('assess'),
expect({'/commitment_grounds/answer@1':{evidence:['checks@1'],caveats:['uncertain']}}),
{same_as:'reading',paths:['/reading_streams/checks/occurrences']},send('correct'),{resume:true},reject('assess'),send('observe',0),reject('assess')]}
];
writeFileSync('tracker.scenarios.json',JSON.stringify({schema:'caveat-scenarios/0.1',source:'tracker.cav',scenarios},null,2));
copyFileSync('tracker.cav','first.cav');
writeFileSync('first.scenarios.json',JSON.stringify({schema:'caveat-scenarios/0.1',source:'first.cav',scenarios},null,2));
