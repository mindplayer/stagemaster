import { useStageVisibility } from "./useStageVisibility";
import { useArrangementEntry } from "./useArrangementEntry";
import { StageLayoutStatus } from "./StageLayoutStatus";
import type { RiggingPreviewPort } from "../../rigging-preview-types";
import { lockTargets, placementTargets, stageTarget } from "../../stage-locks";
import { DockPane } from "../layout/DockPane";
import type { ReactNode } from "react";
import { WorkspaceSurface } from "../workbench/WorkspaceSurface";
import { useStageObjects } from "./useStageObjects";
import { StageObjectDialogs } from "./StageObjectDialogs";
import { useStageRigging } from "./useStageRigging";
import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import type {
  EditCommand,
  EditOperation,
  ProjectView,
} from "../../application-host";
import type {
  FixturePlacement,
  StageEdit,
  StageObject,
  StageSelection,
} from "../../stage-types";
import { selectedStage, stageCommand, objectSpace } from "../../stage-tools";
import { validateEditorForm } from "../workbench/form-validation";
import { StageCanvas } from "./StageCanvas";
import { StageLibraryPanel } from "./StageLibraryPanel";
import { visibleStage, type PlanVisibility } from "./stage-display";
import { StageSelectionInspector } from "./StageSelectionInspector";
import { ArrangementInspector } from "./ArrangementInspector";
import { useStageArrangement } from "./useStageArrangement";
import { arrangementStage } from "./arrangement-session";
import { placementBatch, togglePlacement } from "../../placement-tools";
import "./stage.css";
export interface StageHandle {
  collect(): EditOperation[];
  accept(): void;
  revealFixture(id: string): void;
  selectFixtures(ids: string[], isActive: () => boolean): Promise<boolean>;
}
export const StageWorkspace = forwardRef<
  StageHandle,
  {
    project: ProjectView;
    onSelectedFixtures(ids: string[]): void;
    visible: boolean;
    canvasVisible?: boolean;
    viewControls?: ReactNode;
    onArrangementOpen?(): void;
    generation?: number;
    previewRigging?: RiggingPreviewPort;
    busy: boolean;
    error: string;
    beforeChange(): Promise<boolean>;
    onEdit(command: EditCommand): Promise<ProjectView | null>;
    onPending(value: boolean): void;
  }
