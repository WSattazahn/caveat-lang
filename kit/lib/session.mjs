// One way to load the reactive runtime, dispatch events, read results, and
// save or resume a session. This module has no Node imports; see node.mjs for
// loading the runtime from a directory. Outcomes follow
// spec/caveat-dispatch-0.1.md: accepted and rejected are values, and anything
// else is a CaveatError of kind "fatal" after which the session is unusable.

export const DISPATCH_SCHEMA = 'caveat-dispatch/0.1';
export const ORIGINS = Object.freeze(['policy', 'input', 'evaluation', 'limit']);

// kind: "load", "restore", "payload", "fatal" or "closed".
export class CaveatError extends Error {
  constructor(kind, message, report = null) {
    super(message);
    this.name = 'CaveatError';
    this.kind = kind;
    this.report = report;
  }
}

const messageOf = error => (typeof error === 'string' ? error : String(error?.message ?? error));

// A WebAssembly trap leaves the whole instance in an unknown state, not just
// one session. The runtime refuses further use and must be loaded again.
const isTrap = error => typeof WebAssembly !== 'undefined' && error instanceof WebAssembly.RuntimeError;
// The first line of a module bundle (spec/caveat-0.5-draft.md).
const BUNDLE_MARKER = '#caveat-bundle 1';

function fatalFrom(error) {
  const text = messageOf(error);
  let report = null;
  try {
    const parsed = JSON.parse(text);
    if (parsed && parsed.schema === DISPATCH_SCHEMA && parsed.outcome === 'fatal') report = parsed;
  } catch { /* not a serialized report */ }
  return new CaveatError('fatal', report ? `${report.code}: ${report.message}` : text, report);
}

