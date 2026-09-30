import { useState } from "react";
import type { ProjectView, SceneView } from "../../application-host";
import type { LibraryEdit, ResourceKind } from "../../library-types";
import {
  presetCoverage,
  recallGroup,
  scopeAttributes,
} from "../../library-tools";
import type { RecallMode } from "../../library-tools";
import { uniqueName } from "../../editor-tools";
import { GroupEditor } from "./GroupEditor";
import {
  AttributeMask,
  CopyValuesEditor,
  PresetEditor,
  PresetUsage,
  ValueTable,
} from "./PresetEditor";
import type { ResourceDialog } from "./resource-dialog-types";
import { ManageResource } from "./ManageResource";

export function ResourcePool({
  project,
  scene,
  selected,
  busy,
  error,
  visible,
  beforeChange,
  onEdit,
  onSelect,
  recall,
  onRecall,
}: {
  recall: RecallMode;
  onRecall(value: RecallMode): void;
  project: ProjectView;
  scene?: SceneView;
  selected: string[];
  busy: boolean;
  error: string;
  visible: boolean;
  beforeChange(): Promise<boolean>;
  onEdit(command: LibraryEdit): Promise<ProjectView | null>;
  onSelect(ids: string[]): Promise<boolean>;
}) {
  const [tab, setTab] = useState<ResourceKind>("group");
  const [groupId, setGroupId] = useState("");
  const [presetId, setPresetId] = useState("");
  const [queries, setQueries] = useState({ group: "", preset: "" });
  const [mask, setMask] = useState<string[] | null>(null);
  const [dialog, setDialog] = useState<ResourceDialog | null>(null);
  const fixtures = selected.flatMap(
    (id) => project.fixtures.find((f) => f.id === id) ?? [],
  );
  const available = scopeAttributes(fixtures);
  const attributes =
    mask === null
      ? available.map((a) => a.key)
      : mask.filter((key) => available.some((a) => a.key === key));
  const group = project.groups.find((g) => g.id === groupId);
  const preset = project.presets.find((p) => p.id === presetId);
  const resource = tab === "group" ? group : preset;
  const coverage = preset ? presetCoverage(preset, selected, attributes) : null;
  const query = queries[tab].trim().toLowerCase();
  async function mutate(command: LibraryEdit): Promise<boolean> {
    const next = await onEdit(command);
    if (!next) return false;
    const createdGroup =
      (command.kind === "saveGroup" && command.id === null) ||
      (command.kind === "duplicate" && command.resource === "group");
    const createdPreset =
      command.kind === "recordPreset" ||
      (command.kind === "duplicate" && command.resource === "preset");
    if (createdGroup) {
      setGroupId(next.groups.at(-1)?.id ?? "");
      setQueries((q) => ({ ...q, group: "" }));
    }
    if (createdPreset) {
      setPresetId(next.presets.at(-1)?.id ?? "");
      setQueries((q) => ({ ...q, preset: "" }));
    }
    return true;
  }
  async function open(next: ResourceDialog) {
    if (await beforeChange()) setDialog(next);
  }
  function manage(kind: "rename" | "duplicate" | "remove") {
    if (resource)
      void open({ kind, id: resource.id, resource: tab, name: resource.name });
  }
  return (
    <section
      className="wb-resource-pool"
      hidden={!visible}
      aria-label="灯组与预设池"
    >
      <div className="wb-selection-tools">
        <button
          disabled={busy || !scene || selected.length < 2}
          onClick={() => void open({ kind: "copyValues" })}
        >
          复制属性
        </button>
      </div>
      <div className="wb-resource-toolbar">
        <div className="wb-resource-tabs" aria-label="资源类型">
          <button
            aria-pressed={tab === "group"}
            onClick={() => setTab("group")}
          >
            灯组 <span>{project.groups.length}</span>
          </button>
          <button
            aria-pressed={tab === "preset"}
            onClick={() => setTab("preset")}
          >
            预设 <span>{project.presets.length}</span>
          </button>
        </div>
        <input
          type="search"
          aria-label={tab === "group" ? "搜索灯组" : "搜索预设"}
          placeholder={tab === "group" ? "搜索灯组" : "搜索预设"}
          value={queries[tab]}
          onChange={(e) => setQueries({ ...queries, [tab]: e.target.value })}
        />
        <button
          className="wb-primary"
          disabled={busy || !selected.length || (tab === "preset" && !scene)}
          onClick={() => void open({ kind: tab })}
        >
          {tab === "group" ? "记录灯组" : "记录预设"}
        </button>
      </div>
      {tab === "group" ? (
        <>
          <div className="wb-resource-tools">
            <label>
              召回方式
              <select
                aria-label="灯组召回方式"
                value={recall}
                onChange={(e) => onRecall(e.target.value as RecallMode)}
              >
                <option value="replace">替换选择</option>
                <option value="add">追加选择</option>
                <option value="subtract">扣除选择</option>
              </select>
            </label>
            <button
              disabled={busy || !group}
              onClick={() =>
                group && void open({ kind: "group", id: group.id })
              }
            >
              编辑灯组
            </button>
            <button
              disabled={busy || !group}
              onClick={() => manage("duplicate")}
            >
              复制灯组
            </button>
            <button disabled={busy || !group} onClick={() => manage("remove")}>
              删除灯组
            </button>
          </div>
          <div className="wb-resource-grid">
            {project.groups
              .filter((g) => g.name.toLowerCase().includes(query))
              .map((g) => (
                <button
                  key={g.id}
                  className={g.id === groupId ? "selected" : ""}
                  aria-label={`召回灯组 ${g.name}`}
                  aria-pressed={g.id === groupId}
                  disabled={busy}
                  onClick={async () => {
                    if (
                      await onSelect(
                        recallGroup(selected, g.fixtureIds, recall),
                      )
                    )
                      setGroupId(g.id);
                  }}
                >
                  <span className="wb-resource-number">
                    {project.groups.indexOf(g) + 1}
                  </span>
                  <strong>{g.name}</strong>
                  <small>{g.fixtureIds.length} 台灯具</small>
                </button>
              ))}
          </div>
          {!project.groups.length && (
            <p className="wb-resource-empty">选择灯具后记录灯组</p>
          )}
          {!!project.groups.length &&
            !project.groups.some((g) =>
              g.name.toLowerCase().includes(query),
            ) && <p className="wb-resource-empty">没有匹配的灯组</p>}
          {group && (
            <p className="wb-order-summary">
              {group.name}：
              {group.fixtureIds
                .map((id) => project.fixtures.find((f) => f.id === id)?.name)
                .join(" → ")}
            </p>
          )}
        </>
      ) : (
        <>
          <AttributeMask
            available={available}
            selected={attributes}
            onChange={setMask}
          />
          <div className="wb-resource-grid">
            {project.presets
              .filter((p) => p.name.toLowerCase().includes(query))
              .map((p) => (
                <button
                  key={p.id}
                  aria-label={`选择预设 ${p.name}`}
                  aria-pressed={p.id === presetId}
                  className={p.id === presetId ? "selected" : ""}
                  onClick={() => setPresetId(p.id)}
                >
                  <span className="wb-resource-number">
                    {project.presets.indexOf(p) + 1}
                  </span>
                  <strong>{p.name}</strong>
                  <small>
                    {new Set(p.values.map((v) => v.fixtureId)).size} 台 ·{" "}
                    {p.values.length} 项
                  </small>
                </button>
              ))}
          </div>
          {!project.presets.length && (
            <p className="wb-resource-empty">从当前场景记录可复用的灯具属性</p>
          )}
          {!!project.presets.length &&
            !project.presets.some((p) =>
              p.name.toLowerCase().includes(query),
            ) && <p className="wb-resource-empty">没有匹配的预设</p>}
          {preset && (
            <div className="wb-preset-detail">
              <div className="wb-resource-tools">
                <strong>{preset.name}</strong>
                <span className="wb-dim">
                  匹配 {coverage?.fixtures}/{selected.length} 台 ·{" "}
                  {coverage?.attributes} 项
                </span>
              </div>
              <div className="wb-resource-tools">
                <button
                  className="wb-primary"
                  disabled={busy || !scene || !coverage?.attributes}
                  onClick={() =>
                    scene &&
                    void mutate({
                      kind: "applyPreset",
                      id: preset.id,
                      sceneId: scene.id,
                      fixtureIds: selected,
                      attributes,
                      linked: true,
                    })
                  }
                >
                  引用到场景
                </button>
                <button
                  disabled={busy || !scene || !coverage?.attributes}
                  onClick={() =>
                    scene &&
                    void mutate({
                      kind: "applyPreset",
                      id: preset.id,
                      sceneId: scene.id,
                      fixtureIds: selected,
                      attributes,
                      linked: false,
                    })
                  }
                >
                  应用独立值
                </button>
                <button
                  disabled={busy || !scene || !selected.length}
                  onClick={() => void open({ kind: "preset", id: preset.id })}
                >
                  更新预设
                </button>
                <button disabled={busy} onClick={() => manage("rename")}>
                  改名
                </button>
                <button disabled={busy} onClick={() => manage("duplicate")}>
                  复制
                </button>
                <button disabled={busy} onClick={() => manage("remove")}>
                  删除
                </button>
              </div>
              <details>
                <summary>
                  内容与引用 · {preset.usedByScenes.length} 个场景
                </summary>
                <PresetUsage preset={preset} />
                <ValueTable
                  values={preset.values}
                  fixtures={project.fixtures}
                />
              </details>
            </div>
          )}
          <button
            className="wb-detach"
            disabled={
              busy ||
              !scene?.values.some(
                (v) =>
                  v.mode === "preset" &&
                  selected.includes(v.fixtureId) &&
                  attributes.includes(v.attribute),
              )
            }
            onClick={() =>
              scene &&
              void mutate({
                kind: "detach",
                sceneId: scene.id,
                fixtureIds: selected,
                attributes,
              })
            }
          >
            解除所选属性引用，保留数值
          </button>
        </>
      )}
      {dialog?.kind === "group" && (
        <GroupEditor
          group={project.groups.find((g) => g.id === dialog.id)}
          fixtures={project.fixtures}
          selected={selected}
          name={uniqueName(
            "灯组",
            project.groups.map((g) => g.name),
          )}
          busy={busy}
          error={error}
          onCancel={() => setDialog(null)}
          onEdit={mutate}
        />
      )}
      {dialog?.kind === "preset" && scene && (
        <PresetEditor
          preset={project.presets.find((p) => p.id === dialog.id)}
          scene={scene}
          fixtures={fixtures}
          name={uniqueName(
            "预设",
            project.presets.map((p) => p.name),
          )}
          busy={busy}
          error={error}
          onCancel={() => setDialog(null)}
          onEdit={mutate}
        />
      )}
      {dialog?.kind === "copyValues" && scene && (
        <CopyValuesEditor
          scene={scene}
          fixtures={fixtures}
          busy={busy}
          error={error}
          onCancel={() => setDialog(null)}
          onEdit={mutate}
        />
      )}
      {dialog &&
        ["rename", "duplicate", "remove"].includes(dialog.kind) &&
        "resource" in dialog && (
          <ManageResource
            dialog={dialog}
            project={project}
            busy={busy}
            error={error}
            onCancel={() => setDialog(null)}
            onEdit={mutate}
          />
        )}
    </section>
  );
}
