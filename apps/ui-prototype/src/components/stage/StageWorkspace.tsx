import { WorkspaceSurface } from "../workbench/WorkspaceSurface";
import { RigCreateDialog } from "./RigCreateDialog";
import { RigAttachmentDialog } from "./RigAttachmentDialog";
import type { RigShape } from "../../stage-types";
import {
  type ReactNode,
  forwardRef,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import {
  PlusIcon,
  HouseLineIcon,
  CubeIcon,
  MagnifyingGlassIcon,
} from "@phosphor-icons/react";
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
import { StageOutliner } from "./StageOutliner";
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
}
export const StageWorkspace = forwardRef<
  StageHandle,
  {
    project: ProjectView;
    previs: (selection: {
      selectedId: string;
      onSelect(id: string): Promise<boolean>;
    }) => ReactNode;
    visible: boolean;
    busy: boolean;
    error: string;
    beforeChange(): Promise<boolean>;
    onEdit(command: EditCommand): Promise<ProjectView | null>;
    onPending(value: boolean): void;
  }
>(function StageWorkspace(
  { project, previs, visible, busy, error, beforeChange, onEdit, onPending },
  ref,
) {
  const [view, setView] = useState<"plan" | "three">("plan");
  const [selection, setSelection] = useState<StageSelection | null>(null);
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
  }));
  async function choose(
    target: StageSelection,
    additive = false,
    preserve = false,
  ) {
    if (await beforeChange()) {
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
    setSelection(target);
    setCreation(null);
    setQuery("");
    cancel();
    setView("plan");
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
      setSelection({
        kind: "construction",
        id: next.stage.constructions.at(-1)!.id,
      });
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
      setSelection({
        kind: object.kind,
        id:
          object.kind === "space"
            ? next.stage.spaces.at(-1)!.id
            : next.stage.constructions.at(-1)!.id,
      });
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
      <aside className="stage-browser">
        <header>
          <h2>场地</h2>
          <span>{project.stage.spaces.length} 个空间</span>
        </header>
        <div className="stage-create">
          <button disabled={busy} onClick={() => void create("space")}>
            <HouseLineIcon />
            新建空间
          </button>
          <button disabled={busy} onClick={() => void create("platform")}>
            <CubeIcon />
            新建舞台
          </button>
        </div>
        <label className="stage-search">
          <MagnifyingGlassIcon />
          <input
            aria-label="搜索场地对象"
            placeholder="搜索空间、构件、灯具"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </label>
        <button disabled={busy} onClick={() => void createRig()}>
          新建桁架／灯杆
        </button>
        <StageOutliner
          project={project}
          selection={selection}
          selectedIds={liveIds}
          query={query}
          busy={busy}
          onSelect={(target, additive) => void choose(target, additive)}
        />
        <div className="stage-place">
          <button
            disabled={busy || !project.fixtures.length}
            onClick={() => void arrange(false)}
          >
            批量布灯
          </button>
          <label>
            布置灯具
            <select
              aria-label="待布置灯具"
              value={chosenFixture?.id ?? ""}
              disabled={busy || !unplaced.length}
              onChange={(e) => setFixtureId(e.target.value)}
            >
              {unplaced.length ? (
                unplaced.map((f) => (
                  <option key={f.id} value={f.id}>
                    {f.name}
                  </option>
                ))
              ) : (
                <option value="">没有未布置的灯具</option>
              )}
            </select>
          </label>
          <button
            disabled={busy || !chosenFixture}
            onClick={() => void placeFixture()}
          >
            <PlusIcon />
            放入场地
          </button>
        </div>
      </aside>
      <div className="stage-center">
        <div className="stage-view-tabs" aria-label="舞台视图">
          <button
            aria-pressed={view === "plan"}
            onClick={() => setView("plan")}
          >
            平面布置
          </button>
          <button
            aria-pressed={view === "three"}
            onClick={() => {
              void beforeChange().then((ok) => {
                if (ok) setView("three");
              });
            }}
          >
            三维预演
          </button>
        </div>
        {visible &&
          view === "three" &&
          previs({
            selectedId: selection?.kind === "placement" ? selection.id : "",
            onSelect: async (id) => {
              if (!(await beforeChange())) return false;
              if (
                id &&
                !project.stage.placements.some((p) => p.fixtureId === id)
              )
                return false;
              setSelection(id ? { kind: "placement", id } : null);
              setSelectedIds(id ? [id] : []);
              cancel();
              return true;
            },
          })}
        <div className="stage-plan-container" hidden={view !== "plan"}>
          <StageCanvas
            project={project}
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
      {selection?.kind === "placement" && liveIds.length > 1 ? (
        <aside className="stage-inspector stage-multi-inspector">
          <header>
            <h2>已选 {liveIds.length} 台灯具</h2>
          </header>
          {view === "three" && (
            <p className="wb-dim">
              三维拖动当前灯具：
              {project.fixtures.find((f) => f.id === selection.id)?.name}
            </p>
          )}
          <button
            className="wb-primary"
            disabled={busy}
            onClick={() => void arrange(true)}
          >
            排列与精确调整
          </button>
          <div className="rig-member-actions">
            <button
              disabled={
                busy ||
                !project.stage.constructions.some((c) => c.shape.kind === "rig")
              }
              onClick={() => void hang()}
            >
              挂接到支撑体
            </button>
            <button
              disabled={
                busy ||
                !project.stage.attachments.some((a) =>
                  liveIds.includes(a.fixtureId),
                )
              }
              onClick={() => void detach(liveIds)}
            >
              解除挂接
            </button>
          </div>
          <dl>
            <dt>高度范围</dt>
            <dd>
              {decimal(
                Math.min(
                  ...selectedPlacements.map((p) => Number(p.positionMeters.z)),
                ),
              )}{" "}
              –{" "}
              {decimal(
                Math.max(
                  ...selectedPlacements.map((p) => Number(p.positionMeters.z)),
                ),
              )}{" "}
              米
            </dd>
            <dt>所属空间</dt>
            <dd>
              {new Set(selectedPlacements.map((p) => p.spaceId)).size > 1
                ? "多个空间"
                : (project.stage.spaces.find(
                    (s) => s.id === selectedPlacements[0]?.spaceId,
                  )?.name ?? "未归属")}
            </dd>
          </dl>
          <ol>
            {liveIds.map((id) => (
              <li key={id}>
                {project.fixtures.find((f) => f.id === id)?.name}
              </li>
            ))}
          </ol>
          <button disabled={busy} onClick={() => void choosePlacements([])}>
            清空选择
          </button>
          {(localError || error) && (
            <p className="wb-library-error" role="alert">
              {localError || error}
            </p>
          )}
        </aside>
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
            setSelection({
              kind: "construction",
              id: next.stage.constructions.at(-1)!.id,
            });
            setSelectedIds([]);
            setQuery("");
            setView("plan");
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
