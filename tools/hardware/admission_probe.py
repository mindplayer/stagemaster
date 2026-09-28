"""Test the experimental pre-HELLO admission deadline without pairing or writes."""
import asyncio
import time

from bleak import BleakScanner

from gatt_probe import Probe, SERVICE, report


async def run():
    devices = await BleakScanner.discover(timeout=8, service_uuids=[SERVICE])
    assert len(devices) == 1
    probe = Probe(devices[0])
    try:
        await probe.connect()
        # More than the normal 6 s timeout is permitted BEFORE the first HELLO.
        await asyncio.sleep(10)
        nonce = await probe.exchange(1)
        await probe.exchange(2, nonce, 1)
        started = time.monotonic()
        await asyncio.wait_for(probe.disconnected.wait(), 8)
        elapsed = time.monotonic() - started
        assert 5.0 <= elapsed <= 8, elapsed
        report("established_expiry_pass", seconds=round(elapsed, 3))
    finally:
        if probe.client.is_connected:
            await probe.close()
    probe = Probe(devices[0])
    try:
        await probe.connect()
        started = time.monotonic()
        # Invalid requests cannot turn the bounded 90 s admission into keepalive.
        nonce = await probe.exchange(2, 0, 1, expected=3)
        for sequence in range(2, 19):
            await asyncio.sleep(5)
            await probe.exchange(2, nonce, sequence, expected=3)
        await asyncio.wait_for(probe.disconnected.wait(), 10)
        elapsed = time.monotonic() - started
        assert 88 <= elapsed <= 95, elapsed
        report("admission_expiry_pass", seconds=round(elapsed, 3))
    finally:
        if probe.client.is_connected:
            await probe.close()
    report("PASS", scope="bounded pre-HELLO wait, invalid traffic cannot renew, normal 6 s lease preserved")


if __name__ == "__main__":
    asyncio.run(run())
