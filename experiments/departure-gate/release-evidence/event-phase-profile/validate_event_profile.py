"""Independent data-only validation of fixed-phase native profile JSON.

No subprocesses, native execution, builds, imports of recorded scripts, or source
mutation. Import validate_data(data, mode) from the frozen capture driver, or
use the CLI on an existing stdout JSON. Optional output must be a new file.
"""
from pathlib import Path
import argparse
import hashlib
import json

PHASES = ['outside','transaction_preparation','evaluation','collection','compaction',
          'archive_construction_and_removal','binding_evaluation','final_settlement',
          'commit_cleanup','rollback_cleanup','apply_wrapper']
FLOWS = ['alloc_calls','alloc_zeroed_calls','realloc_calls','dealloc_calls',
         'allocation_failures','realloc_failures','requested_allocated_bytes','requested_freed_bytes']
WORK = ['dirty_owners','references_scanned','vertices_visited','edges_visited',
        'candidates_examined','collected','peak_work_items']

def need(condition, message):
    if not condition: raise ValueError(message)

def natural(value, label):
    need(type(value) is int and value >= 0, label + ' must be a nonnegative integer')
    return value

def array(value, length, label):
    need(type(value) is list and len(value)==length,label + ': wrong array size')
    for n,x in enumerate(value): natural(x,f'{label}[{n}]')
    return value

def validate_allocation(a, sample, header_bytes, label):
    need(type(a) is dict,label + ': missing allocation snapshot')
    arrays=['requested_live_by_origin_at_peak','live_allocations_by_origin_at_peak',
            'requested_live_by_origin_at_system_peak','requested_live_by_origin_at_end']
    for key in arrays: array(a[key],12,label+'.'+key)
    for key,value in a.items():
        if key not in arrays+['requested_peak_phase','system_requested_peak_phase','flow_by_operation_phase']:
            natural(value,label+'.'+key)
    need(a['epoch']>0,label+': epoch must be active')
    start=a['event_start_requested_live_bytes'];peak=a['requested_live_at_peak_bytes'];end=a['requested_live_at_end_bytes']
    system_start=a['event_start_system_requested_live_bytes'];system_peak=a['system_requested_peak_bytes'];system_end=a['system_requested_live_at_end_bytes']
    need(peak>=max(start,end),label+': requested peak below start/end')
    need(system_peak>=max(system_start,system_end),label+': System-request peak below start/end')
    need(system_start>=start and system_end>=end,label+': System-request bytes omit requested payload')
    need(a['requested_additional_peak_bytes']==peak-start==sample['peak_additional_bytes'],label+': requested additional peak mismatch')
    need(a['system_requested_additional_peak_bytes']==system_peak-system_start,label+': System additional peak mismatch')
    for prefix,p,s in [('requested',peak,start),('system_requested',system_peak,system_start)]:
        location=a[prefix+'_peak_phase']
        need(location is None if p==s else type(location) is int and 0<=location<11,label+': first-maximum phase mismatch')
    need(sum(a['requested_live_by_origin_at_peak'])==peak,label+': requested-peak origins do not partition')
    need(sum(a['requested_live_by_origin_at_end'])==end,label+': end origins do not partition')
    metadata=sum(a['live_allocations_by_origin_at_peak'])*header_bytes
    need(a['metadata_bytes_at_requested_peak']==metadata,label+': header/count mismatch')
    need(peak+metadata+a['alignment_padding_bytes_at_requested_peak']==a['system_requested_bytes_at_requested_peak'],label+': requested-peak System partition mismatch')
    need(a['system_requested_bytes_at_requested_peak']<=system_peak,label+': System peak below request-peak System bytes')
    system_origins=sum(a['requested_live_by_origin_at_system_peak'])
    need(system_origins<=peak,label+': request peak below System-peak requested bytes')
    need(system_origins+a['metadata_bytes_at_system_peak']+a['alignment_padding_bytes_at_system_peak']==system_peak,label+': System-peak partition mismatch')
    need(a['metadata_bytes_at_system_peak']%header_bytes==0,label+': System-peak metadata is not whole headers')
    for key in ['requested_live_by_origin_at_peak','requested_live_by_origin_at_system_peak','requested_live_by_origin_at_end']:
        need(a[key][11]<=start,label+': prior-event bytes grew within an event')
    flows=a['flow_by_operation_phase'];need(type(flows) is list and len(flows)==11,label+': wrong operation-flow population')
    for i,f in enumerate(flows):
        need(set(f)==set(FLOWS),label+': unexpected operation-flow fields')
        for key,value in f.items(): natural(value,f'{label}.flow[{i}].{key}')
        need(f['allocation_failures']<=f['alloc_calls']+f['alloc_zeroed_calls'],label+': allocation failure count exceeds calls')
        need(f['realloc_failures']<=f['realloc_calls'],label+': realloc failure count exceeds calls')
        for key in ['requested_live_by_origin_at_peak','requested_live_by_origin_at_system_peak','requested_live_by_origin_at_end']:
            need(a[key][i]<=f['requested_allocated_bytes'],label+': current origin exceeds its logical requested inflow')
    need(start+sum(f['requested_allocated_bytes']-f['requested_freed_bytes'] for f in flows)==end,label+': requested flow is not conserved')

