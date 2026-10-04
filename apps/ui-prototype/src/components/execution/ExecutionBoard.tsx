import { useCallback, useState } from "react";
import type { LiveLevels } from "../../execution-level-gesture";
import type {
  ExecutionAction,
  ExecutionView,
  ExecutionStatus,
} from "../../execution-types";
import type { ExecutionMediaAction } from "../../execution-media-types";
import type { MediaRequestIdentity } from "../../media-seek-receipt";
import {
  allSources,
  boardRows,
  sourceKinds,
  sourcePinKey,
} from "../../execution-board";
import { ExecutionBoardToolbar } from "./ExecutionBoardToolbar";
import { SourceControls } from "./SourceControls";
import { ManualControls } from "./ManualControls";
import { MediaControls } from "./MediaControls";
import { useExecutionPins } from "./useExecutionPins";
import "./execution-board.css";

export function ExecutionBoard({
  runtime,
  disabled,
  observed,
  active,
  onAction,
  onMedia,
  recording,
  live,
}: {
  live?: LiveLevels;
  runtime: ExecutionView;
  disabled: boolean;
  observed: boolean;
  active: boolean;
  onAction(
    source: string,
    action: ExecutionAction,
  ): Promise<ExecutionStatus | undefined>;
  onMedia(action: ExecutionMediaAction): Promise<MediaRequestIdentity | null>;
  recording?: import("../../manual-capture-types").ManualRecordingContext;
}) {
  const [filter, setFilter] = useState({ ...allSources });
  const [drafts, setDrafts] = useState<ReadonlySet<string>>(new Set());
  const onDraftChange = useCallback((id: string, dirty: boolean) => {
    setDrafts((old) => {
      if (old.has(id) === dirty) return old;
      const next = new Set(old);
      if (dirty) next.add(id);
      else next.delete(id);
      return next;
    });
  }, []);
  const available = runtime.catalog.sources.map(sourcePinKey);
  const pins = useExecutionPins(runtime.catalog.projectId, available);
  const presentPins = pins.pins.filter((k) => available.includes(k));
  const rows = boardRows(runtime, filter, pins.pins, drafts);
  const counts: Record<string, number> = {};
  for (const row of rows)
    if (row.status) counts[row.status] = (counts[row.status] ?? 0) + 1;
  const shown = rows.filter((r) => r.shown).length;
  return (
    <section className="execution-board" aria-label="后台节目操作台">
      <ExecutionBoardToolbar
        filter={filter}
        onFilter={setFilter}
        counts={counts}
        shown={shown}
        total={rows.length}
        dirty={drafts.size}
        observed={observed}
        missingPins={pins.missing}
        onPrune={() => pins.edit({ kind: "prune" })}
        hiddenRunning={
          rows.filter((r) => !r.shown && r.status === "Running").length
        }
        hiddenPaused={
          rows.filter((r) => !r.shown && r.status === "Paused").length
        }
      />
      {pins.problem && <p role="alert">{pins.problem}</p>}
      {shown === 0 && (
        <div className="execution-board-empty" role="status">
          <p>
            {filter.scope === "drafts"
              ? "当前筛选下没有未应用输入"
              : "没有符合条件的节目"}
          </p>
          <button onClick={() => setFilter({ ...allSources })}>重置筛选</button>
        </div>
      )}
      <div className="execution-sources">
        {rows.map(({ source, shown, pinned }) => {
          const key = sourcePinKey(source);
          const enabled = active && shown;
          return (
            <div
              key={source.id}
              className="execution-board-slot"
              data-manual={source.selection.kind === "manual"}
              hidden={!shown}
            >
              <div className="execution-board-slot-tools">
                <span>{sourceKinds[source.selection.kind]}</span>
                {drafts.has(source.id) && (
                  <span className="execution-board-draft">未应用输入</span>
                )}
                <button
                  aria-label={`${source.name}${pinned ? "取消固定" : "固定为常用"}`}
                  aria-pressed={pinned}
                  onClick={() => pins.edit({ kind: "toggle", key })}
                >
                  {pinned ? "已固定" : "固定"}
                </button>
                {pinned && (
                  <>
                    <button
                      aria-label={`${source.name}常用前移`}
                      title="常用前移"
                      disabled={presentPins.indexOf(key) === 0}
                      onClick={() => pins.edit({ kind: "earlier", key })}
                    >
                      前移
                    </button>
                    <button
                      aria-label={`${source.name}常用后移`}
                      title="常用后移"
                      disabled={
                        presentPins.indexOf(key) === presentPins.length - 1
                      }
                      onClick={() => pins.edit({ kind: "later", key })}
                    >
                      后移
                    </button>
                  </>
                )}
              </div>
              {source.selection.kind === "audioTimeline" ? (
                <MediaControls
                  runtime={runtime}
                  disabled={disabled || !enabled}
                  sourceId={source.id}
                  onDraftChange={onDraftChange}
                  onAction={(a) =>
                    enabled ? onMedia(a) : Promise.resolve(null)
                  }
                />
              ) : source.selection.kind === "manual" ? (
                <ManualControls
                  live={live}
                  recording={recording}
                  source={source}
                  runtime={runtime}
                  disabled={disabled || !enabled}
                  active={enabled}
                  observed={observed}
                  onDraftChange={onDraftChange}
                  onAction={(a) =>
                    enabled
                      ? onAction(source.id, a)
                      : Promise.resolve(undefined)
                  }
                />
              ) : (
                <SourceControls
                  live={live}
                  source={source}
                  runtime={runtime}
                  disabled={disabled || !enabled}
                  active={enabled}
                  observed={observed}
                  onDraftChange={onDraftChange}
                  onAction={(a) => {
                    if (enabled) onAction(source.id, a);
                  }}
                />
              )}
            </div>
          );
        })}
      </div>
    </section>
  );
}
