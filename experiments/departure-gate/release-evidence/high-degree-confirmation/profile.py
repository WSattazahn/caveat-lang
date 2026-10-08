"""Eight-pair high-degree confirmation; no compilation or source edits."""
from pathlib import Path, PurePosixPath, PureWindowsPath
import argparse
import hashlib
import importlib.util
import json
import os
import platform
import statistics
import subprocess

ROOT = Path(__file__).resolve().parent
HELPER_SHA = '1d654be0d56be0e33a38a1343d5fb10cdb8bde1cefeec0d5d0f68f2e2b6d8911'
helper_path = ROOT / 'native_helpers.py'
assert hashlib.sha256(helper_path.read_bytes()).hexdigest() == HELPER_SHA
spec = importlib.util.spec_from_file_location('frozen_native_helpers', helper_path)
h = importlib.util.module_from_spec(spec)
spec.loader.exec_module(h)
BEFORE = '11c2a6a741609d3b640f27224ab6c9943b07f72c'
AFTER = '590fae59aa4295ac5209f930a1e58ac152bd4603'
AFTER_TREE = 'a5d144827c0d3cfed859dc57b747bed91b33af04'
EXES = {'before': '2286a87f01560aed8f12ee4f5a6f8f5116f5ea79b4d3bc8f72e6016afae9a95f', 'after': '64125fda7f7323469f9d31f4ac5f787bf3b817856a38d25af2b653044be643d2'}
PRIOR_CONTRACT_SHA = 'cf21d21e7af51394d626f87232def44884520908aec05f2b697e8bef398253f4'
FIXTURE_SHA = 'f3fd8fe258e926dab76a7793187215915037f67bc95317ec50556b1344eace4f'
TRIGGERS = ['growth', 'release', 'failed_release']
HISTORICAL_CELL = 'high-degree-1000-t1'


def cells():
    return [{'key': f'high-degree-1000-t{i}', 'name': 'high-degree', 'cycles': 1000, 'trial': i, 'order': ['before', 'after'] if i % 2 else ['after', 'before']} for i in range(1, 9)]


def recorded_join(root, *parts):
    text = str(root)
    cls = PureWindowsPath if (len(text) > 1 and text[1] == ':') or text.startswith('\\\\') else PurePosixPath
    return str(cls(text).joinpath(*parts))


