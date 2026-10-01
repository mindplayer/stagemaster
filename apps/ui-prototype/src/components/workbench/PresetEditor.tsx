import { useState } from "react";
import type { FixtureView, SceneView } from "../../application-host";
import type {
  LibraryEdit,
  PresetUpdate,
  PresetView,
} from "../../library-types";
import {
  attributeName,
  matchingValues,
  scopeAttributes,
  sceneValueLabel,
} from "../../library-tools";
import { LibraryDialog } from "./LibraryDialog";
export function AttributeMask({
  available,
  selected,
  onChange,
}: {
  available: { key: string; label: string }[];
  selected: string[];
  onChange(ids: string[]): void;
}) {
  return (
    <div className="wb-attribute-mask" role="group" aria-label="属性范围">
      {available.map((a) => (
        <label key={a.key}>
          <input
            type="checkbox"
            checked={selected.includes(a.key)}
            onChange={(e) =>
              onChange(
                e.target.checked
                  ? [...selected, a.key]
                  : selected.filter((k) => k !== a.key),
              )
            }
          />
          {a.label}
        </label>
      ))}
      <button
        type="button"
        onClick={() => onChange(available.map((a) => a.key))}
      >
        全部属性
      </button>
      {available.some((a) => ["red", "green", "blue"].includes(a.key)) && (
        <button
          type="button"
          onClick={() =>
            onChange(
              available
                .filter((a) => ["red", "green", "blue"].includes(a.key))
                .map((a) => a.key),
            )
          }
        >
          仅颜色
        </button>
      )}
    </div>
  );
}
export function PresetEditor({
  preset,
  scene,
  fixtures,
  name: initialName,
  busy,
  error,
  onCancel,
  onEdit,
}: {
  preset?: PresetView;
  scene: SceneView;
  fixtures: FixtureView[];
  name: string;
  busy: boolean;
  error: string;
  onCancel(): void;
  onEdit(command: LibraryEdit): Promise<boolean>;
}) {
  const available = scopeAttributes(fixtures);
  const [name, setName] = useState(initialName);
  const [attributes, setAttributes] = useState(() =>
    available
      .filter(
        (a) => !preset || preset.values.some((v) => v.attribute === a.key),
      )
      .map((a) => a.key),
  );
  const [mode, setMode] = useState<PresetUpdate>("existing");
  const ids = fixtures.map((f) => f.id);
  const captured = matchingValues(scene.values, ids, attributes);
  const applicable =
    preset && mode === "existing"
      ? captured.filter((v) =>
          preset.values.some(
            (p) => p.fixtureId === v.fixtureId && p.attribute === v.attribute,
          ),
        )
      : captured;
  return (
    <LibraryDialog
      title={preset ? `更新预设 · ${preset.name}` : "记录预设"}
      busy={busy}
      error={error}
      onCancel={onCancel}
      submit={preset ? "更新预设" : "记录预设"}
      onSubmit={() => {
        if (!name.trim()) throw new Error("请填写预设名称");
        if (!applicable.length)
          throw new Error("当前范围没有可记录的数值，请检查灯具和属性范围");
        const scope = { sceneId: scene.id, fixtureIds: ids, attributes };
        return onEdit(
          preset
            ? { kind: "updatePreset", id: preset.id, mode, ...scope }
            : { kind: "recordPreset", name: name.trim(), ...scope },
        );
      }}
    >
      {!preset && (
        <label>
          预设名称
          <input
            autoFocus
            aria-label="预设名称"
            required
            maxLength={256}
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </label>
      )}
      <p>
        来源：{scene.name} · 已选 {fixtures.length} 台灯具
      </p>
      <AttributeMask
        available={available}
        selected={attributes}
        onChange={setAttributes}
      />
      {preset && (
        <label>
          更新方式
          <select
            autoFocus
            aria-label="预设更新方式"
            value={mode}
            onChange={(e) => setMode(e.target.value as PresetUpdate)}
          >
            <option value="existing">仅更新已有内容</option>
            <option value="merge">合并新增内容</option>
            <option value="replace">完整替换</option>
          </select>
        </label>
      )}
      <p className="wb-dim">
        本次{preset ? "更新" : "记录"} {applicable.length} 项数值 ·
        释放与未记录项不写入
      </p>
      {preset && <PresetUsage preset={preset} />}
      <details>
        <summary>查看本次内容</summary>
        <ValueTable values={applicable} fixtures={fixtures} />
      </details>
    </LibraryDialog>
  );
}
export function PresetUsage({ preset }: { preset: PresetView }) {
  return (
    <div className="wb-preset-usage">
      <strong>引用范围</strong>
      <p>
        场景：{preset.usedByScenes.map((s) => s.name).join("、") || "尚未引用"}
      </p>
      <p>
        场景列表：
        {preset.usedBySequences.map((s) => s.name).join("、") || "尚未引用"}
      </p>
    </div>
  );
}
export function ValueTable({
  values,
  fixtures,
}: {
  values: SceneView["values"];
  fixtures: FixtureView[];
}) {
  return (
    <div className="wb-resource-values">
      <table>
        <thead>
          <tr>
            <th>灯具</th>
            <th>属性</th>
            <th>数值</th>
          </tr>
        </thead>
        <tbody>
          {values.map((v) => (
            <tr key={`${v.fixtureId}:${v.attribute}`}>
              <td>
                {fixtures.find((f) => f.id === v.fixtureId)?.name ??
                  v.fixtureId}
              </td>
              <td>{attributeName(v.attribute)}</td>
              <td>{sceneValueLabel(v, fixtures)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
export function CopyValuesEditor({
  scene,
  fixtures,
  busy,
  error,
  onCancel,
  onEdit,
}: {
  scene: SceneView;
  fixtures: FixtureView[];
  busy: boolean;
  error: string;
  onCancel(): void;
  onEdit(command: LibraryEdit): Promise<boolean>;
}) {
  const [sourceId, setSourceId] = useState(fixtures[0]?.id ?? "");
  const available = fixtures.find((f) => f.id === sourceId)?.attributes ?? [];
  const [attributes, setAttributes] = useState(available.map((a) => a.key));
  return (
    <LibraryDialog
      title="复制灯具属性"
      busy={busy}
      error={error}
      onCancel={onCancel}
      submit="复制独立数值"
      onSubmit={() =>
        onEdit({
          kind: "copyValues",
          sceneId: scene.id,
          sourceId,
          fixtureIds: fixtures
            .filter((f) => f.id !== sourceId)
            .map((f) => f.id),
          attributes,
        })
      }
    >
      <label>
        来源灯具
        <select
          autoFocus
          aria-label="来源灯具"
          value={sourceId}
          onChange={(e) => {
            setSourceId(e.target.value);
            setAttributes(
              fixtures
                .find((f) => f.id === e.target.value)
                ?.attributes.map((a) => a.key) ?? [],
            );
          }}
        >
          {fixtures.map((f) => (
            <option key={f.id} value={f.id}>
              {f.name}
            </option>
          ))}
        </select>
      </label>
      <p>
        写入：
        {fixtures
          .filter((f) => f.id !== sourceId)
          .map((f) => f.name)
          .join("、")}
      </p>
      <AttributeMask
        available={available}
        selected={attributes}
        onChange={setAttributes}
      />
      <p className="wb-dim">
        仅复制当前场景已记录的数值；目标灯具需支持所选属性。
      </p>
    </LibraryDialog>
  );
}
