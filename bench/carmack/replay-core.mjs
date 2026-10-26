// Carmack check 2, the part every platform runs: seeded random event
// sequences through the WebAssembly runtime, each with a save and a restore
// at a random point, digested with SHA-256. No Node imports, so Node on every
// operating system and every browser run exactly this code; the digests must
// then be byte-identical everywhere.
//
// Each sequence also checks itself where it runs: after the save, the
// original session, the session restored from its state save and the one
// restored from its compact save must give the same text for every later
// event. A difference there is a failure on that platform alone.

// xorshift64*, as runtime/examples/replay_save_sweep.rs uses, in BigInt so
// every engine computes the same numbers.
function generator(seed) {
  let state = BigInt.asUintN(64, BigInt(seed) * 0x9E3779B97F4A7C15n + 1n);
  const next = () => {
    state ^= state >> 12n;
    state = BigInt.asUintN(64, state ^ (state << 25n));
    state ^= state >> 27n;
    return BigInt.asUintN(64, state * 0x2545F4914F6CDD1Dn);
  };
  return { below: (bound) => Number(next() % BigInt(bound)) };
}

// A payload value for one interface parameter: a member's name, an
// identifier, or a number at a bound, a whole number, or a fraction. Some
// values are out of range on purpose, so refusals are compared too.
function value(random, parameter) {
  if (Array.isArray(parameter.members) && parameter.members.length) {
    return parameter.members[random.below(parameter.members.length)];
  }
  if (parameter.type === 'identifier') return `id${random.below(6)}`;
  const min = Number.isFinite(parameter.min) ? parameter.min : 0;
  const max = Number.isFinite(parameter.max) ? parameter.max : 100;
  const pick = random.below(32);
  if (pick === 0) return min;
  if (pick === 1) return max;
  if (pick === 2) return max + 1;
  if (pick < 16) return Math.round(min + ((max - min) * random.below(1001)) / 1000);
  return min + ((max - min) * random.below(1_000_001)) / 1_000_000;
}

const hex = (buffer) => [...new Uint8Array(buffer)].map((b) => b.toString(16).padStart(2, '0')).join('');

// One digest over many texts, each length-prefixed so the parts cannot run together.
async function digest(texts) {
  const encoder = new TextEncoder();
  const parts = texts.map((t) => encoder.encode(`${t.length}:${t}\n`));
  const joined = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) { joined.set(p, at); at += p.length; }
  return hex(await globalThis.crypto.subtle.digest('SHA-256', joined));
}

function outcomeText(session, event, payload) {
  try {
    return JSON.stringify(session.dispatchView(event, payload));
  } catch (error) {
    return `error ${error?.kind ?? 'unknown'}: ${error?.message ?? error}`;
  }
}

// programs: [{ name, source }]. Returns { digests: {program: {sequence: hex}},
// failures: [...], combined: hex }.
export async function replayAll(runtime, programs, { sequences = 8, events = 60, seed = 20261025 } = {}) {
  const digests = {};
  const failures = [];
  const all = [];
  for (const [p, program] of programs.entries()) {
    digests[program.name] = {};
    let shape;
    try { shape = runtime.interface(program.source); } catch (error) {
      digests[program.name].skipped = `does not load: ${error?.message ?? error}`;
      all.push(digests[program.name].skipped);
      continue;
    }
    const kinds = shape.events.filter((e) => !e.name.startsWith('__'));
    if (!kinds.length) { digests[program.name].skipped = 'no events'; all.push('no events'); continue; }
    for (let s = 0; s < sequences; s += 1) {
      const random = generator(seed + 1000 * p + s);
      const texts = [];
      const original = runtime.open(program.source);
      const cut = 1 + random.below(events - 1);
      let restored = [];
      for (let i = 0; i < events; i += 1) {
        const kind = kinds[random.below(kinds.length)];
        const payload = Object.fromEntries((kind.parameters ?? []).map((q) => [q.name, value(random, q)]));
        const text = outcomeText(original, kind.name, payload);
        texts.push(text);
        for (const [label, session] of restored) {
          const again = outcomeText(session, kind.name, payload);
          if (again !== text) failures.push({ program: program.name, sequence: s, step: i, restoredFrom: label, original: text.slice(0, 300), restored: again.slice(0, 300) });
        }
        if (i === cut) {
          const state = original.save();
          const compact = original.save({ compact: true, positional: true });
          texts.push(state, compact);
          restored = [['state', runtime.restore(program.source, state)], ['compact', runtime.restore(program.source, compact)]];
          for (const [label, session] of restored) {
            if (session.save() !== state) failures.push({ program: program.name, sequence: s, step: i, restoredFrom: label, problem: 'the restored session saves different bytes' });
          }
        }
      }
      texts.push(original.save());
      original.close();
      for (const [, session] of restored) session.close();
      const hash = await digest(texts);
      digests[program.name][s] = hash;
      all.push(hash);
    }
  }
  return { digests, failures, combined: await digest(all) };
}