def load_prior(prior):
    contract, registration = h.read(prior / 'contract.json'), h.read(prior / 'registration.json')
    assert h.sha((prior / 'contract.json').read_bytes()) == registration['contract_sha256'] == PRIOR_CONTRACT_SHA
    assert contract['driver_sha256'] == HELPER_SHA
    for record in contract['tools'] + contract['fixtures']:
        h.check_record(prior, record)
    identities = {kind: h.load_identity(prior, kind, registration) for kind in ['before', 'after']}
    before, after = identities['before']['build'], identities['after']['build']
    assert before['revision'] == BEFORE and before['runtimeGitTree'] == h.BASE_TREE
    assert after['revision'] == AFTER and after['runtimeGitTree'] == AFTER_TREE
    assert before['rustc'] == after['rustc'] and before['cargo'] == after['cargo']
    assert before['featureTree'] == after['featureTree']
    assert [f['file'] for f in before['inputs']] == [f['file'] for f in after['inputs']]
    changes = [b['file'] for a, b in zip(before['inputs'], after['inputs']) if a['sha256'] != b['sha256']]
    assert changes == ['runtime/src/reactive_departure.rs']
    for kind, identity in identities.items():
        build = identity['build']
        assert identity['artifact']['sha256'] == build['executableSha256'] == EXES[kind]
        assert build['argv'] == h.BUILD_ARGV + ['--target-dir', build['targetDir']]
        assert build['status'] == 0 and build['error'] is None and build['inputsStable']
        assert build['headAfter'] == build['revision'] and not build['trackedDiffAfter']
        assert next(x['sha256'] for x in build['inputs'] if x['file'] == h.HARNESS) == h.HARNESS_SHA
    fixture = next(f for f in contract['fixtures'] if f['name'] == 'high-degree')
    assert fixture['sha256'] == FIXTURE_SHA and fixture['mode'] == 'release'
    expected_augmented = h.sha(((prior / fixture['file']).read_bytes().decode('utf-8') + h.HARNESS_SUFFIX).encode('utf-8'))
    assert fixture['augmented_sha256'] == expected_augmented
    historical = {}
    ledger = [json.loads(line) for line in (prior / 'attempts.jsonl').read_text(encoding='utf-8').splitlines()]
    old_cell = next(c for c in contract['matrix'] if c['key'] == HISTORICAL_CELL)
    for kind in ['before', 'after']:
        key = HISTORICAL_CELL + '-' + kind
        row = h.read(prior / 'capture' / (key + '.json'))
        assert row['status'] == 0 and row['error'] is None and row['kind'] == kind and row['cell'] == old_cell
        assert row['revision'] == identities[kind]['revision'] and row['executable_sha256'] == EXES[kind]
        assert row['contract_sha256'] == PRIOR_CONTRACT_SHA
        expected = [recorded_join(contract['capture_root'], identities[kind]['artifact']['file']), recorded_join(contract['capture_root'], fixture['file']), '1000', 'release', '1', '100']
        assert row['argv'] == expected
        attempt = next(x for x in ledger if x['key'] == key)
        assert attempt['argv'] == row['argv'] and attempt['started_at'] == row['started_at']
        for channel in ['stdout', 'stderr']:
            h.check_record(prior, row[channel])
        historical[kind] = h.read(prior / row['stdout']['file'])
        h.validate_sample(historical[kind], old_cell, fixture)
    comparison = h.compare(historical['before'], historical['after'])
    assert comparison['equal'] and comparison == h.read(prior / 'capture' / (HISTORICAL_CELL + '-comparison.json'))
    return contract, registration, identities, fixture, historical


def phase_arithmetic(rows, phase):
    a = [row['values']['before'][phase]['median_ns'] for row in rows]
    b = [row['values']['after'][phase]['median_ns'] for row in rows]
    ratios = [right / left for left, right in zip(a, b)]
    result = {'before_process_median_ns': h.dist(a), 'after_process_median_ns': h.dist(b), 'paired_latency_ratios': h.dist(ratios), 'paired_latency_deltas_ns': h.dist([right-left for left, right in zip(a, b)]), 'pairs_above_1': sum(r > 1 for r in ratios), 'pairs_above_1_05': sum(r > 1.05 for r in ratios), 'review_trigger': statistics.median(ratios) > 1.05 if phase in TRIGGERS else None}
    for kind in ['before', 'after']:
        result[kind + '_additional_peak_bytes'] = h.dist([row['values'][kind][phase][h.PEAK_FIELD] for row in rows])
    result['paired_peak_delta_bytes'] = h.dist([row['values']['after'][phase][h.PEAK_FIELD] - row['values']['before'][phase][h.PEAK_FIELD] for row in rows])
    result['order_groups'] = {}
    for first in ['before', 'after']:
        group = [ratio for row, ratio in zip(rows, ratios) if row['cell']['order'][0] == first]
        result['order_groups'][first + '_first'] = h.dist(group)
    return result


