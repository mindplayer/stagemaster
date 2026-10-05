"""Read-only, bounded Cargo.lock/TAR verification; never extracts archive files."""
import hashlib
import gzip
import json
import os
import sys
import tarfile
import tomllib


class BoundedReader:
    # Count all decompressed bytes, including TAR extension headers, not only
    # regular members that tarfile exposes to the caller.
    def __init__(self, source):
        self.source = source
        self.remaining = 128 * 1024 * 1024

    def read(self, size):
        if size < 0 or size > self.remaining:
            raise ValueError("原包展开读取预算超限")
        data = self.source.read(size)
        self.remaining -= len(data)
        return data


def verify(request):
    locked = {}
    for package in tomllib.loads(request["lockText"])["package"]:
        key = (package["name"], package["version"], package.get("source"))
        if key in locked:
            raise ValueError("Cargo锁身份重复")
        locked[key] = package
    entries = request["entries"]
    if len(entries) > 1024:
        raise ValueError("原包核对数量超限")
    result = []
    for entry in entries:
        package = locked[(entry["name"], entry["version"], entry["source"])]
        path = entry["archive"]
        descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(descriptor, "rb") as source:
            if not 0 < os.fstat(source.fileno()).st_size <= 64 * 1024 * 1024:
                raise ValueError("原包大小预算无效")
            digest = hashlib.file_digest(source, "sha256").hexdigest()
            if digest != package["checksum"]:
                raise ValueError(f"锁定原包摘要不匹配：{entry['name']}")
            source.seek(0)
            files = entry["files"]
            if not 1 <= len(files) <= 17 or len({file["file"] for file in files}) != len(files):
                raise ValueError("原包核对文件数量或身份无效")
            expected = {f"{entry['name']}-{entry['version']}/{file['file']}": file["sha256"] for file in files}
            found = set()
            total = count = 0
            with gzip.GzipFile(fileobj=source) as compressed, tarfile.open(fileobj=BoundedReader(compressed), mode="r|") as archive:
                for member in archive:
                    count += 1
                    total += member.size
                    if count > 16384 or member.size < 0 or member.size > 32 * 1024 * 1024 or total > 128 * 1024 * 1024:
                        raise ValueError("原包展开读取预算超限")
                    if member.name not in expected:
                        continue
                    if member.name in found or not member.isfile() or member.size > 512 * 1024:
                        raise ValueError("原包告知重复、类型或预算无效")
                    with archive.extractfile(member) as content:
                        actual = hashlib.sha256(content.read(512 * 1024 + 1)).hexdigest()
                    if actual != expected[member.name]:
                        raise ValueError(f"缓存文本与锁定原包不符：{entry['name']}/{member.name}")
                    found.add(member.name)
            if found != set(expected):
                raise ValueError(f"原包缺少缓存告知文本：{entry['name']}")
            result.append({"name": entry["name"], "version": entry["version"], "archiveSha256": digest, "filesCompared": len(found)})
    return result


try:
    request_bytes = sys.stdin.buffer.read(8 * 1024 * 1024 + 1)
    if len(request_bytes) > 8 * 1024 * 1024:
        raise ValueError("原包核对输入预算超限")
    print(json.dumps(verify(json.loads(request_bytes)), ensure_ascii=False))
except (ValueError, KeyError, OSError, tarfile.TarError) as error:
    print(f"Rust告知原包核对失败：{error}", file=sys.stderr)
    sys.exit(1)
