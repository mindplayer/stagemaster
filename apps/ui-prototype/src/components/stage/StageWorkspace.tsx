import {
  lockTargets,
  placementTargets,
  stageTarget,
  movementBlocker,
} from "../../stage-locks";
import { StageMultiInspector } from "./StageMultiInspector";
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
import { bounds, selectedStage, stageCommand } from "../../stage-tools";
import { validateEditorForm } from "../workbench/form-validation";
import { StageCanvas } from "./StageCanvas";
import { StageLibraryPanel } from "./StageLibraryPanel";
import {
  ALL_VISIBLE,
  visibleStage,
  revealStageTarget,
  type PlanVisibility,
} from "./stage-display";
import { StageInspector } from "./StageInspector";
import { ArrangementDialog } from "./ArrangementDialog";
import {
  arrangementDraft,
  placementBatch,
  togglePlacement,
  type ArrangementDraft,
} from "../../placement-tools";
import "./stage.css";
export interface StageHandle {
  collect(): EditOperation[];
  accept(): void;
  revealFixture(id: string): void;
  selectFixture(id: string): Promise<boolean>;
}
export const StageWorkspace = forwardRef<
  StageHandle,
  {
    project: ProjectView;
    onSelectedFixture(id: string): void;
    visible: boolean;
    canvasVisible?: boolean;
    viewControls?: ReactNode;
    busy: boolean;
    error: string;
    beforeChange(): Promise<boolean>;
    onEdit(command: EditCommand): Promise<ProjectView | null>;
    onPending(value: boolean): void;
  }
