// UI-only navigation acceptance. No application host, playback or device access.
import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import { StageOverview } from "../src/components/stage/StageOverview";
import { stageProject } from "./stage-organization-fixture";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/stage/stage.css";
function Harness() {
  const [mode, setMode] = useState("placed"),
    [visible, setVisible] = useState(true);
  const project = stageProject();
  project.id = mode;
  if (mode !== "placed")
    project.stage = {
      spaces: [],
      constructions: [],
      placements: [],
      attachments: [],
    };
  if (mode === "empty") project.fixtures = [];
  return (
    <main
      className="workbench"
      style={{ height: "100vh", display: "flex", flexDirection: "column" }}
    >
      <div>
        <select
          aria-label="验收工程"
          value={mode}
          onChange={(e) => setMode(e.target.value)}
        >
          <option value="placed">分散场地与灯位</option>
          <option value="unplaced">尚未布置</option>
          <option value="empty">空工程</option>
        </select>
        <button onClick={() => setVisible((v) => !v)}>切换查看可见性</button>
      </div>
      <div
        style={{ display: visible ? "flex" : "none", flex: 1, minHeight: 0 }}
      >
        <StageOverview
          key={project.id}
          project={project}
          visible={visible}
          viewControls={<strong>平面查看隔离验收</strong>}
        />
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
