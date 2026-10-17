# Caveat plugin

The Caveatist practice for agents, as one skill, `caveatist`: keep claims with their grounds and caveats, decide with the caveat attached, and withdraw rather than erase. It is optional culture around the [Caveat language](https://github.com/WSattazahn/caveat-lang); it changes how you work, not what any runtime does.

In Claude Code the plugin is that one skill and nothing else: no agents, hooks, commands or MCP servers. It runs nothing, sends nothing and fetches nothing. When the `caveat-lang` runtime is installed, the skill suggests recording decisions in a Caveat program instead of prose; installing the runtime is up to you.

In ChatGPT and Codex the plugin also connects the hosted Caveat MCP server (`.codex-plugin/mcp.json`), so the agent can validate, check, test, explain and trace Caveat programs without installing anything. Its five tools are the published package's read-only authoring tools; [`hosted-mcp/`](https://github.com/WSattazahn/caveat-lang/tree/main/hosted-mcp) is its source.

The skill gives a fixed output: every decision or review ends with a record of its decision, grounds, retained caveats, caveats not pursued, withdrawals and what would reopen it. `skills/caveatist/SKILL.md` includes a worked example.

## Install

In ChatGPT and Codex, install Caveat from OpenAI's plugin directory; those hosts read `.codex-plugin/plugin.json`.

In Claude Code:

```
claude plugin marketplace add WSattazahn/caveat-lang
claude plugin install caveat@caveat-lang
```

The skill is then available as `/caveat:caveatist`; Claude Code reads `.claude-plugin/plugin.json`. The canonical copy of the skill lives at [`caveatism/skills/caveatist/SKILL.md`](https://github.com/WSattazahn/caveat-lang/blob/main/caveatism/skills/caveatist/SKILL.md); the copy here is byte-identical, and the repository's CI checks that they match.

## Privacy and terms

The skill collects, stores and sends no data. It is one instruction file read by your agent; it has no code, network access or storage of its own. In ChatGPT and Codex, tool calls send the program text and events your agent supplies to the hosted server, which keeps nothing between requests; see the [privacy notice](https://wsattazahn.github.io/caveat-lang/plugin-privacy.html) and [terms of service](https://wsattazahn.github.io/caveat-lang/plugin-terms.html). What your agent and its host (ChatGPT, Codex or Claude Code) do with your data is governed by their own terms, not by this plugin.

## Artwork

The Claude Code icon (`.claude-plugin/icon.png`, in the repository copy of this folder) is the Mr. Caveat avatar, copied unchanged from [`caveatism/character/mr-caveat-avatar.png`](https://github.com/WSattazahn/caveat-lang/blob/main/caveatism/character/mr-caveat-avatar.png) (Mr. Caveat fortune-ticket artwork, Caveatism project), licensed [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/); `assets/logo.png` (512×512) and `assets/icon.png` (256×256) are the same avatar downscaled, under the same license; see [`caveatism/LICENSE`](https://github.com/WSattazahn/caveat-lang/blob/main/caveatism/LICENSE) and [`PROVENANCE.md`](https://github.com/WSattazahn/caveat-lang/blob/main/caveatism/character/PROVENANCE.md).
