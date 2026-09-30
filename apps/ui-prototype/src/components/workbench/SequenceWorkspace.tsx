import { SequenceLibrary } from "./SequenceLibrary";
import { SequenceEditToolbar } from "./SequenceEditToolbar";
import { executionPosition } from "./execution-position";
import "./execution-view.css";
import { SequenceStepList } from "./SequenceStepList";
import { DockPane } from "../layout/DockPane";
import { WorkspaceSurface } from "./WorkspaceSurface";
import { forwardRef, useImperativeHandle, useRef, useState } from "react";
import { TrashIcon } from "@phosphor-icons/react";
import type {
  ApplicationHost,
  EditCommand,
  EditOperation,
  ProjectView,
} from "../../application-host";
import type { SequenceEdit } from "../../sequence-types";
import { uniqueName } from "../../editor-tools";
import {
  sequenceDraft,
  sequenceCommands,
  SequenceInputError,
  type SequenceDraft as Draft,
} from "../../sequence-tools";
import { SequenceInspector } from "./SequenceInspector";
import { PreviewPanel } from "./PreviewPanel";
import { DeleteDialog } from "./DeleteDialog";

export interface SequenceHandle {
  collect(): EditOperation[];
  accept(): void;
  reveal(id: string): void;
}
export const SequenceWorkspace = forwardRef<
  SequenceHandle,
  {
    project: ProjectView;
    host: ApplicationHost;
    generation: number;
    busy: boolean;
    visible: boolean;
    onView3d?(): void;
    execution: boolean;
    onExecution(value: boolean): void;
    onEdit(command: EditCommand): Promise<ProjectView | null>;
    beforeChange(): Promise<boolean>;
    onPending(value: boolean): void;
  }
