import { execFileSync } from "node:child_process";
import { lstatSync, realpathSync } from "node:fs";
import { performance } from "node:perf_hooks";
import { setTimeout as delay } from "node:timers/promises";

function query(program, args) {
  return execFileSync(program, args, {
    encoding: "utf8",
    timeout: 2500,
    maxBuffer: 4 * 1024 * 1024,
    env: { PATH: "/usr/bin:/bin:/usr/sbin", LC_ALL: "C" },
  });
}

export function processRows(output) {
  const lines = output.trim().split("\n").filter(Boolean);
  if (lines.length > 4096) throw new Error("报告助手进程检查超出预算");
  return lines.map((line) => {
    const match = line
      .trim()
      .match(
        /^(\d+)\s+(\d+)\s+(\w{3} \w{3}\s+\d{1,2} \d{2}:\d{2}:\d{2} \d{4})\s+(.+)$/,
      );
    if (!match) throw new Error("报告助手进程身份信息不完整");
    const pid = Number(match[1]);
    const uid = Number(match[2]);
    if (!Number.isSafeInteger(pid) || pid <= 0 || !Number.isSafeInteger(uid))
      throw new Error("报告助手进程身份无效");
    return { pid, uid, started: match[3], command: match[4] };
  });
}

export function loadedExecutable(output, pid) {
  const lines = output.trim().split("\n");
  if (lines[0] !== `p${pid}`) throw new Error("报告助手映像的 PID 不匹配");
  const file = lines.find((line) => line.startsWith("n"));
  if (!file) throw new Error("报告助手主可执行文件无法确认");
  return file.slice(1);
}

export const reporterSystem = {
  now: () => performance.now(),
  rows() {
    return processRows(
      query("/bin/ps", ["-ax", "-o", "pid=,uid=,lstart=,args="]),
    );
  },
  executable(pid) {
    return loadedExecutable(
      query("/usr/sbin/lsof", ["-a", "-p", String(pid), "-d", "txt", "-Fn"]),
      pid,
    );
  },
  signal(pid) {
    process.kill(pid, "SIGTERM");
  },
  reportDirectory(path) {
    if (!lstatSync(path).isDirectory() || realpathSync(path) !== path)
      throw new Error("报告助手目录不存在或经过链接");
  },
  delay,
};