def validate_sample(s, mode, label, header_bytes, rejected):
    need(type(s) is dict,label+': sample must be an object')
    for key in ['ns','peak_additional_bytes','phase_total_ns']: natural(s[key],label+'.'+key)
    p=s['phases'];need(type(p['clock_enabled']) is bool,label+': clock flag is not boolean')
    clocks=array(p['phase_ns'],11,label+'.phase_ns');entries=array(p['phase_entries'],11,label+'.phase_entries')
    natural(p['transitions'],label+'.transitions')
    need(p['transitions']==sum(entries),label+': transition counts do not reconcile')
    need(clocks[0]==0,label+': outside is not a timed phase')
    need(sum(clocks)==s['phase_total_ns']<=s['ns'],label+': exclusive phase total exceeds apply')
    for ns,count in zip(clocks,entries):need(count>0 or ns==0,label+': time assigned to unentered phase')
    # Registered fixtures reach either ordinary commit or late binding rejection.
    expected=[1,2,2,1,None,None,1,0 if rejected else 1,0 if rejected else 1,1 if rejected else 0,2]
    for i,n in enumerate(expected):
        if n is not None:need(entries[i]==n,label+': unexpected entry count for '+PHASES[i])
    need(entries[4]==entries[5] and entries[4] in [0,1],label+': unmatched compaction/archive entry')
    need(set(s['work'])==set(WORK),label+': unexpected collector work fields')
    for key,value in s['work'].items():natural(value,label+'.work.'+key)
    if rejected:need(all(v==0 for v in s['work'].values()),label+': rejected work placeholder changed')
    if mode=='timing':
        need(p['clock_enabled'] and s['allocations'] is None,label+': timing mode has allocation recorder or disabled clock')
        natural(s['remainder_ns'],label+'.remainder_ns')
        need(s['phase_total_ns']+s['remainder_ns']==s['ns'],label+': phase plus residual does not equal apply')
    else:
        need(not p['clock_enabled'] and all(v==0 for v in clocks) and s['remainder_ns'] is None,label+': origins mode claims measured phase times')
        validate_allocation(s['allocations'],s,header_bytes,label+'.allocations')