def controls(prior, out):
    assert not out.exists(), 'Fresh controls directory required'
    _, _, _, fixture, historical = load_prior(prior)
    checks = []
    def rejects(name, callback):
        try:
            callback()
        except AssertionError:
            checks.append(name)
        else:
            raise AssertionError('Negative control unexpectedly passed: ' + name)
    for kind, original in historical.items():
        h.validate_sample(original, cells()[0], fixture)
        checks.append(kind + '_real_stdout_shape_and_augmented_digest')
        changed = json.loads(json.dumps(original))
        changed['source_sha256'] = '0' * 64
        rejects(kind + '_wrong_augmented_digest_rejected', lambda: h.validate_sample(changed, cells()[0], fixture))
        changed = json.loads(json.dumps(original))
        changed['failed_release']['samples'] = 4
        rejects(kind + '_wrong_population_rejected', lambda: h.validate_sample(changed, cells()[0], fixture))
        for name, mutate in [('save', lambda x: x['after_release'].__setitem__('save_sha256', 'bad')), ('archive', lambda x: x['archive'].__setitem__('ndjson_sha256', 'bad')), ('retained', lambda x: x['before_release'].__setitem__('retained_rust_heap_bytes', 1)), ('work', lambda x: x['growth']['collector_work'].__setitem__('collected', -1))]:
            changed = json.loads(json.dumps(original))
            mutate(changed)
            assert not h.compare(original, changed)['equal'], name
            checks.append(kind + '_' + name + '_difference_detected')
    assert [cell['order'][0] for cell in cells()] == ['before', 'after'] * 4
    checks.append('exact_eight_alternating_pairs')
    assert recorded_join('C:\\capture\\run-001', 'prior', 'before.exe') == 'C:\\capture\\run-001\\prior\\before.exe'
    assert recorded_join('/capture/run-001', 'prior', 'before.exe') == '/capture/run-001/prior/before.exe'
    checks.append('recorded_windows_and_posix_argv_flavors')
    def synthetic(ratios):
        rows = []
        for cell, ratio in zip(cells(), ratios):
            row = {'cell': cell, 'values': {kind: json.loads(json.dumps(historical[kind])) for kind in ['before', 'after']}}
            for phase in TRIGGERS:
                row['values']['before'][phase]['median_ns'] = 1000000
                row['values']['after'][phase]['median_ns'] = int(ratio * 1000000)
            rows.append(row)
        return rows
    for phase in TRIGGERS:
        assert phase_arithmetic(synthetic([1.05] * 8), phase)['review_trigger'] is False
        assert phase_arithmetic(synthetic([1.0501] * 8), phase)['review_trigger'] is True
        assert phase_arithmetic(synthetic([2.0] + [1.0] * 7), phase)['review_trigger'] is False
        checks.append(phase + '_strict_threshold_and_no_outlier_exclusion')
    out.mkdir(parents=True)
    result = {'schema': 'caveat-high-degree-data-controls/1', 'at': h.stamp(), 'driver_sha256': h.sha(Path(__file__).read_bytes()), 'helper_sha256': HELPER_SHA, 'scope': 'Data-only preserved stdout and in-memory deliberate corruptions. No native execution, build or source edit.', 'checks': checks, 'passed': True}
    h.save(out / 'controls.json', result)
    print(json.dumps(result, indent=2))


