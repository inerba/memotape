import tempfile

import unittest

from pathlib import Path

from metrics import attribution, der, read_rttm

class MetricsTests(unittest.TestCase):

    def test_permutation_has_zero_error(self):

        result = der([(0, 1, "A"), (1, 2, "B")], [(0, 1, "2"), (1, 2, "1")], 0, 2)

        self.assertEqual(result["der"], 0)

        self.assertEqual(result["mapping_hypothesis_to_reference"], {"1": "B", "2": "A"})

    def test_exact_miss_false_alarm_and_confusion(self):

        # Mapping is pinned by long A/1 and B/2 anchor intervals.

        ref = [(0, 5, "A"), (6, 10, "B")]

        hyp = [(0, 4, "1"), (5, 6, "1"), (6, 7, "1"), (7, 10, "2")]

        result = der(ref, hyp, 0, 10)

        self.assertEqual(result["missed_seconds"], 1)

        self.assertEqual(result["false_alarm_seconds"], 1)

        self.assertEqual(result["confusion_seconds"], 1)

        self.assertAlmostEqual(result["der"], 3 / 9)

    def test_overlap_scores_speaker_time_and_duplicate_segments_once(self):

        result = der([(0, 2, "A"), (1, 2, "B"), (0, 2, "A")], [(0, 2, "1")], 0, 2)

        self.assertEqual(result["reference_speaker_seconds"], 3)

        self.assertEqual(result["overlap_seconds"], 1)

        self.assertEqual(result["missed_seconds"], 1)

        self.assertAlmostEqual(result["der"], 1 / 3)

    def test_scoring_region_excludes_unannotated_tail(self):

        self.assertEqual(der([(0, 1, "A")], [(0, 1, "1"), (1, 20, "2")], 0, 1)["der"], 0)

        self.assertEqual(der([(0, 1, "A")], [], 0, 1)["der"], 1)

        with self.assertRaises(ValueError):

            der([], [], 0, 1)

    def test_eight_speakers_and_more_rejected(self):

        ref = [(i, i + 1, str(i)) for i in range(8)]

        hyp = [(i, i + 1, str(7 - i)) for i in range(8)]

        self.assertEqual(der(ref, hyp, 0, 8)["der"], 0)

        with self.assertRaises(ValueError):

            der(ref + [(8, 9, "8")], hyp, 0, 9)

    def test_rttm_rejects_wrong_recording_and_invalid_time(self):

        with tempfile.TemporaryDirectory() as folder:

            path = Path(folder) / "ref.rttm"

            path.write_text("SPEAKER sample 1 0.5 1.0 <NA> <NA> A <NA> <NA>\n")

            self.assertEqual(read_rttm(path, "sample"), [(0.5, 1.5, "A")])

            with self.assertRaises(ValueError):

                read_rttm(path, "other")

            path.write_text("SPEAKER sample 1 -1 1 <NA> <NA> A <NA> <NA>\n")

            with self.assertRaises(ValueError):

                read_rttm(path, "sample")

    def test_attribution_uses_der_region_and_excludes_crossing_units(self):

        ref = [{"start_ms": s, "end_ms": e, "text": "ciao", "speaker": "A"} for s, e in [(0, 100), (100, 200), (150, 250), (200, 300)]]

        hyp = [dict(unit, speaker="1" if index == 1 else "2") for index, unit in enumerate(ref)]

        result = attribution(ref, hyp, {"1": "A"}, 100, 200)

        self.assertEqual(result["correct"], 1)

        self.assertEqual(result["outside_region"], 3)

        self.assertEqual(result["assignment_error_rate"], 0)

    def test_assignment_errors_are_separate_from_asr(self):

        units = [{"start_ms": i * 100, "end_ms": (i + 1) * 100, "text": "ciao", "speaker": "A"} for i in range(4)]

        predictions = [dict(unit, speaker=speaker) for unit, speaker in zip(units, ["1", "2", None, "1"])]

        units[-1]["speaker"] = None

        result = attribution(units, predictions, {"1": "A", "2": "B"}, 0, 400)

        self.assertEqual([result[k] for k in ("correct", "wrong", "unknown", "reference_ambiguous")], [1, 1, 1, 1])

        self.assertAlmostEqual(result["assignment_error_rate"], 2 / 3)

        self.assertIsNone(result["asr_word_error_rate"])

        predictions[0]["text"] = "altro"

        with self.assertRaises(ValueError):

            attribution(units, predictions, {"1": "A"}, 0, 400)

if __name__ == "__main__":

    unittest.main()