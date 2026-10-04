import { useState } from "react";
import type { SceneView } from "../../application-host";
export function ManualSceneDestination({
  mode,
  sceneId,
  scenes,
  disabled,
  onMode,
  onScene,
}: {
  mode: "new" | "merge";
  sceneId: string;
  scenes: SceneView[];
  disabled: boolean;
  onMode(mode: "new" | "merge"): void;
  onScene(id: string): void;
}) {
  const [search, setSearch] = useState("");
  const matches = scenes.filter((s) =>
    s.name.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase()),
  );
  const chosen = scenes.find((s) => s.id === sceneId);
  return (
    <div className="manual-scene-destination">
      <label>
        保存方式
        <select
          aria-label="手动场景保存方式"
          value={mode}
          disabled={disabled}
          onChange={(e) => onMode(e.target.value as typeof mode)}
        >
          <option value="new">录入新场景</option>
          <option value="merge">合并到原场景</option>
        </select>
      </label>
      {mode === "merge" && (
        <>
          <label>
            搜索目标场景
            <input
              type="search"
              aria-label="搜索合并目标"
              value={search}
              disabled={disabled}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="输入场景名称"
            />
          </label>
          <label>
            目标场景
            <select
              aria-label="合并目标场景"
              value={sceneId}
              disabled={disabled}
              onChange={(e) => onScene(e.target.value)}
            >
              <option value="">请选择目标场景</option>
              {chosen && !matches.includes(chosen) && (
                <option value={chosen.id}>{chosen.name}（筛选外已选）</option>
              )}
              {matches.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.name}
                </option>
              ))}
            </select>
          </label>
          {!matches.length && <p>没有匹配的场景</p>}
        </>
      )}
    </div>
  );
}
