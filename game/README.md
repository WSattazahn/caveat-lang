# CAVEAT Game Prototype — The Door

The first game is intentionally tiny. Its purpose is to prove that CAVEAT semantics can drive play rather than merely describe reasoning.

## Scenario

The player stands before a sealed door during an evacuation. A camera reports the corridor beyond is clear. A latch sensor reports the door is unsafe. A maintenance note may qualify the latch reading. The player has a finite investigation budget and must eventually commit to **open**, **wait**, or **reroute** while unresolved caveats may remain.

After commitment, new evidence can arrive and reopen the earlier decision. The game must be able to show both:

- what the world contains now;
- why the player/NPC decision was reasonable when it was made.

## Language features under test

- contradictory evidence without explosion;
- provenance-bearing support/opposition;
- consequential caveats;
- finite examination budget;
- explicit commitment under unresolved qualification;
- immutable commitment snapshot;
- reopening after new evidence;
- dependency-sensitive caveat impact.

## Prototype boundary

No graphics engine yet. Version G0 is a deterministic text simulation driven by `.cav` world state. A graphical client can be added only after the language can support the interaction loop cleanly.
