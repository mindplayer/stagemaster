import { type ReactNode, forwardRef, useImperativeHandle, useRef, useState } from "react";
import {
  PlusIcon,
  HouseLineIcon,
  LightbulbIcon,
  CubeIcon,
  MagnifyingGlassIcon,
} from "@phosphor-icons/react";
import type {
  EditCommand,
  EditOperation,
  ProjectView,
} from "../../application-host";
import type { StageEdit, StageObject, StageSelection } from "../../stage-types";
import {
  bounds,
  decimal,
  rectangle,
  selectedStage,
  stageCommand,
} from "../../stage-tools";
import { uniqueName } from "../../editor-tools";
import { validateEditorForm } from "../workbench/form-validation";
import { StageCanvas } from "./StageCanvas";
import { StageInspector } from "./StageInspector";
import { DeleteDialog } from "../workbench/DeleteDialog";
import "./stage.css";
export interface StageHandle {
  collect(): EditOperation[];
  accept(): void;
}
export const StageWorkspace = forwardRef<
  StageHandle,
  {
    project: ProjectView;
    previs: (selection: { selectedId: string; onSelect(id: string): Promise<boolean> }) => ReactNode;
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
  const [draft, setDraft] = useState<StageObject | null>(null),
    draftRef = useRef<StageObject | null>(null);
  const moving = useRef(false);
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
  useImperativeHandle(ref, () => ({ collect, accept: cancel }));
  async function choose(target: StageSelection) {
    if (await beforeChange()) {
      setSelection(target);
      cancel();
    }
  }
  function edit(command: StageEdit) {
    return onEdit({ op: "stage", command });
  }
  async function addSpace() {
    const max = project.stage.spaces.flatMap((s) =>
      s.outlineMeters.map((p) => Number(p[0])),
    );
    const x = max.length ? Math.max(...max) + 2 : 0;
    const next = await edit({
      op: "putSpace",
      id: null,
      name: uniqueName(
        "空间",
        project.stage.spaces.map((s) => s.name),
      ),
      outlineMeters: rectangle(x, 0, 8, 6),
      floorElevationMeters: "0",
      clearHeightMeters: "5",
    });
    if (next) {
      setSelection({ kind: "space", id: next.stage.spaces.at(-1)!.id });
      setQuery("");
      cancel();
    }
  }
  async function addPlatform() {
    const room = selectedSpace;
    const b = room
      ? bounds(room.outlineMeters.map((p) => [Number(p[0]), Number(p[1])]))
      : null;
    const next = await edit({
      op: "putConstruction",
      id: null,
      name: uniqueName(
        "舞台",
        project.stage.constructions.map((s) => s.name),
      ),
      shape: {
        kind: "platform",
        spaceId: room?.id ?? null,
        outlineMeters: rectangle(b?.minX ?? 0, b?.minY ?? 0, 4, 2),
        baseElevationMeters: room?.floorElevationMeters ?? "0",
        heightMeters: "0.6",
      },
    });
    if (next) {
      setSelection({
        kind: "construction",
        id: next.stage.constructions.at(-1)!.id,
      });
      setQuery("");
      cancel();
    }
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
          ? { op: "removeConstruction", id: target.value.id }
          : { op: "removePlacement", fixtureId: target.value.fixtureId },
    );
    if (next) {
      setDeleteTarget(null);
      setSelection(null);
      cancel();
    }
  }
  const entries: { target: StageSelection; name: string; detail: string }[] = [
    ...project.stage.spaces.map((s) => ({
      target: { kind: "space" as const, id: s.id },
      name: s.name,
      detail: `${s.clearHeightMeters === null ? "开放空间" : `净高 ${s.clearHeightMeters} 米`}`,
    })),
    ...project.stage.constructions.map((c) => ({
      target: { kind: "construction" as const, id: c.id },
      name: c.name,
      detail: c.shape.kind === "enclosure" ? "墙体与地板" : "舞台构件",
    })),
    ...project.stage.placements.map((p) => ({
      target: { kind: "placement" as const, id: p.fixtureId },
      name: project.fixtures.find((f) => f.id === p.fixtureId)?.name ?? "灯具",
      detail: `高度 ${p.positionMeters.z} 米`,
    })),
  ];
  return (
    <div className="stage-workspace" hidden={!visible} aria-hidden={!visible}>
      <aside className="stage-browser">
        <header>
          <h2>场地</h2>
          <span>{project.stage.spaces.length} 个空间</span>
        </header>
        <div className="stage-create">
          <button disabled={busy} onClick={() => void addSpace()}>
            <HouseLineIcon />
            新建空间
          </button>
          <button disabled={busy} onClick={() => void addPlatform()}>
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
        <div className="stage-objects" aria-label="场地对象">
          {entries
            .filter((e) =>
              e.name
                .toLocaleLowerCase()
                .includes(query.trim().toLocaleLowerCase()),
            )
            .map((e) => (
              <button
                key={`${e.target.kind}:${e.target.id}`}
                className={
                  selection?.kind === e.target.kind &&
                  selection.id === e.target.id
                    ? "active"
                    : ""
                }
                aria-pressed={
                  selection?.kind === e.target.kind &&
                  selection.id === e.target.id
                }
                disabled={busy}
                onClick={() => void choose(e.target)}
              >
                {e.target.kind === "space" ? (
                  <HouseLineIcon />
                ) : e.target.kind === "placement" ? (
                  <LightbulbIcon />
                ) : (
                  <CubeIcon />
                )}
                <span>
                  <strong>{e.name}</strong>
                  <small>{e.detail}</small>
                </span>
              </button>
            ))}
          {!entries.length && <p className="wb-dim">尚未创建场地</p>}
        </div>
        <div className="stage-place">
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
          <button aria-pressed={view === "plan"} onClick={() => setView("plan")}>平面布置</button>
          <button aria-pressed={view === "three"} onClick={() => setView("three")}>三维预演</button>
        </div>
        {view === "three" ? previs({
          selectedId: selection?.kind === "placement" ? selection.id : "",
          onSelect: async (id) => {
            if (!(await beforeChange())) return false;
            if (id && !project.stage.placements.some(p => p.fixtureId === id)) return false;
            setSelection(id ? { kind: "placement", id } : null);
            cancel();
            return true;
          },
        }) : <StageCanvas
        project={project}
        selection={selection}
        busy={busy}
        pending={draft !== null}
        onSelect={(target) => void choose(target)}
        onMove={(value) => {
          void edit(stageCommand(value));
        }}
        onGesture={(value) => {
          moving.current = value;
          onPending(value || draftRef.current !== null);
        }}
      />}
      </div>
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
      />
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
                : "此操作可撤销恢复。"
          }
          onCancel={() => setDeleteTarget(null)}
          onDelete={() => void remove()}
        />
      )}
    </div>
  );
});
