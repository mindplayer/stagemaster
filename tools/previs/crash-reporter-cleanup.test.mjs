import assert from "node:assert/strict";
import { test } from "node:test";
import { mkdirSync, mkdtempSync, rmSync, symlinkSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  cleanupReporters,
  ownedReporters,
  prepareReporterCleanup,
} from "./crash-reporter-cleanup.mjs";
import {
  loadedExecutable,
  processRows,
  reporterSystem,
} from "./crash-reporter-system.mjs";

const scope = {
  reportRoot: "/project/tmp/previs-package-x/platform-user/中文 有空格/Crashes",
  executable: "/engine/CrashReportClientEditor",
  uid: 501,
};
const name =
  "CrashReport-UE-StageMasterPreview-pid-123-00112233445566778899AABBCCDDEEFF";
const identity = {
  pid: 999,
  uid: 501,
  started: "Tue Oct  6 12:34:56 2026",
  command: `CrashReportClient ${scope.reportRoot}/${name} -Unattended`,
};
function system(rows = [identity]) {
  const signals = [];
  return {
    rows: () => rows,
    executable: () => scope.executable,
    reportDirectory: () => {},
    now: () => 0,
    delay: async () => {},
    signal: (pid) => {
      signals.push(pid);
      rows = rows.filter((row) => row.pid !== pid);
    },
    signals,
  };
}

test("完整系统进程信息保留空格／中文参数，加载文件须绑定准确 PID", () => {
  assert.deepEqual(
    processRows(` 999 501 ${identity.started} ${identity.command}\n`),
    [identity],
  );
  assert.equal(
    loadedExecutable("p999\nftxt\nn/engine/main\nftxt\nn/library\n", 999),
    "/engine/main",
  );
  assert.throws(() => loadedExecutable("p998\nn/engine/main", 999), /PID/);
  assert.throws(() => loadedExecutable("p999\nftxt", 999), /无法确认/);
});

test("系统输出不完整、PID非法或进程总预算超限拒绝", () => {
  for (const text of ["999 broken", "0 501 Tue Oct  6 12:34:56 2026 cmd"])
    assert.throws(() => processRows(text));
  assert.throws(() => processRows("x\n".repeat(4097)), /预算/);
  assert.deepEqual(processRows(""), []);
});

test("准确非交互助手收尾并保留来源，未匹配实例不发送信号", async () => {
  const foreign = {
    ...identity,
    pid: 998,
    command: identity.command.replace("package-x", "package-y"),
  };
  const adapter = system([identity, foreign]);
  const result = await cleanupReporters(scope, adapter);
  assert.equal(result.status, "complete");
  assert.equal(result.reporters[0].sourcePid, 123);
  assert.equal(result.reporters[0].ended, true);
  assert.deepEqual(adapter.signals, [999]);
  assert.deepEqual(adapter.rows(), [foreign]);
});

test("已有报告助手拒绝启动新命令，不借旧进程资格", () => {
  const adapter = system();
  assert.throws(() => prepareReporterCleanup(scope, adapter), /已有/);
  assert.deepEqual(adapter.signals, []);
  prepareReporterCleanup(scope, system([]));
});

for (const [label, change] of [
  ["UID", { uid: 502 }],
  ["交互", { command: identity.command.replace(" -Unattended", "") }],
  ["未知参数", { command: `${identity.command} -unknown` }],
  ["额外参数", { command: `${identity.command} -NoAnalytics -ImplicitSend` }],
  ["目录穿越", { command: identity.command.replace(name, `../${name}`) }],
  [
    "其他产品",
    { command: identity.command.replace("StageMasterPreview", "OtherGame") },
  ],
  [
    "来源 PID",
    { command: identity.command.replace("pid-123-", "pid-9999999999-") },
  ],
]) {
  test(`${label}不符时明确失败且无信号`, async () => {
    const adapter = system([{ ...identity, ...change }]);
    assert.equal((await cleanupReporters(scope, adapter)).status, "failed");
    assert.deepEqual(adapter.signals, []);
  });
}

test("EnsureReport／引擎已知附加参数保持受限支持", () => {
  for (const command of [
    `${identity.command} -ImplicitSend`,
    `${identity.command.replace("CrashReport-UE-", "EnsureReport-UE-")} -NoAnalytics`,
  ])
    assert.equal(
      ownedReporters(scope, system([{ ...identity, command }])).length,
      1,
    );
});

