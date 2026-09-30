// Isolated UI fixture; edits stay in memory and never reach output hardware.
import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import { stageProject } from "./stage-organization-fixture";
import { StageCanvas } from "../src/components/stage/StageCanvas";
import { StageOutliner } from "../src/components/stage/StageOutliner";
import { StagePlanLayers } from "../src/components/stage/StagePlanLayers";
import {
  ALL_VISIBLE,
  revealStageTarget,
} from "../src/components/stage/stage-display";
import type {
  StageObject,
  StageSelection,
  FixturePlacement,
} from "../src/stage-types";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/stage/stage.css";
function Harness() {
  const [project, setProject] = useState(stageProject);
  const [visibility, setVisibility] = useState(ALL_VISIBLE);
  const [selection, setSelection] = useState<StageSelection | null>(null);
  const [ids, setIds] = useState<string[]>([]);
  const [query, setQuery] = useState("");
  const [edits, setEdits] = useState(0);
  const [moving, setMoving] = useState(false);
  function select(target: StageSelection, additive = false) {
    setVisibility((v) => revealStageTarget(project.stage, v, target));
    setSelection(target);
    setIds(
      target.kind === "placement"
        ? additive
          ? [...new Set([...ids, target.id])]
          : [target.id]
        : [],
    );
  }
  function move(placements: FixturePlacement[]) {
    setEdits((n) => n + 1);
    setProject((p) => ({
      ...p,
      stage: {
        ...p.stage,
        placements: p.stage.placements.map(
          (old) =>
            placements.find((item) => item.fixtureId === old.fixtureId) ?? old,
        ),
      },
    }));
  }
  function moveObject(object: StageObject) {
    if (object.kind === "placement") move([object.value]);
  }
  return (
    <main
      className="workbench"
      style={{
        height: "100vh",
        display: "grid",
        gridTemplateColumns: "260px 1fr",
        gridTemplateRows: "1fr 42px",
      }}
    >
      <aside className="stage-browser">
        <input
          aria-label="搜索场地对象"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <StagePlanLayers
          value={visibility}
          disabled={false}
          onChange={setVisibility}
        />
        <StageOutliner
          project={project}
          selection={selection}
          selectedIds={ids}
          query={query}
          busy={false}
          visibility={visibility}
          onVisibility={setVisibility}
          onSelect={select}
        />
      </aside>
      <StageCanvas
        project={project}
        visibility={visibility}
        selection={selection}
        selectedIds={ids}
        preview={null}
        focusRequest={0}
        busy={false}
        pending={false}
        onSelect={select}
        onSelectPlacements={setIds}
        onMovePlacements={move}
        onMove={moveObject}
        onArrange={() => {}}
        onGesture={setMoving}
      />
      <output style={{ gridColumn: "1 / -1" }} aria-label="验收状态">
        修改 {edits} · 拖动 {moving ? "进行中" : "未开始"} ·{" "}
        {project.stage.placements
          .map(
            (p) =>
              `${p.fixtureId}: ${p.positionMeters.x},${p.positionMeters.y}`,
          )
          .join(" / ")}
      </output>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
