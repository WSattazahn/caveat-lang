// Shared test fixtures: the repository's reactive build and a wrapper that
// lets a test make one session misbehave in a chosen way.
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { createRuntime } from '../lib/session.mjs';
import { loadRuntimeFromDirectory, defaultRuntimeDirectory } from '../lib/node.mjs';

export const repo = fileURLToPath(new URL('../../', import.meta.url));
export const real = await loadRuntimeFromDirectory(defaultRuntimeDirectory());
const namespace = await import(new URL(`../../dist/pkg-reactive/caveat_runtime.js?instance=helpers`, import.meta.url).href);
await namespace.default({ module_or_path: await readFile(new URL('../../dist/pkg-reactive/caveat_runtime_bg.wasm', import.meta.url)) });
export const Real = namespace.WebReactiveSession;

export const thermostat = await readFile(new URL('../../examples/thermostat_history.cav', import.meta.url), 'utf8');
export const trail = await readFile(new URL('../../game/trail_rescue.cav', import.meta.url), 'utf8');

// hooks receive the fake session (with .inner, .restored, .calls) and may
// replace any runtime call. Unhooked calls go to the real session.
export function faulty(hooks = {}) {
  let created = 0;
  class Fake {
    constructor(source, inner) {
      this.inner = inner ?? new Real(source);
      this.number = ++created;
      this.restored = Boolean(inner);
      this.calls = [];
    }
    static restore(source, saved) {
      if (hooks.restore) return new Fake(source, hooks.restore(source, saved));
      return new Fake(source, Real.restore(source, saved));
    }
    dispatch_outcome(event, payload) {
      this.calls.push('dispatch');
      return hooks.dispatch ? hooks.dispatch(this, event, payload) : this.inner.dispatch_outcome(event, payload);
    }
    snapshot() { this.calls.push('snapshot'); return hooks.snapshot ? hooks.snapshot(this) : this.inner.snapshot(); }
    view() { this.calls.push('view'); return hooks.view ? hooks.view(this) : this.inner.view(); }
    save() { this.calls.push('save'); return hooks.save ? hooks.save(this) : this.inner.save(); }
    free() { this.calls.push('free'); if (hooks.free) return hooks.free(this); return this.inner.free(); }
  }
  const sessions = [];
  const Tracked = class extends Fake {
    constructor(...args) { super(...args); sessions.push(this); }
    static restore(source, saved) {
      const made = Fake.restore(source, saved);
      Object.setPrototypeOf(made, Tracked.prototype);
      sessions.push(made);
      return made;
    }
  };
  return { runtime: createRuntime(Tracked), sessions };
}

export function scenarioFile(source, steps, extra = {}) {
  return { schema: 'caveat-scenarios/0.1', source, scenarios: [{ id: 'S1', title: 'test', steps, ...extra }] };
}

export const sourceReader = texts => async path => {
  if (!(path in texts)) throw new Error(`no source ${path}`);
  return texts[path];
};
