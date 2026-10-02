# CAVEAT Language name audit — 2026-10-01

This record supports keeping **CAVEAT Language**, the `caveat-lang` package and
repository, and the `.cav` extension while using `caveat-lang` as the explicit CLI
name. It confirms several existing uses of CAVEAT, including two other tools
that install `caveat`. It does **not** establish exclusive use of the name,
package/domain availability, or trademark clearance.

**Observation window:** 2026-10-02 00:21–00:29 UTC, which is 2026-10-01
17:21–17:29 PDT (`America/Los_Angeles`, UTC−07:00). The filename uses the local
date. This is a frozen discovery record, not a live status page.

**Scope:** public npm and GitHub metadata; PyPI; crates.io; selected other package
registries; relevant computing research; six exact domain lookups plus an `.io`
follow-up; and official US, international and European trademark entry points.
The catalog checks below were performed. Interactive trademark result searches
remain incomplete, with concrete follow-up queries recorded below. No purchases,
registrations, installations of discovered projects, or contacts were made.

## Identity and immediate compatibility result

The [CAVEAT Language repository](https://github.com/WSattazahn/caveat-lang) was
created at **2026-09-17 07:44:04 UTC**, according to GitHub's repository metadata.
Its [npm record](https://registry.npmjs.org/caveat-lang) was created at
**2026-09-25 22:38:55.932 UTC**. At retrieval, `latest` pointed to **0.1.0-rc.5**
and `next` to **0.1.0-rc.6**; rc.6's publication timestamp was
**2026-10-01 21:00:31.795 UTC**. Both published manifests exposed only `caveat`.
The checkout's rc.7 version is not evidence that rc.7 has been published.

The [npm `caveat-cli` record](https://registry.npmjs.org/caveat-cli) identified
**0.19.13** as `latest`, published **2026-09-28 02:25:47.701 UTC**, with
`bin.caveat = dist/caveat.js`. Its purpose is persistent coding-agent memory.
The source-installed Python project
[`goodekang/CAVEAT`](https://github.com/goodekang/CAVEAT) also declares
`caveat = "caveat.cli:main"` in
[its packaging metadata](https://github.com/goodekang/CAVEAT/blob/main/pyproject.toml).
That project names its Python distribution `caveat-verify`; no PyPI publication
was established. Its CLI overlap exists independently of the npm collision.

The rc.7 work retains `caveat` and adds the explicit `caveat-lang` spelling.
The [command identity note](../kit/docs/NAMES.md) and
[locked npm collision fixture](../scripts/fixtures/cli-collision/package-lock.json)
address the known npm collision. This audit does not independently rerun that
fixture or imply that an alias reserves a name across every package manager.
Use the package-gate receipts for the actual install-order verification.

## Relevant identities and overlap

Dates labeled **repository** below are GitHub `created_at`, not first use,
first public availability, authorship priority, or a legal priority date.
Descriptions summarize each project's own metadata or documentation; their
claimed capabilities were not independently evaluated. “Overlap” is an
engineering/discovery assessment, not a trademark-confusion determination.

| Identity and canonical primary source | Verified dated evidence | Purpose and overlap with CAVEAT Language |
|---|---|---|
| [`kitepon/Caveat`](https://github.com/kitepon/Caveat), npm `caveat-cli` | Repository: 2026-04-18 14:17:16 UTC. npm record created: 2026-04-19 01:31:00.815 UTC. | Coding-agent memory using Markdown, SQLite retrieval, MCP and hooks. Same agent audience and **same `caveat` executable**, different language/runtime purpose. |
| [npm `caveat`](https://registry.npmjs.org/caveat) | npm record created: 2016-03-11 13:35:23.735 UTC; `latest` is 1.0.0. | JavaScript console-warning utility; manifest has no `bin`. Exact unqualified npm package-name overlap. Metadata links `epiloque/caveat`, but that repository API returned 404 during this audit. |
| [CEA's historical Caveat, described by its developers in the Frama-C experience report](https://frama-c.com/download/u3cat/CuoqICFP09.pdf) | ICFP 2009 paper, conference August 31–September 2, 2009, section 2.3. Original project creation date not established. | Hoare-logic verification of C software, including avionics work. Close programming/verification terminology; distinct system. The paper cites earlier work, but this record does not turn those citations into a verified launch date. |
| [`DevL/CAVEAT`](https://github.com/DevL/CAVEAT) | Repository: 2015-07-13 12:06:02 UTC. README retrieved through the GitHub API. | Virtual machine/assembly design with registers, instructions and memory mapping. Programming-tool name overlap; no evidence/decision language claim in the inspected README. |
| [`goodekang/CAVEAT`](https://github.com/goodekang/CAVEAT) | Repository: 2026-09-12 04:20:04 UTC. | Active verification for multi-agent LLM reasoning; includes a claim-graph allocator and a **`caveat` CLI**. Agent/reliability terminology and command overlap. |
| [`intellicat-ai/caveat`](https://github.com/intellicat-ai/caveat) | Repository: 2026-04-08 18:13:46 UTC. | OWL ontology for classifying unreliable scientific publications and evidence markers. Meaningful evidence/assessment vocabulary overlap; an ontology rather than this executable language. |
| [`Noobatphython/caveat`](https://github.com/Noobatphython/caveat) | Repository: 2026-08-27 13:37:00 UTC. | Statistics workbench that supplies publisher caveats through WebMCP. Agent-facing evidence/qualification vocabulary overlap; application rather than language. |
| [`lolaaa00/caveat`](https://github.com/lolaaa00/caveat) | Repository: 2026-09-13 10:29:54 UTC. | Context-aware execution checkpoint for agents on GenLayer. Agent decision/verification vocabulary overlap; a different platform and execution model. |
| [`ERAS-Research/caveat`](https://github.com/ERAS-Research/caveat) | Repository: 2025-01-24 22:01:58 UTC. | Context-Aware Verification, Emulation, and Training for laboratory hardware and digital design. Verification-tool terminology overlap. |
| [`big-ucl/caveat`](https://github.com/big-ucl/caveat) | Repository: 2023-10-13 17:24:25 UTC. | Generation of human activity sequences with variational autoencoders. Machine-learning research name overlap, different task. |
| [`CloudSecurityAlliance-WG/CAVEaT`](https://github.com/CloudSecurityAlliance-WG/CAVEaT) | Repository: 2024-03-22 21:05:04 UTC. | Cloud Adversarial Vectors, Exploits, and Threats, including threat-intelligence data. Security/tooling discovery overlap; README described the project as on hiatus at retrieval. |
| [`googlefonts/caveat`](https://github.com/googlefonts/caveat), [npm `@fontsource/caveat`](https://registry.npmjs.org/@fontsource%2fcaveat) | Repository: 2015-10-07 13:28:53 UTC. npm font package created: 2020-12-23 22:15:10.005 UTC. | Handwriting font. Strong exact-name search noise; no runtime or command overlap established. npm `latest` was 5.3.0, with no `bin`. |
| [CAVEAT computer-use-agent benchmark, arXiv:2609.27273](https://arxiv.org/abs/2609.27273) | v1: 2026-09-23 03:00:00 UTC; v2: 2026-09-27 20:49:01 UTC. | Li, Epperson, Deng and Huang study agents in environments with misaligned incentives and describe CAVEAT-Harness. Close agent/decision research terminology; benchmark and interventions rather than this language. |
| [Caveat at `caveat.ai`](https://caveat.ai/privacy) | Privacy-policy effective date: 2026-07-23. Domain registration: 2022-08-02; neither date proves product launch. | AI-assisted invention disclosure and patentability work. Software/AI name overlap, different application domain. |

These are selected identities, not a claim that all similarly named projects
are equivalent competitors. The saved GitHub search contains **216 distinct
repositories across three pages**, including additional health, contract,
finance, mapping and miscellaneous projects. The search covered names, not all
README text, private repositories, deleted repositories or other code hosts.

## Catalog checks and their limits

The linked JSON files retain selected public fields, exact request URLs,
request-start timestamps and HTTP outcomes. They are excerpts of responses,
not authenticated registry archives or full mirrors.

| Catalog | Method and observed result | Limit / reproducible follow-up |
|---|---|---|
| npm | Direct metadata for `caveat`, `caveat-lang`, `caveat-cli`, and `@fontsource/caveat`; dates, tags and `bin` fields recorded above. | Four specific identities, not every package containing the word. Re-read those registry URLs if a particular release or executable changes. |
| GitHub | Public repository search `caveat in:name`, sorted by stars, pages 1–3 of 100; 216 distinct rows, `incomplete_results: false` on each response. Direct project metadata and selected READMEs inspected. | The API's result set is not all historical use. Pagination was not an atomic snapshot; no cross-host or private-code search. Exact request URLs and returned descriptions are saved. |
| PyPI | `https://pypi.org/simple/` JSON names scanned case-insensitively for `caveat`: **903,847 names**, one match, `fontpkg-caveat`. Direct JSON for `caveat`, `caveat-lang`, `caveat-cli` returned 404. | Only the public index at retrieval. Reserved, removed or future names are not established as available. Source-installed Python tools can still expose `caveat`. |
| PyPI font match | [Official metadata](https://pypi.org/pypi/fontpkg-caveat/json): font family package, version 2.0; earliest upload in returned releases **2026-09-15 01:28:07.301811 UTC**. | Earliest returned upload is not a project-creation claim. |
| crates.io | Direct `/api/v1/crates/{name}` for `caveat`, `caveat-lang`, `caveat-cli`: 404. Text query `q=caveat&per_page=30` returned total 2,547, first 30 inspected. | Broad text matches include descriptions about caveats/macaroons; those totals are **not 2,547 name collisions**. Remaining text-result pages were not reviewed; exact-name responses do not establish reservation eligibility. |
| RubyGems | [Exact `caveat` metadata](https://rubygems.org/api/v1/gems/caveat.json): 404. | No conclusion about similarly named gems; follow-up query is `caveat` in RubyGems search if Ruby distribution is contemplated. |
| NuGet | [Search `q=caveat`, prereleases included](https://azuresearch-usnc.nuget.org/query?q=caveat&take=30&prerelease=true): six results, all differently named packages whose descriptions matched. [Exact flat-container name](https://api.nuget.org/v3-flatcontainer/caveat/index.json): 404. | No ownership or availability conclusion. Saved results show the six actual identities. |
| Maven Central | Broad `q=caveat` request timed out. Retried [exact artifact query `a:caveat`](https://search.maven.org/solrsearch/select?q=a:caveat&rows=30&wt=json): successful response, `numFound: 0`. | Exact artifact query succeeded; broad group/artifact/description search remains incomplete. Retry the saved broad URL if Java distribution becomes relevant. |
| Packagist | [Search `q=caveat`](https://packagist.org/search.json?q=caveat): five results, no package named `caveat` in that returned set. | Composer names normally include a vendor; the result is scoped to this query, not an unqualified-name reservation check. |
| Homebrew | Exact [formula](https://formulae.brew.sh/api/formula/caveat.json) and [cask](https://formulae.brew.sh/api/cask/caveat.json) endpoints returned 404. | Third-party taps and executables bundled under other formula names were not checked. No claim about all OS package managers (for example apt, winget or Chocolatey). |
| Research literature | Web queries for `CAVEAT` with programming/verification, CEA and arXiv, followed by primary paper reads. The 2009 CEA report and 2026 agent benchmark are recorded above. | Discovery searches are not a complete bibliographic index. Ordinary uses of “caveat” and “caveat emptor” were not treated as project identities. No peer-review or performance endorsement is implied. |

Evidence: [registry responses](name-audit/registry-evidence-2026-10-01.json) and
[additional catalogs, GitHub pages and domains](name-audit/catalog-domain-evidence-2026-10-01.json).

## Domains

RDAP servers for `.dev`, `.ai` and `.org` were selected from the
[IANA DNS bootstrap](https://data.iana.org/rdap/dns.json); `.com` was checked
against the Verisign registry. Registration events are domain-record dates,
not evidence of when a product launched, who originated a name, or whether a
registrant is willing to transfer it. No registrant contact details were saved.

| Domain | Observed primary evidence | Interpretation |
|---|---|---|
| `caveat.com` | [Registry RDAP](https://rdap.verisign.com/com/v1/domain/caveat.com): registration **1996-05-22 04:00:00 UTC**. HTTP site fetch returned 403. | Registered; website purpose not verified. |
| `caveat.dev` | [Registry RDAP](https://pubapi.registry.google/rdap/domain/caveat.dev): registration **2025-05-04 12:57:59.564 UTC**. | Registered; website purpose not verified. |
| `caveat.ai` | [Registry RDAP](https://rdap.identitydigital.services/rdap/domain/caveat.ai): registration **2022-08-02 16:05:40 UTC**. [Product policy](https://caveat.ai/privacy) describes invention/patent software. | Registered and associated with a different software service at retrieval. |
| `caveat.org` | [Registry RDAP](https://rdap.publicinterestregistry.org/rdap/domain/caveat.org): registration **1995-10-22 04:00:00.29 UTC**. | Registered; website purpose not inspected. |
| `caveat-lang.com` | [Exact registry RDAP](https://rdap.verisign.com/com/v1/domain/caveat-lang.com): 404. | No domain object returned by this query. **Not a purchase-availability statement.** |
| `caveat-lang.dev` | [Exact registry RDAP](https://pubapi.registry.google/rdap/domain/caveat-lang.dev): 404. | Same limit; no reservation or purchase was attempted. |
| `caveat.io` | Direct website could not be read. [IANA's `.io` delegation](https://www.iana.org/domains/root/db/io.html) provides `whois.nic.io`; `.io` was absent from the retrieved RDAP bootstrap subset. | Registration/date **unverified here**. Follow up with `caveat.io` at the registry WHOIS service. A search-engine-discovered resale listing is not substituted for registry verification. |

No inference about the availability of an entire brand follows from a domain
404. The existing GitHub Pages address already identifies the language's owner
and repository; this audit does not require a domain change.

## Trademark databases: findings and unfinished searches

This section records public database observations, not legal advice or a legal
clearance opinion. The official
[USPTO search page](https://www.uspto.gov/trademarks/search) distinguishes a
federal search from comprehensive clearance work. WIPO's
[database description](https://www.wipo.int/en/web/global-brand-database)
identifies participating collections and advises considering national/regional
registers as well.

| Official source | What was actually retrieved | What remains unresolved |
|---|---|---|
| [USPTO Trademark Search](https://tmsearch.uspto.gov/) | Official entry page reached, but no result data rendered in the text browser. The older `trademarksearch.uspto.gov` address also failed. | Run word-mark searches for `CAVEAT`, `CAVEAT LANGUAGE` and `CAVEAT-LANG`, including live and dead records and similar-word searches; inspect goods/services and record stable serial/registration numbers. Search broadly before narrowing to software-related goods. **No negative result is claimed.** |
| [USPTO TTABVUE cancellation 92078868](https://ttabvue.uspto.gov/ttabvue/v?pno=92078868&pty=CAN) | Direct HTML retrieval succeeded. Proceeding filed **2022-01-18**, terminated **2022-07-18**. It displayed CAVEAT serial **87139754**, registration **5166173**, Smack Global, Inc., as `CANCELLED - SECTION 18`; and CAVEAT serial **90138142**, registration **6903694**, Hello Caveat, LLC, as `REGISTERED`. | These are displayed fields in a specific proceeding, **not an exhaustive mark search or independent confirmation of current TSDR status**. Goods/services and field overlap were not established. |
| [TSDR serial 90138142](https://tsdr.uspto.gov/#caseNumber=90138142&caseSearchType=US_APPLICATION&caseType=DEFAULT&searchType=statusSearch) | HTTP 200 returned the application shell without the case data. | Open this serial and 87139754 in a working interactive session; verify current status, owner, goods/services and dated prosecution record. |
| [WIPO Global Brand Database](https://branddb.wipo.int/) | Entry point and official collection description reached; no query results retrievable. | Run the same word variants; record the collections and jurisdictions actually searched, status and goods/services. Do not treat participation as global coverage. |
| [EUIPO/TMview](https://www.tmdn.org/tmview/) | Entry point returned an empty text-rendered application. | Run the same variants in a working session, recording office/jurisdiction filters, results and goods/services. |

A browser-based follow-up was attempted through the available computer-use
interface: opening the in-app browser reported `Browser is not available: iab`,
and the browser inventory was empty. No CAPTCHA, account creation or permission
change was attempted. These concrete interface limits leave the three live
result searches open; they do not turn them into “no conflicting marks found.”

## Recorded decision and bounded follow-up

Continue the existing rc.7 namespace hardening: prominent **CAVEAT Language**
identity, explicit `caveat-lang` executable, supported `caveat` shorthand,
version/help identity checks and the small disambiguation note. Package metadata
proves why the command distinction matters. The other projects justify precise
project links and wording in discovery material; they do not themselves require
new language features or a rename.

Before treating a pre-1.0 naming review as complete, finish the three interactive
trademark result searches and the identified TSDR record checks, with the owner
choosing the relevant markets and any professional legal review. The broader
Maven query and `.io` WHOIS lookup remain specifically documented research gaps.
There is no legal clearance conclusion to carry into a release record.

Preserve this dated audit and its evidence. Record materially new confirmed
collisions or completed follow-ups in a dated addendum instead of silently
rewriting the historical observations or repeatedly reopening the entire name
question. This document creates no monitoring automation.


## Addendum — 2026-10-02 00:40 UTC

**Local time:** 2026-10-01 17:40 PDT. These four follow-up requests preserve the
original 00:21–00:29 UTC observations above and close the two catalog/domain gaps
identified there. Selected responses and exact timestamps are in the
[follow-up evidence](name-audit/followup-evidence-2026-10-02.json).

- **Broad Maven query completed.** Retrying the original
  [`q=caveat&rows=30&wt=json` query](https://search.maven.org/solrsearch/select?q=caveat&rows=30&wt=json)
  returned HTTP 200 and `numFound: 2`. Both returned coordinates were
  Fontsource-related WebJars: `org.webjars.npm:fontsource__caveat` and
  `org.webjars.npm:fontsource-variable__caveat`, each with indexed latest version
  **5.1.1**. All results for this specific query were returned. This resolves
  the earlier timeout; it is not a claim about every Java package or artifact
  repository.
- **`caveat.io` registry lookup completed.** A direct WHOIS query for `caveat.io`,
  terminated by CRLF, to **`whois.nic.io:43`** (the server named by
  [IANA](https://www.iana.org/domains/root/db/io.html)) returned a domain record.
  Its creation date was **2021-08-06 14:31:28 UTC**, registry expiry
  **2027-08-06 14:31:28 UTC**, and status `clientTransferProhibited`. Nameservers
  were `ns1.atom.com` and `ns2.atom.com`. This verifies the registered domain;
  it does not establish a sale offer, ownership history, availability or
  trademark rights. Only domain-level fields were retained; no personal
  registrant/contact data was saved.
- **Specific TSDR case checks remain blocked by authentication.** The official
  case-status XML endpoints for serials
  [90138142](https://tsdrapi.uspto.gov/ts/cd/casestatus/sn90138142/info.xml) and
  [87139754](https://tsdrapi.uspto.gov/ts/cd/casestatus/sn87139754/info.xml)
  both returned **HTTP 401** to unauthenticated requests. The
  [USPTO API guide](https://www.uspto.gov/sites/default/files/documents/tm-enterprise-api-user-guide-v2.pdf)
  describes the API-key requirement. No key was sought, read or created, and
  no alternative host was used to bypass authentication. Consequently this
  addendum does not verify current goods/services, ownership or live status
  for either mark. The existing TTABVUE observations keep their original,
  narrower scope.

The remaining named research limits are the interactive USPTO, WIPO and
TMview searches and the two current TSDR case checks. The Maven and `.io`
follow-ups are now complete within their stated query scope. The naming
recommendation and absence of a legal-clearance conclusion are unchanged.
