// Isolated transaction host. Real workspace/components; no device, playback or native bridge.
import { createRoot } from "react-dom/client";
import { useRef, useState } from "react";
import {
  StageWorkspace,
  type StageHandle,
} from "../src/components/stage/StageWorkspace";
import { PerformanceLayout } from "../src/components/layout/PerformanceLayout";
import { stageProject } from "./stage-organization-fixture";
import type {
  EditCommand,
  EditOperation,
  ProjectView,
} from "../src/application-host";
import "../src/base.css";
import "../src/workbench.css";

function Harness() {
  const [project, setProject] = useState(() => {
    const p = stageProject();
    p.fixtures.forEach((f, i) => {
      f.name = ["前场灯", "观众灯", "待布置灯"][i];
    });
    p.stage.placements.pop();
    return p;
  });
  const source = useRef(project);
  source.current = project;
  const stage = useRef<StageHandle>(null);
  const [pending, setPending] = useState(false),
    [busy, setBusy] = useState(false);
  const [error, setError] = useState(""),
    [visible, setVisible] = useState(true);
  const [reveal, setReveal] = useState(0),
    [edits, setEdits] = useState(0);
  const [saved, setSaved] = useState("");
  const history = useRef<ProjectView[]>([]);
  function apply(commands: EditOperation[]) {
    const next = structuredClone(source.current);
    for (const c of commands) {
      if (c.op !== "stage" || c.command.op !== "putPlacement")
        throw new Error("本验收只接受真实灯位命令");
      const placement = c.command.placement;
      const index = next.stage.placements.findIndex(
        (v) => v.fixtureId === placement.fixtureId,
      );
      if (index < 0) next.stage.placements.push(placement);
      else next.stage.placements[index] = placement;
    }
    history.current.push(source.current);
    source.current = next;
    setProject(next);
    setEdits((n) => n + 1);
  }
  async function flush() {
    setBusy(true);
    setError("");
    try {
      const ops = stage.current?.collect() ?? [];
      if (ops.length) apply(ops);
      stage.current?.accept();
      return true;
    } catch (e) {
      setError((e as Error).message);
      return false;
    } finally {
      setBusy(false);
    }
  }
  async function edit(command: EditCommand) {
    if (!(await flush())) return null;
    apply(command.op === "batch" ? command.commands : [command]);
    return source.current;
  }
  return (
    <main className="workbench" style={{ height: "100vh" }}>
      <PerformanceLayout
        mode="stage"
        revealEditing={reveal}
        beforeChange={flush}
        busy={busy}
        toolbar={
          <>
            <button
              onClick={() =>
                void stage.current?.selectFixtures(
                  ["front", "audience"],
                  () => true,
                )
              }
            >
              选择两台
            </button>
            <button
              onClick={async () => {
                if (await flush())
                  setSaved(JSON.stringify(source.current.stage.placements));
              }}
            >
              保存验收
            </button>
            <button
              onClick={async () => {
                if (await flush()) setVisible(!visible);
              }}
            >
              切换工作区
            </button>
            <button
              onClick={async () => {
                if (!(await flush())) return;
                const last = history.current.pop();
                if (last) {
                  source.current = last;
                  setProject(last);
                  setEdits((n) => n - 1);
                }
              }}
            >
              撤销验收
            </button>
            <button
              onClick={() =>
                setProject((p) => ({
                  ...p,
                  stage: {
                    ...p.stage,
                    editLocks: [{ kind: "placement", targetId: "front" }],
                  },
                }))
              }
            >
              外部锁定
            </button>
          </>
        }
      >
        <StageWorkspace
          ref={stage}
          project={project}
          visible={visible}
          busy={busy}
          error={error}
          beforeChange={flush}
          onEdit={edit}
          onPending={setPending}
          onSelectedFixtures={() => {}}
          onArrangementOpen={() => setReveal((n) => n + 1)}
        />
      </PerformanceLayout>
      <output aria-label="验收状态">
        修改 {edits} · 草稿 {String(pending)} · 工作区 {String(visible)} ·{" "}
        {error}
      </output>
      <output aria-label="实际灯位">
        {JSON.stringify(project.stage.placements)}
      </output>
      <output aria-label="保存内容">{saved}</output>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