>(function StageWorkspace(
  {
    project,
    onSelectedFixture,
    visible,
    canvasVisible = true,
    viewControls,
    busy,
    error,
    beforeChange,
    onEdit,
    onPending,
  },
  ref,
) {
  const [selection, setSelection] = useState<StageSelection | null>(null);
  const [planVisibility, setPlanVisibility] =
    useState<PlanVisibility>(ALL_VISIBLE);
  const projectRef = useRef(project);
  projectRef.current = project;
  const selectedFixture = selection?.kind === "placement" ? selection.id : "";
  useEffect(
    () => onSelectedFixture(selectedFixture),
    [selectedFixture, onSelectedFixture],
  );
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
  const [arrangement, setArrangement] = useState<{
    ids: string[];
    draft: ArrangementDraft;
  } | null>(null);
  const [draft, setDraft] = useState<StageObject | null>(null),
    draftRef = useRef<StageObject | null>(null);
  const moving = useRef(false);
  const [focusRequest, setFocusRequest] = useState(0);
  const [query, setQuery] = useState("");
  const [fixtureId, setFixtureId] = useState("");
  const [localError, setLocalError] = useState("");
  const form = useRef<HTMLFormElement>(null);
  const object = draft ?? selectedStage(project.stage, selection);
  const selectedSpace =
    object?.kind === "space"
      ? object.value
      : project.stage.spaces.find(
          (s) =>
            s.id ===
            (object?.kind === "placement"
              ? object.value.spaceId
              : object?.kind === "construction"
                ? object.value.shape.spaceId
                : null),
        );
  const unplaced = project.fixtures.filter(
    (f) => !project.stage.placements.some((p) => p.fixtureId === f.id),
  );
  const chosenFixture = unplaced.find((f) => f.id === fixtureId) ?? unplaced[0];
  const objects = useStageObjects({
    project,
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
    object,
    liveIds,
    beforeChange,
    edit,
    busy,
    error,
    onResult(ids, next) {
      revealPlacements(ids, next);
      setSelectedIds(ids);
      setSelection(ids.length ? { kind: "placement", id: ids.at(-1)! } : null);
      setQuery("");
      setFocusRequest((v) => v + 1);
      cancel();
    },
  });
  function cancel() {
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
  function revealInPlan(target: StageSelection, source = projectRef.current) {
    setPlanVisibility((old) => revealStageTarget(source.stage, old, target));
  }
  function revealPlacements(ids: string[], source = projectRef.current) {
    setPlanVisibility((old) =>
      ids.reduce(
        (value, id) =>
          revealStageTarget(source.stage, value, { kind: "placement", id }),
        old,
      ),
    );
  }
  function collect(): EditOperation[] {
    if (moving.current) throw new Error("请先完成拖动，或按 Esc 取消");
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
    accept: cancel,
    async selectFixture(id) {
      if (!(await beforeChange())) return false;
      if (id && !project.stage.placements.some((p) => p.fixtureId === id))
        return false;
      if (id) revealInPlan({ kind: "placement", id });
      setSelection(id ? { kind: "placement", id } : null);
      setSelectedIds(id ? [id] : []);
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
  async function arrange(selected = false) {
    if (selected && movementBlocker(project.stage, placementTargets(liveIds))) {
      setLocalError("所选灯位包含锁定对象，请先解锁再移动或排列");
      return;
    }
    if (!(await beforeChange())) return;
    const ids = selected ? liveIds : unplaced.map((f) => f.id);
    const items = project.stage.placements.filter((p) =>
      ids.includes(p.fixtureId),
    );
    const points: [number, number][] = items.length
      ? items.map((p) => [
          Number(p.positionMeters.x),
          Number(p.positionMeters.y),
        ])
      : (selectedSpace?.outlineMeters.map((p) => [
          Number(p[0]),
          Number(p[1]),
        ]) ?? []);
    const b = points.length ? bounds(points) : null;
    const initial = arrangementDraft(
      b ? (b.minX + b.maxX) / 2 : 0,
      b ? (b.minY + b.maxY) / 2 : 0,
      items.length
        ? Number(items[0].positionMeters.z)
        : Number(selectedSpace?.floorElevationMeters ?? 0) +
            Number(selectedSpace?.clearHeightMeters ?? 4) -
            0.5,
      selectedSpace?.id ?? null,
    );
    if (selected) initial.mode = "move";
    setArrangement({ ids, draft: initial });
  }
  async function applyPlacements(placements: FixturePlacement[]) {
    const next = await onEdit(placementBatch(placements));
    if (!next) return false;
    const ids = placements.map((p) => p.fixtureId);
    revealPlacements(ids, next);
    setSelectedIds(ids);
    setSelection({ kind: "placement", id: ids.at(-1)! });
    setQuery("");
    cancel();
    setFocusRequest((v) => v + 1);
    return true;
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
          onArrange={() => void arrange(false)}
          onPlace={() => void objects.placeFixture()}
          fixtureId={fixtureId}
          onFixtureId={setFixtureId}
        />
      </DockPane>
      <DockPane region="viewport" visible={visible && canvasVisible}>
        <div className="stage-center">
          {viewControls}
          <div className="stage-plan-container">
            <StageCanvas
              project={project}
              visibility={planVisibility}
              selection={selection}
              selectedIds={liveIds}
              preview={draft}
              focusRequest={focusRequest}
              busy={busy}
              pending={draft !== null}
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
                onPending(value || draftRef.current !== null);
              }}
            />
          </div>
        </div>
      </DockPane>
      <DockPane region="inspector" visible={visible}>
        {selection?.kind === "placement" && liveIds.length > 1 ? (
          <StageMultiInspector
            project={project}
            ids={liveIds}
            busy={busy}
            error={localError || error}
            onArrange={() => void arrange(true)}
            onHang={() => void rigging.hang()}
            onDetach={() => void rigging.detach(liveIds)}
            onClear={() => void choosePlacements([])}
            onLock={(locked) => void setLocked(locked)}
          />
        ) : (
          <StageInspector
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
        )}
      </DockPane>
      {rigging.dialog}
      {arrangement && (
        <ArrangementDialog
          project={project}
          initialIds={arrangement.ids}
          initial={arrangement.draft}
          busy={busy}
          error={error}
          onCancel={() => setArrangement(null)}
          onApply={applyPlacements}
        />
      )}
      <StageObjectDialogs
        project={project}
        busy={busy}
        error={error}
        actions={objects}
      />
    </WorkspaceSurface>
  );
});
