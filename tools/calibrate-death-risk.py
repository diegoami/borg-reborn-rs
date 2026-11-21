#!/usr/bin/env python3
"""Score #790 death-risk forecasts on extracted Zoo cells.

Pass one or more roots containing summary.json and trace-core.jsonl (for
example, the extracted raw shards). Legacy @1/@2 cells support F1. Instrumented
@3 cells also support F2/F3 and a held-out F4 fit (pip install scikit-learn).
By default, compare all four on the same held-out starts when fitting.
"""

import argparse
from array import array
from bisect import bisect_left, bisect_right
from collections import Counter
from dataclasses import dataclass
from hashlib import sha256
import json
from math import isfinite, log, nan
from pathlib import Path
import sys


V2_SCHEMA = "borg-reborn-rs/core-trace@2"
LEGACY_SCHEMAS = {"borg-reborn-rs/core-trace@1", V2_SCHEMA}
RISK_SCHEMA = "borg-reborn-rs/core-trace@3"
FORMULAS = ("f1_danger_hp", "f2_time_to_die", "f3_race_escapes")
FITTED = "f4_fitted"
THRESHOLDS = (10, 25, 50, 75, 90)
FEATURES = (
    "hp_fraction", "incoming_over_hp", "hurt_hp_per_action", "fallback_danger",
    "adjacent_monsters", "fastest_speed_delta",
    "phase_scrolls", "phase_usable", "other_escape_held", "cure_potions",
    "cure_usable", "stairs_distance", "stairs_usable", "clevel", "depth",
    "confused", "blind", "poisoned", "paralyzed",
)
# Retain the absence separately from a measured distance or speed delta.
NULLABLE_FEATURES = ("fastest_speed_delta", "stairs_distance")


@dataclass
class Cell:
    start: tuple
    end: int
    dead: bool
    turns: array
    stop_eligible: bytearray
    scores: dict
    features: array

    def label_ranges(self, horizon):
        """Return the first positive row and the end of observable labels."""
        if self.dead:
            return bisect_left(self.turns, self.end - horizon), len(self.turns)
        return len(self.turns), bisect_right(self.turns, self.end - horizon)


def read_cell(summary, path, include_features, *, draft_v2_risk=False):
    trace = path.with_name("trace-core.jsonl")
    if not trace.is_file():
        return None
    cell = Cell(
        (summary["seed"], summary["race_idx"], summary["class_idx"]),
        summary["game_turns"], summary["dead"], array("q"), bytearray(),
        {name: array("d") for name in FORMULAS}, array("f"),
    )
    schema = None
    with trace.open() as lines:
        for number, line in enumerate(lines, 1):
            try:
                row = json.loads(line)
                turn = row["turn"]
                if cell.turns and turn < cell.turns[-1]:
                    raise ValueError("game turn decreased within the same cell")
                if turn > cell.end:
                    raise ValueError("think follows the summary's final game turn")
                row_schema = row["schema"]
                if row_schema != RISK_SCHEMA and row_schema not in LEGACY_SCHEMAS:
                    raise ValueError(f"unsupported trace schema {row_schema!r}")
                if schema is not None and row_schema != schema:
                    raise ValueError("mixed trace schemas within the same cell")
                schema = row_schema
                instrumented = schema == RISK_SCHEMA or (schema == V2_SCHEMA and draft_v2_risk)
                risk = row["death_risk"] if instrumented else None
                if not instrumented:
                    if "death_risk" in row:
                        raise ValueError("death_risk requires core-trace@3 or --draft-v2-risk for the draft pilot")
                    f1 = min(100.0, max(0.0, 100.0 * row["danger"] / max(1, row["hp"])))
                    cell.scores[FORMULAS[0]].append(f1)
                else:
                    for name in FORMULAS:
                        percent = float(risk[name])
                        if not 0 <= percent <= 100:
                            raise ValueError(f"{name} outside 0–100")
                        cell.scores[name].append(percent)
                    if include_features:
                        features = risk["features"]
                        for name in FEATURES:
                            value = features[name]
                            if value is not None and not isfinite(value):
                                raise ValueError(f"non-finite feature {name}")
                            cell.features.append(nan if value is None else value)
                        cell.features.extend(float(features[name] is None) for name in NULLABLE_FEATURES)
                cell.turns.append(turn)
                cell.stop_eligible.append(risk["stop_eligible"] if risk is not None else row["depth"] > 0)
            except (KeyError, TypeError, ValueError) as exc:
                raise ValueError(f"{trace}:{number}: {exc}") from exc
    return cell if cell.turns else None


