from pathlib import Path

import argparse

import difflib

import hashlib

import json

repo = Path(__file__).resolve().parents[2]

parser = argparse.ArgumentParser()
parser.add_argument('--baseline', default=r'C:\Users\inerba\.codex\worktrees\0acd\sbobino\.scratch\nemotron3-diarizzazione\baselines\08')
parser.add_argument('--name', choices=['08','08-extension'], default='08')
args = parser.parse_args()
baseline = Path(args.baseline)

manifest = json.loads((baseline/'manifest.json').read_text(encoding='utf-8-sig'))

changes = []

patch = []

for entry in manifest:

    relative = entry['path'].replace('\\','/')

    old = baseline/relative

    current = repo/relative

    before = old.read_bytes()

    if hashlib.sha256(before).hexdigest().lower() != entry['sha256'].lower():

        raise RuntimeError(f'baseline changed: {relative}')

    after = current.read_bytes() if current.exists() else b''

    if before != after:

        changes.append(relative)

        patch.extend(difflib.unified_diff(before.decode('utf-8-sig').replace('\r\n','\n').splitlines(True),after.decode('utf-8-sig').replace('\r\n','\n').splitlines(True),fromfile='a/'+relative,tofile='b/'+relative))

new_files = ['src-tauri/src/engine/windows_benchmark.rs', 'tools/nemotron3-benchmark/run.ps1',

    'tools/nemotron3-benchmark/metrics.py', 'tools/nemotron3-benchmark/test_metrics.py',

    'tools/nemotron3-benchmark/README.md', 'tools/nemotron3-benchmark/isolate_diff.py', 'tools/nemotron3-benchmark/summarize.py', '.scratch/nemotron3-diarizzazione/ui08-da-verificare.md',

    '.scratch/nemotron3-diarizzazione/report08.md', '.scratch/nemotron3-diarizzazione/review-08.md',
    '.scratch/nemotron3-diarizzazione/report08-addendum.md',
    'tools/nemotron3-benchmark/corpus.py', 'tools/nemotron3-benchmark/synthesize.ps1',
    'tools/nemotron3-benchmark/compare-corpus.ps1', 'tools/nemotron3-benchmark/score_corpus.py',
    'tools/nemotron3-benchmark/test_corpus.py', '.scratch/nemotron3-diarizzazione/corpus08/.gitignore',
    '.scratch/nemotron3-diarizzazione/baselines/08-extension/.gitignore']

baseline_paths = {entry['path'].replace('\\','/') for entry in manifest}

for relative in new_files:

    if relative in baseline_paths:
        continue

    path = repo/relative

    if not path.exists():

        continue

    changes.append(relative)

    patch.extend(difflib.unified_diff([],path.read_text(encoding='utf-8-sig').splitlines(True),fromfile='/dev/null',tofile='b/'+relative))

(repo/f'.scratch/nemotron3-diarizzazione/{args.name}.diff').write_text(''.join(patch),encoding='utf-8',newline='\n')

(repo/f'.scratch/nemotron3-diarizzazione/{args.name}-files.json').write_text(json.dumps(changes,indent=2)+'\n',encoding='utf-8',newline='\n')

print(json.dumps(changes,indent=2))