import { useRef, useState } from "react";
import type { FixtureView, SceneView } from "../../application-host";
import type { SceneEffect } from "../../effect-types";
import { effectTargetIssues } from "../../effect-targets";
import { searchResources } from "../resources/resource-search";
import { EffectTargetReport } from "./EffectTargetReport";
import { LibraryDialog } from "./LibraryDialog";
import "./effect-targets.css";

const PAGE_SIZE = 50;
export function EffectReuseDialog({
  scenes,
  fixtures,
  selected,
  onChoose,
  onCancel,
}: {
  scenes: SceneView[];
  fixtures: FixtureView[];
  selected: string[];
  onChoose(effect: SceneEffect, fixtures?: string[]): void;
  onCancel(): void;
}) {
  const [query, setQuery] = useState("");
  const [source, setSource] = useState("");
  const [replace, setReplace] = useState(selected.length > 0);
  const [page, setPage] = useState(0);
  const results = useRef<HTMLDivElement>(null);
  const entries = scenes.flatMap((scene) =>
    scene.effects.map((effect) => ({
      id: JSON.stringify([scene.id, effect.id]),
      label: effect.name,
      detail: `${scene.name} · ${effect.fixtureIds.length} 台 · ${effect.periodMs / 1000} 秒／轮`,
      scene,
      effect,
    })),
  );
  const matches = searchResources(entries, query);
  const lastPage = Math.max(0, Math.ceil(matches.length / PAGE_SIZE) - 1);
  const currentPage = Math.min(page, lastPage);
  const shown = matches.slice(
    currentPage * PAGE_SIZE,
    (currentPage + 1) * PAGE_SIZE,
  );
  const found = entries.find((e) => e.id === source);
  const targets = replace ? selected : (found?.effect.fixtureIds ?? []);
  const issues = found
    ? effectTargetIssues(
        targets,
        fixtures,
        found.effect.channels,
        found.effect.waveform === "position",
      )
    : [];
  function focusResult(index: number) {
    const buttons =
      results.current?.querySelectorAll<HTMLButtonElement>("button");
    if (!buttons?.length) return;
    const button = buttons[Math.max(0, Math.min(index, buttons.length - 1))];
    button.focus();
    button.scrollIntoView({ block: "nearest" });
  }
  return (
    <LibraryDialog
      title="复用已有效果"
      busy={false}
      error=""
      submit="继续编辑"
      submitDisabled={!found || issues.length > 0}
      onCancel={onCancel}
      onSubmit={async () => {
        if (!found) throw new Error("请选择要复用的效果");
        if (issues.length) throw new Error("请先调整目标灯具");
        onChoose(found.effect, replace ? selected : undefined);
        return true;
      }}
    >
      <input
        autoFocus
        type="search"
        aria-label="搜索已有效果"
        placeholder="搜索效果或场景"
        value={query}
        onChange={(e) => {
          setQuery(e.target.value);
          setPage(0);
        }}
        onKeyDown={(e) => {
          if (e.nativeEvent.isComposing) return;
          if (e.key === "ArrowDown" || e.key === "ArrowUp") {
            e.preventDefault();
            focusResult(e.key === "ArrowDown" ? 0 : shown.length - 1);
          }
        }}
      />
      <div
        ref={results}
        className="effect-reuse-list"
        role="group"
        aria-label="已有效果"
      >
        {shown.map((entry, index) => (
          <button
            type="button"
            key={entry.id}
            aria-pressed={source === entry.id}
            tabIndex={index === 0 ? 0 : -1}
            onClick={() => setSource(entry.id)}
            onKeyDown={(e) => {
              const next =
                e.key === "ArrowDown"
                  ? index + 1
                  : e.key === "ArrowUp"
                    ? index - 1
                    : e.key === "Home"
                      ? 0
                      : e.key === "End"
                        ? shown.length - 1
                        : null;
              if (next !== null) {
                e.preventDefault();
                focusResult(next);
              }
            }}
          >
            <strong>{entry.label}</strong>
            <small>{entry.detail}</small>
          </button>
        ))}
      </div>
      {!shown.length && <p role="status">没有符合条件的效果</p>}
      <div className="effect-reuse-pages" aria-label="效果分页">
        <span>
          {matches.length} 项{query && ` / 共 ${entries.length} 项`}
        </span>
        {lastPage > 0 && (
          <>
            <button
              type="button"
              disabled={!currentPage}
              onClick={() => setPage(currentPage - 1)}
            >
              上一页
            </button>
            <span>
              {currentPage + 1} / {lastPage + 1}
            </span>
            <button
              type="button"
              disabled={currentPage === lastPage}
              onClick={() => setPage(currentPage + 1)}
            >
              下一页
            </button>
          </>
        )}
      </div>
      {found && (
        <div className="effect-reuse-source" role="status">
          <strong>已选：{found.effect.name}</strong>
          <span>来源：{found.scene.name}</span>
          {!shown.some((e) => e.id === source) && (
            <small>所选效果不在当前结果页，选择仍保留。</small>
          )}
          {query && (
            <button
              type="button"
              onClick={() => {
                setQuery("");
                setPage(0);
              }}
            >
              清除效果筛选
            </button>
          )}
        </div>
      )}
      <label className="effect-reuse-target">
        <input
          type="checkbox"
          disabled={!selected.length}
          checked={replace}
          onChange={(e) => setReplace(e.target.checked)}
        />
        改用当前所选的 {selected.length} 台灯具
      </label>
      {found && (
        <p>
          {replace ? "当前选灯" : "来源效果灯具"}：{targets.length} 台 ·
          保留其顺序{!issues.length && " · 所需属性均支持"}
        </p>
      )}
      <EffectTargetReport issues={issues} />
      <p className="wb-dim">独立副本默认停用，继续后可调整参数并启用。</p>
    </LibraryDialog>
  );
}
