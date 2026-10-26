# Agents 0.1 corpus sweep

[Agents 0.1](../../spec/caveat-agents-0.1.md) adds one statement,
`agent NAME [sends EVENT, …];`, and an optional agent with each event. This
sweep checks that no program in the repository changes.

```sh
node experiments/agents-sweep/sweep.mjs
# with a published package to compare against, after npm run build:
npm install --prefix /tmp/caveat-1.3.0 caveat-lang@1.3.0
node experiments/agents-sweep/sweep.mjs --against /tmp/caveat-1.3.0/node_modules/caveat-lang
```

## What it reads

- Every tracked `.cav` file, with comments and quoted text removed.
- Each statement that begins with the word `agent`. Before this profile such a
  statement did not load.
- Each other use of the words `agent` or `sends`, which must keep meaning what
  it meant.
- With `--against`, each program loaded in the published runtime and in this
  checkout's build, comparing the interface and the initial snapshot byte for
  byte. A program that does not load in one must not load in the other.

## Result

[`RESULTS.json`](RESULTS.json), against caveat-lang 1.3.0:

- 203 tracked programs. None has a statement beginning with `agent`, and none
  uses the word `agent` or `sends` anywhere outside quoted text and comments.
- 172 load in both runtimes, with the same interface and the same initial
  snapshot, byte for byte. The other 31 (modules, bundles and game programs
  the reactive runtime does not load) are refused by both.

What an event does is held to the same bytes by the existing gates, which run
every tracked program and fixture without agents: the view-delta gate over the
corpus, the save fixtures of every release since rc.17, and every scenario
file. An event sent with no agent takes the path it took before; the only new
work on it is one lookup of the event among the events some agent sends,
which is empty in a program that declares no agent.
