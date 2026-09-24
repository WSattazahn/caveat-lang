// Runs inside VS Code's extension host, started by scripts/editor-check.mjs.
// Opens each file, and records the language VS Code gives it and the tokens
// VS Code's own tokenizer produces for it.
const { readFileSync, writeFileSync } = require('node:fs');
const vscode = require('vscode');

exports.run = async function run() {
  const files = JSON.parse(readFileSync(process.env.CAVEAT_EDITOR_FILES, 'utf8'));
  const extension = vscode.extensions.getExtension('caveat-lang.caveat');
  if (!extension) throw new Error('caveat-lang.caveat is not installed');
  const result = {
    vscode: vscode.version,
    extension: { id: extension.id, version: extension.packageJSON.version, path: extension.extensionPath },
    files: {},
  };
  for (const file of files) {
    const document = await vscode.workspace.openTextDocument(vscode.Uri.file(file));
    // The command VS Code's own colorization tests use: [{c: text, t: scopes}].
    const tokens = await vscode.commands.executeCommand('_workbench.captureSyntaxTokens', document.uri);
    result.files[file] = { languageId: document.languageId, tokens: (tokens ?? []).map(({ c, t }) => [c, t]) };
  }
  writeFileSync(process.env.CAVEAT_EDITOR_RESULT, JSON.stringify(result));
};
