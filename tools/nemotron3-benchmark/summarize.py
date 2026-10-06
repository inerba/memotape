from pathlib import Path
import json
import statistics
import math

root = Path(__file__).resolve().parents[2] / '.scratch/nemotron3-diarizzazione/benchmark08'
summary = []
expected = [f'{backend}-{inputs}-300' for backend in ('cpu','vulkan') for inputs in (1,2)]
for folder in [root/name for name in expected]:
    if not (folder/'report.json').exists() or not (folder/'process-samples.json').exists():
        continue
    report = json.loads((folder/'report.json').read_text(encoding='utf-8-sig'))
    samples = json.loads((folder/'process-samples.json').read_text(encoding='utf-8-sig'))
    gpu = json.loads((folder/'whole-gpu-samples.json').read_text(encoding='utf-8-sig'))
    elapsed = (samples[-1]['wallMs'] - samples[0]['wallMs']) / 1000
    cpu = samples[-1]['cpuSeconds'] - samples[0]['cpuSeconds']
    gpu_rows = [list(map(float,s['wholeGpu'].split(',')[-3:])) for s in gpu if s['exitCode'] == 0]
    memory = lambda key: {'peak_mib':max(s[key] for s in samples)/1048576,
        'first_stream_window_median_mib':statistics.median(s[key] for s in samples if 30000<=s['wallMs']<=60000)/1048576,
        'last_stream_window_median_mib':statistics.median(s[key] for s in samples if 270000<=s['wallMs']<=300000)/1048576}
    measures = []
    for measure in report['measures']:
        units = measure['units']
        bins = []
        for start,end in [(0,100000),(100000,200000),(200000,300000)]:
            values = sorted(u['label_available_ms']-u['audio_delivered_ms'] for u in units
                if start <= u['end_ms'] < end and u['label_available_ms'] is not None and u['audio_delivered_ms'] is not None)
            bins.append({'audio_region_ms':[start,end],'n':len(values),
                'p95_ms': values[max(0, int(math.ceil(len(values)*0.95))-1)] if values else None})
        measures.append({k:v for k,v in measure.items() if k!='units'} | {'audio_to_label_time_bins':bins})
    summary.append({'case':folder.name,'backend':report['backend'],'inputs':report['inputs'],
        'process_sampled_wall_seconds':elapsed,'process_sampled_cpu_seconds':cpu,
        'process_mean_cpu_cores_including_startup_and_final':cpu/elapsed,
        'working_set':memory('workingSet'),'private_bytes':memory('privateBytes'),
        'whole_gpu':{'peak_memory_mib':max(g[0] for g in gpu_rows),
            'mean_utilization_percent':statistics.mean(g[1] for g in gpu_rows),
            'peak_utilization_percent':max(g[1] for g in gpu_rows)},
        'load_warmup_ms':report['load_warmup_ms'],'stop_drain_ms':report['stop_drain_ms'],
        'final_analysis_ms':report['final_analysis_ms'],'live_results':report['live_results'],
        'final_outcome':report['final_outcome'],'integrity':report['integrity'],'measures':measures})
matrix_path = root/'matrix-300-summary.json'
matrix = json.loads(matrix_path.read_text(encoding='utf-8-sig')) if matrix_path.exists() else []
if isinstance(matrix,dict):
    matrix = [matrix]
complete = {case['case'] for case in summary}
missing = [name for name in expected if name not in complete]
failed = [case for case in matrix if case['exitCode'] != 0 or case['timeout'] or case.get('validReport', True) is False]
live_failed = [case['case'] for case in summary if any(result != 'Ok(())' for result in case['live_results'])]
result = {'live_diarization_failed_cases':live_failed, 'all_realtime_healthy':not missing and not failed and not live_failed, 'expected_cases':expected, 'complete_cases':len(summary), 'matrix_complete':not missing and not failed,
    'missing_or_in_progress':missing, 'failed_cases':failed, 'cases':summary}
(root/'summary300.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(result,ensure_ascii=False,indent=2))