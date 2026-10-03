import test from 'node:test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from '../lib/serve.mjs';
import { real } from './helpers.mjs';
import { checkRecoveryClient, RECOVERY_CASES } from '../../scripts/check-recovery-client.mjs';

for (const fixture of RECOVERY_CASES) test(`${fixture.name} survives host, restore and CLI boundaries`, async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-recovery-'));
  try {
    await checkRecoveryClient({ runtime: real, createServer, directory, cases: [fixture],
      command: [process.execPath, fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url))] });
  } finally { await rm(directory, { recursive: true, force: true }); }
});
