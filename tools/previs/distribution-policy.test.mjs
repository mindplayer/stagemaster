import assert from "node:assert/strict";
import { test } from "node:test";
import {
  assessDistributionPrerequisites,
  distributionRoles,
} from "./distribution-policy.mjs";

const fixture = () => ({
  minimumMacOS: "14.0",
  closureMinimumMacOS: "14.0",
  images: distributionRoles.map((role) => ({
    role,
    program: `/project/${role}`,
    minimumMacOS: role === "node" ? "13.5" : "14.0",
    architectures: ["arm64"],
    executable: true,
    strict: { status: 0 },
    developerId: { status: 0 },
    entitlements: { status: 0, values: {} },
    signature: {
      identifier: role,
      teamId: role === "node" ? "HX7739G8FX" : "ABCDEFGHIJ",
      flags: 0x10000,
      adHoc: false,
      hardenedRuntime: true,
      timestamp: "Jun 18, 2026 at 02:18:26",
      authorities: ["Developer ID Application: Vendor"],
    },
  })),
});
const blocked = (change, code) => {
  const evidence = fixture();
  change(evidence.images[3], evidence);
  const result = assessDistributionPrerequisites(evidence);
  assert.equal(result.status, "blocked");
  assert.ok(result.issues.some((issue) => issue.code === code));
  assert.equal(result.releaseQualified, false);
};
test("静态条件满足且第三方合法不同Team只称先决通过，不是客户发行", () => {
  const result = assessDistributionPrerequisites(fixture());
  assert.equal(result.status, "static-prerequisites-passed");
  assert.equal(result.releaseQualified, false);
  assert.ok(result.remaining.length >= 4);
});
test("严格封套通过的ad-hoc仍拒绝", () =>
  blocked((i) => {
    i.signature.adHoc = true;
    i.signature.flags = 2;
  }, "ad-hoc"));
test("Authority文字正确但实际Apple证书要求失败仍拒绝", () =>
  blocked((i) => {
    i.developerId.status = 1;
  }, "developer-id"));
test("没有Team不能靠签名退出零合格", () =>
  blocked((i) => {
    i.signature.teamId = null;
  }, "team"));
test("没有Hardened Runtime拒绝", () =>
  blocked((i) => {
    i.signature.hardenedRuntime = false;
    i.signature.flags = 0;
  }, "runtime"));
test("没有安全时间戳拒绝，Signed Time不替代", () =>
  blocked((i) => {
    i.signature.timestamp = null;
  }, "timestamp"));
test("Game调试资格true拒绝", () =>
  blocked((i) => {
    i.entitlements.values["com.apple.security.get-task-allow"] = true;
  }, "debug"));
test("调试资格字符串false是畸形，不获得通过", () =>
  blocked((i) => {
    i.entitlements.values["com.apple.security.get-task-allow"] = "false";
  }, "debug"));
test("绝对路径读写开发资格拒绝", () =>
  blocked((i) => {
    i.entitlements.values[
      "com.apple.security.temporary-exception.files.absolute-path.read-write"
    ] = ["/project/tmp/"];
  }, "absolute-path"));
test("绝对路径只读也不是可移动资格", () =>
  blocked((i) => {
    i.entitlements.values[
      "com.apple.security.temporary-exception.files.absolute-path.read-only"
    ] = "/project/data/";
  }, "absolute-path"));
test("读取资格失败不能按空资格通过", () =>
  blocked((i) => {
    i.entitlements = { status: 1, values: null };
  }, "entitlements"));
test("最低系统低于Game依赖拒绝", () =>
  blocked((i, e) => {
    e.minimumMacOS = "13.5";
  }, "minimum-os"));
test("最低系统低于Node入口要求拒绝", () =>
  blocked((i, e) => {
    e.images[2].minimumMacOS = "15.0";
  }, "minimum-os"));
test("内部验收标识不能作为发行候选", () =>
  blocked((i, e) => {
    e.images[0].signature.identifier = "cn.stagemaster.acceptance.instance";
  }, "internal-id"));
test("缺失签名元数据不能获得通过", () =>
  blocked((i) => {
    i.signature = null;
  }, "signature-metadata"));
test("不重复角色／不丢角色", () => {
  const a = fixture();
  a.images.pop();
  assert.throws(() => assessDistributionPrerequisites(a), /四个/);
  const b = fixture();
  b.images[3].role = "node";
  assert.throws(() => assessDistributionPrerequisites(b), /角色/);
});
test("签名与工具状态未知不能默认为零", () => {
  const a = fixture();
  a.images[0].strict.status = null;
  assert.throws(() => assessDistributionPrerequisites(a), /元数据/);
});
