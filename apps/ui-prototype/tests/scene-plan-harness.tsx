// Isolated selection-only fixture: no engine, audio, renderer or device calls.
import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import { stageProject } from "./stage-organization-fixture";
import { FixturePlan } from "../src/components/scene-plan/FixturePlan";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/stage/stage.css";
const project = stageProject();
project.fixtures.push(
  { ...project.fixtures[0], id: "floor", name: "地排灯" },
  { ...project.fixtures[0], id: "unplaced", name: "待布置灯" },
);
project.stage.placements.push({
  ...project.stage.placements[0],
  fixtureId: "floor",
  positionMeters: { x: "0", y: "2", z: "0.5" },
});
function denseProject() {
  const dense = stageProject();
  dense.fixtures = Array.from({ length: 80 }, (_, n) => ({
    ...dense.fixtures[0],
    id: `dense-${n}`,
    name: `摇头灯 ${n + 1} · 👨‍👩‍👧‍👦é舞台长名称检查`,
    universe: 1,
    address: n * 4 + 1,
  }));
  dense.stage.placements = dense.fixtures.map((f, n) => ({
    ...dense.stage.placements[0],
    fixtureId: f.id,
    positionMeters: {
      x: String(n < 12 ? 0 : ((n % 16) - 8) * 0.35),
      y: String(n < 12 ? 2 : Math.floor(n / 16) * 0.5),
      z: "5",
    },
  }));
  return dense;
}
function Harness() {
  const [current, setCurrent] = useState(project);
  const [only, setOnly] = useState(false);
  const [selected, setSelected] = useState<string[]>([]),
    [query, setQuery] = useState(""),
    [blocked, setBlocked] = useState(false),
    [count, setCount] = useState(0);
  return (
    <main
      className="workbench"
      style={{ height: "100vh", display: "flex", flexDirection: "column" }}
    >
      <div>
        <button
          onClick={() => {
            setCurrent(denseProject());
            setSelected([]);
            setQuery("");
            setOnly(false);
          }}
        >
          密集灯位验收
        </button>
        <button onClick={() => setSelected(["loose", "front"])}>
          从灯组选两台
        </button>
        <label>
          <input
            type="checkbox"
            checked={blocked}
            onChange={(e) => setBlocked(e.target.checked)}
          />
          模拟未通过草稿校验
        </label>
      </div>
      <FixturePlan
        project={current}
        visible
        selected={selected}
        query={query}
        onlySelected={only}
        onFilter={setOnly}
        busy={false}
        onQuery={setQuery}
        onSelect={async (ids) => {
          if (blocked) return false;
          setSelected(ids);
          setCount((n) => n + 1);
          return true;
        }}
        viewControls={<strong>平面选灯</strong>}
      />
      <output aria-label="选择状态">
        选择 {selected.join(",") || "空"} · 提交 {count}
      </output>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
