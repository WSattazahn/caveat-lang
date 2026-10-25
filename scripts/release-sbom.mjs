// Writes the CycloneDX 1.6 software bill of materials that the GitHub release
// attaches beside the published npm tarball. The package has no npm
// dependencies; what it ships from others is the Rust crates compiled into its
// WebAssembly runtime. Those are the crates THIRD_PARTY_NOTICES.md lists (CI
// checks that file against the linked crate tree), each with the SHA-256
// checksum Cargo.lock records for its .crate file. Both files are read from
// the release's own revision with git, and the document's time is that
// commit's. Needs only Node, git and tar; no network. The Runtime workflow runs
// the same command on every tested candidate.
//
//   node scripts/release-sbom.mjs --tarball FILE --revision SHA --out FILE
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const repository = 'https://github.com/WSattazahn/caveat-lang';

// "## name version" headings, each followed by "Declared license: `EXPR`."
export function parseNotices(text) {
  const crates = [];
  const pattern = /^## (\S+) (\S+)\n\nDeclared license: `([^`]+)`\./gm;
  for (const match of text.matchAll(pattern)) crates.push({ name: match[1], version: match[2], license: match[3] });
  if (crates.length === 0) throw new Error('the notices list no crates');
  return crates;
}

// Registry checksums by "name version" from Cargo.lock's [[package]] tables.
export function parseLockChecksums(text) {
  const checksums = new Map();
  for (const table of text.split(/^\[\[package\]\]$/m).slice(1)) {
    const field = key => new RegExp(`^${key} = "([^"]*)"$`, 'm').exec(table)?.[1];
    const name = field('name');
    const version = field('version');
    const checksum = field('checksum');
    if (name && version && checksum) checksums.set(`${name} ${version}`, checksum);
  }
  return checksums;
}

// A stable urn:uuid derived from the tarball digest, so the same release
// always yields the same document.
function serialNumber(sha256) {
  const hex = createHash('sha256').update(`caveat-lang sbom ${sha256}`).digest('hex').slice(0, 32).split('');
  hex[12] = '5';
  hex[16] = ((parseInt(hex[16], 16) & 0x3) | 0x8).toString(16);
  const text = hex.join('');
  return `urn:uuid:${text.slice(0, 8)}-${text.slice(8, 12)}-${text.slice(12, 16)}-${text.slice(16, 20)}-${text.slice(20)}`;
}

export function buildSbom({ manifest, sha256, crates, checksums, revision, timestamp }) {
  if (manifest.name !== 'caveat-lang') throw new Error(`the tarball is ${manifest.name}, not caveat-lang`);
  if (!/^[0-9a-f]{64}$/.test(sha256)) throw new Error('sha256 must be 64 lowercase hex characters');
  if (!/^[0-9a-f]{40}$/.test(revision)) throw new Error('revision must be a complete Git SHA');
  if (Number.isNaN(Date.parse(timestamp))) throw new Error(`timestamp ${timestamp} is not a date`);
  for (const kind of ['dependencies', 'optionalDependencies', 'peerDependencies', 'bundleDependencies', 'bundledDependencies']) {
    if (manifest[kind] && Object.keys(manifest[kind]).length > 0) throw new Error(`the package declares ${kind}; list them in the SBOM first`);
  }
  const root = `pkg:npm/caveat-lang@${manifest.version}`;
  const components = crates.map(crate => {
    const checksum = checksums.get(`${crate.name} ${crate.version}`);
    if (!checksum) throw new Error(`${crate.name} ${crate.version} has no registry checksum in Cargo.lock`);
    const purl = `pkg:cargo/${crate.name}@${crate.version}`;
    return {
      type: 'library',
      'bom-ref': purl,
      name: crate.name,
      version: crate.version,
      scope: 'required',
      hashes: [{ alg: 'SHA-256', content: checksum }],
      licenses: [{ expression: crate.license, acknowledgement: 'declared' }],
      purl,
      externalReferences: [{ type: 'distribution', url: `https://crates.io/api/v1/crates/${crate.name}/${crate.version}/download` }],
    };
  });
  return {
    bomFormat: 'CycloneDX',
    specVersion: '1.6',
    serialNumber: serialNumber(sha256),
    version: 1,
    metadata: {
      timestamp: new Date(timestamp).toISOString(),
      tools: { components: [{ type: 'application', name: 'scripts/release-sbom.mjs', externalReferences: [{ type: 'vcs', url: `${repository}/blob/main/scripts/release-sbom.mjs` }] }] },
      component: {
        type: 'library',
        'bom-ref': root,
        name: 'caveat-lang',
        version: manifest.version,
        description: 'CAVEAT Language developer kit and WebAssembly runtime',
        hashes: [{ alg: 'SHA-256', content: sha256 }],
        licenses: [{ license: { id: manifest.license } }],
        purl: root,
        externalReferences: [
          { type: 'vcs', url: `${repository}/tree/${revision}` },
          { type: 'distribution', url: `https://registry.npmjs.org/caveat-lang/-/caveat-lang-${manifest.version}.tgz` },
        ],
      },
      properties: [{ name: 'caveat-lang:revision', value: revision }],
    },
    components,
    dependencies: [
      { ref: root, dependsOn: components.map(component => component['bom-ref']) },
      ...components.map(component => ({ ref: component['bom-ref'], dependsOn: [] })),
    ],
  };
}

