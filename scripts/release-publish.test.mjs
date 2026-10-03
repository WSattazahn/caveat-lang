import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import test from 'node:test';
import { parseReleaseInputs, requireUnpublished, selectRuntimeCandidate } from './release-publish.mjs';
import { evaluateProvenance, evaluateRegistryMetadata, parseArguments } from './verify-npm-publication.mjs';

const repository = 'WSattazahn/caveat-lang';
const revision = 'a'.repeat(40);
const digest = 'b'.repeat(64);
const inputs = () => ({ tag: 'v0.1.0-rc.11', sha256: digest, distTag: 'latest', ref: 'refs/tags/v0.1.0-rc.11', revision });

test('dispatch inputs name a release tag, the run ref, a SHA256 and a known dist-tag', () => {
  assert.deepEqual(parseReleaseInputs(inputs()), { tag: 'v0.1.0-rc.11', version: '0.1.0-rc.11', sha256: digest, distTag: 'latest', revision });
  const refusals = [
    [{ tag: '0.1.0-rc.11', ref: 'refs/tags/0.1.0-rc.11' }, /look like v0\.1\.0-rc\.11/],
    [{ ref: 'refs/heads/main' }, /from the tag v0\.1\.0-rc\.11 itself/],
    [{ sha256: digest.toUpperCase() }, /64 lowercase/],
    [{ sha256: '' }, /64 lowercase/],
    [{ distTag: 'beta' }, /one of next, latest/],
    [{ distTag: undefined }, /one of next, latest/],
    [{ revision: 'abc' }, /complete Git SHA/],
  ];
  for (const [change, expected] of refusals) assert.throws(() => parseReleaseInputs({ ...inputs(), ...change }), expected);
});

function github({ runs, artifacts, workflowId = 7 }) {
  return async url => {
    if (url.endsWith('/actions/workflows/runtime.yml')) return { id: workflowId };
    if (url.includes('/actions/workflows/runtime.yml/runs?')) {
      assert.match(url, new RegExp(`event=push&branch=main&head_sha=${revision}`));
      return { workflow_runs: runs };
    }
    if (url.includes('/artifacts')) return { artifacts };
    throw new Error(`unexpected ${url}`);
  };
}
const runFor = (id, change = {}) => ({ id, workflow_id: 7, path: '.github/workflows/runtime.yml', repository: { full_name: repository },
  head_repository: { full_name: repository }, event: 'push', head_branch: 'main', head_sha: revision, status: 'completed', conclusion: 'success', ...change });
const artifactFor = (runId, change = {}) => ({ id: 99, name: 'kit-package-candidate', expired: false, workflow_run: { id: runId, head_sha: revision }, ...change });

test('the newest successful Runtime push run for the revision supplies its tested package', async () => {
  const api = github({ runs: [runFor(10), runFor(12)], artifacts: [artifactFor(12), { name: 'browser-dist' }] });
  assert.deepEqual(await selectRuntimeCandidate({ api, repository, revision }), { runtime_run_id: 12, artifact_id: 99 });
});

test('a missing, failed, foreign or expired candidate is refused', async () => {
  const cases = [
    [{ runs: [], artifacts: [] }, /No Runtime run/],
    [{ runs: [runFor(10), runFor(12, { conclusion: 'failure' })], artifacts: [artifactFor(12)] }, /has not succeeded/],
    [{ runs: [runFor(12, { status: 'in_progress', conclusion: null })], artifacts: [artifactFor(12)] }, /has not succeeded/],
    [{ runs: [runFor(12, { head_repository: { full_name: 'fork/caveat-lang' } })], artifacts: [artifactFor(12)] }, /belong to this repository/],
    [{ runs: [runFor(12, { workflow_id: 8 })], artifacts: [artifactFor(12)] }, /not the Runtime workflow/],
    [{ runs: [runFor(12)], artifacts: [] }, /exactly one kit-package-candidate/],
    [{ runs: [runFor(12)], artifacts: [artifactFor(12), artifactFor(12)] }, /exactly one/],
    [{ runs: [runFor(12)], artifacts: [artifactFor(12, { expired: true })] }, /has expired/],
    [{ runs: [runFor(12)], artifacts: [artifactFor(12, { workflow_run: { id: 12, head_sha: 'c'.repeat(40) } })] }, /does not belong/],
  ];
  for (const [state, expected] of cases) await assert.rejects(selectRuntimeCandidate({ api: github(state), repository, revision }), expected);
});

test('an already published version is refused before approval', async () => {
  await requireUnpublished({ version: '0.1.0-rc.11', fetch: async () => ({ status: 404 }) });
  await assert.rejects(requireUnpublished({ version: '0.1.0-rc.10', fetch: async () => ({ status: 200 }) }), /already on npm/);
  await assert.rejects(requireUnpublished({ version: '0.1.0-rc.11', fetch: async () => ({ status: 503 }) }), /HTTP 503/);
});

