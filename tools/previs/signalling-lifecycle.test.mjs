import assert from "node:assert/strict";
import { randomBytes } from "node:crypto";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { createConnection } from "node:net";
import { createInterface } from "node:readline";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const within = (promise) =>
  Promise.race([
    promise,
    new Promise((_, reject) => {
      const timer = setTimeout(
        () => reject(new Error("所属信令生命周期超时")),
        2500,
      );
      timer.unref();
    }),
  ]);

test(
  "父进程 EOF 结束所属信令服务并释放两个真实回环端口",
  { timeout: 6000 },
  async () => {
    const child = spawn(
      process.execPath,
      [fileURLToPath(new URL("./signalling.mjs", import.meta.url))],
      {
        env: {
          ...process.env,
          STAGEMASTER_STREAM_RENDERER_TOKEN: randomBytes(32).toString("hex"),
          STAGEMASTER_STREAM_VIEWER_TOKEN: randomBytes(32).toString("hex"),
        },
        stdio: ["pipe", "pipe", "pipe"],
      },
    );
    const exit = once(child, "exit");
    const lines = createInterface({ input: child.stdout });
    try {
      const [line] = await within(once(lines, "line"));
      const ports = JSON.parse(line);
      assert.deepEqual(Object.keys(ports).sort(), [
        "rendererPort",
        "viewerPort",
      ]);
      assert.notEqual(ports.rendererPort, ports.viewerPort);
      for (const port of Object.values(ports))
        assert.ok(Number.isInteger(port) && port > 0 && port <= 65535);
      child.stdin.end();
      assert.deepEqual(await within(exit), [0, null]);
      for (const port of Object.values(ports)) {
        const socket = createConnection({ host: "127.0.0.1", port });
        try {
          const [error] = await within(once(socket, "error"));
          assert.equal(error.code, "ECONNREFUSED");
        } finally {
          socket.destroy();
        }
      }
    } finally {
      lines.close();
      if (child.exitCode === null && child.signalCode === null) child.kill();
      await within(exit);
    }
  },
);
