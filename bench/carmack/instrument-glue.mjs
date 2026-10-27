// Investigation only: copies a runtime directory and makes the session
// constructor check, before calling into wasm, that the source bytes the
// glue wrote are really in wasm memory. A difference throws PASS_MISMATCH.
import { cpSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
const [from, to] = process.argv.slice(2);
cpSync(from, to, { recursive: true });
const file = path.join(to, 'caveat_runtime.js');
const text = readFileSync(file, 'utf8');
const anchor = '        const ret = wasm.webreactivesession_new(ptr0, len0);';
if (text.split(anchor).length !== 2) throw new Error('anchor not found once');
const check = `        {
            const live = wasm.memory.buffer;
            const cached = cachedUint8ArrayMemory0;
            const back = new TextDecoder().decode(new Uint8Array(live, ptr0, len0));
            if (back !== source) {
                let first = 0;
                while (first < back.length && back[first] === source[first]) first += 1;
                throw new Error('PASS_MISMATCH ' + JSON.stringify({ len0, ptr0, firstDiff: first, liveBytes: live.byteLength, cachedBytes: cached ? cached.byteLength : null, cachedIsLive: cached ? cached.buffer === live : null }));
            }
        }
`;
writeFileSync(file, text.replace(anchor, check + anchor));
console.log(`instrumented ${file}`);
