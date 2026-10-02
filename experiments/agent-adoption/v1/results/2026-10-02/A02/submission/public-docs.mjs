import {readFileSync} from 'node:fs';
const paths=process.argv.slice(2);
for(const path of paths) console.log(JSON.stringify({path,text:readFileSync(path,'utf8')}));
