// One invocation, one runtime, one JSON result. Imported only for declaration
// checks; direct execution reads the bounded request and exits after one job.
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { AUTHORING_LIMITS, AuthoringError, runAuthoringOperation } from './authoring.mjs';

async function main() {
  try {
    if (process.argv.length !== 3) throw new AuthoringError('input', 'worker needs one trusted runtime directory');
    const chunks = [];
    let size = 0;
    for await (const chunk of process.stdin) {
      size += chunk.length;
      if (size > AUTHORING_LIMITS.messageBytes) throw new AuthoringError('limit', 'worker input exceeds 4 MiB UTF-8');
      chunks.push(chunk);
    }
    let request;
    try { request = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(Buffer.concat(chunks))); }
    catch { throw new AuthoringError('input', 'worker input must be one UTF-8 JSON object'); }
    if (!request || typeof request !== 'object' || Array.isArray(request)
      || Object.keys(request).some(key => !['tool', 'arguments'].includes(key))) {
      throw new AuthoringError('input', 'worker request must contain only tool and arguments');
    }
    const result = await runAuthoringOperation(request.tool, request.arguments, { runtimeDirectory: process.argv[2] });
    const output = JSON.stringify(result);
    if (Buffer.byteLength(output, 'utf8') > AUTHORING_LIMITS.outputBytes) throw new AuthoringError('limit', 'worker result exceeds 4 MiB UTF-8');
    process.stdout.write(`${output}\n`);
  } catch (error) {
    process.exitCode = 1;
    process.stdout.write(`${JSON.stringify({ schema: 'caveat-authoring-error/0.1', error: {
      kind: error instanceof AuthoringError ? error.kind : 'runtime', message: error.message,
    } })}\n`);
  }
}

if (process.argv[1] && pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url) await main();