for (const label of ["映像", "目录链接", "系统探测"]) {
  test(`${label}无法确认时不杀进程、不假完成`, async () => {
    const adapter = system();
    if (label === "映像") adapter.executable = () => "/other/program";
    else if (label === "目录链接")
      adapter.reportDirectory = () => {
        throw new Error("linked");
      };
    else
      adapter.rows = () => {
        throw new Error("probe failed");
      };
    assert.equal((await cleanupReporters(scope, adapter)).status, "failed");
    assert.deepEqual(adapter.signals, []);
  });
}

test("发送前启动时间改变的复用 PID 拒绝，不能杀新进程", async () => {
  const adapter = system();
  let queries = 0;
  adapter.rows = () => [
    ++queries === 1
      ? identity
      : { ...identity, started: "Tue Oct  6 12:35:56 2026" },
  ];
  assert.equal((await cleanupReporters(scope, adapter)).status, "failed");
  assert.deepEqual(adapter.signals, []);
});

test("查询后已结束或 ESRCH 仍须独立确认消失", async () => {
  for (const signalRace of [false, true]) {
    const adapter = system();
    let queries = 0;
    adapter.rows = () => (++queries <= (signalRace ? 2 : 1) ? [identity] : []);
    adapter.signal = () => {
      throw Object.assign(new Error("gone"), { code: "ESRCH" });
    };
    const result = await cleanupReporters(scope, adapter);
    assert.equal(result.status, "complete");
    assert.equal(result.reporters[0].ended, true);
  }
});

test("拒绝终止、迟到的新助手和忽略 SIGTERM 均留失败，无强制升级", async () => {
  for (const mode of ["refusal", "late", "ignore"]) {
    const adapter = system();
    let queries = 0;
    if (mode === "refusal")
      adapter.signal = () => {
        throw new Error("EPERM");
      };
    else if (mode === "late")
      adapter.rows = () =>
        ++queries <= 2
          ? [identity]
          : queries === 3
            ? []
            : [{ ...identity, pid: 1000 }];
    else adapter.signal = (pid) => adapter.signals.push(pid);
    assert.equal((await cleanupReporters(scope, adapter)).status, "failed");
    assert.ok(adapter.signals.length <= 1);
  }
});

test("助手预算和总收尾期限是硬失败，不能无限扫描或继续发送", async () => {
  const tooMany = system(
    Array.from({ length: 17 }, (_, n) => ({ ...identity, pid: 1000 + n })),
  );
  assert.equal((await cleanupReporters(scope, tooMany)).status, "failed");
  assert.deepEqual(tooMany.signals, []);
  const expired = system();
  let clock = 0;
  expired.now = () => (clock += 6000);
  assert.equal((await cleanupReporters(scope, expired)).status, "failed");
  assert.deepEqual(expired.signals, []);
});

test("实际目录探测拒绝缺失／链接报告目录，未读取报告内容", (t) => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const temporary = mkdtempSync(join(root, "tmp/previs-reporter-directory-"));
  t.after(() => rmSync(temporary, { recursive: true, force: true }));
  const report = join(temporary, "report");
  mkdirSync(report);
  reporterSystem.reportDirectory(report);
  const linked = join(temporary, "linked");
  symlinkSync(report, linked);
  assert.throws(() => reporterSystem.reportDirectory(linked), /链接/);
  assert.throws(() =>
    reporterSystem.reportDirectory(join(temporary, "missing")),
  );
});

test("助手在 ps 与映像查询之间真正结束须复核，不将探测错误误报为遗留", async () => {
  for (const ended of [true, false]) {
    const adapter = system();
    let queries = 0;
    adapter.rows = () => (++queries <= 3 || !ended ? [identity] : []);
    adapter.signal = (pid) => adapter.signals.push(pid);
    adapter.executable = () => {
      if (queries >= 3) throw new Error("lsof failed");
      return scope.executable;
    };
    const result = await cleanupReporters(scope, adapter);
    assert.equal(result.status, ended ? "complete" : "failed");
    assert.equal(result.reporters[0].ended, ended);
    assert.deepEqual(adapter.signals, [999]);
  }
});