>(function StageWorkspace(
  {
    project,
    onSelectedFixtures,
    visible,
    canvasVisible = true,
    viewControls,
    onArrangementOpen,
    generation = 0,
    previewRigging,
    busy,
    error,
    beforeChange,
    onEdit,
    onPending,
  },
  ref,
) {
  const [selection, setSelection] = useState<StageSelection | null>(null);
  const { planVisibility, setPlanVisibility, revealInPlan, revealPlacements } =
    useStageVisibility(project);
  const projectRef = useRef(project);
  projectRef.current = project;
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const selectedPlacements = project.stage.placements.filter((p) =>
    selectedIds.includes(p.fixtureId),
  );
  const liveIds =
    selection?.kind === "placement"
      ? selectedIds.filter((id) =>
          selectedPlacements.some((p) => p.fixtureId === id),
        )
      : [];
  const selectionKey = JSON.stringify(liveIds);
  useEffect(
    () => onSelectedFixtures(JSON.parse(selectionKey) as string[]),
    [selectionKey, onSelectedFixtures],
  );
  const [draft, setDraft] = useState<StageObject | null>(null),
    draftRef = useRef<StageObject | null>(null);
  const moving = useRef(false);
  const [focusRequest, setFocusRequest] = useState(0);
  const [query, setQuery] = useState("");
  const [fixtureId, setFixtureId] = useState("");
  const [localError, setLocalError] = useState("");
  const form = useRef<HTMLFormElement>(null);
  const arrangement = useStageArrangement({
    project,
    onPending,
    onAccepted: acceptPlacements,
  });
  const object = draft ?? selectedStage(project.stage, selection);
  const selectedSpace = objectSpace(project.stage, object);
  const unplaced = project.fixtures.filter(
    (f) => !project.stage.placements.some((p) => p.fixtureId === f.id),
  );
  const chosenFixture = unplaced.find((f) => f.id === fixtureId) ?? unplaced[0];
  const objects = useStageObjects({
    project,
    busy,
    visible,
    onError: setLocalError,
    object,
    selectedSpace,
    fixtureId: chosenFixture?.id,
    beforeChange,
    edit,
    onResult(target, next, focus) {
      if (target) revealInPlan(target, next);
      setSelection(target);
      setSelectedIds(target?.kind === "placement" ? [target.id] : []);
      setQuery("");
      cancel();
      if (focus) setFocusRequest((v) => v + 1);
    },
  });
  const rigging = useStageRigging({
    project,
    visible,
    onError: setLocalError,
    object,
    liveIds,
    beforeChange,
    edit,
    busy,
    error,
    generation,
    preview: previewRigging,
    onPending,
    onOpen() {
      onArrangementOpen?.();
    },
    onAccepted: acceptPlacements,
    onResult(ids, next) {
      revealPlacements(ids, next);
      setSelectedIds(ids);
      setSelection(ids.length ? { kind: "placement", id: ids.at(-1)! } : null);
      setQuery("");
      setFocusRequest((v) => v + 1);
      cancel();
    },
  });
  const rigTarget = rigging.draft.session?.rig;
  const enterArrangement = useArrangementEntry({
    project,
    visible,
    busy,
    beforeChange,
    cancel,
    open: arrangement.open,
    reveal: revealPlacements,
    onOpen: onArrangementOpen,
    onError: setLocalError,
  });
  useEffect(() => {
    if (rigTarget) revealInPlan({ kind: "construction", id: rigTarget });
  }, [rigTarget]);
  function acceptPlacements(ids: string[], placements: FixturePlacement[]) {
    revealPlacements(ids, {
      ...projectRef.current,
      stage: arrangementStage(projectRef.current.stage, placements),
    });
    setSelectedIds(ids);
    setSelection({ kind: "placement", id: ids.at(-1)! });
    setQuery("");
  }
  function cancel() {
    form.current
      ?.querySelectorAll<
        HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement
      >("input,select,textarea")
      .forEach((field) => field.setCustomValidity(""));
    arrangement.cancel();
    rigging.draft.cancel();
    draftRef.current = null;
    setDraft(null);
    setLocalError("");
    onPending(moving.current);
  }
  async function changePlanVisibility(next: PlanVisibility) {
    if (!(await beforeChange())) return;
    const shown = visibleStage(projectRef.current.stage, next);
    const ids = liveIds.filter((id) =>
      shown.placements.some((p) => p.fixtureId === id),
    );
    setSelectedIds(ids);
    setSelection(
      selection?.kind === "placement"
        ? ids.length
          ? { kind: "placement", id: ids.at(-1)! }
          : null
        : selectedStage(shown, selection)
          ? selection
          : null,
    );
    setPlanVisibility(next);
    cancel();
  }
  function collect(): EditOperation[] {
    if (moving.current) throw new Error("请先完成拖动，或按 Esc 取消");
    if (arrangement.session) return arrangement.collect();
    if (rigging.draft.session) return rigging.draft.collect();
    if (!draftRef.current) return [];
    try {
      validateEditorForm(form.current);
      return [{ op: "stage", command: stageCommand(draftRef.current) }];
    } catch (reason) {
      setLocalError(reason instanceof Error ? reason.message : String(reason));
      throw reason;
    }
  }
  useImperativeHandle(ref, () => ({
    collect,
    accept() {
      arrangement.accept();
      rigging.draft.accept();
      cancel();
    },
    async selectFixtures(ids, isActive) {
      if (!isActive() || !(await beforeChange()) || !isActive()) return false;
      if (
        ids.length > 1024 ||
        new Set(ids).size !== ids.length ||
        ids.some(
          (id) =>
            !projectRef.current.stage.placements.some(
              (p) => p.fixtureId === id,
            ),
        )
      )
        return false;
      revealPlacements(ids);
      setSelection(ids.length ? { kind: "placement", id: ids.at(-1)! } : null);
      setSelectedIds(ids);
      cancel();
      return true;
    },
    revealFixture(id) {
      const placed = project.stage.placements.some((p) => p.fixtureId === id);
      if (placed) revealInPlan({ kind: "placement", id });
      setFixtureId(id);
      setQuery("");
      setSelection(placed ? { kind: "placement", id } : null);
      setSelectedIds(placed ? [id] : []);
      setFocusRequest((n) => n + 1);
      cancel();
      requestAnimationFrame(() =>
        document
          .querySelector<HTMLSelectElement>('select[aria-label="待布置灯具"]')
          ?.focus(),
      );
    },
  }));
  async function choose(
    target: StageSelection,
    additive = false,
    preserve = false,
  ) {
    if (await beforeChange()) {
      revealInPlan(target);
      if (target.kind === "placement") {
        const next =
          preserve && liveIds.includes(target.id)
            ? liveIds
            : togglePlacement(liveIds, target.id, additive);
        setSelectedIds(next);
        setSelection(
          next.length ? { kind: "placement", id: next.at(-1)! } : null,
        );
      } else {
        setSelection(target);
        setSelectedIds([]);
      }
      cancel();
    }
  }
  async function choosePlacements(ids: string[], additive = false) {
    if (!(await beforeChange())) return;
    const next = additive ? [...new Set([...liveIds, ...ids])] : ids;
    revealPlacements(next);
    setSelectedIds(next);
    setSelection(next.length ? { kind: "placement", id: next.at(-1)! } : null);
    cancel();
  }
  async function setLocked(locked: boolean) {
    if (!(await beforeChange())) return;
    const targets = liveIds.length
      ? placementTargets(liveIds)
      : object
        ? [stageTarget(object)]
        : [];
    if (!targets.length) return;
    const next = await edit({
      op: "setEditLocks",
      targets: lockTargets(targets),
      locked,
    });
    if (next) cancel();
  }
  function arrange(selected = false) {
    return enterArrangement({
      ids: selected ? [...liveIds] : null,
      spaceId: selectedSpace?.id,
    });
  }
  function edit(command: StageEdit) {
    return onEdit({ op: "stage", command });
  }
  return (
    <WorkspaceSurface
      className="stage-workspace"
      visible={visible}
      label="舞台工作区"
    >
      <DockPane region="library" visible={visible}>
        <StageLibraryPanel
          project={project}
          busy={busy}
          query={query}
          onQuery={setQuery}
          selection={selection}
          selectedIds={liveIds}
          visibility={planVisibility}
          onVisibility={(next) => void changePlanVisibility(next)}
          onSelect={(target, additive) => void choose(target, additive)}
          onCreate={(kind) => void objects.create(kind)}
          onCreateRig={() => void objects.createRig()}
          onCreateSeating={() => void objects.createSeating()}
          onArrange={() => void arrange(false)}
          onPlace={() => void objects.placeFixture()}
          fixtureId={fixtureId}
          onFixtureId={setFixtureId}
        />
      </DockPane>
      <DockPane region="viewport" visible={visible && canvasVisible}>
        <div className="stage-center">
          {viewControls}
          {arrangement.session && (
            <StageLayoutStatus
              kind="排列"
              count={arrangement.session.ids.length}
              problem={arrangement.problem}
            />
          )}
          {rigging.draft.session && (
            <StageLayoutStatus
              kind="挂接"
              count={rigging.draft.session.ids.length}
              problem={rigging.draft.problem}
              computing={rigging.draft.computing}
            />
          )}
          <div className="stage-plan-container">
            <StageCanvas
              project={project}
              visibility={planVisibility}
              selection={selection}
              selectedIds={
                rigging.draft.session?.ids ??
                arrangement.session?.ids ??
                liveIds
              }
              placementPreview={
                rigging.draft.projection?.placements ?? arrangement.preview
              }
              draftConstructionId={rigging.draft.session?.rig}
              placementEditing={
                !!arrangement.session || !!rigging.draft.session
              }
              preview={draft}
              focusRequest={focusRequest}
              busy={busy}
              pending={
                draft !== null ||
                !!arrangement.session ||
                !!rigging.draft.session
              }
              onSelect={(target, additive, preserve) =>
                void choose(target, additive, preserve)
              }
              onSelectPlacements={(ids, additive) =>
                void choosePlacements(ids, additive)
              }
              onMovePlacements={(placements) => {
                try {
                  void onEdit(placementBatch(placements));
                } catch (reason) {
                  setLocalError((reason as Error).message);
                }
              }}
              onArrange={() => void arrange(true)}
              onMove={(value) => {
                void edit(stageCommand(value));
              }}
              onGesture={(value) => {
                moving.current = value;
                onPending(
                  value ||
                    draftRef.current !== null ||
                    arrangement.pending() ||
                    rigging.draft.pending(),
                );
              }}
            />
          </div>
        </div>
      </DockPane>
      <DockPane region="inspector" visible={visible}>
        {rigging.inspector ||
          (arrangement.session ? (
            <ArrangementInspector
              project={project}
              arrangement={arrangement}
              busy={busy}
              error={error}
              onApply={() => {
                arrangement.prepareApply();
                void beforeChange();
              }}
            />
          ) : (
            <StageSelectionInspector
              ids={liveIds}
              onArrange={() => void arrange(true)}
              onClear={() => void choosePlacements([])}
              object={object}
              project={project}
              pending={draft !== null}
              busy={busy}
              form={form}
              error={localError || error}
              onChange={(value) => {
                draftRef.current = value;
                setDraft(value);
                onPending(true);
                setLocalError("");
              }}
              onApply={() => {
                void beforeChange();
              }}
              onLock={(locked) => void setLocked(locked)}
              onCancel={cancel}
              onDuplicate={() => void objects.duplicate()}
              onDelete={() => void objects.requestDelete()}
              onEnclose={() => void objects.enclose()}
              onHang={() => void rigging.hang()}
              onSelectMounted={(id) =>
                void choosePlacements(
                  project.stage.attachments
                    .filter((a) => a.constructionId === id)
                    .map((a) => a.fixtureId),
                )
              }
              onDetach={(ids) => void rigging.detach(ids)}
            />
          ))}
      </DockPane>
      <StageObjectDialogs
        project={project}
        busy={busy}
        error={localError || error}
        actions={objects}
      />
    </WorkspaceSurface>
  );
});
