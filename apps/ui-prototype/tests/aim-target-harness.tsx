// Exercises real position fields/gestures; captured commands do not represent native solving.
import { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  PositionPanel,
  type PositionHandle,
} from "../src/components/fixtures/PositionPanel";
import { stageProject } from "./stage-organization-fixture";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/stage/stage.css";
import "../src/components/fixtures/fixtures.css";
const project = stageProject();
project.fixtures = project.fixtures.map((f, i) => ({
  ...f,
  name: `摇头灯 ${i + 1}`,
  positioning: {
    kind: "intersectingOrthogonal",
    pan: { minDegrees: "-270", maxDegrees: "270", reversed: false },
    tilt: { minDegrees: "-135", maxDegrees: "135", reversed: false },
  },
  attributes: [
    { key: "pan", label: "水平", defaultValue: 32768 },
    { key: "tilt", label: "垂直", defaultValue: 32768 },
  ],
}));
const scene = { id: "scene", name: "位置验收", values: [], effects: [] };
function Harness() {
  const editor = useRef<PositionHandle>(null);
  const [pending, setPending] = useState(false),
    [busy, setBusy] = useState(false),
    [reject, setReject] = useState(false),
    [missing, setMissing] = useState(false),
    [mounted, setMounted] = useState(true);
  const [calls, setCalls] = useState(0),
    [command, setCommand] = useState(""),
    [error, setError] = useState("");
  function apply() {
    try {
      const operations = editor.current?.collect() ?? [];
      if (!operations.length) return true;
      setCalls((n) => n + 1);
      setCommand(JSON.stringify(operations));
      if (reject) throw new Error("验收拒绝：目标不在机械行程内");
      editor.current?.accept();
      setError("");
      return true;
    } catch (e) {
      setError(String(e));
      return false;
    }
  }
  return (
    <main
      className="workbench"
      style={{ padding: 20, display: "flex", alignItems: "start", gap: 24 }}
    >
      <aside style={{ width: 360 }}>
        <button onClick={() => setBusy(!busy)}>切换忙状态</button>
        <button onClick={() => setMissing(!missing)}>切换未布置</button>
        <button onClick={() => setMounted(!mounted)}>切换面板</button>
        <label>
          <input
            type="checkbox"
            checked={reject}
            onChange={(e) => setReject(e.target.checked)}
          />
          拒绝求解
        </label>
        <output>
          提交 {calls} · 草稿 {String(pending)} · {command}
        </output>
        <p role="alert">{error}</p>
      </aside>
      <div style={{ width: 360 }}>
        {mounted && (
          <PositionPanel
            ref={editor}
            project={
              missing
                ? { ...project, stage: { ...project.stage, placements: [] } }
                : project
            }
            fixtures={project.fixtures.slice(0, 2)}
            scene={scene}
            busy={busy}
            onPending={setPending}
            onApply={apply}
            beforeChange={async () => apply()}
          />
        )}
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
