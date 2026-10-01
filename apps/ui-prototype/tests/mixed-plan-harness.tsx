// Isolated real components. No engine, audio, device or file access.
import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import { mixedPlanProject } from "./mixed-plan-fixture";
import { StageCanvas } from "../src/components/stage/StageCanvas";
import { StageOverview } from "../src/components/stage/StageOverview";
import { FixturePlan } from "../src/components/scene-plan/FixturePlan";
import { ALL_VISIBLE } from "../src/components/stage/stage-display";
import type { StageSelection } from "../src/stage-types";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/stage/stage.css";
function Harness() {
  const [project, setProject] = useState(mixedPlanProject);
  const [view, setView] = useState("stage"),
    [selection, setSelection] = useState<StageSelection | null>(null);
  const [ids, setIds] = useState<string[]>([]),
    [query, setQuery] = useState("");
  const [only, setOnly] = useState(false),
    [hide, setHide] = useState(false),
    [edits, setEdits] = useState(0);
  return (
    <main
      className="workbench"
      style={{ height: "100vh", display: "flex", flexDirection: "column" }}
    >
      <div>
        <select
          aria-label="验收视图"
          value={view}
          onChange={(e) => setView(e.target.value)}
        >
          <option value="stage">布置</option>
          <option value="scene">选灯</option>
          <option value="overview">只读查看</option>
        </select>
        <button
          onClick={() => {
            setSelection({ kind: "construction", id: "seats-3" });
            setIds([]);
          }}
        >
          选择观众座区 4
        </button>
        <button
          onClick={() => {
            setSelection(null);
            setIds(["f-23"]);
          }}
        >
          选择光束 24
        </button>
        <label>
          <input
            type="checkbox"
            checked={hide}
            onChange={(e) => setHide(e.target.checked)}
          />
          隐藏座区
        </label>
      </div>
      {view === "stage" ? (
        <StageCanvas
          project={project}
          visibility={{ ...ALL_VISIBLE, hiddenLayers: hide ? ["seating"] : [] }}
          selection={selection}
          selectedIds={ids}
          preview={null}
          focusRequest={0}
          busy={false}
          pending={false}
          onSelect={(target) => {
            setSelection(target);
            setIds(target.kind === "placement" ? [target.id] : []);
          }}
          onSelectPlacements={setIds}
          onMovePlacements={() => setEdits((n) => n + 1)}
          onArrange={() => {}}
          onGesture={() => {}}
          onMove={(object) => {
            setEdits((n) => n + 1);
            if (object.kind === "construction")
              setProject((p) => ({
                ...p,
                stage: {
                  ...p.stage,
                  constructions: p.stage.constructions.map((c) =>
                    c.id === object.value.id ? object.value : c,
                  ),
                },
              }));
          }}
        />
      ) : view === "scene" ? (
        <FixturePlan
          project={project}
          visible
          selected={ids}
          query={query}
          onlySelected={only}
          busy={false}
          onQuery={setQuery}
          onFilter={setOnly}
          onSelect={async (ids) => {
            setIds(ids);
            return true;
          }}
          viewControls={<strong>场景</strong>}
        />
      ) : (
        <StageOverview
          project={project}
          visible
          viewControls={<strong>编排</strong>}
        />
      )}
      <output aria-label="验收状态">
        修改 {edits} · 对象 {selection?.id ?? "无"} · 灯具{" "}
        {ids.join(",") || "无"}
      </output>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
