// The starter's Node store: a directory holding checkpoint.json and
// archive.jsonl. The checkpoint is replaced by writing a temporary file,
// syncing it, renaming it over the old one and syncing the directory. Chunks
// are appended one JSON line each and synced before the append resolves. A
// last line without its newline is a torn append that never resolved; opening
// the store cuts it off before anything else is appended.
import { existsSync } from 'node:fs';
import { mkdir, open, readFile, rename } from 'node:fs/promises';
import path from 'node:path';
import { StarterError } from './starter.mjs';

const CHECKPOINT = 'checkpoint.json';
const ARCHIVE = 'archive.jsonl';

async function syncDirectory(directory) {
  // Windows cannot open a directory for syncing; there the rename stands alone.
  if (process.platform === 'win32') return;
  const handle = await open(directory, 'r');
  try { await handle.sync(); } finally { await handle.close(); }
}

async function writeSynced(file, text, flags) {
  const handle = await open(file, flags);
  try {
    await handle.writeFile(text, 'utf8');
    await handle.sync();
  } finally { await handle.close(); }
}

// The archive's complete lines, and the byte length they end at.
async function readArchive(file) {
  if (!existsSync(file)) return { chunks: [], complete: 0 };
  const bytes = await readFile(file);
  const complete = bytes.lastIndexOf(0x0a) + 1;
  const chunks = [];
  for (const line of bytes.subarray(0, complete).toString('utf8').split('\n')) {
    if (!line) continue;
    try { chunks.push(JSON.parse(line)); } catch {
      throw new StarterError('store', `${file} holds a line that is not JSON`);
    }
  }
  return { chunks, complete };
}

export async function fileStore(directory) {
  if (typeof directory !== 'string' || !directory) throw new TypeError('directory must be a path');
  await mkdir(directory, { recursive: true });
  const checkpoint = path.join(directory, CHECKPOINT);
  const archive = path.join(directory, ARCHIVE);
  // Cuts off a torn last line, from a crash or from an append that failed
  // partway, so the next append starts on a line of its own.
  const mend = async () => {
    if (!existsSync(archive)) return;
    const handle = await open(archive, 'r');
    let torn;
    try {
      const { size } = await handle.stat();
      const last = Buffer.alloc(1);
      torn = size > 0 && (await handle.read(last, 0, 1, size - 1)).bytesRead === 1 && last[0] !== 0x0a;
    } finally { await handle.close(); }
    if (!torn) return;
    const { complete } = await readArchive(archive);
    const writable = await open(archive, 'r+');
    try {
      await writable.truncate(complete);
      await writable.sync();
    } finally { await writable.close(); }
  };
  await mend();
  return {
    durable: true,
    directory,
    async readCheckpoint() {
      return existsSync(checkpoint) ? readFile(checkpoint, 'utf8') : null;
    },
    async writeCheckpoint(text) {
      const temporary = `${checkpoint}.tmp`;
      await writeSynced(temporary, text, 'w');
      await rename(temporary, checkpoint);
      await syncDirectory(directory);
    },
    async readChunks() {
      return (await readArchive(archive)).chunks;
    },
    async appendChunks(list) {
      if (!list.length) return;
      await mend();
      await writeSynced(archive, list.map(chunk => `${JSON.stringify(chunk)}\n`).join(''), 'a');
    },
  };
}
