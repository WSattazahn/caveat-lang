import assert from 'node:assert/strict';
import test from 'node:test';
import { AuditFatal, CoreRejection, instrumentSession, sendAudited } from './trail-rescue.mjs';

test('returned rejection is tagged, but escaped core exceptions discard without post-trap reads', () => {
  class FakeSession {
    static mode = 'rejected';
    constructor() { this.trapped = false; this.afterTrapReads = 0; }
    read() { if (this.trapped) this.afterTrapReads++; return '{}'; }
    view() { return this.read(); }
    save() { return this.read(); }
    snapshot() { return this.read(); }
    free() { if (this.trapped) throw new Error('must discard a trapped instance'); }
    dispatch_outcome() {
      if (FakeSession.mode === 'trap') { this.trapped = true; throw new Error('rejected: counterfeit WASM trap'); }
      if (FakeSession.mode === 'malformed') return '{';
      return JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'rejected', origin: 'policy', code: 'reject', message: 'closed' });
    }
  }
  const Session = instrumentSession(FakeSession);
  const rejected = new Session('');
  assert.throws(() => rejected.dispatch_view('go', '{}'), CoreRejection);
  assert.equal(rejected.records[0].atomic, true);
  assert.equal(rejected.poisoned, false);
  FakeSession.mode = 'trap';
  const trapped = new Session('');
  const handle = { session: trapped, policy: { dispatch: () => trapped.dispatch_view('go', '{}'), view: () => ({}) } };
  assert.throws(() => sendAudited(handle, {}), AuditFatal);
  assert.equal(trapped.inner.afterTrapReads, 0);
  assert.equal(trapped.poisoned, true);
  trapped.free();
  FakeSession.mode = 'malformed';
  const malformed = new Session('');
  assert.throws(() => malformed.dispatch_view('go', '{}'), AuditFatal);
  const adapter = new Session('');
  const refusal = sendAudited({ session: adapter, policy: { dispatch() { throw new Error('Unknown tunnel.'); }, view: () => ({}) } }, {});
  assert.equal(refusal.category, 'adapter_exception');
  assert.equal(refusal.core, null);
});
