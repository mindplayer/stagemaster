import { createRoot } from "react-dom/client";
import { useRef, useState } from "react";
import {
  SequenceGroupProperties,
  type GroupPropertiesHandle,
} from "../src/components/workbench/SequenceGroupProperties";
import type { StepView } from "../src/sequence-types";
import "../src/base.css";
import "../src/workbench.css";
const steps: StepView[] = [
  {
    id: "a",
    number: "1",
    name: "开场",
    sceneId: "s",
    delayMs: 0,
    fadeMs: 1000,
    waitMs: null,
    script: { section: "第一幕", trigger: "第一句", notes: "独立备注一" },
  },
  {
    id: "b",
    number: "2",
    name: "转场",
    sceneId: "s",
    delayMs: 500,
    fadeMs: 2000,
    waitMs: null,
    script: { section: "第二幕", trigger: "第二句", notes: "独立备注二" },
  },
];
function Harness() {
  const editor = useRef<GroupPropertiesHandle>(null);
  const [pending, setPending] = useState(false),
    [busy, setBusy] = useState(false),
    [reject, setReject] = useState(false),
    [one, setOne] = useState(false);
  const [commits, setCommits] = useState(0),
    [report, setReport] = useState(""),
    [error, setError] = useState("");
  async function apply() {
    try {
      const commands = editor.current?.collect() ?? [];
      if (reject) throw new Error("验收拒绝，保留草稿");
      if (commands.length) {
        setCommits((n) => n + 1);
        setReport(JSON.stringify(commands));
      }
      editor.current?.accept();
      setError("");
      return true;
    } catch (reason) {
      setError(String(reason));
      return false;
    }
  }
  return (
    <main className="workbench" style={{ padding: 24 }}>
      <button onClick={() => setBusy((v) => !v)}>切换忙状态</button>
      <label>
        <input
          type="checkbox"
          checked={reject}
          onChange={(e) => setReject(e.target.checked)}
        />
        拒绝提交
      </label>
      <button
        disabled={busy}
        onClick={async () => {
          if (await apply()) setOne((v) => !v);
        }}
      >
        改变选择
      </button>
      <p role="status">
        待修改 {pending ? "有" : "无"} · 提交 {commits} · 已选 {one ? 1 : 2}
      </p>
      <output>{report}</output>
      <p>{error}</p>
      <div style={{ width: 300, maxWidth: "100%" }}>
        <SequenceGroupProperties
          ref={editor}
          sequenceId="seq"
          steps={one ? steps.slice(0, 1) : steps}
          busy={busy}
          onPending={setPending}
          beforeChange={apply}
        />
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
