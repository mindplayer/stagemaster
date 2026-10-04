import test from "node:test";
import "./execution-batch.test.ts";
import assert from "node:assert/strict";
import {
  allSources,
  boardRows,
  sourcePinKey,
  sourceStatus,
} from "../src/execution-board.ts";
import {
  editExecutionPins,
  readExecutionPins,
  saveExecutionPins,
} from "../src/execution-pins.ts";
import { boardFixture } from "./execution-board-fixture.ts";

test("64 项按类型、名称、状态和未应用输入交集浏览，不改变目录和优先级", () => {
  const runtime = boardFixture();
  const before = structuredClone(runtime);
  const rows = boardRows(
    runtime,
    { ...allSources, query: "蓝色 场景列表", kind: "sequence" },
    [],
    new Set(),
  );
  assert.equal(rows.filter((r) => r.shown).length, 8);
  assert.equal(
    boardRows(
      runtime,
      { ...allSources, status: "Running" },
      [],
      new Set(),
    ).filter((r) => r.shown).length,
    2,
  );
  assert.equal(
    boardRows(
      runtime,
      { ...allSources, status: "Paused" },
      [],
      new Set(),
    ).filter((r) => r.shown).length,
    2,
  );
  assert.deepEqual(
    boardRows(
      runtime,
      { ...allSources, scope: "drafts" },
      [],
      new Set(["source-5", "manual"]),
    )
      .filter((r) => r.shown)
      .map((r) => r.source.id),
    ["source-5", "manual"],
  );
  assert.deepEqual(runtime, before);
});
test("常用排序只改变显示，结束、手动和未知分别表示", () => {
  const r = boardFixture();
  const keys = [
    sourcePinKey(r.catalog.sources[9]),
    sourcePinKey(r.catalog.sources[0]),
  ];
  const rows = boardRows(
    r,
    { ...allSources, scope: "pinned" },
    keys,
    new Set(),
  );
  assert.deepEqual(
    rows.filter((x) => x.shown).map((x) => x.source.id),
    ["source-9", "source-0"],
  );
  assert.equal(sourceStatus(r.catalog.sources[63], r), null);
  assert.equal(sourceStatus(r.catalog.sources[3], r), "Finished");
  r.observation.snapshot = null;
  assert.equal(sourceStatus(r.catalog.sources[0], r), "Unknown");
  assert.equal(sourceStatus(r.catalog.sources[62], r), "Unknown");
});
test("固定按原对象身份跨后台恢复，未载入项不被隐式删除，移动越过缺席项", () => {
  const keys = ["scene:a", "scene:b", "sequence:c"];
  let projects = keys.reduce(
    (p, key) => editExecutionPins(p, "p", keys, { kind: "toggle", key }),
    [] as ReturnType<typeof readExecutionPins>,
  );
  projects = editExecutionPins(projects, "p", [keys[0], keys[2]], {
    kind: "earlier",
    key: keys[2],
  });
  assert.deepEqual(projects[0].sources, [keys[2], keys[1], keys[0]]);
  projects = editExecutionPins(projects, "p", [keys[0], keys[2]], {
    kind: "prune",
  });
  assert.deepEqual(projects[0].sources, [keys[2], keys[0]]);
  projects = editExecutionPins(projects, "other", [keys[0]], {
    kind: "toggle",
    key: keys[0],
  });
  assert.deepEqual(projects[1].sources, [keys[2], keys[0]]);
});
test("常用容量、损坏和超大存储有界，写失败不丢失本次偏好", () => {
  const keys = Array.from({ length: 17 }, (_, i) => `scene:${i}`);
  const full = [{ projectId: "p", sources: keys.slice(0, 16) }];
  assert.throws(
    () => editExecutionPins(full, "p", keys, { kind: "toggle", key: keys[16] }),
    /最多固定/,
  );
  assert.equal(
    editExecutionPins(full, "p", keys, { kind: "toggle", key: keys[0] })[0]
      .sources.length,
    15,
  );
  assert.deepEqual(readExecutionPins("{"), []);
  assert.deepEqual(readExecutionPins(" ".repeat(65537)), []);
  const payload = {
    version: 1,
    projects: Array.from({ length: 25 }, (_, i) => ({
      projectId: `p${i}`,
      sources: keys,
    })),
  };
  const read = readExecutionPins(JSON.stringify(payload));
  assert.equal(read.length, 20);
  assert.equal(read[0].sources.length, 16);
  assert.deepEqual(
    readExecutionPins(JSON.stringify({ version: 2, projects: full })),
    [],
  );
  assert.match(
    saveExecutionPins(full, () => {
      throw Error("full");
    }),
    /本次使用/,
  );
  assert.equal(full[0].sources.length, 16);
});
