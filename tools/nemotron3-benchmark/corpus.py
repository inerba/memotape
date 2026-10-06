"""Construct finite independent-reference audio cases; never reads diarizer output."""
from pathlib import Path, PurePosixPath
import argparse
import array
import hashlib
import json
import math
import random
import re
import subprocess
import tarfile
import wave

ROOT = Path(__file__).resolve().parents[2]
CORPUS = ROOT / '.scratch/nemotron3-diarizzazione/corpus08'
RATE = 16000
ARCHIVES = ['codex-20161203-aur', 'wperw-20170829-naz', 'Grigomax-20170503-ulc',
            'rrobotics-20160824-tqk', 'marianomarini-20160603-pmd',
            'DavideMiccich-20160328-daj', 'remix_tj-20160329-nki', 'OscarCappa-20140902-ttm']
TEXTS = ['La riunione comincia domani alle nove.', 'Confermo la consegna entro venerdì sera.',
         'Il costo previsto comprende tutte le spese.', 'Abbiamo raccolto i documenti per il progetto.',
         'Possiamo anticipare la telefonata di mezzora.', 'Serve una verifica prima della firma finale.',
         'La nuova proposta riduce i tempi necessari.', 'Vorrei controllare insieme gli ultimi dettagli.',
         'Il responsabile chiamerà dopo la pausa pranzo.', 'Questo dato deve essere corretto nel verbale.',
         'Teniamo aperta la discussione fino a domani.', 'Grazie, abbiamo concluso il primo punto.']
VOICES = [('elsa', 'Microsoft Elsa'), ('cosimo', 'Microsoft Cosimo'), ('zira', 'Microsoft Zira Desktop')]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def save_json(path, value):
    Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')


def safe_extract(archive, folder):
    """Validate every entry before any write; no links, traversal or device entries."""
    folder = Path(folder).resolve()
    with tarfile.open(archive, 'r:gz') as source:
        members = source.getmembers()
        if sum(m.size for m in members) > 80 * 1024 * 1024:
            raise ValueError('archive exceeds finite benchmark limit')
        seen = set()
        for member in members:
            path = PurePosixPath(member.name)
            if path.is_absolute() or '..' in path.parts or '\\' in member.name or ':' in member.name:
                raise ValueError('unsafe archive path')
            if not (member.isfile() or member.isdir()):
                raise ValueError('archive links/special files forbidden')
            target = (folder / member.name).resolve()
            if not target.is_relative_to(folder):
                raise ValueError('archive escapes destination')
            if target in seen or target.exists():
                raise ValueError('refusing duplicate/existing archive destination')
            seen.add(target)
        for member in members:
            target = folder / member.name
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                with source.extractfile(member) as stream:
                    target.write_bytes(stream.read())


def pcm(path):
    with wave.open(str(path), 'rb') as source:
        assert source.getnchannels() == 1 and source.getsampwidth() == 2
        assert source.getframerate() == RATE
        values = array.array('h', source.readframes(source.getnframes()))
    return [x / 32768 for x in values]


def write_wav(path, values):
    samples = array.array('h', [round(max(-1, min(0.999969, x)) * 32768) for x in values])
    with wave.open(str(path), 'wb') as output:
        output.setparams((1, 2, RATE, 0, 'NONE', 'not compressed'))
        output.writeframes(samples.tobytes())


def activity(values, db=-45):
    """10 ms RMS, 20 ms edge padding, merge <=120 ms gaps, before mixing/noise."""
    threshold = 10 ** (db / 20)
    step = RATE // 100
    active = []
    for start in range(0, len(values), step):
        frame = values[start:start + step]
        if frame and math.sqrt(sum(x*x for x in frame) / len(frame)) >= threshold:
            active.append((max(0, start - 2*step), min(len(values), start + 3*step)))
    merged = []
    for start, end in active:
        if merged and start <= merged[-1][1] + 12*step:
            merged[-1] = (merged[-1][0], max(end, merged[-1][1]))
        else:
            merged.append((start, end))
    return merged