def register(repo, prior, out, control_path):
    assert not out.exists(), 'Fresh registration directory required'
    old, old_reg, identities, fixture, _ = load_prior(prior)
    source = h.source_check(repo, identities['after']['build'])
    assert h.selected_inputs(repo) == identities['after']['build']['inputs']
    proof = h.read(control_path)
    assert proof['passed'] and proof['driver_sha256'] == h.sha(Path(__file__).read_bytes()) and proof['helper_sha256'] == HELPER_SHA
    out.mkdir(parents=True)
    records = {}
    def preserve(relative):
        if relative not in records:
            records[relative] = h.copy(prior / relative, out / 'prior' / relative, out)
    for name in ['contract.json', 'registration.json', 'attempts.jsonl']:
        preserve(name)
    for record in old['tools'] + old['fixtures']:
        preserve(record['file'])
    for kind, identity in identities.items():
        for name in ['identities/' + kind + '.json', 'identities/' + kind + '.pin.json']:
            preserve(name)
        for record in [identity['artifact'], *identity['records']]:
            preserve(record['file'])
        for suffix in ['.json', '.stdout', '.stderr', '-source-after.json']:
            preserve('capture/' + HISTORICAL_CELL + '-' + kind + suffix)
    preserve('capture/' + HISTORICAL_CELL + '-comparison.json')
    tools = [h.copy(ROOT / name, out / name, out) for name in ['profile.py', 'native_helpers.py', 'PROTOCOL.md']]
    controls_record = h.copy(control_path, out / 'data-controls.json', out)
    contract = {'schema': 'caveat-high-degree-confirmation-protocol/1', 'registered_at': h.stamp(), 'capture_root': str(out), 'prior_layout': 'prior', 'driver_sha256': h.sha(Path(__file__).read_bytes()), 'helper_sha256': HELPER_SHA, 'tools': tools, 'input_records': list(records.values()), 'data_controls': controls_record, 'source_at_registration': source, 'baseline_revision': BEFORE, 'candidate_revision': AFTER, 'candidate_runtime_tree': AFTER_TREE, 'exe_sha256': EXES, 'fixture': fixture, 'matrix': cells(), 'settings': {'cycles': 1000, 'mode': 'release', 'drain_every': 1, 'probes': 100, 'timeout_seconds': 1800, 'cleared_environment': h.CLEARED_ENV, 'os': platform.platform(), 'python': platform.python_version()}, 'criteria': {'review_phases': TRIGGERS, 'paired_median_ratio_strictly_above': 1.05, 'primary_pairs': 8, 'new_memory_target': None, 'historical_timings_in_new_aggregate': False, 'stop_on_latency_trigger': False}, 'scope': 'Separate high-degree confirmation only. Historical outputs provide semantic/retention references, never new latency samples. No change to completed memory experiment.'}
    h.save(out / 'contract.json', contract)
    h.save(out / 'registration.json', {'at': h.stamp(), 'contract_sha256': h.sha((out / 'contract.json').read_bytes()), 'driver_sha256': contract['driver_sha256'], 'protocol_sha256': h.sha((out / 'PROTOCOL.md').read_bytes())})
    print(json.dumps({'registered': True, 'pairs': 8, 'processes': 16, 'contract_sha256': h.sha((out / 'contract.json').read_bytes())}))


def load(out):
    contract, pin = h.read(out / 'contract.json'), h.read(out / 'registration.json')
    assert h.sha((out / 'contract.json').read_bytes()) == pin['contract_sha256']
    assert h.sha(Path(__file__).read_bytes()) == contract['driver_sha256'] == pin['driver_sha256']
    assert h.sha((out / 'PROTOCOL.md').read_bytes()) == pin['protocol_sha256']
    assert contract['matrix'] == cells() and contract['criteria']['review_phases'] == TRIGGERS
    for record in [*contract['tools'], *contract['input_records'], contract['data_controls']]:
        h.check_record(out, record)
    _, _, identities, fixture, historical = load_prior(out / contract['prior_layout'])
    assert fixture == contract['fixture']
    return contract, pin, identities, historical


def argv_for(root, contract, kind):
    return [recorded_join(root, contract['prior_layout'], kind + '.exe'), recorded_join(root, contract['prior_layout'], contract['fixture']['file']), '1000', 'release', '1', '100']


