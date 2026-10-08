"""Offline arithmetic for the frozen 12-cell / 48-process event-phase capture.

Never imports captured scripts or invokes commands/native programs. Reads raw
stdout, metadata, ledger and artifacts; writes two new exclusive outputs only
after complete arithmetic succeeds. Historical accepted timing is never used.

python analyze_event_phases.py --capture RUN --json NEW_JSON --markdown NEW_MD
"""
from pathlib import Path
import argparse
from collections import Counter
import datetime
import hashlib
import json
import statistics


PHASES = ['outside', 'transaction_preparation', 'evaluation', 'collection',
          'compaction', 'archive_construction_and_removal', 'binding_evaluation',
          'final_settlement', 'commit_cleanup', 'rollback_cleanup', 'apply_wrapper']
GROUPS = ['growth', 'growth_last_10_percent', 'steady', 'unrelated_changes',
          'failed_release', 'release']
FLOWS = ['alloc_calls', 'alloc_zeroed_calls', 'realloc_calls', 'dealloc_calls',
         'allocation_failures', 'realloc_failures', 'requested_allocated_bytes',
         'requested_freed_bytes']
ORIGINS = PHASES + ['prior_event']
STATE = ['save_sha256', 'serialized_save_bytes', 'retired_dynamic_records',
         'withdrawals', 'undrained']
ARCHIVE = ['ndjson_sha256', 'ndjson_bytes', 'records', 'provenance_nodes',
           'growth_ndjson_bytes', 'growth_records', 'growth_nodes',
           'max_undrained_items', 'items_at_growth_end']
ALLOCATION_SCALARS = ['event_start_requested_live_bytes', 'event_start_system_requested_live_bytes',
    'requested_live_at_peak_bytes', 'requested_additional_peak_bytes',
    'metadata_bytes_at_requested_peak', 'alignment_padding_bytes_at_requested_peak',
    'system_requested_bytes_at_requested_peak', 'system_requested_peak_bytes',
    'system_requested_additional_peak_bytes', 'metadata_bytes_at_system_peak',
    'alignment_padding_bytes_at_system_peak', 'requested_live_at_end_bytes',
    'system_requested_live_at_end_bytes']


