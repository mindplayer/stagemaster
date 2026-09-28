"""Bounded authenticated read for the development pairing experiment.

Bleak's pinned CoreBluetooth public read hard-codes a 20-second delegate timeout.
The native pairing sheet needs the experiment's longer, still bounded deadline.
Keep this test-only dependency adapter out of the product transport.
"""
import asyncio
import sys


async def read_authenticated(client, uuid, timeout=80):
    characteristic = client.services.get_characteristic(uuid)
    if characteristic is None:
        raise RuntimeError("authenticated probe characteristic is absent")
    if sys.platform == "darwin":
        # The public wrapper does not forward a timeout keyword. Use the existing
        # CoreBluetooth delegate, preserving its native authentication and cleanup.
        delegate = client._backend._delegate
        if delegate is None:
            raise RuntimeError("CoreBluetooth delegate is unavailable")
        operation = delegate.read_characteristic(
            characteristic.obj, use_cached=False, timeout=timeout
        )
    else:
        operation = client.read_gatt_char(characteristic)
    return bytes(await asyncio.wait_for(operation, timeout))
