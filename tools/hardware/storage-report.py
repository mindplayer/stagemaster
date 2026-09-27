#!/usr/bin/env python3
"""Inspect an already built Xtensa ELF with the installed binutils; never contact a board."""
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ELF = ROOT / 'target/esp32-storage-check/xtensa-esp32s3-none-elf/release/stagemaster-esp32-probe'


def command(*args):
    return subprocess.check_output(args, text=True)


symbols = command('xtensa-esp32s3-elf-nm', '-SC', str(ELF))
resource = next(line.split() for line in symbols.splitlines()
                if line.endswith('stagemaster_esp32_probe::package_storage::RESOURCE_BYTES'))
address, length = int(resource[0], 16), int(resource[1], 16)
assert length == 24
raw = command('xtensa-esp32s3-elf-objdump', '-s', f'--start-address={address}',
              f'--stop-address={address + length}', str(ELF))
words = []
for line in raw.splitlines():
    tokens = line.split()
    if tokens and re.fullmatch(r'[0-9a-f]{8}', tokens[0]):
        for token in tokens[1:]:
            if not re.fullmatch(r'[0-9a-f]{8}', token):
                break
            words.append(int.from_bytes(bytes.fromhex(token), 'little'))
assert len(words) == 6
labels = ['service_including_store', 'store', 'snapshot', 'frame', 'assembler', 'driver_reference']
sections = {}
for line in command('xtensa-esp32s3-elf-size', '-A', str(ELF)).splitlines():
    values = line.split()
    if len(values) == 3 and values[0].startswith('.'):
        sections[values[0]] = int(values[1])
# Function-entry sizes are not full call-chain peaks. Inlining and nested callers matter.
entries = {}
current = ''
for line in command('xtensa-esp32s3-elf-objdump', '-Cd', str(ELF)).splitlines():
    label = re.fullmatch(r'[0-9a-f]+ <(.+)>:', line)
    if label:
        current = label[1]
    entry = re.search(r'\bentry\s+a1, (0x[0-9a-f]+|[0-9]+)', line)
    if entry:
        entries[current] = int(entry[1], 0)
selected = {name: size for name, size in entries.items()
            if any(part in name for part in ['stagemaster_nor_store::io::',
                                             'stagemaster_nor_store::metadata::',
                                             'esp_bootloader_esp_idf::partitions::read_partition_table',
                                             'stagemaster_transfer::service::Service'])}
print(json.dumps({
    'elf': str(ELF.relative_to(ROOT)),
    'type_bytes': dict(zip(labels, words)),
    'sections': {k: sections[k] for k in ['.bss', '.data', '.data.wifi', '.stack']},
    'function_entry_stack_bytes_not_call_chain_peak': selected,
    'largest_entry': sorted(entries.items(), key=lambda item: item[1], reverse=True)[:5],
    'hardware_measured': False,
}, ensure_ascii=False, indent=2))
