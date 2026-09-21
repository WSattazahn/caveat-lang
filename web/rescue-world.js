import { createBeaconWorld } from './beacon-world.js';

/** A fixed-camera presentation profile. Evaluated source bindings control all
 * entity poses and parts; source cues request transient geometric effects. */
export function createRescueWorld(canvas, options = {}) {
  return createBeaconWorld(canvas, { ...options, mode: 'rescue', interactiveCamera: false });
}
