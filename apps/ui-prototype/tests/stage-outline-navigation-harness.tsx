// Actual library; only selection is local. No engineering writes or output.
import { useState } from "react";
import { createRoot } from "react-dom/client";
import { StageLibraryPanel } from "../src/components/stage/StageLibraryPanel";
import { stageProject } from "./stage-organization-fixture";
import { ALL_VISIBLE } from "../src/components/stage/stage-display";
import type { StageSelection } from "../src/stage-types";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/stage/stage.css";
const project = stageProject();
project.fixtures = Array.from({ length: 80 }, (_, i) => ({
  ...project.fixtures[0],
  id: `f${i}`,
  name: `摇头灯 ${String(i + 1).padStart(2, "0")}`,
}));
project.stage.placements = project.fixtures.map((f) => ({
  ...project.stage.placements[0],
  fixtureId: f.id,
}));
project.stage.attachments = [];
function Harness() {
  const [selection, setSelection] = useState<StageSelection | null>({
    kind: "placement",
    id: "f0",
  });
  const [ids, setIds] = useState(["f0"]);
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState(false);
  const [reject, setReject] = useState(false);
  const [calls, setCalls] = useState(0);
  const [visibility, setVisibility] = useState(ALL_VISIBLE);
  function choose(target: StageSelection, additive = false) {
    setCalls((n) => n + 1);
    if (reject) return;
    setSelection(target);
    setIds(
      target.kind === "placement"
        ? additive
          ? [...new Set([...ids, target.id])]
          : [target.id]
        : [],
    );
  }
  return (
    <main
      className="workbench"
      style={{
        height: "100vh",
        padding: 12,
        display: "flex",
        flexDirection: "row",
        gap: 12,
      }}
    >
      <div
        aria-label="外层滚动验收"
        style={{ height: 700, overflow: "auto", width: 310 }}
      >
        <div style={{ height: 660, display: "flex" }}>
          <StageLibraryPanel
            project={project}
            selection={selection}
            selectedIds={ids}
            busy={busy}
            query={query}
            onQuery={setQuery}
            visibility={visibility}
            onVisibility={setVisibility}
            onSelect={choose}
            onCreate={() => {}}
            onCreateRig={() => {}}
            onCreateSeating={() => {}}
            onArrange={() => {}}
            onPlace={() => {}}
            fixtureId=""
            onFixtureId={() => {}}
          />
        </div>
        <div style={{ height: 400 }}>外层尾部</div>
      </div>
      <section>
        <button onClick={() => choose({ kind: "placement", id: "f79" })}>
          画布选中末灯
        </button>
        <label>
          <input
            type="checkbox"
            checked={busy}
            onChange={(e) => setBusy(e.target.checked)}
          />
          宿主忙
        </label>
        <label>
          <input
            type="checkbox"
            checked={reject}
            onChange={(e) => setReject(e.target.checked)}
          />
          拒绝新选择
        </label>
        <pre aria-label="验收状态">
          {JSON.stringify(
            { selection, ids, calls, query, visibility },
            null,
            2,
          )}
        </pre>
      </section>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
