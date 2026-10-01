import { createRoot } from "react-dom/client";
import { useState } from "react";
import { SceneSelectionBar } from "../src/components/workbench/SceneSelectionBar";
import { QuickPresets } from "../src/components/workbench/QuickPresets";
import type { ProjectView } from "../src/application-host";
import type { RecallMode } from "../src/library-tools";
import "../src/base.css";
import "../src/workbench.css";
const fixture = (id: string) => ({
  id,
  name: id,
  profileId: "mode",
  profileName: "灯型",
  domainId: "d",
  domainName: "灯光",
  universe: 1,
  address: 1,
  footprint: 1,
  attributes: [{ key: "dimmer", label: "亮度", defaultValue: 0 }],
});
const project: ProjectView = {
  id: "ux042-pins-test",
  name: "固定资源验收",
  description: "",
  audio: null,
  profiles: [],
  domains: [],
  sequences: [],
  stage: { spaces: [], constructions: [], placements: [], attachments: [] },
  fixtures: [fixture("a"), fixture("b"), fixture("c")],
  scenes: [{ id: "s", name: "测试场景", values: [], effects: [] }],
  groups: [
    { id: "g1", name: "前排", fixtureIds: ["b", "a"] },
    { id: "g2", name: "后排", fixtureIds: ["c", "b"] },
  ],
  presets: Array.from({ length: 9 }, (_, i) => ({
    id: `p${i + 1}`,
    name: `亮度 ${i + 1}`,
    values: [
      {
        fixtureId: "a",
        attribute: "dimmer",
        value: (i + 1) * 5000,
        mode: "value",
        presetName: null,
        presetId: null,
      },
    ],
    usedByScenes: [],
    usedBySequences: [],
  })),
};
function Harness() {
  const [selected, setSelected] = useState(["b", "a"]),
    [preset, setPreset] = useState("p1"),
    [recall, setRecall] = useState<RecallMode>("replace");
  const [busy, setBusy] = useState(false),
    [reject, setReject] = useState(false),
    [removed, setRemoved] = useState(false),
    [other, setOther] = useState(false),
    [renamed, setRenamed] = useState(false);
  const [mask, setMask] = useState<string[] | null>(null),
    [count, setCount] = useState(0),
    [report, setReport] = useState("");
  const data = {
    ...project,
    id: other ? "ux042-other-test" : project.id,
    groups: project.groups
      .filter((g) => !removed || g.id !== "g1")
      .map((g) => ({
        ...g,
        name: renamed && g.id === "g1" ? "前区摇头灯" : g.name,
      })),
  };
  return (
    <main className="workbench" style={{ padding: 20 }}>
      <button onClick={() => setBusy((v) => !v)}>切换忙</button>
      <button onClick={() => setReject((v) => !v)}>拒绝选灯</button>
      <button onClick={() => setRemoved((v) => !v)}>删除或恢复前排</button>
      <button onClick={() => setRenamed((v) => !v)}>改名</button>
      <button onClick={() => setOther((v) => !v)}>换工程</button>
      <p role="status">
        已选 {selected.join("→")} · 预设 {preset} · 应用 {count} · 拒绝{" "}
        {reject ? "是" : "否"}
      </p>
      <output>{report}</output>
      <div style={{ width: 800, maxWidth: "100%" }}>
        <SceneSelectionBar
          project={data}
          selected={selected}
          busy={busy}
          recall={recall}
          onRecall={setRecall}
          onSelect={async (ids) => {
            if (reject) return false;
            setSelected(ids);
            return true;
          }}
        />
        <QuickPresets
          project={data}
          scene={data.scenes[0]}
          selected={selected}
          busy={busy}
          presetId={preset}
          mask={mask}
          onPreset={setPreset}
          onMask={setMask}
          onEdit={async (command) => {
            setReport(JSON.stringify(command));
            setCount((n) => n + 1);
            return data;
          }}
        />
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
