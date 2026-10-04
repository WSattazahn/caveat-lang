# Caveatism

The culture around Mr. Caveat and the Caveat language: a fictional philosophy, its archive, its atlas, a practice an agent may adopt, and the Archive kept as a program.

Three rules govern everything in this directory, and they come from the material itself:

1. **Fiction, not doctrine.** The Atlas is "a fictional cultural atlas"; the Archive is "unfinished" by its own heading. Nothing here is a contract of the language. The language's contracts live in `spec/`, its package in `kit/`, its release records in `docs/releases/`. Nothing in this directory is a release gate, and no claim about the language may be made here that the repository's records do not support.
2. **Preserve, don't rewrite.** The Archive Rule: "do not protect Caveatism by changing it every time it is challenged." Superseded versions stay; amendments are appended; contradictions may remain visible. The canon program enforces the same rule mechanically.
3. **Classify before absorbing.** New material carries a marker — observed, proposed, tested, mythological, analogy, rejected, unresolved — before it joins the record.

## Layout

| Path | What it is |
| --- | --- |
| `archive/` | The Caveatist Archive, v1.1, with v1.0 and its amendments preserved inside it. |
| `atlas/` | The Caveatist Atlas, v1.0: culture, institutions and material life. |
| `agent/CAVEATIST.md` | The practice, written for an agent. Optional; linked from `AGENTS.md` as culture. |
| `skills/caveatist/SKILL.md` | The same practice as a loadable skill (copied to `/skills/caveatist/` for installers; the copy here is canonical, CI checks they match). |
| `canon/` | The Archive as a Caveat program, with its events and scenarios. Verified on `caveat-lang@0.1.0-rc.11`. |
| `character/` | Mr. Caveat: the hero, avatar and social images, their prompts, candidates, proofs, validation and provenance. |

## Mr. Caveat

A mechanical fortune teller who always has a caveat. Cautious, curious, confidently wrong, always with a fine print. He is a recurring figure, not an authority: the Archive leaves open whether he should be imitated, resisted, consulted or laughed at.

## Licenses

Text in this directory (the Archive, the Atlas, the agent practice, the skill and these notes) is licensed under CC BY-SA 4.0. The artwork in `character/` is licensed under CC BY-SA 4.0 by the notice in `LICENSE`, with the scope and qualifications stated there. The Caveat language and kit keep their own license (MIT); nothing here changes it. The canon program in `canon/` is source under the repository's MIT license so that it can live in the program corpus.
