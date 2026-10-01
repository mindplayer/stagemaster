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
const project: ProjectView = {
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
function Harness() {
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
        selected={["lamp"]}
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
        selected={["lamp"]}
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
