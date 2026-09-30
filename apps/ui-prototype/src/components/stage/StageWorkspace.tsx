import { StageMultiInspector } from "./StageMultiInspector";
import { DockPane } from "../layout/DockPane";
import type { ReactNode } from "react";
import { WorkspaceSurface } from "../workbench/WorkspaceSurface";
import { RigCreateDialog } from "./RigCreateDialog";
import { RigAttachmentDialog } from "./RigAttachmentDialog";
import type { RigShape } from "../../stage-types";
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
import {
  bounds,
  decimal,
  selectedStage,
  stageCommand,
} from "../../stage-tools";
import { uniqueName } from "../../editor-tools";
import { validateEditorForm } from "../workbench/form-validation";
import { StageCanvas } from "./StageCanvas";
import { StageCreateDialog, type Creation } from "./StageCreateDialog";
import { StageLibraryPanel } from "./StageLibraryPanel";
import {
  ALL_VISIBLE,
  visibleStage,
  revealStageTarget,
  type PlanVisibility,
} from "./stage-display";
import { StageInspector } from "./StageInspector";
import { DeleteDialog } from "../workbench/DeleteDialog";
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
  const [rigCreation, setRigCreation] = useState<{
    shape: RigShape;
    name: string;
  } | null>(null);
  const [hanging, setHanging] = useState<{ ids: string[]; rig: string } | null>(
    null,
  );
  const [creation, setCreation] = useState<Creation | null>(null);
  const [focusRequest, setFocusRequest] = useState(0);
  const [query, setQuery] = useState("");
  const [fixtureId, setFixtureId] = useState("");
  const [localError, setLocalError] = useState("");
  const [deleteTarget, setDeleteTarget] = useState<StageObject | null>(null);
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
  async function arrange(selected = false) {
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
  async function createRig() {
    if (!(await beforeChange())) return;
    const b = selectedSpace
      ? bounds(
          selectedSpace.outlineMeters.map((p) => [Number(p[0]), Number(p[1])]),
        )
      : null;
    setRigCreation({
      name: uniqueName(
        "桁架",
        project.stage.constructions.map((c) => c.name),
      ),
      shape: {
        kind: "rig",
        rigKind: "truss",
        spaceId: selectedSpace?.id ?? null,
        positionMeters: {
          x: decimal(b ? (b.minX + b.maxX) / 2 : 0),
          y: decimal(b ? (b.minY + b.maxY) / 2 : 0),
          z: decimal(
            Number(selectedSpace?.floorElevationMeters ?? 0) +
              Number(selectedSpace?.clearHeightMeters ?? 5) -
              0.3,
          ),
        },
        yawDegrees: "0",
        lengthMeters: "6",
        widthMeters: "0.3",
        heightMeters: "0.3",
      },
    });
  }
  async function hang() {
    if (!(await beforeChange())) return;
    const rig =
      object?.kind === "construction" && object.value.shape.kind === "rig"
        ? object.value.id
        : (project.stage.attachments.find((a) => liveIds.includes(a.fixtureId))
            ?.constructionId ??
          project.stage.constructions.find((c) => c.shape.kind === "rig")?.id ??
          "");
    setHanging({
      rig,
      ids: liveIds.length
        ? liveIds
        : project.stage.attachments
            .filter((a) => a.constructionId === rig)
            .map((a) => a.fixtureId),
    });
  }
  async function detach(ids: string[]) {
    if (!(await beforeChange())) return;
    await edit({
      op: "attachFixtures",
      constructionId: null,
      fixtureIds: ids,
      layout: null,
    });
  }
  async function create(kind: "space" | "platform") {
    if (!(await beforeChange())) return;
    const max = project.stage.spaces.flatMap((s) =>
      s.outlineMeters.map((p) => Number(p[0])),
    );
    const b = selectedSpace
      ? bounds(
          selectedSpace.outlineMeters.map((p) => [Number(p[0]), Number(p[1])]),
        )
      : null;
    setCreation({
      kind,
      name: uniqueName(
        kind === "space" ? "空间" : "舞台",
        (kind === "space"
          ? project.stage.spaces
          : project.stage.constructions
        ).map((v) => v.name),
      ),
      x:
        kind === "space"
          ? max.length
            ? Math.max(...max) + 2
            : 0
          : (b?.minX ?? 0),
      y: kind === "space" ? 0 : (b?.minY ?? 0),
      elevation:
        kind === "space" ? "0" : (selectedSpace?.floorElevationMeters ?? "0"),
      spaceId: selectedSpace?.id ?? null,
    });
  }
  async function createObject(command: StageEdit) {
    const next = await edit(command);
    if (!next) return;
    const target: StageSelection =
      command.op === "putSpace"
        ? { kind: "space", id: next.stage.spaces.at(-1)!.id }
        : { kind: "construction", id: next.stage.constructions.at(-1)!.id };
    revealInPlan(target, next);
    setSelection(target);
    setCreation(null);
    setQuery("");
    cancel();
    setFocusRequest((v) => v + 1);
  }
  async function placeFixture() {
    if (!chosenFixture) return;
    const room = selectedSpace;
    const b = room
      ? bounds(room.outlineMeters.map((p) => [Number(p[0]), Number(p[1])]))
      : null;
    const next = await edit({
      op: "putPlacement",
      placement: {
        fixtureId: chosenFixture.id,
        spaceId: room?.id ?? null,
        positionMeters: {
          x: decimal(b ? (b.minX + b.maxX) / 2 : 0),
          y: decimal(b ? (b.minY + b.maxY) / 2 : 0),
          z: decimal(
            Number(room?.floorElevationMeters ?? 0) +
              Number(room?.clearHeightMeters ?? 4) -
              0.5,
          ),
        },
        rotationDegreesXYZ: { x: "0", y: "0", z: "0" },
      },
    });
    if (next) {
      revealInPlan({ kind: "placement", id: chosenFixture.id }, next);
      setSelection({ kind: "placement", id: chosenFixture.id });
      setSelectedIds([chosenFixture.id]);
      cancel();
    }
  }
  async function enclose() {
    if (object?.kind !== "space") return;
    const next = await edit({
      op: "putConstruction",
      id: null,
      name: `${object.value.name}围护`,
      shape: {
        kind: "enclosure",
        spaceId: object.value.id,
        wallThicknessMeters: "0.2",
        floorThicknessMeters: "0.1",
        ceilingThicknessMeters: null,
      },
    });
    if (next) {
      const target: StageSelection = {
        kind: "construction",
        id: next.stage.constructions.at(-1)!.id,
      };
      revealInPlan(target, next);
      setSelection(target);
      cancel();
    }
  }
  async function duplicate() {
    if (!object || object.kind === "placement") return;
    const next = await edit({
      op: object.kind === "space" ? "duplicateSpace" : "duplicateConstruction",
      id: object.value.id,
      name: uniqueName(
        `${object.value.name} 副本`,
        [...project.stage.spaces, ...project.stage.constructions].map(
          (s) => s.name,
        ),
      ),
    });
    if (next) {
      const target: StageSelection = {
        kind: object.kind,
        id:
          object.kind === "space"
            ? next.stage.spaces.at(-1)!.id
            : next.stage.constructions.at(-1)!.id,
      };
      revealInPlan(target, next);
      setSelection(target);
      setQuery("");
      cancel();
    }
  }
  async function remove() {
    const target = deleteTarget;
    if (!target) return;
    const next = await edit(
      target.kind === "space"
        ? { op: "removeSpace", id: target.value.id, detachMembers: true }
        : target.kind === "construction"
          ? {
              op: "removeConstruction",
              id: target.value.id,
              detachFixtures: target.value.shape.kind === "rig",
            }
          : { op: "removePlacement", fixtureId: target.value.fixtureId },
    );
    if (next) {
      setDeleteTarget(null);
      setSelection(null);
      setSelectedIds([]);
      cancel();
    }
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
          onCreate={(kind) => void create(kind)}
          onCreateRig={() => void createRig()}
          onArrange={() => void arrange(false)}
          onPlace={() => void placeFixture()}
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
            onHang={() => void hang()}
            onDetach={() => void detach(liveIds)}
            onClear={() => void choosePlacements([])}
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
            onCancel={cancel}
            onDuplicate={() => void duplicate()}
            onDelete={() => {
              void beforeChange().then((ok) => {
                if (ok) setDeleteTarget(object);
              });
            }}
            onEnclose={() => void enclose()}
            onHang={() => void hang()}
            onSelectMounted={(id) =>
              void choosePlacements(
                project.stage.attachments
                  .filter((a) => a.constructionId === id)
                  .map((a) => a.fixtureId),
              )
            }
            onDetach={(ids) => void detach(ids)}
          />
        )}
      </DockPane>
      {rigCreation && (
        <RigCreateDialog
          project={project}
          initial={rigCreation.shape}
          name={rigCreation.name}
          busy={busy}
          error={error}
          onCancel={() => setRigCreation(null)}
          onApply={async (command) => {
            const next = await edit(command);
            if (!next) return false;
            const target: StageSelection = {
              kind: "construction",
              id: next.stage.constructions.at(-1)!.id,
            };
            revealInPlan(target, next);
            setSelection(target);
            setSelectedIds([]);
            setQuery("");
            setFocusRequest((v) => v + 1);
            cancel();
            return true;
          }}
        />
      )}
      {hanging && (
        <RigAttachmentDialog
          project={project}
          initialIds={hanging.ids}
          initialRig={hanging.rig}
          busy={busy}
          error={error}
          onCancel={() => setHanging(null)}
          onApply={async (command) => {
            const next = await edit(command);
            if (!next) return false;
            if (command.op === "attachFixtures") {
              revealPlacements(command.fixtureIds, next);
              setSelectedIds(command.fixtureIds);
              setSelection({
                kind: "placement",
                id: command.fixtureIds.at(-1)!,
              });
            }
            setQuery("");
            setFocusRequest((v) => v + 1);
            cancel();
            return true;
          }}
        />
      )}
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
      {creation && (
        <StageCreateDialog
          initial={creation}
          busy={busy}
          error={error}
          onCreate={(command) => void createObject(command)}
          onCancel={() => setCreation(null)}
        />
      )}
      {deleteTarget && (
        <DeleteDialog
          name={
            deleteTarget.kind === "placement"
              ? "此灯位"
              : deleteTarget.value.name
          }
          busy={busy}
          error={error}
          description={
            deleteTarget.kind === "space"
              ? "同时移除该空间的围护。灯具和舞台保留原位置并解除空间归属，灯光编排保留。"
              : deleteTarget.kind === "placement"
                ? "灯具配适与编排保留，只移除安装位置。"
                : deleteTarget.kind === "construction" &&
                    deleteTarget.value.shape.kind === "rig"
                  ? "保留全部灯具和灯位，解除挂接后删除支撑体；一次撤销可恢复。"
                  : "此操作可撤销恢复。"
          }
          onCancel={() => setDeleteTarget(null)}
          onDelete={() => void remove()}
        />
      )}
    </WorkspaceSurface>
  );
});
