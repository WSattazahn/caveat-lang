import { readFile } from 'node:fs/promises';
import init, { WebReactiveSession } from '../../dist/pkg-reactive/caveat_runtime.js';
import { createPolicyFromSession } from '../../web/trail-rescue-policy.js';

export const source = await readFile(new URL('../../game/trail_rescue.cav', import.meta.url), 'utf8');
export const ready = init({ module_or_path: await readFile(new URL('../../dist/pkg-reactive/caveat_runtime_bg.wasm', import.meta.url)) });
export function createPolicy(saved) {
  return createPolicyFromSession(WebReactiveSession, source, saved);
}
export function createSourcePolicy(program, saved) {
  return createPolicyFromSession(WebReactiveSession, program, saved);
}
