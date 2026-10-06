"""Score independent construction references; no hypothesis labels enter reference units."""
from pathlib import Path
import argparse
import hashlib
import json
from corpus import CORPUS, RATE, save_json, sha
from metrics import read_rttm, der, attribution


def frozen_units(rows):
    units=[]
    for phrase in rows:
        encoded=phrase['text'].encode('utf-8')
        for timing in phrase['tempi']:
            units.append({'start_ms':timing['inizio_ms'], 'end_ms':timing['fine_ms'],
                          'text':encoded[timing['inizio_byte']:timing['fine_byte']].decode('utf-8')})
    return units


def reference_speaker(start_ms, end_ms, turns):
    """Conservative label: only one active identity and >=25% supported time.

    Any boundary crossing between identities or overlap is reference-ambiguous.
    No acoustic support also remains null, never a hypothesis-derived label.
    """
    left,right=start_ms/1000,end_ms/1000
    intersections=[(max(left,s),min(right,e),speaker) for s,e,speaker in turns if s<right and e>left]
    speakers={speaker for s,e,speaker in intersections}
    if len(speakers)!=1:
        return None
    intervals=sorted((s,e) for s,e,_ in intersections)
    merged=[]
    for s,e in intervals:
        if merged and s<=merged[-1][1]:
            merged[-1]=(merged[-1][0],max(e,merged[-1][1]))
        else:
            merged.append((s,e))
    supported=sum(e-s for s,e in merged)
    return next(iter(speakers)) if supported >= 0.25*(right-left) else None


def compose_mapping(projected_to_raw, raw_to_reference):
    return {str(projected):raw_to_reference.get(str(raw)) for projected,raw in projected_to_raw.items()}


def score_case(folder, backend):
    manifest=json.loads((folder/'manifest.json').read_text(encoding='utf-8'))
    assert sha(folder/'audio.wav')==manifest['audio_sha256']
    comparison=folder/f'comparison-{backend}'
    frozen=json.loads((comparison/'frozen-asr.json').read_text(encoding='utf-8'))
    units=frozen_units(frozen)
    assert units,'no timed ASR units; cannot report text attribution'
    canonical=json.dumps(units,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode('utf-8')
    unit_hash=hashlib.sha256(canonical).hexdigest()
    report={'case':manifest['case'],'duration_seconds':manifest['duration_seconds'],
            'source_kind':manifest['kind'],'declared_identities':manifest['speakers'],
            'reference_kind':manifest['boundary_reference'],'audio_sha256':manifest['audio_sha256'],
            'frozen_asr_sha256':sha(comparison/'frozen-asr.json'),'unit_text_time_sha256':unit_hash,
            'asr_coverage':{'phrases':len(frozen),'untimed_phrases':sum(not p['tempi'] for p in frozen),
                            'text_utf8_bytes':sum(len(p['text'].encode('utf-8')) for p in frozen),
                            'timed_text_utf8_bytes':sum(len(u['text'].encode('utf-8')) for u in units),
                            'timed_units':len(units),'word_error_rate':None},
            'backend':backend,'scores':{}}
    for db in (-55,-45,-35):
        ref_path=folder/f'reference{db}.rttm'
        assert sha(ref_path)==manifest['reference_sha256'][str(db)]
        turns=read_rttm(ref_path,'sample')
        assert {t[2] for t in turns}==set(manifest['speakers'])
        assert all(0<=s<e<=manifest['duration_seconds'] for s,e,_ in turns)
        references=[u|{'speaker':reference_speaker(u['start_ms'],u['end_ms'],turns)} for u in units]
        save_json(comparison/f'reference-units{db}.json',references)
        models={}
        for name in ('sortformer','nemotron3'):
            hypotheses=json.loads((comparison/f'{name}-units.json').read_text(encoding='utf-8'))
            projected_units=[{k:u[k] for k in ('start_ms','end_ms','text')} for u in hypotheses]
            assert projected_units==units,'models did not use exactly frozen ASR units'
            hypothesis_hash=hashlib.sha256(json.dumps(projected_units,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode('utf-8')).hexdigest()
            metrics=der(turns,read_rttm(comparison/f'{name}.rttm','sample'),0,manifest['duration_seconds'])
            namespace_path=comparison/f'{name}-speaker-namespace.json'
            namespace=json.loads(namespace_path.read_text(encoding='utf-8'))
            mapped=compose_mapping(namespace['projected_to_raw'],metrics['mapping_hypothesis_to_reference'])
            assert all(u['speaker'] is None or str(u['speaker']) in mapped for u in hypotheses)
            metrics['text_attribution']=attribution(references,hypotheses,mapped,0,manifest['duration_seconds']*1000)
            metrics['mapping_projected_to_reference']=mapped
            metrics['speaker_namespace_source']=namespace['source']
            metrics['speaker_namespace_sha256']=sha(namespace_path)
            metrics['same_asr_unit_hash']=hypothesis_hash
            metrics['inference']=json.loads((comparison/f'{name}-inference.json').read_text(encoding='utf-8'))
            models[name]=metrics
        report['scores'][str(db)]=models
    save_json(comparison/'scores.json',report)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--backend',default='vulkan')
    parser.add_argument('--case')
    args=parser.parse_args()
    folders=[CORPUS/'cases'/args.case] if args.case else sorted((CORPUS/'cases').iterdir())
    reports=[]
    for folder in folders:
        if not (folder/f'comparison-{args.backend}'/'frozen-asr.json').exists():
            continue
        report=score_case(folder,args.backend)
        reports.append(report)
        for model,metrics in report['scores']['-45'].items():
            a=metrics['text_attribution']
            print(report['case'],model,'DER',round(metrics['der']*100,2),'assignment',a['correct'],a['wrong'],a['unknown'],'ambiguous',a['reference_ambiguous'],'n',a['evaluated_units'])
    expected=[c['case'] for c in json.loads((CORPUS/'corpus-manifest.json').read_text(encoding='utf-8'))['cases']]
    save_json(CORPUS/f'scores-{args.backend}.json',{'expected_cases':expected,'complete_cases':len(reports),
              'missing_cases':[c for c in expected if c not in {r['case'] for r in reports}],
              'reference_is_manual_gold':False,'reports':reports})


if __name__=='__main__':
    main()