def capture(repo, out, contract, pin, identities, historical):
    assert str(out) == contract['capture_root'], 'Capture requires registered output path'
    ledger = out / 'attempts.jsonl'
    assert not ledger.exists(), 'No silent retry, restart or resume'
    (out / 'capture').mkdir(exist_ok=False)
    h.raw_save(ledger, b'')
    env = dict(os.environ)
    cleared = {key: env.pop(key, None) is not None for key in h.CLEARED_ENV}
    h.save(out / 'capture-environment.json', {'at': h.stamp(), 'cleared_overrides_present': cleared})
    for cell in contract['matrix']:
        values = {}
        for kind in cell['order']:
            identity = identities[kind]
            source_before = h.source_check(repo, identities['after']['build'])
            h.check_record(out / 'prior', identity['artifact'])
            h.check_record(out / 'prior', contract['fixture'])
            key, argv, started = cell['key'] + '-' + kind, argv_for(out, contract, kind), h.stamp()
            print('BEGIN', key, started, flush=True)
            with ledger.open('a', encoding='utf-8', newline='\n') as stream:
                stream.write(json.dumps({'key': key, 'argv': argv, 'started_at': started, 'contract_sha256': pin['contract_sha256']}) + '\n')
            try:
                result = subprocess.run(argv, cwd=repo, env=env, capture_output=True, timeout=contract['settings']['timeout_seconds'])
                stdout, stderr, status, error = result.stdout, result.stderr, result.returncode, None
            except subprocess.TimeoutExpired as exc:
                stdout, stderr, status, error = exc.stdout or b'', exc.stderr or b'', None, repr(exc)
            except OSError as exc:
                stdout, stderr, status, error = b'', b'', None, repr(exc)
            finished, streams = h.stamp(), {}
            for channel, data in [('stdout', stdout), ('stderr', stderr)]:
                path = out / 'capture' / (key + '.' + channel)
                h.raw_save(path, data)
                streams[channel] = {'file': path.relative_to(out).as_posix(), 'sha256': h.sha(data)}
            row = {'key': key, 'cell': cell, 'kind': kind, 'revision': identity['revision'], 'runtime_tree': identity['runtimeGitTree'], 'executable_sha256': EXES[kind], 'contract_sha256': pin['contract_sha256'], 'argv': argv, 'cwd': str(repo), 'source_before': source_before, 'started_at': started, 'finished_at': finished, 'status': status, 'error': error, **streams}
            h.save(out / 'capture' / (key + '.json'), row)
            assert status == 0 and error is None, (key, status, error)
            h.save(out / 'capture' / (key + '-source-after.json'), h.source_check(repo, identities['after']['build']))
            h.check_record(out / 'prior', identity['artifact'])
            values[kind] = json.loads(stdout)
            h.validate_sample(values[kind], cell, contract['fixture'])
            reference = h.compare(historical[kind], values[kind])
            h.save(out / 'capture' / (key + '-historical-comparison.json'), reference)
            assert reference['equal'], reference
            print('END', key, status, flush=True)
        comparison = h.compare(values['before'], values['after'])
        h.save(out / 'capture' / (cell['key'] + '-comparison.json'), comparison)
        assert comparison['equal'], comparison
        print('PAIR', cell['trial'], json.dumps({phase: {'before_ns': values['before'][phase]['median_ns'], 'after_ns': values['after'][phase]['median_ns'], 'ratio': values['after'][phase]['median_ns'] / values['before'][phase]['median_ns'], 'before_peak_B': values['before'][phase][h.PEAK_FIELD], 'after_peak_B': values['after'][phase][h.PEAK_FIELD]} for phase in TRIGGERS}), flush=True)
    print(json.dumps({'complete': True, 'processes': 16, 'attempts_sha256': h.sha(ledger.read_bytes())}), flush=True)


