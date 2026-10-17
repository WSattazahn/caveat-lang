# Caveat plugin

The Caveatist practice for agents, as one skill, `caveatist`: keep claims with their grounds and caveats, decide with the caveat attached, and withdraw rather than erase. It is optional culture around the [Caveat language](https://github.com/WSattazahn/caveat-lang); it changes how you work, not what any runtime does.

The plugin contains one skill and nothing else: no agents, hooks, commands or MCP servers. It runs nothing, sends nothing and fetches nothing. When the `caveat-lang` runtime is installed, the skill suggests recording decisions in a Caveat program instead of prose; installing the runtime is up to you.

The skill gives a fixed output: every decision or review ends with a record of its decision, grounds, retained caveats, caveats not pursued, withdrawals and what would reopen it. `skills/caveatist/SKILL.md` includes a worked example.

## Install

In ChatGPT and Codex, install Caveat from OpenAI's plugin directory; those hosts read `.codex-plugin/plugin.json`.

In Claude Code:

```
claude plugin marketplace add WSattazahn/caveat-lang
claude plugin install caveat@caveat-lang
```

The skill is then available as `/caveat:caveatist`; Claude Code reads `.claude-plugin/plugin.json`. The canonical copy of the skill lives at [`caveatism/skills/caveatist/SKILL.md`](https://github.com/WSattazahn/caveat-lang/blob/main/caveatism/skills/caveatist/SKILL.md); the copy here is byte-identical, and the repository's CI checks that they match.

## Privacy

This plugin collects, stores and sends no data. It is one instruction file read by your agent; it has no code, network access or storage of its own. What your agent and its host (ChatGPT, Codex or Claude Code) do with your data is governed by their own terms, not by this plugin.

## Artwork

The Claude Code icon (`.claude-plugin/icon.png`, in the repository copy of this folder) is the Mr. Caveat avatar, copied unchanged from [`caveatism/character/mr-caveat-avatar.png`](https://github.com/WSattazahn/caveat-lang/blob/main/caveatism/character/mr-caveat-avatar.png) (Mr. Caveat fortune-ticket artwork, Caveatism project), licensed [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/); `assets/logo.png` (512×512) and `assets/icon.png` (256×256) are the same avatar downscaled, under the same license; see [`caveatism/LICENSE`](https://github.com/WSattazahn/caveat-lang/blob/main/caveatism/LICENSE) and [`PROVENANCE.md`](https://github.com/WSattazahn/caveat-lang/blob/main/caveatism/character/PROVENANCE.md).
