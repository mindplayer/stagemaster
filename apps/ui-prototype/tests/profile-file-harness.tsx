// Explicit UI-only outcomes; native tests cover core semantics and the filesystem.
import { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  ProfileWorkspace,
  type ProfileHandle,
} from "../src/components/fixtures/ProfileWorkspace";
import { applicationHost } from "../src/hosts/application-host";
import type { EditOperation, ProjectView } from "../src/application-host";
import type {
  ImportedProfile,
  ExportedProfile,
} from "../src/profile-file-types";
import "../src/base.css";
import "../src/workbench.css";
const definition = {
  name: "测试模式",
  manufacturer: "测试",
  model: "旧款",
  mode: "调光",
  footprint: 1,
  channels: [{ attribute: "dimmer", coarse: 1, fine: null, defaultValue: 0 }],
};
const initial: ProjectView = {
  id: "test",
  name: "工程",
  description: "",
  audio: null,
  domains: [],
  profiles: [{ ...definition, id: "p", revision: "r", authorable: true }],
  fixtures: [],
  groups: [],
  presets: [],
  scenes: [],
  sequences: [],
  stage: { spaces: [], constructions: [], placements: [], attachments: [] },
};
function Harness() {
  const [project, setProject] = useState(initial),
    [generation, setGeneration] = useState(1),
    [visible, setVisible] = useState(true);
  const [error, setError] = useState(""),
    [pending, setPending] = useState(false),
    [calls, setCalls] = useState(0),
    [commits, setCommits] = useState(0);
  const [completion, setCompletion] = useState<((mode: string) => void) | null>(
    null,
  );
  const editor = useRef<ProfileHandle>(null),
    current = useRef({ project, generation });
  current.current = { project, generation };
  function apply(ops: EditOperation[]) {
    if (!ops.length) return;
    const next = {
      ...current.current.project,
      profiles: [...current.current.project.profiles],
    };
    for (const op of ops)
      if (op.op === "fixture" && op.command.op === "saveProfile") {
        next.profiles.push({
          ...op.command.definition,
          id: `new-${next.profiles.length}`,
          revision: "new",
          authorable: true,
        });
      }
    current.current = {
      project: next,
      generation: current.current.generation + 1,
    };
    setProject(next);
    setGeneration(current.current.generation);
    setCommits((n) => n + 1);
    editor.current?.accept();
    setError("");
  }
  async function flush() {
    try {
      apply(editor.current?.collect() ?? []);
      return true;
    } catch (e) {
      setError(String(e));
      return false;
    }
  }
  const host = {
    ...applicationHost,
    importProfile: async (version: number) => {
      setCalls((n) => n + 1);
      return new Promise<ImportedProfile | null>((resolve, reject) =>
        setCompletion(() => (mode: string) => {
          setCompletion(null);
          if (mode === "error") reject(new Error("文件版本不兼容"));
          else
            resolve(
              mode === "cancel"
                ? null
                : {
                    generation: version,
                    fileName: "模式.smfixture.json",
                    source: { profileId: "source", revision: "revision" },
                    definition,
                  },
            );
        }),
      );
    },
    exportProfile: async (version: number, id: string) => {
      setCalls((n) => n + 1);
      return new Promise<ExportedProfile>((resolve, reject) =>
        setCompletion(() => (mode: string) => {
          setCompletion(null);
          if (mode === "error") reject(new Error("目标文件已变化"));
          else
            resolve({
              generation: version,
              profileId: id,
              revision: "r",
              path: mode === "cancel" ? null : "/验收/模式.smfixture.json",
              warning: null,
            });
        }),
      );
    },
  };
  return (
    <div
      className="workbench"
      style={{ height: "100vh", display: "flex", flexDirection: "column" }}
    >
      <div>
        <button onClick={() => setGeneration((n) => n + 1)}>改变版本</button>
        <button onClick={() => setVisible((v) => !v)}>切换页面</button>
        <button onClick={() => void flush()}>外部收集草稿</button>
        <button onClick={() => completion?.("success")}>返回成功</button>
        <button onClick={() => completion?.("cancel")}>返回取消</button>
        <button onClick={() => completion?.("error")}>返回错误</button>
        <span>
          接口 {calls} · 提交 {commits} · 草稿 {String(pending)} · 模式{" "}
          {project.profiles.length}
        </span>
      </div>
      <ProfileWorkspace
        ref={editor}
        {...{ project, generation, visible, host, error }}
        busy={false}
        onPending={setPending}
        capture={async () =>
          (await flush()) ? current.current.generation : null
        }
        beforeChange={flush}
        onEdit={async (command) => {
          apply([command as EditOperation]);
          return current.current.project;
        }}
        onBack={() => void flush()}
      />
    </div>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