def analyze(out, contract, pin, identities, historical):
    ledger = out / 'attempts.jsonl'
    attempts = [json.loads(line) for line in ledger.read_text(encoding='utf-8').splitlines()]
    expected = [cell['key'] + '-' + kind for cell in cells() for kind in cell['order']]
    assert [x['key'] for x in attempts] == expected, 'Incomplete/unexpected ledger; no complete result claimed'
    rows, records, index, previous_end = [], [], 0, None
    for cell in cells():
        values = {}
        for kind in cell['order']:
            key = cell['key'] + '-' + kind
            meta_path = out / 'capture' / (key + '.json')
            row = h.read(meta_path)
            assert row['key'] == key and row['kind'] == kind and row['cell'] == cell
            assert row['status'] == 0 and row['error'] is None
            assert row['revision'] == identities[kind]['revision'] and row['runtime_tree'] == identities[kind]['runtimeGitTree']
            assert row['executable_sha256'] == EXES[kind] and row['contract_sha256'] == pin['contract_sha256']
            assert row['argv'] == argv_for(contract['capture_root'], contract, kind)
            assert attempts[index] == {field: row[field] for field in ['key', 'argv', 'started_at', 'contract_sha256']}
            assert row['started_at'] <= row['finished_at'] and (previous_end is None or previous_end <= row['started_at'])
            previous_end = row['finished_at']
            after_path = out / 'capture' / (key + '-source-after.json')
            after = h.read(after_path)
            for check in [row['source_before'], after]:
                assert check['head'] == AFTER and check['runtime_tree'] == AFTER_TREE and check['inputs_match'] and check['tracked_clean']
            assert row['source_before']['at'] <= row['started_at'] <= row['finished_at'] <= after['at']
            for channel in ['stdout', 'stderr']:
                h.check_record(out, row[channel])
                records.append(row[channel])
            values[kind] = h.read(out / row['stdout']['file'])
            h.validate_sample(values[kind], cell, contract['fixture'])
            historical_path = out / 'capture' / (key + '-historical-comparison.json')
            comparison = h.compare(historical[kind], values[kind])
            assert comparison['equal'] and comparison == h.read(historical_path)
            records += [{'file': path.relative_to(out).as_posix(), 'sha256': h.sha(path.read_bytes())} for path in [meta_path, after_path, historical_path]]
            index += 1
        path = out / 'capture' / (cell['key'] + '-comparison.json')
        comparison = h.compare(values['before'], values['after'])
        assert comparison['equal'] and comparison == h.read(path)
        records.append({'file': path.relative_to(out).as_posix(), 'sha256': h.sha(path.read_bytes())})
        rows.append({'cell': cell, 'values': values, 'comparison': comparison})
    phases = {phase: phase_arithmetic(rows, phase) for phase in h.PHASES}
    result = {'schema': 'caveat-high-degree-confirmation-arithmetic/1', 'at': h.stamp(), 'contract_sha256': pin['contract_sha256'], 'driver_sha256': contract['driver_sha256'], 'attempts_sha256': h.sha(ledger.read_bytes()), 'before_revision': BEFORE, 'after_revision': AFTER, 'pairs': 8, 'processes': 16, 'criteria': contract['criteria'], 'review_triggers': {phase: phases[phase]['review_trigger'] for phase in TRIGGERS}, 'native_projection_and_retention_equal': True, 'historical_native_projection_and_retention_equal': True, 'phases': phases, 'rows': rows, 'historical_reference': historical, 'records': records}
    prefix = 'analysis-' + h.stamp().replace(':', '-')
    h.save(out / (prefix + '.json'), result)
    h.raw_save(out / (prefix + '.md'), markdown(result).encode('utf-8'))
    print(json.dumps({'analysis': prefix + '.json', 'markdown': prefix + '.md', 'review_triggers': result['review_triggers'], 'phases': {phase: phases[phase] for phase in TRIGGERS}}, indent=2))


