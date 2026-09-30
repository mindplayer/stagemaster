import { ResourcePicker } from "../resources/ResourcePicker";
import "./scene-selection-bar.css";
import type { ProjectView } from "../../application-host";
import { recallGroup, type RecallMode } from "../../library-tools";

export function SceneSelectionBar({
  project,
  selected,
  busy,
  recall,
  onRecall,
  onSelect,
}: {
  project: ProjectView;
  selected: string[];
  busy: boolean;
  recall: RecallMode;
  onRecall(value: RecallMode): void;
  onSelect(ids: string[]): Promise<boolean>;
}) {
  const group = project.groups.find(
    (g) =>
      g.fixtureIds.length === selected.length &&
      g.fixtureIds.every((id, index) => id === selected[index]),
  );
  return (
    <div className="scene-selection-bar" role="group" aria-label="当前选灯">
      <strong
        title={selected
          .map((id) => project.fixtures.find((f) => f.id === id)?.name)
          .join(" → ")}
      >
        已选 {selected.length} 台
      </strong>
      <ResourcePicker
        label="快捷选择灯组"
        placeholder="选择灯组…"
        value={group?.id}
        disabled={busy}
        options={project.groups.map((g) => ({
          id: g.id,
          label: g.name,
          detail: `${g.fixtureIds.length} 台灯具`,
        }))}
        onSelect={(id) => {
          const target = project.groups.find((g) => g.id === id);
          if (target)
            void onSelect(recallGroup(selected, target.fixtureIds, recall));
        }}
      />
      <select
        aria-label="快捷灯组召回方式"
        value={recall}
        disabled={busy}
        onChange={(e) => onRecall(e.target.value as RecallMode)}
      >
        <option value="replace">替换选择</option>
        <option value="add">追加选择</option>
        <option value="subtract">扣除选择</option>
      </select>
      <div className="scene-selection-actions">
        <button
          disabled={busy || !selected.length}
          onClick={() => void onSelect(selected.filter((_, i) => i % 2 === 0))}
        >
          取奇数位
        </button>
        <button
          disabled={busy || selected.length < 2}
          onClick={() => void onSelect(selected.filter((_, i) => i % 2 === 1))}
        >
          取偶数位
        </button>
        <button
          disabled={busy || selected.length < 2}
          onClick={() => void onSelect([...selected].reverse())}
        >
          反转顺序
        </button>
        <button
          disabled={busy || !project.fixtures.length}
          onClick={() =>
            void onSelect(
              project.fixtures
                .filter((f) => !selected.includes(f.id))
                .map((f) => f.id),
            )
          }
        >
          反选全场
        </button>
        <button
          disabled={busy || !selected.length}
          onClick={() => void onSelect([])}
        >
          清空选择
        </button>
      </div>
    </div>
  );
}
