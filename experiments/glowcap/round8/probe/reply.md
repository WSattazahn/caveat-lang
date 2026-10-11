# Probe reply: Gemini (counted)

- **Received:** the owner posted it in this round's thread at 2026-10-11 03:03:21 UTC (message `cmsg_015odJFpwogpoW5yAoJxQmJwEd7QNK5EJgFHSiFkEaDyKg`), headed "gemini". That was after the thread asked for a brand-new Gemini chat or a ChatGPT Temporary Chat, with no memory, no custom instructions, and web and tools off, and with the prompt pasted unchanged.
- **Model:** Gemini, as named by the owner; the exact model and revision were not given. Gemini is a Google model, a family other than the authors'.
- **Prompt:** `probe/prompt.txt`, registered at stage 1 (SHA256 `55e771f8…`).
- **Status: counted.** It is the first run to meet "The learnability probe": a family other than the authors', and nothing in the reply suggests it saw Caveat's syntax (its constructs and host API are guesses, and it says so). The chat settings are as the owner reported them; this record cannot check them.
- **After rc.17.** The reply arrived after rc.17 was published, so any guide or packet item it yields goes to a later candidate, and round 8 runs on rc.17's docs as they are ("Timing"; amendment 6). It arrived before any author started.

The reply follows exactly as posted. The owner's label has been moved into this header. The project chat delivered `<`, `>`, `&`, `"` and `'` as HTML entities, and they are restored here.

---

1. The Caveat Program
Code snippet

```
caveat tasted_in_dark;
caveat taste_faded;
caveat secondhand;

evidence absorb_ev { id: string, kind: string }
evidence taste_ev { id: string, kind: string }
evidence witness_ev { id: string, kind: string }

state glow_timer: number = 0;
state heavy_timer: number = 0;

struct Mushroom {
    id: string;
    life: number;
    consumed_timer: number; // -1 if present
    consumed_by: any;
    tasted_kind: string;
    taste_timer: number;    // -1 if untasted
    taste_ev: any;
}

state mushrooms: list<Mushroom> = [
    Mushroom{id: "cave", life: 1, consumed_timer: -1, consumed_by: null, tasted_kind: "", taste_timer: -1, taste_ev: null},
    Mushroom{id: "pool", life: 1, consumed_timer: -1, consumed_by: null, tasted_kind: "", taste_timer: -1, taste_ev: null},
    Mushroom{id: "ruin", life: 1, consumed_timer: -1, consumed_by: null, tasted_kind: "", taste_timer: -1, taste_ev: null},
    Mushroom{id: "grove", life: 1, consumed_timer: -1, consumed_by: null, tasted_kind: "", taste_timer: -1, taste_ev: null}
];

archive Support;
archive Contradict;

decision Trust {
    state: string = "none";
    history: list<any> = [];
    glowcaps_since_reopen: list<any> = [];
}

fn ev_name(base: string, id: string, life: number) -> string {
    if (life == 1) { return base + "_" + id; }
    return base + "_" + id + "_" + life;
}

fn union_caveats(ev_list: list<any>) -> list<string> {
    let cavs = set();
    for (let ev in ev_list) {
        for (let c in ev.caveats()) cavs.add(c);
    }
    return cavs.to_list();
}

fn handle_observation(kind: string, ev: any) {
    if (kind == "glowcap") { Support.append(ev); } 
    else if (kind == "duskcap") { Contradict.append(ev); }

    let b_state = get_belief().state;

    if (Trust.state == "none" && b_state == "probably_safe") {
        Trust.commit([ev]);
        Trust.history.push({ change: "committed", because: Trust.basis });
    } else if (Trust.state == "committed" && kind == "duskcap") {
        Trust.reopen([ev]);
        Trust.glowcaps_since_reopen = [];
        Trust.history.push({ change: "reopened", because: [ev] });
    } else if (Trust.state == "reopened" && kind == "glowcap") {
        Trust.glowcaps_since_reopen.push(ev);
        if (Trust.glowcaps_since_reopen.length == 2) {
            Trust.commit(Trust.glowcaps_since_reopen);
            Trust.history.push({ change: "committed", because: Trust.basis });
        }
    }
}

event tick(payload: any) {
    let dt = payload.dt;
    require dt >= 0 && dt <= 0.1;

    if (glow_timer > 0) { glow_timer = max(0, glow_timer - dt); }
    if (heavy_timer > 0) { heavy_timer = max(0, heavy_timer - dt); }

    for (let m in mushrooms) {
        if (m.taste_timer >= 0) {
            m.taste_timer += dt;
            if (m.taste_timer >= 60 && !m.taste_ev.has_caveat(taste_faded)) {
                m.taste_ev.add_caveat(taste_faded);
            }
        }
        if (m.consumed_timer >= 0) {
            m.consumed_timer += dt;
            if (m.consumed_timer >= 45 && m.life < 64) {
                m.life += 1;
                m.consumed_timer = -1;
                m.consumed_by = null;
                m.tasted_kind = "";
                m.taste_timer = -1;
                m.taste_ev = null;
            }
        }
    }
}

