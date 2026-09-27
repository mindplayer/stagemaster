import { WorkspaceSurface } from "./WorkspaceSurface";
import { forwardRef, useImperativeHandle, useRef, useState } from "react";
import {
  PlusIcon,
  CopyIcon,
  ArrowUpIcon,
  ArrowDownIcon,
  TrashIcon,
  ListNumbersIcon,
} from "@phosphor-icons/react";
import type {
  ApplicationHost,
  EditCommand,
  EditOperation,
  ProjectView,
} from "../../application-host";
import type { SequenceEdit, SequenceView } from "../../sequence-types";
import { uniqueName } from "../../editor-tools";
import {
  seconds,
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
}
export const SequenceWorkspace = forwardRef<
  SequenceHandle,
  {
    project: ProjectView;
    host: ApplicationHost;
    generation: number;
    busy: boolean;
    visible: boolean;
    onEdit(command: EditCommand): Promise<ProjectView | null>;
    beforeChange(): Promise<boolean>;
    onPending(value: boolean): void;
  }
>(function SequenceWorkspace(
  { project, host, generation, busy, visible, onEdit, beforeChange, onPending },
  ref,
) {
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
  useImperativeHandle(ref, () => ({ collect, accept: cancel }));
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
      <aside className="wb-library">
        <div className="wb-section-title">
          <h2>
            <ListNumbersIcon />
            场景列表
          </h2>
          <span>{project.sequences.length}</span>
        </div>
        <input
          aria-label="搜索列表"
          placeholder="搜索列表"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <div className="wb-library-actions">
          <button
            disabled={busy || !project.scenes.length}
            onClick={() => void addSequence()}
          >
            <PlusIcon />
            新建列表
          </button>
          <button
            aria-label="复制列表"
            disabled={busy || !sequence}
            onClick={() => void duplicateSequence()}
          >
            <CopyIcon />
          </button>
        </div>
        <div className="wb-scene-list">
          {lists.map((s) => (
            <button
              key={s.id}
              aria-pressed={sequence?.id === s.id}
              className={sequence?.id === s.id ? "active" : ""}
              disabled={busy}
              onClick={() => void chooseSequence(s.id)}
            >
              <div>
                <strong>{s.name}</strong>
                <small>
                  {s.steps.length} 步 · {s.repeat === "loop" ? "循环" : "单次"}
                </small>
              </div>
            </button>
          ))}
          {!lists.length && (
            <p className="wb-dim">
              {project.sequences.length
                ? "未找到列表"
                : project.scenes.length
                  ? "尚未创建列表"
                  : "请先在编排中创建场景"}
            </p>
          )}
        </div>
      </aside>
      <section className="wb-sequence-content">
        <div className="wb-content-heading">
          <div>
            <span className="wb-eyebrow">节目编排</span>
            <h1>{sequence?.name ?? "场景列表"}</h1>
          </div>
          {sequence && (
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
        {sequence && (
          <>
            <div className="wb-sequence-add">
              <select
                aria-label="要加入的场景"
                value={
                  project.scenes.some((s) => s.id === addSceneId)
                    ? addSceneId
                    : (project.scenes[0]?.id ?? "")
                }
                onChange={(e) => setAddSceneId(e.target.value)}
                disabled={busy}
              >
                {project.scenes.map((s) => (
                  <option value={s.id} key={s.id}>
                    {s.name}
                  </option>
                ))}
              </select>
              <button
                disabled={busy || !project.scenes.length}
                onClick={() => void insertStep()}
              >
                <PlusIcon />
                加入步骤
              </button>
            </div>
            <div className="wb-sequence-actions">
              <input
                aria-label="搜索步骤"
                placeholder="搜索编号、步骤或场景"
                value={stepQuery}
                onChange={(e) => setStepQuery(e.target.value)}
              />
              <button
                aria-label="上移步骤"
                disabled={busy || index <= 0}
                onClick={() =>
                  void edit({
                    kind: "moveStep",
                    id: sequence.id,
                    stepId: step!.id,
                    index: index - 1,
                  })
                }
              >
                <ArrowUpIcon />
              </button>
              <button
                aria-label="下移步骤"
                disabled={
                  busy || index < 0 || index >= sequence.steps.length - 1
                }
                onClick={() =>
                  void edit({
                    kind: "moveStep",
                    id: sequence.id,
                    stepId: step!.id,
                    index: index + 1,
                  })
                }
              >
                <ArrowDownIcon />
              </button>
              <button
                aria-label="复制步骤"
                disabled={busy || !step}
                onClick={() => void duplicateStep()}
              >
                <CopyIcon />
              </button>
              <button
                aria-label="删除步骤"
                disabled={busy || !step || sequence.steps.length === 1}
                title={
                  sequence.steps.length === 1
                    ? "列表至少保留一个步骤"
                    : "删除步骤"
                }
                onClick={() => {
                  setLocalError("");
                  setDeleteTarget("step");
                }}
              >
                <TrashIcon />
              </button>
            </div>
            {step && !steps.some((s) => s.id === step.id) && (
              <p className="wb-dim">
                当前选中“{step.name}”未匹配筛选。
                <button onClick={() => setStepQuery("")}>清除筛选</button>
              </p>
            )}
            <div className="wb-steps" role="group" aria-label="列表步骤">
              {steps.map((s) => (
                <button
                  aria-pressed={s.id === step?.id}
                  key={s.id}
                  disabled={busy}
                  className={s.id === step?.id ? "active" : ""}
                  onClick={() => void chooseStep(s.id)}
                  onKeyDown={(e) => {
                    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
                      e.preventDefault();
                      const next =
                        steps[
                          steps.indexOf(s) + (e.key === "ArrowDown" ? 1 : -1)
                        ];
                      if (next)
                        void chooseStep(next.id).then(
                          (ok) =>
                            ok &&
                            document.getElementById(`step-${next.id}`)?.focus(),
                        );
                    }
                  }}
                  id={`step-${s.id}`}
                >
                  <span className="wb-step-number">{s.number}</span>
                  <div>
                    <strong>{s.name}</strong>
                    <small>
                      {project.scenes.find((c) => c.id === s.sceneId)?.name}
                    </small>
                  </div>
                  <span className="wb-step-time">
                    渐变 {seconds(s.fadeMs)} 秒
                    <small>
                      {s.delayMs ? `延时 ${seconds(s.delayMs)} 秒 · ` : ""}
                      {s.waitMs === null
                        ? "手动推进"
                        : `等待 ${seconds(s.waitMs)} 秒后推进`}
                    </small>
                  </span>
                </button>
              ))}
              {!steps.length && <p className="wb-dim">未找到步骤</p>}
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
        />
      </section>
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
