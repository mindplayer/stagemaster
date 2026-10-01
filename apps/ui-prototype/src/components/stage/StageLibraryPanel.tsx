import {
  PlusIcon,
  HouseLineIcon,
  CubeIcon,
  MagnifyingGlassIcon,
} from "@phosphor-icons/react";
import type { ProjectView } from "../../application-host";
import type { StageSelection } from "../../stage-types";
import type { PlanVisibility } from "./stage-display";
import { StageOutliner } from "./StageOutliner";
import { StagePlanLayers } from "./StagePlanLayers";
export function StageLibraryPanel({
  project,
  busy,
  query,
  onQuery,
  selection,
  selectedIds,
  visibility,
  onVisibility,
  onSelect,
  onCreate,
  onCreateRig,
  onCreateSeating,
  onArrange,
  onPlace,
  fixtureId,
  onFixtureId,
}: {
  project: ProjectView;
  busy: boolean;
  query: string;
  onQuery(query: string): void;
  selection: StageSelection | null;
  selectedIds: string[];
  visibility: PlanVisibility;
  onVisibility(value: PlanVisibility): void;
  onSelect(target: StageSelection, additive?: boolean): void;
  onCreate(kind: "space" | "platform"): void;
  onCreateRig(): void;
  onCreateSeating(): void;
  onArrange(): void;
  onPlace(): void;
  fixtureId: string;
  onFixtureId(id: string): void;
}) {
  const unplaced = project.fixtures.filter(
    (f) => !project.stage.placements.some((p) => p.fixtureId === f.id),
  );
  const chosenFixture = unplaced.find((f) => f.id === fixtureId) ?? unplaced[0];
  return (
    <aside className="stage-browser">
      <header>
        <h2>场地</h2>
        <span>{project.stage.spaces.length} 个空间</span>
      </header>
      <div className="stage-create">
        <button disabled={busy} onClick={() => onCreate("space")}>
          <HouseLineIcon />
          新建空间
        </button>
        <button disabled={busy} onClick={() => onCreate("platform")}>
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
          onChange={(e) => onQuery(e.target.value)}
        />
      </label>
      <div className="stage-create">
        <button disabled={busy} onClick={() => onCreateRig()}>
          桁架／灯杆
        </button>
        <button disabled={busy} onClick={onCreateSeating}>
          新建座区
        </button>
      </div>
      <StagePlanLayers
        value={visibility}
        disabled={busy}
        onChange={(next) => onVisibility(next)}
      />
      <StageOutliner
        project={project}
        selection={selection}
        selectedIds={selectedIds}
        query={query}
        busy={busy}
        visibility={visibility}
        onVisibility={(next) => onVisibility(next)}
        onSelect={(target, additive) => onSelect(target, additive)}
      />
      <div className="stage-place">
        <button
          disabled={busy || !project.fixtures.length}
          onClick={() => onArrange()}
        >
          批量布灯
        </button>
        <label>
          布置灯具
          <select
            aria-label="待布置灯具"
            value={chosenFixture?.id ?? ""}
            disabled={busy || !unplaced.length}
            onChange={(e) => onFixtureId(e.target.value)}
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
        <button disabled={busy || !chosenFixture} onClick={() => onPlace()}>
          <PlusIcon />
          放入场地
        </button>
      </div>
    </aside>
  );
}
