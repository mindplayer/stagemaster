import test from "node:test";
import assert from "node:assert/strict";
import { stageProject } from "./stage-organization-fixture.ts";
import { stageLabelItems } from "../src/stage-label-items.ts";
import { layoutPlanLabels, boxesOverlap } from "../src/plan-label-layout.ts";
import { translated } from "../src/stage-tools.ts";
import { seatingLayout } from "../src/seating-tools.ts";
import type { SeatingShape } from "../src/stage-types.ts";
const identity = <T>(value: T) => value;
const noSelection = () => false;
test("三种标注模式遵守过滤后的场地，不隐藏几何或更改工程", () => {
  const p = stageProject(),
    before = structuredClone(p);
  const all = stageLabelItems(
    p.fixtures,
    p.stage,
    "name",
    1,
    identity,
    noSelection,
    [],
  );
  assert.equal(all.length, 30);
  assert.equal(all.filter((i) => i.symbol).length, 3);
  assert.equal(new Set(all.map((i) => i.id)).size, all.length);
  assert.equal(
    all.find((i) => i.id === "construction:piece-0")!.text,
    "座椅构件 1",
  );
  const addresses = stageLabelItems(
    p.fixtures,
    p.stage,
    "address",
    1,
    identity,
    noSelection,
    [],
  );
  assert.equal(addresses.length, 3);
  assert.ok(addresses.every((i) => i.kind === "placement"));
  assert.deepEqual(
    stageLabelItems(p.fixtures, p.stage, "none", 1, identity, noSelection, []),
    [],
  );
  const filtered = { ...p.stage, constructions: [], spaces: [] };
  assert.equal(
    stageLabelItems(p.fixtures, filtered, "name", 1, identity, noSelection, [])
      .length,
    3,
  );
  assert.deepEqual(p, before);
});
test("拖动草稿和平面单位统一投影，不把旧位置用于标注", () => {
  const p = stageProject();
  const items = (unit: number, moved: boolean) =>
    stageLabelItems(
      p.fixtures,
      p.stage,
      "name",
      unit,
      (o) => (moved ? translated(o, 3, -4) : o),
      noSelection,
      [],
    );
  const old = items(0.5, false),
    moved = items(0.5, true);
  moved.forEach((i, n) => {
    assert.equal(i.x, old[n].x + 6);
    assert.equal(i.y, old[n].y + 8);
  });
  assert.deepEqual(items(0, true), []);
  assert.deepEqual(items(NaN, false), []);
});
test("旋转座区锚点使用世界包围盒，保留座数与选中优先", () => {
  const p = stageProject();
  const shape: SeatingShape = {
    kind: "seating",
    spaceId: null,
    positionMeters: { x: "2", y: "3", z: "0" },
    yawDegrees: "70",
    rows: 3,
    columns: 6,
    seatWidthMeters: "0.45",
    seatDepthMeters: "0.45",
    columnSpacingMeters: "0.55",
    rowSpacingMeters: "0.8",
    aisle: null,
  };
  p.stage.constructions = [{ id: "seats", name: "斜向观众区", shape }];
  const entries = stageLabelItems(
    p.fixtures,
    p.stage,
    "name",
    0.2,
    identity,
    (kind, id) => kind === "construction" && id === "seats",
    [],
  );
  const item = entries.find((i) => i.objectId === "seats")!;
  const outline = seatingLayout(shape)!.outline;
  assert.ok(
    Math.abs(item.y - Math.max(...outline.map((p) => -p[1])) / 0.2) < 1e-5,
  );
  assert.equal(item.text, "斜向观众区 · 18 座");
  assert.equal(item.symbol, false);
  assert.equal(item.priority, 0);
  assert.equal(item.selected, true);
  const invalid = stageLabelItems(
    p.fixtures,
    p.stage,
    "name",
    0.2,
    (o) =>
      o.kind === "construction" && o.value.shape.kind === "seating"
        ? { ...o, value: { ...o.value, shape: { ...o.value.shape, rows: 0 } } }
        : o,
    (kind, id) => kind === "construction" && id === "seats",
    [],
  );
  assert.deepEqual(
    invalid.find((i) => i.objectId === "seats"),
    item,
  );
});
test("混合名称共同避让灯具符号与序号，选中构件优先且不会占用虚拟符号区域", () => {
  const items = [
    { id: "fixture", x: 0, y: 0, width: 5, selected: true, priority: 1 },
    {
      id: "seating",
      x: 0,
      y: 0,
      width: 8,
      selected: true,
      priority: 0,
      symbol: false,
    },
    {
      id: "space",
      x: 0,
      y: 0,
      width: 8,
      selected: false,
      priority: 300000,
      symbol: false,
    },
  ];
  const result = layoutPlanLabels(items);
  assert.equal(result.placed[0].id, "seating");
  for (const [n, a] of result.placed.entries()) {
    for (const b of result.placed.slice(n + 1))
      assert.equal(boxesOverlap(a, b), false);
    assert.equal(
      boxesOverlap(a, { x: -1.3, y: -2.2, width: 2.6, height: 3.5 }),
      false,
    );
  }
  const anchor = { ...items[1], x: 0, y: 2, priority: 10 };
  const uncluttered = layoutPlanLabels([items[0], anchor]);
  assert.equal(
    uncluttered.placed[0].y,
    1.7,
    "non-symbol anchor must not reserve a false fixture obstacle",
  );
  assert.deepEqual(result, layoutPlanLabels([...items].reverse()));
});