def load_cells(roots, include_features, *, draft_v2_risk=False):
    summaries = {path.resolve() for root in roots for path in root.rglob("summary.json")}
    traces = {path.resolve() for root in roots for path in root.rglob("trace-core.jsonl")}
    cells = []
    excluded = Counter()
    excluded["trace without summary"] = len(traces - {p.with_name("trace-core.jsonl") for p in summaries})
    for path in sorted(summaries):
        summary = json.loads(path.read_text())
        if summary["stop_reason"] == "death" and summary["dead"]:
            pass
        elif summary["stop_reason"] == "turns" and not summary["dead"]:
            pass
        else:
            excluded[f"summary outcome {summary['stop_reason']} (dead={summary['dead']})"] += 1
            continue
        cell = read_cell(summary, path, include_features, draft_v2_risk=draft_v2_risk)
        if cell is None:
            excluded["missing or empty trace"] += 1
        else:
            cells.append(cell)
    return cells, excluded


def start_bucket(start):
    """Use the same split for every arm and pilot that shares a birth."""
    return sha256(repr(start).encode()).digest()[0] % 5


def fit_risk(cells, horizon):
    """Fit on four birth buckets and predict only the fifth."""
    train = [cell for cell in cells if start_bucket(cell.start) != 0]
    test = [cell for cell in cells if start_bucket(cell.start) == 0]
    if not test or not any(cell.dead for cell in train) or not any(not cell.dead for cell in train):
        return {}, "need held-out starts and both outcomes among training starts"
    try:
        import numpy as np
        from sklearn.impute import SimpleImputer
        from sklearn.linear_model import LogisticRegression
        from sklearn.pipeline import make_pipeline
        from sklearn.preprocessing import StandardScaler
    except ImportError as exc:
        raise ValueError("F4 requires scikit-learn (or pass --no-fit)") from exc

    def matrix(cell):
        return np.frombuffer(cell.features, dtype=np.float32).reshape(
            len(cell.turns), len(FEATURES) + len(NULLABLE_FEATURES),
        )

    matrices, labels = [], []
    for cell in train:
        positive, end = cell.label_ranges(horizon)
        matrices.append(matrix(cell)[:end])
        label = np.zeros(end, dtype=np.uint8)
        label[positive:] = 1
        labels.append(label)
    y = np.concatenate(labels)
    if len(y) == 0 or y.min() == y.max():
        return {}, "training labels have only one class after censoring"
    model = make_pipeline(
        SimpleImputer(strategy="median", keep_empty_features=True),
        StandardScaler(),
        LogisticRegression(max_iter=1000),
    )
    model.fit(np.concatenate(matrices), y)
    predictions = {
        id(cell): array("d", model.predict_proba(matrix(cell))[:, 1] * 100)
        for cell in test
    }
    return predictions, f"{len(train)} training cells, {len(test)} held-out cells"


def score(cells, formula, horizon, predictions):
    positives, negatives = Counter(), Counter()
    caught_any = [0] * len(THRESHOLDS)
    caught_first = [0] * len(THRESHOLDS)
    false_stops = [0] * len(THRESHOLDS)
    deaths = survivors = 0
    for cell in cells:
        values = predictions[id(cell)] if formula == FITTED else cell.scores[formula]
        positive, end = cell.label_ranges(horizon)
        negatives.update(values[:min(positive, end)])
        positives.update(values[positive:end])
        # A real stop ends the game at the first eligible crossing. Also
        # report whether any crossing occurred within the window, regardless
        # of premature crossings. Collapse repeated crossings in a survivor.
        first = [None] * len(THRESHOLDS)
        recent_max = 0.0
        for turn, value, eligible in zip(cell.turns, values, cell.stop_eligible):
            if not eligible or value < THRESHOLDS[0]:
                continue
            for i, threshold in enumerate(THRESHOLDS):
                if value < threshold:
                    break
                if first[i] is None:
                    first[i] = turn
            if turn >= cell.end - horizon:
                recent_max = max(recent_max, value)
        if cell.dead:
            deaths += 1
            for i, (turn, threshold) in enumerate(zip(first, THRESHOLDS)):
                caught_any[i] += recent_max >= threshold
                caught_first[i] += turn is not None and turn >= cell.end - horizon
        else:
            survivors += 1
            for i, turn in enumerate(first):
                false_stops[i] += turn is not None

    n_pos, n_neg = positives.total(), negatives.total()
    n = n_pos + n_neg
    brier = log_loss = pairs = negatives_so_far = 0.0
    bins = [[0, 0.0, 0] for _ in range(10)]
    for value in sorted(positives.keys() | negatives.keys()):
        pos, neg = positives[value], negatives[value]
        p = value / 100.0
        brier += pos * (1 - p) ** 2 + neg * p ** 2
        clipped = min(1 - 1e-15, max(1e-15, p))
        log_loss -= pos * log(clipped) + neg * log(1 - clipped)
        pairs += pos * (negatives_so_far + neg / 2)
        negatives_so_far += neg
        bin_ = bins[min(9, int(value / 10))]
        bin_[0] += pos + neg
        bin_[1] += value * (pos + neg)
        bin_[2] += pos

    ratio = lambda a, b: a / b if b else None
    return {
        "cells": len(cells), "deaths": deaths, "survivors": survivors,
        "thinks": n, "positives": n_pos,
        "brier": ratio(brier, n),
        "log_loss": ratio(log_loss, n),
        "auc": ratio(pairs, n_pos * n_neg),
        "reliability": [
            {"decile": i, "n": count, "predicted_percent": ratio(total, count),
             "observed_percent": ratio(100 * pos, count)}
            for i, (count, total, pos) in enumerate(bins)
        ],
        "stop_tradeoff": [
            {"threshold": t, "deaths_caught_any": any_, "deaths_caught_first": first_,
             "death_share_any": ratio(any_, deaths), "death_share_first": ratio(first_, deaths),
             "false_stops": false, "stops_per_surviving_game": ratio(false, survivors)}
            for t, any_, first_, false in zip(THRESHOLDS, caught_any, caught_first, false_stops)
        ],
    }


