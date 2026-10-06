import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { test } from "node:test";
import {
  chmodSync,
  cpSync,
  closeSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { setTimeout as delay } from "node:timers/promises";
import { runCommand } from "./package-renderer.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));

for (const mode of ["failure", "success", "cancel", "cancel-cleanup"])
  test(`实际独立报告助手在${mode}后收尾，报告文件保持`, async (t) => {
    const temporary = mkdtempSync(join(root, "tmp/previs-package-"));
    const engine = join(temporary, "engine");
    const program = join(engine, "Engine/Build/BatchFiles/RunUAT.sh");
    const executable = join(
      engine,
      "Engine/Binaries/Mac/CrashReportClientEditor.app/Contents/MacOS/CrashReportClientEditor",
    );
    const home = join(temporary, "platform-user");
    const reports = join(
      home,
      "Library/Application Support/Epic/UnrealEngine/5.8/Saved/Crashes",
    );
    const report = join(
      reports,
      "CrashReport-UE-StageMasterPreview-pid-12345-00112233445566778899AABBCCDDEEFF",
    );
    const marker = join(report, "stopped");
    const ready = join(report, "ready");
    const sentinel = join(report, "original-report.txt");
    const fixtureLog = join(temporary, "reporter-fixture.log");
    const wrapperLog = join(temporary, "wrapper.log");
    const spawned = join(temporary, "reporter-spawned");
    const parentReady = join(temporary, "parent-ready");
    t.after(async () => {
      if (existsSync(wrapperLog)) console.log(readFileSync(wrapperLog, "utf8"));
      if (!existsSync(ready) && existsSync(fixtureLog))
        console.error(readFileSync(fixtureLog, "utf8"));
      if (existsSync(spawned)) {
        const pid = Number(readFileSync(spawned, "utf8"));
        try {
          process.kill(pid, "SIGTERM");
        } catch (error) {
          if (error.code !== "ESRCH") throw error;
        }
        for (let attempt = 0; attempt < 40; attempt++) {
          try {
            process.kill(pid, 0);
          } catch (error) {
            if (error.code !== "ESRCH") throw error;
            break;
          }
          await delay(50);
        }
      }
      rmSync(temporary, { recursive: true, force: true });
    });
    for (const path of [report, join(engine, "Engine/Build/BatchFiles")])
      mkdirSync(path, { recursive: true });
    mkdirSync(join(executable, ".."), { recursive: true });
    cpSync(process.execPath, executable);
    writeFileSync(join(report, "package.json"), '{"main":"reporter.cjs"}');
    writeFileSync(
      join(report, "reporter.cjs"),
      `const fs=require('node:fs');process.on('SIGTERM',()=>{fs.writeFileSync(${JSON.stringify(marker)},'terminated');setTimeout(()=>process.exit(0),${mode === "cancel-cleanup" ? 700 : 0})});fs.writeFileSync(${JSON.stringify(ready)},String(process.pid));setInterval(()=>{},1000);`,
    );
    writeFileSync(sentinel, "retain crash evidence");
    const descriptor = openSync(fixtureLog, "wx");
    closeSync(descriptor);
    writeFileSync(
      program,
      `#!${process.execPath}\nconst {spawn}=require('node:child_process');const fs=require('node:fs');const fd=fs.openSync(${JSON.stringify(fixtureLog)},'a');const child=spawn(${JSON.stringify(executable)},[${JSON.stringify(report)},'-Unattended'],{argv0:'CrashReportClient',detached:true,stdio:['ignore',fd,fd]});fs.closeSync(fd);fs.writeFileSync(${JSON.stringify(spawned)},String(child.pid));child.on('error',error=>console.error(error));child.unref();let attempts=0;const timer=setInterval(()=>{if(fs.existsSync(${JSON.stringify(ready)})){clearInterval(timer);fs.writeFileSync(${JSON.stringify(parentReady)},'ready');${mode === "cancel" ? "setInterval(()=>{},1000)" : `process.exit(${mode === "success" || mode === "cancel-cleanup" ? 0 : 7})`}}else if(++attempts===500){process.exit(9)}},20);`,
    );
    chmodSync(program, 0o700);
    const command = {
      program,
      args: [],
      log: join(temporary, "command.log"),
      crashReporters: "editor",
    };
    const plan = {
      root,
      engine,
      temporary,
      env: { TMPDIR: join(root, "tmp"), CFFIXED_USER_HOME: home },
    };
    if (mode === "failure") {
      await assert.rejects(runCommand(command, plan), (error) => {
        assert.match(error.message, /失败：7/);
        assert.equal(error.crashReporters.status, "complete");
        return true;
      });
    } else if (mode === "success") {
      const completed = await runCommand(command, plan);
      assert.equal(completed.code, 0);
      assert.equal(completed.crashReporters.status, "complete");
    } else {
      const module = new URL("./package-renderer.mjs", import.meta.url).href;
      const wrapperDescriptor = openSync(wrapperLog, "wx");
      const wrapper = spawn(
        process.execPath,
        [
          "--input-type=module",
          "-e",
          `import {runCommand} from ${JSON.stringify(module)};try{await runCommand(${JSON.stringify(command)},${JSON.stringify(plan)})}catch(error){console.log(JSON.stringify({wrapperError:error.message,cleanup:error.crashReporters}));if(!error.message.includes('已取消')||error.crashReporters?.status!=='complete')process.exitCode=2;else process.exitCode=1;}`,
        ],
        {
          cwd: root,
          env: { ...process.env, TMPDIR: join(root, "tmp") },
          stdio: ["ignore", wrapperDescriptor, wrapperDescriptor],
        },
      );
      closeSync(wrapperDescriptor);
      const exit = once(wrapper, "exit");
      t.after(() => wrapper.kill("SIGTERM"));
      const cancellationReady =
        mode === "cancel-cleanup" ? marker : parentReady;
      for (
        let attempt = 0;
        attempt < 600 && !existsSync(cancellationReady);
        attempt++
      )
        await delay(20);
      assert.ok(
        existsSync(cancellationReady),
        "actual detached helper ready before one explicit cancellation",
      );
      wrapper.kill("SIGINT");
      assert.equal((await exit)[0], 1);
    }
    assert.equal(readFileSync(marker, "utf8"), "terminated");
    assert.equal(readFileSync(sentinel, "utf8"), "retain crash evidence");
    const cleanup = JSON.parse(
      readFileSync(`${command.log}.crash-reporters.json`, "utf8"),
    );
    assert.equal(cleanup.status, "complete");
    assert.equal(cleanup.reporters.length, 1);
    assert.equal(cleanup.reporters[0].signal, "SIGTERM");
    assert.equal(cleanup.reporters[0].ended, true);
    console.log(JSON.stringify({ case: mode, fixtureOnly: true, cleanup }));
  });

test("已有助手收尾记录在执行前拒绝，旧证据与日志不覆盖", async (t) => {
  const temporary = mkdtempSync(join(root, "tmp/previs-package-"));
  t.after(() => rmSync(temporary, { recursive: true, force: true }));
  const engine = join(temporary, "engine");
  const command = {
    program: join(engine, "Engine/Build/BatchFiles/RunUAT.sh"),
    args: [],
    log: join(temporary, "never-run.log"),
    crashReporters: "editor",
  };
  const record = `${command.log}.crash-reporters.json`;
  writeFileSync(record, "keep original evidence");
  await assert.rejects(
    runCommand(command, {
      root,
      engine,
      temporary,
      env: { CFFIXED_USER_HOME: join(temporary, "platform-user") },
    }),
    /已有报告助手收尾记录/,
  );
  assert.equal(readFileSync(record, "utf8"), "keep original evidence");
  assert.ok(!existsSync(command.log), "no process or new log before refusal");
});
