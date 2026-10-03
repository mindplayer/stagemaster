"""Bounded USB log capture; opening the port may reset a board via its USB driver.

Pairing codes are shown locally, never saved. Inspect boot reasons in the capture.
"""
import argparse
import re
import time
from pathlib import Path

import serial


def run(args):
    root = Path(__file__).resolve().parents[2]
    output = Path(args.log).resolve()
    assert output.is_relative_to(root / "logs"), "logs must stay inside this project"
    connection = serial.Serial(port=None, baudrate=115200, timeout=0.1)
    connection.dtr = False
    connection.rts = False
    connection.port = args.port
    connection.open()
    started = time.monotonic()
    deadline = started + args.seconds
    pending = b""
    try:
        with output.open("a", encoding="utf-8") as log:
            while time.monotonic() < deadline:
                pending += connection.read(connection.in_waiting or 1)
                while b"\n" in pending:
                    line, pending = pending.split(b"\n", 1)
                    text = line.decode("utf-8", errors="replace").strip()
                    if not text:
                        continue
                    clean = re.sub(r"(DEVELOPMENT PAIRING CODE: )\d+", r"\1[REDACTED]", text)
                    if args.timestamps:
                        stamp = f"@{time.monotonic() - started:.6f} "
                        clean = stamp + clean
                        text = stamp + text
                    log.write(clean + "\n")
                    log.flush()
                    # The live, single-use code is needed for the authorized macOS pairing.
                    print(text, flush=True)
    finally:
        connection.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", required=True)
    parser.add_argument("--log", required=True)
    parser.add_argument("--seconds", type=int, choices=range(1, 601), default=180)
    parser.add_argument("--timestamps", action="store_true", help="增加宿主单调时间，不代表线路波形时间")
    try:
        run(parser.parse_args())
    except KeyboardInterrupt:
        print("USB 观察已停止", flush=True)
