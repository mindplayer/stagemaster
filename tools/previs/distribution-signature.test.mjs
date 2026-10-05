import assert from "node:assert/strict";
import { test } from "node:test";
import {
  parseDistributionSignature,
  developerIdRequirement,
} from "./distribution-signature.mjs";
const signed =
  "Identifier=node\nCodeDirectory v=20500 size=42 flags=0x10000(runtime) hashes=1+1 location=embedded\nTeamIdentifier=HX7739G8FX\nAuthority=Developer ID Application: Node.js Foundation\nTimestamp=Jun 18, 2026 at 02:18:26\n";
test("有效CodeDirectory后追加畸形重复行仍拒绝", () => {
  assert.throws(
    () => parseDistributionSignature(signed + "CodeDirectory flags=unknown\n"),
    /重复/,
  );
});
test("解析实际Node签名字段，不以Authority文本取代信任", () => {
  const parsed = parseDistributionSignature(signed);
  assert.equal(parsed.hardenedRuntime, true);
  assert.equal(parsed.adHoc, false);
  assert.equal(parsed.teamId, "HX7739G8FX");
  assert.equal("trusted" in parsed, false);
  assert.match(developerIdRequirement, /anchor apple generic/);
  assert.match(
    developerIdRequirement,
    /certificate 1\[field\.1\.2\.840\.113635\.100\.6\.2\.6\]/,
  );
  assert.match(
    developerIdRequirement,
    /certificate leaf\[field\.1\.2\.840\.113635\.100\.6\.1\.13\]/,
  );
});
test("解析实际ad-hoc无Team无Timestamp", () => {
  const p = parseDistributionSignature(
    signed
      .replace("0x10000(runtime)", "0x2(adhoc)")
      .replace("HX7739G8FX", "not set")
      .replace(/^Timestamp=.*\n/m, ""),
  );
  assert.equal(p.adHoc, true);
  assert.equal(p.teamId, null);
  assert.equal(p.timestamp, null);
});
test("runtime字样不能代替runtime位", () => {
  const p = parseDistributionSignature(
    signed.replace("0x10000(runtime)", "0x2(runtime)"),
  );
  assert.equal(p.hardenedRuntime, false);
});
test("重复／缺失Identifier、Team、CodeDirectory和Timestamp拒绝", () => {
  for (const line of [
    "Identifier=node\n",
    "TeamIdentifier=HX7739G8FX\n",
    "Timestamp=Jun 18, 2026 at 02:18:26\n",
    signed.split("\n")[1] + "\n",
  ])
    assert.throws(() => parseDistributionSignature(signed + line), /重复/);
  assert.throws(
    () => parseDistributionSignature(signed.replace(/^Identifier=.*\n/m, "")),
    /缺失/,
  );
});
test("未知标志格式、超长、控制字符和非法Team拒绝", () => {
  for (const value of [
    signed.replace("flags=0x10000", "flags=runtime"),
    signed.replace("HX7739G8FX", "bad"),
    signed + "\0",
    "x".repeat(4 * 1024 * 1024 + 1),
  ])
    assert.throws(() => parseDistributionSignature(value), /无效|缺失|超限/);
});
