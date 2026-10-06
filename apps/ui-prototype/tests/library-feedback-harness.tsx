// Real preset component and production callback; controlled replies, no host/device/player.
import { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { QuickPresets } from "../src/components/workbench/QuickPresets";
import { libraryEditActions } from "../src/library-edit-actions";
import type { ProjectView, Snapshot } from "../src/application-host";
import "../src/base.css";
import "../src/workbench.css";

const project: ProjectView = {
  id: "feedback-fixture",
  name: "隔离资源反馈",
  description: "",
  audio: null,
  profiles: [],
  domains: [],
  groups: [],
  sequences: [],
  fixtures: [
    {
      id: "lamp",
      name: "测试灯具",
      profileId: "generic",
      profileName: "测试亮度",
      domainId: "domain",
      domainName: "舞台",
      footprint: 1,
      universe: 1,
      address: 1,
      attributes: [{ key: "dimmer", label: "亮度", defaultValue: 0 }],
    },
  ],
  scenes: [{ id: "scene", name: "测试场景", effects: [], values: [] }],
  presets: [
    {
      id: "preset",
      name: "测试预设",
      usedByScenes: [],
      usedBySequences: [],
      values: [
        {
          fixtureId: "lamp",
          attribute: "dimmer",
          mode: "value",
          value: 32768,
          presetName: null,
          presetId: null,
        },
      ],
    },
  ],
  stage: { spaces: [], constructions: [], placements: [], attachments: [] },
};
const initial: Snapshot = {
  project,
  generation: 42,
  fileName: "fixture.json",
  dirty: true,
  canUndo: true,
  canRedo: true,
  recovery: { state: "clean", capturedAtMs: null, problem: null },
};
function Harness() {
  const [snapshot, setSnapshot] = useState(initial);
  const current = useRef(snapshot);
  const [mode, setMode] = useState<"unchanged" | "changed" | "rejected">(
    "unchanged",
  );
  const [busy, setBusy] = useState(false),
    [notice, setNotice] = useState("");
  const [error, setError] = useState(""),
    [calls, setCalls] = useState(0);
  const [updates, setUpdates] = useState(0),
    [preset, setPreset] = useState("preset");
  const [mask, setMask] = useState<string[] | null>(null);
  const action = libraryEditActions(
    async (work) => {
      setBusy(true);
      setError("");
      try {
        await work();
        return true;
      } catch (reason) {
        setError(String(reason));
        return false;
      } finally {
        setBusy(false);
      }
    },
    () => current.current,
    async () => {
      setCalls((value) => value + 1);
      if (mode === "rejected") throw Error("宿主拒绝资源操作");
      if (mode === "changed") {
        current.current = {
          ...current.current,
          generation: current.current.generation + 1,
          canRedo: false,
        };
        setSnapshot(current.current);
      }
    },
    (value) => {
      setUpdates((count) => count + 1);
      setNotice(value);
    },
  );
  return (
    <main
      className="workbench"
      style={{ display: "block", padding: 24, height: "100vh" }}
    >
      <h1>隔离资源反馈验收</h1>
      <p>正式快捷预设组件；宿主回复为可控夹具，不是原生资格。</p>
      <div>
        <button onClick={() => setMode("unchanged")}>宿主无变更</button>
        <button onClick={() => setMode("changed")}>宿主有变更</button>
        <button onClick={() => setMode("rejected")}>宿主拒绝</button>
      </div>
      <output aria-label="请求计数">
        请求 {calls} / 成功反馈 {updates}
      </output>
      <output aria-label="历史状态">
        代次 {snapshot.generation} / 原有撤销 {String(snapshot.canUndo)} /
        原有重做 {String(snapshot.canRedo)}
      </output>
      <p role={error ? "alert" : "status"}>{error || notice}</p>
      <QuickPresets
        project={snapshot.project!}
        scene={snapshot.project!.scenes[0]}
        selected={["lamp"]}
        busy={busy}
        presetId={preset}
        mask={mask}
        onPreset={setPreset}
        onMask={setMask}
        onEdit={action}
      />
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
