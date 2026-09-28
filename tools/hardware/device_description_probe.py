"""DEVICE-002A real-board description check; no install or light-output commands.

Run once, reset the board separately with the documented espflash command, then
run with --previous to distinguish reconnect identity from boot identity.
"""
import argparse
import asyncio
import json
import struct
from pathlib import Path

from bleak import BleakScanner

from gatt_probe import Probe, SERVICE, report

DESCRIPTION = "f889ed64-0100-4e83-968e-799ab99558fa"


def decode(data, session):
    assert len(data) == 96, f"description length {len(data)}"
    assert data[:8] == b"SMDC\x01\x00\x60\x00", data[:8].hex()
    device, boot = data[8:24], data[24:40]
    assert device[:10] == b"SMESP32S3\0"
    assert device[10:] not in (bytes(6), bytes([255]) * 6)
    assert device[10] & 1 == 0
    assert boot != bytes(16)
    assert struct.unpack_from("<Q", data, 40)[0] == session != 0
    assert struct.unpack_from("<HHHHI", data, 48) == (1, 0, 2, 0, 1)
    # This firmware implements ONLY diagnostics: all budgets and auth are zero.
    assert data[60:] == bytes(36), "unimplemented business capability advertised"
    return dict(device_id=device.hex(), boot_id=boot.hex(), session=session,
                firmware="0.2.0", capabilities=1, authentication=0,
                description_bytes=len(data))


async def run(args):
    devices = await BleakScanner.discover(timeout=8, service_uuids=[SERVICE])
    assert len(devices) == 1, f"Expected one diagnostic board, found {len(devices)}"
    observations = []
    for cycle in range(3):
        probe = Probe(devices[0])
        try:
            await probe.connect()
            session = await probe.exchange(1)
            characteristic = probe.client.services.get_characteristic(DESCRIPTION)
            assert characteristic is not None and "read" in characteristic.properties
            assert not any(p.startswith("write") for p in characteristic.properties)
            raw = bytes(await probe.client.read_gatt_char(DESCRIPTION))
            description = decode(raw, session)
            initial = await probe.info()  # Also asserts self-test + output-disabled flags.
            for seq in range(1, 9):
                await asyncio.sleep(2)
                await probe.exchange(2, session, seq)
                assert bytes(await probe.client.read_gatt_char(DESCRIPTION)) == raw
            steady = await probe.info()
            assert steady["ticks"] > initial["ticks"] + 450
            assert initial["heap_used"] == steady["heap_used"]
            assert steady["heap_used"] + steady["heap_free"] == 128 * 1024
            if observations:
                assert description["device_id"] == observations[0]["device_id"]
                assert description["boot_id"] == observations[0]["boot_id"]
                assert session not in [o["session"] for o in observations]
            observations.append(description)
            report("description_pass", cycle=cycle + 1, **description, **steady)
        finally:
            await probe.close()
        await asyncio.sleep(1)
    if args.previous:
        previous = json.loads(args.previous.read_text())["observations"][0]
        assert previous["device_id"] == observations[0]["device_id"]
        assert previous["boot_id"] != observations[0]["boot_id"]
        report("boot_identity_pass", stable_device=True, new_boot=True)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(dict(observations=observations), indent=2) + "\n")
    report("PASS", scope="read-only identity, 3 sessions, 24 heartbeats, output disabled")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--previous", type=Path)
    asyncio.run(run(parser.parse_args()))
