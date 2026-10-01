import test from "node:test";
import assert from "node:assert/strict";
import {
  layoutPlanLabels,
  boxesOverlap,
  type PlanLabelItem,
} from "../src/plan-label-layout.ts";
import { fitPlanLabel } from "../src/plan-label-text.ts";
const item = (id: string, x = 0, y = 0, priority = 1000): PlanLabelItem => ({
  id,
  x,
  y,
  priority,
  width: 5,
  selected: priority < 1000,
});
test("标签优先级与身份决定稳定排布，不随绘制层次和输入顺序改变", () => {
  const items = [item("z"), item("a"), item("selected", 0, 0, 1)];
  const before = structuredClone(items);
  const a = layoutPlanLabels(items),
    b = layoutPlanLabels([...items].reverse());
  assert.deepEqual(a, b);
  assert.deepEqual(items, before);
  assert.equal(a.placed[0].id, "selected");
  for (const [i, x] of a.placed.entries())
    for (const y of a.placed.slice(i + 1))
      assert.equal(boxesOverlap(x, y), false);
});
test("同坐标灯位允许部分省略，仍保留选中优先且不删除输入身份", () => {
  const items = Array.from({ length: 512 }, (_, n) =>
    item(String(n).padStart(4, "0")),
  );
  items[511].priority = 0;
  items[511].selected = true;
  const result = layoutPlanLabels(items);
  assert.equal(result.placed[0].id, "0511");
  assert.ok(result.hidden.length > 500);
  assert.equal(
    new Set([...result.placed.map((i) => i.id), ...result.hidden]).size,
    512,
  );
  assert.ok(result.placed.length <= 8);
});
test("标签不遮挡符号与选中序号，视窗外灯具不计作省略", () => {
  const items = [item("a", 0, 0, 1), item("b", 6, 2), item("c", 100, 100)];
  const v = { x: -8, y: -8, width: 20, height: 20 };
  const { placed, hidden } = layoutPlanLabels(items, v);
  assert.equal(placed.length, 2);
  assert.equal(hidden.length, 0);
  for (const p of placed) {
    assert.ok(
      p.x >= v.x &&
        p.y >= v.y &&
        p.x + p.width <= v.x + v.width &&
        p.y + p.height <= v.y + v.height,
    );
    for (const i of items)
      assert.equal(
        boxesOverlap(p, {
          x: i.x - 1.3,
          y: i.y - (i.selected ? 2.2 : 1.3),
          width: 2.6,
          height: i.selected ? 3.5 : 2.6,
        }),
        false,
      );
  }
  const tiny = layoutPlanLabels([item("a")], {
    x: -1,
    y: -1,
    width: 2,
    height: 2,
  });
  assert.deepEqual(tiny.hidden, ["a"]);
});
test("空输入和非法几何不会进入空间桶循环，平移不改变相对排布", () => {
  assert.deepEqual(layoutPlanLabels([]), { placed: [], hidden: [] });
  assert.deepEqual(layoutPlanLabels([{ ...item("bad"), x: NaN }]), {
    placed: [],
    hidden: [],
  });
  assert.deepEqual(
    layoutPlanLabels([{ ...item("huge"), x: Number.MAX_VALUE }]),
    { placed: [], hidden: [] },
  );
  const items = [item("a"), item("b", 12, 8)];
  const a = layoutPlanLabels(items),
    b = layoutPlanLabels(
      items.map((i) => ({ ...i, x: i.x + 100, y: i.y - 30 })),
    );
  assert.deepEqual(
    b.placed,
    a.placed.map((i) => ({ ...i, x: i.x + 100, y: i.y - 30 })),
  );
});
test("标签截短保留组合字符和表情字素，完整短标签不被改写", () => {
  const measure = (s: string) =>
    [...new Intl.Segmenter(undefined, { granularity: "grapheme" }).segment(s)]
      .length;
  assert.deepEqual(fitPlanLabel("1.007", measure), { text: "1.007", width: 5 });
  assert.deepEqual(fitPlanLabel("👨‍👩‍👧‍👦é摇头灯很长", measure, 4), {
    text: "👨‍👩‍👧‍👦é摇…",
    width: 4,
  });
});