>(function SequenceWorkspace(
  {
    project,
    host,
    generation,
    busy,
    visible,
    onView3d,
    execution,
    onExecution,
    onEdit,
    beforeChange,
    onPending,
  },
  ref,
) {
  const [position, setPosition] = useState(() => executionPosition(null));
  const [sequenceId, setSequenceId] = useState(project.sequences[0]?.id ?? "");
  const [stepId, setStepId] = useState(
    project.sequences[0]?.steps[0]?.id ?? "",
  );
  const rememberedSteps = useRef<Record<string, string>>({});
  const [query, setQuery] = useState("");
  const [stepQuery, setStepQuery] = useState("");
  const [addSceneId, setAddSceneId] = useState("");
  const [draft, setDraft] = useState<Draft | null>(null);
  const draftRef = useRef<Draft | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<"sequence" | "step" | null>(
    null,
  );
  const [localError, setLocalError] = useState("");
  const form = useRef<HTMLFormElement>(null);
  const sequence =
    project.sequences.find((s) => s.id === sequenceId) ?? project.sequences[0];
  const step =
    sequence?.steps.find((s) => s.id === stepId) ?? sequence?.steps[0];
  const data =
    draft ?? (sequence && step ? sequenceDraft(sequence, step) : null);
  function pinSelection() {
    if (sequence && step) {
      setSequenceId(sequence.id);
      setStepId(step.id);
      rememberedSteps.current[sequence.id] = step.id;
    }
  }
  function change(patch: Partial<Draft>) {
    pinSelection();
    if (data) {
      const next = { ...data, ...patch };
      draftRef.current = next;
      setDraft(next);
      onPending(true);
      setLocalError("");
    }
  }
  function cancel() {
    draftRef.current = null;
    setDraft(null);
    setLocalError("");
    onPending(false);
  }
  function collect(): EditOperation[] {
    const d = draftRef.current;
    if (!d || !sequence || !step) return [];
    try {
      return sequenceCommands(d, sequence, step);
    } catch (reason) {
      setLocalError(reason instanceof Error ? reason.message : String(reason));
      if (reason instanceof SequenceInputError)
        requestAnimationFrame(() =>
          form.current
            ?.querySelector<HTMLInputElement>(`[name="${reason.field}"]`)
            ?.focus(),
        );
      throw reason;
    }
  }
  useImperativeHandle(ref, () => ({
    collect,
    accept: cancel,
    reveal(id) {
      const target = project.sequences.find((s) => s.id === id);
      if (!target) return;
      setSequenceId(id);
      setStepId(
        target.steps.find((s) => s.id === rememberedSteps.current[id])?.id ??
          target.steps[0]?.id ??
          "",
      );
      setQuery("");
      setStepQuery("");
      cancel();
    },
  }));
  const edit = (command: SequenceEdit) => {
    pinSelection();
    return onEdit({ op: "sequence", command });
  };
  async function chooseSequence(id: string) {
    if (await beforeChange()) {
      setSequenceId(id);
      const next = project.sequences.find((s) => s.id === id);
      setStepId(rememberedSteps.current[id] ?? next?.steps[0]?.id ?? "");
      setStepQuery("");
      cancel();
    }
  }
  async function chooseStep(id: string) {
    if (await beforeChange()) {
      setStepId(id);
      if (sequence) rememberedSteps.current[sequence.id] = id;
      cancel();
      return true;
    }
    return false;
  }
  async function addSequence() {
    const scene =
      project.scenes.find((s) => s.id === addSceneId) ?? project.scenes[0];
    if (!scene) return;
    const next = await edit({
      kind: "add",
      name: uniqueName(
        "场景列表",
        project.sequences.map((s) => s.name),
      ),
      sceneId: scene.id,
    });
    if (next) {
      setSequenceId(next.sequences.at(-1)!.id);
      setStepId(next.sequences.at(-1)!.steps[0]!.id);
      setQuery("");
      cancel();
    }
  }
  async function insertStep() {
    if (!sequence) return;
    const scene =
      project.scenes.find((s) => s.id === addSceneId) ?? project.scenes[0];
    if (!scene) return;
    const previous = new Set(sequence.steps.map((s) => s.id));
    const next = await edit({
      kind: "insertStep",
      id: sequence.id,
      sceneId: scene.id,
      afterId: step?.id ?? null,
    });
    if (next) {
      setStepId(
        next.sequences
          .find((s) => s.id === sequence.id)!
          .steps.find((s) => !previous.has(s.id))!.id,
      );
      setStepQuery("");
      cancel();
    }
  }
  async function duplicateStep() {
    if (!sequence || !step) return;
    const ids = new Set(sequence.steps.map((s) => s.id));
    const next = await edit({
      kind: "duplicateStep",
      id: sequence.id,
      stepId: step.id,
    });
    if (next) {
      setStepId(
        next.sequences
          .find((s) => s.id === sequence.id)!
          .steps.find((s) => !ids.has(s.id))!.id,
      );
      setStepQuery("");
      cancel();
    }
  }
  async function duplicateSequence() {
    if (!sequence) return;
    const next = await edit({
      kind: "duplicate",
      id: sequence.id,
      name: uniqueName(
        `${sequence.name} 副本`,
        project.sequences.map((s) => s.name),
      ),
    });
    if (next) {
      setSequenceId(next.sequences.at(-1)!.id);
      setStepId(next.sequences.at(-1)!.steps[0]!.id);
      setQuery("");
      cancel();
    }
  }
  const lists = project.sequences.filter((s) =>
    s.name.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()),
  );
  const steps =
    sequence?.steps.filter((s) =>
      `${s.number} ${s.name} ${project.scenes.find((c) => c.id === s.sceneId)?.name ?? ""}`
        .toLocaleLowerCase()
        .includes(stepQuery.trim().toLocaleLowerCase()),
    ) ?? [];
  const index = sequence?.steps.findIndex((s) => s.id === step?.id) ?? -1;
  return (
    <WorkspaceSurface
      className="wb-sequence-workspace"
      visible={visible}
      label="列表工作区"
    >
      <DockPane region="library" visible={visible}>
        <SequenceLibrary
          lists={lists}
          count={project.sequences.length}
          selectedId={sequence?.id}
          hasScenes={!!project.scenes.length}
          query={query}
          setQuery={setQuery}
          busy={busy}
          onAdd={() => void addSequence()}
          onDuplicate={() => void duplicateSequence()}
          onSelect={(id) => void chooseSequence(id)}
        />
      </DockPane>
      <DockPane region={execution ? "full" : "editor"} visible={visible}>
        <section className="wb-sequence-content" data-execution={execution}>
          <div className="wb-content-heading">
            <div>
              <span className="wb-eyebrow">节目编排</span>
              <h1>{sequence?.name ?? "场景列表"}</h1>
            </div>
            <div
              className="execution-view-switch"
              role="group"
              aria-label="列表工作方式"
            >
              <button
                disabled={busy}
                aria-pressed={!execution}
                onClick={() =>
                  void beforeChange().then((ok) => ok && onExecution(false))
                }
              >
                步骤编排
              </button>
              <button
                disabled={busy}
                aria-pressed={execution}
                onClick={() =>
                  void beforeChange().then((ok) => ok && onExecution(true))
                }
              >
                执行视图
              </button>
            </div>
            {sequence && !execution && (
              <button
                aria-label="删除列表"
                disabled={busy}
                onClick={() => {
                  setLocalError("");
                  setDeleteTarget("sequence");
                }}
              >
                <TrashIcon />
              </button>
            )}
          </div>
          {execution && (
            <div className="execution-list-picker">
              <label htmlFor="execution-list">执行列表</label>
              <select
                id="execution-list"
                disabled={busy}
                value={sequence?.id ?? ""}
                onChange={(e) => void chooseSequence(e.target.value)}
              >
                {project.sequences.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.name}
                  </option>
                ))}
              </select>
              <span className="wb-dim">{sequence?.steps.length ?? 0} 步</span>
            </div>
          )}
          {sequence && (
            <>
              {execution ? (
                <div className="execution-search">
                  <input
                    aria-label="搜索步骤"
                    placeholder="搜索编号、步骤或场景"
                    value={stepQuery}
                    onChange={(e) => setStepQuery(e.target.value)}
                  />
                  <button
                    disabled={
                      !position.currentId || position.sequenceId !== sequence.id
                    }
                    onClick={() => {
                      setStepQuery("");
                      requestAnimationFrame(() =>
                        document
                          .getElementById(`step-${position.currentId}`)
                          ?.scrollIntoView({ block: "center" }),
                      );
                    }}
                  >
                    定位当前
                  </button>
                </div>
              ) : (
                <SequenceEditToolbar
                  scenes={project.scenes}
                  busy={busy}
                  addSceneId={addSceneId}
                  setAddSceneId={setAddSceneId}
                  stepQuery={stepQuery}
                  setStepQuery={setStepQuery}
                  index={index}
                  stepCount={sequence.steps.length}
                  hasStep={!!step}
                  onInsert={() => void insertStep()}
                  onDuplicate={() => void duplicateStep()}
                  onMove={(index) =>
                    void edit({
                      kind: "moveStep",
                      id: sequence.id,
                      stepId: step!.id,
                      index,
                    })
                  }
                  onDelete={() => {
                    setLocalError("");
                    setDeleteTarget("step");
                  }}
                />
              )}
              <div className="wb-steps-region">
                {step && !steps.some((s) => s.id === step.id) && (
                  <p className="wb-dim">
                    当前选中“{step.name}”未匹配筛选。
                    <button onClick={() => setStepQuery("")}>清除筛选</button>
                  </p>
                )}
                <SequenceStepList
                  steps={steps}
                  selectedId={step?.id ?? ""}
                  scenes={project.scenes}
                  busy={busy}
                  onSelect={chooseStep}
                  position={
                    position.sequenceId === sequence.id ? position : undefined
                  }
                />
              </div>
            </>
          )}
          <PreviewPanel
            host={host}
            sequence={sequence}
            stepId={step?.id ?? ""}
            generation={generation}
            busy={busy}
            beforeAction={beforeChange}
            visible={visible}
            onView3d={onView3d}
            execution={execution}
            onPosition={setPosition}
          />
        </section>
      </DockPane>
      <DockPane region="inspector" visible={visible && !execution}>
        <aside className="wb-properties wb-sequence-properties">
          {sequence && step && data && (
            <SequenceInspector
              form={form}
              busy={busy}
              index={index}
              sequence={sequence}
              project={project}
              data={data}
              pending={!!draft}
              localError={localError}
              change={change}
              onApply={() => void beforeChange()}
              onCancel={cancel}
            />
          )}
        </aside>
      </DockPane>
      {deleteTarget && (
        <DeleteDialog
          name={
            deleteTarget === "sequence"
              ? (sequence?.name ?? "")
              : (step?.name ?? "")
          }
          busy={busy}
          error={localError}
          onCancel={() => setDeleteTarget(null)}
          onDelete={() => {
            if (!sequence || !step) return;
            void edit(
              deleteTarget === "sequence"
                ? { kind: "remove", id: sequence.id }
                : { kind: "removeStep", id: sequence.id, stepId: step.id },
            ).then((next) => {
              if (next) {
                setDeleteTarget(null);
                cancel();
              } else setLocalError("未能删除，请查看错误提示并重试");
            });
          }}
        />
      )}
    </WorkspaceSurface>
  );
});
