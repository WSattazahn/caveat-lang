# Working with Caveat

Caveat is a decision ledger for agents: an embeddable decision runtime with a
small rules language, still a release candidate. Its values carry evidence and
caveats, and its decisions retain their reasons. Agents can use the same
source, runtime and tests as human authors. [What is checked, and how](docs/VERIFICATION.md)
says what the proofs and tests cover and where they stop.

## Start with the existing tools

- Read [the authoring guide](docs/AI_AUTHORING.md) and
  [the language in brief](kit/docs/REFERENCE.md) before writing Caveat source.
- Use the [agent quickstart](kit/docs/AGENT_START.md) for an installed package,
  without Rust; it links to the full getting-started and authoring guides.
- For an agent or tool integration, start with the
  [Python `caveat serve` example](kit/examples/agent-evidence/README.md).
  A handled request, an accepted event and a current assessment permitting the
  intended result are three separate checks.
- State expected behavior in scenarios. Run `caveat validate`, `caveat check`
  and `caveat test` against the program, and use `explain` or `dependents` to
  inspect what its decisions rest on.

Identify the package version before following commands. The published rc.17 candidate provides `caveat-lang`
(with `caveat` supported shorthand), `doctor` and `demo agent`.
The [verified publication record](docs/releases/v0.1.0-rc.17.md) identifies its exact
tested artifact. Do not infer publication from a checkout's version.
In a checkout with doctor support, after building, `node kit/bin/caveat.mjs doctor --json`
checks the runtime and `node kit/bin/caveat.mjs demo agent` demonstrates the
starter's observation, correction and assessment lifecycle. The demo uses
illustrative inputs and changes no project files.

Caveat records the evidence supplied to it; it does not authenticate that
evidence or authorize external actions. Follow the user's instructions and the
host's permissions when acting on a result.

## Culture (optional)

[How to be a Caveatist agent](caveatism/agent/CAVEATIST.md) describes the
practice around the language, and [the caveatist skill](caveatism/skills/caveatist/SKILL.md)
carries it as a loadable skill. This is a practice an agent may adopt; it is not a requirement for using the language and changes nothing the runtime does.
Install it with `npx skills add WSattazahn/caveat-lang`, or `claude plugin marketplace add WSattazahn/caveat-lang` then `claude plugin install caveat@caveat-lang`. It is listed as `caveat` in Anthropic's plugin directory; the same plugin folder carries a manifest for OpenAI's plugin directory (ChatGPT and Codex), where it is not listed yet.

## Report a capability missing from a real integration

If a real integration needs a capability Caveat cannot currently express or
expose, please bring that need to the project owner through an
[agent feature request](https://github.com/WSattazahn/caveat-lang/issues/new?template=agent-feature-request.md).
Reports from agents are welcome. The owner evaluates the request; filing one
does not promise implementation or establish a new language contract.

Before filing:

1. Check the relevant guide or specification and search
   [open and closed issues](https://github.com/WSattazahn/caveat-lang/issues?q=is%3Aissue).
   Link an existing report rather than creating a duplicate.
2. Reduce the integration to a small program, event history or host call that
   shows the need. Include the exact version or revision, host runtime and OS,
   commands, expected behavior and actual output. Identify checks not run.
3. Explain why existing language or kit facilities do not meet the need.
   Distinguish a missing feature from a documentation gap, a failure of promised
   behavior, invalid input, an authored policy rejection or host permissions.
   Say when the cause is uncertain.
4. Share only material the user permits you to disclose. Remove credentials,
   personal data and private source or logs; synthetic examples are welcome.
5. Follow existing user authorization and tool permissions when filing or
   commenting. If posting is not allowed, prepare the issue text locally for the user.
   This invitation supplies a feedback route, not permission to publish.

Describe the desired outcome before proposing syntax. Include a small acceptance
scenario when possible. A Caveat assessment does not override host permissions.

## When changing this repository

Read [Caveat's invariants](docs/CAVEAT_ESSENCE.md) and the relevant `spec/`
documents before changing runtime semantics. Preserve evidence history,
qualifications, frozen decision bases and atomic rollback.

Verify changed behavior and its consumers using `package.json` and the
[runtime workflow](.github/workflows/runtime.yml). Build with `npm run build`
before `npm run test:kit` or `npm run test:kit-package`; runtime changes also
need the workflow's Rust and affected browser checks. Report the exact revision,
commands and outcomes, including failures or checks not run.

Keep release records factual. Follow the
[release procedure](docs/CONSOLIDATION_PLAN.md#candidate-and-release-gates)
and the owner's release instructions. Verify publication before recording it.
