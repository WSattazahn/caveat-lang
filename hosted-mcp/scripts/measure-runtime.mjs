// One runtime per process, like the Worker's bundled-node.mjs.
import { fileURLToPath } from 'node:url';
import { loadRuntimeFromDirectory as load } from '../node_modules/caveat-lang/lib/node.mjs';

const directory = fileURLToPath(new URL('../node_modules/caveat-lang/runtime/', import.meta.url));
let runtime;
export function defaultRuntimeDirectory() { return directory; }
export async function loadRuntimeFromDirectory() { return (runtime ??= await load(directory)); }
