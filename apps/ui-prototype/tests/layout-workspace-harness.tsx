// Test surface only: no files, devices, UE process or physical output.
import { useContext, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { PerformanceLayout } from "../src/components/layout/PerformanceLayout";
import {
  DockPane,
  ViewportRevealContext,
} from "../src/components/layout/DockPane";
import { LiveVideoFixture } from "./live-video-fixture";
import "../src/base.css";
import "../src/workbench.css";
function Reveal() {
  const reveal = useContext(ViewportRevealContext);
  return <button onClick={reveal}>显式查看舞台</button>;
}

function Harness() {
  const [page, setPage] = useState("scenes");
  const [invalid, setInvalid] = useState(false);
  const [draft, setDraft] = useState("保持草稿");
  const [report, setReport] = useState("等待验收");
  const field = useRef<HTMLInputElement>(null);
  return (
    <main className="workbench">
      <output>{report}</output>
      <PerformanceLayout
        mode={page}
        beforeChange={async () => {
          if (invalid) {
            setReport("字段无效，保持属性可见");
            field.current?.focus();
            return false;
          }
          return true;
        }}
        toolbar={
          <nav aria-label="验收工作区">
            {[
              "scenes",
              "sequences",
              "audio",
              "execution",
              "stage",
              "settings",
            ].map((p) => (
              <button key={p} onClick={() => setPage(p)}>
                {p}
              </button>
            ))}
          </nav>
        }
      >
        <DockPane region="library" visible={page !== "settings"}>
          <input aria-label="资源筛选" defaultValue="" />
        </DockPane>
        <DockPane region="viewport" keepConnected visible={page !== "settings"}>
          <LiveVideoFixture />
          <p>中央舞台</p>
        </DockPane>
        <DockPane
          region="viewport"
          className="viewport-transport-pane"
          visible={page === "audio"}
        >
          <button>独立音乐控制验收</button>
        </DockPane>
        <DockPane
          region="editor"
          visible={["scenes", "sequences", "audio"].includes(page)}
        >
          <p>编排</p>
          <Reveal />
        </DockPane>
        <DockPane region="inspector" visible={page !== "settings"}>
          <input
            aria-label="属性草稿"
            ref={field}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
          />
          <label>
            <input
              type="checkbox"
              checked={invalid}
              onChange={(e) => setInvalid(e.target.checked)}
            />
            无效字段
          </label>
        </DockPane>
        <DockPane region="full" visible={page === "settings"}>
          <p>工程管理</p>
        </DockPane>
      </PerformanceLayout>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
