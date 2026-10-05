import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { once } from "node:events";
import { test } from "node:test";
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { setTimeout as delay } from "node:timers/promises";
import { runCommand } from "./package-renderer.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));

test("正式 CLI 无例外授权时在任何引擎运行前拒绝，额外参数也拒绝", () => {
  const program = fileURLToPath(
    new URL("./package-renderer.mjs", import.meta.url),
  );
  const env = { ...process.env, TMPDIR: join(root, "tmp") };
  delete env.STAGEMASTER_ALLOW_PLATFORM_TEMP;
  const refusal = spawnSync(process.execPath, [program], {
    cwd: root,
    env,
    encoding: "utf8",
  });
  assert.equal(refusal.status, 1);
  assert.match(refusal.stderr, /用户允许/);
  const extra = spawnSync(process.execPath, [program, "unknown"], {
    cwd: root,
    env,
    encoding: "utf8",
  });
  assert.equal(extra.status, 1);
  assert.match(extra.stderr, /不接受额外参数/);
});

test("正式 CLI 缺引擎明确失败，不启动 UAT", () => {
  const program = fileURLToPath(
    new URL("./package-renderer.mjs", import.meta.url),
  );
  const result = spawnSync(process.execPath, [program], {
    cwd: root,
    env: {
      ...process.env,
      TMPDIR: join(root, "tmp"),
      STAGEMASTER_ALLOW_PLATFORM_TEMP: "1",
      STAGEMASTER_UE_ROOT: join(root, "tmp/not-installed-previs-engine"),
    },
    encoding: "utf8",
  });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /ENOENT/);
  assert.ok(!result.stdout.includes("执行："));
});
function setup(t) {
  const directory = mkdtempSync(join(root, "tmp/previs-process-test-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  return {
    directory,
    plan: {
      root,
      env: {
        TMPDIR: join(root, "tmp"),
        UE_SKIP_UBT_SDK_SETUP: "1",
        DYLD_LIBRARY_PATH: join(root, "tmp/outside-library"),
      },
    },
    command: {
      program: process.execPath,
      args: [],
      log: join(directory, "child.log"),
    },
  };
}

test("真实子进程参数／双输出／日志保留，构建环境不继承 SDK 跳过且 HOME 不变", async (t) => {
  const { plan, command } = setup(t);
  command.args = [
    "-e",
    'console.log(JSON.stringify({args:process.argv.slice(1),skip:process.env.UE_SKIP_UBT_SDK_SETUP,loader:process.env.DYLD_LIBRARY_PATH,home:process.env.HOME})); console.error("stderr");',
    "space argument",
  ];
  await runCommand(command, plan);
  const lines = readFileSync(command.log, "utf8").trim().split("\n");
  assert.deepEqual(JSON.parse(lines[0]), {
    args: ["space argument"],
    home: process.env.HOME,
  });
  assert.equal(lines[1], "stderr");
});

test("子进程非零退出保留输出并明确失败", async (t) => {
  const { plan, command } = setup(t);
  command.args = ["-e", 'console.error("actual failure");process.exit(7)'];
  await assert.rejects(runCommand(command, plan), /失败：7/);
  assert.match(readFileSync(command.log, "utf8"), /actual failure/);
});

test("成功回执给出真正所属进程 PID 与终态，不能拿其他 PID 证据替代", async (t) => {
  const { plan, command } = setup(t);
  command.args = ["-e", "console.log(process.pid)"];
  const result = await runCommand(command, plan);
  assert.equal(result.pid, Number(readFileSync(command.log, "utf8").trim()));
  assert.equal(result.code, 0);
  assert.equal(result.signal, null);
});

test("缺失可执行程序不挂起，不泄漏取消处理器", async (t) => {
  const { plan, command } = setup(t);
  command.program = join(root, "tmp/not-installed-previs-test");
  const before = process.listenerCount("SIGINT");
  await assert.rejects(runCommand(command, plan), { code: "ENOENT" });
  assert.equal(process.listenerCount("SIGINT"), before);
});

test("不能覆盖已有日志或在日志不可写时启动", async (t) => {
  const { plan, command } = setup(t);
  writeFileSync(command.log, "keep previous evidence");
  await assert.rejects(runCommand(command, plan), { code: "EEXIST" });
  assert.equal(readFileSync(command.log, "utf8"), "keep previous evidence");
});

test(
  "取消只终止所属进程组，实际子孙均收到终止",
  { timeout: 10000 },
  async (t) => {
    const { directory, plan, command } = setup(t);
    const parentMarker = join(directory, "parent-stopped");
    const grandchildMarker = join(directory, "grandchild-stopped");
    const grandchild = `const fs=require('node:fs');process.on('SIGTERM',()=>{fs.writeFileSync(${JSON.stringify(grandchildMarker)},'terminated');process.exit(0)});console.log('grandchild-ready');setInterval(()=>{},1000);`;
    command.args = [
      "-e",
      `const {spawn}=require('node:child_process');spawn(process.execPath,['-e',${JSON.stringify(grandchild)}],{stdio:'inherit'});process.on('SIGTERM',()=>{require('node:fs').writeFileSync(${JSON.stringify(parentMarker)},'terminated');setTimeout(()=>process.exit(0),100)});setInterval(()=>{},1000);`,
    ];
    const module = new URL("./package-renderer.mjs", import.meta.url).href;
    const wrapper = spawn(
      process.execPath,
      [
        "--input-type=module",
        "-e",
        `import {runCommand} from ${JSON.stringify(module)};try{await runCommand(${JSON.stringify(command)},${JSON.stringify(plan)})}catch{process.exitCode=1}`,
      ],
      {
        cwd: root,
        env: { ...process.env, TMPDIR: join(root, "tmp") },
        stdio: "ignore",
      },
    );
    t.after(() => wrapper.kill("SIGTERM"));
    const exit = once(wrapper, "exit");
    let ready = false;
    for (let attempt = 0; attempt < 50; attempt++) {
      if (
        existsSync(command.log) &&
        readFileSync(command.log, "utf8").includes("grandchild-ready")
      ) {
        ready = true;
        break;
      }
      await delay(50);
    }
    assert.ok(ready, "actual descendants ready before cancellation");
    wrapper.kill("SIGINT");
    const [code] = await exit;
    assert.equal(code, 1);
    assert.equal(readFileSync(parentMarker, "utf8"), "terminated");
    assert.equal(readFileSync(grandchildMarker, "utf8"), "terminated");
  },
);
