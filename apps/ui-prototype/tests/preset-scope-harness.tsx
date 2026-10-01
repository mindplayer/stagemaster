import { withFullScopeFixtures } from "./preset-scopes-project";
import { useState } from "react";
import { createRoot } from "react-dom/client";
import { ResourcePool } from "../src/components/workbench/ResourcePool";
import { QuickPresets } from "../src/components/workbench/QuickPresets";
import type { ProjectView } from "../src/application-host";
import { attributeName } from "../src/library-tools";
import "../src/base.css";
import "../src/workbench.css";

const keys = ["dimmer", "zoom", "focus", "iris"];
const values = keys.map((attribute) => ({
  fixtureId: "lamp",
  attribute,
  value: 32768,
  mode: "value" as const,
  presetId: null,
  presetName: null,
}));
const baseProject: ProjectView = {
  id: "p",
  name: "范围验收",
  description: "",
  audio: null,
  profiles: [],
  domains: [],
  groups: [],
  sequences: [],
  stage: { spaces: [], constructions: [], placements: [], attachments: [] },
  fixtures: [
    {
      id: "lamp",
      name: "镜头灯",
      profileId: "mode",
      profileName: "模式",
      domainId: "d",
      domainName: "灯光",
      universe: 1,
      address: 1,
      footprint: 4,
      attributes: keys.map((key) => ({
        key,
        label: attributeName(key),
        defaultValue: 0,
      })),
    },
  ],
  scenes: [{ id: "s", name: "场景", effects: [], values }],
  presets: [
    {
      id: "preset",
      name: "已有变焦",
      values: values.filter((v) => v.attribute === "zoom"),
      usedByScenes: [],
      usedBySequences: [],
    },
  ],
};
const full = new URLSearchParams(location.search).has("full");
const scopedProject = full ? withFullScopeFixtures(baseProject) : baseProject;
const project = new URLSearchParams(location.search).has("copy")
  ? {
      ...scopedProject,
      fixtures: [
        ...scopedProject.fixtures,
        ...Array.from({ length: 61 }, (_, i) => ({
          ...scopedProject.fixtures[0],
          id: `receiver-${i + 1}`,
          name: `接收灯 ${String(i + 1).padStart(2, "0")}`,
          universe: 2 + Math.floor(i / 50),
          address: (i % 50) * 10 + 1,
        })),
      ],
    }
  : scopedProject;
function Harness() {
  const [selection, setSelection] = useState("all");
  const selected = project.fixtures
    .filter((f) => selection === "all" || f.id === selection)
    .map((f) => f.id);
  const [mask, setMask] = useState<string[] | null>(null);
  const [presetId, setPresetId] = useState("preset");
  const [reject, setReject] = useState(false);
  const [report, setReport] = useState("");
  const [error, setError] = useState("");
  const [commits, setCommits] = useState(0);
  const onEdit = async (command: unknown) => {
    setReport(JSON.stringify(command));
    if (reject) {
      setError("验收拒绝：请保留当前草稿");
      return null;
    }
    setCommits((n) => n + 1);
    setError("");
    return project;
  };
  return (
    <main className="workbench" style={{ padding: 24 }}>
      <nav>
        {full && (
          <select
            aria-label="验收灯具选择"
            value={selection}
            onChange={(e) => setSelection(e.target.value)}
          >
            <option value="all">全部灯具</option>
            <option value="wheel">仅色盘灯</option>
            <option value="rgb">仅染色灯</option>
            <option value="empty">无灯具</option>
          </select>
        )}
        <label>
          <input
            type="checkbox"
            checked={reject}
            onChange={(e) => setReject(e.target.checked)}
          />
          拒绝提交
        </label>
        <button onClick={() => setMask(null)}>未限定范围</button>
      </nav>
      <output role="status">
        提交 {commits} · 范围 {mask === null ? "未限定" : mask.join(",")} ·{" "}
        {report}
      </output>
      <QuickPresets
        project={project}
        scene={project.scenes[0]}
        selected={selected}
        busy={false}
        presetId={presetId}
        mask={mask}
        onMask={setMask}
        onPreset={setPresetId}
        onEdit={onEdit}
      />
      <ResourcePool
        project={project}
        scene={project.scenes[0]}
        selected={selected}
        busy={false}
        error={error}
        visible
        beforeChange={async () => {
          setError("");
          return true;
        }}
        onEdit={onEdit}
        onSelect={async () => true}
        recall="replace"
        onRecall={() => {}}
        presetId={presetId}
        onPreset={setPresetId}
        mask={mask}
        onMask={setMask}
      />
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
