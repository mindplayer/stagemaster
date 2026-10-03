import unittest
from capacity_report import analyze
from test_runtime_frame_report import log, valid


def board():
    return log(valid()) + '\nOUTPUT WATCHDOG armed reset_ms=1000\nUART LOGIC ONLY phase=Ready quiet=false submitted=1481 completed=1480 fault=None; RS485=disabled'


def controls(count=4):
    return '\n'.join([*(f'CAPACITY PROGRAM index={i} passed=true' for i in range(count)),
                      'CAPACITY OFFLINE passed=true', f'PASS：容量验收 {count} 节目；停止归还'])


class Reports(unittest.TestCase):
    def stopped_board(self, seconds=180, advancing=True, extra_command=False):
        samples = valid()
        for at, frames, commands in ((38, 1520, 0), (38 + seconds, 1520 + 40 * seconds if advancing else 1520, int(extra_command))):
            samples.append(dict(at_us=at * 1_000_000, attempts=frames, frames=frames,
                                running_frames=1240, offline_frames=1240, instance=0,
                                running=0, elapsed_ms=0, commands=commands))
        return board().replace(log(valid()), log(samples))

    def test_post_stop_observation_requires_continuing_frames_without_commands(self):
        result = analyze(self.stopped_board(), controls(), 4, 180)
        self.assertTrue(result['passed'], result)
        self.assertEqual(result['post_stop_seconds'], 180)
        for text in (board(), self.stopped_board(seconds=179),
                     self.stopped_board(advancing=False), self.stopped_board(extra_command=True)):
            self.assertFalse(analyze(text, controls(), 4, 180)['passed'])

    def test_complete_control_and_output_evidence_pass_both_corpus_sizes(self):
        for count in (4, 64):
            result = analyze(board(), controls(count), count)
            self.assertTrue(result['passed'], result)
            self.assertEqual(result['verified_program_controls'], count)
            self.assertFalse(result['physical_output_verified'])
            self.assertFalse(result['full_stack_high_water_verified'])

    def test_control_pass_alone_never_hides_late_frames_or_output_faults(self):
        for text in (board().replace('max_gap_us=25000', 'max_gap_us=50001'),
                     board().replace('fault=None', 'fault=Some(Driver)'),
                     board().replace('OUTPUT WATCHDOG armed', 'NOT ARMED'),
                     board() + '\nOUTPUT WATCHDOG fault=Deadline',
                     board() + '\nDetected a write to the stack guard value',
                     board() + '\nESP-ROM:reset'):
            self.assertFalse(analyze(text, controls(), 4)['passed'])

    def test_missing_duplicate_or_unfinished_program_controls_fail(self):
        for text in (controls().replace('index=2', 'index=1'), controls(3),
                     controls().replace('CAPACITY OFFLINE passed=true', ''),
                     controls().split('PASS：')[0]):
            self.assertFalse(analyze(board(), text, 4)['passed'])


if __name__ == '__main__':
    unittest.main()
