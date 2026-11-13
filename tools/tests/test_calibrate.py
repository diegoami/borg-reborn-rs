"""Tests for tools/calibrate.py."""

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('calibrate', HERE.parent / 'calibrate.py')
cal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cal)


def state_line(turn, hp, mhp=20):
    return f'{turn} 1,1 d1/1 hp{hp}/{mhp} sp0/0 f5000 au0 xp0 lv1 t- o- p0 e0 r0\n'


def source(name, dist=1, speed=110, awake=True, group=1, ranged=False):
    return {'index': 1, 'race': 1, 'name': name, 'dist': dist, 'speed': speed,
            'awake': awake, 'in_view': True, 'confused': False, 'stunned': False,
            'group': group, 'confidence': 1.0, 'rounds': 1.0, 'expected': 2.0,
            'worst': 4.0, 'ranged': ranged}


class CalibrateTest(unittest.TestCase):
    def make_run(self, thinks, states, summary, messages=''):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        run = Path(tmp.name) / 'run-a'
        run.mkdir()
        (run / 'trace.jsonl').write_text(''.join(json.dumps(t) + '\n' for t in thinks))
        (run / 'state.log').write_text(''.join(state_line(t, hp) for t, hp in states))
        (run / 'summary.json').write_text(json.dumps(summary))
        (run / 'messages.log').write_text(messages)
        return run

    def think(self, turn, expected=0.0, worst=0.0, p_death=0.0, sources=(), choice='Hold'):
        return {'turn': turn, 'choice': choice, 'threat': {
            'horizon': 1, 'expected': expected, 'worst': worst, 'p_death': p_death,
            'disabling': 0.0, 'summoning': 0.0, 'economic': 0.0,
            'sources': list(sources)}}

    def test_damage_is_the_sum_of_drops_in_the_window(self):
        run = self.make_run(
            [self.think(10, 2.0, 4.0, sources=[source('a jackal')]), self.think(20)],
            [(10, 20), (15, 17), (18, 19), (20, 18)],
            {'game_turns': 30, 'dead': False, 'hp': 18},
            messages='1 15 39 The jackal bites you.\n')
        ws = cal.windows(run)
        self.assertEqual(ws[0]['actual'], 3 + 1)  # 20->17 and 19->18; healing not netted
        self.assertEqual(ws[0]['hp'], 20)
        self.assertEqual(ws[0]['messages'], ['The jackal bites you.'])
        self.assertEqual(ws[1]['actual'], 0)

    def test_windows_split_by_key_not_by_turn(self):
        # Fang acts at turn 20 before the second think's key (it had more
        # energy, game-world.c:run_game_loop): its message carries key 2,
        # the first window's last key, and its damage shows at key 3.
        run = self.make_run(
            [self.think(10, 2.0, 4.0, sources=[source('Fang')], choice='Attack(1)'),
             self.think(20, choice='Attack(1)')],
            [(10, 20), (12, 20), (20, 17), (20, 17)],
            {'game_turns': 30, 'dead': False, 'hp': 17},
            messages='1 10 3 You miss Fang.\n2 20 39 Fang bites you.\n'
                     '3 20 3 You hit Fang.\n')
        ws = cal.windows(run)
        self.assertEqual(ws[0]['messages'], ['You miss Fang.', 'Fang bites you.'])
        self.assertEqual(ws[0]['actual'], 3)
        self.assertEqual(ws[1]['messages'], ['You hit Fang.'])
        self.assertEqual(ws[1]['actual'], 0)

    def test_the_last_think_at_a_turn_is_the_window(self):
        run = self.make_run(
            [self.think(10, 9.0, 9.0), self.think(10, 1.0, 2.0, choice='Attack(1)'),
             self.think(20)],
            [(10, 20), (20, 20)], {'game_turns': 20, 'dead': False, 'hp': 20})
        ws = cal.windows(run)
        self.assertEqual(len(ws), 2)
        self.assertEqual(ws[0]['expected'], 1.0)
        self.assertEqual(ws[0]['kind'], 'Attack')

    def test_a_death_adds_the_final_drop(self):
        run = self.make_run(
            [self.think(10, 5.0, 8.0, 0.6, [source('Fang')])],
            [(10, 6)], {'game_turns': 12, 'dead': True, 'hp': -2})
        ws = cal.windows(run)
        self.assertTrue(ws[0]['died'])
        self.assertEqual(ws[0]['actual'], 8)
        rep = cal.analyse(ws)
        self.assertEqual(rep['deaths'], 1)
        self.assertEqual(rep['threatened']['over_worst'], 0)
        self.assertEqual(rep['by_monster']['Fang']['windows'], 1)

    def test_breakdowns_and_unattributed_damage(self):
        run = self.make_run(
            [self.think(10, 2.0, 4.0, sources=[source('a cave spider', dist=2, speed=120,
                                                         group=3)]),
             self.think(20, 2.0, 4.0, sources=[source('a grey mold', awake=False)]),
             self.think(30), self.think(40)],
            [(10, 20), (20, 14), (30, 14), (35, 13), (40, 13)],
            {'game_turns': 50, 'dead': False, 'hp': 13})
        rep = cal.analyse(cal.windows(run))
        self.assertEqual(rep['threatened']['windows'], 2)
        self.assertEqual(rep['threatened']['over_worst'], 1)  # 6 > 4
        self.assertIn('2-3', rep['by_distance'])
        self.assertIn('fast (>110)', rep['by_speed'])
        self.assertIn('asleep', rep['by_status'])
        self.assertEqual(rep['by_group']['same-race group']['windows'], 1)
        self.assertEqual(rep['quiet']['with_damage'], 1)
        self.assertEqual(rep['outliers_over_worst'][0]['run'], 'run-a')
        self.assertIn('Brier', cal.markdown(rep))

    def test_old_traces_without_sources_still_count(self):
        run = self.make_run(
            [{'turn': 10, 'choice': 'Hold', 'threat': {'expected': 1.0, 'worst': 2.0,
                                                        'p_death': 0.0}}],
            [(10, 20)], {'game_turns': 20, 'dead': False, 'hp': 20})
        rep = cal.analyse(cal.windows(run))
        self.assertEqual(rep['by_sources']['0 (no detail)']['windows'], 1)
        self.assertIsNone(rep['single_source'])

    def test_blows_are_counted_from_the_engine_messages(self):
        run = self.make_run(
            [self.think(10, 2.0, 4.0, sources=[source('jackal')], choice='Attack(1)'),
             self.think(20, 2.0, 4.0, sources=[source('jackal')], choice='Walk(1)'),
             self.think(30)],
            [(10, 20), (15, 17), (20, 17), (25, 15), (30, 15)],
            {'game_turns': 40, 'dead': False, 'hp': 15},
            messages='1 15 39 The jackal bites you.\n1 15 39 The jackal misses you.\n'
                     '2 15 39 The cave spider bites you.\n3 25 39 The jackal bites you.\n')
        rep = cal.analyse(cal.windows(run), blows={'jackal': [('BITE', 'HURT', 2.0),
                                                              ('BITE', 'HURT', 2.0)]})
        d = rep['blows']['jackal']
        # Only the standing window counts: the Walk may end elsewhere.
        self.assertEqual((d['windows'], d['hits'], d['misses']), (1, 1, 1))
        self.assertEqual(d['predicted_blows'], 2.0)  # 1 round x 2 blows
        self.assertEqual((d['blows_ratio'], d['hit_rate'], d['damage_per_hit']), (1.0, 0.5, 3.0))
        self.assertEqual(rep['standing']['windows'], 1)
        self.assertIn('Blows:', cal.markdown(rep))

    def test_only_engine_blow_phrases_are_blows(self):
        # #165: a spell line names the character but is not a melee blow.
        acts = ('hits you', 'bites you')
        msgs = ['The apprentice points at you and curses.',
                'The apprentice hits you.', 'The apprentice misses you.',
                'The apprentice blinks.']
        self.assertEqual(cal.blow_messages('apprentice', msgs, acts), (1, 1))
        self.assertIn('bites you', cal.BLOW_ACTS)
        self.assertNotIn('points at you', ' '.join(cal.BLOW_ACTS))

    def test_disabling_is_set_against_onset_messages(self):
        eye = dict(source('floating eye'), expected=0.0, worst=0.0, disabling=0.6)
        t = self.think(10, sources=[eye], choice='Rest(75)')
        t['threat']['disabling'] = 0.6
        run = self.make_run([t, self.think(50)], [(10, 20), (50, 18)],
                            {'game_turns': 60, 'dead': False, 'hp': 18},
                            messages='1 12 39 The floating eye gazes at you.\n'
                                     '1 12 0 You are paralysed!\n')
        rep = cal.analyse(cal.windows(run), blows={})
        top = [b for b in rep['disabling'] if b['range'][0] == 0.5][0]
        self.assertEqual((top['windows'], top['disabled']), (1, 1))

    def test_the_bootstrap_is_deterministic(self):
        ws = [{'expected': 1.0, 'actual': a, 'worst': 2.0} for a in (0, 1, 2, 0, 3)]
        import random
        self.assertEqual(cal.bootstrap(ws, random.Random(0)),
                         cal.bootstrap(ws, random.Random(0)))


if __name__ == '__main__':
    unittest.main()
