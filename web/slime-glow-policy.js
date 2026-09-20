import { WebReactiveSession } from './pkg/caveat_runtime.js';

// Initialize the generated WebAssembly module before constructing this host.
// Eligibility, state transitions, feedback and qualifications stay in source.
export class SlimeGlowPolicy {
  #source;
  #session;

  constructor(source) {
    this.#source = source;
    this.#session = new WebReactiveSession(source);
  }

  #liveSession() {
    if (!this.#session) throw new Error('Slime glow policy has been freed');
    return this.#session;
  }

  snapshot() {
    return JSON.parse(this.#liveSession().snapshot());
  }

  #dispatch(event) {
    return JSON.parse(this.#liveSession().dispatch(event, '{}'));
  }

  caveEntered() {
    return this.#dispatch('cave_entered');
  }

  absorbMushroom() {
    return this.#dispatch('absorb_mushroom');
  }

  toggleGlow() {
    return this.#dispatch('toggle_glow');
  }

  reset() {
    const previous = this.#liveSession();
    // Construct first: a loader failure must not discard the previous round.
    const next = new WebReactiveSession(this.#source);
    this.#session = next;
    previous.free();
    return this.snapshot();
  }

  free() {
    this.#session?.free();
    this.#session = null;
  }
}
