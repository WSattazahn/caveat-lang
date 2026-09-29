// Pinning a child process to a set of logical processors, at a priority.
//
// A mask is a BigInt (bit N = logical processor N) or null for no pinning.
// On this laptop the performance cores are not interchangeable: the same
// bytes run up to a third faster on some than on others (RESULTS.md, core
// placement), so every result records the mask it ran under, and a run can
// rotate through several masks (`--affinity-set`) like targets.
import { spawnSync } from 'node:child_process';

// "0x3C00", "3C00", "none" or "" to a mask.
export function parseMask(text) {
  if (text === null || text === undefined || text === '' || text === 'none') return null;
  const clean = String(text).trim();
  return BigInt(clean.startsWith('0x') || clean.startsWith('0X') ? clean : `0x${clean}`);
}

export const maskLabel = (mask) => (mask === null ? 'none' : `0x${mask.toString(16).toUpperCase()}`);

export function maskProcessors(mask) {
  if (mask === null) return null;
  return [...mask.toString(2)].reverse().flatMap((bit, index) => (bit === '1' ? [index] : []));
}

export const maskOf = (processors) => processors.reduce((mask, processor) => mask | (1n << BigInt(processor)), 0n);

// "0x3,0xC00,0xC00000" to masks, in the order given.
export function parseMaskSet(text) {
  if (!text) return null;
  const masks = text.split(',').filter(Boolean).map(parseMask);
  if (!masks.length || masks.some((mask) => mask === null)) throw new Error(`--affinity-set needs masks, got ${text}`);
  if (new Set(masks.map(maskLabel)).size !== masks.length) throw new Error(`--affinity-set repeats a mask: ${text}`);
  return masks;
}

// Runs command with args, pinned and prioritized: Windows `start /b /wait
// /<priority> /affinity <mask>`, Linux `taskset`.
export function runPinned(command, args, { affinity = null, priority = 'normal', ...options } = {}) {
  const base = { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, timeout: 60 * 60 * 1000, ...options };
  if (process.platform === 'win32' && (affinity !== null || priority !== 'normal')) {
    const flags = [priority === 'normal' ? '' : `/${priority}`, affinity === null ? '' : `/affinity ${affinity.toString(16).toUpperCase()}`].filter(Boolean).join(' ');
    const line = `start "" /b /wait ${flags} ${[command, ...args].map((value) => `"${value}"`).join(' ')}`;
    return spawnSync('cmd.exe', ['/d', '/s', '/c', line], { ...base, windowsVerbatimArguments: true });
  }
  if (process.platform === 'linux' && affinity !== null) return spawnSync('taskset', [`0x${affinity.toString(16)}`, command, ...args], base);
  return spawnSync(command, args, base);
}