event absorb(payload: any) {
    let m = null;
    for (let idx in mushrooms) { if (mushrooms[idx].id == payload.id) m = mushrooms[idx]; }
    require m != null;
    require payload.kind == "glowcap" || payload.kind == "duskcap";
    require m.consumed_timer == -1;
    require m.tasted_kind == "" || m.tasted_kind == payload.kind;
    
    let view_info = get_mushroom_view(m);
    require view_info.canAbsorb == true;

    let ev = observe absorb_ev(ev_name("absorb", m.id, m.life), { id: m.id, kind: payload.kind });
    m.consumed_timer = 0;
    m.consumed_by = ev;

    if (payload.kind == "glowcap") {
        glow_timer = 30;
    } else {
        if (heavy_timer > 0) { heavy_timer = min(30, heavy_timer + 20); } 
        else { heavy_timer = 20; }
    }
    handle_observation(payload.kind, ev);
}

event taste(payload: any) {
    let m = null;
    for (let idx in mushrooms) { if (mushrooms[idx].id == payload.id) m = mushrooms[idx]; }
    require m != null;
    require payload.kind == "glowcap" || payload.kind == "duskcap";
    require m.consumed_timer == -1;
    require m.tasted_kind == "";
    
    let view_info = get_mushroom_view(m);
    require view_info.canTaste == true;

    let ev = observe taste_ev(ev_name("taste", m.id, m.life), { id: m.id, kind: payload.kind });
    if (glow_timer <= 0) { ev.add_caveat(tasted_in_dark); }
    
    m.tasted_kind = payload.kind;
    m.taste_timer = 0;
    m.taste_ev = ev;

    handle_observation(payload.kind, ev);
}

event witness(payload: any) {
    let m = null;
    for (let idx in mushrooms) { if (mushrooms[idx].id == payload.id) m = mushrooms[idx]; }
    require m != null;
    require payload.kind == "glowcap" || payload.kind == "duskcap";
    require m.consumed_timer == -1;
    require m.tasted_kind == "" || m.tasted_kind == payload.kind;

    let ev = observe witness_ev(ev_name("witness", m.id, m.life), { id: m.id, kind: payload.kind });
    ev.add_caveat(secondhand);

    m.consumed_timer = 0;
    m.consumed_by = ev;

    handle_observation(payload.kind, ev);
}

fn get_belief() -> any {
    let supp = Support.all();
    let con = Contradict.all();
    let s_count = supp.length;
    let c_count = con.length;
    
    let state = "none";
    let text = "";
    let note = "";
    
    if (s_count > 0 && c_count == 0) {
        state = "probably_safe";
        text = "Glowing mushrooms give you light.";
        note = s_count == 1 ? "Based on one observation." : "Based on " + s_count + " observations.";
    } else if (c_count > 0 && s_count == 0) {
        state = "probably_unsafe";
        text = "Glowing mushrooms make you heavy.";
        note = c_count == 1 ? "Based on one observation." : "Based on " + c_count + " observations.";
    } else if (s_count > 0 && c_count > 0) {
        state = "uncertain";
        text = "Not every glowing mushroom is safe. Taste before absorbing.";
    }
    
    return {
        state: state,
        text: text,
        note: note,
        supportedBy: supp,
        contradictedBy: con,
        caveats: union_caveats(supp.concat(con))
    };
}

