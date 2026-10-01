// Export dialog outcomes and stale results only; no filesystem or native service.
import { useState } from "react";
import { createRoot } from "react-dom/client";
import { SequenceReportExport } from "../src/components/projects/SequenceReportExport";
import { applicationHost } from "../src/hosts/application-host";
import type { ProjectView } from "../src/application-host";
import type { SequenceReportExport as Receipt } from "../src/report-types";
import "../src/base.css";
import "../src/workbench.css";
const project: ProjectView = {
  id: "report",
  name: "报表验收",
  description: "",
  audio: null,
  profiles: [],
  domains: [],
  groups: [],
  presets: [],
  sequences: ["开场剧本", "返场剧本"].map((name, i) => ({
    id: `s${i}`,
    name,
    tracking: "inherited",
    repeat: "once",
    steps: [],
  })),
  scenes: [],
  fixtures: [
    {
      id: "a",
      name: "未配适灯",
      profileId: "p",
      profileName: "模式",
      domainId: "d",
      domainName: "域",
      footprint: 4,
      universe: null,
      address: null,
      attributes: [],
    },
  ],
  stage: { spaces: [], placements: [], attachments: [], constructions: [] },
};
function Harness() {
  const [generation, setGeneration] = useState(1),
    [calls, setCalls] = useState(0);
  const [invalid, setInvalid] = useState(false),
    [visible, setVisible] = useState(true);
  const [empty, setEmpty] = useState(false),
    [epoch, setEpoch] = useState(0);
  const [completion, setCompletion] = useState<((mode: string) => void) | null>(
    null,
  );
  return (
    <main className="workbench" style={{ padding: 24 }}>
      <button onClick={() => setEmpty((v) => !v)}>切换空列表</button>
      <button onClick={() => setEpoch((n) => n + 1)}>切换工程</button>
      <button onClick={() => setGeneration((n) => n + 1)}>改变工程版本</button>
      <button onClick={() => setInvalid((v) => !v)}>切换无效草稿</button>
      <button onClick={() => setVisible((v) => !v)}>切换页面</button>
      <button onClick={() => completion?.("success")}>返回成功</button>
      <button onClick={() => completion?.("cancel")}>返回取消</button>
      <button onClick={() => completion?.("error")}>返回错误</button>
      <p>
        接口调用 {calls} 次 · 版本 {generation} · 无效草稿 {String(invalid)}
      </p>
      <SequenceReportExport
        key={epoch}
        project={empty ? { ...project, sequences: [] } : project}
        generation={generation}
        hasDrafts={invalid}
        visible={visible}
        busy={false}
        capture={async () => (invalid ? null : generation)}
        host={{
          ...applicationHost,
          exportSequenceReport: async (version, sequenceId) => {
            setCalls((n) => n + 1);
            return new Promise<Receipt>((resolve, reject) =>
              setCompletion(() => (mode: string) => {
                setCompletion(null);
                if (mode === "error") reject(new Error("测试：磁盘文件已变化"));
                else
                  resolve({
                    generation: version,
                    path: mode === "cancel" ? null : "/验收/节目单.csv",
                    sequenceId,
                    sequenceName: project.sequences.find(
                      (s) => s.id === sequenceId,
                    )!.name,
                    stepCount: 6,
                    warning: null,
                  });
              }),
            );
          },
        }}
      />
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
