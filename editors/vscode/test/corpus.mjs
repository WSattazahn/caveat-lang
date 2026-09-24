// The tracked .cav files of the repository, found with git.
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

export const root = fileURLToPath(new URL('../../../', import.meta.url));

export function trackedCavFiles() {
  return execFileSync('git', ['ls-files', '-z', '--', '*.cav'], { cwd: root, encoding: 'utf8' })
    .split('\0')
    .filter(Boolean);
}
