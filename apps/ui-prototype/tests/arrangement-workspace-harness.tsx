import { translationPreview } from "../src/components/stage/object-translation";
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
  const [hold, setHold] = useState(false);
  const [mounted, setMounted] = useState(true);
  const release = useRef<((accepted: boolean) => void) | null>(null);
  const history = useRef<ProjectView[]>([]);
  function apply(commands: EditOperation[]) {
    const next = structuredClone(source.current);
    for (const c of commands) {
      if (c.op !== "stage") throw new Error("本验收只接受场地命令");
      if (c.command.op === "translateObjects") {
        const preview = translationPreview(
          next.stage,
          c.command.targets.map((t) => ({ kind: t.kind, id: t.targetId })),
          c.command.deltaMeters,
        );
        next.stage.constructions = next.stage.constructions.map(
          (value) =>
            (
              preview({ kind: "construction", value }) as {
                kind: "construction";
                value: typeof value;
              }
            ).value,
        );
        next.stage.placements = next.stage.placements.map(
          (value) =>
            (
              preview({ kind: "placement", value }) as {
                kind: "placement";
                value: typeof value;
              }
            ).value,
        );
        continue;
      }
      if (c.command.op === "putSpace") {
        const { op: _, id, ...values } = c.command;
        const space = {
          ...values,
          id: id ?? `space-${next.stage.spaces.length}`,
        };
        const index = next.stage.spaces.findIndex((s) => s.id === space.id);
        if (index < 0) next.stage.spaces.push(space);
        else next.stage.spaces[index] = space;
        continue;
      }
      if (c.command.op !== "putPlacement")
        throw new Error("本验收只接受真实空间与灯位命令");
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
      const id = source.current.id;
      if (hold) {
        const accepted = await new Promise<boolean>((resolve) => {
          release.current = resolve;
        });
        release.current = null;
        if (!accepted) throw new Error("受控提交失败");
        if (source.current.id !== id) return false;
      }
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
            <label>
              <input
                type="checkbox"
                checked={hold}
                onChange={(e) => setHold(e.target.checked)}
              />
              等待提交
            </label>
            <button onClick={() => release.current?.(true)}>完成提交</button>
            <button onClick={() => release.current?.(false)}>拒绝提交</button>
            <button onClick={() => setVisible((v) => !v)}>外部显隐</button>
            <button onClick={() => setMounted((v) => !v)}>外部装卸</button>
            <button
              onClick={() => setProject((p) => ({ ...p, id: p.id + "-next" }))}
            >
              外部换工程
            </button>
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
                  setSaved(JSON.stringify(source.current.stage));
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
        {mounted && (
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
        )}
      </PerformanceLayout>
      <output aria-label="验收状态">
        修改 {edits} · 草稿 {String(pending)} · 工作区 {String(visible)} ·{" "}
        {error}
      </output>
      <output aria-label="实际灯位">
        {JSON.stringify(project.stage.placements)}
      </output>
      <output aria-label="保存内容">{saved}</output>
      <output aria-label="实际构件">
        {JSON.stringify(project.stage.constructions)}
      </output>
      <output aria-label="实际空间">
        {JSON.stringify(project.stage.spaces)}
      </output>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
