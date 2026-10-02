import { useRevealItem, type RevealItem } from "../layout/useRevealItem";
import { stepMatches } from "../../sequence-script-tools";
import { SequenceLibrary } from "./SequenceLibrary";
import { SequenceEditToolbar } from "./SequenceEditToolbar";
import { executionPosition } from "./execution-position";
import "./execution-view.css";
import { SequenceStepBrowser } from "./SequenceStepBrowser";
import { DockPane } from "../layout/DockPane";
import { WorkspaceSurface } from "./WorkspaceSurface";
import { forwardRef, useImperativeHandle, useRef, useState } from "react";
import { SequenceWorkspaceHeading } from "./SequenceWorkspaceHeading";
import { SequenceGroupEditor } from "./SequenceGroupEditor";
import type { GroupPropertiesHandle } from "./SequenceGroupProperties";
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
import { SequenceExecutionPanel } from "../execution/SequenceExecutionPanel";
import { DeleteDialog } from "./DeleteDialog";

export interface SequenceHandle {
  collect(): EditOperation[];
  accept(): void;
  reveal(id: string, stepId?: string): boolean;
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
  const [revealRequest, setRevealRequest] = useState<RevealItem | null>(null);
  const revealRoot = useRevealItem(revealRequest, visible, busy);
  const [batch, setBatch] = useState(false);
  const [background, setBackground] = useState(false);
  const groupProperties = useRef<GroupPropertiesHandle>(null);
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
    groupProperties.current?.accept();
    draftRef.current = null;
    setDraft(null);
    setLocalError("");
    onPending(false);
  }
  function collect(): EditOperation[] {
    const d = draftRef.current;
    const grouped = groupProperties.current?.collect() ?? [];
    if (!d || !sequence || !step) return grouped;
    try {
      return [...grouped, ...sequenceCommands(d, sequence, step)];
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
    reveal(id, requestedStep) {
      const target = project.sequences.find((s) => s.id === id);
      if (
        !target ||
        (requestedStep &&
          !target.steps.some((step) => step.id === requestedStep))
      )
        return false;
      setSequenceId(id);
      setStepId(
        requestedStep ??
          target.steps.find((s) => s.id === rememberedSteps.current[id])?.id ??
          target.steps[0]?.id ??
          "",
      );
      setQuery("");
      setStepQuery("");
      if (requestedStep) {
        rememberedSteps.current[id] = requestedStep;
        setBatch(false);
        onExecution(false);
        setRevealRequest((previous) => ({
          id: requestedStep,
          serial: (previous?.serial ?? 0) + 1,
        }));
      }
      cancel();
      return true;
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
      stepMatches(
        s,
        project.scenes.find((c) => c.id === s.sceneId)?.name ?? "",
        stepQuery,
      ),
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
        <section
          ref={revealRoot}
          className="wb-sequence-content"
          data-execution={execution}
          data-background={execution && background}
        >
          <SequenceWorkspaceHeading
            sequence={sequence}
            sequences={project.sequences}
            execution={execution}
            batch={batch}
            busy={busy}
            position={position}
            onExecution={(value) =>
              void beforeChange().then((ok) => ok && onExecution(value))
            }
            onBatch={() =>
              void beforeChange().then((ok) => ok && setBatch(!batch))
            }
            onChoose={(id) => void chooseSequence(id)}
            onDelete={() => {
              setLocalError("");
              setDeleteTarget("sequence");
            }}
          />
          {sequence &&
            !(execution && background) &&
            (!execution && batch ? (
              <SequenceGroupEditor
                key={sequence.id}
                sequence={sequence}
                scenes={project.scenes}
                busy={busy}
                visible={visible}
                query={stepQuery}
                setQuery={setStepQuery}
                onEdit={edit}
                propertiesRef={groupProperties}
                beforeChange={beforeChange}
                onPending={onPending}
              />
            ) : (
              <SequenceStepBrowser
                sequence={sequence}
                steps={steps}
                selectedId={step?.id ?? ""}
                scenes={project.scenes}
                busy={busy}
                visible={visible}
                execution={execution}
                position={position}
                query={stepQuery}
                setQuery={setStepQuery}
                onSelect={chooseStep}
              >
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
              </SequenceStepBrowser>
            ))}
          <SequenceExecutionPanel
            project={project}
            background={background}
            onBackgroundChange={setBackground}
            host={host}
            sequence={sequence}
            stepId={!execution && batch ? "" : (step?.id ?? "")}
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
      <DockPane region="inspector" visible={visible && !execution && !batch}>
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
