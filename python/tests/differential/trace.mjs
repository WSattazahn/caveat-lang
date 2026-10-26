// Replays scenario files through the npm kit's WebAssembly runtime and prints
// what the runtime returned at every step, as one JSON document. trace.py
// prints the same document from the Python package's native runtime;
// test_differential.py requires the two to be identical.
//
// node trace.mjs RUNTIME_DIR FILE.scenarios.json...
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { payloadText } from '../../../kit/lib/session.mjs';

const [runtimeDir, ...files] = process.argv.slice(2);
const bindings = await import(pathToFileURL(path.join(runtimeDir, 'caveat_runtime.js')).href);
bindings.initSync({ module: readFileSync(path.join(runtimeDir, 'caveat_runtime_bg.wasm')) });
const Session = bindings.WebReactiveSession;

// The runtime's text, or what it threw.
const attempt = read => {
  try { return { text: read() }; } catch (error) { return { error: String(error?.message ?? error) }; }
};

function program(source) {
  return {
    check: attempt(() => Session.check(source)),
    check_unused: attempt(() => Session.check_with(source, true)),
    check_records: attempt(() => Session.check_options(source, JSON.stringify({ unused: false, records: true }))),
    interface: attempt(() => Session.interface(source)),
  };
}

function ending(session, source) {
  const end = {
    snapshot: attempt(() => session.snapshot()),
    view: attempt(() => session.view()),
    view_v2: attempt(() => session.view_v2()),
    save: attempt(() => session.save()),
    save_compact: attempt(() => session.save_compact()),
    save_positional: attempt(() => session.save_compact_positional()),
    undrained: attempt(() => String(session.undrained())),
    archive: attempt(() => session.drain_archive()),
  };
  for (const kind of ['save', 'save_compact', 'save_positional']) {
    if (end[kind].text === undefined) continue;
    end[`${kind}_restored`] = attempt(() => {
      const restored = Session.restore(source, end[kind].text);
      try { return restored.snapshot(); } finally { restored.free(); }
    });
  }
  return end;
}

function scenario(item, source) {
  const steps = [];
  let session = new Session(source);
  let checkpoint = null;
  let variant = 0;
  for (const step of item.steps) {
    if ('send' in step) {
      const payload = 'payload' in step ? step.payload : {};
      const text = payloadText(payload);
      for (let repetition = 0; repetition < (step.repeat ?? 1); repetition++) {
        // Turn through the three dispatch forms so each is compared.
        variant = (variant + 1) % 3;
        const read = 'agent' in step
          ? (variant === 0 ? () => session.dispatch_outcome_as(step.send, text, step.agent)
            : variant === 1 ? () => session.dispatch_view_outcome_as(step.send, text, step.agent)
              : () => session.dispatch_view_delta_outcome_as(step.send, text, step.agent))
          : (variant === 0 ? () => session.dispatch_outcome(step.send, text)
            : variant === 1 ? () => session.dispatch_view_outcome(step.send, text)
              : () => session.dispatch_view_delta_outcome(step.send, text));
        const outcome = attempt(read);
        steps.push({ payload: text, outcome });
        if (outcome.error !== undefined) return { steps, fatal: true };
      }
    } else if ('checkpoint' in step) {
      checkpoint = attempt(() => session.save());
      steps.push({ checkpoint });
    } else if ('resume' in step && checkpoint?.text !== undefined) {
      const resumed = attempt(() => Session.restore(source, checkpoint.text));
      steps.push({ resume: resumed.error ?? 'ok' });
      if (resumed.error === undefined) {
        session.free();
        session = resumed.text; // the restored session
      }
    }
  }
  const end = ending(session, source);
  session.free();
  return { steps, end };
}

const out = [];
for (const file of files) {
  const doc = JSON.parse(readFileSync(file, 'utf8'));
  const sourceOf = name => readFileSync(path.resolve(path.dirname(file), name), 'utf8');
  const fileSource = sourceOf(doc.source);
  const entry = { file, program: program(fileSource), scenarios: [] };
  for (const item of doc.scenarios) {
    const source = item.source ? sourceOf(item.source) : fileSource;
    let result;
    try { result = scenario(item, source); } catch (error) { result = { load: String(error?.message ?? error) }; }
    entry.scenarios.push({ id: item.id, ...result });
  }
  out.push(entry);
}
process.stdout.write(JSON.stringify(out));