def need(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def ratio(a, b):
    return a / b if b else None


def median(values):
    values = [v for v in values if v is not None]
    return statistics.median(values) if values else None


def distribution(values):
    values = [v for v in values if v is not None]
    return dict(count=len(values), min=min(values) if values else None,
                median=median(values), max=max(values) if values else None)


def projection(data):
    return {**{k: data[k] for k in ['source_sha256', 'cycles', 'mode', 'drain_every']},
            **{state: {k: data[state][k] for k in STATE} for state in ['before_release', 'after_release']},
            'archive': {k: data['archive'][k] for k in ARCHIVE},
            'phases': {group: {'samples': data[group]['samples'],
                **({'work': data[group]['collector_work']} if group != 'failed_release' and data[group]['samples'] else {})}
                for group in GROUPS}}


def events(data, group):
    if group == 'growth_last_10_percent':
        return data['event_samples']['growth'][data['cycles'] * 9 // 10:]
    return data['event_samples'][group]


def verify_summary(data, group):
    samples = events(data, group)
    report = data[group]
    need(report['samples'] == len(samples), f'{group}: raw count')
    if not samples:
        return
    times = sorted(s['ns'] for s in samples)
    n = len(times)
    expected = dict(median_ns=times[n // 2], p99_ns=times[(n * 99 + 99) // 100 - 1],
                    max_ns=times[-1], mean_ns=sum(times) // n,
                    max_dispatch_additional_heap_bytes=max(s['peak_additional_bytes'] for s in samples))
    for field, value in expected.items():
        need(report[field] == value, f'{group}: native {field} differs from raw')


def validate_sample(sample, kind):
    p = sample['phases']
    need(len(p['phase_ns']) == len(PHASES) == len(p['phase_entries']), 'phase lengths')
    need(sum(p['phase_entries']) == p['transitions'], 'entry/transition total')
    need(p['phase_ns'][0] == 0, 'outside timed')
    need(sample['phase_total_ns'] == sum(p['phase_ns']), 'phase sum')
    if kind == 'timing':
        need(p['clock_enabled'] and sample['allocations'] is None, 'timing mode flags')
        need(sample['phase_total_ns'] + sample['remainder_ns'] == sample['ns'], 'exclusive time plus residual')
        return
    need(not p['clock_enabled'] and not any(p['phase_ns']) and sample['remainder_ns'] is None, 'origins clocks enabled')
    a = sample['allocations']
    need(len(a['requested_live_by_origin_at_peak']) == 12, 'origin bucket count')
    need(sum(a['requested_live_by_origin_at_peak']) == a['requested_live_at_peak_bytes'], 'requested peak partition')
    need(sum(a['requested_live_by_origin_at_end']) == a['requested_live_at_end_bytes'], 'requested end partition')
    need(a['requested_live_at_peak_bytes'] - a['event_start_requested_live_bytes'] == sample['peak_additional_bytes'] == a['requested_additional_peak_bytes'], 'additional requested peak')
    need(a['system_requested_peak_bytes'] - a['event_start_system_requested_live_bytes'] == a['system_requested_additional_peak_bytes'], 'additional System peak')
    need(a['requested_live_at_peak_bytes'] + a['metadata_bytes_at_requested_peak'] + a['alignment_padding_bytes_at_requested_peak'] == a['system_requested_bytes_at_requested_peak'], 'System at requested peak partition')
    need(sum(a['requested_live_by_origin_at_system_peak']) + a['metadata_bytes_at_system_peak'] + a['alignment_padding_bytes_at_system_peak'] == a['system_requested_peak_bytes'], 'System peak partition')
    net = sum(f['requested_allocated_bytes'] - f['requested_freed_bytes'] for f in a['flow_by_operation_phase'])
    need(a['event_start_requested_live_bytes'] + net == a['requested_live_at_end_bytes'], 'requested flow conservation')


def timing_statistics(samples):
    return {
        'samples': len(samples),
        'apply_ns': distribution(s['ns'] for s in samples),
        'phase_total_ns': distribution(s['phase_total_ns'] for s in samples),
        'residual_ns': distribution(s['remainder_ns'] for s in samples),
        'residual_share': distribution(ratio(s['remainder_ns'], s['ns']) for s in samples),
        'transitions': distribution(s['phases']['transitions'] for s in samples),
        'phases': {name: {
            'ns': distribution(s['phases']['phase_ns'][i] for s in samples),
            'share_of_apply': distribution(ratio(s['phases']['phase_ns'][i], s['ns']) for s in samples),
            'entries': distribution(s['phases']['phase_entries'][i] for s in samples),
            'total_recorded_ns': sum(s['phases']['phase_ns'][i] for s in samples),
        } for i, name in enumerate(PHASES)},
        'raw_release_samples': samples if len(samples) <= 3 else None,
    }


def location(index):
    return 'event_start_baseline' if index is None else PHASES[index]


def origin_statistics(samples):
    allocations = [s['allocations'] for s in samples]
    if not samples:
        return {'samples': 0}
    maximum_index = max(range(len(samples)), key=lambda i: allocations[i]['requested_live_at_peak_bytes'])
    return {
        'samples': len(samples),
        'apply_ns': distribution(s['ns'] for s in samples),
        'scalars': {f: distribution(a[f] for a in allocations) for f in ALLOCATION_SCALARS},
        'net_requested_end_minus_start_bytes': distribution(a['requested_live_at_end_bytes'] - a['event_start_requested_live_bytes'] for a in allocations),
        'requested_peak_locations': dict(Counter(location(a['requested_peak_phase']) for a in allocations)),
        'system_peak_locations': dict(Counter(location(a['system_requested_peak_phase']) for a in allocations)),
        'origins': {name: {
            'live_bytes_at_requested_peak': distribution(a['requested_live_by_origin_at_peak'][i] for a in allocations),
            'live_count_at_requested_peak': distribution(a['live_allocations_by_origin_at_peak'][i] for a in allocations),
            'share_of_global_requested_peak': distribution(ratio(a['requested_live_by_origin_at_peak'][i], a['requested_live_at_peak_bytes']) for a in allocations),
            'live_bytes_at_system_peak': distribution(a['requested_live_by_origin_at_system_peak'][i] for a in allocations),
            'live_bytes_at_end': distribution(a['requested_live_by_origin_at_end'][i] for a in allocations),
        } for i, name in enumerate(ORIGINS)},
        'flows_by_operation_phase': {name: {f: {
            'per_event': distribution(a['flow_by_operation_phase'][i][f] for a in allocations),
            'total_over_events': sum(a['flow_by_operation_phase'][i][f] for a in allocations),
        } for f in FLOWS} for i, name in enumerate(PHASES)},
        'raw_release_samples': samples if len(samples) <= 3 else None,
        'event_with_largest_absolute_requested_peak': {
            'index_within_group': maximum_index, 'sample': samples[maximum_index],
        },
    }


def aggregate_primary(rows, group):
    values = [r['groups'][group] for r in rows]
    timing_events = [s for v in values for s in v['timing']['raw_release_samples']]
    return {
        'processes': len(values),
        'plain_native_median_ns': distribution(v['controls']['plain_native_median_ns'] for v in values),
        'timing_native_median_ns': distribution(v['controls']['timing_native_median_ns'] for v in values),
        'origins_native_median_ns': distribution(v['controls']['origins_native_median_ns'] for v in values),
        'timing_to_plain_ratio': distribution(v['controls']['timing_to_plain_ratio'] for v in values),
        'origins_to_plain_ratio': distribution(v['controls']['origins_to_plain_ratio'] for v in values),
        'individual_timing_apply_range_ns': {'min': min(s['ns'] for s in timing_events), 'max': max(s['ns'] for s in timing_events)},
        'residual_process_median_ns': distribution(v['timing']['residual_ns']['median'] for v in values),
        'residual_process_median_share': distribution(v['timing']['residual_share']['median'] for v in values),
        'phases': {name: {
            'process_median_ns': distribution(v['timing']['phases'][name]['ns']['median'] for v in values),
            'process_median_event_share': distribution(v['timing']['phases'][name]['share_of_apply']['median'] for v in values),
            'individual_ns_range': {'min': min(s['phases']['phase_ns'][i] for s in timing_events), 'max': max(s['phases']['phase_ns'][i] for s in timing_events)},
            'individual_event_share_range': {'min': min(ratio(s['phases']['phase_ns'][i], s['ns']) for s in timing_events), 'max': max(ratio(s['phases']['phase_ns'][i], s['ns']) for s in timing_events)},
        } for i, name in enumerate(PHASES)},
        'origins_scalars_process_medians': {f: distribution(v['origins']['scalars'][f]['median'] for v in values) for f in ALLOCATION_SCALARS},
        'origin_bytes_at_peak_process_medians': {name: distribution(v['origins']['origins'][name]['live_bytes_at_requested_peak']['median'] for v in values) for name in ORIGINS},
        'origin_flow_process_medians': {name: {f: distribution(v['origins']['flows_by_operation_phase'][name][f]['per_event']['median'] for v in values) for f in FLOWS} for name in PHASES},
        'peak_locations_all_events': dict(sum((Counter(v['origins']['requested_peak_locations']) for v in values), Counter())),
        'peak_comparison_per_process': [dict(key=r['key'], **r['groups'][group]['peaks']) for r in rows],
    }


def markdown(result):
    out = []
    def line(text=''): out.append(text)
    def table(headers, rows):
        line('| ' + ' | '.join(headers) + ' |')
        line('| ' + ' | '.join(['---'] * len(headers)) + ' |')
        for row in rows: line('| ' + ' | '.join(map(str, row)) + ' |')
        line()
    def ms(value): return 'n/a' if value is None else f'{value / 1e6:.5f}'
    def percent(value): return 'n/a' if value is None else f'{value * 100:.3f}%'
    def count(value): return f'{value:,.0f}'
    line('# Event-phase profiling: raw arithmetic findings')
    line()
    line(f"Revision `{result['revision']}`; runtime tree `{result['runtime_tree']}`. All 12 cells / 48 process observations were read from preserved raw stdout, with artifact/stream/ledger hashes checked. The 12 accepted-reference processes provide semantic equality only; none of their timing or memory values enters the following comparisons.")
    line()
    line('P = fresh ordinary counting-allocator control, T = fixed-boundary timing mode, O = allocation-origin mode with phase clocks disabled. O includes header/TLS/counter/peak-copy overhead and is not an alternative timing estimate. Each primary rejection value first reduces three samples within its process; distributions then retain four process values. Individual ranges are reported without treating those twelve events as independent processes.')
    line()
    line('## Primary process values')
    line()
    entries = []
    for row in result['rows']:
        if not row['primary']: continue
        for group in ['release', 'failed_release']:
            v = row['groups'][group]; c = v['controls']
            entries.append([row['trial'], group, ms(c['plain_native_median_ns']), ms(c['timing_native_median_ns']), ms(c['origins_native_median_ns']), f"{c['timing_to_plain_ratio']:.5f}", f"{c['origins_to_plain_ratio']:.5f}", ms(v['timing']['residual_ns']['median'])])
    table(['Pair', 'Outcome', 'P apply ms', 'T apply ms', 'O apply ms', 'T/P', 'O/P', 'T residual ms'], entries)
    for group, a in result['primary'].items():
        line(f"**{group}** process-median distributions (min / median / max):")
        line()
        table(['Measure', 'Min', 'Median', 'Max'], [[f, *[(percent if 'share' in f else ms if f.endswith('_ns') else lambda v: f'{v:.5f}')(a[f][k]) for k in ['min', 'median', 'max']]]
            for f in ['plain_native_median_ns', 'timing_native_median_ns', 'origins_native_median_ns', 'timing_to_plain_ratio', 'origins_to_plain_ratio', 'residual_process_median_ns', 'residual_process_median_share']])
        line(f"Individual T apply range: {ms(a['individual_timing_apply_range_ns']['min'])}–{ms(a['individual_timing_apply_range_ns']['max'])} ms. Peak locations across O events: `{json.dumps(a['peak_locations_all_events'], sort_keys=True)}`.")
        line()
        table(['Exclusive phase', 'Process median ms, min/median/max', 'Per-event share, process min/median/max'], [[name, ' / '.join(ms(v['process_median_ns'][k]) for k in ['min', 'median', 'max']), ' / '.join(percent(v['process_median_event_share'][k]) for k in ['min', 'median', 'max'])] for name, v in a['phases'].items()])
    line('Phase shares are calculated from each event before taking medians. Independent phase medians need not add to median total apply; only each raw event reconciles exclusive phase sum plus residual. The raw JSON contains every primary individual T/O successful and rejected sample; ordinary controls emit phase summaries only.')
    line()
    line('## Global requested peak and live origins')
    line()
    line('Each row below is one actual O event peak partition, not a sum of phase maxima or independently aggregated origin medians. Preevent storage includes the live harness/sample buffers and, during rejection, the saved rollback-reference string. Current-event origin is the last successful allocation/reallocation phase; realloc counts a full logical replacement, not physical copying.')
    line()
    peak_rows = []
    for row in result['rows']:
        if not row['primary']: continue
        for group in ['release', 'failed_release']:
            for i, sample in enumerate(row['groups'][group]['origins']['raw_release_samples']):
                a = sample['allocations']
                origins = '; '.join(f'{name}={count(n)}' for name, n in zip(ORIGINS, a['requested_live_by_origin_at_peak']) if n)
                peak_rows.append([f"{row['trial']}/{group}/{i+1}", location(a['requested_peak_phase']), count(a['event_start_requested_live_bytes']), count(a['requested_live_at_peak_bytes']), count(a['requested_additional_peak_bytes']), count(a['requested_live_at_end_bytes']-a['event_start_requested_live_bytes']), origins])
    table(['Process/event', 'Peak location', 'Start B', 'Absolute peak B', 'Additional peak B', 'Net end−start B', 'Live origin partition at that peak, B'], peak_rows)
    table(['Process/event', 'Metadata at requested peak B', 'Padding at requested peak B', 'System request at requested peak B', 'Separate System peak B / location'], [[f"{row['trial']}/{group}/{i+1}", count(s['allocations']['metadata_bytes_at_requested_peak']), count(s['allocations']['alignment_padding_bytes_at_requested_peak']), count(s['allocations']['system_requested_bytes_at_requested_peak']), f"{count(s['allocations']['system_requested_peak_bytes'])} / {location(s['allocations']['system_requested_peak_phase'])}"] for row in result['rows'] if row['primary'] for group in ['release', 'failed_release'] for i,s in enumerate(row['groups'][group]['origins']['raw_release_samples'])])
    for group, a in result['primary'].items():
        line(f"**{group}:** across-process medians of per-process requested traffic/calls. These cumulative logical flows are different from live-at-peak origins; per-phase median rows must not be summed as a median event total.")
        line()
        table(['Operation phase', 'Requested inflow B', 'Requested freed B', 'alloc / zeroed / realloc / dealloc calls'], [[name, count(v['requested_allocated_bytes']['median']), count(v['requested_freed_bytes']['median']), ' / '.join(count(v[f]['median']) for f in ['alloc_calls','alloc_zeroed_calls','realloc_calls','dealloc_calls'])] for name,v in a['origin_flow_process_medians'].items()])
    line('## Growth and supplemental controls')
    line()
    line('Single-triple supplemental timing is descriptive. The growth tail is the final 10% of the same growth samples, not a separate/additional workload. Tail origin rows use the single event with the largest absolute requested peak within that tail; they are observed partitions, not phase-peak sums.')
    line()
    tail_rows=[]
    for row in result['rows']:
        v=row['groups']['growth_last_10_percent']; c=v['controls']; best=v['origins']['event_with_largest_absolute_requested_peak']['sample']['allocations']
        dominant=sorted(((n,s['share_of_apply']['median']) for n,s in v['timing']['phases'].items()),key=lambda p:p[1],reverse=True)[:3]
        tail_rows.append([row['key'], ms(c['plain_native_median_ns']),ms(c['timing_native_median_ns']),ms(c['origins_native_median_ns']),'; '.join(n+' '+percent(s) for n,s in dominant),count(best['requested_live_at_peak_bytes']),'; '.join(n+'='+count(v) for n,v in zip(ORIGINS,best['requested_live_by_origin_at_peak']) if v)])
    table(['Case', 'Tail P ms', 'Tail T ms', 'Tail O ms', 'Three largest median event shares', 'Selected absolute O peak B', 'Selected O peak origins B'],tail_rows)
    table(['Supplement', 'Outcome', 'P/T/O apply medians ms', 'T/P; O/P', 'Largest median timing phase / share'],[[row['key'],group,' / '.join(ms(row['groups'][group]['controls'][v+'_native_median_ns']) for v in ['plain','timing','origins']),f"{row['groups'][group]['controls']['timing_to_plain_ratio']:.5f}; {row['groups'][group]['controls']['origins_to_plain_ratio']:.5f}",max(row['groups'][group]['timing']['phases'],key=lambda n:row['groups'][group]['timing']['phases'][n]['share_of_apply']['median'])+' / '+percent(max(v['share_of_apply']['median'] for v in row['groups'][group]['timing']['phases'].values()))]for row in result['rows'] if not row['primary'] for group in ['release','failed_release'] if row['groups'][group]['samples']])
    line('## Peak, retained heap, save and archive comparisons')
    line()
    table(['Case', 'T/O sample-buffer requested B', 'T/O pre-session harness baseline B'], [[row['key'], ' / '.join(count(row['harness'][mode]['sample_buffer_requested_bytes']) for mode in ['timing','origins']), ' / '.join(count(row['harness'][mode]['harness_baseline_requested_bytes']) for mode in ['timing','origins'])] for row in result['rows']])
    table(['Case/group','P requested peak B','T−P B','O−P B'],[[row['key']+'/'+group,count(v['peaks']['plain']),count(v['peaks']['timing_delta']),count(v['peaks']['origins_delta'])]for row in result['rows'] for group,v in row['groups'].items() if v['samples']])
    table(['Case','P retained before/after B','T−P retained before/after B','O−P retained before/after B','Save before/after B','Archive growth/total B'],[[row['key'],' / '.join(count(row['retained'][s]['plain'])for s in ['before_release','after_release']),' / '.join(count(row['retained'][s]['timing_delta'])for s in ['before_release','after_release']),' / '.join(count(row['retained'][s]['origins_delta'])for s in ['before_release','after_release']),' / '.join(count(row['semantics'][s]['serialized_save_bytes'])for s in ['before_release','after_release']),count(row['semantics']['archive']['growth_ndjson_bytes'])+' / '+count(row['semantics']['archive']['ndjson_bytes'])]for row in result['rows']])
    line('Requested heap includes requested container capacity but excludes allocator metadata/free lists, stack and OS reservations/RSS. Sample buffers are excluded from retained-session baselines but stay real live bytes in absolute peak origin partitions. Emitted archive bytes are serialization volume, not a persistent buffer in this draining harness. Unchanged required-chain retention must remain separate from transient allocations and removal work.')
    line()
    line('## Candidate source attribution, pending review')
    line()
    for group,candidates in result['candidate_phase_ranking'].items():
        line(f"{group}: " + '; '.join(c['phase']+' '+percent(c['median_event_share']) for c in candidates[:3]) + '.')
    line()
    line('This ranking identifies broad instrumented phases only. It does not establish which container or copy operation caused their cost. A concrete next implementation and numerical acceptance target remain unselected pending inspection of these phase results against the exact source, control perturbation, required information and rollback boundaries. If that mapping is ambiguous, the result must remain explicitly unattributed rather than naming an unsupported optimization. No optimization is implemented or authorized by this arithmetic report.')
    line()
    line(f"Contract SHA-256 `{result['contract_sha256']}`. Analysis script SHA-256 `{result['script_sha256']}`. Complete per-process phase/time/origin/flow distributions, individual primary samples, retained/archive/count projections and checked input hashes are in the JSON companion. Ordinary control raw event times are unavailable; only its native summaries can be compared. No native executable, test or build was launched by this script.")
    return '\n'.join(out)+'\n'


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--capture',type=Path,required=True)
    p.add_argument('--json',type=Path,required=True)
    p.add_argument('--markdown',type=Path,required=True)
    args=p.parse_args()
    need(not args.json.exists() and not args.markdown.exists(),'Refuse overwrite; choose new analysis paths')
    root=args.capture.resolve(); manifest={}
    def payload(relative, expected=None):
        file=(root/relative).resolve();need(file.is_relative_to(root),'Path escapes capture')
        raw=file.read_bytes();actual=sha(raw);need(expected is None or actual==expected,f'Hash mismatch: {relative}')
        manifest[relative]=actual;return raw
    def read(relative, expected=None):return json.loads(payload(relative,expected).decode('utf-8'))
    contract=read('contract.json');registration=read('registration.json')
    need(manifest['contract.json']==registration['contract_sha256'],'Contract drift')
    need(contract['driver_sha256']==registration['driver_sha256'],'Driver registration mismatch')
    payload('frozen-profile.py',contract['driver_sha256'])
    payload(contract['validator']['file'],contract['validator']['sha256'])
    for item in [*contract['fixtures'],*contract['artifacts'].values(),*contract['accepted_source_identity']['records'],*contract['current_build_records']]:payload(item['file'],item['sha256'])
    cells=contract['matrix'];need(len(cells)==12 and sum(c['primary'] for c in cells)==4,'Population changed')
    attempts=[json.loads(line)for line in payload('attempts.jsonl').decode('utf-8').splitlines() if line]
    expected=[(c,m)for c in cells for m in ['accepted',*c['order']]]
    need(len(attempts)==48 and [a['key']for a in attempts]==[c['key']+'-'+m for c,m in expected],'Attempt ledger differs')
    rows=[];raw_events=Counter();last_finish=None
    for cell in cells:
        values={}
        for mode in ['accepted',*cell['order']]:
            key=cell['key']+'-'+mode;record=read('capture/'+key+'.json')
            need(record['status']==0 and record['error'] is None,f'Failed process: {key}')
            attempt=next(a for a in attempts if a['key']==key)
            need(record['argv']==attempt['argv']and record['started_at']==attempt['started_at'],'Command ledger mismatch')
            need(last_finish is None or last_finish<=record['started_at'],'Process intervals overlap')
            last_finish=record['finished_at']
            kind='diagnostic' if mode=='timing' else mode
            need(record['executable_sha256']==contract['artifacts'][kind]['sha256'],'Executable mismatch')
            need(record['revision']==(contract['accepted_source_identity']['native_build_revision']if mode=='accepted' else contract['revision']),'Source identity mismatch')
            after=read('capture/'+key+'-source-after.json')
            for check in [record['source_before'],after]:need(check['head']==contract['revision']and check['inputs_match']and check['tracked_clean'],'Recorded source observation failed')
            data=read(record['stdout']['file'],record['stdout']['sha256']);payload(record['stderr']['file'],record['stderr']['sha256'])
            need(data['instrumented']is True,'Collector work missing')
            if mode in ['timing','origins']:
                need(data['schema']=='caveat-event-phase-native-profile/1'and data['profiling_mode']==mode and data['phase_names']==PHASES,'Diagnostic schema mismatch')
                all_samples=([data['event_samples']['initialize']]if data['event_samples']['initialize']is not None else [])+sum([data['event_samples'][g]for g in GROUPS if g!='growth_last_10_percent'],[])
                for sample in all_samples:validate_sample(sample,mode)
                for group in GROUPS:verify_summary(data,group)
                raw_events[mode]+=len(all_samples)
            values[mode]=data
        sem=projection(values['accepted'])
        need(all(projection(values[m])==sem for m in ['plain','timing','origins']),f'Semantic mismatch: {cell["key"]}')
        comparison=read('capture/'+cell['key']+'-comparison.json');need(comparison['semantic_match'],'Pair validity failed')
        groups={}
        for group in GROUPS:
            count=values['plain'][group]['samples']
            if not count:groups[group]={'samples':0};continue
            native={m:values[m][group]['median_ns']for m in ['plain','timing','origins']}
            peaks={m:values[m][group]['max_dispatch_additional_heap_bytes']for m in ['plain','timing','origins']}
            groups[group]={'samples':count,'controls':{**{m+'_native_median_ns':v for m,v in native.items()},'timing_to_plain_ratio':ratio(native['timing'],native['plain']),'origins_to_plain_ratio':ratio(native['origins'],native['plain'])},'peaks':{**peaks,'timing_delta':peaks['timing']-peaks['plain'],'origins_delta':peaks['origins']-peaks['plain']},'timing':timing_statistics(events(values['timing'],group)),'origins':origin_statistics(events(values['origins'],group))}
        retained={}
        for state in ['before_release','after_release']:
            v={m:values[m][state]['retained_rust_heap_bytes']for m in ['plain','timing','origins']}
            retained[state]={**v,'timing_delta':v['timing']-v['plain'],'origins_delta':v['origins']-v['plain']}
        rows.append({**cell,'groups':groups,'retained':retained,'semantics':sem,
                     'harness':{m:{k:values[m][k]for k in ['sample_buffer_requested_bytes','harness_baseline_requested_bytes','origin_layout']}for m in ['timing','origins']},
                     'max_heap_freed_by_drain':{m:values[m]['archive']['max_heap_freed_by_drain']for m in ['plain','timing','origins']}})
    primary={g:aggregate_primary([r for r in rows if r['primary']],g)for g in ['release','failed_release']}
    ranking={g:sorted([{'phase':name,'median_event_share':v['process_median_event_share']['median'],'median_ns':v['process_median_ns']['median']}for name,v in a['phases'].items()if name!='outside'],key=lambda v:v['median_event_share'],reverse=True)for g,a in primary.items()}
    result={'schema':'caveat-event-phase-arithmetic/1','at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'revision':contract['revision'],'runtime_tree':contract['runtime_tree'],'contract_sha256':registration['contract_sha256'],'script_sha256':sha(Path(__file__).read_bytes()),'capture':str(root),'cells':12,'measured_processes':36,'accepted_reference_processes':12,'finite_semantic_equality':True,'raw_diagnostic_events':dict(raw_events),'primary':primary,'rows':rows,'candidate_phase_ranking':ranking,'recommendation_status':'Broad phase attribution only; concrete container/implementation target requires source review after capture. No optimization target preselected.','conventions':['Rejected samples reduce within process before across-four-process distributions; individual range is not independent trial count.','Derived medians average the central pair. Native summary/control ratios retain original upper-middle convention for even populations.','Per-event shares are computed before medians. Independent phase/origin medians are not an additive decomposition of a median total.','Growth-tail overlaps growth. Origin-mode phase clocks are disabled and its outer timing is observer-perturbed.','Peak location is distinct from surviving origins at one actual global peak; prior_event includes preexisting harness buffers.','Requested traffic includes full successful realloc replacements; not physical copied bytes or cumulative live memory.','Accepted-reference process timing/memory never enters comparison. Ordinary control emits no per-event samples.'],'input_hashes':[{'file':f,'sha256':h}for f,h in sorted(manifest.items())]}
    report=markdown(result)
    with args.json.open('x',encoding='utf-8',newline='\n')as f:json.dump(result,f,indent=2,allow_nan=False);f.write('\n')
    with args.markdown.open('x',encoding='utf-8',newline='\n')as f:f.write(report)
    print(json.dumps({'json':str(args.json.resolve()),'json_sha256':sha(args.json.read_bytes()),'markdown':str(args.markdown.resolve()),'markdown_sha256':sha(args.markdown.read_bytes()),'cells':12,'processes':48,'candidate_phase_ranking':ranking},indent=2))


if __name__=='__main__':main()
