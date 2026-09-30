import type { ProjectView } from "../../application-host";
import { displayMeters } from "./stage-display";
export function StageMultiInspector({
  project,
  ids,
  busy,
  error,
  onArrange,
  onHang,
  onDetach,
  onClear,
}: {
  project: ProjectView;
  ids: string[];
  busy: boolean;
  error: string;
  onArrange(): void;
  onHang(): void;
  onDetach(): void;
  onClear(): void;
}) {
  const selectedPlacements = project.stage.placements.filter((p) =>
    ids.includes(p.fixtureId),
  );
  return (
    <aside className="stage-inspector stage-multi-inspector">
      <header>
        <h2>已选 {ids.length} 台灯具</h2>
      </header>
      <button
        className="wb-primary"
        disabled={busy}
        onClick={() => onArrange()}
      >
        排列与精确调整
      </button>
      <div className="rig-member-actions">
        <button
          disabled={
            busy ||
            !project.stage.constructions.some((c) => c.shape.kind === "rig")
          }
          onClick={() => onHang()}
        >
          挂接到支撑体
        </button>
        <button
          disabled={
            busy ||
            !project.stage.attachments.some((a) => ids.includes(a.fixtureId))
          }
          onClick={() => onDetach()}
        >
          解除挂接
        </button>
      </div>
      <dl>
        <dt>高度范围</dt>
        <dd>
          {displayMeters(
            Math.min(
              ...selectedPlacements.map((p) => Number(p.positionMeters.z)),
            ),
          )}{" "}
          –{" "}
          {displayMeters(
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
        {ids.map((id) => (
          <li key={id}>{project.fixtures.find((f) => f.id === id)?.name}</li>
        ))}
      </ol>
      <button disabled={busy} onClick={() => onClear()}>
        清空选择
      </button>
      {error && (
        <p className="wb-library-error" role="alert">
          {error}
        </p>
      )}
    </aside>
  );
}
