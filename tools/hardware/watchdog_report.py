"""Validate explicit, timestamped S3 watchdog injections; not waveform timing."""
import argparse
import json
import re
from pathlib import Path

FAULTS = {
    "sender-stall": None,
    "sender-exit": "SenderExited",
    "worker-stall": "Deadline",
    "worker-failed": "Worker",
}


def analyze(text, case):
    assert case in FAULTS, "unknown injection"
    events = []
    resets = []
    previous = 0.0
    for line in text.splitlines():
        match = re.match(r"^@(\d+\.\d+) (.*)$", line)
        if not match:
            continue
        now, message = float(match[1]), match[2]
        assert now >= previous, "host clock moved backwards"
        previous = now
        if "rst:" in message:
            resets.append((now, message))
        if message.startswith("WATCHDOG "):
            assert f"case={case} " in message, "mixed injection case"
            events.append((now, message))

    def selected(prefix):
        return [(t, m) for t, m in events if m.startswith(prefix)]

    boots = selected("WATCHDOG PROBE ")
    armed = selected("WATCHDOG ARMED ")
    injections = selected("WATCHDOG INJECT ")
    trips = selected("WATCHDOG TRIPPED ")
    recovered = selected("WATCHDOG RECOVERED ")
    assert len(boots) == 2, "need original boot and exactly one hardware reboot"
    assert "CoreMwdt1" not in boots[0][1], "capture started after fault"
    assert "reset=Some(CoreMwdt1)" in boots[1][1], "wrong reset source"
    assert len(armed) == len(injections) == 1, "missing or repeated fault injection"
    assert "reset_ms=1000 fault_after_ms=3000" in armed[0][1], "wrong watchdog budget"
    assert boots[0][0] <= armed[0][0] < injections[0][0] < boots[1][0]
    activity = re.search(r"worker_progress=(\d+)", injections[0][1])
    assert activity and 80 <= int(activity[1]) < 2**32 - 1, "no prior worker progress"
    fault = FAULTS[case]
    if fault is None:
        assert not trips, "synchronous stall unexpectedly returned to supervisor"
    else:
        assert len(trips) == 1 and f"fault={fault}" in trips[0][1], "wrong software trip"
        assert injections[0][0] <= trips[0][0] < boots[1][0]
    watchdog_resets = [(t, m) for t, m in resets if "rst:0x8 (TG1WDT_SYS_RST)" in m]
    assert len(watchdog_resets) == 1, "need independent ROM watchdog reset evidence"
    reset_at = watchdog_resets[0][0]
    assert injections[0][0] < reset_at < boots[1][0]
    assert not any(injections[0][0] < t and t != reset_at for t, _ in resets), "unexpected additional reset"
    # Use the ROM reset event. The application line can arrive with its next USB
    # report; its reception timestamp is not when the watchdog reset occurred.
    delay = reset_at - injections[0][0]
    low, high = (2, 4) if case == "worker-stall" else (0.5, 2)
    assert low <= delay <= high, "reset outside predeclared host-observation bounds"
    assert boots[1][0] - reset_at <= 2, "late application boot observation"
    seconds = [int(re.search(r"stable_seconds=(\d+)", m)[1]) for _, m in recovered]
    assert len(seconds) >= 12 and seconds == list(range(1, len(seconds) + 1)), "missing stable reboot observation"
    assert boots[1][0] < recovered[0][0] and recovered[-1][0] - boots[1][0] >= 11
    assert all("RS485=disabled" in m for _, m in boots + recovered), "missing output guard"
    return {
        "case": case,
        "reset": "CoreMwdt1",
        "host_observed_reset_seconds": round(delay, 6),
        "host_observed_application_seconds": round(boots[1][0] - injections[0][0], 6),
        "stable_seconds": seconds[-1],
        "physical_dmx_verified": False,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("case", choices=FAULTS)
    parser.add_argument("log", type=Path)
    args = parser.parse_args()
    print(json.dumps(analyze(args.log.read_text(), args.case), ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
