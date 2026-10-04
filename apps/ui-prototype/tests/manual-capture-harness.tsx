import { createRoot } from "react-dom/client";
import { useState } from "react";
import { ManualSceneRecorder } from "../src/components/execution/ManualSceneRecorder";
import { applicationHost } from "../src/hosts/application-host";
import { manualFixture } from "./execution-manual-fixture";
import { stageProject } from "./stage-organization-fixture";
import type {
  ManualCapture,
  ManualCapturePort,
} from "../src/manual-capture-types";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/execution/execution.css";
import "../src/components/execution/manual-readings.css";
const runtime = manualFixture();
runtime.controlling = false;
const source = runtime.catalog.sources.find((s) => s.id === "manual")!;
const held = runtime.observation.snapshot!.state.sources.find(
  (s) => s.id === "manual",
)!;
held.level = 0;
held.held = runtime.catalog.fixtures!.map((f, i) => ({
  fixtureId: f.id,
  attribute: i === 1 ? "color-wheel" : "dimmer",
}));
held.heldValues = held.held.map((_, i) => (i === 1 ? 37008 : i * 1000));
let ticket: ManualCapture | null = null;
let delay = false;
let serial = 0;
const messages: string[] = [];
const port: ManualCapturePort = async (request) => {
  if (request.kind === "cancel") {
    messages.push(`取消 ${request.token}`);
    if (ticket?.token === request.token) ticket = null;
    return null;
  }
  const readings = held
    .held!.map((target, i) => ({ ...target, value: held.heldValues![i] }))
    .filter((r) => !request.selected || request.selected.includes(r.fixtureId));
  const result: ManualCapture = {
    generation: request.generation,
    token: String(++serial),
    sourceName: "手动层",
    revision: "4",
    readings,
    merge: request.sceneId
      ? {
          sceneId: request.sceneId,
          sceneName: "已有呼吸场景",
          added: 0,
          replaced: readings.length,
          unchanged: 0,
          preserved: 8,
          effects: 1,
          rows: readings.map((r, i) => ({
            fixtureId: r.fixtureId,
            attribute: r.attribute,
            change: "replaced",
            previousMode: i === 0 ? "preset" : "literal",
            previousValue: i === 0 ? 65535 : 0,
            previousPreset: i === 0 ? "共享亮度" : null,
            effectNames: i === 0 ? ["呼吸"] : [],
          })),
        }
      : null,
    fixtures: runtime.catalog.fixtures!.filter((f) =>
      readings.some((r) => r.fixtureId === f.id),
    ),
  };
  ticket = structuredClone(result);
  if (delay) await new Promise((resolve) => setTimeout(resolve, 1500));
  return result;
};
function Harness() {
  const [project, setProject] = useState(() => ({
    ...stageProject(),
    id: runtime.catalog.projectId,
    scenes: [
      { id: "merge-scene", name: "已有呼吸场景", values: [], effects: [] },
    ],
  }));
  const [generation, setGeneration] = useState(1);
  const [active, setActive] = useState(true);
  const [pending, setPending] = useState(false);
  const [reject, setReject] = useState(false);
  const [narrow, setNarrow] = useState(false);
  const [readable, setReadable] = useState(true);
  const [revision, setRevision] = useState(0);
  const [recorded, setRecorded] = useState<ManualCapture | null>(null);
  return (
    <main
      className="workbench"
      style={{
        display: "block",
        padding: 20,
        overflow: "auto",
        height: "100vh",
      }}
    >
      <h1>手动录入组件验收</h1>
      <div className="execution-buttons">
        <label>
          <input
            type="checkbox"
            checked={active}
            onChange={(e) => setActive(e.target.checked)}
          />
          显示面板
        </label>
        <label>
          <input
            type="checkbox"
            checked={pending}
            onChange={(e) => setPending(e.target.checked)}
          />
          未应用输入
        </label>
        <label>
          <input
            type="checkbox"
            checked={reject}
            onChange={(e) => setReject(e.target.checked)}
          />
          提交拒绝
        </label>
        <label>
          <input
            type="checkbox"
            onChange={(e) => {
              delay = e.target.checked;
            }}
          />
          延迟采集
        </label>
        <label>
          <input
            type="checkbox"
            checked={narrow}
            onChange={(e) => setNarrow(e.target.checked)}
          />
          窄栏
        </label>
        <label>
          <input
            type="checkbox"
            checked={!readable}
            onChange={(e) => setReadable(!e.target.checked)}
          />
          观察失联
        </label>
        <button
          onClick={() => {
            setGeneration(generation + 1);
          }}
        >
          改变工程代次
        </button>
        <button
          onClick={() => {
            held.heldValues![0] = 65535;
            setRevision(revision + 1);
          }}
        >
          外部改变现场值
        </button>
      </div>
      <div
        hidden={!active}
        style={{ width: narrow ? 300 : 850, maxWidth: "100%" }}
      >
        <ManualSceneRecorder
          context={{
            host: { ...applicationHost, manualCapture: port },
            project,
            generation,
            busy: false,
            beforeCapture: async () => generation,
            onMerge: async (g, t) => {
              if (reject) throw Error("测试：工程容量不足，记录保留");
              if (g !== generation || ticket?.token !== t || !ticket.merge)
                throw Error("测试：记录过期");
              setRecorded(structuredClone(ticket));
              ticket = null;
              setGeneration(generation + 1);
            },
            onRecord: async (g, t, name) => {
              if (reject) throw Error("测试：工程容量不足，记录保留");
              if (g !== generation || ticket?.token !== t)
                throw Error("测试：记录过期");
              setRecorded(structuredClone(ticket));
              setProject({
                ...project,
                scenes: [
                  ...project.scenes,
                  { id: "new-scene", name, effects: [], values: [] },
                ],
              });
              ticket = null;
              setGeneration(generation + 1);
            },
          }}
          runtime={runtime}
          source={source}
          active={active}
          observed={readable}
          pending={pending}
          selected={[
            runtime.catalog.fixtures![0].id,
            runtime.catalog.fixtures![1].id,
          ]}
        />
      </div>
      <pre id="recorded">{JSON.stringify(recorded)}</pre>
      <pre>{JSON.stringify(messages)}</pre>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
