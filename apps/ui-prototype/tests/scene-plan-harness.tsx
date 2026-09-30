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
function Harness() {
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
        project={project}
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
