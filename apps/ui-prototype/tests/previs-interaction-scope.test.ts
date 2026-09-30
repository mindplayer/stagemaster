import test from "node:test";
import assert from "node:assert/strict";
import { PrevisInteractionScope } from "../src/previs-interaction-scope.ts";

test("离开舞台后，延迟进入移动和位置提案失效", () => {
  const scope = new PrevisInteractionScope();
  scope.update("stage", true);
  const active = scope.capture();
  assert.equal(active(), true);
  scope.update("audio", false);
  assert.equal(active(), false);
  assert.equal(scope.capture()(), false);
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
