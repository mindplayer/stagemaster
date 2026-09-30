// Component-only acceptance. No file reads, writes or device operations.
import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import { applicationHost } from "../src/hosts/application-host";
import type { ApplicationHost } from "../src/application-host";
import type { RecentProject } from "../src/recent-types";
import { ProjectStart } from "../src/components/projects/ProjectStart";
import { RecentProjectsDialog } from "../src/components/projects/RecentProjectsDialog";
import "../src/base.css";
import "../src/workbench.css";

let entries: RecentProject[] = [
  {
    id: "1",
    name: "蓝金之夜",
    path: "/演出/剧场甲/蓝金之夜.project.json",
    openedAtMs: 1790800000000,
    available: true,
  },
  {
    id: "2",
    name: "蓝金之夜",
    path: "/演出/剧场乙/蓝金之夜.project.json",
    openedAtMs: 1790710000000,
    available: true,
  },
  {
    id: "3",
    name: "失效的工程",
    path: "/已移走/测试.project.json",
    openedAtMs: 1790600000000,
    available: false,
  },
];
const host: ApplicationHost = {
  ...applicationHost,
  kind: "desktop",
  recent: async (request) => {
    if (request.kind === "forget")
      entries = entries.filter((e) => e.id !== request.id);
    return [...entries];
  },
};
function Harness() {
  const [dialog, setDialog] = useState(false);
  const [status, setStatus] = useState("");
  const browse = () => setStatus("已请求文件选择器");
  const open = async (id: string) => {
    setStatus(`已请求工程 ${id}`);
    return true;
  };
  return (
    <main className="workbench">
      <button onClick={() => setDialog(true)}>查看最近目录</button>
      <output aria-label="验收状态">{status}</output>
      <ProjectStart
        host={host}
        busy={false}
        onNew={() => {}}
        onOpen={browse}
        onRecover={() => {}}
        onRecent={open}
      />
      {dialog && (
        <RecentProjectsDialog
          host={host}
          busy={false}
          error=""
          onOpen={open}
          onBrowse={browse}
          onClose={() => setDialog(false)}
        />
      )}
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
