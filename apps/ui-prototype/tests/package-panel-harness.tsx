// Isolated dev-only component fixture; excluded from the application build and all product routes.
import React from "react";
import { createRoot } from "react-dom/client";
import { PackagePanel } from "../src/components/workbench/PackagePanel";
import type { ApplicationHost, ProjectView } from "../src/application-host";
import type { PackageResult } from "../src/package-types";
import "../src/workbench.css";
let finish: ((result: PackageResult) => void) | null = null;
const unavailable = async () => {
  throw new Error("本组件测试不调用该接口");
};
const host: ApplicationHost = {
  execution: async () => { throw new Error("此测试不提供后台执行"); },
  importEffectTemplate: unavailable,
  exportEffectTemplate: unavailable,
  cancelEffectTemplate: unavailable,
  exportPatchReport: unavailable,
  exportSequenceReport: unavailable,
  importProfile: unavailable,
  exportProfile: unavailable,
  recent: async () => [],
  audio: async () => {throw new Error("此组件不调用音频");},
  audioPrepare: async () => {throw new Error("此组件不调用音频");},
  audioCancel: async () => {},
  kind: "desktop",
  output: async () => { throw new Error("此验收未使用预演总控"); },
  device: unavailable,
  installation: unavailable,
  startInstallation: unavailable,
  buildPackage: async () =>
    new Promise((resolve) => {
      finish = resolve;
    }),
  exportPackage: unavailable,
  recovery: unavailable,
  check: unavailable,
  request: unavailable,
  preview: unavailable,
  previs: unavailable,
  onCloseRequested: async () => () => {},
};
const project: ProjectView = {
  audio: null,
  id: "test",
  name: "组件竞态测试",
  description: "",
  profiles: [],
  domains: [],
  fixtures: [],
  scenes: [{ id: "a", name: "测试场景", effects: [], values: [] }],
  groups: [],
  presets: [],
  sequences: [],
  stage: { spaces: [], constructions: [], placements: [], attachments: [] },
};
createRoot(document.getElementById("root")!).render(
  <div className="workbench" style={{ height: "auto", padding: 20 }}>
    <h1>播放包组件竞态测试</h1>
    <button
      onClick={() =>
        finish?.({
          generation: 1,
          token: null,
          report: null,
          issues: [{ message: "后台完成结果", location: null }],
        })
      }
    >
      完成后台请求
    </button>
    <PackagePanel
      host={host}
      project={project}
      generation={1}
      hasDrafts={false}
      visible={true}
      busy={false}
      capture={async () => 1}
      onLocate={async () => true}
      onInstall={async () => {}}
      installReason="隔离组件测试不连接设备"
    />
  </div>,
);
