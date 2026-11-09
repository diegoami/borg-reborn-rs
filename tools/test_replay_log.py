# Adapted from better-borg-c 85b6ba8e3748e061db1ff1206163c27272d2de31; GPL-2.0-or-later or Angband licence.
"""Recorded log visibility, delta reconstruction and same-turn precision."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('log_viewer', ROOT / 'tools/borg_replay_viewer.py')
viewer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(viewer)


def frame(index, turn, message, full=False):
    return {'index': index, 'turn': turn, 'phase': 'input', 'full': full,
            'width': 80, 'height': 2, 'hp': 20, 'sp': 2, 'level': 4, 'depth': 1,
            'cells': [[i, ord(c), 1] for i, c in enumerate(message.ljust(80))], 'palette': None}


class ReplayLogs(unittest.TestCase):
    def fixture(self, directory, frames, trace=''):
        (directory / 'frames.jsonl').write_text(''.join(json.dumps(f) + '\n' for f in frames))
        (directory / 'trace.txt').write_text(trace)
        movie = viewer.Frames(directory)
        self.addCleanup(movie.close)
        return movie, viewer.ReplayLog(directory, movie)

    def text(self, rows):
        return '\n'.join(t for t, _ in rows)

    def test_same_turn_future_notes_and_messages_are_not_exposed(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)
            trace = ('BORG 10 # Attacking with weapon Rapier\nBORG 10 # Casting Phase Door\n'
                     'BORG 11 # Quaffing unknown potion\nMESSAGE FUTURE UNTImED BUFFER\n')
            frames, log = self.fixture(p, [frame(1,10,'Before',True),
                frame(2,10,'You cast Phase Door'),frame(3,11,'New level')], trace)
            first = frames.get(0)[0]
            for mode in [0,1]:
                rows,_=log.window(first,0,20,80,mode=mode)
                self.assertNotIn('Casting Phase Door',self.text(rows))
                self.assertIn('completed turn 9',self.text(rows))
            rows,_=log.window(first,0,20,80,mode=2)
            self.assertIn('Before',self.text(rows))
            self.assertNotIn('You cast',self.text(rows))
            later = frames.get(1)[0]
            rows,_=log.window(later,1,20,80,mode=2)
            self.assertIn('You cast Phase Door',self.text(rows))
            self.assertNotIn('New level',self.text(rows))
            last = frames.get(2)[0]
            rows,_=log.window(last,2,20,80)
            self.assertIn('Casting Phase Door',self.text(rows))
            self.assertNotIn('Quaffing',self.text(rows))
            self.assertNotIn('UNTImED',self.text(rows))
            (p/'record-complete.json').write_text('{}')
            completed=viewer.ReplayLog(p,frames)
            rows,_=completed.window(last,2,20,80)
            self.assertIn('Quaffing unknown potion',self.text(rows))

    def test_action_filter_and_all_notes_have_distinct_scopes(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)
            movie,log=self.fixture(p,[frame(1,2,'You miss',True)],
                'BORG 1 # Tracking a white jelly\nBORG 1 # Attacking with weapon Rapier\n')
            state=movie.get(0)[0]
            actions,_=log.window(state,0,15,80,mode=0)
            all_notes,_=log.window(state,0,15,80,mode=1)
            self.assertIn('Attacking',self.text(actions))
            self.assertNotIn('Tracking',self.text(actions))
            self.assertIn('Tracking',self.text(all_notes))

    def test_message_changes_reconstruct_deltas_and_repeated_after_clear(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)
            movie,log=self.fixture(p,[frame(1,1,'You hit',True),frame(2,2,'You hit'),
                frame(3,3,''),frame(4,4,'You hit')])
            self.assertEqual([(r[0],r[3]) for r in movie.messages],[(0,'You hit'),(3,'You hit')])
            rows,_=log.window(movie.get(3)[0],3,20,80,mode=2)
            self.assertIn('1 F1 input: You hit',self.text(rows))
            self.assertIn('4 F4 input: You hit',self.text(rows))

    def test_history_window_is_bounded_and_can_reach_oldest_entry(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)
            trace=''.join(f'BORG {t} # Casting spell {t}\n' for t in range(1,1000))
            movie,log=self.fixture(p,[frame(1,1000,'Done',True)],trace)
            state=movie.get(0)[0]
            latest,_=log.window(state,0,10,45)
            self.assertLessEqual(len(latest),10)
            self.assertIn('Casting spell 999',self.text(latest))
            oldest,back=log.window(state,0,10,45,back=10000)
            self.assertEqual(back,998)
            self.assertIn('Casting spell 1',self.text(oldest))
            self.assertNotIn('Casting spell 999',self.text(oldest))


if __name__ == '__main__':
    unittest.main()
