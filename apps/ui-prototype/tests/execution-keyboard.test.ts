import { test } from "node:test";
import assert from "node:assert/strict";
import {
  ExecutionKeyGate,
  type ExecutionKeyInput,
  type ExecutionKeyState,
} from "../src/execution-keyboard.ts";

const input = (code: string, extra: Partial<ExecutionKeyInput> = {}) => ({
  code,
  repeat: false,
  isComposing: false,
  altKey: false,
  ctrlKey: false,
  metaKey: false,
  shiftKey: false,
  ...extra,
});
const ready: ExecutionKeyState = {
  busy: false,
  ready: true,
  canNext: true,
  status: "idle",
};

test("空格每次松开只推进一次，忽略长按与未带 repeat 标记的重复事件", () => {
  const gate = new ExecutionKeyGate();
  assert.equal(gate.press(input("Space"), ready).command, "next");
  for (let i = 0; i < 100; i++) {
    assert.deepEqual(
      gate.press(input("Space", { repeat: i % 2 === 0 }), ready),
      {
        handled: true,
      },
    );
  }
  gate.release("KeyP");
  assert.equal(gate.press(input("Space"), ready).command, undefined);
  gate.release("Space");
  assert.equal(gate.press(input("Space"), ready).command, "next");
  gate.reset();
  // Re-entering while a physical key remains held must not trigger a repeat.
  assert.equal(
    gate.press(input("Space", { repeat: true }), ready).command,
    undefined,
  );
  assert.equal(gate.press(input("Space"), ready).command, undefined);
  gate.release("Space");
  assert.equal(gate.press(input("Space"), ready).command, "next");
});

test("忙、旧版本和无下一步的按键丢弃，不在解除限制后补发", () => {
  for (const blocked of [
    { busy: true },
    { ready: false },
    { canNext: false },
  ]) {
    const gate = new ExecutionKeyGate();
    assert.deepEqual(gate.press(input("Space"), { ...ready, ...blocked }), {
      handled: true,
    });
    assert.equal(gate.press(input("Space"), ready).command, undefined);
    gate.release("Space");
    assert.equal(gate.press(input("Space"), ready).command, "next");
  }
  for (const code of ["KeyP", "KeyS"]) {
    const gate = new ExecutionKeyGate();
    assert.equal(
      gate.press(input(code), { ...ready, status: "running", busy: true })
        .command,
      undefined,
    );
    assert.equal(
      gate.press(input(code), { ...ready, status: "running" }).command,
      undefined,
    );
  }
});

test("只处理四种明确按键，修饰键、输入法和普通文字保持原操作", () => {
  for (const extra of [
    { altKey: true },
    { ctrlKey: true },
    { metaKey: true },
    { shiftKey: true },
    { isComposing: true },
    { keyCode: 229 },
  ]) {
    for (const code of ["Space", "KeyP", "KeyS", "Escape"]) {
      const gate = new ExecutionKeyGate();
      assert.deepEqual(gate.press(input(code, extra), ready), {
        handled: false,
      });
      assert.equal(gate.press(input(code), ready).command, undefined);
      gate.release(code);
    }
  }
  const gate = new ExecutionKeyGate();
  for (const code of ["Enter", "Tab", "KeyA", "ArrowDown", ""]) {
    assert.deepEqual(gate.press(input(code), ready), { handled: false });
  }
});

test("暂停、继续、停止按实际播放状态决定，退出不受忙或旧版本限制", () => {
  for (const status of ["idle", "running", "paused", "finished"] as const) {
    const gate = new ExecutionKeyGate();
    const state = { ...ready, status };
    assert.equal(
      gate.press(input("KeyP"), state).command,
      status === "running"
        ? "pause"
        : status === "paused"
          ? "resume"
          : undefined,
    );
    assert.equal(
      gate.press(input("KeyS"), state).command,
      status === "idle" ? undefined : "stop",
    );
    gate.reset();
    assert.equal(
      gate.press(input("KeyP"), { ...state, ready: false }).command,
      undefined,
    );
    assert.equal(
      gate.press(input("KeyS"), { ...state, ready: false }).command,
      undefined,
    );
    assert.equal(
      gate.press(input("Escape"), { ...state, ready: false, busy: true })
        .command,
      "exit",
    );
  }
});
