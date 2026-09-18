# Workflow learned from the video

The [reference video](https://youtu.be/L4lKXCj6oSY) presents an iterative production workflow:

- [1:44](https://youtu.be/L4lKXCj6oSY?t=104): establish concept targets before treating a scene as finished.
- [2:50](https://youtu.be/L4lKXCj6oSY?t=170): connect Blender/GLB assets, Three.js presentation, and Rapier physics where the game requires them.
- [3:45](https://youtu.be/L4lKXCj6oSY?t=225): inspect screenshots, fix visible defects, and test actual functionality.
- [6:37](https://youtu.be/L4lKXCj6oSY?t=397): use instancing to reduce repeated-object rendering costs.
- [13:21](https://youtu.be/L4lKXCj6oSY?t=801): commit, deploy, and verify the published result.

## Adaptation to The Last Beacon

The lesson is to connect design, implementation, inspection, and correction. This project established its visual direction through the designed interface and procedural island scene; it did not generate concept images before construction. Three.js procedural geometry fits the compact island, with source-authored movement and collision equations replacing the need for an external physics engine in this small game.

CAVEAT remains authoritative for decisions and consequences, including evidence requirements and nine outcomes across 324 legal routes. Exhaustive tests follow available options; local browser QA checks WebAssembly, mobile presentation, and endings. Screenshot inspection feeds visible defect fixes back into the build. Deployment status is confirmed separately through the hosting service; local browser checks do not claim to test the published URL.

## Action-game iteration

The Light the Way revision applies the same inspect-and-correct loop to a clearer goal: guide a visible ferry into the harbor. Screenshot review caught a reversed ferry heading; complete playthroughs exposed a nearly straight safe route, so the Caveat source now places alternating reef barriers. Mouse and native touch tests complete the rescue; keyboard, collision, retry, and failed-loading checks verify the actual controls. Caveat's reactive events drive motion and use observed evidence to change the ferry's speed.
