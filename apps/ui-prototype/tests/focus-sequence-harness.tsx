// Actual sequence components; preview reads an empty snapshot and edits are refused.
import { useState } from "react";
import { createRoot } from "react-dom/client";
import { PerformanceLayout } from "../src/components/layout/PerformanceLayout";
import { DockPane } from "../src/components/layout/DockPane";
import { SequenceWorkspace } from "../src/components/workbench/SequenceWorkspace";
import type { ApplicationHost } from "../src/application-host";
import { applicationHost } from "../src/hosts/application-host";
import { stageProject } from "./stage-organization-fixture";
import "../src/base.css";
import "../src/workbench.css";
const project = stageProject();
project.scenes = [
  { id: "scene", name: "暖金应答与对白转场", effects: [], values: [] },
];
project.sequences = [
  {
    id: "sequence",
    name: "Volare · 灯光独立检查（不带音乐）",
    tracking: "isolated",
    repeat: "once",
    steps: Array.from({ length: 24 }, (_, i) => ({
      id: `step-${i}`,
      number: String(i + 1),
      name: `长名称步骤 ${i + 1} · 对白转场`,
      sceneId: "scene",
      delayMs: 1500,
      fadeMs: 1205,
      waitMs: null,
    })),
  },
];
if (applicationHost.kind !== "browser")
  throw new Error("此入口仅用于浏览器布局验收");
const host: ApplicationHost = {
  ...applicationHost,
  preview: async () => ({ epoch: 0, controlSerial: 0, loaded: null }),
};
function Harness() {
  const [execution, setExecution] = useState(false);
  return (
    <main className="workbench" style={{ height: "100vh" }}>
      <PerformanceLayout
        mode={execution ? "execution" : "sequences"}
        toolbar={<span>真实步骤组件 · 布局验收</span>}
      >
        <DockPane region="viewport">
          <p>场地占位，仅测试布局</p>
        </DockPane>
        <SequenceWorkspace
          project={project}
          host={host}
          generation={0}
          busy={false}
          visible
          execution={execution}
          onExecution={setExecution}
          beforeChange={async () => true}
          onPending={() => {}}
          onEdit={async () => null}
        />
      </PerformanceLayout>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
