import { createRoot } from "react-dom/client";
import { useState } from "react";
import { SequenceStepBrowser } from "../src/components/workbench/SequenceStepBrowser";
import type { SequenceView } from "../src/sequence-types";
import type { ExecutionPosition } from "../src/components/workbench/execution-position";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/workbench/execution-view.css";
const sequence: SequenceView = {
  id: "seq",
  name: "长剧本",
  tracking: "isolated",
  repeat: "once",
  steps: Array.from({ length: 60 }, (_, i) => ({
    id: `row-${i + 1}`,
    number: `${i + 1}`,
    name: `步骤 ${i + 1}`,
    sceneId: "s",
    delayMs: 0,
    fadeMs: 1000,
    waitMs: null,
  })),
};
function Harness() {
  const [query, setQuery] = useState(""),
    [selected, setSelected] = useState("row-1");
  const [visible, setVisible] = useState(true),
    [busy, setBusy] = useState(false);
  const [position, setPosition] = useState<ExecutionPosition>({
    epoch: 1,
    sequenceId: "seq",
    currentId: "row-30",
    nextId: "row-31",
    status: "running",
    stale: false,
  });
  return (
    <main className="workbench" style={{ padding: 20 }}>
      <button
        onClick={() =>
          setPosition((p) => {
            const n = Number(p.currentId?.split("-")[1] ?? 0) + 1;
            return { ...p, currentId: `row-${n}`, nextId: `row-${n + 1}` };
          })
        }
      >
        模拟推进
      </button>
      <button
        onClick={() => setPosition((p) => ({ ...p, epoch: p.epoch + 1 }))}
      >
        重新载入
      </button>
      <button onClick={() => setPosition((p) => ({ ...p, stale: !p.stale }))}>
        切换过期
      </button>
      <button
        onClick={() =>
          setPosition((p) => ({
            ...p,
            sequenceId: p.sequenceId === "seq" ? "other" : "seq",
          }))
        }
      >
        切换播放列表
      </button>
      <button onClick={() => setVisible((v) => !v)}>切换可见</button>
      <button onClick={() => setBusy((v) => !v)}>切换忙</button>
      <button
        onClick={() =>
          setPosition((p) => ({
            ...p,
            currentId: null,
            nextId: "row-1",
            status: "idle",
          }))
        }
      >
        停止到待执行
      </button>
      <p role="status">
        选中 {selected} · 当前 {position.currentId} · 版本 {position.epoch}
      </p>
      <div
        style={{ height: 550, overflow: "auto", border: "1px solid gray" }}
        aria-label="外层验收容器"
      >
        <div style={{ height: 80 }}>外层滚动隔离</div>
        <div
          className="navigation-harness"
          style={{
            width: 680,
            maxWidth: "100%",
            display: visible ? "flex" : "none",
            flexDirection: "column",
            height: 360,
          }}
        >
          <SequenceStepBrowser
            sequence={sequence}
            steps={sequence.steps.filter((s) => s.name.includes(query))}
            selectedId={selected}
            scenes={[]}
            busy={busy}
            visible={visible}
            execution={true}
            position={position}
            query={query}
            setQuery={setQuery}
            onSelect={async (id) => {
              setSelected(id);
              return true;
            }}
          />
        </div>
        <div style={{ height: 400 }}>外层结尾</div>
      </div>
      <style>{`.navigation-harness .wb-steps-region{overflow:auto;min-height:0;flex:1}.navigation-harness .wb-steps>button{min-height:64px;flex-shrink:0}`}</style>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
