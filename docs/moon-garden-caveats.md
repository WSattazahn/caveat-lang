# Moon Garden development caveat ledger

This file is deliberately part of the feature. CAVEAT should not only model uncertainty inside the story; the work on the story should record the important uncertainties and what was done about them.

## Caveat: this could become another technical demo instead of a game

**Consequence:** high.

**Response:** Moon Garden is a short, complete cozy mystery with a beginning, two investigation beats, a commitment that can reopen, a revised commitment, three endings, a visible character payoff, a replay button, and a compact decision trail. The root page is the game rather than a debug dashboard.

**Status:** mitigated in implementation; visual QA is required before merge.

## Caveat: the UI could merely pretend to be CAVEAT-driven

**Consequence:** catastrophic for the experiment.

**Response:** game/moon_garden.cav owns the evidence, caveats, investigation options, attention budget, choices, retained uncertainty, and commitment reopening. The browser loads that source, builds a CAVEAT Map, takes option labels from the map, and advances the actual WebSession. A CI playthrough validates the source with caveat-map and exercises the CAVEAT CLI.

**Status:** resolved by architecture and CI.

## Caveat: polish can regress on the phone where the game is actually being tried

**Consequence:** high.

**Response:** the page is mobile-first, contains no external art/font dependency, uses a finished SVG garden scene, and has an iPhone WebKit Playwright test that checks the complete playthrough, horizontal overflow, console errors, ending visibility, and start/end screenshots.

**Status:** gated by CI.

## Caveat: replacing the root demo could destroy the old Door work

**Consequence:** material.

**Response:** the original root experience is preserved at web/door.html; the 3D Door remains at web/3d.html. Both remain in the Pages bundle, and existing Door visual QA remains in place.

**Status:** resolved.

## Caveat: a deployed build could omit the CAVEAT source file

**Consequence:** high.

**Response:** both build workflows explicitly copy game/moon_garden.cav into the browser distribution. The deployed smoke test opens the live root page and completes the story, which fails if the source is absent.

**Status:** gated by deploy QA.

## Caveat: “something Simone would actually want to play” is an external human judgment

**Consequence:** material.

No reliable preference profile for Simone exists in the repository or available project context, so this cannot honestly be marked resolved by code. The implementation therefore optimizes for a broadly inviting target rather than inventing preferences: short session, cozy setting, cat-search mystery, no punishment loop, readable phone controls, clear payoff, and no developer-facing clutter in the main experience.

**Status:** requires Simone’s playtest. The software caveats above should be resolved before asking her to judge the game itself.

## Remaining architectural caveat

Moon Garden intentionally proves that the CAVEAT reasoning/session layer can drive a presentable game. Its garden illustration and scene choreography are still scenario-specific presentation code. The next engine milestone should extract those presentation primitives so a new .cav story can reuse them without another bespoke page.

This is not allowed to block Moon Garden from looking finished; it is an engine-generalization task, not a player-facing excuse.
