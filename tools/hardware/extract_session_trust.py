"""Extract this experiment's public trust pin from one fresh, controlled USB boot log."""
import argparse
import json
from pathlib import Path


def run(args):
    root = Path(__file__).resolve().parents[2]
    source = Path(args.log).resolve()
    output = Path(args.output).resolve()
    if not source.is_relative_to(root / "logs") or not output.is_relative_to(root / "data"):
        raise ValueError("输入须在项目 logs，输出须在项目 data")
    rows = [line.removeprefix("SECURE-PROBE-TRUST ") for line in source.read_text().splitlines()
            if line.startswith("SECURE-PROBE-TRUST ")]
    if len(rows) != 1:
        raise ValueError("需要恰好一次启动的 USB 公钥记录，不能混用多个启动")
    record = json.loads(rows[0])
    if set(record) != {"device", "boot", "public_key"}:
        raise ValueError("公钥记录字段不受支持")
    for name, length in [("device", 16), ("boot", 16), ("public_key", 32)]:
        values = record[name]
        if not isinstance(values, list) or len(values) != length or any(
                type(value) is not int or not 0 <= value <= 255 for value in values) or not any(values):
            raise ValueError(f"公钥记录的 {name} 无效")
    if bytes(record["device"]).hex() != args.device.lower():
        raise ValueError("不是本次指定的设备")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(record) + "\n")
    print("已提取 USB 启动身份及公钥；没有私钥；重启后此文件失效")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--log", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--device", required=True, help="本次受控设备的稳定标识，十六进制")
    run(parser.parse_args())
