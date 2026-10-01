import type { ReactNode } from "react";
import type { ProjectView } from "../../application-host";
import type { SequenceView } from "../../sequence-types";
import type { ExecutionPosition } from "./execution-position";
import { SequenceStepList } from "./SequenceStepList";
import { navigationTargets } from "./step-navigation";
import { useStepFollow } from "./useStepFollow";
import "./step-navigation.css";

export function SequenceStepBrowser({
  sequence,
  steps,
  selectedId,
  scenes,
  busy,
  visible,
  execution,
  position,
  query,
  setQuery,
  onSelect,
  children,
}: {
  sequence: SequenceView;
  steps: SequenceView["steps"];
  selectedId: string;
  scenes: ProjectView["scenes"];
  busy: boolean;
  visible: boolean;
  execution: boolean;
  position: ExecutionPosition;
  query: string;
  setQuery(value: string): void;
  onSelect(id: string): Promise<boolean>;
  children?: ReactNode;
}) {
  const targets = navigationTargets(
    position,
    sequence.id,
    sequence.steps.map((s) => s.id),
  );
  const ready = position.sequenceId === sequence.id && !position.stale;
  const follow = useStepFollow({
    scope: `${sequence.id}/${position.epoch}`,
    enabled: visible && execution && ready,
    current: targets.current,
    next: targets.next,
    query,
    clearQuery: () => setQuery(""),
  });
  const selected = sequence.steps.find((s) => s.id === selectedId);
  return (
    <>
      {execution ? (
        <div className="execution-search" aria-label="执行列表导航">
          <input
            aria-label="搜索步骤"
            placeholder="搜索步骤、幕场或台词"
            value={query}
            onChange={(event) => {
              follow.stop();
              setQuery(event.target.value);
            }}
          />
          <div className="execution-navigation-actions">
            <button
              disabled={!targets.current}
              onClick={() => follow.locate(targets.current)}
            >
              定位当前
            </button>
            <button
              disabled={!targets.next}
              onClick={() => follow.locate(targets.next)}
            >
              定位下一步
            </button>
            <button
              aria-pressed={follow.following}
              disabled={!targets.current && !targets.next}
              title="仅滚动列表；搜索或手动浏览会关闭跟随，不改变所选步骤"
              onClick={follow.toggle}
            >
              {follow.following ? "正在跟随" : "跟随执行"}
            </button>
          </div>
        </div>
      ) : (
        children
      )}
      <div
        className="wb-steps-region step-browser-viewport"
        ref={follow.viewport}
        aria-label="步骤浏览区"
        onScroll={follow.onScroll}
        onWheel={follow.stop}
        onPointerDown={follow.stop}
        onTouchStart={follow.stop}
        onKeyDown={(event) => {
          if (
            [
              "ArrowUp",
              "ArrowDown",
              "PageUp",
              "PageDown",
              "Home",
              "End",
            ].includes(event.key)
          )
            follow.stop();
        }}
      >
        {selected && !steps.some((s) => s.id === selected.id) && (
          <p className="wb-dim">
            当前选中“{selected.name}”未匹配筛选。
            <button onClick={() => setQuery("")}>清除筛选</button>
          </p>
        )}
        <SequenceStepList
          steps={steps}
          selectedId={selectedId}
          scenes={scenes}
          busy={busy}
          onSelect={(id) => {
            follow.stop();
            return onSelect(id);
          }}
          position={ready ? position : undefined}
        />
      </div>
    </>
  );
}
