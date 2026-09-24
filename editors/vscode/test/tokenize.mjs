// Tokenizes text with the engine VS Code uses: vscode-textmate over
// vscode-oniguruma, one line at a time, carrying the rule stack across lines.
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import textmate from 'vscode-textmate';
import oniguruma from 'vscode-oniguruma';

const require = createRequire(import.meta.url);
export const grammarPath = new URL('../syntaxes/caveat.tmLanguage.json', import.meta.url);

let loaded;

export function loadGrammar() {
  loaded ??= (async () => {
    const wasm = readFileSync(require.resolve('vscode-oniguruma/release/onig.wasm'));
    await oniguruma.loadWASM({ data: wasm });
    const registry = new textmate.Registry({
      onigLib: Promise.resolve({
        createOnigScanner: patterns => new oniguruma.OnigScanner(patterns),
        createOnigString: text => new oniguruma.OnigString(text),
      }),
      loadGrammar: async scopeName => scopeName === 'source.caveat'
        ? textmate.parseRawGrammar(readFileSync(grammarPath, 'utf8'), grammarPath.pathname)
        : null,
    });
    return registry.loadGrammar('source.caveat');
  })();
  return loaded;
}

// The scopes of every character of `text` (line breaks get an empty list,
// as the editor never tokenizes them).
export async function scopesOf(text) {
  const grammar = await loadGrammar();
  const scopes = new Array(text.length).fill(null).map(() => []);
  let stack = textmate.INITIAL;
  let offset = 0;
  for (const line of text.split('\n')) {
    const result = grammar.tokenizeLine(line, stack);
    for (const token of result.tokens) {
      for (let index = token.startIndex; index < Math.min(token.endIndex, line.length); index++) {
        scopes[offset + index] = token.scopes;
      }
    }
    stack = result.ruleStack;
    offset += line.length + 1;
  }
  return scopes;
}

export const has = (scopes, prefix) => scopes.some(scope => scope === prefix || scope.startsWith(`${prefix}.`));
