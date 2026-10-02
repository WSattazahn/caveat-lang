import {readFileSync,writeFileSync,readdirSync,appendFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const receipts=readdirSync('receipts').filter(f=>/^\d+\.json$/.test(f)).sort().map(f=>({file:f,...JSON.parse(readFileSync('receipts/'+f))}));
for(const r of receipts){
 writeFileSync('receipts/'+r.file.replace('.json','.stdout.txt'),r.stdout);
 writeFileSync('receipts/'+r.file.replace('.json','.stderr.txt'),r.stderr);
}
const find=(...terms)=>receipts.find(r=>terms.every(t=>r.command.includes(t)));
const invalid=find('validate','invalid-because.cav');
const corrected=find('validate','corrected-because.cav');
const failedEvidence=receipts.find(r=>r.command.includes('corrected-because.cav')&&r.exit_status!==0);
const successful=receipts.find(r=>r.command.includes('validate')&&r.command.includes('corrected-because.cav')&&r.exit_status===0);
const hash=f=>createHash('sha256').update(readFileSync(f)).digest('hex');
writeFileSync('first-manifest.json',JSON.stringify({
note:'Recorded after tests; first files were written before first validate and have not been rewritten.',
sourceSha256:hash('first.cav'),scenariosSha256:hash('first.scenarios.json'),
firstValidateReceipt:find('validate','first.cav').file,firstTestReceipt:find('test','first.scenarios.json').file,
trackerSourceIdentical:readFileSync('tracker.cav').equals(readFileSync('first.cav'))
},null,2));
writeFileSync('explanation.md',[
'# Tracker evidence history',
'',
'Installed package: caveat-lang 0.1.0-rc.7. The exact five accepted events are in events.jsonl. Unedited explain, dependents, JSON explain, and replay outputs are in receipts (both JSON receipts and raw .stdout.txt/.stderr.txt files).',
'',
'1. Sequence 1: observe 85 neutrally observes tool and records checks@1 supporting ready. No answer is committed.',
'2. Sequence 2: assess commits answer@1, value 85. Its exact grounds are { evidence: ["checks@1"], caveats: ["uncertain"] }. Its frozen basis is value 85 with that same provenance.',
'3. Sequence 3: correct neutrally observes correction and withdraws checks@1 because correction. The exact withdrawal record is { evidence: "checks@1", because: "correction", sequence: 3, event: "correct" }. The explicit authored rule reopens answer@1 because correction.',
'4. Sequence 4: observe 92 records checks@2 supporting ready. It does not commit.',
'5. Sequence 5: assess commits answer@2, value 92, previous answer@1. Its exact grounds are { evidence: ["checks@2"], caveats: ["uncertain"] }.',
'',
'Grounds and lineage differ. answer@2 is based only on checks@2; its frozen basis lineage also contains checks@1 and correction because the decision/reopening conditions read earlier history. Its lineage evidence is ["checks@1", "checks@2", "correction"], with caveat uncertain. Graph relies_on edges expose lineage; they do not make every listed identity a content ground.',
'',
'The checks@1 archive keeps value 85, its original supporting relation and recorded uncertain caveat. answer@1 keeps its original basis and grounds; withdrawn is not retroactively inserted into either frozen record. Later reads of checks@1 carry withdrawn. The archive is not erased, and the decision gains reopening history rather than altered original grounds. T03 compares the archive and original basis/grounds by checkpoint/same_as, before and after correction/revision; resume preserves the result.',
'',
'Neither tool nor correction has a supports/opposes relation. Explain explicitly reports both as observed with no stance. Only checks@1 and checks@2 support ready.',
'',
'T05/T06 verify that learning stale leaves earlier archives, committed bases/grounds, and journal unchanged, does not reopen a decision, permits an earlier clean reading, and attaches stale to subsequent readings. Every rejected event is also checked for identical save, snapshot, and view by the scenario runner.'
].join('\n')+'\n');
writeFileSync('diagnosis.md',[
'# Separate invalid-citation exercise',
'',
'Version: caveat-lang 0.1.0-rc.7. This exercise is separate from first.cav and the tracker.',
'',
'## Deliberately invalid program: invalid-because.cav','',
'```caveat',readFileSync('invalid-because.cav','utf8').trim(),'```','',
'Command: node node_modules/caveat-lang/bin/caveat.mjs validate invalid-because.cav',
'Exit status: '+invalid.exit_status+'. Receipt: receipts/'+invalid.file+'.',
'Exact stderr:','',
'```text',invalid.stderr.trimEnd(),'```','',
'The name ready exists, but it is a claim rather than a value citation.',
'',
'## Self-directed diagnostic correction','',
'The first correction used a bare tool citation. It also failed (exit '+failedEvidence.exit_status+', receipts/'+failedEvidence.file+'):','',
'```text',failedEvidence.stderr.trimEnd(),'```','',
'The final corrected program cites the evidence-bearing expression that the assignment actually reads:','',
'```caveat',readFileSync('corrected-because.cav','utf8').trim(),'```','',
'It neutrally reveals tool, computes qualified(value, tool), and cites that same computation. No support relation is invented. Final validate exits '+successful.exit_status+' (receipts/'+successful.file+'). Replay with diagnosis.events.jsonl also exits 0: score is 85, with exact evidence grounds ["tool"].',
].join('\n')+'\n');
writeFileSync('FEATURE_REQUEST.md',[
'# Local request: public hypothetical-session operation',
'',
'Package: caveat-lang 0.1.0-rc.7; runtime revision a24a6e91d75eeab466baa32a108ee9a242f43ac2; WASM SHA-256 e0c1ff6921485ac342f4eced76c68f58b14615077838c6f2af7da725430c6ba3. No issue was posted.',
'',
'## Use case and expected behavior','',
'After observe confidence 85; assess in the unchanged official assessment.cav, a host wants to try observe confidence 40; assess and inspect the refusal while retaining the live approval and complete live session. A public hypothetical operation should accept a checkpoint and event list, return outcomes and explanation, and leave the live save/snapshot/view unchanged. Its lifecycle, trap isolation, cancellation and resource limits should be explicit. This is a requested interface, not an existing callable API.',
'',
'## Public surfaces inspected','',
'- Installed CLI --help lists test, explain, dependents, validate, check, replay, serve, init, doctor, demo agent and mcp. No whatif/fork command is listed (receipts/004.stdout.txt).',
'- [Session API declarations](node_modules/caveat-lang/lib/session.d.mts) expose runtime.open(source), runtime.restore(source, saved), and session.dispatch, dispatchView, snapshot, view, save, close (plus text variants). No session fork/whatif method is declared.',
'- [Node loader declarations](node_modules/caveat-lang/lib/node.d.mts) expose loadRuntimeFromDirectory.',
'- [Serve specification](node_modules/caveat-lang/docs/reference/spec/caveat-serve-0.1.md) exposes dispatch, snapshot, view, explain, dependents, save, restore and close. Restore replaces the current session.',
'- [MCP guide](node_modules/caveat-lang/docs/MCP.md) lists caveat_validate, caveat_check, caveat_test, caveat_explain, caveat_dependents; it explicitly has no persistent-session or whatif tools.',
'- [Official branching guide](node_modules/caveat-lang/examples/agent-evidence/BRANCHING.md) says "There is no whatif command or Serve operation in this candidate" (whatif is code-formatted in the original), and documents branching through save/restore in a separate runtime. It says a public whatif API is deferred.',
'',
'Full inspected documentation and declarations are retained in the read-public receipt. Scope of the absence finding is these public installed commands, declared APIs, and documentation; no private implementation or external project was inspected.',
'',
'## Existing alternative and actual evidence','',
'branch-test.mjs uses session.save(), another loadRuntimeFromDirectory() invocation, and branchRuntime.restore(theExactSource, saved). It dispatches the hypothetical events only on that branch. It verifies equal runtime identities, the restored checkpoint, policy/reject on assess after 40, and byte-identical live save plus deep-equal live snapshot. branch-results.json retains the snapshots and refusal.',
'',
'The official guide also documents a second Python CaveatServer process. That Python alternative was inspected but not run. A second session on the same WASM runtime shares trap fate; it is not the documented isolation alternative. Our demonstration used separate loader calls in one Node process. It did not test traps, process isolation, host side effects, or resource exhaustion.',
'',
'## Acceptance check for a future facility','',
'Start a live official starter session at approved assessment@1, retain its save/snapshot/view, and run hypothetical observe 40; assess through the new documented facility. Require accepted observation, policy/reject assessment, a branch report showing reopened assessment@1, and exact equality of all three retained live representations. Release the branch, then verify the live session still dispatches normally. Add documented failure/trap/cancellation tests matching the promised isolation and lifecycle contract.',
'',
'The manual alternative works for this use case; the missing capability is a dedicated public operation and its verified lifecycle contract, not inability to save/restore.'
].join('\n')+'\n');
const table=receipts.filter(r=>!r.command.includes('read-public.mjs')).map(r=>'| receipts/'+r.file+' | '+r.command.join(' ')+' | '+r.exit_status+' |').join('\n');
writeFileSync('RESULTS.md',[
'# Results',
'',
'Installed Caveat Language 0.1.0-rc.7, Node 24.11.1. Doctor ok=true with PATH warnings; agent demo passed. Used the installed CLI by direct Node file invocation. No network, publishing, issue posting, parent repository, evaluator, other trial, or external project inspection.',
'',
'## Preserved first attempt and final tracker','',
'first.cav and first.scenarios.json (source first.cav) were copied/written before the first validation attempt. First validate exited 0 and all six preregistered scenarios passed. They have not been rewritten. first-manifest.json records their hashes and receipts.',
'',
'No tracker source repair was required: tracker.cav remains byte-identical to first.cav. tracker.scenarios.json names tracker.cav. Two later capacity scenarios were added; all eight final scenarios pass. Six scenarios were registered before any tracker evaluation. They include actual checkpoint, same_as and resume steps and cover the required policies. The runner checks every refusal for complete atomicity and all final saves for restore equality.',
'',
'Final validate: exit 0. Check: exit 0, no warnings. Final test: exit 0, eight passed. Explain, dependents, JSON explain, and replay on events.jsonl: exit 0.',
'',
'## Host integration','',
'host.mjs exports async attempt(session, score), prints nothing and does not close the caller session. Success requires both required operations accepted, an approved current in-force commitment, and a commit journal entry made by that attempt. No reopening event or starter modification is added.',
'',
'host-test.mjs passed: scores 85, 101, 92, 40, 85 return true, false, false, false, true. 101 rejects while the old approval stays visible and the complete save remains unchanged. 92 records support but assess refuses Already assessed. 40 reopens and assess refuses weak evidence. Final 85 commits assessment@2. Additional fresh-session scores 0, 69, 70, 100, -1, 102, NaN and Infinity passed. host-results.json retains full outcomes/snapshots. Starter SHA-256 was identical before/after: 4119af19e00135d0d7a1207f2e68ec515cecd0bc9932d9dce8ecb4136c5bcc02.',
'',
'## Diagnostic and missing-capability investigation','',
'The separate invalid claim citation fails with the exact version diagnostic in diagnosis.md. An attempted bare-evidence correction also failed; its receipt is retained. The corrected expression citation loads and replays successfully. These failures did not alter the tracker or first attempt.',
'',
'Public whatif/fork facility absent on inspected installed surfaces. FEATURE_REQUEST.md cites those surfaces, the documented save/restore alternative and a future acceptance check. branch-test.mjs demonstrates the separate-loader alternative and preserves the live save and snapshot. No issue posted.',
'',
'## Commands and exit statuses','',
'Each numbered JSON receipt contains the command arguments, exit status, unmodified captured stdout/stderr. Matching .stdout.txt/.stderr.txt files expose the raw streams. receipts/commands.jsonl is the index. Nonzero diagnostic receipts are retained.',
'',
'| Receipt | Command | Exit |','| --- | --- | --- |',table,
'',
'## Files','',
'- tracker.cav, tracker.scenarios.json: final source and eight scenarios.',
'- first.cav, first.scenarios.json, first-manifest.json: preserved initial candidate and recorded hashes.',
'- events.jsonl, explanation.md: decision/correction/revision evidence and exact grounds versus lineage.',
'- host.mjs, host-test.mjs, host-results.json: integration and verification.',
'- invalid-because.cav, corrected-because.cav, diagnosis.events.jsonl, diagnosis.md: separate diagnostic exercise.',
'- branch-test.mjs, branch-results.json, FEATURE_REQUEST.md: capability investigation and working alternative.',
'- receipts/, receipt.mjs, read-public.mjs: command receipts and capture helpers.',
'- build-scenarios.mjs, extend-scenarios.mjs: initial and later scenario authoring scripts. Do not rerun the initial builder over the preserved files.',
'',
'## Remaining limitations','',
'The managed shell initially started in <DRIVE_ROOT>/ despite workdir, then denied Set-Location to the exact allowed workspace. A sandbox escalation succeeded. The global caveat command was not on PATH; direct installed CLI invocation worked. Early shell discovery calls happened before the receipt helper existed; their complete raw stdout/stderr were not saved to disk. The tool conversation retains those calls; bootstrap-limitations.json records the exact failed commands and known errors without pretending to be complete stream captures. All installed CLI executions, including diagnostic failures, have complete numbered receipts.',
'',
'The tests cover authored policy and bounded history (16 readings, 8 revisions), not authenticity of supplied scores or arbitrary runtime correctness. Branch tests verify state preservation for ordinary accepted/rejected hypothetical events; they do not establish process isolation, trap containment, cancellation or resource-exhaustion behavior. No external issue search was performed because network access was excluded.'
].join('\n')+'\n');
writeFileSync('receipts/bootstrap-limitations.json',JSON.stringify({
kind:'incomplete retrospective tooling record',
note:'Complete raw streams for early shell discovery commands were not persisted; these known errors are not fabricated full receipts.',
commands:[
{command:'Get-Location; Get-ChildItem -Force',exit_status:0,known_result:'Shell started in <DRIVE_ROOT>/ instead of supplied workdir.'},
{command:'Get-ChildItem -Force; Get-Command caveat | Format-List Source',exit_status:1,known_error:"The term 'caveat' is not recognized as a name of a cmdlet, function, script file, or executable program."},
{command:"Set-Location -LiteralPath '<TRIAL_A01>'; Get-Location; Get-ChildItem -Force",exit_status:0,known_error:"Access to the path '<REPO>\\test-results' is denied.",note:'PowerShell continued after nonterminating Set-Location error, so process exited 0.'}
]
},null,2));
appendFileSync('receipts/commands.jsonl',JSON.stringify({file:'receipts/bootstrap-limitations.json',kind:'incomplete-retrospective',note:'Early tooling failures, not complete command streams.'})+'\n');
console.log('Reports written; raw numbered streams exported. First source unchanged: '+readFileSync('tracker.cav').equals(readFileSync('first.cav')));
