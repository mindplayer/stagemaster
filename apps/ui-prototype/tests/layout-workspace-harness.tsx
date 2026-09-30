// Test surface only: no files, devices, UE process or physical output.
import { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { PerformanceLayout } from "../src/components/layout/PerformanceLayout";
import { DockPane } from "../src/components/layout/DockPane";
import { LiveVideoFixture } from "./live-video-fixture";
import "../src/base.css";
import "../src/workbench.css";

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
            {["scenes", "stage", "settings"].map((p) => (
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
        <DockPane region="editor" visible={page === "scenes"}>
          <p>编排</p>
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
