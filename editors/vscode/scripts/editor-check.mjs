// Checks the packaged extension in VS Code itself. Installs the .vsix from
// `npm run package` into a fresh profile, opens every tracked .cav file and
// the fixture, and requires that VS Code gives each the Caveat language and
// tokenizes it exactly as the tests' tokenizer does.
//
// CAVEAT_VSCODE names a VS Code executable to use. Without it, VS Code is
// downloaded: CAVEAT_VSCODE_VERSION, or else the oldest version the manifest
// supports. On Linux without a display, run this under xvfb-run.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { downloadAndUnzipVSCode, resolveCliPathFromVSCodeExecutablePath, runTests } from '@vscode/test-electron';
import { root, trackedCavFiles } from '../test/corpus.mjs';
import { difference } from '../test/editor/compare.mjs';
import { scopesOf } from '../test/tokenize.mjs';

const here = fileURLToPath(new URL('../', import.meta.url));
const manifest = JSON.parse(readFileSync(join(here, 'package.json'), 'utf8'));
const vsix = join(root, 'test-results', 'vscode', `${manifest.name}-${manifest.version}.vsix`);
assert.ok(existsSync(vsix), `${vsix} is missing; run npm run package first`);

const version = process.env.CAVEAT_VSCODE_VERSION ?? manifest.engines.vscode.replace(/^\^/, '');
const executable = process.env.CAVEAT_VSCODE
  || await downloadAndUnzipVSCode({ version, cachePath: join(root, 'test-results', 'vscode-download') });
const label = process.env.CAVEAT_VSCODE ? 'local' : version;
const work = join(root, 'test-results', 'vscode-editor', label);
rmSync(work, { recursive: true, force: true });
mkdirSync(work, { recursive: true });
const profile = [`--extensions-dir=${join(work, 'extensions')}`, `--user-data-dir=${join(work, 'user-data')}`];

// Install the package the way a user would, with VS Code's command line.
const cli = resolveCliPathFromVSCodeExecutablePath(executable);
const shell = process.platform === 'win32';
const quote = arg => (shell ? `"${arg}"` : arg);
const install = spawnSync(quote(cli), [...profile, '--install-extension', vsix].map(quote), { shell, encoding: 'utf8' });
assert.equal(install.status, 0, `install failed:\n${install.stdout}\n${install.stderr}`);

const files = [...new Set([...trackedCavFiles().map(file => join(root, file)), join(here, 'test', 'fixtures', 'scopes.cav')])];
const listed = join(work, 'files.json');
const resultPath = join(work, 'result.json');
writeFileSync(listed, JSON.stringify(files));
await runTests({
  vscodeExecutablePath: executable,
  extensionDevelopmentPath: join(here, 'test', 'editor', 'harness'),
  extensionTestsPath: join(here, 'test', 'editor', 'suite.cjs'),
  launchArgs: [...profile, '--disable-gpu', '--disable-telemetry'],
  extensionTestsEnv: { CAVEAT_EDITOR_FILES: listed, CAVEAT_EDITOR_RESULT: resultPath },
});

const result = JSON.parse(readFileSync(resultPath, 'utf8'));
assert.equal(result.extension.version, manifest.version);
assert.ok(result.extension.path.startsWith(join(work, 'extensions')), `the extension was not loaded from the installed .vsix: ${result.extension.path}`);

let codeUnits = 0;
for (const file of files) {
  const { languageId, tokens } = result.files[file];
  assert.equal(languageId, 'caveat', `${file} opened as ${languageId}`);
  const text = readFileSync(file, 'utf8').replace(/\r\n/g, '\n');
  const differs = difference(text, await scopesOf(text), tokens);
  assert.equal(differs, null, `${file}: ${differs?.message}`);
  codeUnits += text.replace(/\n/g, '').length;
}
const summary = { vscode: result.vscode, extension: result.extension.id, files: files.length, codeUnits };
writeFileSync(join(work, 'summary.json'), `${JSON.stringify(summary, null, 2)}\n`);
console.log(`VS Code ${result.vscode}: ${files.length} files, ${codeUnits} UTF-16 code units tokenized as the tests expect`);