def validate_summary(summary,samples,label):
    n=len(samples);need(summary['samples']==n,label+': sample count differs from raw events')
    if not n:return
    times=sorted(s['ns'] for s in samples)
    expected={'median_ns':times[n//2],'p99_ns':times[(n*99+99)//100-1],
      'max_ns':times[-1],'mean_ns':sum(times)//n,'max_dispatch_additional_heap_bytes':max(s['peak_additional_bytes'] for s in samples)}
    for key,value in expected.items():need(summary[key]==value,label+': raw/summary mismatch for '+key)
    expected_work={key:sum(s['work'][key] for s in samples) for key in WORK if key!='peak_work_items'}
    expected_work['max_work_items']=max(s['work']['peak_work_items'] for s in samples)
    need(summary['collector_work']==expected_work,label+': raw/summary collector work mismatch')

def validate_data(data,mode,probes=100):
    need(mode in ['timing','origins'],'unsupported diagnostic mode')
    need(data['schema']=='caveat-event-phase-native-profile/1','wrong native schema')
    need(data['profiling_mode']==mode and data['instrumented'] is True,'wrong native mode/counters')
    need(data['phase_names']==PHASES and data['phase_count']==11,'wrong fixed phase inventory')
    cycles=natural(data['cycles'],'cycles');need(cycles>0,'no growth population')
    need(data['mode'] in ['release','initialize'],'unexpected fixture mode')
    need(data['drain_every']==1,'unexpected drain frequency')
    for key in ['sample_buffer_requested_bytes','harness_baseline_requested_bytes']:natural(data[key],key)
    need(data['harness_baseline_requested_bytes']>=data['sample_buffer_requested_bytes'],'sample buffers exceed pre-session baseline')
    layout=data['origin_layout'];header=0
    if mode=='timing':need(layout is None,'timing layout must be null')
    else:
        header=natural(layout['header_bytes'],'header_bytes');need(header>0,'zero header size')
        need(layout['origin_buckets']==12 and layout['prior_event_index']==11 and layout['origin_phase_names']==PHASES,'wrong origin layout')
    e=data['event_samples'];keys=['initialize','growth','steady','unrelated_changes','failed_release','release']
    need(set(e)==set(keys),'wrong raw event populations')
    counts={'growth':cycles,'steady':probes,'unrelated_changes':probes,'failed_release':3 if data['mode']=='release' else 0,'release':1 if data['mode']=='release' else 0}
    chronological=[]
    need((e['initialize'] is not None)==(data['mode']=='initialize'),'wrong initialization sample presence')
    if e['initialize'] is not None:
        validate_sample(e['initialize'],mode,'initialize',header,False);chronological.append(e['initialize'])
    for key,n in counts.items():
        need(type(e[key]) is list and len(e[key])==n,key+': wrong raw event count')
        for i,s in enumerate(e[key]):validate_sample(s,mode,f'{key}[{i}]',header,key=='failed_release')
        chronological.extend(e[key]);validate_summary(data[key],e[key],key)
    validate_summary(data['growth_last_10_percent'],e['growth'][cycles*9//10:],'growth_last_10_percent')
    if mode=='origins':need([s['allocations']['epoch'] for s in chronological]==list(range(1,len(chronological)+1)),'allocation epochs do not follow raw event order')
    return {'schema':'caveat-event-phase-data-validation/1','valid':True,'mode':mode,'rawEvents':len(chronological),
      'rawRejectedEvents':counts['failed_release'],'phaseCount':11,'originBuckets':12 if mode=='origins' else None,
      'rawSummariesReconciled':6,'exclusivePhaseAndResidualReconciled':mode=='timing',
      'originPartitionsAndRequestedFlowReconciled':mode=='origins',
      'scope':'Saved-data consistency and finite registered fixture boundaries; not native execution or proof of authentic origin.'}

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--input',type=Path,required=True)
    p.add_argument('--mode',choices=['timing','origins'],required=True);p.add_argument('--output',type=Path);p.add_argument('--probes',type=int,default=100)
    a=p.parse_args()
    if a.output:need(not a.output.exists(),'Refuse overwrite')
    raw=a.input.read_bytes();result=validate_data(json.loads(raw.decode('utf-8-sig')),a.mode,a.probes)
    result.update(input=str(a.input.resolve()),inputSha256=hashlib.sha256(raw).hexdigest(),scriptSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
    if a.output:
        with a.output.open('x',encoding='utf-8',newline='\n') as f:json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps(result))

if __name__=='__main__':main()
