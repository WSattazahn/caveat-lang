// A fatal outcome for the tests of the paths that report one. No program is
// known to end its session through evaluation: the bound text past 65,536
// bytes that served here became a refusal (D025, `limit/text_limit`), value
// provenance past its name bytes before that (D022, `limit/provenance_limit`)
// and `id_text` of a number that is no handle before that (F309). So the
// fault is injected at the runtime boundary: a session that throws the fatal
// report, as the WASM bridge throws one for a fault no code classifies, for
// the event named `fault`. `faulting()` is that runtime in this process;
// `faultingRuntimeDirectory()` is the same as a `--runtime` directory for the
// CLI and the authoring worker.
import { copyFile, mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { defaultRuntimeDirectory } from '../lib/node.mjs';
import { faulty } from './helpers.mjs';

export const FATAL_EVENT = 'fault';
export const FATAL_MESSAGE = 'fixture fault: a fault no catalog code classifies';
export const FATAL_REPORT = Object.freeze({ schema: 'caveat-dispatch/0.1', outcome: 'fatal', code: 'unclassified', message: FATAL_MESSAGE });

// The real runtime, with sessions that throw the fatal report when `event`
// is dispatched on either path. Other events reach the real session.
export function faulting(event = FATAL_EVENT) {
  const through = (fake, name, payload, call) => {
    if (name === event) throw JSON.stringify(FATAL_REPORT);
    return fake.inner[call](name, payload);
  };
  return faulty({
    dispatch: (fake, name, payload) => through(fake, name, payload, 'dispatch_outcome'),
    dispatchView: (fake, name, payload) => through(fake, name, payload, 'dispatch_view_outcome'),
  });
}

const root = fileURLToPath(new URL('../../', import.meta.url));
let made = 0;

// A runtime directory for `--runtime`: the real wasm beside a module that
// wraps the real one so that every dispatch of `event` throws the report.
export async function faultingRuntimeDirectory(event = FATAL_EVENT) {
  const real = defaultRuntimeDirectory();
  const directory = path.join(root, 'test-results', 'faulting-runtime', `${process.pid}-${Date.now()}-${made++}`);
  await mkdir(directory, { recursive: true });
  await copyFile(path.join(real, 'caveat_runtime_bg.wasm'), path.join(directory, 'caveat_runtime_bg.wasm'));
  const realModule = pathToFileURL(path.join(real, 'caveat_runtime.js')).href;
  await writeFile(path.join(directory, 'caveat_runtime.js'), `// Written by kit/test/fatal-fixture.mjs: the real runtime, whose sessions
// throw the fatal report for the event named ${JSON.stringify(event)}.
const real = await import(${JSON.stringify(realModule)} + '?faulting=' + encodeURIComponent(import.meta.url));
const REPORT = ${JSON.stringify(JSON.stringify(FATAL_REPORT))};
export class WebReactiveSession extends real.WebReactiveSession {
  static restore(source, saved) {
    const made = real.WebReactiveSession.restore(source, saved);
    Object.setPrototypeOf(made, WebReactiveSession.prototype);
    return made;
  }
}
for (const name of Object.getOwnPropertyNames(real.WebReactiveSession.prototype)) {
  const inner = real.WebReactiveSession.prototype[name];
  if (!name.startsWith('dispatch') || typeof inner !== 'function') continue;
  WebReactiveSession.prototype[name] = function (event, ...rest) {
    if (event === ${JSON.stringify(event)}) throw REPORT;
    return inner.call(this, event, ...rest);
  };
}
export default function init(args) { return real.default(args); }
`);
  return directory;
}