// The fields a CycloneDX 1.6 consumer needs, and every reference resolving.
// The full schema check was run against the official bom-1.6 schema when this
// script was written; this keeps the document from drifting from it.
export function validateSbom(sbom) {
  const problems = [];
  const require = (condition, message) => { if (!condition) problems.push(message); };
  require(sbom.bomFormat === 'CycloneDX' && sbom.specVersion === '1.6' && sbom.version === 1, 'not a CycloneDX 1.6 document');
  require(/^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(sbom.serialNumber ?? ''), 'serialNumber is not a urn:uuid');
  const sha256 = hashes => Array.isArray(hashes) && hashes.length === 1 && hashes[0].alg === 'SHA-256' && /^[0-9a-f]{64}$/.test(hashes[0].content);
  const root = sbom.metadata?.component;
  require(root?.type === 'library' && root['bom-ref'] === root.purl && sha256(root.hashes), 'metadata.component lacks its purl or SHA-256');
  const refs = new Set([root?.['bom-ref']]);
  for (const component of sbom.components ?? []) {
    const label = `${component.name} ${component.version}`;
    require(component.type === 'library' && component['bom-ref'] === component.purl && /^pkg:cargo\/[^@]+@.+$/.test(component.purl ?? ''), `${label} lacks a cargo purl`);
    require(sha256(component.hashes), `${label} lacks a SHA-256`);
    require(component.licenses?.length === 1 && typeof component.licenses[0].expression === 'string', `${label} lacks a license expression`);
    require(!refs.has(component['bom-ref']), `${label} is listed twice`);
    refs.add(component['bom-ref']);
  }
  require((sbom.components ?? []).length > 0, 'no components');
  for (const dependency of sbom.dependencies ?? []) {
    for (const ref of [dependency.ref, ...dependency.dependsOn]) require(refs.has(ref), `dependency ${ref} names no component`);
  }
  require(sbom.dependencies?.[0]?.ref === root?.['bom-ref'] && sbom.dependencies[0].dependsOn.length === refs.size - 1, 'the package does not depend on every component');
  if (problems.length > 0) throw new Error(`invalid SBOM: ${problems.join('; ')}`);
  return sbom;
}

function parseArguments(argv) {
  const options = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    if (!['--tarball', '--revision', '--out'].includes(key) || argv[index + 1] === undefined) throw new Error(`unexpected argument ${key}`);
    options[key.slice(2)] = argv[index + 1];
  }
  for (const key of ['tarball', 'revision', 'out']) if (!options[key]) throw new Error(`--${key} is required`);
  return options;
}

export function main(argv, cwd = process.cwd()) {
  const options = parseArguments(argv);
  const git = args => execFileSync('git', args, { cwd, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
  const revision = git(['rev-parse', '--verify', `${options.revision}^{commit}`]).trim();
  const tarball = readFileSync(options.tarball);
  const manifest = JSON.parse(execFileSync('tar', ['-xzOf', options.tarball, 'package/package.json'], { encoding: 'utf8' }));
  const sbom = validateSbom(buildSbom({
    manifest,
    sha256: createHash('sha256').update(tarball).digest('hex'),
    crates: parseNotices(git(['show', `${revision}:THIRD_PARTY_NOTICES.md`]).replace(/\r\n/g, '\n')),
    checksums: parseLockChecksums(git(['show', `${revision}:runtime/Cargo.lock`]).replace(/\r\n/g, '\n')),
    revision,
    timestamp: git(['show', '-s', '--format=%cI', revision]).trim(),
  }));
  writeFileSync(options.out, `${JSON.stringify(sbom, null, 2)}\n`);
  console.log(`${options.out}: caveat-lang ${manifest.version} at ${revision} with ${sbom.components.length} compiled-in crates`);
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    console.error(`release-sbom: ${error.message}`);
    process.exit(1);
  }
}
