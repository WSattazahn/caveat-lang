# Caveat plugin

The Caveatist practice for agents, as one skill, `caveatist`: keep claims with their grounds and caveats, decide with the caveat attached, and withdraw rather than erase. It is optional culture around the [Caveat language](https://github.com/WSattazahn/caveat-lang); it changes how you work, not what any runtime does.

The plugin contains one skill and nothing else: no agents, hooks, commands or MCP servers. It runs nothing, sends nothing and fetches nothing. When the `caveat-lang` runtime is installed, the skill suggests recording decisions in a Caveat program instead of prose; installing the runtime is up to you.

Install in Claude Code:

```
claude plugin marketplace add WSattazahn/caveat-lang
claude plugin install caveat@caveat-lang
```

The skill is then available as `/caveat:caveatist`. The canonical copy lives at [`caveatism/skills/caveatist/SKILL.md`](https://github.com/WSattazahn/caveat-lang/blob/main/caveatism/skills/caveatist/SKILL.md); the copy here is byte-identical, and the repository's CI checks that they match.

## Privacy

This plugin collects, stores and sends no data. It is one instruction file read by your agent; it has no code, network access or storage of its own. What your agent and Claude Code do with your data is governed by their own terms, not by this plugin.

The icon is the Mr. Caveat avatar, copied unchanged from [`caveatism/character/mr-caveat-avatar.png`](https://github.com/WSattazahn/caveat-lang/blob/main/caveatism/character/mr-caveat-avatar.png) (Mr. Caveat fortune-ticket artwork, Caveatism project), licensed [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/); see [`caveatism/LICENSE`](https://github.com/WSattazahn/caveat-lang/blob/main/caveatism/LICENSE) and [`PROVENANCE.md`](https://github.com/WSattazahn/caveat-lang/blob/main/caveatism/character/PROVENANCE.md).
