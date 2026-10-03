// Real workspace and draft lifecycle; deliberately controlled RPC replies, never a geometry engine.
import { createRoot } from "react-dom/client";
import { useRef, useState } from "react";
import {
  StageWorkspace,
  type StageHandle,
} from "../src/components/stage/StageWorkspace";
import { PerformanceLayout } from "../src/components/layout/PerformanceLayout";
import { stageProject } from "./stage-organization-fixture";
import type { EditCommand, EditOperation } from "../src/application-host";
import type {
  RiggingCommand,
  RiggingProjection,
} from "../src/rigging-preview-types";
import "../src/base.css";
import "../src/workbench.css";

function Harness() {
  const [project, setProject] = useState(stageProject);
  const source = useRef(project);
  source.current = project;
  const stage = useRef<StageHandle>(null);
  const [generation, setGeneration] = useState(1);
  const [pending, setPending] = useState(false),
    [busy, setBusy] = useState(false);
  const [visible, setVisible] = useState(true),
    [error, setError] = useState("");
  const [edits, setEdits] = useState(0),
    [reveal, setReveal] = useState(0);
  const [requests, setRequests] = useState(0),
    [failApply, setFailApply] = useState(false);
  const request = useRef<{
    generation: number;
    command: RiggingCommand;
    resolve(value: RiggingProjection): void;
    reject(error: Error): void;
  } | null>(null);
  const projected = useRef<RiggingProjection | null>(null);
  function complete(fail: boolean) {
    const job = request.current;
    if (!job) return;
    request.current = null;
    if (fail) {
      job.reject(new Error("受控预览失败"));
      return;
    }
    const result = {
      generation: job.generation,
      changed: !!job.command.layout,
      placements: source.current.stage.placements
        .filter((p) => job.command.fixtureIds.includes(p.fixtureId))
        .map((p) =>
          job.command.layout
            ? { ...p, positionMeters: { x: "2", y: "3", z: "5.75" } }
            : p,
        ),
    };
    projected.current = result;
    job.resolve(result);
  }
  function apply(ops: EditOperation[]) {
    if (failApply) throw new Error("受控提交失败，草稿保留");
    const next = structuredClone(source.current);
    for (const op of ops) {
      if (op.op !== "stage") throw new Error("仅验证场地命令");
      if (op.command.op === "putPlacement") {
        const value = op.command.placement;
        next.stage.placements = next.stage.placements.map((p) =>
          p.fixtureId === value.fixtureId ? value : p,
        );
      } else if (op.command.op === "attachFixtures") {
        const points = projected.current?.placements ?? [];
        next.stage.placements = next.stage.placements.map(
          (p) => points.find((v) => v.fixtureId === p.fixtureId) ?? p,
        );
      } else throw new Error("未准备的验收命令");
    }
    source.current = next;
    setProject(next);
    setGeneration((g) => g + 1);
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
                void stage.current?.selectFixtures(["front"], () => true)
              }
            >
              选择前灯
            </button>
            <button onClick={() => void flush()}>保存验收</button>
            <button
              onClick={async () => {
                if (await flush()) setVisible(!visible);
              }}
            >
              切换工作区
            </button>
            <button onClick={() => complete(false)}>完成预览</button>
            <button onClick={() => complete(true)}>预览失败</button>
            <label>
              <input
                type="checkbox"
                checked={failApply}
                onChange={(e) => setFailApply(e.target.checked)}
              />
              提交失败
            </label>
          </>
        }
      >
        <StageWorkspace
          ref={stage}
          project={project}
          generation={generation}
          visible={visible}
          busy={busy}
          error={error}
          beforeChange={flush}
          onEdit={edit}
          onPending={setPending}
          onSelectedFixtures={() => {}}
          onArrangementOpen={() => setReveal((n) => n + 1)}
          previewRigging={(g, c) => {
            setRequests((n) => n + 1);
            return new Promise((resolve, reject) => {
              request.current = { generation: g, command: c, resolve, reject };
            });
          }}
        />
      </PerformanceLayout>
      <output aria-label="验收状态">
        修改 {edits} · 草稿 {String(pending)} · 工作区 {String(visible)} · 请求{" "}
        {requests} · {error}
      </output>
      <output aria-label="实际灯位">
        {JSON.stringify(project.stage.placements)}
      </output>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
