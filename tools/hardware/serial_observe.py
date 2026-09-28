"""Bounded USB log capture without reset. Pairing code is shown locally, not saved."""
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
    deadline = time.monotonic() + args.seconds
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
    run(parser.parse_args())
