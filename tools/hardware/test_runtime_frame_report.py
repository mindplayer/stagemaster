"""Protect the acceptance gates against incomplete or misleading telemetry."""
import unittest
from runtime_frame_report import analyze, COUNTERS, FIELDS


def log(samples):
    lines = ['ESP-ROM:esp32s3-20210327', 'heap=51000/80072 heap_peak=54000']
    for sample in samples:
        values = dict.fromkeys(FIELDS, 0)
        values.update(v=1, min_gap_us=25000, max_gap_us=25000, max_work_us=150,
                      instance=1, running=1)
        values.update(sample)
        lines.append('RUNTIME_SAMPLE ' + ' '.join(f'{k}={v}' for k, v in values.items()))
    return '\n'.join(lines)


def valid():
    return [
        dict(at_us=1_000_000, attempts=40, frames=40, running_frames=40, offline_frames=40, elapsed_ms=1000),
        dict(at_us=31_000_000, attempts=1240, frames=1240, running_frames=1240, offline_frames=1240, elapsed_ms=31000),
        dict(at_us=32_000_000, attempts=1280, frames=1280, running_frames=1240, offline_frames=1240, running=0, elapsed_ms=31000),
        dict(at_us=37_000_000, attempts=1480, frames=1480, running_frames=1240, offline_frames=1240, running=0, elapsed_ms=31000),
    ]


class Reports(unittest.TestCase):
    def test_actual_offline_frames_and_pause_are_required(self):
        result = analyze(log(valid()))
        self.assertTrue(result['passed'], result)
        self.assertEqual(result['offline_frame_hz'], 40)
        self.assertFalse(result['physical_output_verified'])
        self.assertEqual(result['heap_peak_bytes'], 54000)

    def test_diagnostic_ticks_alone_are_not_frame_evidence(self):
        self.assertFalse(analyze('ESP-ROM:x\nLIVE ticks=999999 elapsed_ms=90000')['passed'])

    def test_step_clock_may_wrap_but_frozen_progress_fails(self):
        samples = valid()
        samples[0]['elapsed_ms'] = 1500
        samples[1]['elapsed_ms'] = 500
        self.assertTrue(analyze(log(samples))['passed'])
        samples[1]['elapsed_ms'] = 1500
        self.assertFalse(analyze(log(samples))['passed'])

    def test_other_instances_errors_late_gaps_and_new_commands_fail(self):
        for field, value in [('instance', 2), ('errors', 1), ('clock_errors', 1),
                             ('max_gap_us', 50001), ('commands', 1), ('online', 1)]:
            with self.subTest(field=field):
                samples = valid()
                samples[1][field] = value
                self.assertFalse(analyze(log(samples))['passed'])

    def test_no_pause_or_only_duplicate_reports_cannot_pass(self):
        self.assertFalse(analyze(log(valid()[:2]))['passed'])
        self.assertFalse(analyze(log([valid()[0]] * 20))['passed'])
        self.assertFalse(analyze(log(valid()) + '\nESP-ROM:newboot')['passed'])

    def test_all_cumulative_counters_and_time_must_not_regress(self):
        for field in (*COUNTERS, 'at_us'):
            with self.subTest(field=field):
                samples = valid()
                samples[0][field] = 2_000_000_000
                self.assertFalse(analyze(log(samples))['passed'])


if __name__ == '__main__':
    unittest.main()
