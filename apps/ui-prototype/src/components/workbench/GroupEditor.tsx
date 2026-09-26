import { useState } from "react";
import type { FixtureView } from "../../application-host";
import type { GroupView, LibraryEdit } from "../../library-types";
import { moveMember } from "../../library-tools";
import { LibraryDialog } from "./LibraryDialog";
export function GroupEditor({
  group,
  fixtures,
  selected,
  name: initialName,
  busy,
  error,
  onCancel,
  onEdit,
}: {
  group?: GroupView;
  fixtures: FixtureView[];
  selected: string[];
  name: string;
  busy: boolean;
  error: string;
  onCancel(): void;
  onEdit(command: LibraryEdit): Promise<boolean>;
}) {
  const [name, setName] = useState(group?.name ?? initialName);
  const [ids, setIds] = useState(group?.fixtureIds ?? selected);
  const [query, setQuery] = useState("");
  return (
    <LibraryDialog
      title={group ? "编辑灯组" : "记录灯组"}
      busy={busy}
      error={error}
      onCancel={onCancel}
      onSubmit={() => {
        if (!name.trim()) throw new Error("请填写灯组名称");
        if (!ids.length) throw new Error("灯组至少需要一台灯具");
        return onEdit({
          kind: "saveGroup",
          id: group?.id ?? null,
          name: name.trim(),
          fixtureIds: ids,
        });
      }}
    >
      <label>
        灯组名称
        <input
          autoFocus
          aria-label="灯组名称"
          required
          maxLength={256}
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </label>
      <div className="wb-resource-tools">
        <strong>选灯顺序 · {ids.length} 台</strong>
        <button
          type="button"
          disabled={!ids.length}
          onClick={() => setIds([...ids].reverse())}
        >
          反转顺序
        </button>
        <button
          type="button"
          disabled={!selected.length}
          onClick={() => setIds([...selected])}
        >
          使用当前选择
        </button>
      </div>
      <ol className="wb-group-order">
        {ids.map((id, index) => (
          <li key={id}>
            <span>{index + 1}</span>
            <strong>
              {fixtures.find((f) => f.id === id)?.name ?? "灯具已移除"}
            </strong>
            <button
              type="button"
              aria-label={`上移第 ${index + 1} 台灯具`}
              disabled={!index}
              onClick={() => setIds(moveMember(ids, index, -1))}
            >
              ↑
            </button>
            <button
              type="button"
              aria-label={`下移第 ${index + 1} 台灯具`}
              disabled={index === ids.length - 1}
              onClick={() => setIds(moveMember(ids, index, 1))}
            >
              ↓
            </button>
            <button
              type="button"
              aria-label={`移出${fixtures.find((f) => f.id === id)?.name}`}
              onClick={() => setIds(ids.filter((v) => v !== id))}
            >
              移出
            </button>
          </li>
        ))}
      </ol>
      <input
        type="search"
        aria-label="搜索可加入灯组的灯具"
        placeholder="搜索可加入的灯具"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <div className="wb-group-available">
        {fixtures
          .filter(
            (f) =>
              !ids.includes(f.id) &&
              f.name.toLowerCase().includes(query.trim().toLowerCase()),
          )
          .map((f) => (
            <button
              type="button"
              key={f.id}
              onClick={() => setIds([...ids, f.id])}
            >
              ＋ {f.name}
            </button>
          ))}
      </div>
    </LibraryDialog>
  );
}
