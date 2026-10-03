"""Verify actual worker frame telemetry; never treats diagnostic LIVE ticks as frames."""
import argparse
import json
import re
from pathlib import Path

COUNTERS = ('attempts', 'frames', 'running_frames', 'offline_frames', 'absent',
            'errors', 'clock_errors', 'over_30ms', 'over_50ms', 'backpressure',
            'commands', 'repeated_samples')
FIELDS = set(COUNTERS) | {'v', 'at_us', 'min_gap_us', 'max_gap_us', 'max_work_us',
                          'instance', 'sampled_ms', 'elapsed_ms', 'running', 'online',
                          'fingerprint'}


def segments(samples, predicate):
    result, current = [], []
    for sample in samples:
        if predicate(sample):
            if current and sample['instance'] != current[-1]['instance']:
                result.append(current)
                current = []
            current.append(sample)
        elif current:
            result.append(current)
            current = []
    if current:
        result.append(current)
    return result


def elapsed(group):
    return (group[-1]['at_us'] - group[0]['at_us']) / 1_000_000 if group else 0


def analyze(text):
    samples, failures = [], []
    if text.count('ESP-ROM:') != 1:
        failures.append('需要同一次完整启动的串口记录')
    if re.search(r'panicked|unavailable;|assertion .*failed', text):
        failures.append('设备日志包含运行故障')
    for line in text.splitlines():
        if not line.startswith('RUNTIME_SAMPLE '):
            continue
        try:
            parts = dict(item.split('=', 1) for item in line.split()[1:])
            if set(parts) != FIELDS:
                raise ValueError('字段缺失或未知')
            sample = {k: int(v, 16 if k == 'fingerprint' else 10) for k, v in parts.items()}
            if sample['v'] != 1 or any(v < 0 for v in sample.values()):
                raise ValueError('版本或计数非法')
            if samples and sample == samples[-1]:
                continue  # unchanged publication is not additional frame evidence
            if samples and (sample['at_us'] <= samples[-1]['at_us'] or any(
                    sample[k] < samples[-1][k] for k in COUNTERS)):
                raise ValueError('时间或累计计数倒退')
            samples.append(sample)
        except (ValueError, KeyError) as error:
            failures.append(f'运行帧记录不完整：{error}')
    if not samples:
        failures.append('没有实际节目帧记录')
    if any(s['errors'] or s['clock_errors'] for s in samples):
        failures.append('帧生成或时钟存在错误')
    if any(s['max_gap_us'] > 50_000 or s['over_50ms'] for s in samples):
        failures.append('调度尝试间隔超过预设 50 毫秒门槛')
    groups = segments(samples, lambda s: s['running'] == 1 and s['online'] == 0 and s['instance'] > 0)
    offline = max(groups, key=elapsed, default=[])
    offline_seconds = elapsed(offline)
    rate = 0
    if offline_seconds < 30:
        failures.append('缺少同实例至少 30 秒的连续断线运行记录')
    else:
        first, last = offline[0], offline[-1]
        count = last['frames'] - first['frames']
        rate = count / offline_seconds
        if not 39 <= rate <= 41:
            failures.append('断线生成帧率未达到预设 39～41 Hz')
        if count != last['offline_frames'] - first['offline_frames'] or count <= 0:
            failures.append('断线帧与实际成功帧计数不一致')
        if last['commands'] != first['commands']:
            failures.append('断线区间仍有运行命令')
        # elapsed_ms is local to the active step: follow/loop may legitimately reset it.
        # Exact step/cycle advancement is checked by the native control probe.
        if not any(a['elapsed_ms'] != b['elapsed_ms'] for a, b in zip(offline, offline[1:])):
            failures.append('断线区间节目进度没有推进')
    paused = segments(samples, lambda s: s['running'] == 0 and s['instance'] > 0)
    pause_ok = any(elapsed(g) >= 4 and g[-1]['frames'] > g[0]['frames']
                   and g[-1]['elapsed_ms'] == g[0]['elapsed_ms']
                   and g[-1]['running_frames'] == g[0]['running_frames'] for g in paused)
    if not pause_ok:
        failures.append('缺少暂停期间持续生成帧且进度保持的记录')
    memory = [tuple(map(int, match)) for match in re.findall(
        r'heap=(\d+)/(\d+) heap_peak=(\d+)', text)]
    if not memory:
        failures.append('缺少运行内存采样')
    return {
        'passed': not failures, 'failures': failures, 'samples': len(samples),
        'offline_seconds': offline_seconds, 'offline_frame_hz': rate,
        'paused_frame_hold': pause_ok,
        'max_gap_us': max((s['max_gap_us'] for s in samples), default=0),
        'max_work_us': max((s['max_work_us'] for s in samples), default=0),
        'heap_peak_bytes': max((m[2] for m in memory), default=0),
        'min_heap_free_bytes': min((m[1] for m in memory), default=0),
        'physical_output_verified': False,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--log', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    source, target = Path(args.log).resolve(), Path(args.output).resolve()
    if not source.is_relative_to(root / 'logs') or not target.is_relative_to(root / 'data'):
        parser.error('输入日志／输出报告须位于本项目 logs／data')
    result = analyze(source.read_text())
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(result, ensure_ascii=False, indent=2))
    raise SystemExit(0 if result['passed'] else 1)


if __name__ == '__main__':
    main()
