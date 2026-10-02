import {readFileSync,writeFileSync} from 'node:fs';
const path='tracker.scenarios.json';const doc=JSON.parse(readFileSync(path,'utf8'));
doc.scenarios.push({id:'S8',title:'Reading limit refuses atomically after 16 readings',steps:[{send:'observe',payload:{value:70},repeat:16},{checkpoint:'full'},{send:'observe',payload:{value:100},rejected:{origin:'limit',code:'history_limit'}},{same_as:'full',paths:['/sequence','/observations','/reading_streams','/relations']},{resume:true},{send:'assess'},{expect:{'/commitment_grounds/answer@1/evidence':['checks@16']}}]});
const steps=[];for(let i=0;i<8;i++)steps.push({send:'observe',payload:{value:70+i}},{send:'assess'});
steps.push({send:'observe',payload:{value:99}},{checkpoint:'limit'},{send:'assess',rejected:{origin:'limit',code:'history_limit'}},{same_as:'limit',paths:['/sequence','/values','/decision_series','/commitment_bases','/commitment_grounds']},{resume:true});
doc.scenarios.push({id:'S9',title:'Decision limit rolls back selection state and retains prior history',steps});
writeFileSync(path,JSON.stringify(doc,null,2)+'\n');console.log('Added S8 and S9 to final scenarios only; first files untouched.');
