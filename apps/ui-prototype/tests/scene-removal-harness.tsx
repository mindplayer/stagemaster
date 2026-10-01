import { createRoot } from "react-dom/client";
import { useRef, useState } from "react";
import { SceneRemovalDialog } from "../src/components/workbench/SceneRemovalDialog";
import { useSceneRemoval } from "../src/components/workbench/useSceneRemoval";
import { usageProject } from "./scene-usage-fixture";
import "../src/base.css";
import "../src/workbench.css";
function Harness() {
  const [project, setProject] = useState(() => {
    const p = usageProject();
    p.scenes.push(
      ...Array.from({ length: 25 }, (_, i) => ({
        id: `free-${i}`,
        name: `副本 ${i + 1}`,
        values: [],
        effects: [],
      })),
    );
    return p;
  });
  const current = useRef(project);
  current.current = project;
  const [busy, setBusy] = useState(false),
    [reject, setReject] = useState(false),
    [failure, setFailure] = useState(false),
    [error, setError] = useState("");
  const [edits, setEdits] = useState(0),
    [located, setLocated] = useState("");
  const removal = useSceneRemoval({
    read: () => current.current,
    run: async (work, flush = true) => {
      if (flush && reject) {
        setError("草稿拒绝");
        return false;
      }
      setBusy(true);
      setError("");
      try {
        await work();
        return true;
      } catch (e) {
        setError(String(e));
        return false;
      } finally {
        setBusy(false);
      }
    },
    edit: async (command) => {
      if (failure) throw new Error("宿主拒绝");
      if (command.op !== "batch") throw new Error("expected batch");
      setEdits((n) => n + 1);
      const ids = command.commands.map((c) => ("id" in c ? c.id : ""));
      const next = {
        ...current.current,
        scenes: current.current.scenes.filter((s) => !ids.includes(s.id)),
      };
      current.current = next;
      setProject(next);
    },
    onRemoved: () => {},
  });
  return (
    <main className="workbench" style={{ display: "block", padding: 20 }}>
      <label>
        <input
          type="checkbox"
          checked={reject}
          onChange={(e) => setReject(e.target.checked)}
        />
        拒绝草稿
      </label>
      <label>
        <input
          type="checkbox"
          checked={failure}
          onChange={(e) => setFailure(e.target.checked)}
        />
        模拟宿主失败
      </label>
      <button onClick={() => removal.request(["s", "free-0"])}>
        检查混合组
      </button>
      <button
        onClick={() =>
          removal.request(
            project.scenes
              .filter((s) => s.id.startsWith("free-"))
              .map((s) => s.id),
          )
        }
      >
        检查所有副本
      </button>
      <p role="status">
        场景 {project.scenes.length} · 事务 {edits} · {located} · {error}
      </p>
      {removal.intent && (
        <SceneRemovalDialog
          project={project}
          ids={removal.intent.ids}
          busy={busy}
          error={[removal.problem, error].filter(Boolean).join("\n")}
          onCancel={removal.close}
          onRemove={() => void removal.remove()}
          onLocate={(id, target) => {
            setLocated(`${id}:${target.kind}:${target.id}`);
            removal.dismiss();
          }}
        />
      )}
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
