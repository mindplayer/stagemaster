"""Controlled persistent-binding acceptance; never installs a show or enables output.

--pair expects the new-pairing connection to be closed before a proof can be read.
Only a fresh, authenticated reconnect may then succeed. Native passkey entry stays
with macOS. Run again after a separately authorized board reset to check durability.
"""
import argparse
import asyncio
import struct
import time

from bleak import BleakScanner
from bleak.exc import BleakError

from device_description_probe import DESCRIPTION, decode
from gatt_probe import Probe, SERVICE, report
from secure_link_probe import PROOF
from secure_read import read_authenticated


async def discover():
    devices = await BleakScanner.discover(timeout=5, service_uuids=[SERVICE])
    assert len(devices) == 1, f"expected one board, found {len(devices)}"
    report("discovered", address=devices[0].address)
    return devices[0]


async def pair():
    probe = Probe(await discover())
    try:
        await probe.connect()
        report("awaiting_native_pairing")
        try:
            await read_authenticated(probe.client, PROOF)
        except (BleakError, asyncio.TimeoutError) as error:
            report("pairing_read_ended", detail=str(error), kind=type(error).__name__)
            await asyncio.wait_for(probe.disconnected.wait(), 5)
            report("pairing_connection_closed", detail=str(error))
        else:
            raise AssertionError("fresh pairing must disconnect before granting authority")
    finally:
        await probe.close()


async def verify(seconds):
    probe = Probe(await discover())
    try:
        await probe.connect()
        proof = await read_authenticated(probe.client, PROOF)
        session = await probe.exchange(1)
        assert proof == b"SMTP\x01\0\0\0" + struct.pack("<Q", session)
        description = decode(bytes(await probe.client.read_gatt_char(DESCRIPTION)), session)
        initial = await probe.info()
        timings = []
        for sequence in range(1, seconds // 2 + 1):
            await asyncio.sleep(2)
            started = time.monotonic()
            await probe.exchange(2, session, sequence)
            timings.append((time.monotonic() - started) * 1000)
        steady = await probe.info()
        assert steady["ticks"] > initial["ticks"] + seconds * 30
        assert steady["heap_used"] == initial["heap_used"]
        report("durable_binding_read_pass", description=description, heartbeats=len(timings),
               maximum_roundtrip_ms=round(max(timings), 3))
        return session
    finally:
        await probe.close()


async def run(args):
    if args.pair:
        await pair()
        await asyncio.sleep(3)
    first = await verify(args.seconds)
    await asyncio.sleep(1)
    second = await verify(10)
    assert first != second
    if args.expiry:
        await verify_expiry()
    report("PASS", scope="persistent authenticated read and reconnect; no package/output writes")


async def verify_expiry():
    probe = Probe(await discover())
    try:
        await probe.connect()
        await read_authenticated(probe.client, PROOF)
        session = await probe.exchange(1)
        await probe.exchange(2, session, 1)
        started = time.monotonic()
        for _ in range(2):
            await asyncio.sleep(2)
            await probe.exchange(2, session, 1, expected=4)
        await asyncio.wait_for(probe.disconnected.wait(), 4)
        elapsed = time.monotonic() - started
        assert 5.5 <= elapsed <= 8, elapsed
        report("authenticated_expiry_pass", seconds=round(elapsed, 3))
    finally:
        await probe.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--pair", action="store_true")
    parser.add_argument("--expiry", action="store_true")
    parser.add_argument("--seconds", type=int, choices=range(10, 301, 2), default=10)
    asyncio.run(run(parser.parse_args()))
