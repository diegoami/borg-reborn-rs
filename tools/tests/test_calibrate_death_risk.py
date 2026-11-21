"""Edge cases for calibration metrics on retained Zoo cells."""
from array import array
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location(
    "calibrate_death_risk", Path(__file__).resolve().parents[1] / "calibrate-death-risk.py",
)
risk = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(risk)


def cell(turns, values, dead, eligible=None):
    return risk.Cell(
        (1, 2, 3), 1000, dead, array("q", turns),
        bytearray([1] * len(turns) if eligible is None else eligible),
        {risk.FORMULAS[0]: array("d", values)}, array("f"),
    )


class CalibrationTest(unittest.TestCase):
    def test_censoring_and_same_turn_death(self):
        dead = cell([899, 900, 1000], [0, 50, 50], True)
        survivor = cell([899, 900, 901], [0, 50, 100], False)
        output = risk.score([dead, survivor], risk.FORMULAS[0], 100, {})
        self.assertEqual(output["thinks"], 5)
        self.assertEqual(output["positives"], 2)
        self.assertAlmostEqual(output["auc"], 5 / 6)
        self.assertAlmostEqual(output["brier"], 0.15)
        self.assertEqual(output["reliability"][5]["observed_percent"], 200 / 3)

    def test_first_stop_differs_from_a_late_crossing(self):
        dead = cell([50, 100, 950], [100, 80, 95], True, [0, 1, 1])
        survivor = cell([50, 100, 950], [100, 0, 80], False, [0, 1, 1])
        report = risk.score([dead, survivor], risk.FORMULAS[0], 100, {})["stop_tradeoff"]
        self.assertEqual((report[0]["deaths_caught_first"], report[0]["deaths_caught_any"]), (0, 1))
        self.assertEqual((report[-1]["deaths_caught_first"], report[-1]["deaths_caught_any"]), (1, 1))
        self.assertEqual((report[0]["false_stops"], report[-1]["false_stops"]), (1, 0))

    def test_legacy_trace_reconstructs_the_danger_score(self):
        for schema in sorted(risk.LEGACY_SCHEMAS):
            with self.subTest(schema=schema), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                summary = dict(seed=1, race_idx=2, class_idx=3, game_turns=100, dead=False, stop_reason="turns")
                (root / "summary.json").write_text(json.dumps(summary))
                rows = [dict(schema=schema, turn=1, hp=20, danger=5, depth=0),
                        dict(schema=schema, turn=10, hp=2, danger=9, depth=1)]
                (root / "trace-core.jsonl").write_text("\n".join(map(json.dumps, rows)))
                loaded, excluded = risk.load_cells([root], True)
                self.assertEqual(sum(excluded.values()), 0)
                self.assertEqual(list(loaded[0].scores[risk.FORMULAS[0]]), [25.0, 100.0])
                self.assertEqual(list(loaded[0].stop_eligible), [0, 1])
                self.assertEqual(list(loaded[0].features), [])

    def test_v3_keeps_plain_melee_and_fallback_separate_for_f4(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            summary = dict(seed=1, race_idx=2, class_idx=3, game_turns=100, dead=True)
            scores = dict(zip(risk.FORMULAS, (30, 40, 50)))
            rows = []
            for turn, hurt, fallback in [(1, 5, 0), (2, 0, 5)]:
                features = dict.fromkeys(risk.FEATURES, 0)
                features.update(incoming_over_hp=0.5, hurt_hp_per_action=hurt, fallback_danger=fallback)
                rows.append(dict(schema=risk.RISK_SCHEMA, turn=turn,
                                 death_risk=dict(scores, features=features, stop_eligible=True)))
            (root / "trace-core.jsonl").write_text("\n".join(map(json.dumps, rows)))
            loaded = risk.read_cell(summary, root / "summary.json", True)
            width = len(risk.FEATURES) + len(risk.NULLABLE_FEATURES)
            hurt = risk.FEATURES.index("hurt_hp_per_action")
            fallback = risk.FEATURES.index("fallback_danger")
            self.assertEqual([loaded.features[hurt], loaded.features[hurt + width]], [5, 0])
            self.assertEqual([loaded.features[fallback], loaded.features[fallback + width]], [0, 5])

    def test_v3_cannot_silently_fall_back_to_legacy(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            summary = dict(seed=1, race_idx=2, class_idx=3, game_turns=100, dead=False)
            row = dict(schema=risk.RISK_SCHEMA, turn=1, hp=20, danger=5, depth=1)
            (root / "trace-core.jsonl").write_text(json.dumps(row))
            with self.assertRaisesRegex(ValueError, "trace-core.jsonl:1:.*death_risk"):
                risk.read_cell(summary, root / "summary.json", True)

    def test_draft_v2_pilot_requires_explicit_compatibility(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            summary = dict(seed=1, race_idx=2, class_idx=3, game_turns=100, dead=False)
            scores = dict(zip(risk.FORMULAS, (30, 40, 50)))
            row = dict(schema=risk.V2_SCHEMA, turn=1, death_risk=dict(scores, stop_eligible=True))
            (root / "trace-core.jsonl").write_text(json.dumps(row))
            with self.assertRaisesRegex(ValueError, "--draft-v2-risk"):
                risk.read_cell(summary, root / "summary.json", False)
            loaded = risk.read_cell(summary, root / "summary.json", False, draft_v2_risk=True)
            self.assertEqual([loaded.scores[name][0] for name in risk.FORMULAS], [30, 40, 50])


if __name__ == "__main__":
    unittest.main()