function isPlainObject(value) {
  if (value === null || typeof value !== 'object') return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function checkPayload(value, at) {
  if (value === null || typeof value === 'string' || typeof value === 'boolean') return;
  if (typeof value === 'number') {
    if (!Number.isFinite(value)) throw new CaveatError('payload', `payload ${at || 'value'} is not a finite number`);
    return;
  }
  if (Array.isArray(value)) {
    for (let index = 0; index < value.length; index++) {
      if (!Object.hasOwn(value, index)) throw new CaveatError('payload', `payload ${at}/${index} is a hole; JSON would send null`);
      checkPayload(value[index], `${at}/${index}`);
    }
    return;
  }
  if (isPlainObject(value)) {
    for (const key in value) {
      if (Object.hasOwn(value, key)) checkPayload(value[key], `${at}/${key}`);
    }
    return;
  }
  throw new CaveatError('payload', `payload ${at || 'value'} cannot be sent as JSON`);
}

// Payloads must survive standard JSON unchanged apart from negative zero, which
// JSON writes as 0. Anything else (NaN, Infinity, undefined, functions, class
// instances, holes in arrays) is refused before the session is touched.
export function payloadText(payload) {
  checkPayload(payload, '');
  return JSON.stringify(payload);
}

// accepted names what an accepted outcome carries: dispatch() the snapshot,
// dispatchView() the view, dispatchViewDelta() the delta.
// The view fields dispatchView can select, in the view's order.
export const VIEW_FIELDS = Object.freeze([
  'bindings', 'binding_explanations', 'cues', 'effects', 'commitments',
  'commitment_grounds', 'decision_series', 'decision_journal', 'relations',
]);

// dispatchView's field selection as the JSON text the runtime takes, or null
// for the whole view. Checked before the runtime is called.
function viewFields(options) {
  if (options === undefined || options === null) return null;
  if (typeof options !== 'object') throw new TypeError('dispatchView options must be an object');
  const { fields } = options;
  if (fields === undefined) return null;
  if (!Array.isArray(fields)) throw new TypeError('fields must be an array of view field names');
  const seen = new Set();
  for (const field of fields) {
    if (!VIEW_FIELDS.includes(field)) {
      throw new TypeError(`unknown view field ${JSON.stringify(field)}; the fields are ${VIEW_FIELDS.join(', ')}`);
    }
    if (seen.has(field)) throw new TypeError(`view field ${JSON.stringify(field)} is named twice`);
    seen.add(field);
  }
  return JSON.stringify(fields);
}

// The agent an options object names, or null when it names none. See
// spec/caveat-agents-0.1.md. Checked before the runtime is called.
function agentOption(options) {
  if (options === undefined || options === null) return null;
  if (typeof options !== 'object') throw new TypeError('dispatch options must be an object');
  const { agent } = options;
  if (agent === undefined || agent === null) return null;
  if (typeof agent !== 'string' || !agent) throw new TypeError('agent must be a non-empty string');
  return agent;
}

// A historical interpretation is an explicit host choice. Empty options keep
// the ordinary restore path, including on runtime builds without this feature.
function restoreOptionsText(options) {
  if (options === undefined) return null;
  if (!isPlainObject(options)) throw new TypeError('restore options must be an object');
  const unknown = Reflect.ownKeys(options).find(key => key !== 'moduleBindings');
  if (unknown !== undefined) throw new TypeError(`unknown restore option ${String(unknown)}`);
  const moduleBindings = Object.hasOwn(options, 'moduleBindings') ? options.moduleBindings : undefined;
  if (moduleBindings === undefined) return null;
  if (moduleBindings !== 'scoped' && moduleBindings !== 'legacy-hold-capture') {
    throw new TypeError('moduleBindings must be "scoped" or "legacy-hold-capture"');
  }
  return JSON.stringify({ moduleBindings });
}

// A runtime build that predates agents throws CaveatError("load") for an
// event sent with one, and the session is untouched.
const predatesAgents = method => new CaveatError('load', `this runtime build has no ${method}; it predates agents`);

function validOutcome(outcome, accepted = 'snapshot') {
  if (!outcome || outcome.schema !== DISPATCH_SCHEMA) return false;
  if (outcome.outcome === 'accepted') return isPlainObject(outcome[accepted]);
  if (outcome.outcome === 'rejected') {
    return ORIGINS.includes(outcome.origin) && typeof outcome.code === 'string' && typeof outcome.message === 'string';
  }
  return false;
}

// dispatchView's reuse option: true, or false when it is absent. Checked
// before the runtime is called.
function reuseOption(options) {
  if (options === undefined || options === null) return false;
  const { reuse } = options;
  if (reuse === undefined || reuse === false) return false;
  if (reuse !== true) throw new TypeError('reuse must be true or false');
  return true;
}

// The View 0.1 patch (spec/caveat-view-patch-0.1.md) applied to the view the
// session holds, or to nothing when the patch says it applies to nothing.
// Every member the event left alone is the held one; every new part is
// frozen, so the whole view is read-only.
const VIEW_PATCH_SCHEMA = 'caveat-reactive-view-patch/0.1';

// Fast object check for JSON.parse'd patch trees (every non-null, non-array
// object from JSON.parse is a plain object).
function isJsonObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function deepFreeze(value) {
  if (value !== null && typeof value === 'object' && !Object.isFrozen(value)) {
    if (Array.isArray(value)) {
      for (let i = 0; i < value.length; i += 1) deepFreeze(value[i]);
    } else {
      for (const key in value) {
        if (Object.hasOwn(value, key)) deepFreeze(value[key]);
      }
    }
    Object.freeze(value);
  }
  return value;
}

// Freeze a freshly JSON.parse'd tree: every node is an own-property object or
// array and not yet frozen.
function deepFreezeNew(value) {
  if (value === null || typeof value !== 'object') return value;
  if (Array.isArray(value)) {
    for (let i = 0; i < value.length; i += 1) deepFreezeNew(value[i]);
  } else {
    for (const key in value) {
      if (Object.hasOwn(value, key)) deepFreezeNew(value[key]);
    }
  }
  return Object.freeze(value);
}

// Set a member as JSON.parse would: an own property, even for __proto__.
function put(target, key, value) {
  if (key === '__proto__') Object.defineProperty(target, key, { value, enumerable: true, writable: true, configurable: true });
  else target[key] = value;
}

function own(object, key) {
  return object !== undefined && object !== null && Object.hasOwn(object, key) ? object[key] : undefined;
}

function patchedMap(held, change, nested) {
  if (!isJsonObject(change) || !isJsonObject(change.set)) throw new TypeError('malformed map change');
  if (change.keys === undefined && held !== undefined && held !== null && !Object.hasOwn(held, '__proto__')) {
    const result = { ...held };
    const { set } = change;
    for (const key in set) {
      if (!Object.hasOwn(set, key) || !Object.hasOwn(held, key)) continue;
      const entry = set[key];
      result[key] = nested ? patchedMap(held[key], entry, false) : deepFreezeNew(entry);
    }
    return Object.freeze(result);
  }
  const keys = change.keys ?? Object.keys(held ?? {});
  if (!Array.isArray(keys)) throw new TypeError('malformed map keys');
  const result = {};
  for (const key of keys) {
    if (Object.hasOwn(change.set, key)) {
      const entry = change.set[key];
      put(result, key, nested ? patchedMap(own(held, key), entry, false) : deepFreezeNew(entry));
    } else {
      if (held === undefined || held === null || !Object.hasOwn(held, key)) throw new TypeError(`map key ${JSON.stringify(key)} is neither held nor set`);
      put(result, key, held[key]);
    }
  }
  return Object.freeze(result);
}

function patchedList(held, change) {
  if (!isJsonObject(change) || !Array.isArray(change.items)) throw new TypeError('malformed list change');
  const { items } = change;
  const len = items.length;
  if (len > 0 && Array.isArray(items[0])) {
    const [start, length] = items[0];
    if (!held || !Number.isInteger(start) || !Number.isInteger(length) || start < 0 || length < 1 || start + length > held.length) throw new TypeError('malformed list run');
    if (len === 2 && !Array.isArray(items[1])) {
      const result = held.slice(start, start + length);
      result.push(deepFreezeNew(items[1]));
      return Object.freeze(result);
    }
    let allNewAfter = true;
    for (let i = 1; i < len; i += 1) {
      if (Array.isArray(items[i])) { allNewAfter = false; break; }
    }
    if (allNewAfter) {
      const result = held.slice(start, start + length);
      for (let i = 1; i < len; i += 1) result.push(deepFreezeNew(items[i]));
      return Object.freeze(result);
    }
  }
  const result = [];
  for (let i = 0; i < len; i += 1) {
    const item = items[i];
    if (Array.isArray(item)) {
      const [start, length] = item;
      if (!held || !Number.isInteger(start) || !Number.isInteger(length) || start < 0 || length < 1 || start + length > held.length) throw new TypeError('malformed list run');
      for (let index = start; index < start + length; index += 1) result.push(held[index]);
    } else {
      result.push(deepFreezeNew(item));
    }
  }
  return Object.freeze(result);
}

// Commitments are a list in the view and a map by action in the patch.
function patchedCommitments(held, change) {
  if (!isJsonObject(change) || !isJsonObject(change.set)) throw new TypeError('malformed map change');
  if (change.keys === undefined && held) {
    const { set } = change;
    const result = new Array(held.length);
    for (let i = 0; i < held.length; i += 1) {
      const commitment = held[i];
      result[i] = Object.hasOwn(set, commitment.action) ? deepFreezeNew(set[commitment.action]) : commitment;
    }
    return Object.freeze(result);
  }
  let byAction;
  if (held) {
    byAction = {};
    for (let i = 0; i < held.length; i += 1) put(byAction, held[i].action, held[i]);
  }
  const map = patchedMap(byAction, change, false);
  return Object.freeze(Object.keys(map).length === 0 ? [] : (change.keys ?? held.map(c => c.action)).map(action => map[action]));
}

function patchedWhole(change) {
  if (!isJsonObject(change) || !Object.hasOwn(change, 'value')) throw new TypeError('malformed whole change');
  return deepFreezeNew(change.value);
}

function patchedView(held, patch) {
  if (!isJsonObject(patch) || patch.schema !== VIEW_PATCH_SCHEMA || !isJsonObject(patch.members)) throw new TypeError('malformed view patch');
  const base = patch.since === null ? null : held;
  if (patch.since !== null && (!held || held.sequence !== patch.since)) throw new TypeError('view patch applies to a view not held');
  const m = patch.members;
  let bindings, binding_explanations, cues, effects, commitments, commitment_grounds, decision_series, decision_journal, relations;
  if (Object.hasOwn(m, 'bindings')) bindings = patchedMap(base?.bindings, m.bindings, true);
  else if (base) bindings = base.bindings;
  else throw new TypeError('view patch leaves out bindings');
  if (Object.hasOwn(m, 'binding_explanations')) binding_explanations = patchedMap(base?.binding_explanations, m.binding_explanations, true);
  else if (base) binding_explanations = base.binding_explanations;
  else throw new TypeError('view patch leaves out binding_explanations');
  if (Object.hasOwn(m, 'cues')) cues = patchedWhole(m.cues);
  else if (base) cues = base.cues;
  else throw new TypeError('view patch leaves out cues');
  if (Object.hasOwn(m, 'effects')) effects = patchedWhole(m.effects);
  else if (base) effects = base.effects;
  else throw new TypeError('view patch leaves out effects');
  if (Object.hasOwn(m, 'commitments')) commitments = patchedCommitments(base?.commitments, m.commitments);
  else if (base) commitments = base.commitments;
  else throw new TypeError('view patch leaves out commitments');
  if (Object.hasOwn(m, 'commitment_grounds')) commitment_grounds = patchedMap(base?.commitment_grounds, m.commitment_grounds, false);
  else if (base) commitment_grounds = base.commitment_grounds;
  else throw new TypeError('view patch leaves out commitment_grounds');
  if (Object.hasOwn(m, 'decision_series')) decision_series = patchedMap(base?.decision_series, m.decision_series, false);
  else if (base) decision_series = base.decision_series;
  else throw new TypeError('view patch leaves out decision_series');
  if (Object.hasOwn(m, 'decision_journal')) decision_journal = patchedList(base?.decision_journal, m.decision_journal);
  else if (base) decision_journal = base.decision_journal;
  else throw new TypeError('view patch leaves out decision_journal');
  if (Object.hasOwn(m, 'relations')) relations = patchedList(base?.relations, m.relations);
  else if (base) relations = base.relations;
  else throw new TypeError('view patch leaves out relations');
  return Object.freeze({
    schema: patch.view_schema,
    sequence: patch.sequence,
    last_event: patch.last_event,
    bindings,
    binding_explanations,
    cues,
    effects,
    commitments,
    commitment_grounds,
    decision_series,
    decision_journal,
    relations,
  });
}

// ------------------------------------------------ binary view-patch decoder

const VIEW_SCHEMA_0_1 = 'caveat-reactive-view/0.1';
const EMPTY_ARRAY = Object.freeze([]);
const RELATION_NAMES = ['supports', 'opposes', 'qualifies', 'in_context', 'retains', 'reopens', 'relies_on'];
const UTF8_DECODER = new TextDecoder('utf-8', { fatal: true });
const F64_BYTES = new Uint8Array(8);
const F64_VIEW = new Float64Array(F64_BYTES.buffer);
const F64_DV = new DataView(F64_BYTES.buffer);
const LITTLE_ENDIAN = new Uint8Array(new Uint16Array([1]).buffer)[0] === 1;

function decodeBinaryViewPatch(buf, held) {
  if (!(buf instanceof Uint8Array) || buf.length === 0) throw new TypeError('malformed binary view patch');
  let pos = 0;
  const end = buf.length;

  function readByte() {
    if (pos >= end) throw new TypeError('truncated binary view patch');
    return buf[pos++];
  }

  function readVarint() {
    if (pos >= end) throw new TypeError('truncated binary view patch');
    let b = buf[pos++];
    if (b < 0x80) return b;
    let val = b & 0x7f;
    let shift = 7;
    for (;;) {
      if (pos >= end) throw new TypeError('truncated binary view patch');
      b = buf[pos++];
      if (shift === 63) {
        if (b > 1) throw new TypeError('varint overflow in binary view patch');
        return val + b * (2 ** 63);
      }
      if (b < 0x80) return shift < 28 ? val | (b << shift) : val + b * (2 ** shift);
      val = shift < 28 ? val | ((b & 0x7f) << shift) : val + (b & 0x7f) * (2 ** shift);
      shift += 7;
    }
  }

  function readF64() {
    if (pos + 8 > end) throw new TypeError('truncated binary view patch');
    F64_BYTES[0] = buf[pos];
    F64_BYTES[1] = buf[pos + 1];
    F64_BYTES[2] = buf[pos + 2];
    F64_BYTES[3] = buf[pos + 3];
    F64_BYTES[4] = buf[pos + 4];
    F64_BYTES[5] = buf[pos + 5];
    F64_BYTES[6] = buf[pos + 6];
    F64_BYTES[7] = buf[pos + 7];
    pos += 8;
    const value = LITTLE_ENDIAN ? F64_VIEW[0] : F64_DV.getFloat64(0, true);
    if (!Number.isFinite(value)) throw new TypeError('non-finite number in binary view patch');
    return value;
  }

  function readStr() {
    const tag = readVarint();
    if (tag > 0xffffffff) throw new TypeError('string too large in binary view patch');
    const len = tag >>> 1;
    if (len === 0) return '';
    if (pos + len > end) throw new TypeError('truncated string in binary view patch');
    const start = pos;
    pos += len;
    if ((tag & 1) === 0 && len <= 48) {
      let s = '';
      let i = start;
      const limit = pos;
      while (i + 8 <= limit) {
        s += String.fromCharCode(
          buf[i], buf[i + 1], buf[i + 2], buf[i + 3],
          buf[i + 4], buf[i + 5], buf[i + 6], buf[i + 7],
        );
        i += 8;
      }
      while (i < limit) s += String.fromCharCode(buf[i++]);
      return s;
    }
    return UTF8_DECODER.decode(buf.subarray(start, pos));
  }

  function readOptStr() {
    const tag = readByte();
    if (tag === 0) return null;
    if (tag === 1) return readStr();
    throw new TypeError('malformed optional string tag');
  }

  function readStrList() {
    const len = readVarint();
    if (len === 0) return EMPTY_ARRAY;
    if (len > end - pos) throw new TypeError('truncated string list in binary view patch');
    const arr = new Array(len);
    for (let i = 0; i < len; i += 1) arr[i] = readStr();
    return Object.freeze(arr);
  }

  function readBindingValue() {
    const tag = readByte();
    if (tag === 0) return readF64();
    if (tag === 1) return false;
    if (tag === 2) return true;
    if (tag === 3) return readStr();
    throw new TypeError('malformed binding value');
  }

  function readProvenance() {
    const flags = readByte();
    if ((flags & ~7) !== 0) throw new TypeError('malformed provenance flags');
    const evidence = readStrList();
    const caveats = readStrList();
    if (flags === 0) return Object.freeze({ evidence, caveats });
    const obj = { evidence, caveats };
    if (flags & 1) {
      const n = readVarint();
      if (n > end - pos) throw new TypeError('truncated departed list in binary view patch');
      const departed = new Array(n);
      for (let i = 0; i < n; i += 1) {
        const history = readStr();
        const read = readVarint();
        const from = readVarint();
        const through = readVarint();
        const departed_at = readVarint();
        const archive_ref = readOptStr();
        const marker = archive_ref === null
          ? { history, read, from, through, departed_at }
          : { history, read, from, through, departed_at, archive_ref };
        departed[i] = Object.freeze(marker);
      }
      obj.departed = Object.freeze(departed);
    }
    if (flags & 2) obj.inherited = readStrList();
    if (flags & 4) obj.kept = readStrList();
    return Object.freeze(obj);
  }

  function readEffect() {
    const kind = readByte();
    if (kind === 0) {
      const stream = readStr();
      const id = readStr();
      const value = readF64();
      const flags = readByte();
      if ((flags & ~7) !== 0) throw new TypeError('malformed sample effect flags');
      const obj = { kind: 'sample', stream, id, value };
      if (flags & 1) obj.relation = readStr();
      if (flags & 2) obj.target = readStr();
      if (flags & 4) obj.note = readStr();
      return Object.freeze(obj);
    }
    if (kind === 1) {
      const evidence = readStr();
      const flags = readByte();
      if ((flags & ~3) !== 0) throw new TypeError('malformed reveal effect flags');
      const obj = { kind: 'reveal', evidence };
      if (flags & 1) obj.relation = readStr();
      if (flags & 2) obj.target = readStr();
      return Object.freeze(obj);
    }
    if (kind === 2) return Object.freeze({ kind: 'examine', caveat: readStr(), cost: readVarint() });
    if (kind === 3) return Object.freeze({ kind: 'commit', action: readStr(), retained: readStrList() });
    if (kind === 4) return Object.freeze({ kind: 'reopen', action: readStr(), because: readStr() });
    if (kind === 5) return Object.freeze({ kind: 'qualify', evidence: readStr(), caveat: readStr() });
    if (kind === 6) return Object.freeze({ kind: 'unqualify', evidence: readStr(), caveat: readStr(), occurrences: readStrList() });
    if (kind === 7) return Object.freeze({ kind: 'renew', evidence: readStr(), occurrence: readStr() });
    if (kind === 8) return Object.freeze({ kind: 'withdraw', evidence: readStr(), because: readStr() });
    if (kind === 9) return Object.freeze({ kind: 'retire', history: readStr(), record: readStr() });
    if (kind === 10) return Object.freeze({ kind: 'depart', history: readStr(), record: readStr() });
    throw new TypeError('malformed effect');
  }

  function readCommitment() {
    const action = readStr();
    const openByte = readByte();
    if (openByte !== 0 && openByte !== 1) throw new TypeError('malformed commitment open flag');
    const open = openByte === 1;
    const retained = readStrList();
    const authLen = readVarint();
    let retained_authorship;
    if (authLen === 0) {
      retained_authorship = EMPTY_ARRAY;
    } else {
      if (authLen > end - pos) throw new TypeError('truncated retained authorship in binary view patch');
      retained_authorship = new Array(authLen);
      for (let i = 0; i < authLen; i += 1) {
        const caveat = readStr();
        const written_by = readOptStr();
        const weTag = readByte();
        if (weTag > 2) throw new TypeError('malformed written_elsewhere tag');
        const written_elsewhere = weTag === 0 ? null : weTag === 2;
        retained_authorship[i] = Object.freeze({ caveat, written_by, written_elsewhere });
      }
      Object.freeze(retained_authorship);
    }
    const reopened_by = readStrList();
    return Object.freeze({ action, open, retained, retained_authorship, reopened_by });
  }

  function readDecisionSeries() {
    const limit = readVarint();
    const current = readOptStr();
    const revLen = readVarint();
    let revisions;
    if (revLen === 0) {
      revisions = EMPTY_ARRAY;
    } else {
      if (revLen > end - pos) throw new TypeError('truncated revisions in binary view patch');
      revisions = new Array(revLen);
      for (let i = 0; i < revLen; i += 1) {
        const id = readStr();
        const previous = readOptStr();
        const ordinal = readVarint();
        const sequence = readVarint();
        const event = readStr();
        const hasOutcome = readByte();
        if (hasOutcome === 0) {
          revisions[i] = Object.freeze({ id, previous, ordinal, sequence, event });
        } else if (hasOutcome === 1) {
          const value = readF64();
          const because = readStrList();
          const hasMat = readByte();
          if (hasMat !== 0 && hasMat !== 1) throw new TypeError('malformed materialized flag');
          const materialized = hasMat === 1 ? readStrList() : null;
          const oSeq = readVarint();
          const oEvent = readStr();
          const lineage = readProvenance();
          const outcome = materialized === null
            ? { value, because, sequence: oSeq, event: oEvent, lineage }
            : { value, because, materialized, sequence: oSeq, event: oEvent, lineage };
          revisions[i] = Object.freeze({ id, previous, ordinal, sequence, event, outcome: Object.freeze(outcome) });
        } else {
          throw new TypeError('malformed revision outcome flag');
        }
      }
      Object.freeze(revisions);
    }
    const selection_qualifications = readProvenance();
    const flags = readByte();
    if ((flags & ~3) !== 0) throw new TypeError('malformed decision series flags');
    if (flags === 0) return Object.freeze({ limit, current, revisions, selection_qualifications });
    const obj = { limit, current, revisions, selection_qualifications };
    if (flags & 1) obj.on_hold = readStrList();
    if (flags & 2) obj.deferred = readStrList();
    return Object.freeze(obj);
  }

  function readJournalEntry() {
    const decision = readStr();
    const commitment = readStr();
    const change = readStr();
    const sequence = readVarint();
    const event = readStr();
    const flags = readByte();
    if ((flags & ~63) !== 0) throw new TypeError('malformed journal entry flags');
    const obj = { decision, commitment, change, sequence, event };
    if (flags & 1) obj.agent = readStr();
    if (flags & 2) obj.elapsed = readF64();
    if (flags & 4) obj.value = readF64();
    obj.because = readStrList();
    obj.caveats = readStrList();
    if (flags & 8) obj.permitted_by = readStr();
    if (flags & 16) obj.outcome = readF64();
    if (flags & 32) obj.materialized = readStrList();
    return Object.freeze(obj);
  }

  function readRelation() {
    const from = readStr();
    const rel = RELATION_NAMES[readByte()];
    if (rel === undefined) throw new TypeError('malformed relation tag');
    const to = readStr();
    return Object.freeze({ from, relation: rel, to, origin: 'live' });
  }

  function applyBinMap(heldMap, readValue) {
    const hasKeys = readByte();
    if (hasKeys === 0) {
      const setCount = readVarint();
      if (heldMap === undefined || heldMap === null) throw new TypeError('missing held map');
      if (!Object.hasOwn(heldMap, '__proto__')) {
        const result = { ...heldMap };
        for (let i = 0; i < setCount; i += 1) {
          const key = readStr();
          const val = readValue();
          if (!Object.hasOwn(heldMap, key)) throw new TypeError(`map key ${JSON.stringify(key)} is not held`);
          result[key] = val;
        }
        return Object.freeze(result);
      }
      const updates = new Map();
      for (let i = 0; i < setCount; i += 1) {
        const key = readStr();
        const val = readValue();
        if (!Object.hasOwn(heldMap, key)) throw new TypeError(`map key ${JSON.stringify(key)} is not held`);
        updates.set(key, val);
      }
      const result = {};
      for (const key of Object.keys(heldMap)) {
        put(result, key, updates.has(key) ? updates.get(key) : heldMap[key]);
      }
      return Object.freeze(result);
    }
    if (hasKeys !== 1) throw new TypeError('malformed map change flag');
    const keyCount = readVarint();
    if (keyCount > end - pos) throw new TypeError('truncated map keys in binary view patch');
    const keys = new Array(keyCount);
    for (let i = 0; i < keyCount; i += 1) keys[i] = readStr();
    const setCount = readVarint();
    if (setCount > keyCount) throw new TypeError('map set exceeds keys in binary view patch');
    const result = {};
    if (setCount === keyCount) {
      for (let i = 0; i < setCount; i += 1) {
        const key = readStr();
        if (key !== keys[i]) throw new TypeError('map set key mismatch in binary view patch');
        put(result, key, readValue());
      }
      return Object.freeze(result);
    }
    if (setCount === 0) {
      for (let i = 0; i < keyCount; i += 1) {
        const key = keys[i];
        if (heldMap === undefined || heldMap === null || !Object.hasOwn(heldMap, key)) {
          throw new TypeError(`map key ${JSON.stringify(key)} is neither held nor set`);
        }
        put(result, key, heldMap[key]);
      }
      return Object.freeze(result);
    }
    if (setCount === 1) {
      const k0 = readStr();
      const v0 = readValue();
      let used = false;
      for (let i = 0; i < keyCount; i += 1) {
        const key = keys[i];
        if (key === k0) {
          used = true;
          put(result, key, v0);
        } else {
          if (heldMap === undefined || heldMap === null || !Object.hasOwn(heldMap, key)) {
            throw new TypeError(`map key ${JSON.stringify(key)} is neither held nor set`);
          }
          put(result, key, heldMap[key]);
        }
      }
      if (!used) throw new TypeError(`map key ${JSON.stringify(k0)} is not in keys`);
      return Object.freeze(result);
    }
    const updates = new Map();
    for (let i = 0; i < setCount; i += 1) updates.set(readStr(), readValue());
    for (let i = 0; i < keyCount; i += 1) {
      const key = keys[i];
      if (updates.has(key)) {
        put(result, key, updates.get(key));
      } else {
        if (heldMap === undefined || heldMap === null || !Object.hasOwn(heldMap, key)) {
          throw new TypeError(`map key ${JSON.stringify(key)} is neither held nor set`);
        }
        put(result, key, heldMap[key]);
      }
    }
    return Object.freeze(result);
  }

  function applyBinNested(heldOuter, readValue) {
    const hasKeys = readByte();
    if (hasKeys === 0) {
      const setCount = readVarint();
      if (heldOuter === undefined || heldOuter === null) throw new TypeError('missing held nested map');
      if (!Object.hasOwn(heldOuter, '__proto__')) {
        const result = { ...heldOuter };
        for (let i = 0; i < setCount; i += 1) {
          const target = readStr();
          if (!Object.hasOwn(heldOuter, target)) throw new TypeError(`nested map key ${JSON.stringify(target)} is not held`);
          result[target] = applyBinMap(heldOuter[target], readValue);
        }
        return Object.freeze(result);
      }
      const updates = new Map();
      for (let i = 0; i < setCount; i += 1) {
        const target = readStr();
        if (!Object.hasOwn(heldOuter, target)) throw new TypeError(`nested map key ${JSON.stringify(target)} is not held`);
        updates.set(target, applyBinMap(own(heldOuter, target), readValue));
      }
      const result = {};
      for (const key of Object.keys(heldOuter)) {
        put(result, key, updates.has(key) ? updates.get(key) : heldOuter[key]);
      }
      return Object.freeze(result);
    }
    if (hasKeys !== 1) throw new TypeError('malformed nested map change flag');
    const keyCount = readVarint();
    if (keyCount > end - pos) throw new TypeError('truncated nested map keys in binary view patch');
    const keys = new Array(keyCount);
    for (let i = 0; i < keyCount; i += 1) keys[i] = readStr();
    const setCount = readVarint();
    if (setCount > keyCount) throw new TypeError('nested map set exceeds keys in binary view patch');
    const result = {};
    if (setCount === 1) {
      const t0 = readStr();
      const v0 = applyBinMap(own(heldOuter, t0), readValue);
      let used = false;
      for (let i = 0; i < keyCount; i += 1) {
        const key = keys[i];
        if (key === t0) {
          used = true;
          put(result, key, v0);
        } else {
          if (heldOuter === undefined || heldOuter === null || !Object.hasOwn(heldOuter, key)) {
            throw new TypeError(`nested map key ${JSON.stringify(key)} is neither held nor set`);
          }
          put(result, key, heldOuter[key]);
        }
      }
      if (!used) throw new TypeError(`nested map key ${JSON.stringify(t0)} is not in keys`);
      return Object.freeze(result);
    }
    const updates = new Map();
    for (let i = 0; i < setCount; i += 1) {
      const target = readStr();
      updates.set(target, applyBinMap(own(heldOuter, target), readValue));
    }
    for (let i = 0; i < keyCount; i += 1) {
      const key = keys[i];
      if (updates.has(key)) {
        put(result, key, updates.get(key));
      } else {
        if (heldOuter === undefined || heldOuter === null || !Object.hasOwn(heldOuter, key)) {
          throw new TypeError(`nested map key ${JSON.stringify(key)} is neither held nor set`);
        }
        put(result, key, heldOuter[key]);
      }
    }
    return Object.freeze(result);
  }

  function applyBinCommitments(heldList) {
    const hasKeys = readByte();
    if (hasKeys === 0) {
      const setCount = readVarint();
      if (!heldList) throw new TypeError('missing held commitments');
      const result = heldList.slice();
      if (setCount === 1) {
        const action = readStr();
        const val = readCommitment();
        let found = false;
        for (let i = 0; i < result.length; i += 1) {
          if (result[i].action === action) { result[i] = val; found = true; break; }
        }
        if (!found) throw new TypeError(`commitment ${JSON.stringify(action)} is not held`);
        return Object.freeze(result);
      }
      const updates = new Map();
      for (let i = 0; i < setCount; i += 1) updates.set(readStr(), readCommitment());
      let matched = 0;
      for (let i = 0; i < result.length; i += 1) {
        const updated = updates.get(result[i].action);
        if (updated !== undefined) { result[i] = updated; matched += 1; }
      }
      if (matched !== updates.size) throw new TypeError('commitment update key is not held');
      return Object.freeze(result);
    }
    if (hasKeys !== 1) throw new TypeError('malformed commitments change flag');
    const keyCount = readVarint();
    if (keyCount > end - pos) throw new TypeError('truncated commitment keys in binary view patch');
    const keys = new Array(keyCount);
    for (let i = 0; i < keyCount; i += 1) keys[i] = readStr();
    const setCount = readVarint();
    if (setCount > keyCount) throw new TypeError('commitments set exceeds keys in binary view patch');
    if (keyCount === 0) return EMPTY_ARRAY;
    let byAction = null;
    if (heldList && heldList.length > 0 && setCount < keyCount) {
      byAction = new Map();
      for (let i = 0; i < heldList.length; i += 1) byAction.set(heldList[i].action, heldList[i]);
    }
    const result = new Array(keyCount);
    if (setCount === keyCount) {
      for (let i = 0; i < keyCount; i += 1) {
        if (readStr() !== keys[i]) throw new TypeError('commitment key mismatch in binary view patch');
        result[i] = readCommitment();
      }
      return Object.freeze(result);
    }
    if (setCount === 1) {
      const a0 = readStr();
      const v0 = readCommitment();
      let used = false;
      for (let i = 0; i < keyCount; i += 1) {
        const k = keys[i];
        if (k === a0) {
          used = true;
          result[i] = v0;
        } else {
          if (!byAction || !byAction.has(k)) throw new TypeError(`commitment ${JSON.stringify(k)} is neither held nor set`);
          result[i] = byAction.get(k);
        }
      }
      if (!used) throw new TypeError(`commitment ${JSON.stringify(a0)} is not in keys`);
      return Object.freeze(result);
    }
    const updates = new Map();
    for (let i = 0; i < setCount; i += 1) updates.set(readStr(), readCommitment());
    for (let i = 0; i < keyCount; i += 1) {
      const k = keys[i];
      if (updates.has(k)) {
        result[i] = updates.get(k);
      } else {
        if (!byAction || !byAction.has(k)) throw new TypeError(`commitment ${JSON.stringify(k)} is neither held nor set`);
        result[i] = byAction.get(k);
      }
    }
    return Object.freeze(result);
  }

  function applyBinList(heldList, readItem) {
    const itemCount = readVarint();
    if (itemCount === 0) return EMPTY_ARRAY;
    if (itemCount > end - pos) throw new TypeError('truncated list in binary view patch');
    const tag0 = readByte();
    if (tag0 === 0) {
      const start = readVarint();
      const length = readVarint();
      if (!heldList || length < 1 || start + length > heldList.length) throw new TypeError('malformed list run');
      if (itemCount === 2 && pos < end && buf[pos] === 1) {
        pos += 1;
        const result = heldList.slice(start, start + length);
        result.push(readItem());
        return Object.freeze(result);
      }
      const result = heldList.slice(start, start + length);
      for (let i = 1; i < itemCount; i += 1) {
        const tag = readByte();
        if (tag === 0) {
          const s = readVarint();
          const l = readVarint();
          if (l < 1 || s + l > heldList.length) throw new TypeError('malformed list run');
          for (let j = s; j < s + l; j += 1) result.push(heldList[j]);
        } else if (tag === 1) {
          result.push(readItem());
        } else {
          throw new TypeError('malformed list item tag');
        }
      }
      return Object.freeze(result);
    }
    if (tag0 !== 1) throw new TypeError('malformed list item tag');
    const result = new Array(itemCount);
    result[0] = readItem();
    let outIdx = 1;
    for (let i = 1; i < itemCount; i += 1) {
      const tag = readByte();
      if (tag === 0) {
        const s = readVarint();
        const l = readVarint();
        if (!heldList || l < 1 || s + l > heldList.length) throw new TypeError('malformed list run');
        for (let j = s; j < s + l; j += 1) result[outIdx++] = heldList[j];
      } else if (tag === 1) {
        result[outIdx++] = readItem();
      } else {
        throw new TypeError('malformed list item tag');
      }
    }
    result.length = outIdx;
    return Object.freeze(result);
  }

  const outcomeTag = readByte();
  if (outcomeTag === 0) {
    const origin = ORIGINS[readByte()];
    const code = readStr();
    const message = readStr();
    if (!origin || pos !== end) throw new TypeError('malformed rejected binary patch');
    return { schema: DISPATCH_SCHEMA, outcome: 'rejected', origin, code, message };
  }
  if (outcomeTag !== 1) throw new TypeError('malformed binary patch outcome tag');

  const hasSince = readByte();
  if (hasSince !== 0 && hasSince !== 1) throw new TypeError('malformed binary patch since flag');
  const since = hasSince === 1 ? readVarint() : null;
  const base = since === null ? null : held;
  if (since !== null && (!held || held.sequence !== since)) throw new TypeError('view patch applies to a view not held');
  const sequence = readVarint();
  const last_event = readOptStr();
  if (pos + 2 > end) throw new TypeError('truncated binary view patch');
  const mask = buf[pos] | (buf[pos + 1] << 8);
  pos += 2;
  if ((mask & ~0x1ff) !== 0) throw new TypeError('unknown member bits in binary view patch');
  if (base === null && mask !== 0x1ff) throw new TypeError('full binary view patch leaves out members');

  const bindings = (mask & 1) ? applyBinNested(base?.bindings, readBindingValue) : base.bindings;
  const binding_explanations = (mask & 2) ? applyBinNested(base?.binding_explanations, readProvenance) : base.binding_explanations;
  let cues;
  if (mask & 4) {
    const cuesTag = readByte();
    if (cuesTag === 0) {
      cues = EMPTY_ARRAY;
    } else if (cuesTag === 1) {
      const parsed = JSON.parse(readStr());
      if (!Array.isArray(parsed)) throw new TypeError('malformed cues in binary view patch');
      cues = deepFreezeNew(parsed);
    } else {
      throw new TypeError('malformed cues tag in binary view patch');
    }
  } else {
    cues = base.cues;
  }
  let effects;
  if (mask & 8) {
    const n = readVarint();
    if (n === 0) {
      effects = EMPTY_ARRAY;
    } else {
      if (n > end - pos) throw new TypeError('truncated effects in binary view patch');
      effects = new Array(n);
      for (let i = 0; i < n; i += 1) effects[i] = readEffect();
      Object.freeze(effects);
    }
  } else {
    effects = base.effects;
  }
  const commitments = (mask & 16) ? applyBinCommitments(base?.commitments) : base.commitments;
  const commitment_grounds = (mask & 32) ? applyBinMap(base?.commitment_grounds, readProvenance) : base.commitment_grounds;
  const decision_series = (mask & 64) ? applyBinMap(base?.decision_series, readDecisionSeries) : base.decision_series;
  const decision_journal = (mask & 128) ? applyBinList(base?.decision_journal, readJournalEntry) : base.decision_journal;
  const relations = (mask & 256) ? applyBinList(base?.relations, readRelation) : base.relations;
  if (pos !== end) throw new TypeError('trailing bytes in binary view patch');

  const view = Object.freeze({
    schema: VIEW_SCHEMA_0_1,
    sequence,
    last_event,
    bindings,
    binding_explanations,
    cues,
    effects,
    commitments,
    commitment_grounds,
    decision_series,
    decision_journal,
    relations,
  });
  return { schema: DISPATCH_SCHEMA, outcome: 'accepted', view };
}

// An accepted dispatch outcome as the runtime writes it: these fields in this
// order, the snapshot last. The snapshot's text, or null for any other text,
// which is parsed and checked in full.
const ACCEPTED_PREFIX = `{"schema":"${DISPATCH_SCHEMA}","outcome":"accepted","snapshot":{`;
function acceptedSnapshotText(raw) {
  if (typeof raw !== 'string' || !raw.startsWith(ACCEPTED_PREFIX) || !raw.endsWith('}}')) return null;
  return raw.slice(ACCEPTED_PREFIX.length - 1, -1);
}

// Whether raw is a rejected outcome as the runtime writes it.
const REJECTED_PREFIX = `{"schema":"${DISPATCH_SCHEMA}","outcome":"rejected",`;
const rejectedOutcome = raw => typeof raw === 'string' && raw.startsWith(REJECTED_PREFIX);

// {schema, outcome: "accepted", snapshot}, the snapshot parsed when first
// read. Malformed text is fatal when it is read.
function lazyAccepted(text, malformed) {
  const outcome = { schema: DISPATCH_SCHEMA, outcome: 'accepted' };
  Object.defineProperty(outcome, 'snapshot', {
    enumerable: true,
    configurable: true,
    get() {
      let snapshot;
      try { snapshot = JSON.parse(text); } catch { throw malformed(); }
      if (!isPlainObject(snapshot)) throw malformed();
      Object.defineProperty(outcome, 'snapshot', { value: snapshot, enumerable: true, configurable: true, writable: true });
      return snapshot;
    },
    set(value) {
      Object.defineProperty(outcome, 'snapshot', { value, enumerable: true, configurable: true, writable: true });
    },
  });
  return outcome;
}

export class CaveatSession {
  #inner;
  #runtime;
  #state = 'open';
  // The snapshot text the last accepted dispatch() or snapshot() read, while
  // no call can have changed the session since.
  #lastSnapshot = null;
  // The read-only view dispatchView({ reuse: true }) last returned.
  #heldView = null;

  constructor(inner, runtime) {
    this.#inner = inner;
    this.#runtime = runtime;
  }

  get state() { return this.#state; }

  // A trap in any session of this runtime makes every session unusable.
  #usable() {
    if (this.#state === 'open' && this.#runtime.trapped) this.#state = 'fatal';
    if (this.#state === 'open') return;
    if (this.#state === 'closed') throw new CaveatError('closed', 'session is closed');
    throw new CaveatError('fatal', this.#runtime.trapped ? 'the runtime instance trapped; load it again' : 'session is unusable after a fatal outcome');
  }

  #fail(error) {
    this.#state = 'fatal';
    this.#heldView = null;
    if (isTrap(error)) {
      this.#runtime.markTrapped();
      return new CaveatError('fatal', this.#runtime.trapMessage(error));
    }
    return error instanceof CaveatError ? error : fatalFrom(error);
  }

  #call(read) {
    this.#usable();
    try { return read(); } catch (error) { throw this.#fail(error); }
  }

  // Runtime text that must be JSON. Malformed text is fatal like any other
  // runtime fault, and is checked inside the same guard.
  #json(read, what) {
    const text = this.#call(read);
    try {
      return { text, value: JSON.parse(text) };
    } catch {
      throw this.#fail(new CaveatError('fatal', `${what}() returned text that is not JSON`));
    }
  }

  // Returns {outcome: "accepted", snapshot} or {outcome: "rejected", origin,
  // code, message}. Throws CaveatError("fatal") for everything else. An
  // accepted snapshot is parsed when it is first read, and snapshot() reuses
  // it until the session changes rather than building it again.
  //
  // dispatch(event, payload, { agent }) says which agent sent the event, for
  // a program that declares agents (spec/caveat-agents-0.1.md).
  dispatch(event, payload = {}, options = {}) {
    this.#usable();
    const agent = agentOption(options);
    if (agent !== null && typeof this.#inner.dispatch_outcome_as !== 'function') throw predatesAgents('dispatch_outcome_as');
    if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
    const text = payloadText(payload);
    let raw;
    try {
      raw = agent === null ? this.#inner.dispatch_outcome(event, text) : this.#inner.dispatch_outcome_as(event, text, agent);
    } catch (error) { this.#lastSnapshot = null; throw this.#fail(error); }
    const accepted = acceptedSnapshotText(raw);
    if (accepted !== null) {
      this.#lastSnapshot = accepted;
      return lazyAccepted(accepted, () => this.#fail(new CaveatError('fatal', 'dispatch returned text that is not JSON')));
    }
    let outcome;
    // A rejected event leaves the session as it was, and its snapshot with it.
    if (!rejectedOutcome(raw)) this.#lastSnapshot = null;
    try { outcome = JSON.parse(raw); } catch { throw this.#fail(new CaveatError('fatal', 'dispatch returned text that is not JSON')); }
    if (!validOutcome(outcome)) {
      // Not a fatal report either: keep what arrived for diagnostics only.
      const error = new CaveatError('fatal', `unrecognised dispatch outcome: ${String(raw).slice(0, 200)}`);
      error.received = outcome;
      throw this.#fail(error);
    }
    return outcome;
  }

  // The same transaction, payload handling and failures as dispatch(), with the
  // view in place of the snapshot: {outcome: "accepted", view}, where view is
  // what view() returns after the event, or the rejected outcome dispatch()
  // returns. The runtime does not build the snapshot. A runtime build that
  // predates it throws CaveatError("load") and the session is untouched.
  //
  // dispatchView(event, payload, { fields }) gives a view with only the named
  // fields of VIEW_FIELDS, each exactly as view() has it, plus schema,
  // sequence and last_event. The runtime builds no other field. An unknown or
  // repeated name throws TypeError, and a runtime build without selection
  // throws CaveatError("load"); either leaves the session untouched.
  dispatchView(event, payload = {}, options = {}) {
    this.#usable();
    const fields = viewFields(options);
    const agent = agentOption(options);
    if (reuseOption(options)) {
      if (fields !== null) throw new TypeError('reuse and fields cannot be combined');
      return this.#dispatchReusedView(event, payload, agent);
    }
    if (agent !== null && fields === null && typeof this.#inner.dispatch_view_outcome_as !== 'function') throw predatesAgents('dispatch_view_outcome_as');
    if (agent !== null && fields !== null && typeof this.#inner.dispatch_view_fields_outcome_as !== 'function') throw predatesAgents('dispatch_view_fields_outcome_as');
    if (typeof this.#inner.dispatch_view_outcome !== 'function') {
      throw new CaveatError('load', 'this runtime build has no dispatch_view_outcome; it predates dispatchView');
    }
    if (fields !== null && typeof this.#inner.dispatch_view_fields_outcome !== 'function') {
      throw new CaveatError('load', 'this runtime build has no dispatch_view_fields_outcome; it predates dispatchView fields');
    }
    if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
    const text = payloadText(payload);
    this.#lastSnapshot = null;
    let raw;
    try {
      if (agent === null) {
        raw = fields === null
          ? this.#inner.dispatch_view_outcome(event, text)
          : this.#inner.dispatch_view_fields_outcome(event, text, fields);
      } else {
        raw = fields === null
          ? this.#inner.dispatch_view_outcome_as(event, text, agent)
          : this.#inner.dispatch_view_fields_outcome_as(event, text, fields, agent);
      }
    } catch (error) { throw this.#fail(error); }
    let outcome;
    try { outcome = JSON.parse(raw); } catch { throw this.#fail(new CaveatError('fatal', 'dispatchView returned text that is not JSON')); }
    if (!validOutcome(outcome, 'view')) {
      // Not a fatal report either: keep what arrived for diagnostics only.
      const error = new CaveatError('fatal', `unrecognised dispatch outcome: ${String(raw).slice(0, 200)}`);
      error.received = outcome;
      throw this.#fail(error);
    }
    return outcome;
  }

  // dispatchView(event, payload, { reuse: true }): the same view, read-only,
  // with every part the event left alone the same object as in the view the
  // last such call returned. See spec/caveat-view-patch-0.1.md.
  #dispatchReusedView(event, payload, agent) {
    const noBinary = agent === null
      ? typeof this.#inner.dispatch_view_patch_binary_outcome !== 'function'
      : typeof this.#inner.dispatch_view_patch_binary_outcome_as !== 'function';
    if (!noBinary) {
      if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
      const text = payloadText(payload);
      this.#lastSnapshot = null;
      const held = this.#heldView?.sequence;
      let bytes;
      try {
        bytes = agent === null
          ? this.#inner.dispatch_view_patch_binary_outcome(event, text, held)
          : this.#inner.dispatch_view_patch_binary_outcome_as(event, text, held, agent);
      } catch (error) { this.#heldView = null; throw this.#fail(error); }
      let decoded;
      try {
        decoded = decodeBinaryViewPatch(bytes, this.#heldView);
      } catch (error) {
        this.#heldView = null;
        throw this.#fail(new CaveatError('fatal', `dispatchView returned malformed binary view patch: ${messageOf(error)}`));
      }
      if (decoded.outcome !== 'accepted') return decoded;
      this.#heldView = decoded.view;
      return decoded;
    }
    if (typeof this.#inner.dispatch_view_patch_outcome !== 'function') {
      throw new CaveatError('load', 'this runtime build has no dispatch_view_patch_outcome; it predates dispatchView reuse');
    }
    if (agent !== null && typeof this.#inner.dispatch_view_patch_outcome_as !== 'function') throw predatesAgents('dispatch_view_patch_outcome_as');
    if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
    const text = payloadText(payload);
    this.#lastSnapshot = null;
    const held = this.#heldView?.sequence;
    let raw;
    try {
      raw = agent === null
        ? this.#inner.dispatch_view_patch_outcome(event, text, held)
        : this.#inner.dispatch_view_patch_outcome_as(event, text, held, agent);
    } catch (error) { this.#heldView = null; throw this.#fail(error); }
    let outcome;
    try { outcome = JSON.parse(raw); } catch { this.#heldView = null; throw this.#fail(new CaveatError('fatal', 'dispatchView returned text that is not JSON')); }
    if (!validOutcome(outcome, 'patch')) {
      this.#heldView = null;
      const error = new CaveatError('fatal', `unrecognised dispatch outcome: ${String(raw).slice(0, 200)}`);
      error.received = outcome;
      throw this.#fail(error);
    }
    if (outcome.outcome !== 'accepted') return outcome;
    let view;
    try { view = patchedView(this.#heldView, outcome.patch); } catch {
      // The event is applied either way: read the whole view instead, and
      // start the next patch from nothing.
      view = deepFreeze(this.#call(() => JSON.parse(this.#inner.view())));
    }
    this.#heldView = view;
    return { schema: outcome.schema, outcome: 'accepted', view };
  }

  // What dispatchView(event, payload, options) would return, run on a copy of
  // the session that is then dropped: this session, its save, its snapshot
  // and its view are unchanged, whatever the outcome. A fatal outcome throws
  // CaveatError("fatal") but leaves this session usable, since the copy
  // failed and not it; a trap still makes the whole runtime unusable. A
  // runtime build that predates it throws CaveatError("load"). See
  // spec/caveat-dispatch-0.1.md, "The what-if path".
  whatIf(event, payload = {}, options = {}) {
    this.#usable();
    const fields = viewFields(options);
    // whatIf(event, payload, { agent }) asks what the event would do sent by
    // that agent (spec/caveat-agents-0.1.md), as dispatchView would answer it.
    const agent = agentOption(options);
    if (typeof this.#inner.what_if_view_outcome !== 'function') {
      throw new CaveatError('load', 'this runtime build has no what_if_view_outcome; it predates whatIf');
    }
    if (fields !== null && typeof this.#inner.what_if_view_fields_outcome !== 'function') {
      throw new CaveatError('load', 'this runtime build has no what_if_view_fields_outcome; it predates whatIf fields');
    }
    if (agent !== null && fields === null && typeof this.#inner.what_if_view_outcome_as !== 'function') throw predatesAgents('what_if_view_outcome_as');
    if (agent !== null && fields !== null && typeof this.#inner.what_if_view_fields_outcome_as !== 'function') throw predatesAgents('what_if_view_fields_outcome_as');
    if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
    const text = payloadText(payload);
    let raw;
    try {
      if (agent === null) {
        raw = fields === null
          ? this.#inner.what_if_view_outcome(event, text)
          : this.#inner.what_if_view_fields_outcome(event, text, fields);
      } else {
        raw = fields === null
          ? this.#inner.what_if_view_outcome_as(event, text, agent)
          : this.#inner.what_if_view_fields_outcome_as(event, text, fields, agent);
      }
    } catch (error) {
      if (isTrap(error)) throw this.#fail(error);
      throw error instanceof CaveatError ? error : fatalFrom(error);
    }
    let outcome;
    try { outcome = JSON.parse(raw); } catch { throw new CaveatError('fatal', 'whatIf returned text that is not JSON'); }
    if (!validOutcome(outcome, 'view')) {
      const error = new CaveatError('fatal', `unrecognised dispatch outcome: ${String(raw).slice(0, 200)}`);
      error.received = outcome;
      throw error;
    }
    return outcome;
  }

  snapshotText() { return this.#json(() => this.#inner.snapshot(), 'snapshot').text; }
  // A fresh object each call, from the snapshot text kept until the session
  // changes. Compact text, where the runtime has it, parses faster than the
  // pretty text snapshotText() returns.
  snapshot() {
    this.#usable();
    if (this.#lastSnapshot === null) {
      const pretty = typeof this.#inner.snapshot_compact !== 'function';
      this.#lastSnapshot = this.#call(() => (pretty ? this.#inner.snapshot() : this.#inner.snapshot_compact()));
    }
    try {
      return JSON.parse(this.#lastSnapshot);
    } catch {
      this.#lastSnapshot = null;
      throw this.#fail(new CaveatError('fatal', 'snapshot() returned text that is not JSON'));
    }
  }
  // view() is View 0.1; view({ schema: '0.2' }) is View 0.2
  // (spec/caveat-view-0.2.md). A runtime build that predates View 0.2 throws
  // CaveatError("load") for 0.2.
  viewText(options) { return this.#json(this.#viewReader(options), 'view').text; }
  view(options) { return this.#json(this.#viewReader(options), 'view').value; }

  // Checked before the runtime is called, so a wrong schema or an older
  // runtime leaves the session usable.
  #viewReader(options = {}) {
    this.#usable();
    const schema = options?.schema ?? '0.1';
    if (schema === '0.1') return () => this.#inner.view();
    if (schema !== '0.2') throw new TypeError("view schema must be '0.1' or '0.2'");
    if (typeof this.#inner.view_v2 !== 'function') {
      throw new CaveatError('load', 'this runtime build has no view_v2; it predates View 0.2');
    }
    return () => this.#inner.view_v2();
  }

  // The same transaction, payload handling and failures as dispatch(), with
  // the View 0.2 delta in place of the snapshot: {outcome: "accepted", delta},
  // which applied to view({ schema: '0.2' }) before the event gives the view
  // after it, or the rejected outcome dispatch() returns, with no delta. A
  // runtime build that predates it throws CaveatError("load") and the session
  // is untouched.
  dispatchViewDelta(event, payload = {}, options = {}) {
    this.#usable();
    const agent = agentOption(options);
    if (typeof this.#inner.dispatch_view_delta_outcome !== 'function') {
      throw new CaveatError('load', 'this runtime build has no dispatch_view_delta_outcome; it predates dispatchViewDelta');
    }
    if (agent !== null && typeof this.#inner.dispatch_view_delta_outcome_as !== 'function') throw predatesAgents('dispatch_view_delta_outcome_as');
    if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
    const text = payloadText(payload);
    this.#lastSnapshot = null;
    let raw;
    try {
      raw = agent === null
        ? this.#inner.dispatch_view_delta_outcome(event, text)
        : this.#inner.dispatch_view_delta_outcome_as(event, text, agent);
    } catch (error) { throw this.#fail(error); }
    let outcome;
    try { outcome = JSON.parse(raw); } catch { throw this.#fail(new CaveatError('fatal', 'dispatchViewDelta returned text that is not JSON')); }
    if (!validOutcome(outcome, 'delta')) {
      // Not a fatal report either: keep what arrived for diagnostics only.
      const error = new CaveatError('fatal', `unrecognised dispatch outcome: ${String(raw).slice(0, 200)}`);
      error.received = outcome;
      throw this.#fail(error);
    }
    return outcome;
  }

  // The save is JSON text; restore it with the exact source it came from.
  // save({ compact: true }) is the replay save while the session's accepted
  // events fit in one and it is smaller, otherwise the same state save
  // (spec/caveat-replay-save-0.1.md). With positional: true as well it is the
  // positional replay save, which holds more events
  // (spec/caveat-replay-save-0.2.md). A runtime build that predates either
  // throws CaveatError("load") and the session is untouched.
  save(options = {}) {
    if (options?.compact !== true) return this.#json(() => this.#inner.save(), 'save').text;
    this.#usable();
    if (options.positional === true) {
      if (typeof this.#inner.save_compact_positional !== 'function') {
        throw new CaveatError('load', 'this runtime build has no save_compact_positional; it predates positional replay saves');
      }
      return this.#json(() => this.#inner.save_compact_positional(), 'save').text;
    }
    if (typeof this.#inner.save_compact !== 'function') {
      throw new CaveatError('load', 'this runtime build has no save_compact; it predates compact saves');
    }
    return this.#json(() => this.#inner.save_compact(), 'save').text;
  }

  // The archive items produced since the last drain, and an empty
  // archive (spec/caveat-departure-0.1.md). The archive is not saved: a host
  // that keeps exact history stores departed records and provenance nodes
  // together beside its saves. A runtime that predates departure departs
  // nothing, so its archive is empty.
  drainArchive() {
    this.#lastSnapshot = null;
    if (typeof this.#inner.drain_archive !== 'function') {
      this.#usable();
      return [];
    }
    const { value } = this.#json(() => this.#inner.drain_archive(), 'drainArchive');
    if (!Array.isArray(value)) throw this.#fail(new CaveatError('fatal', 'drainArchive() returned JSON that is not a list'));
    return value;
  }

  // How many archive items (records and provenance nodes) await draining.
  get undrained() {
    if (typeof this.#inner.undrained !== 'function') {
      this.#usable();
      return 0;
    }
    return this.#call(() => this.#inner.undrained());
  }

  // After a fatal outcome, or once the runtime has trapped, the session is
  // abandoned without calling into the runtime, including free(). A trap
  // during free() marks the runtime trapped for every other session.
  close() {
    this.#lastSnapshot = null;
    this.#heldView = null;
    if (this.#state === 'open' && !this.#runtime.trapped) {
      this.#state = 'closed';
      this.#runtime.released(this);
      try { this.#inner.free(); } catch (error) { if (isTrap(error)) this.#runtime.markTrapped(); }
    }
    this.#state = 'closed';
  }
}

// Trap state belongs to a WebAssembly instance, not to a wrapper. Every
// runtime made from the same session class shares one instance and so one
// lifecycle: a trap seen through any of them stops all of them.
const lifecycles = new WeakMap();
function lifecycleOf(SessionClass) {
  let lifecycle = lifecycles.get(SessionClass);
  if (!lifecycle) {
    // open counts the sessions made through this instance and not yet closed
    // or collected: what a trap for lack of memory reports.
    lifecycle = { trapped: false, open: 0 };
    lifecycle.collected = typeof FinalizationRegistry === 'function'
      ? new FinalizationRegistry(() => { lifecycle.open -= 1; })
      : null;
    lifecycles.set(SessionClass, lifecycle);
  }
  return lifecycle;
}

// SessionClass is the runtime's WebReactiveSession, or a stand-in with the same
// methods: new SessionClass(source), SessionClass.restore(source, saved),
// dispatch_outcome, snapshot, view, save and free, and optionally
// dispatch_view_outcome (for dispatchView), view_v2 and
// dispatch_view_delta_outcome (for View 0.2), drain_archive and undrained (for
// drainArchive and undrained), save_compact (for save({ compact: true })),
// save_compact_positional (for save({ compact: true, positional: true })),
// snapshot_compact (a faster snapshot()), SessionClass.check(source),
// optionally SessionClass.check_with(source, unused) and
// SessionClass.check_options(source, optionsJson) (for check's options), and
// SessionClass.interface(source) and SessionClass.restore_options(source,
// saved, optionsJson) (for an explicit historical module interpretation).
export function createRuntime(SessionClass, identity = {}) {
  const lifecycle = lifecycleOf(SessionClass);
  const refuse = () => new CaveatError('fatal', 'this runtime instance trapped; load a fresh one');
  // A session made through this runtime, counted until it is closed or
  // collected.
  const opened = inner => {
    const session = new CaveatSession(inner, runtime);
    lifecycle.open += 1;
    lifecycle.collected?.register(session, undefined, session);
    return session;
  };
  // The error for a call that trapped, kind as given unless it trapped. A
  // call that restores or checks a save passes its text, and which it was
  // doing, for trapMessage.
  const failure = (error, kind, save, during) => {
    if (!isTrap(error)) return new CaveatError(kind, messageOf(error));
    lifecycle.trapped = true;
    return new CaveatError('fatal', runtime.trapMessage(error, save, during));
  };
  const runtime = {
    identity: Object.freeze({ ...identity }),
    get trapped() { return lifecycle.trapped; },
    markTrapped() { lifecycle.trapped = true; },
    // How many sessions made through this runtime instance are neither
    // closed nor collected.
    get openSessions() { return lifecycle.open; },
    released(session) {
      if (lifecycle.collected?.unregister(session) ?? true) lifecycle.open -= 1;
    },
    // What a trap means. A runtime that can say it ran out of memory
    // (out_of_memory) is named as the cause, with the sessions still open:
    // a host that never closes its sessions exhausts memory this way, and
    // the bare trap says only "unreachable" (Glowcap round 8, item 7). A
    // call that was restoring or checking a save passes the save's text and
    // `during`, 'restoring' (the default) or 'checking': restore takes a save
    // of any size (D070), and one too large for memory runs out however few
    // sessions are open, so the message names the restore or check and the
    // save's length. Open sessions hold memory as well, so it counts them and,
    // when there are any, still says to close them.
    trapMessage(error, save, during = 'restoring') {
      if (typeof SessionClass.out_of_memory !== 'function') return messageOf(error);
      let outOfMemory = false;
      try { outOfMemory = SessionClass.out_of_memory() === true; } catch { /* the trap stands as reported */ }
      if (!outOfMemory) return messageOf(error);
      if (typeof save === 'string') {
        const checking = during === 'checking';
        return `the runtime ran out of WebAssembly memory while ${checking ? 'checking' : 'restoring'} a save of ${save.length} characters, `
          + `with ${lifecycle.open} other session(s) open (trap: ${messageOf(error)}). `
          + `${checking ? 'Checking' : 'Restoring'} a save needs memory that grows with its length. `
          + (lifecycle.open > 0 ? 'Close each session with close() once it is no longer needed. ' : '')
          + 'This runtime instance is now unusable; load it again.';
      }
      return `the runtime ran out of WebAssembly memory with ${lifecycle.open} session(s) open `
        + `(trap: ${messageOf(error)}). Close each session with close() once it is no longer needed. `
        + 'This runtime instance is now unusable; load it again.';
    },
    open(source) {
      if (lifecycle.trapped) throw refuse();
      if (typeof source !== 'string') throw new TypeError('source must be text');
      try { return opened(new SessionClass(source)); } catch (error) {
        throw failure(error, 'load');
      }
    },
    // Advisory warnings for a program that loads (spec/caveat-check-0.1.md).
    // A program that does not load throws as open does. `{ unused: true }`
    // adds C007 and `{ records: true }` C008, each reported only when asked
    // for. `{ records: true, save }` also adds C009 for that save (its text);
    // a save that does not restore throws as restore does. `{ withdrawals:
    // true }` adds C010 and C011, and `{ names: true }` C012.
    check(source, options = {}) {
      if (lifecycle.trapped) throw refuse();
      if (typeof source !== 'string') throw new TypeError('source must be text');
      if (typeof SessionClass.check !== 'function') {
        throw new CaveatError('load', 'this runtime build has no check; it predates caveat check');
      }
      const unused = options?.unused === true;
      const records = options?.records === true;
      const withdrawals = options?.withdrawals === true;
      const names = options?.names === true;
      const save = options?.save;
      if (save !== undefined && typeof save !== 'string') throw new TypeError('save must be text');
      if (save !== undefined && !records) throw new TypeError('a save is checked with records: true');
      if (unused && typeof SessionClass.check_with !== 'function') {
        throw new CaveatError('load', 'this runtime build has no unused-declaration check; it predates caveat check --unused');
      }
      if (records && typeof SessionClass.check_options !== 'function') {
        throw new CaveatError('load', 'this runtime build has no records check; it predates caveat check --records');
      }
      if (withdrawals && typeof SessionClass.check_options !== 'function') {
        throw new CaveatError('load', 'this runtime build has no withdrawals check; it predates caveat check --withdrawals');
      }
      if (names && typeof SessionClass.check_options !== 'function') {
        throw new CaveatError('load', 'this runtime build has no near-miss-commitment check; it predates caveat check --names');
      }
      // Check reads no bundle, and the runtime refuses one before it reads a
      // save, so a bundle's save is not restored first: a save of a bundle
      // that needs its module binding reading named would ask for a choice
      // check does not take (D097).
      if (save !== undefined && !source.startsWith(BUNDLE_MARKER)) {
        // The program must load first, so its error is reported as load.
        try { new SessionClass(source).free(); } catch (error) { throw failure(error, 'load'); }
        try { SessionClass.restore(source, save).free(); } catch (error) { throw failure(error, 'restore', save, 'checking'); }
      }
      try {
        if (withdrawals || names) return JSON.parse(SessionClass.check_options(source, JSON.stringify({ unused, records, ...(withdrawals ? { withdrawals } : {}), ...(names ? { names } : {}), save })));
        if (records) return JSON.parse(SessionClass.check_options(source, JSON.stringify({ unused, records, save })));
        return JSON.parse(unused ? SessionClass.check_with(source, true) : SessionClass.check(source));
      } catch (error) {
        // check_options reads the save with its options and restores it
        // again, when there is one; it can run out of memory reading them.
        throw failure(error, 'load', save, 'checking');
      }
    },
    // The program's events, states, bindings and declared names
    // (spec/caveat-interface-0.1.md). A program that does not load throws as
    // open does.
    interface(source) {
      if (lifecycle.trapped) throw refuse();
      if (typeof source !== 'string') throw new TypeError('source must be text');
      if (typeof SessionClass.interface !== 'function') {
        throw new CaveatError('load', 'this runtime build has no interface; it predates caveat types');
      }
      try { return JSON.parse(SessionClass.interface(source)); } catch (error) {
        throw failure(error, 'load');
      }
    },
    restore(source, saved, options) {
      if (lifecycle.trapped) throw refuse();
      if (typeof source !== 'string' || typeof saved !== 'string') throw new TypeError('source and save must be text');
      const optionsText = restoreOptionsText(options);
      if (optionsText !== null && typeof SessionClass.restore_options !== 'function') {
        throw new CaveatError('load', 'this runtime build has no restore_options; it predates module binding choices');
      }
      try {
        return opened(optionsText === null
          ? SessionClass.restore(source, saved)
          : SessionClass.restore_options(source, saved, optionsText));
      } catch (error) {
        throw failure(error, 'restore', saved);
      }
    },
  };
  return runtime;
}

let freshInstances = 0;

// module: a URL for caveat_runtime.js (relative URLs resolve against this file,
// as import() does), or its imported namespace. wasm: whatever the runtime's
// init accepts (bytes, a URL or a Response).
//
// Loading the same URL twice gives wrappers over the same instance. If that
// instance has trapped, a URL is imported again under a unique query, which
// gives a separate module with its own memory, so recovery never reuses the
// trapped instance. A namespace cannot be re-imported, so it is refused.
//
// A URL is only known at run time, so a bundler cannot follow it. The
// comments tell webpack and Vite to leave both imports to the platform's own
// import() instead of warning about an expression they cannot resolve; a
// bundled host passes the namespace it imported itself.
export async function loadRuntime({ module, wasm, identity }) {
  const url = typeof module === 'string' || module instanceof URL ? new URL(String(module), import.meta.url) : null;
  let namespace = url ? await import(/* webpackIgnore: true */ /* @vite-ignore */ url.href) : module;
  if (lifecycles.get(namespace.WebReactiveSession)?.trapped) {
    if (!url) throw new CaveatError('fatal', 'this runtime instance trapped; pass its module URL to load a fresh one');
    const fresh = new URL(url.href);
    freshInstances += 1;
    fresh.searchParams.set('caveat-instance', String(freshInstances));
    namespace = await import(/* webpackIgnore: true */ /* @vite-ignore */ fresh.href);
  }
  await namespace.default({ module_or_path: wasm });
  if (typeof namespace.WebReactiveSession?.prototype?.dispatch_outcome !== 'function') {
    throw new CaveatError('load', 'this runtime build has no dispatch_outcome; it predates the dispatch outcome contract');
  }
  return createRuntime(namespace.WebReactiveSession, identity);
}

// One journal change, copied so the caller cannot alter the view.
function journalChange(entry) {
  const change = { sequence: entry.sequence, event: entry.event, because: [...entry.because], caveats: [...entry.caveats] };
  if (entry.value !== undefined) change.value = entry.value;
  if (entry.permitted_by !== undefined) change.permitted_by = entry.permitted_by;
  return change;
}

// Every revision of one decision series, oldest first, read from a view or
// snapshot: what each was committed on and, once reopened, what reopened it.
// The current revision is in force or reopened, as its commitment says; every
// earlier one is superseded. The series lists every revision; a windowed
// journal may have retired a revision's entries, and that revision is then
// listed with `retired: true` and null for each change the journal no longer
// holds, rather than left out.
export function decisionHistory(view, decision) {
  const series = view.decision_series?.[decision];
  if (!series) throw new TypeError(`${decision} is not a decision series of this program`);
  const revisions = new Map(series.revisions.map(item => [item.id, { revision: item.id, status: 'superseded', retired: true, committed: null, reopened: null }]));
  for (const entry of view.decision_journal) {
    const revision = entry.decision === decision ? revisions.get(entry.commitment) : undefined;
    if (!revision) continue;
    if (entry.change === 'committed') {
      revision.committed = journalChange(entry);
      revision.retired = false;
    } else if (entry.change === 'reopened') {
      // A hold's `held`, `deferred` and `released` entries reopen nothing.
      revision.reopened = journalChange(entry);
    }
  }
  const current = series.current === null ? undefined : revisions.get(series.current);
  if (current) {
    const commitment = view.commitments.find(item => item.action === series.current);
    // View 0.1 and the snapshot say `open`; View 0.2 says `reopened`.
    const reopened = commitment ? commitment.open ?? commitment.reopened : true;
    current.status = reopened === false ? 'in_force' : 'reopened';
  }
  return [...revisions.values()];
}
