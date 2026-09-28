"""Read-only DEVICE-002B experiment; no installation or output interface.

The native OS owns passkey entry. Display the board's ephemeral code locally over
USB; this client neither chooses a fixed code nor skips authentication errors.
"""
import asyncio
import struct

from bleak import BleakScanner
from bleak.exc import BleakError

from gatt_probe import Probe, SERVICE, report

PROOF = "f889ed71-0100-4e83-968e-799ab99558fa"


async def keep_alive(probe, session):
    for sequence in range(1, 91):
        await asyncio.sleep(2)
        await probe.exchange(2, session, sequence)


async def run():
    devices = await BleakScanner.discover(timeout=8, service_uuids=[SERVICE])
    assert len(devices) == 1, f"Expected one diagnostic board, found {len(devices)}"
    for attempt in range(2):
        probe = Probe(devices[0])
        heartbeats = None
        proof_read = None
        try:
            await probe.connect()
            info = await probe.info()
            # Pair before HELLO: macOS serializes ordinary ATT while its native
            # pairing sheet waits for the user. Never pretend a heartbeat passed.
            proof_read = asyncio.create_task(probe.client.read_gatt_char(PROOF))
            report("awaiting_authenticated_read", attempt=attempt + 1)
            data = bytes(await asyncio.wait_for(proof_read, 80))
            session = await probe.exchange(1)
            assert data == b"SMTP\x01\0\0\0" + struct.pack("<Q", session)
            heartbeats = asyncio.create_task(keep_alive(probe, session))
            report("authenticated_read_pass", attempt=attempt + 1)
            await asyncio.sleep(10)
            assert not heartbeats.done(), "heartbeat stopped after pairing"
            steady = await probe.info()
            assert steady["ticks"] > info["ticks"] + 300
            report("authenticated_heartbeat_pass", attempt=attempt + 1, **steady)
        finally:
            for task in [proof_read, heartbeats]:
                if task:
                    task.cancel()
            await asyncio.gather(*(t for t in [proof_read, heartbeats] if t), return_exceptions=True)
            try:
                await probe.close()
            except BleakError as error:
                # Keep the original read/authentication failure as the cause.
                report("disconnect_unconfirmed", detail=str(error))
        await asyncio.sleep(1)
    report("PASS", scope="authenticated read before HELLO, then normal heartbeats and reconnect; no writes")


if __name__ == "__main__":
    asyncio.run(run())