fn get_mushroom_view(m: Mushroom) -> any {
    let b = get_belief();
    let con = b.contradictedBy;
    let supp = b.supportedBy;
    
    let present = m.consumed_timer == -1;
    let label = "";
    let canAbsorb = false;
    let canTaste = false;
    let because = [];
    let why_absorb = { reason: "", because: [], caveats: [] };
    let why_taste = { reason: "", because: [], caveats: [] };

    if (!present) {
        why_absorb = { reason: "Already eaten", because: [m.consumed_by], caveats: union_caveats([m.consumed_by]) };
        why_taste = { reason: "Already eaten", because: [m.consumed_by], caveats: union_caveats([m.consumed_by]) };
    } else if (m.tasted_kind == "glowcap") {
        label = "Glowcap";
        if (m.taste_ev.has_caveat(taste_faded)) { label = "Probably a glowcap (taste has faded)"; }
        else if (m.taste_ev.has_caveat(tasted_in_dark)) { label = "Probably a glowcap (tasted in the dark)"; }
        canAbsorb = true;
        because = [m.taste_ev];
        why_taste = { reason: "Already tasted", because: [m.taste_ev], caveats: union_caveats([m.taste_ev]) };
    } else if (m.tasted_kind == "duskcap") {
        label = "Duskcap — avoid";
        if (m.taste_ev.has_caveat(taste_faded)) { label = "Probably a duskcap (taste has faded)"; }
        else if (m.taste_ev.has_caveat(tasted_in_dark)) { label = "Probably a duskcap (tasted in the dark)"; }
        because = [m.taste_ev];
        why_absorb = { reason: "Known duskcap", because: [m.taste_ev], caveats: union_caveats([m.taste_ev]) };
        why_taste = { reason: "Already tasted", because: [m.taste_ev], caveats: union_caveats([m.taste_ev]) };
    } else if (con.length >= 2) {
        label = "Too risky — taste first";
        canTaste = true;
        because = con;
        why_absorb = { reason: "Too risky untasted", because: con, caveats: union_caveats(con) };
    } else if (b.state == "none") {
        label = "Glowing mushroom";
        canAbsorb = true;
        canTaste = true;
    } else if (b.state == "probably_safe") {
        label = "Probably a glowcap";
        canAbsorb = true;
        canTaste = true;
        because = supp;
    } else if (b.state == "probably_unsafe") {
        label = "Probably a duskcap";
        canAbsorb = true;
        canTaste = true;
        because = con;
    } else if (b.state == "uncertain") {
        label = "Could be a duskcap — taste first";
        canAbsorb = true;
        canTaste = true;
        because = con; 
    }

    return {
        present: present,
        label: label,
        canAbsorb: canAbsorb,
        canTaste: canTaste,
        because: because,
        caveats: union_caveats(because),
        why: { absorb: why_absorb, taste: why_taste }
    };
}

query get_view() -> any {
    let mv = {};
    for (let m in mushrooms) {
        mv[m.id] = get_mushroom_view(m);
    }
    
    return {
        slime: {
            glowing: glow_timer > 0,
            heavy: heavy_timer > 0,
            heavySeconds: ceil(heavy_timer)
        },
        mushrooms: mv,
        belief: get_belief(),
        decision: {
            state: Trust.state,
            basis: Trust.basis,
            reopenedBy: Trust.reopenedBy,
            caveats: union_caveats(Trust.basis),
            history: Trust.history
        }
    };
}

```

2. The JavaScript Adapter
JavaScript

```
import { Runtime } from 'caveat-lang';
import policySource from './glowcap.cav';

export function createPolicy(savedData = null) {
    const rt = new Runtime(policySource);
    
    if (savedData) {
        rt.load(JSON.parse(savedData));
    }
    
    return {
        dispatch(event) {
            // Unhandled rejections from Caveat automatically abort the transaction and throw host-side.
            rt.dispatch(event.type, event.payload || event);
        },
        view() {
            // Maps the unified internal representation straight to the host interface
            return rt.query('get_view');
        },
        save() {
            // The JSON save must be stringified, meeting the CR12 requirement to roundtrip and fit within 4096 bytes.
            return JSON.stringify(rt.save());
        }
    };
}

```

3. Assumptions

* Syntax/Data types: I assumed Caveat has syntax resembling Rust/TypeScript `struct`, `state`, `event`, `query`, and pure functions (`fn`). It utilizes JSON-like anonymous objects and arrays for formatting data.

* Epistemic primitives: I assumed `archive` allows accumulating categorized evidence (`Support`, `Contradict`). I assumed `decision` blocks inherently possess `state`, `basis`, and `reopenedBy` properties natively managed by `decision.commit()` and `decision.reopen()`, while freezing caveats internally upon commit.

* Evidence observation: I assumed `observe TYPE(id, payload)` generates a unique evidence pointer incorporating the provided ID string, freezing the payload alongside its occurrence.

* Caveat tagging: I assumed `.add_caveat()` and `.has_caveat()` are native evidence methods, and that `.caveats()` returns an iterable/array of attached caveats.

* Transactions: I assumed `require` works identically to `assert`/throws, halting the execution event mid-flight without committing partial graph or numeric changes (as mandated in the documentation constraint).

* Math operations: I assumed standard math standard libraries `max`, `min`, and `ceil` are present in Caveat's native prelude for `heavySeconds` rounding up.

* CR10 State formatting vs Renewable Primitive: Instead of guessing a complex implicit `renewable` keyword behavior for CR10, I modeled the mushroom life-cycles and ID suffix generation manually using `life` integers. This keeps state size tiny ensuring stringified state stays `< 4096 bytes`.

* CR11 Formatting: Assumed `why.absorb` and `why.taste` can default to empty strings and arrays when the action is allowed.

* JSON API: Assumed the `caveat-lang` Node package exports a `Runtime` class which exposes `.load(json)`, `.save()`, `.dispatch(event, payload)`, and `.query(name)`.