def prepare_public():
    public = CORPUS / 'public'
    public.mkdir(exist_ok=True)
    speakers = []
    for archive_name in ARCHIVES:
        archive = CORPUS / 'archives' / (archive_name + '.tgz')
        folder = public / archive_name
        if not folder.exists():
            safe_extract(archive, public)
        readme = (folder / 'etc/README').read_text(encoding='utf-8', errors='replace')
        match = re.search(r'User Name:\s*([^\r\n]+)', readme)
        if not match or 'Language: IT' not in readme:
            raise ValueError('missing declared Italian speaker identity')
        license_text = (folder / 'LICENSE').read_text(encoding='utf-8', errors='replace')
        if 'GNU General Public License' not in license_text or 'version 3' not in license_text:
            raise ValueError('archive license requires inspection')
        speaker = match.group(1).strip()
        clips = []
        for wav_path in sorted((folder/'wav').glob('*.wav'))[:8]:
            with wave.open(str(wav_path), 'rb') as source:
                info = {'rate': source.getframerate(), 'channels': source.getnchannels(),
                        'sample_width': source.getsampwidth(), 'frames': source.getnframes()}
            target = CORPUS/'stems'/(archive_name+'-'+wav_path.name)
            if not target.exists():
                subprocess.run(['ffmpeg', '-nostdin', '-hide_banner', '-loglevel', 'error',
                                '-i', str(wav_path), '-ac', '1', '-ar', str(RATE), '-c:a', 'pcm_s16le', str(target)], check=True)
            clips.append({'path':target.relative_to(CORPUS).as_posix(), 'sha256':sha(target),
                          'original_path':wav_path.relative_to(CORPUS).as_posix(),
                          'original_sha256':sha(wav_path), 'original_codec':info})
        if len(clips) < 2:
            raise ValueError('insufficient audio clips')
        speakers.append({'speaker':speaker, 'kind':'source-labelled human reading', 'archive':archive.name,
                         'archive_sha256':sha(archive), 'archive_bytes':archive.stat().st_size,
                         'source_page':'https://www.voxforge.org/home/downloads/speech/italian-speech-files/'+archive_name,
                         'download':'https://repository.voxforge1.org/downloads/it/Trunk/Audio/Original/48kHz_16bit/'+archive.name,
                         'license':'GPL-3.0-or-later', 'license_sha256':sha(folder/'LICENSE'),
                         'readme_sha256':sha(folder/'etc/README'), 'clips':clips})
    assert len({s['speaker'] for s in speakers}) == 8
    save_json(CORPUS/'public-sources.json', speakers)
    return speakers


def rttm(path, turns):
    Path(path).write_text(''.join(f'SPEAKER sample 1 {start/RATE:.6f} {(end-start)/RATE:.6f} <NA> <NA> {speaker} <NA> <NA>\n'
                                for start,end,speaker in sorted(turns)), encoding='utf-8', newline='\n')


