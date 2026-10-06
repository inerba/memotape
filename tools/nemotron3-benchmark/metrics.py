"""Exact interval DER (zero collar, overlap scored) and separate frozen-ASR attribution.

No external dependency. Reference must be independently annotated, never model output.

"""

import argparse

import json

from pathlib import Path

def read_rttm(path, recording):

    turns = []

    for number, line in enumerate(Path(path).read_text(encoding="utf-8").splitlines(), 1):

        if not line.strip() or line.startswith("#"):

            continue

        fields = line.split()

        if len(fields) != 10 or fields[0] != "SPEAKER":

            raise ValueError(f"invalid RTTM line {number}")

        if fields[1] != recording:

            raise ValueError(f"unexpected recording at line {number}")

        start, duration = float(fields[3]), float(fields[4])

        if not (0 <= start < start + duration < float("inf")):

            raise ValueError(f"invalid interval at line {number}")

        turns.append((start, start + duration, fields[7]))

    return turns

def best_mapping(reference, hypothesis, weights):

    """Maximum shared speaker-time, one-to-one mapping; subset DP handles <=8 labels."""

    size = max(len(reference), len(hypothesis))

    if size > 8:

        raise ValueError("benchmark supports at most eight labels per recording")

    reference = list(reference) + [None] * (size - len(reference))

    hypothesis = list(hypothesis) + [None] * (size - len(hypothesis))

    states = {0: (0.0, [])}

    for speaker in hypothesis:

        following = {}

        for mask, (score, choices) in states.items():

            for index, ref in enumerate(reference):

                if mask & (1 << index):

                    continue

                candidate = (score + weights.get((ref, speaker), 0.0), choices + [ref])

                key = mask | (1 << index)

                if key not in following or candidate[0] > following[key][0]:

                    following[key] = candidate

        states = following

    chosen = states.get((1 << size) - 1, (0.0, []))[1]

    return {speaker: ref for speaker, ref in zip(hypothesis, chosen) if speaker is not None}

def der(reference, hypothesis, start, end):

    if not (0 <= start < end < float("inf")):

        raise ValueError("explicit scoring region must be finite and nonempty")

    boundaries = sorted({start, end} | {max(start, min(end, t)) for turns in (reference, hypothesis) for s, e, _ in turns for t in (s, e)})

    cells, weights = [], {}

    ref_labels, hyp_labels = set(), set()

    for left, right in zip(boundaries, boundaries[1:]):

        middle = (left + right) / 2

        refs = {label for s, e, label in reference if s <= middle < e}

        hyps = {label for s, e, label in hypothesis if s <= middle < e}

        duration = right - left

        ref_labels.update(refs)

        hyp_labels.update(hyps)

        for ref in refs:

            for hyp in hyps:

                weights[ref, hyp] = weights.get((ref, hyp), 0.0) + duration

        cells.append((refs, hyps, duration))

    mapping = best_mapping(sorted(ref_labels), sorted(hyp_labels), weights)

    missed = false_alarm = confusion = reference_time = overlap_time = 0.0

    for refs, hyps, duration in cells:

        mapped = {mapping.get(label) for label in hyps}

        matched = len(refs & mapped)

        missed += max(len(refs) - len(hyps), 0) * duration

        false_alarm += max(len(hyps) - len(refs), 0) * duration

        confusion += (min(len(refs), len(hyps)) - matched) * duration

        reference_time += len(refs) * duration

        overlap_time += (len(refs) > 1) * duration

    if reference_time == 0:

        raise ValueError("no annotated speech in scoring region")

    return {"collar_seconds": 0, "score_overlap": True, "region": [start, end],

            "reference_speaker_seconds": reference_time, "overlap_seconds": overlap_time,

            "missed_seconds": missed, "false_alarm_seconds": false_alarm,

            "confusion_seconds": confusion, "der": (missed + false_alarm + confusion) / reference_time,

            "mapping_hypothesis_to_reference": mapping}

def attribution(reference, hypothesis, mapping, start_ms, end_ms):

    """Equal units from a frozen ASR, annotated independently with one voice or null.

    This measures assignment, excluding ASR recognition correctness. Units are words

    only when reliable word times exist; otherwise indivisible ASR segments.

    """

    if len(reference) != len(hypothesis) or not reference:

        raise ValueError("frozen ASR units must be nonempty and identical")

    counts = {"correct": 0, "wrong": 0, "unknown": 0, "reference_ambiguous": 0, "outside_region": 0}

    for ref, hyp in zip(reference, hypothesis):

        if any(ref.get(key) != hyp.get(key) for key in ("start_ms", "end_ms", "text")):

            raise ValueError("ASR units differ: do not compare different recognition outputs")

        if not (0 <= ref["start_ms"] < ref["end_ms"] and isinstance(ref["text"], str) and ref["text"].strip()):

            raise ValueError("invalid frozen ASR unit")

        if ref["start_ms"] < start_ms or ref["end_ms"] > end_ms:

            counts["outside_region"] += 1

            continue

        if ref.get("speaker") is None:

            counts["reference_ambiguous"] += 1

        elif hyp.get("speaker") is None:

            counts["unknown"] += 1

        elif mapping.get(str(hyp["speaker"])) == str(ref["speaker"]):

            counts["correct"] += 1

        else:

            counts["wrong"] += 1

    denominator = counts["correct"] + counts["wrong"] + counts["unknown"]

    return dict(counts, region_ms=[start_ms, end_ms], evaluated_units=denominator, assignment_error_rate=(counts["wrong"] + counts["unknown"]) / denominator if denominator else None,

                asr_word_error_rate=None)

def main():

    parser = argparse.ArgumentParser(description=__doc__)

    parser.add_argument("reference_rttm")

    parser.add_argument("hypothesis_rttm")

    parser.add_argument("--recording", required=True)

    parser.add_argument("--start", type=float, required=True)

    parser.add_argument("--end", type=float, required=True)

    parser.add_argument("--reference-units")

    parser.add_argument("--hypothesis-units")

    args = parser.parse_args()

    report = der(read_rttm(args.reference_rttm, args.recording), read_rttm(args.hypothesis_rttm, args.recording), args.start, args.end)

    if bool(args.reference_units) != bool(args.hypothesis_units):

        parser.error("provide both unit files")

    if args.reference_units:

        report["text_attribution"] = attribution(json.loads(Path(args.reference_units).read_text(encoding="utf-8")), json.loads(Path(args.hypothesis_units).read_text(encoding="utf-8")), report["mapping_hypothesis_to_reference"], args.start * 1000, args.end * 1000)

    print(json.dumps(report, ensure_ascii=False, indent=2))

if __name__ == "__main__":

    main()