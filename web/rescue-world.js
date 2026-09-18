import { createBeaconWorld } from './beacon-world.js';

/** The rescue camera and effects are presentation only. CAVEAT owns every
 * position update, collision, discovery, damage event and winning condition. */
export function createRescueWorld(canvas, options = {}) {
  return createBeaconWorld(canvas, { ...options, mode: 'rescue', interactiveCamera: false });
}
