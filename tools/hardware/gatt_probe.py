"""Explicit test client for PLAYER-002A; never controls fixtures or installs a show.

Run with the project-local Python environment documented in tools/hardware/README.md.
The independent struct codec is a cross-language wire-format check, not product UI code.
"""
import asyncio
import json
import struct
import time

from bleak import BleakClient, BleakScanner

SERVICE = "f889ed60-0100-4e83-968e-799ab99558fa"
RX = "f889ed61-0100-4e83-968e-799ab99558fa"
TX = "f889ed62-0100-4e83-968e-799ab99558fa"
INFO = "f889ed63-0100-4e83-968e-799ab99558fa"
WIRE = struct.Struct("<BBBBQII")


def report(event, **data):
    print(json.dumps({"event": event, **data}, ensure_ascii=False), flush=True)


class Probe:
    def __init__(self, device):
        self.disconnected = asyncio.Event()
        self.client = BleakClient(device, disconnected_callback=lambda _: self.disconnected.set())
        self.responses = asyncio.Queue()

    async def connect(self):
        await self.client.connect()
        assert self.client.services.get_service(SERVICE) is not None
        await self.client.start_notify(TX, lambda _, data: self.responses.put_nowait(bytes(data)))
        report("connected", mtu=self.client.mtu_size)

    async def exchange(self, kind, session=0, sequence=0, expected=0, version=1):
        assert self.responses.empty(), "unexpected unsolicited reply"
        await self.client.write_gatt_char(RX, WIRE.pack(version, kind, 0, 0, session, sequence, 0), response=True)
        response = await asyncio.wait_for(self.responses.get(), 2)
        v, op, code, reserved, nonce, seq, interval = WIRE.unpack(response)
        # An undecodable version has no trustworthy request fields to echo.
        expected_op, expected_seq = (128, 0) if expected == 2 else (kind | 128, sequence)
        assert (v, op, code, reserved, seq) == (1, expected_op, expected, 0, expected_seq), response.hex()
        assert nonce != 0 and interval == 2000
        if session and expected != 3:
            assert nonce == session
        assert bytes(await self.client.read_gatt_char(TX)) == response, "read/notify disagree"
        return nonce

    async def info(self):
        data = bytes(await self.client.read_gatt_char(INFO))
        v, flags, reserved1, reserved2, uptime, ticks, used, free = struct.unpack("<BBBBIIII", data)
        assert (v, flags, reserved1, reserved2) == (1, 3, 0, 0), data.hex()
        result = dict(uptime_ms=uptime, ticks=ticks, heap_used=used, heap_free=free)
        report("device_info", **result)
        return result

    async def close(self):
        await self.client.disconnect()


async def main():
    devices = await BleakScanner.discover(timeout=8, service_uuids=[SERVICE])
    assert len(devices) == 1, f"Expected one StageMaster diagnostic device, found {len(devices)}"
    device = devices[0]
    report("discovered", name=device.name, address=device.address)
    probe = Probe(device)
    try:
        await probe.connect()
        initial = await probe.info()
        nonce = await probe.exchange(1)
        await probe.exchange(2, nonce, 1, expected=2, version=2)
        await probe.exchange(2, nonce ^ 1, 1, expected=3)
        await probe.exchange(2, nonce, 1)
        await probe.exchange(2, nonce, 1, expected=4)
        await probe.exchange(2, nonce, 0, expected=4)
        # The malformed ATT value must fail at the adapter, without poisoning the next request.
        try:
            await probe.client.write_gatt_char(RX, b"\x01\x02", response=True)
        except Exception as error:
            report("short_write_rejected", reason=str(error))
        else:
            raise AssertionError("short GATT write unexpectedly accepted")
        for seq in range(2, 17):
            await asyncio.sleep(2)
            await probe.exchange(2, nonce, seq)
        steady = await probe.info()
        assert steady["ticks"] > initial["ticks"] + 900, "local playback stalled during BLE traffic"
        report("heartbeat_pass", count=16, duration_seconds=30)
        started = time.monotonic()
        # Rejected duplicates must NOT extend the lease.
        for _ in range(2):
            await asyncio.sleep(2)
            await probe.exchange(2, nonce, 16, expected=4)
        await asyncio.wait_for(probe.disconnected.wait(), 4)
        elapsed = time.monotonic() - started
        assert 5.5 <= elapsed <= 8, elapsed
        report("expiry_pass", seconds=round(elapsed, 3))
    finally:
        await probe.close()
    previous = nonce
    for cycle in range(3):
        probe = Probe(device)
        try:
            await probe.connect()
            current = await probe.exchange(1)
            assert current != previous, "connection reused a session"
            await probe.exchange(2, previous, 17, expected=3)
            await probe.exchange(2, current, 1)
            info = await probe.info()
            assert info["ticks"] > steady["ticks"], "local playback stopped on disconnect"
            report("reconnect_pass", cycle=cycle + 1)
            previous = current
        finally:
            await probe.close()
        await asyncio.sleep(1)
    report("PASS", scope="GATT discovery/read/write/notify, heartbeat, expiry, reconnect; RS485 disabled")


if __name__ == "__main__":
    asyncio.run(main())