def markdown(result):
    lines = ['# High-degree confirmation results', '', f"Exact BEFORE `{BEFORE}` and AFTER `{AFTER}` artifacts; eight alternating pairs / sixteen fresh processes. All finite native projections and retained heaps match within pairs and their preserved same-variant historical references. Historical timings are excluded from every new aggregate.", '', 'A median paired ratio strictly above 1.05 is a phase-specific review trigger, not a retroactive memory-experiment failure. No reruns or threshold changes.', '', '| Phase | BEFORE process median ms, min / median / max | AFTER process median ms, min / median / max | Paired ratio min / median / max | Pairs >1.05 | Review trigger |', '|---|---:|---:|---:|---:|---|']
    for phase, item in result['phases'].items():
        ms = lambda v: ' / '.join(f"{v[k]/1e6:.5f}" for k in ['min', 'median', 'max'])
        ratio = item['paired_latency_ratios']
        lines.append(f"| {phase} | {ms(item['before_process_median_ns'])} | {ms(item['after_process_median_ns'])} | {ratio['min']:.5f} / {ratio['median']:.5f} / {ratio['max']:.5f} | {item['pairs_above_1_05']}/8 | {item['review_trigger'] if phase in TRIGGERS else 'Descriptive only'} |")
    lines += ['', '## Every pair and phase', '', 'Rejected times below are each process median of three attempts. The unchanged harness does not emit individual rejected samples. Full growth and last10% overlap; short-sample p99 and other phase summaries remain descriptive.', '', '| Pair | Order | Phase | BEFORE ms | AFTER ms | Ratio | BEFORE peak B | AFTER peak B | Delta B |', '|---|---|---|---:|---:|---:|---:|---:|---:|']
    for row in result['rows']:
        for phase in h.PHASES:
            a, b = row['values']['before'][phase], row['values']['after'][phase]
            lines.append(f"| {row['cell']['trial']} | {' then '.join(row['cell']['order'])} | {phase} | {a['median_ns']/1e6:.5f} | {b['median_ns']/1e6:.5f} | {b['median_ns']/a['median_ns']:.5f} | {a[h.PEAK_FIELD]:,} | {b[h.PEAK_FIELD]:,} | {b[h.PEAK_FIELD]-a[h.PEAK_FIELD]:+,} |")
    lines += ['', '## Order sensitivity and memory', '', '| Phase | BEFORE-first ratio median / range | AFTER-first ratio median / range | BEFORE peak range B | AFTER peak range B |', '|---|---:|---:|---:|---:|']
    for phase, item in result['phases'].items():
        a, b = item['order_groups']['before_first'], item['order_groups']['after_first']
        pa, pb = item['before_additional_peak_bytes'], item['after_additional_peak_bytes']
        lines.append(f"| {phase} | {a['median']:.5f} / {a['min']:.5f}–{a['max']:.5f} | {b['median']:.5f} / {b['min']:.5f}–{b['max']:.5f} | {pa['min']:,}–{pa['max']:,} | {pb['min']:,}–{pb['max']:,} |")
    lines += ['', 'Whole-apply additional requested heap includes live container capacity, transaction copies and archive staging. It excludes allocator internal storage, free lists, stack and OS reservation/RSS. No new byte target was selected and independent peaks are not summed.', '', '| Pair | Retained before / after release B, equal variants | Save bytes before / after, equal variants | Archive bytes / records / provenance, equal variants |', '|---|---:|---:|---:|']
    for row in result['rows']:
        data = row['values']['before']
        a, b, arc = data['before_release'], data['after_release'], data['archive']
        lines.append(f"| {row['cell']['trial']} | {a['retained_rust_heap_bytes']:,} / {b['retained_rust_heap_bytes']:,} | {a['serialized_save_bytes']:,} / {b['serialized_save_bytes']:,} | {arc['ndjson_bytes']:,} / {arc['records']:,} / {arc['provenance_nodes']:,} |")
    lines += ['', 'The JSON retains every original phase summary (median/mean/max/p99, work, peak), semantic/retention projection and historical reference observation. Failed-work counters remain placeholders. This finite repeated fixed-N16 workload is not an increasing-degree study, a stable-tail estimate or proof of causation. Background host variation remains possible. Any recommendation must consider all three triggers, pair/order distributions and absolute times without automatically authorizing an optimization.', '']
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=['controls', 'register', 'run', 'analyze'])
    parser.add_argument('--out', required=True)
    parser.add_argument('--repo', default='C:/Dev/caveat-lang')
    parser.add_argument('--prior')
    parser.add_argument('--controls')
    args = parser.parse_args()
    repo, out = Path(args.repo).resolve(), Path(args.out).resolve()
    if args.action == 'controls':
        assert args.prior, '--prior required'
        controls(Path(args.prior).resolve(), out)
    elif args.action == 'register':
        assert args.prior and args.controls, '--prior and --controls required'
        register(repo, Path(args.prior).resolve(), out, Path(args.controls).resolve())
    else:
        contract, pin, identities, historical = load(out)
        if args.action == 'run':
            capture(repo, out, contract, pin, identities, historical)
        else:
            analyze(out, contract, pin, identities, historical)


if __name__ == '__main__':
    main()
