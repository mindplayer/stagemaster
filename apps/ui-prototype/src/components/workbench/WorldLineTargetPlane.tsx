import { useState } from "react";
import type { ProjectView } from "../../application-host";
import type { WorldLinePath } from "../../effect-types";
import { AimTargetPlane } from "../fixtures/AimTargetPlane";

export function WorldLineTargetPlane({
  project,
  fixtureIds,
  path,
  busy,
  onChange,
}: {
  project: ProjectView;
  fixtureIds: string[];
  path: WorldLinePath;
  busy: boolean;
  onChange(path: WorldLinePath): void;
}) {
  const [endpoint, setEndpoint] = useState<"fromMeters" | "toMeters">(
    "fromMeters",
  );
  const [mounted, setMounted] = useState(false);
  const isStart = endpoint === "fromMeters";
  return (
    <details
      className="world-line-plane"
      onToggle={(event) => {
        // WebKit can measure a closed details subtree. Fit only after its
        // first reveal; retain the mounted camera across subsequent folds.
        if (event.currentTarget.open) setMounted(true);
      }}
    >
      <summary>在场地上选起终点</summary>
      <div className="aim-target-tools" aria-label="轨迹端点选择">
        <button
          type="button"
          aria-pressed={isStart}
          disabled={busy}
          onClick={() => setEndpoint("fromMeters")}
        >
          选择起点
        </button>
        <button
          type="button"
          aria-pressed={!isStart}
          disabled={busy}
          onClick={() => setEndpoint("toMeters")}
        >
          选择终点
        </button>
      </div>
      {mounted && (
        <AimTargetPlane
          project={project}
          fixtureIds={fixtureIds}
          {...path[endpoint]}
          targetLabel={isStart ? "起点" : "终点"}
          secondaryPoint={{
            ...path[isStart ? "toMeters" : "fromMeters"],
            label: isStart ? "终点" : "起点",
          }}
          scopeKey={endpoint}
          disabled={busy}
          onChange={(point) =>
            onChange({
              ...path,
              [endpoint]: { ...path[endpoint], ...point },
            })
          }
        />
      )}
      <p className="position-effect-note">
        仅改变所选端点的左右、前后位置，高度保持。虚线为轨迹俯视投影。
      </p>
    </details>
  );
}
