import assert from "node:assert/strict";
import {
  chmodSync,
  cpSync,
  readFileSync,
  renameSync,
  symlinkSync,
  truncateSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { fixture } from "./notice-handoff-fixtures.mjs";
import { assembleNoticeHandoff } from "./notice-handoff.mjs";
import { readHandoffInputs } from "./notice-handoff-inputs.mjs";
import { permissionTree, saveHandoffFile } from "./notice-handoff-files.mjs";
import { fileInventory } from "../previs/signalling-package-files.mjs";
const inspect = () =>
  ["desktop", "host", "renderer", "node"].map((role) => ({
    role,
    verifyExit: 0,
    readExit: 0,
    details: "test-only not a real signature\n",
  }));
const assemble = (f, dependencies = {}) =>
  assembleNoticeHandoff(f.root, f.referencePath, f.materials, f.destination, {
    inspect,
    ...dependencies,
  });
const receipt = (f) =>
  JSON.parse(readFileSync(join(f.output, "handoff.json"), "utf8"));
test("完整复制、材料原字节、来源、执行位保留且不转移客户资格", async (t) => {
  const f = await fixture(t),
    original = await fileInventory(f.bundle),
    modes = permissionTree(f.bundle),
    record = await assemble(f);
  assert.equal(record.status, "internal-review-materials-ready");
  assert.equal(record.automaticLaunch, false);
  assert.equal(record.customerReleaseQualified, false);
  assert.equal(record.commercialReleaseApproved, false);
  assert.equal(record.sourceNativeQualificationTransferred, false);
  assert.deepEqual(
    await fileInventory(join(f.output, record.copiedBundle)),
    original,
  );
  assert.deepEqual(permissionTree(join(f.output, record.copiedBundle)), modes);
  assert.deepEqual(await fileInventory(f.bundle), original);
  assert.deepEqual(permissionTree(f.bundle), modes);
  for (const name of ["notices.json", "THIRD-PARTY-NOTICES.txt"])
    assert.deepEqual(
      readFileSync(join(f.output, "licenses", name)),
      readFileSync(join(f.root, f.materials, "licenses", name)),
    );
  assert.match(
    readFileSync(join(f.output, "README.md"), "utf8"),
    /请勿从此副本启动/,
  );
  await assert.rejects(assemble(f), /已存在/);
  assert.deepEqual(receipt(f), record);
});
test("复制过程中失败留下failed回执，不覆盖来源", async (t) => {
  const f = await fixture(t),
    original = await fileInventory(f.bundle);
  await assert.rejects(
    assemble(f, {
      copy() {
        throw new Error("controlled copy failure");
      },
    }),
    /controlled copy failure/,
  );
  assert.equal(receipt(f).status, "failed");
  assert.deepEqual(await fileInventory(f.bundle), original);
});
test("复制后的字节或执行位变化拒绝，不能给半份副本完成状态", async (t) => {
  for (const mutation of ["bytes", "mode"]) {
    const f = await fixture(t);
    await assert.rejects(
      assemble(f, {
        copy(source, target) {
          cpSync(source, target, { recursive: true });
          const file = join(target, "Contents/MacOS/stagemaster");
          if (mutation === "bytes") writeFileSync(file, "changed\n");
          else chmodSync(file, 0o644);
        },
      }),
      /复制字节|复制权限/,
    );
    assert.equal(receipt(f).status, "failed");
  }
});
test("副本包内链接与原元数据不同拒绝", async (t) => {
  const f = await fixture(t);
  await assert.rejects(
    assemble(f, {
      copy(source, target) {
        cpSync(source, target, { recursive: true });
        symlinkSync("stagemaster", join(target, "Contents/MacOS/extra"));
      },
    }),
    /复制字节/,
  );
  assert.equal(receipt(f).status, "failed");
});
test("签名失败或复制后资格变更拒绝，不尝试重签", async (t) => {
  const f = await fixture(t);
  await assert.rejects(
    assemble(f, {
      inspect() {
        throw new Error("controlled signature refusal");
      },
    }),
    /signature refusal/,
  );
  assert.equal(receipt(f).status, "failed");
  const g = await fixture(t);
  let calls = 0;
  await assert.rejects(
    assemble(g, {
      inspect() {
        const entries = inspect();
        if (calls++) entries[0].details = "changed permission\n";
        return entries;
      },
    }),
    /签名／权限/,
  );
  assert.equal(calls, 2);
  assert.equal(receipt(g).status, "failed");
});
test("旁置材料写入失败不标完成，不改原材料", async (t) => {
  const f = await fixture(t),
    original = readFileSync(join(f.root, f.materials, "licenses/notices.json"));
  await assert.rejects(
    assemble(f, {
      write(file, bytes) {
        if (file.endsWith("THIRD-PARTY-NOTICES.txt"))
          throw new Error("controlled write failure");
        saveHandoffFile(file, bytes);
      },
    }),
    /write failure/,
  );
  assert.equal(receipt(f).status, "failed");
  assert.deepEqual(
    readFileSync(join(f.root, f.materials, "licenses/notices.json")),
    original,
  );
});
test("复制期间原文变化或产品來源变化必须拒绝", async (t) => {
  for (const mutation of ["notice", "source"]) {
    const f = await fixture(t);
    await assert.rejects(
      assemble(f, {
        copy(source, target) {
          cpSync(source, target, { recursive: true });
          writeFileSync(
            join(
              f.root,
              mutation === "notice"
                ? f.materials + "/licenses/THIRD-PARTY-NOTICES.txt"
                : "Cargo.toml",
            ),
            "changed in test\n",
          );
        },
      }),
      /在复制中变化/,
    );
    assert.equal(receipt(f).status, "failed");
  }
});
test("输入正文NUL、非UTF8与输入目录链接拒绝", async (t) => {
  const f = await fixture(t);
  f.write(
    f.materials + "/licenses/THIRD-PARTY-NOTICES.txt",
    Buffer.from([0xff]),
  );
  await assert.rejects(
    readHandoffInputs(f.root, f.referencePath, f.materials, f.destination),
    /encoded data|UTF/,
  );
  const g = await fixture(t);
  g.write(
    g.materials + "/licenses/THIRD-PARTY-NOTICES.txt",
    Buffer.from([0x41, 0]),
  );
  await assert.rejects(
    readHandoffInputs(g.root, g.referencePath, g.materials, g.destination),
    /NUL/,
  );
});
test("回执路径被链接替换不得改写其指向的源文件", async (t) => {
  const f = await fixture(t),
    original = readFileSync(join(f.root, "Cargo.toml"));
  await assert.rejects(
    assemble(f, {
      copy() {
        const p = join(f.output, "handoff.json");
        renameSync(p, p + ".original");
        symlinkSync(join(f.root, "Cargo.toml"), p);
        throw new Error("controlled replacement");
      },
    }),
    /回执所有权/,
  );
  assert.deepEqual(readFileSync(join(f.root, "Cargo.toml")), original);
  assert.equal(
    JSON.parse(readFileSync(join(f.output, "handoff.json.original"), "utf8"))
      .status,
    "assembling",
  );
});
test("候选单文件字节预算超限在读哈希前拒绝", async (t) => {
  const f = await fixture(t);
  truncateSync(join(f.bundle, "Contents/MacOS/stagemaster"), 1024 ** 3 + 1);
  await assert.rejects(
    readHandoffInputs(f.root, f.referencePath, f.materials, f.destination),
    /字节预算/,
  );
});
test("签名诊断入口重复或正文超限拒绝，不把有限材料变成无界日志", async (t) => {
  for (const mutation of ["duplicate", "budget"]) {
    const f = await fixture(t);
    await assert.rejects(
      assemble(f, {
        inspect() {
          const entries = inspect();
          if (mutation === "duplicate") entries[1].role = "desktop";
          else entries[0].details = "x".repeat(64 * 1024 + 1);
          return entries;
        },
      }),
      /入口身份|预算超限/,
    );
    assert.equal(receipt(f).status, "failed");
  }
});
test("签名元数据含相似验证文字也不得被路径归一化隐藏", async (t) => {
  const f = await fixture(t);
  let calls = 0;
  await assert.rejects(
    assemble(f, {
      inspect() {
        const entries = inspect();
        entries[0].details +=
          "Authority=" +
          (calls++ ? "Changed" : "Original") +
          ": valid on disk\n";
        return entries;
      },
    }),
    /签名／权限/,
  );
  assert.equal(receipt(f).status, "failed");
});
