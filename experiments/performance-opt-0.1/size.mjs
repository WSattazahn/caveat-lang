// Package and runtime size, base against branch: the WebAssembly and glue of
// both builds (dist/pkg-reactive, which the kit bundles, and dist/pkg), raw
// and gzipped, the kit's session library, and the kit tarball `npm pack`
// makes when the kit is staged as scripts/test-kit-package.mjs stages it.
//
//   node experiments/performance-opt-0.1/size.mjs --base=CHECKOUT,DIST --branch=CHECKOUT,DIST --out=size.json
//
// CHECKOUT is a caveat-lang checkout (its kit/, LICENSE, THIRD_PARTY_NOTICES.md
// and the files kit/pack-docs.json names are packed as they are on disk);
// DIST is the output of `npm run build` in that checkout.
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { gzipSync } from 'node:zlib';

const arg = (name) => process.argv.find((value) => value.startsWith(`--${name}=`))?.slice(name.length + 3);
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');

function npm(args, cwd) {
  const result = process.platform === 'win32'
    ? spawnSync(`npm ${args.map((value) => `"${value}"`).join(' ')}`, { cwd, encoding: 'utf8', shell: true })
    : spawnSync('npm', args, { cwd, encoding: 'utf8' });
  if (result.status !== 0) throw new Error(`npm ${args.join(' ')} failed:\n${result.stdout}\n${result.stderr}`);
  return result.stdout;
}

function git(args, cwd) {
  return spawnSync('git', args, { cwd, encoding: 'utf8' }).stdout.trim();
}

async function file(where) {
  const bytes = await readFile(where);
  return { bytes: bytes.length, gzipBytes: gzipSync(bytes, { level: 9 }).length, sha256: sha256(bytes) };
}

async function measure(label, spec) {
  const [checkout, dist] = spec.split(',').map((part) => path.resolve(part));
  const buildInfo = JSON.parse(await readFile(path.join(dist, 'build-info.json'), 'utf8'));
  const builds = {};
  for (const build of ['pkg-reactive', 'pkg']) {
    builds[build] = {
      wasm: await file(path.join(dist, build, 'caveat_runtime_bg.wasm')),
      glue: await file(path.join(dist, build, 'caveat_runtime.js')),
    };
  }
  // Stage the kit in a scratch directory and pack it there.
  const scratch = await mkdtemp(path.join(os.tmpdir(), `caveat-size-${label}-`));
  try {
    const kit = path.join(scratch, 'kit');
    // npm pack takes only what kit/package.json's "files" lists.
    await cp(path.join(checkout, 'kit'), kit, { recursive: true, filter: (source) => !source.split(path.sep).includes('node_modules') });
    await mkdir(path.join(kit, 'runtime'), { recursive: true });
    for (const name of ['caveat_runtime.js', 'caveat_runtime_bg.wasm']) await cp(path.join(dist, 'pkg-reactive', name), path.join(kit, 'runtime', name));
    await cp(path.join(dist, 'build-info.json'), path.join(kit, 'runtime', 'build-info.json'));
    for (const name of ['LICENSE', 'THIRD_PARTY_NOTICES.md']) await cp(path.join(checkout, name), path.join(kit, name));
    const packDocs = JSON.parse(await readFile(path.join(checkout, 'kit', 'pack-docs.json'), 'utf8'));
    for (const reference of packDocs.reference) {
      await mkdir(path.dirname(path.join(kit, 'docs', 'reference', reference)), { recursive: true });
      await cp(path.join(checkout, reference), path.join(kit, 'docs', 'reference', reference));
    }
    const out = path.join(scratch, 'out');
    await mkdir(out);
    const packed = JSON.parse(npm(['pack', '--json', '--pack-destination', out], kit))[0];
    const tarball = await readFile(path.join(out, packed.filename));
    return {
      label,
      checkout: { revision: git(['rev-parse', 'HEAD'], checkout), changedFiles: git(['status', '--porcelain'], checkout).split('\n').filter(Boolean).length },
      dist: { buildInfo },
      builds,
      sessionLibrary: await file(path.join(checkout, 'kit', 'lib', 'session.mjs')),
      tarball: { filename: packed.filename, bytes: tarball.length, sha256: sha256(tarball), unpackedBytes: packed.unpackedSize, entries: packed.entryCount },
    };
  } finally {
    await rm(scratch, { recursive: true, force: true });
  }
}

const base = await measure('base', arg('base'));
const branch = await measure('branch', arg('branch'));
const delta = (pick) => pick(branch) - pick(base);
const report = {
  schema: 'caveat-performance-opt-size/0.1',
  command: `node experiments/performance-opt-0.1/size.mjs ${process.argv.slice(2).join(' ')}`,
  base,
  branch,
  delta: {
    'pkg-reactive wasm bytes': delta((side) => side.builds['pkg-reactive'].wasm.bytes),
    'pkg-reactive wasm gzip bytes': delta((side) => side.builds['pkg-reactive'].wasm.gzipBytes),
    'pkg-reactive glue bytes': delta((side) => side.builds['pkg-reactive'].glue.bytes),
    'pkg wasm bytes': delta((side) => side.builds.pkg.wasm.bytes),
    'pkg wasm gzip bytes': delta((side) => side.builds.pkg.wasm.gzipBytes),
    'pkg glue bytes': delta((side) => side.builds.pkg.glue.bytes),
    'kit session.mjs bytes': delta((side) => side.sessionLibrary.bytes),
    'kit tarball bytes': delta((side) => side.tarball.bytes),
    'kit unpacked bytes': delta((side) => side.tarball.unpackedBytes),
  },
};
const text = `${JSON.stringify(report, null, 1)}\n`;
if (arg('out')) await writeFile(arg('out'), text);
else process.stdout.write(text);
