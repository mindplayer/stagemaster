"""Capacity acceptance combines unchanged frame gates with native control and UART evidence."""
import argparse
import json
import re
from pathlib import Path
from runtime_frame_report import analyze as frame_report


def stopped_tail_seconds(board):
    """Measure the final idle interval with continuing frames and no new commands."""
    tail = []
    for line in board.splitlines():
        if not line.startswith('RUNTIME_SAMPLE '):
            continue
        try:
            fields = dict(item.split('=', 1) for item in line.split()[1:])
            sample = tuple(int(fields[k]) for k in ('at_us', 'frames', 'commands', 'instance', 'running'))
        except (ValueError, KeyError):
            tail = []
            continue  # the frame analyzer also rejects the malformed record
        if tail and sample == tail[-1]:
            continue
        if sample[3:] != (0, 0) or sample[1] == 0:
            tail = []
            continue
        if tail and (sample[0] <= tail[-1][0] or sample[1] <= tail[-1][1]
                     or sample[2] != tail[-1][2]):
            tail = []
        tail.append(sample)
    return (tail[-1][0] - tail[0][0]) / 1_000_000 if tail else 0


def analyze(board, controls, programs, post_stop_seconds=0):
    result = frame_report(board)
    failures = result['failures']
    completed = [int(n) for n in re.findall(r'^CAPACITY PROGRAM index=(\d+) passed=true$', controls, re.M)]
    if completed != list(range(programs)):
        failures.append('没有逐个完成预定节目')
    if controls.count('CAPACITY OFFLINE passed=true') != 1:
        failures.append('缺少同启动同实例断线控制验收')
    if f'PASS：容量验收 {programs} 节目；' not in controls:
        failures.append('原生验收未正常结束并归还控制')
    if board.count('OUTPUT WATCHDOG armed') != 1 or re.search(
            r'OUTPUT WATCHDOG fault|UART QUEUE fault|stack guard', board):
        failures.append('看门狗／栈／发送故障或启动证据不完整')
    uart = re.findall(r'UART LOGIC ONLY phase=(\w+) quiet=(\w+) submitted=(\d+) completed=(\d+) fault=([^;]+); RS485=disabled', board)
    if not uart or any(row[4] != 'None' for row in uart) or max((int(row[3]) for row in uart), default=0) < 100:
        failures.append('缺少实际 UART 完成或存在输出故障')
    stopped_seconds = stopped_tail_seconds(board)
    if stopped_seconds < post_stop_seconds:
        failures.append('停止节目后的持续出帧观察时间不足')
    result.update(passed=not failures, verified_program_controls=len(completed),
                  uart_max_completed=max((int(row[3]) for row in uart), default=0),
                  post_stop_seconds=stopped_seconds, required_post_stop_seconds=post_stop_seconds,
                  physical_output_verified=False, full_stack_high_water_verified=False)
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--board', type=Path, required=True)
    parser.add_argument('--controls', type=Path, required=True)
    parser.add_argument('--programs', type=int, choices=[4, 64], required=True)
    parser.add_argument('--post-stop-seconds', type=int, default=0)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if not 0 <= args.post_stop_seconds <= 540:
        parser.error('停止后观察门槛必须在 0～540 秒之间')
    root = Path(__file__).resolve().parents[2]
    if any(not path.resolve().is_relative_to(root / 'logs') for path in (args.board, args.controls)) or not args.output.resolve().is_relative_to(root / 'data/MEMORY-002'):
        parser.error('日志／报告必须位于本项目规定目录')
    report = analyze(args.board.read_text(), args.controls.read_text(), args.programs, args.post_stop_seconds)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(report, ensure_ascii=False, indent=2))
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
