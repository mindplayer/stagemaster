import { useState } from "react";
import type { SceneView } from "../../application-host";
import type { SceneEffect } from "../../effect-types";
import { LibraryDialog } from "./LibraryDialog";
export function EffectReuseDialog({
  scenes,
  selected,
  onChoose,
  onCancel,
}: {
  scenes: SceneView[];
  selected: string[];
  onChoose(effect: SceneEffect, fixtures?: string[]): void;
  onCancel(): void;
}) {
  const [query, setQuery] = useState("");
  const [source, setSource] = useState("");
  const [replace, setReplace] = useState(selected.length > 0);
  const entries = scenes.flatMap((scene) =>
    scene.effects.map((effect) => ({ scene, effect })),
  );
  const matches = entries.filter(({ scene, effect }) =>
    (scene.name + " " + effect.name)
      .toLowerCase()
      .includes(query.trim().toLowerCase()),
  );
  return (
    <LibraryDialog
      title="复用已有效果"
      busy={false}
      error=""
      submit="继续编辑"
      onCancel={onCancel}
      onSubmit={async () => {
        const found = entries.find((e) => e.effect.id === source);
        if (!found) throw new Error("请选择要复用的效果");
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
        onChange={(e) => setQuery(e.target.value)}
      />
      <div className="effect-reuse-list" role="group" aria-label="已有效果">
        {matches.map(({ scene, effect }) => (
          <button
            type="button"
            key={effect.id}
            aria-pressed={source === effect.id}
            onClick={() => setSource(effect.id)}
          >
            <strong>{effect.name}</strong>
            <small>
              {scene.name} · {effect.fixtureIds.length} 台
            </small>
          </button>
        ))}
      </div>
      {matches.length === 0 && <p className="wb-dim">没有符合条件的效果</p>}
      <label className="effect-reuse-target">
        <input
          type="checkbox"
          disabled={!selected.length}
          checked={replace}
          onChange={(e) => setReplace(e.target.checked)}
        />
        改用当前所选的 {selected.length} 台灯具
      </label>
      <p className="wb-dim">
        复制后独立编辑，默认停用；可在下一步调整灯具、参数并启用。
      </p>
    </LibraryDialog>
  );
}
