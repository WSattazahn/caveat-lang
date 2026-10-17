// Cloudflare Worker entry for the hosted Caveat MCP server.
import { runAuthoringOperation } from '../node_modules/caveat-lang/lib/authoring.mjs';
import manifest from '../node_modules/caveat-lang/package.json';
import { createMcpHandler } from './handler.mjs';

const serverInfo = { name: 'caveat-lang', version: manifest.version, title: 'CAVEAT Language' };
const home = `CAVEAT Language ${manifest.version} MCP server (Streamable HTTP).
POST JSON-RPC to /mcp. Tools: caveat_validate, caveat_check, caveat_test, caveat_explain, caveat_dependents.
Inline source only; nothing is stored. Terms: https://wsattazahn.github.io/caveat-lang/plugin-terms.html
Privacy: https://wsattazahn.github.io/caveat-lang/plugin-privacy.html
Source: https://github.com/WSattazahn/caveat-lang/tree/main/hosted-mcp
`;
let handler;

export default {
  fetch(request, env) {
    handler ??= createMcpHandler({
      run: (name, args) => runAuthoringOperation(name, args),
      serverInfo, home,
      // About 6 ms of CPU for the starter's 12 steps once warm; the free plan
      // allows 10 ms a request (README.md).
      testSteps: 16,
      // Set with `npx wrangler secret put OPENAI_APPS_CHALLENGE` when OpenAI's
      // portal asks to verify the domain.
      challenge: typeof env?.OPENAI_APPS_CHALLENGE === 'string' && env.OPENAI_APPS_CHALLENGE.trim() ? env.OPENAI_APPS_CHALLENGE.trim() : null,
      // Feature-detected: without the binding the server runs unlimited.
      limiter: typeof env?.LIMITER?.limit === 'function' ? env.LIMITER : null,
    });
    return handler(request);
  },
};
