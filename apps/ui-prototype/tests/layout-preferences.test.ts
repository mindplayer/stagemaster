import test from "node:test";
import assert from "node:assert/strict";
import {
  defaultLayout,
  readLayout,
  panelSize,
} from "../src/components/layout/layout-preferences.ts";

test("损坏、过期或非对象布局回到可操作的默认值", () => {
  for (const value of [null, "{", "null", "false", '"old"', "[]", "{}"])
    assert.deepEqual(readLayout(value), defaultLayout);
});
test("持久化布局限制尺寸并拒绝字符串和非有限数字", () => {
  assert.deepEqual(
    readLayout(
      '{"library":-50,"inspector":999999,"editor":"320","showLibrary":false,"showInspector":"false"}',
    ),
    { ...defaultLayout, library: 180, inspector: 420, showLibrary: false },
  );
  assert.equal(panelSize("editor", Infinity), defaultLayout.editor);
  assert.equal(panelSize("editor", NaN), defaultLayout.editor);
  assert.equal(panelSize("editor", -1), 180);
  assert.equal(panelSize("editor", 800), 560);
});
test("布局可往返且不共享默认对象", () => {
  const value = {
    ...defaultLayout,
    library: 276,
    editor: 260,
    showInspector: false,
  };
  assert.deepEqual(readLayout(JSON.stringify(value)), value);
  const fresh = readLayout(null);
  fresh.library = 300;
  fresh.focusedTasks.push("scenes");
  assert.equal(defaultLayout.library, 240);
  assert.deepEqual(defaultLayout.focusedTasks, []);
});

test("旧布局保持尺寸，新查看偏好有界去重且只接受真实编排任务", () => {
  const old = readLayout('{"library":276,"editor":440}');
  assert.deepEqual(old.focusedTasks, []);
  assert.equal(old.editor, 440);
  const current = readLayout(
    '{"focusedTasks":["audio","settings","audio",null,"scenes",17]}',
  );
  assert.deepEqual(current.focusedTasks, ["scenes", "audio"]);
  assert.deepEqual(readLayout(JSON.stringify(current)), current);
  for (const value of ["true", '"audio"', "{}", "null"]) {
    assert.deepEqual(
      readLayout('{"focusedTasks":' + value + "}").focusedTasks,
      [],
    );
  }
});
