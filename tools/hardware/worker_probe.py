"""Observe the compile-time local NOR harness while measuring real diagnostic BLE.

No GATT install/pairing request is sent. Firmware itself must have been explicitly
built/flashed for this test. Serial is opened with reset control lines deasserted.
"""
import argparse
import asyncio
import json
import re
import threading
import time
from pathlib import Path

import serial
from bleak import BleakScanner
from device_description_probe import DESCRIPTION, decode
from gatt_probe import Probe, SERVICE, report


def serial_watch(port, stop, records):
    connection = serial.Serial(port=None, baudrate=115200, timeout=0.1)
    connection.dtr = False
    connection.rts = False
    connection.port = port
    connection.open()
    pending = b""
    try:
        while not stop.is_set():
            pending += connection.read(connection.in_waiting or 1)
            while b"\n" in pending:
                line, pending = pending.split(b"\n", 1)
                text = line.decode("utf-8", errors="replace").strip()
                if text:
                    records.append(dict(time=time.monotonic(), text=text))
                    report("serial", line=text)
    finally:
        connection.close()


async def run(args):
    lines, timings, snapshots = [], [], []
    result = dict(scope="local worker harness plus diagnostic BLE; no authenticated GATT installation",
                  passed=False, timings=timings, snapshots=snapshots, serial=lines)
    stopped = threading.Event()
    reader = asyncio.create_task(asyncio.to_thread(serial_watch, args.port, stopped, lines))
    probe = None
    try:
        devices = await BleakScanner.discover(timeout=8, service_uuids=[SERVICE])
        assert len(devices) == 1, f"expected one diagnostic board, found {len(devices)}"
        probe = Probe(devices[0])
        await probe.connect()
        session = await probe.exchange(1)
        description = decode(bytes(await probe.client.read_gatt_char(DESCRIPTION)), session)
        initial = await probe.info()
        result.update(description=description, initial=initial)
        # Must attach before the 45-second local harness begins, not after the work.
        assert initial["uptime_ms"] < 40000, "missed test window; reset board and rerun"
        start = time.monotonic()
        seq = 0
        while time.monotonic() - start < args.seconds:
            await asyncio.sleep(0.25)
            seq += 1
            before = time.monotonic()
            await probe.exchange(2, session, seq)
            timings.append(dict(time=before, milliseconds=(time.monotonic()-before)*1000))
            if seq % 20 == 0:
                snapshots.append(await probe.info())
            if reader.done():
                reader.result()
                raise AssertionError("serial reader ended early")
        steady = await probe.info()
        result["final"] = steady
        assert not probe.disconnected.is_set()
        assert steady["ticks"] > initial["ticks"] + args.seconds * 30
        assert steady["heap_used"] == initial["heap_used"], "retained heap changed during test"
        log = "\n".join(line["text"] for line in lines)
        assert "panicked" not in log and "Backtrace" not in log
        assert "LOCAL WORKER PROBE PASS" in log, "local harness did not finish"
        if args.expect_write:
            assert "LOCAL WORKER WRITE TEST installed" in log
            counters = re.findall(r"writes=(\d+) erases=(\d+)", log)
            assert counters and int(counters[-1][0]) > 0 and int(counters[-1][1]) > 0
        if args.expect_existing or args.expect_readonly_recovery:
            assert re.search(r"RecoveryReport .*selected: Some\(Commit", log), "no durable recovery"
            if args.expect_existing:
                assert "LOCAL WORKER WRITE TEST installed" in log
            if args.expect_readonly_recovery:
                assert "local-write-test=false" in log
                assert "LOCAL WORKER WRITE TEST" not in log
            counters = re.findall(r"writes=(\d+) erases=(\d+)", log)
            assert counters and all(w == "0" and e == "0" for w, e in counters), "same package was rewritten"
        result["nor_samples"] = [dict(writes=int(w), erases=int(e))
                                 for w, e in re.findall(r"writes=(\d+) erases=(\d+)", log)]
        if args.expect_byte_channel:
            channels = [dict(payload=int(p), requests=int(r), incoming_parts=int(i),
                             outgoing_parts=int(o), endpoint_bytes=int(s))
                        for p, r, i, o, s in re.findall(
                            r"LOCAL BYTE CHANNEL payload=(\d+) requests=(\d+) incoming_parts=(\d+) outgoing_parts=(\d+) endpoint_bytes=(\d+)", log)]
            expected = [20, 244] if args.expect_write or args.expect_existing else [20]
            assert [channel["payload"] for channel in channels] == expected, "missing byte-channel run"
            assert all(channel["requests"] > 0 and channel["incoming_parts"] >= channel["requests"]
                       and channel["outgoing_parts"] >= channel["requests"]
                       and channel["endpoint_bytes"] < 3000 for channel in channels)
            result["local_byte_channels"] = channels
        if args.reference_replay:
            expected = re.findall(r"帧摘要 ([0-9a-f]{64})", args.reference_replay.read_text())
            actual = re.findall(r"LOCAL WORKER REPLAY index=(\d+) frames=400 digest=([0-9a-f]{64})", log)
            assert expected and actual, "missing reference or device replay"
            assert [int(index) for index, _ in actual] == list(range(len(expected)))
            assert [digest for _, digest in actual] == expected, "device frames differ from host"
            assert "LOCAL WORKER REPLAY PASS" in log
            result["replay"] = dict(programs=len(expected), frames_per_program=400, matched=True)
        values = sorted(t["milliseconds"] for t in timings)
        result["passed"] = True
        report("PASS", heartbeats=len(values), max_ms=max(values),
               p99_ms=values[min(len(values)-1, int(len(values)*0.99))])
    except BaseException as error:
        result["failure"] = repr(error)
        # Keep the original failure and capture the board's late disconnect,
        # recovery or panic before closing USB. Do not reconnect or retry into PASS.
        if isinstance(error, Exception):
            report("failure_observation", reason=repr(error), seconds=10)
            await asyncio.sleep(10)
        raise
    finally:
        try:
            if probe:
                await probe.close()
        finally:
            stopped.set()
            try:
                await reader
            finally:
                values = sorted(t["milliseconds"] for t in timings)
                result.update(heartbeats=len(values), max_ms=max(values) if values else None,
                              p99_ms=values[min(len(values)-1, int(len(values)*0.99))] if values else None)
                args.output.parent.mkdir(parents=True, exist_ok=True)
                args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2)+"\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seconds", type=int, default=60)
    parser.add_argument("--expect-write", action="store_true")
    parser.add_argument("--expect-existing", action="store_true")
    parser.add_argument("--expect-readonly-recovery", action="store_true")
    parser.add_argument("--expect-byte-channel", action="store_true")
    parser.add_argument("--reference-replay", type=Path)
    asyncio.run(run(parser.parse_args()))
