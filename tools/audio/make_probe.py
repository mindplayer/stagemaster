"""Generate a quiet, original 32-second signal for native audio acceptance (not product content)."""
import math
from pathlib import Path
import struct
import wave

root = Path(__file__).resolve().parents[2]
target = root / 'data' / 'AUDIO-001' / '节奏验收.wav'
target.parent.mkdir(parents=True, exist_ok=True)
with wave.open(str(target), 'wb') as output:
    output.setparams((1, 2, 44100, 0, 'NONE', 'not compressed'))
    for second in range(32):
        block = bytearray()
        for sample in range(44100):
            time = second + sample / 44100
            envelope = max(0, 1 - (time % 0.5) / 0.08)
            tone = .08 * envelope * math.sin(2 * math.pi * (180 if int(time * 2) % 4 == 0 else 520) * time)
            tone += .02 * math.sin(2 * math.pi * (220 + 55 * (second // 8)) * time)
            block.extend(struct.pack('<h', int(tone * 32767)))
        output.writeframes(block)
print(target)
