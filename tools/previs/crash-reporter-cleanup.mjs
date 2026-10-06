import { isDeepStrictEqual } from "node:util";
import { join } from "node:path";
import { reporterSystem } from "./crash-reporter-system.mjs";

export function ownedReporters(
  scope,
  system = reporterSystem,
  deadline = system.now() + 5000,
) {
  const prefix = `CrashReportClient ${scope.reportRoot}/`;
  const reporters = [];
  for (const row of system.rows()) {
    if (system.now() >= deadline) throw new Error("报告助手检查超过收尾期限");
    if (!row.command.startsWith(prefix)) continue;
    const match = row.command
      .slice(prefix.length)
      .match(
        /^((?:CrashReport|EnsureReport)-UE-StageMasterPreview-pid-([1-9]\d{0,9})-[A-Fa-f0-9]{32}) -Unattended(?: -(?:ImplicitSend|NoAnalytics))?$/,
      );
    if (!match || row.uid !== scope.uid)
      throw new Error("实例报告助手的参数或用户无法确认");
    if (Number(match[2]) > 2147483647)
      throw new Error("报告助手的来源 PID 无效");
    const report = join(scope.reportRoot, match[1]);
    system.reportDirectory(report);
    let executable;
    try {
      executable = system.executable(row.pid);
    } catch (error) {
      // The helper may exit between ps and lsof. Only fresh OS absence permits
      // accepting that race; a live/unconfirmable or reused PID stays a failure.
      const current = system.rows().find((item) => item.pid === row.pid);
      if (!current) continue;
      if (!isDeepStrictEqual(current, row))
        throw new Error("映像查询期间报告助手身份已变化");
      throw error;
    }
    if (executable !== scope.executable)
      throw new Error("实例报告助手的实际可执行文件不匹配");
    reporters.push({
      ...row,
      executable,
      report,
      sourcePid: Number(match[2]),
    });
    if (reporters.length > 16) throw new Error("实例报告助手数超过收尾预算");
  }
  if (system.now() >= deadline) throw new Error("报告助手检查超过收尾期限");
  return reporters;
}

export function prepareReporterCleanup(scope, system = reporterSystem) {
  if (ownedReporters(scope, system).length)
    throw new Error("实例已有报告助手，拒绝借用旧进程启动新验证");
}

export async function cleanupReporters(scope, system = reporterSystem) {
  const evidence = { status: "running", reporters: [] };
  const deadline = system.now() + 5000;
  try {
    for (const identity of ownedReporters(scope, system, deadline)) {
      const item = { ...identity, signal: null, ended: false };
      evidence.reporters.push(item);
      const current = ownedReporters(scope, system, deadline).find(
        (row) => row.pid === identity.pid,
      );
      if (!current) {
        item.ended = true;
        continue;
      }
      if (!isDeepStrictEqual(identity, current))
        throw new Error("报告助手身份已变化，拒绝向复用 PID 发信号");
      try {
        system.signal(identity.pid);
        item.signal = "SIGTERM";
      } catch (error) {
        if (error.code !== "ESRCH") throw error;
      }
      for (let attempt = 0; attempt < 20; attempt++) {
        await system.delay(100);
        const next = ownedReporters(scope, system, deadline).find(
          (row) => row.pid === identity.pid,
        );
        if (!next || !isDeepStrictEqual(identity, next)) {
          item.ended = true;
          break;
        }
      }
      if (!item.ended) throw new Error("报告助手在收尾期限内仍未结束");
    }
    if (ownedReporters(scope, system, deadline).length)
      throw new Error("命令结束后仍有未确认收尾的实例报告助手");
    evidence.status = "complete";
  } catch (error) {
    evidence.status = "failed";
    evidence.error = error.message;
  }
  return evidence;
}
