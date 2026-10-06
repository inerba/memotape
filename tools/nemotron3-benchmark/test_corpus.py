import io
import tarfile
import tempfile
import unittest
from pathlib import Path
from corpus import RATE, activity, safe_extract
from score_corpus import compose_mapping, frozen_units, reference_speaker
from metrics import attribution


class CorpusTests(unittest.TestCase):
    def archive(self, path, name='speaker/file.txt', kind=tarfile.REGTYPE, duplicate=False):
        with tarfile.open(path, 'w:gz') as source:
            for _ in range(2 if duplicate else 1):
                member=tarfile.TarInfo(name)
                member.type=kind
                member.size=2 if kind==tarfile.REGTYPE else 0
                member.linkname='../outside'
                source.addfile(member,io.BytesIO(b'ok') if member.size else None)

    def test_tar_valid_extract_and_existing_refused(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp); archive=root/'sample.tgz'; target=root/'output'
            self.archive(archive)
            safe_extract(archive,target)
            self.assertEqual((target/'speaker/file.txt').read_bytes(),b'ok')
            with self.assertRaises(ValueError): safe_extract(archive,target)
            self.assertEqual((target/'speaker/file.txt').read_bytes(),b'ok')

    def test_tar_traversal_links_and_duplicates_rejected_before_write(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp); archive=root/'sample.tgz'
            for name,kind,duplicate in [('../outside',tarfile.REGTYPE,False),
                    ('/absolute',tarfile.REGTYPE,False),('C:/outside',tarfile.REGTYPE,False),
                    ('speaker/link',tarfile.SYMTYPE,False),('speaker/link',tarfile.LNKTYPE,False),
                    ('speaker/file',tarfile.REGTYPE,True)]:
                with self.subTest(name=name,kind=kind,duplicate=duplicate):
                    self.archive(archive,name,kind,duplicate)
                    with self.assertRaises(ValueError): safe_extract(archive,root/'output')
                    self.assertFalse((root/'output').exists())
                    self.assertFalse((root/'outside').exists())

    def test_activity_excludes_silence_and_retains_actual_pulse(self):
        step=RATE//100
        samples=[0.0]*(20*step)+[0.1]*(10*step)+[0.0]*(20*step)
        self.assertEqual(activity(samples),[(18*step,32*step)])
        self.assertEqual(activity([0.0]*RATE),[])
        # A very quiet clean pulse crosses -55dB but not -45dB, independently known.
        self.assertTrue(activity([0.004]*RATE,-55))
        self.assertFalse(activity([0.004]*RATE,-45))

    def test_reference_uses_only_construction_and_handles_overlap_boundaries(self):
        turns=[(0,1,'A'),(0.8,1.8,'B')]
        self.assertEqual(reference_speaker(100,700,turns),'A')
        self.assertIsNone(reference_speaker(750,950,turns))
        self.assertIsNone(reference_speaker(1900,2100,turns))
        self.assertIsNone(reference_speaker(1750,2000,turns))
        self.assertEqual(reference_speaker(1000,1600,turns),'B')

    def test_projected_ids_compose_with_raw_mapping_and_permutation(self):
        reference=[{'start_ms':100,'end_ms':200,'text':'prima','speaker':'A'},
                   {'start_ms':300,'end_ms':400,'text':'seconda','speaker':'B'}]
        hypothesis=[reference[0]|{'speaker':'1'},reference[1]|{'speaker':'2'}]
        # First phrase raw4 becomes displayed1, next raw3 becomes displayed2.
        mapped=compose_mapping({'1':4,'2':3},{'4':'A','3':'B'})
        swapped=compose_mapping({'1':3,'2':4},{'3':'A','4':'B'})
        self.assertEqual(mapped,{'1':'A','2':'B'})
        self.assertEqual(mapped,swapped)
        self.assertEqual(attribution(reference,hypothesis,mapped,0,500)['correct'],2)
        self.assertEqual(attribution(reference,hypothesis,{'4':'A','3':'B'},0,500)['wrong'],2)

    def test_frozen_units_respect_utf8_byte_boundaries(self):
        rows=[{'text':'è sì','tempi':[{'inizio_byte':0,'fine_byte':3,'inizio_ms':100,'fine_ms':200},
                                     {'inizio_byte':3,'fine_byte':6,'inizio_ms':200,'fine_ms':300}]}]
        self.assertEqual(frozen_units(rows),[{'start_ms':100,'end_ms':200,'text':'è '},
                                            {'start_ms':200,'end_ms':300,'text':'sì'}])


if __name__=='__main__':
    unittest.main()