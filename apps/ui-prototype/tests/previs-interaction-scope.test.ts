import test from "node:test";
import assert from "node:assert/strict";
import { PrevisInteractionScope } from "../src/previs-interaction-scope.ts";
import { sameFixtureSelection } from "../src/previs-selection.ts";
import { previsInteractions } from "../src/components/workbench/previs-interactions.ts";

test("离开舞台后，延迟进入移动和位置提案失效", () => {
  const scope = new PrevisInteractionScope();
  scope.update("stage", true);
  const active = scope.capture();
  assert.equal(active(), true);
  scope.update("audio", false);
  assert.equal(active(), false);
  assert.equal(scope.capture()(), false);
});

test("三维选组变化与返回使旧提案失效，普通重渲染保留手势", () => {
  const scope = new PrevisInteractionScope();
  scope.update(`stage:${JSON.stringify(["a", "b"])}`, true);
  const pending = scope.capture();
  scope.update(`stage:${JSON.stringify(["a", "b"])}`, true);
  assert.equal(pending(), true);
  scope.update(`stage:${JSON.stringify(["b", "a"])}`, true);
  assert.equal(pending(), false);
  scope.update(`stage:${JSON.stringify(["a", "b"])}`, true);
  assert.equal(pending(), false);
  assert.equal(sameFixtureSelection(["a", "b"], ["b", "a"]), false);
});
test("桌面队列等待期间取消或换组选灯，不提交迟到的位移", async () => {
  const requests: unknown[] = [];
  let queued: (() => Promise<void>) | undefined;
  let alive = true;
  const interactions = previsInteractions({
    page: "stage",
    stage: { current: null },
    selectedIds: ["a", "b"],
    project: () => null,
    run: async (work) => {
      queued = work;
      return true;
    },
    request: async (command) => {
      requests.push(command);
    },
    select: () => {},
    notice: () => {},
  });
  const proposal = {
    generation: 7,
    version: "9",
    fixtureIds: ["a", "b"],
    deltaMeters: { x: "1", y: "0", z: "0" },
  };
  await interactions.onTranslation(proposal, () => alive);
  alive = false;
  await assert.rejects(queued!, /上下文已变化/);
  assert.deepEqual(requests, []);
  alive = true;
  await interactions.onTranslation(
    { ...proposal, fixtureIds: ["a"] },
    () => alive,
  );
  await assert.rejects(queued!, /上下文已变化/);
  assert.deepEqual(requests, []);
  await interactions.onTranslation(proposal, () => alive);
  await queued!();
  assert.deepEqual(requests, [{ kind: "previsTranslation", ...proposal }]);
});

test("过时三维选择不污染新页面，查看页面不能写灯位", async () => {
  const selected: string[][] = [];
  let queued: (() => Promise<void>) | undefined;
  const common = {
    stage: { current: null },
    selectedIds: ["a"],
    project: () => null,
    run: async (work: () => Promise<void>) => {
      queued = work;
      return true;
    },
    request: async () => {
      throw new Error("不应写入");
    },
    select: (ids: string[]) => selected.push(ids),
    notice: () => {},
  };
  const view = previsInteractions({ ...common, page: "scenes" });
  await view.onSelect([], () => false);
  await assert.rejects(queued!, /上下文已变化/);
  assert.deepEqual(selected, []);
  await view.onSelect([], () => true);
  await queued!();
  assert.deepEqual(selected, [[]]);
  await view.onTranslation(
    {
      generation: 1,
      version: "1",
      fixtureIds: ["a"],
      deltaMeters: { x: "1", y: "0", z: "0" },
    },
    () => true,
  );
  await assert.rejects(queued!, /上下文已变化/);
  const other = previsInteractions({ ...common, page: "audio" });
  assert.equal(await other.onSelect(["a"], () => true), false);
});
test("返回舞台不会重新允许之前的提案", () => {
  const scope = new PrevisInteractionScope();
  scope.update("stage", true);
  const old = scope.capture();
  scope.update("audio", false);
  scope.update("stage", true);
  assert.equal(old(), false);
  assert.equal(scope.capture()(), true);
});
test("普通重渲染保留当前操作，权限撤回立即生效", () => {
  const scope = new PrevisInteractionScope();
  scope.update("stage", true);
  const active = scope.capture();
  scope.update("stage", true);
  assert.equal(active(), true);
  scope.update("stage", false);
  assert.equal(active(), false);
});

test("整组旋转与缩放沿用队列上下文保护，失效不写工程", async () => {
  const requests: unknown[] = [];
  let queued: (() => Promise<void>) | undefined;
  const actions = previsInteractions({
    page: "stage",
    stage: { current: null },
    selectedIds: ["a", "b"],
    project: () => null,
    run: async (work) => {
      queued = work;
      return true;
    },
    request: async (command) => {
      requests.push(command);
    },
    select: () => {},
    notice: () => {},
  });
  const proposal = {
    generation: 1,
    version: "9",
    fixtureIds: ["a", "b"],
    yawDegrees: "90",
    spacingScale: "2",
  };
  await actions.onTransform(proposal, () => false);
  await assert.rejects(queued!, /上下文已变化/);
  assert.deepEqual(requests, []);
  await actions.onTransform(
    { ...proposal, fixtureIds: ["b", "a"] },
    () => true,
  );
  await assert.rejects(queued!, /上下文已变化/);
  assert.deepEqual(requests, []);
  await actions.onTransform(proposal, () => true);
  await queued!();
  assert.deepEqual(requests, [{ kind: "previsTransform", ...proposal }]);
});

test("取消手势作废已排队提案，后续操作仍可使用同一选择", () => {
  const scope = new PrevisInteractionScope();
  scope.update("stage:a,b", true);
  const previous = scope.capture();
  scope.invalidate();
  assert.equal(previous(), false);
  assert.equal(scope.capture()(), true);
});
