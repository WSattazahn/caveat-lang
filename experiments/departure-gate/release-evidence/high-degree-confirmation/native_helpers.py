"""Frozen early-boxing capture and data-only arithmetic. Never builds runtime."""
from pathlib import Path
import argparse
import datetime
import hashlib
import json
import os
import platform
import statistics
import subprocess

BASE = '11c2a6a741609d3b640f27224ab6c9943b07f72c'
BASE_TREE = '89f3a5bde145a3071ce6e41652a866e6b722f3b1'
HARNESS = 'runtime/examples/collector_profile.rs'
HARNESS_SHA = '1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357'
PHASES = ['growth', 'growth_last_10_percent', 'steady', 'unrelated_changes', 'failed_release', 'release']
BUILD_ARGV = ['cargo', 'build', '--release', '--locked', '--manifest-path', 'runtime/Cargo.toml', '--example', 'collector_profile', '--no-default-features', '--features', 'collector-metrics']
CLEARED_ENV = ['RUSTFLAGS', 'RUSTDOCFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_BUILD_TARGET', 'CARGO_TARGET_DIR', 'RUSTUP_TOOLCHAIN', 'RUSTC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_BUILD_RUSTFLAGS', 'RUST_MIN_STACK']
PEAK_FIELD = 'max_dispatch_additional_heap_bytes'
HARNESS_SUFFIX = '\nevent profile_probe;\nevidence profile_side from "profile side observation"; renewable profile_side window 1;\nevent profile_mutate;\non profile_mutate renew profile_side;\non profile_mutate reveal profile_side supports seen;\n'
sha = lambda b: hashlib.sha256(b).hexdigest()
stamp = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
read = lambda p: json.loads(p.read_text(encoding='utf-8'))


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('x', encoding='utf-8', newline='\n') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def raw_save(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('xb') as stream:
        stream.write(data)


def copy(src, dst, root):
    raw_save(dst, src.read_bytes())
    return {'file': dst.relative_to(root).as_posix(), 'sha256': sha(dst.read_bytes())}


def check_record(root, record):
    path = (root / record['file']).resolve()
    assert path.is_relative_to(root), ('path outside frozen root', path)
    assert sha(path.read_bytes()) == record['sha256'], record


def command(repo, argv):
    return subprocess.run(argv, cwd=repo, capture_output=True, check=True).stdout.decode('utf-8').strip()


def git(repo, *args):
    return command(repo, ['git', '-c', 'safe.directory=' + str(repo), *args])


def selected_inputs(repo):
    files = set(git(repo, 'ls-files', '--', 'runtime', 'rust-toolchain.toml').splitlines())
    for name in ['.cargo/config', '.cargo/config.toml', 'runtime/.cargo/config', 'runtime/.cargo/config.toml']:
        if (repo / name).is_file():
            files.add(name)
    return [{'file': f, 'sha256': sha((repo / f).read_bytes())} for f in sorted(files)]


def source_check(repo, identity):
    assert git(repo, 'rev-parse', 'HEAD') == identity['revision']
    assert git(repo, 'rev-parse', 'HEAD:runtime') == identity['runtimeGitTree']
    assert not git(repo, 'status', '--porcelain', '--untracked-files=no')
    for item in identity['inputs']:
        assert sha((repo / item['file']).read_bytes()) == item['sha256'], item
    return {'at': stamp(), 'head': identity['revision'], 'runtime_tree': identity['runtimeGitTree'], 'inputs_match': True, 'tracked_clean': True}


def matrix():
    cells = [dict(name='release-mutual', cycles=3000, trial=i+1, primary=True) for i in range(8)]
    cells += [dict(name=name, cycles=n, trial=1, primary=False) for name, n in [('release-mutual', 60), ('release-mutual', 300), ('release-mutual', 1000), ('release-self', 3000), ('high-degree', 1000), ('reachable-chain', 300), ('reachable-chain', 1000), ('reachable-chain', 3000)]]
    for index, cell in enumerate(cells):
        cell['order'] = ['before', 'after'] if index % 2 == 0 else ['after', 'before']
        cell['key'] = f"{cell['name']}-{cell['cycles']}-t{cell['trial']}"
    return cells


def register(repo, out, reference, script):
    assert not out.exists(), 'Fresh output directory required; do not overwrite partial registration'
    assert git(repo, 'rev-parse', 'HEAD') == BASE
    assert git(repo, 'rev-parse', 'HEAD:runtime') == BASE_TREE
    assert not git(repo, 'status', '--porcelain', '--untracked-files=no')
    assert sha((repo / HARNESS).read_bytes()) == HARNESS_SHA
    prior = read(reference / 'contract.json')
    prior_registration = read(reference / 'registration.json')
    assert sha((reference / 'contract.json').read_bytes()) == prior_registration['contract_sha256']
    assert prior['revision'] == BASE and prior['runtime_tree'] == BASE_TREE
    fixtures = [dict(f) for f in prior['fixtures'] if f['name'] in {'release-mutual', 'release-self', 'high-degree', 'reachable-chain'}]
    assert len(fixtures) == 4
    for fixture in fixtures:
        assert sha((repo / fixture['source']).read_bytes()) == fixture['sha256']
        assert sha((reference / fixture['file']).read_bytes()) == fixture['sha256']
        fixture['augmented_sha256'] = sha(((repo / fixture['source']).read_bytes().decode('utf-8') + HARNESS_SUFFIX).encode('utf-8'))
    out.mkdir(parents=True)
    for fixture in fixtures:
        copy(repo / fixture['source'], out / fixture['file'], out)
    records = [copy(reference / name, out / 'historical-reference' / name, out) for name in ['contract.json', 'registration.json']]
    tools = [copy(script, out / 'frozen-profile.py', out), copy(script.with_name('PROTOCOL.md'), out / 'PROTOCOL.md', out), copy(script.with_name('build.py'), out / 'frozen-build.py', out), copy(repo / HARNESS, out / 'original-collector_profile.rs', out)]
    contract = {
        'schema': 'caveat-early-boxing-protocol/1', 'registered_at': stamp(), 'baseline': BASE, 'baseline_runtime_tree': BASE_TREE,
        'capture_root': str(out), 'repo_at_registration': str(repo), 'matrix': matrix(), 'fixtures': fixtures,
        'harness_sha256': HARNESS_SHA, 'driver_sha256': sha(script.read_bytes()), 'build_helper_sha256': sha(script.with_name('build.py').read_bytes()), 'allowed_changed_native_inputs': ['runtime/src/reactive_departure.rs'], 'tools': tools, 'historical_reference': records,
        'settings': {'drain_every': 1, 'probes': 100, 'timeout_seconds': 1800, 'cleared_environment': CLEARED_ENV, 'build_argv_prefix': BUILD_ARGV, 'os': platform.platform(), 'python': platform.python_version()},
        'criteria': {'reference_peaks': {'release': 27403251, 'failed_release': 27403464}, 'ceilings': {'release': 24662925, 'failed_release': 24663117}, 'minimum_reduction': 0.10, 'paired_median_latency_ratio_max': 1.05, 'primary_pairs': 8, 'all_primary_peaks_must_meet_ceiling': True, 'fresh_before_peaks_must_equal_reference': True, 'retained_heap_must_equal': True},
        'scope': 'Whole-apply additional requested heap and apply latency; exact finite save/archive/work/count projections. Failed work counters are placeholders. Supplemental timings descriptive. No silent retries or threshold adaptation.'}
    save(out / 'contract.json', contract)
    save(out / 'registration.json', {'at': stamp(), 'contract_sha256': sha((out / 'contract.json').read_bytes()), 'driver_sha256': contract['driver_sha256'], 'protocol_sha256': sha((out / 'PROTOCOL.md').read_bytes())})
    print(json.dumps({'registered': True, 'cells': 16, 'processes': 32, 'contract_sha256': sha((out / 'contract.json').read_bytes())}))


def load_contract(out, script):
    contract, registration = read(out / 'contract.json'), read(out / 'registration.json')
    assert sha((out / 'contract.json').read_bytes()) == registration['contract_sha256']
    assert sha(script.read_bytes()) == contract['driver_sha256'] == registration['driver_sha256']
    if script.with_name('build.py').exists():
        assert sha(script.with_name('build.py').read_bytes()) == contract['build_helper_sha256']
    assert sha((out / 'PROTOCOL.md').read_bytes()) == registration['protocol_sha256']
    for record in contract['tools'] + contract['historical_reference'] + contract['fixtures']:
        check_record(out, record)
    return contract, registration


def freeze(repo, out, contract, registration, kind, build_dir):
    assert not (out / 'identities' / (kind + '.json')).exists(), 'Identity already frozen'
    build = read(build_dir / 'build.json')
    assert build['kind'] == kind and build['status'] == 0 and build['error'] is None
    assert build['inputsStable'] and build['headAfter'] == build['revision'] and not build['trackedDiffAfter']
    assert build['argv'] == BUILD_ARGV + ['--target-dir', build['targetDir']]
    assert 'featureTree' in build and build['featureTree']
    assert set(build['cleared_environment']) == set(CLEARED_ENV)
    assert build['inputs'] == selected_inputs(repo)
    assert next(x['sha256'] for x in build['inputs'] if x['file'] == HARNESS) == HARNESS_SHA
    if kind == 'before':
        assert build['revision'] == BASE and build['runtimeGitTree'] == BASE_TREE
    else:
        assert build['revision'] != BASE and git(repo, 'merge-base', BASE, build['revision']) == BASE
        before = load_identity(out, 'before', registration)
        old = before['build']
        assert old['rustc'] == build['rustc'] and old['cargo'] == build['cargo']
        assert [f['file'] for f in old['inputs']] == [f['file'] for f in build['inputs']]
        changes = [new['file'] for original, new in zip(old['inputs'], build['inputs']) if original['sha256'] != new['sha256']]
        assert changes == contract['allowed_changed_native_inputs'], ('Changed native input scope mismatch', changes)
        assert old['targetDir'] != build['targetDir']
        left = json.dumps(old['featureTree'], sort_keys=True).replace(old['targetDir'].replace('\\', '\\\\'), '<TARGET>')
        right = json.dumps(build['featureTree'], sort_keys=True).replace(build['targetDir'].replace('\\', '\\\\'), '<TARGET>')
        assert left == right, 'Recorded feature trees differ'
    source = source_check(repo, build)
    records = []
    for channel in ['stdout', 'stderr']:
        assert sha((build_dir / ('build.' + channel)).read_bytes()) == build[channel + 'Sha256']
    assert sha((build_dir / (kind + '.exe')).read_bytes()) == build['executableSha256']
    for item in build['inputs']:
        assert sha((build_dir / 'source' / item['file']).read_bytes()) == item['sha256']
    for name in ['build.json', 'build.stdout', 'build.stderr']:
        records.append(copy(build_dir / name, out / 'build-identity' / kind / name, out))
    for item in build['inputs']:
        records.append(copy(build_dir / 'source' / item['file'], out / 'build-identity' / kind / 'source' / item['file'], out))
    artifact = copy(build_dir / (kind + '.exe'), out / (kind + '.exe'), out)
    identity = {'schema': 'caveat-early-boxing-identity/1', 'at': stamp(), 'kind': kind, 'revision': build['revision'], 'runtimeGitTree': build['runtimeGitTree'], 'contract_sha256': registration['contract_sha256'], 'build': build, 'source_check': source, 'artifact': artifact, 'records': records}
    save(out / 'identities' / (kind + '.json'), identity)
    save(out / 'identities' / (kind + '.pin.json'), {'identity_sha256': sha((out / 'identities' / (kind + '.json')).read_bytes()), 'contract_sha256': registration['contract_sha256']})
    print(json.dumps({'frozen': kind, 'revision': build['revision'], 'executable_sha256': artifact['sha256']}))


def load_identity(out, kind, registration):
    path = out / 'identities' / (kind + '.json')
    identity = read(path)
    pin = read(path.with_name(kind + '.pin.json'))
    assert sha(path.read_bytes()) == pin['identity_sha256']
    assert identity['contract_sha256'] == pin['contract_sha256'] == registration['contract_sha256']
    for record in [identity['artifact'], *identity['records']]:
        check_record(out, record)
    return identity


def fixture_for(contract, cell):
    return next(f for f in contract['fixtures'] if f['name'] == cell['name'])


def expected_argv(root, contract, cell, identity):
    fixture = fixture_for(contract, cell)
    return [str(Path(root) / identity['artifact']['file']), str(Path(root) / fixture['file']), str(cell['cycles']), fixture['mode'], '1', '100']


def validate_sample(data, cell, fixture):
    assert data['schema'] == 'caveat-collector-native-profile/1' and data['instrumented'] is True
    assert data['source_sha256'] == fixture['augmented_sha256'], 'Observed augmented source differs from frozen fixture/harness'
    assert data['cycles'] == cell['cycles'] and data['mode'] == fixture['mode'] and data['drain_every'] == 1
    expected = {'growth': cell['cycles'], 'growth_last_10_percent': cell['cycles'] - cell['cycles'] * 9 // 10, 'steady': 100, 'unrelated_changes': 100, 'failed_release': 3 if fixture['mode'] == 'release' else 0, 'release': 1 if fixture['mode'] == 'release' else 0}
    for phase in PHASES:
        sample = data[phase]
        assert sample['samples'] == expected[phase], (phase, sample['samples'], expected[phase])
        if sample['samples']:
            for field in ['median_ns', 'mean_ns', 'max_ns', 'p99_ns', PEAK_FIELD]:
                assert isinstance(sample[field], int) and sample[field] >= 0, (phase, field)
            assert sample['max_ns'] >= sample['p99_ns'] >= sample['median_ns']
            if phase == 'failed_release':
                assert all(value == 0 for value in sample['collector_work'].values()), 'Expected rejected-work placeholders'


def compare(a, b):
    differences = []
    def eq(path, x, y):
        if x != y:
            differences.append({'field': path, 'before': x, 'after': y})
    for field in ['schema', 'instrumented', 'source_sha256', 'cycles', 'mode', 'drain_every']:
        eq(field, a[field], b[field])
    for state in ['before_release', 'after_release']:
        for field in ['save_sha256', 'serialized_save_bytes', 'retired_dynamic_records', 'withdrawals', 'undrained', 'retained_rust_heap_bytes']:
            eq(state + '.' + field, a[state][field], b[state][field])
    for field in ['ndjson_sha256', 'ndjson_bytes', 'records', 'provenance_nodes', 'growth_ndjson_bytes', 'growth_records', 'growth_nodes', 'max_undrained_items', 'items_at_growth_end']:
        eq('archive.' + field, a['archive'][field], b['archive'][field])
    for phase in PHASES:
        eq(phase + '.samples', a[phase]['samples'], b[phase]['samples'])
        if a[phase]['samples'] and phase != 'failed_release':
            eq(phase + '.collector_work', a[phase]['collector_work'], b[phase]['collector_work'])
    return {'equal': not differences, 'differences': differences, 'scope': 'Finite save/ordered archive/count/successful work projections and exact retained requested heap; rejected work is placeholder.'}


def capture(repo, out, contract, registration, identities):
    assert str(out) == contract['capture_root'], 'Capture must use registered directory'
    ledger = out / 'attempts.jsonl'
    assert not ledger.exists(), 'No silent resume or restart'
    (out / 'capture').mkdir(exist_ok=False)
    raw_save(ledger, b'')
    env = dict(os.environ)
    environment = {key: env.pop(key, None) is not None for key in CLEARED_ENV}
    save(out / 'capture-environment.json', {'at': stamp(), 'cleared_overrides_present': environment})
    for cell in contract['matrix']:
        values = {}
        for kind in cell['order']:
            identity = identities[kind]
            fixture = fixture_for(contract, cell)
            check_record(out, identity['artifact'])
            check_record(out, fixture)
            source_before = source_check(repo, identities['after']['build'])
            key = cell['key'] + '-' + kind
            argv = expected_argv(out, contract, cell, identity)
            started = stamp()
            print('BEGIN', key, started, flush=True)
            with ledger.open('a', encoding='utf-8', newline='\n') as stream:
                stream.write(json.dumps({'key': key, 'started_at': started, 'argv': argv, 'contract_sha256': registration['contract_sha256']}) + '\n')
            try:
                result = subprocess.run(argv, cwd=repo, env=env, capture_output=True, timeout=contract['settings']['timeout_seconds'])
                stdout, stderr, status, error = result.stdout, result.stderr, result.returncode, None
            except subprocess.TimeoutExpired as exc:
                stdout, stderr, status, error = exc.stdout or b'', exc.stderr or b'', None, repr(exc)
            except OSError as exc:
                stdout, stderr, status, error = b'', b'', None, repr(exc)
            finished = stamp()
            streams = {}
            for channel, data in [('stdout', stdout), ('stderr', stderr)]:
                path = out / 'capture' / (key + '.' + channel)
                raw_save(path, data)
                streams[channel] = {'file': path.relative_to(out).as_posix(), 'sha256': sha(data)}
            row = {'key': key, 'cell': cell, 'kind': kind, 'revision': identity['revision'], 'runtime_tree': identity['runtimeGitTree'], 'contract_sha256': registration['contract_sha256'], 'executable_sha256': identity['artifact']['sha256'], 'argv': argv, 'cwd': str(repo), 'source_before': source_before, 'started_at': started, 'finished_at': finished, 'status': status, 'error': error, **streams}
            save(out / 'capture' / (key + '.json'), row)
            assert status == 0 and error is None, (key, status, error)
            save(out / 'capture' / (key + '-source-after.json'), source_check(repo, identities['after']['build']))
            check_record(out, identity['artifact'])
            values[kind] = json.loads(stdout)
            validate_sample(values[kind], cell, fixture)
            print('END', key, status, flush=True)
        comparison = compare(values['before'], values['after'])
        save(out / 'capture' / (cell['key'] + '-comparison.json'), comparison)
        assert comparison['equal'], comparison
        primary = {phase: {'before_peak': values['before'][phase][PEAK_FIELD], 'after_peak': values['after'][phase][PEAK_FIELD], 'latency_ratio': values['after'][phase]['median_ns'] / values['before'][phase]['median_ns']} for phase in ['release', 'failed_release'] if values['before'][phase]['samples']}
        print('PAIR', cell['key'], json.dumps(primary), flush=True)
    print(json.dumps({'capture_complete': True, 'processes': len(contract['matrix']) * 2, 'attempts_sha256': sha(ledger.read_bytes())}), flush=True)


def dist(values):
    return {'count': len(values), 'min': min(values), 'median': statistics.median(values), 'max': max(values), 'values': values}


def analyze(out, contract, registration, identities):
    attempts = [json.loads(line) for line in (out / 'attempts.jsonl').read_text(encoding='utf-8').splitlines()]
    expected = [cell['key'] + '-' + kind for cell in contract['matrix'] for kind in cell['order']]
    assert [row['key'] for row in attempts] == expected, 'Incomplete/unexpected attempt ledger; no complete result claimed'
    rows, all_records, index, previous_finish = [], [], 0, None
    for cell in contract['matrix']:
        values = {}
        for kind in cell['order']:
            identity = identities[kind]
            key = cell['key'] + '-' + kind
            row_path = out / 'capture' / (key + '.json')
            row = read(row_path)
            assert row['key'] == key and row['cell'] == cell and row['kind'] == kind
            assert row['status'] == 0 and row['error'] is None
            assert row['revision'] == identity['revision'] and row['runtime_tree'] == identity['runtimeGitTree']
            assert row['contract_sha256'] == registration['contract_sha256']
            assert row['executable_sha256'] == identity['artifact']['sha256']
            assert row['argv'] == expected_argv(contract['capture_root'], contract, cell, identity)
            attempt = attempts[index]
            assert attempt['argv'] == row['argv'] and attempt['started_at'] == row['started_at'] and attempt['contract_sha256'] == registration['contract_sha256']
            assert row['started_at'] <= row['finished_at'] and (previous_finish is None or previous_finish <= row['started_at'])
            previous_finish = row['finished_at']
            after_path = out / 'capture' / (key + '-source-after.json')
            after = read(after_path)
            for observation in [row['source_before'], after]:
                assert observation['head'] == identities['after']['revision'] and observation['runtime_tree'] == identities['after']['runtimeGitTree']
                assert observation['inputs_match'] and observation['tracked_clean']
            assert row['source_before']['at'] <= row['started_at'] <= row['finished_at'] <= after['at']
            for channel in ['stdout', 'stderr']:
                check_record(out, row[channel])
                all_records.append(row[channel])
            all_records += [{'file': str(path.relative_to(out).as_posix()), 'sha256': sha(path.read_bytes())} for path in [row_path, after_path]]
            values[kind] = read(out / row['stdout']['file'])
            validate_sample(values[kind], cell, fixture_for(contract, cell))
            index += 1
        comparison = compare(values['before'], values['after'])
        comparison_path = out / 'capture' / (cell['key'] + '-comparison.json')
        assert comparison == read(comparison_path) and comparison['equal']
        all_records.append({'file': comparison_path.relative_to(out).as_posix(), 'sha256': sha(comparison_path.read_bytes())})
        phases = {}
        for phase in PHASES:
            a, b = values['before'][phase], values['after'][phase]
            if a['samples'] == 0:
                phases[phase] = {'samples': 0}
                continue
            phases[phase] = {'samples': a['samples'], 'before': a, 'after': b, 'peak_delta_bytes': b[PEAK_FIELD] - a[PEAK_FIELD], 'peak_reduction_fraction': 1 - b[PEAK_FIELD] / a[PEAK_FIELD] if a[PEAK_FIELD] else None, 'latency_ratio': b['median_ns'] / a['median_ns'], 'latency_delta_ns': b['median_ns'] - a['median_ns']}
            if cell['primary'] and phase in contract['criteria']['ceilings']:
                phases[phase]['before_matches_reference'] = a[PEAK_FIELD] == contract['criteria']['reference_peaks'][phase]
                phases[phase]['after_meets_fixed_ceiling'] = b[PEAK_FIELD] <= contract['criteria']['ceilings'][phase]
        rows.append({'cell': cell, 'phases': phases, 'comparison': comparison, 'retained': {state: {'before': values['before'][state], 'after': values['after'][state]} for state in ['before_release', 'after_release']}, 'archive': {kind: values[kind]['archive'] for kind in ['before', 'after']}})
    primary = {}
    for phase in ['release', 'failed_release']:
        selected = [row['phases'][phase] for row in rows if row['cell']['primary']]
        ratios = [x['latency_ratio'] for x in selected]
        primary[phase] = {
            'before_peak_bytes': dist([x['before'][PEAK_FIELD] for x in selected]), 'after_peak_bytes': dist([x['after'][PEAK_FIELD] for x in selected]),
            'before_process_median_ns': dist([x['before']['median_ns'] for x in selected]), 'after_process_median_ns': dist([x['after']['median_ns'] for x in selected]),
            'paired_latency_ratios': dist(ratios), 'paired_latency_deltas_ns': dist([x['latency_delta_ns'] for x in selected]),
            'peak_reduction_fractions': dist([x['peak_reduction_fraction'] for x in selected]),
            'comparable': all(x['before_matches_reference'] for x in selected), 'all_peaks_meet_ceiling': all(x['after_meets_fixed_ceiling'] for x in selected),
            'latency_guard_pass': statistics.median(ratios) <= contract['criteria']['paired_median_latency_ratio_max'],
            'individual_pairs_over_latency_guard': sum(ratio > 1.05 for ratio in ratios)}
    outcome = {'comparable': all(x['comparable'] for x in primary.values()), 'memory_pass': all(x['all_peaks_meet_ceiling'] for x in primary.values()), 'latency_guard_pass': all(x['latency_guard_pass'] for x in primary.values()), 'native_projection_and_retention_equal': True}
    outcome['experiment_pass'] = all(outcome.values())
    result = {'schema': 'caveat-early-boxing-arithmetic/1', 'at': stamp(), 'contract_sha256': registration['contract_sha256'], 'attempts_sha256': sha((out / 'attempts.jsonl').read_bytes()), 'driver_sha256': contract['driver_sha256'], 'before_revision': identities['before']['revision'], 'after_revision': identities['after']['revision'], 'criteria': contract['criteria'], 'cells': 16, 'processes': 32, 'outcome': outcome, 'primary': primary, 'rows': rows, 'input_records': all_records, 'limitations': ['Native original harness emits phase summaries, not individual rejected-event samples.', 'Rejected collector-work values are placeholders.', 'Supplemental one-pair timings are descriptive; growth tail overlaps growth.', 'Requested heap is not RSS; collection-only and archive-origin bytes are not isolated.', 'No threshold/denominator adapts when baseline differs.']}
    prefix = 'analysis-' + stamp().replace(':', '-')
    save(out / (prefix + '.json'), result)
    raw_save(out / (prefix + '.md'), markdown(result).encode('utf-8'))
    print(json.dumps({'analysis': prefix + '.json', 'markdown': prefix + '.md', 'outcome': outcome, 'primary': primary}, indent=2))


def markdown(result):
    lines = ['# Early boxing: frozen comparison results', '', f"BEFORE `{result['before_revision']}`; AFTER `{result['after_revision']}`. All32 registered processes completed and raw streams/ledger verified. Native finite save/archive/work projections and retained heap match in all16 pairs.", '', '```json', json.dumps(result['outcome'], indent=2), '```', '', 'The fixed criteria were registered before source implementation. Any fresh baseline difference makes comparability unresolved; it does not change ceilings. Primary rejected time is each process\'s median of3 events, then8 paired ratios. Individual rejected times are not emitted by the unchanged harness.', '', '| Outcome | Before peak B min/median/max | After peak B min/median/max | Paired latency ratio min/median/max | Comparable | All peaks pass | Latency guard |', '|---|---:|---:|---:|---|---|---|']
    for phase, item in result['primary'].items():
        fmt = lambda d: f"{d['min']:,} / {d['median']:,} / {d['max']:,}"
        ratios = item['paired_latency_ratios']
        lines.append(f"| {phase} | {fmt(item['before_peak_bytes'])} | {fmt(item['after_peak_bytes'])} | {ratios['min']:.5f} / {ratios['median']:.5f} / {ratios['max']:.5f} | {item['comparable']} | {item['all_peaks_meet_ceiling']} | {item['latency_guard_pass']} |")
    lines += ['', '| Primary pair | Outcome | Before B | After B | Reduction | Before ms | After ms | Ratio | Ceiling pass |', '|---|---|---:|---:|---:|---:|---:|---:|---|']
    for row in result['rows']:
        if row['cell']['primary']:
            for phase in ['release', 'failed_release']:
                item = row['phases'][phase]
                lines.append(f"| {row['cell']['trial']} | {phase} | {item['before'][PEAK_FIELD]:,} | {item['after'][PEAK_FIELD]:,} | {item['peak_reduction_fraction']:.3%} | {item['before']['median_ns']/1e6:.5f} | {item['after']['median_ns']/1e6:.5f} | {item['latency_ratio']:.5f} | {item['after_meets_fixed_ceiling']} |")
    lines += ['', '## Every measured phase, including supplements', '', 'Absolute requested additional peaks and paired changes below. Positive byte deltas are increases, even if the bulk target passes. One-pair supplements and short-sample p99 are descriptive; raw stdout retains mean/max/p99 and all counters. Full growth and last10% overlap.', '', '| Cell | Phase | Before B | After B | Delta B | Before ms | After ms | Ratio |', '|---|---|---:|---:|---:|---:|---:|---:|']
    for row in result['rows']:
        for phase, item in row['phases'].items():
            if item['samples']:
                lines.append(f"| {row['cell']['key']} | {phase} | {item['before'][PEAK_FIELD]:,} | {item['after'][PEAK_FIELD]:,} | {item['peak_delta_bytes']:+,} | {item['before']['median_ns']/1e6:.5f} | {item['after']['median_ns']/1e6:.5f} | {item['latency_ratio']:.5f} |")
    lines += ['', '## Retention, save and archive', '', 'Both variants match every value in this table. Retained heap is requested session heap after harness report corrections; archive bytes are cumulative host-drained output, not additional dispatch peak.', '', '| Cell | Retained before/after B | Save before/after B | Archive B / records / provenance | Growth archive B |', '|---|---:|---:|---:|---:|']
    for row in result['rows']:
        a, b = row['retained']['before_release']['before'], row['retained']['after_release']['before']
        archive = row['archive']['before']
        lines.append(f"| {row['cell']['key']} | {a['retained_rust_heap_bytes']:,} / {b['retained_rust_heap_bytes']:,} | {a['serialized_save_bytes']:,} / {b['serialized_save_bytes']:,} | {archive['ndjson_bytes']:,} / {archive['records']:,} / {archive['provenance_nodes']:,} | {archive['growth_ndjson_bytes']:,} |")
    lines += ['', 'No individual rejected sample array, intermediate snapshot proof, allocator-internal usage or RSS is inferred. Failed-work counters are placeholders. Separate correctness gates establish their own wider finite coverage. Host/background variation remains possible; all attempts are preserved and no failed target is rerun or dropped.', '']
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=['register', 'freeze', 'run', 'analyze'])
    parser.add_argument('--repo', default='C:/Dev/caveat-lang')
    parser.add_argument('--out', required=True)
    parser.add_argument('--reference')
    parser.add_argument('--kind', choices=['before', 'after'])
    parser.add_argument('--build')
    args = parser.parse_args()
    repo, out, script = Path(args.repo).resolve(), Path(args.out).resolve(), Path(__file__).resolve()
    if args.action == 'register':
        assert args.reference, '--reference required'
        register(repo, out, Path(args.reference).resolve(), script)
        return
    contract, registration = load_contract(out, script)
    if args.action == 'freeze':
        assert args.kind and args.build, '--kind and --build required'
        freeze(repo, out, contract, registration, args.kind, Path(args.build).resolve())
        return
    identities = {kind: load_identity(out, kind, registration) for kind in ['before', 'after']}
    if args.action == 'run':
        capture(repo, out, contract, registration, identities)
    else:
        analyze(out, contract, registration, identities)


if __name__ == '__main__':
    main()
