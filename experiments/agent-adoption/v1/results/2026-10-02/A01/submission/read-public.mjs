import {readFileSync} from 'node:fs';
for (const path of process.argv.slice(2)) {
 console.log('FILE: '+path);
 process.stdout.write(readFileSync(path,'utf8'));
 console.log();
}
