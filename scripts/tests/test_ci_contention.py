import datetime
import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location('ci_contention', Path(__file__).resolve().parents[1] / 'ci-contention.py')
cc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cc)

T0 = datetime.datetime(2026, 11, 28, 12, 0, tzinfo=datetime.timezone.utc)


def job(created, waited, ran):
    c = T0 + datetime.timedelta(seconds=created)
    s = c + datetime.timedelta(seconds=waited)
    return {'created': c, 'started': s, 'completed': s + datetime.timedelta(seconds=ran), 'name': 'j', 'run': 1, 'branch': 'x'}


class ContentionTest(unittest.TestCase):
    def test_wait_and_percentiles(self):
        self.assertEqual(cc.wait(job(0, 3, 10)), 3)
        self.assertEqual(cc.percentile([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 0.9), 9)
        self.assertIsNone(cc.percentile([], 0.5))
        self.assertEqual(cc.describe([]), {'jobs': 0})

    def test_busy_counts_queued_and_running_jobs(self):
        ours = [job(0, 5, 100), job(10, 0, 20)]
        self.assertEqual(cc.busy_at(T0 + datetime.timedelta(seconds=3), ours), 1)   # first one still queued
        self.assertEqual(cc.busy_at(T0 + datetime.timedelta(seconds=15), ours), 2)
        self.assertEqual(cc.busy_at(T0 + datetime.timedelta(seconds=500), ours), 0)

    def test_zoo_waits_are_split_by_whether_our_ci_was_in_flight(self):
        ours = [job(0, 2, 60)]
        zoo = [job(30, 120, 10), job(40, 90, 10), job(1000, 1, 10)]
        result = cc.contention(ours, zoo)
        self.assertEqual(result['zoo_while_reborn_in_flight']['jobs'], 2)
        self.assertEqual(result['zoo_while_reborn_in_flight']['median_s'], 105.0)
        self.assertEqual(result['zoo_otherwise']['median_s'], 1.0)
        self.assertEqual(result['peak_reborn_jobs_in_flight'], 1)
        self.assertIn('median 105.0 s', cc.render(result, 'the last 7 days'))

    def test_delays_are_attributed_to_who_held_the_slots(self):
        # 3 slots: the zoo runs 3 long jobs (a), then a 4th zoo job waits behind its own; a 5th waits
        # when 1 zoo + 2 of ours hold the slots (b); a 6th waits with nothing of either side to blame (c)
        zoo_running = [job(0, 0, 1000) for _ in range(3)]
        behind_itself = job(10, 100, 10)
        zoo_b = [job(2000, 0, 1000), job(2005, 90, 10)]
        ours = [job(1990, 0, 1000), job(1991, 0, 1000)]
        lonely = job(5000, 60, 10)
        got = cc.attribute_delays(ours, zoo_running + [behind_itself] + zoo_b + [lonely], slots=3)
        self.assertEqual(got['self']['jobs'], 1)
        self.assertEqual(got['reborn']['jobs'], 1)
        self.assertEqual(got['other']['jobs'], 1)
        self.assertEqual(got['reborn']['total_wait_h'], 0.03)

    def test_no_zoo_jobs_is_reported_not_an_error(self):
        result = cc.contention([job(0, 1, 5)], [])
        self.assertIn('none', cc.render(result, 'the last 7 days'))


if __name__ == '__main__':
    unittest.main()
