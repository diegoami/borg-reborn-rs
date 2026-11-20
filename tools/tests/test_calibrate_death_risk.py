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
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            summary = dict(seed=1, race_idx=2, class_idx=3, game_turns=100, dead=False, stop_reason="turns")
            (root / "summary.json").write_text(json.dumps(summary))
            rows = [dict(turn=1, hp=20, danger=5, depth=0), dict(turn=10, hp=2, danger=9, depth=1)]
            (root / "trace-core.jsonl").write_text("\n".join(map(json.dumps, rows)))
            loaded, excluded = risk.load_cells([root], True)
        self.assertEqual(sum(excluded.values()), 0)
        self.assertEqual(list(loaded[0].scores[risk.FORMULAS[0]]), [25.0, 100.0])
        self.assertEqual(list(loaded[0].stop_eligible), [0, 1])
        self.assertEqual(list(loaded[0].features), [])


if __name__ == "__main__":
    unittest.main()