def construct(name, speakers, condition):
    folder = CORPUS / 'cases' / name
    if folder.exists():
        raise ValueError('refusing to overwrite a corpus case')
    folder.mkdir(parents=True)
    values = [0.0] * (120*RATE)
    placements = []
    turns = {db:[] for db in (-55,-45,-35)}
    cursor = RATE
    rng = random.Random(8082026)
    for index in range(16):
        source = speakers[index % len(speakers)]
        clip = source['clips'][(index // len(speakers)) % len(source['clips'])]
        original = pcm(CORPUS / clip['path'])
        raw_activity = activity(original, -55)
        if not raw_activity:
            raise ValueError('empty stem activity')
        left = max(0, raw_activity[0][0]-320)
        right = min(len(original), raw_activity[-1][1]+320, left+int(4.5*RATE))
        stem = original[left:right]
        if condition == 'rapid_overlap_noise' and index % 6 == 4:
            stem = stem[:int(2.25*RATE)]
        rms = math.sqrt(sum(x*x for x in stem)/len(stem))
        gain = min(0.075/rms, 0.75/max(abs(x) for x in stem))
        stem = [x*gain for x in stem]
        if condition != 'clean' and index in (3,7,11):
            cursor -= int(0.85*RATE)
        if index == 8:
            cursor += 4*RATE
        start = cursor
        end = start + len(stem)
        assert end < len(values)
        for offset,x in enumerate(stem):
            values[start+offset] += x
        stem_path = folder/f'stem-{index:02}.wav'
        write_wav(stem_path, stem)
        # Annotate exact emitted PCM, before noise and independent of both diarizers.
        emitted = pcm(stem_path)
        for db in turns:
            turns[db] += [(start+a,start+b,source['speaker']) for a,b in activity(emitted,db)]
        placements.append({'index':index, 'speaker':source['speaker'], 'source':clip['path'],
                           'source_sha256':clip['sha256'], 'source_crop_samples':[left,left+len(stem)],
                           'start_sample':start,'end_sample':end, 'gain':gain,
                           'stem':stem_path.name,'stem_sha256':sha(stem_path)})
        cursor = end + int((0.2 if condition=='rapid_overlap_noise' else 0.45)*RATE)
    duration = max(60*RATE, cursor+3*RATE)
    if duration > 120*RATE:
        raise ValueError('finite case exceeds 120 seconds')
    values = values[:duration]
    noise_rms = 0.006 if condition=='rapid_overlap_noise' else 0
    if noise_rms:
        # Gaussian white noise with a finite explicit silent start/end.
        for i in range(RATE,len(values)-RATE):
            values[i] += rng.gauss(0,noise_rms)
    peak = max(abs(x) for x in values)
    assert peak < 1, 'no clipped mixture permitted'
    write_wav(folder/'audio.wav', values)
    for db,ref in turns.items():
        rttm(folder/f'reference{db}.rttm',ref)
    manifest={'case':name,'kind':speakers[0]['kind'],'speaker_count':len(speakers),
              'speakers':[s['speaker'] for s in speakers], 'condition':condition,
              'duration_samples':duration,'rate':RATE,'duration_seconds':duration/RATE,
              'seed':8082026,'noise_rms':noise_rms,'mixture_peak':peak,
              'boundary_reference':'construction identity + clean-stem energy proxy; NOT manually verified gold',
              'activity':{'rms_frame_ms':10,'padding_ms':20,'merge_gap_ms':120,'db_thresholds':[-55,-45,-35],
                          'primary_db':-45}, 'placements':placements,
              'audio_sha256':sha(folder/'audio.wav'),
              'reference_sha256':{str(db):sha(folder/f'reference{db}.rttm') for db in turns}}
    save_json(folder/'manifest.json',manifest)
    print(name, round(duration/RATE,3),'seconds',len(speakers),'source identities')
    return manifest


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--prepare-jobs',action='store_true')
    parser.add_argument('--construct',action='store_true')
    args=parser.parse_args()
    CORPUS.mkdir(exist_ok=True)
    (CORPUS/'.gitignore').write_text('*\n!.gitignore\n',encoding='utf-8',newline='\n')
    (CORPUS/'stems').mkdir(exist_ok=True)
    if args.prepare_jobs:
        save_json(CORPUS/'tts-jobs.json',[{'voice':voice,'speaker':speaker,'text':text,
            'output':str((CORPUS/'stems'/f'tts-{speaker}-{i:02}.wav').resolve())}
            for speaker,voice in VOICES for i,text in enumerate(TEXTS)])
    if args.construct:
        public=prepare_public()
        tts=[{'speaker':speaker,'kind':'Windows TTS, Italian text; Zira en-US accent if present',
              'voice_name':voice,'clips':[{'path':f'stems/tts-{speaker}-{i:02}.wav',
                                        'sha256':sha(CORPUS/'stems'/f'tts-{speaker}-{i:02}.wav')}
                                       for i in range(len(TEXTS))]} for speaker,voice in VOICES]
        cases=[construct('tts2-clean',tts[:2],'clean'),
               construct('tts3-overlap-noise',tts,'rapid_overlap_noise'),
               construct('human2-clean',public[:2],'clean'),
               construct('human4-overlap',public[:4],'overlap'),
               construct('human8-rapid-overlap-noise',public,'rapid_overlap_noise')]
        save_json(CORPUS/'corpus-manifest.json',{'cases':cases,'public_sources':public,
                   'tts_sources':tts,'tts_texts':TEXTS,'generator_sha256':sha(__file__),
                   'ffmpeg_version':subprocess.run(['ffmpeg','-version'],capture_output=True,text=True,check=True).stdout.splitlines()[0]})


if __name__=='__main__':
    main()