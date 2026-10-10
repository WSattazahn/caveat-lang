# V1 search-visibility baseline: provenance and annotations

[`grok-baseline-2026-10-09T04-33-13Z.txt`](grok-baseline-2026-10-09T04-33-13Z.txt)
is a third-party report, archived unchanged. This file says where it came from
and how to read it. It does not edit, correct or extend the report.

## Provenance

| Item | Value |
| --- | --- |
| Author | Grok, in the owner's own chat, using that session's `web_search` tool |
| Reported capture time | Searches issued immediately after `2026-10-09T04:33:13Z` (the report's own statement; per-query times were not returned) |
| Supplied to the project | By the owner on 2026-10-09 05:11:16 UTC, as the attachment `Caveat_Grok_Visibility_Baseline_V1.txt` to the complete 1.0 brief |
| Archived | 2026-10-10, under the [Path to 1.0](../../releases/path-to-1.0.md) plan, PR C3 |
| Bytes | 11,836, CRLF line endings, no trailing newline |
| SHA-256 | `9d56591e3d7fa08a324cbd032201b519d04b12c00780affbde2d1e4c17cd33e4` |

`.gitattributes` marks the file `-text`, so a checkout on any platform keeps
these bytes. Check with `sha256sum docs/visibility/v1/*.txt`.

We did not reproduce the report. Its results are the supplier's observations
at its capture time, not current state and not an independent measurement by
this project.

## What the report says

- None of the six problem queries (P1 to P6) returned `caveat-lang` or its
  repository among the ten summarized entries each.
- The exact-name control C1 (`caveat-lang`) returned the package, the
  repository and documentation, with rc.16 shown in several snippets.
- The ambiguous control C2 (`caveat agent MCP`) returned `caveat-lang` eighth,
  after several entries for an unrelated product, `caveat-cli`
  (kitepon-rgb/Caveat), and also `@yodolabs/caveat-mcp` and capnagent.
- The MCP Registry's latest entry and npm `next` were rc.15; npm `latest` and
  `server.json` on `main` were rc.16. The registry description was the
  authoring-bridge sentence from `server.json`.

## How to read it

- **Order is not rank.** The order is the search tool's return order. The
  report says so itself. It is not a measured organic position on any public
  search engine.
- **Summaries, not responses.** The entries are the tool's summarized extracts
  and the registry document is an excerpt. The complete tool responses were
  not supplied and are not reconstructed here.
- **Sixty entries are not sixty trials.** Six queries with ten entries each
  are one run of one tool, not sixty independent agent searches.
- **P4 and P5 have weak software intent.** Their wording ("invalidate approval
  when supporting evidence is withdrawn", "preserve original decision reasons
  after new evidence") drew legal, procurement and HR material. A miss there
  says less about software discovery than a miss on P1, P3 or P6. The queries
  stay unchanged in V1; a reworded set would be V2.
- **No cause is inferred.** The report does not show why the problem queries
  missed, and this file does not guess.
- **Not adoption evidence.** The report makes no claim about downloads,
  installs, selection or use, and neither does this archive.
- **The version lag was not intentional.** It is context we established
  afterwards, not part of the report: since rc.13, moving npm `next` and
  publishing the registry entry are owner closeout steps after each release,
  and for rc.16 they had not been done yet. The
  [Path to 1.0 baseline](../../releases/path-to-1.0.md#verified-baseline)
  records the same state on 2026-10-10.

Later runs follow the [frozen protocol](../PROTOCOL.md).