def evaluate(cells, horizons, fit):
    rich = [cell for cell in cells if cell.scores[FORMULAS[1]]]
    results = {}
    for horizon in horizons:
        scores = {}
        predictions, fit_note = {}, "disabled" if not fit else "no instrumented traces"
        if rich and fit:
            predictions, fit_note = fit_risk(rich, horizon)
        if predictions:
            held_out = [cell for cell in rich if id(cell) in predictions]
            scores = {name: score(held_out, name, horizon, predictions) for name in (*FORMULAS, FITTED)}
        elif rich:
            scores = {name: score(rich, name, horizon, {}) for name in FORMULAS}
        else:
            scores = {FORMULAS[0]: score(cells, FORMULAS[0], horizon, {})}
        legacy = [cell for cell in cells if not cell.scores[FORMULAS[1]]]
        if rich and legacy:
            scores["f1_legacy"] = score(legacy, FORMULAS[0], horizon, {})
        results[horizon] = {"fit": fit_note, "formulas": scores}
    return results


def print_report(report):
    def fmt(value):
        return f"{value:.6g}" if value is not None else "n/a"

    print(f"{report['cells']} usable cells, {report['instrumented']} instrumented")
    for reason, count in report["excluded"].items():
        if count:
            print(f"Excluded {count} with {reason}")
    for horizon, result in report["horizons"].items():
        print(f"\nHorizon {horizon} game turns. F4 fit {result['fit']}.")
        for name, metric in result["formulas"].items():
            print(f"\n{name}  {metric['cells']} cells  {metric['thinks']} labeled thinks"
                  f"  {metric['positives']} positive  {metric['deaths']} deaths  {metric['survivors']} survivors")
            print("  ".join(f"{key} {fmt(metric[key])}" for key in ("brier", "log_loss", "auc")))
            print("decile       thinks  predicted %   observed %")
            for row in metric["reliability"]:
                d = row["decile"]
                print(f"{d * 10:2}–{(d + 1) * 10:<3} {row['n']:>10}"
                      f" {fmt(row['predicted_percent']):>12} {fmt(row['observed_percent']):>12}")
            print("threshold  deaths any  deaths first  first share  any share  false stops  stops/survivor")
            for row in metric["stop_tradeoff"]:
                print(f"{row['threshold']:>9} {row['deaths_caught_any']:>11} {row['deaths_caught_first']:>13}"
                      f" {fmt(row['death_share_first']):>12} {fmt(row['death_share_any']):>10}"
                      f" {row['false_stops']:>12} {fmt(row['stops_per_surviving_game']):>15}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("roots", type=Path, nargs="+", help="directories containing extracted Zoo cells")
    parser.add_argument("--horizons", type=int, nargs="+", default=(100, 300, 1000))
    parser.add_argument("--draft-v2-risk", action="store_true",
                        help="read instrumented @2 rows from the original Zoo #534 draft pilot only")
    parser.add_argument("--no-fit", action="store_true", help="score F1–F3 over all available cells")
    parser.add_argument("--json", action="store_true", help="print machine-readable metrics")
    args = parser.parse_args()
    if any(t < 1 for t in args.horizons):
        parser.error("horizons must be positive engine turns")
    if any(not root.is_dir() for root in args.roots):
        parser.error("each root must be a directory")
    try:
        cells, excluded = load_cells(args.roots, not args.no_fit, draft_v2_risk=args.draft_v2_risk)
        if not cells:
            raise ValueError(f"no scoreable cells ({dict(excluded)})")
        report = {
            "cells": len(cells),
            "instrumented": sum(bool(cell.scores[FORMULAS[1]]) for cell in cells),
            "excluded": dict(excluded),
            "horizons": evaluate(cells, args.horizons, not args.no_fit),
        }
    except (KeyError, OSError, ValueError) as exc:
        print(f"{parser.prog}: {exc}", file=sys.stderr)
        return 1
    if args.json:
        print(json.dumps(report, indent=2, allow_nan=False))
    else:
        print_report(report)
    return 0


if __name__ == "__main__":
    sys.exit(main())