const bytes = Buffer.from('synthetic tarball');
const sha512 = createHash('sha512').update(bytes).digest('hex');
const integrity = `sha512-${createHash('sha512').update(bytes).digest('base64')}`;
const published = () => ({ name: 'caveat-lang', version: '0.1.0-rc.11', dist: { tarball: 'https://registry.npmjs.org/caveat-lang/-/caveat-lang-0.1.0-rc.11.tgz', integrity,
  attestations: { url: 'https://registry.npmjs.org/-/npm/v1/attestations/caveat-lang@0.1.0-rc.11', provenance: { predicateType: 'https://slsa.dev/provenance/v1' } } } });

test('registry metadata without a provenance attestation fails publication verification', () => {
  assert.equal(evaluateRegistryMetadata(published(), { version: '0.1.0-rc.11', sha512 }).predicateType, 'https://slsa.dev/provenance/v1');
  const cases = [
    [metadata => { delete metadata.dist.attestations; }, /has no dist\.attestations: it was not published with provenance/],
    [metadata => { metadata.dist.attestations.provenance = undefined; }, /no SLSA provenance/],
    [metadata => { metadata.dist.integrity = 'sha512-other'; }, /integrity differs/],
    [metadata => { metadata.dist.tarball = 'https://example.test/caveat-lang.tgz'; }, /public registry/],
    [metadata => { metadata.version = '0.1.0-rc.10'; }, /another version/],
  ];
  for (const [mutate, expected] of cases) {
    const metadata = published();
    mutate(metadata);
    assert.throws(() => evaluateRegistryMetadata(metadata, { version: '0.1.0-rc.11', sha512 }), expected);
  }
});

function bundle(change = statement => statement) {
  const statement = change({ subject: [{ name: 'pkg:npm/caveat-lang@0.1.0-rc.11', digest: { sha512 } }], predicate: {
    buildDefinition: { externalParameters: { workflow: { ref: 'refs/tags/v0.1.0-rc.11', repository: 'https://github.com/WSattazahn/caveat-lang', path: '.github/workflows/publish-npm.yml' } },
      resolvedDependencies: [{ uri: 'git+https://github.com/WSattazahn/caveat-lang@refs/tags/v0.1.0-rc.11', digest: { gitCommit: revision } }] },
    runDetails: { builder: { id: 'https://github.com/actions/runner/github-hosted' } } } });
  const payload = Buffer.from(JSON.stringify(statement)).toString('base64');
  return { attestations: [{ predicateType: 'https://github.com/npm/attestation/tree/main/specs/publish/v0.1', bundle: {} },
    { predicateType: 'https://slsa.dev/provenance/v1', bundle: { dsseEnvelope: { payload } } }] };
}

test('provenance must name these bytes, the publish workflow, the release tag and its revision', () => {
  const expected = { version: '0.1.0-rc.11', sha512, tag: 'v0.1.0-rc.11', revision };
  assert.equal(evaluateProvenance(bundle(), expected).revision, revision);
  const cases = [
    [statement => { statement.subject[0].digest.sha512 = '0'.repeat(128); return statement; }, /other bytes/],
    [statement => { statement.predicate.buildDefinition.externalParameters.workflow.repository = 'https://github.com/fork/caveat-lang'; return statement; }, /another repository/],
    [statement => { statement.predicate.buildDefinition.externalParameters.workflow.path = '.github/workflows/runtime.yml'; return statement; }, /another workflow/],
    [statement => { statement.predicate.buildDefinition.externalParameters.workflow.ref = 'refs/heads/main'; return statement; }, /not the release tag/],
    [statement => { statement.predicate.buildDefinition.resolvedDependencies[0].digest.gitCommit = 'c'.repeat(40); return statement; }, /another source revision/],
  ];
  for (const [change, message] of cases) assert.throws(() => evaluateProvenance(bundle(change), expected), message);
  assert.throws(() => evaluateProvenance({ attestations: [] }, expected), /one SLSA provenance/);
});

test('publication verification arguments are complete', () => {
  const args = ['--version', '0.1.0-rc.11', '--sha256', digest, '--revision', revision, '--dist-tag', 'latest'];
  assert.equal(parseArguments(args).distTag, 'latest');
  assert.throws(() => parseArguments(args.slice(0, 6)), /--dist-tag/);
  assert.throws(() => parseArguments([...args.slice(0, 7), 'beta']), /--dist-tag/);
  assert.throws(() => parseArguments(['--unknown', 'x']), /Expected/);
});
