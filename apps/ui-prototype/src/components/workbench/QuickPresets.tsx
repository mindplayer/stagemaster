import { ResourcePicker } from "../resources/ResourcePicker";
import { useState } from "react";
import type { ProjectView, SceneView } from "../../application-host";
import type { LibraryEdit } from "../../library-types";
import { presetCoverage, scopeAttributes } from "../../library-tools";
import "./quick-presets.css";

export function QuickPresets({
  project,
  scene,
  selected,
  busy,
  presetId,
  mask,
  onPreset,
  onMask,
  onEdit,
}: {
  project: ProjectView;
  scene?: SceneView;
  selected: string[];
  busy: boolean;
  presetId: string;
  mask: string[] | null;
  onPreset(id: string): void;
  onMask(keys: string[] | null): void;
  onEdit(command: LibraryEdit): Promise<ProjectView | null>;
}) {
  const [linked, setLinked] = useState(true);
  const available = scopeAttributes(
    project.fixtures.filter((f) => selected.includes(f.id)),
  );
  const attributes = available
    .map((a) => a.key)
    .filter((a) => mask === null || mask.includes(a));
  const preset = project.presets.find((p) => p.id === presetId);
  const coverage = preset ? presetCoverage(preset, selected, attributes) : null;
  const scopes = [
    { id: "all", name: "全部属性", keys: null },
    { id: "light", name: "仅亮度", keys: ["dimmer"] },
    { id: "color", name: "仅颜色", keys: ["red", "green", "blue"] },
    { id: "position", name: "仅位置", keys: ["pan", "tilt"] },
    { id: "optics", name: "仅镜头与光圈", keys: ["zoom", "focus", "iris"] },
  ];
  const scope =
    scopes.find((s) => JSON.stringify(s.keys) === JSON.stringify(mask))?.id ??
    "custom";
  if (!project.presets.length) return null;
  return (
    <section className="quick-presets" aria-label="快捷预设">
      <span>预设</span>
      <ResourcePicker
        label="快捷预设"
        placeholder="选择预设"
        value={preset?.id}
        disabled={busy}
        options={[
          { id: "", label: "不选择预设" },
          ...project.presets.map((p) => ({
            id: p.id,
            label: p.name,
            detail: `${new Set(p.values.map((v) => v.fixtureId)).size} 台灯具 · ${p.values.length} 项属性`,
            keywords: [
              ...new Set(
                p.values.map(
                  (v) =>
                    available.find((a) => a.key === v.attribute)?.label ??
                    v.attribute,
                ),
              ),
            ].join(" "),
          })),
        ]}
        onSelect={onPreset}
      />
      <select
        aria-label="快捷预设属性范围"
        disabled={busy}
        value={scope}
        onChange={(e) =>
          onMask(scopes.find((s) => s.id === e.target.value)!.keys)
        }
      >
        {scopes.map((s) => (
          <option key={s.id} value={s.id}>
            {s.name}
          </option>
        ))}
        {scope === "custom" && (
          <option value="custom" disabled>
            自选属性
          </option>
        )}
      </select>
      <select
        aria-label="快捷预设应用方式"
        disabled={busy}
        value={linked ? "linked" : "values"}
        onChange={(e) => setLinked(e.target.value === "linked")}
      >
        <option value="linked">保持引用</option>
        <option value="values">独立数值</option>
      </select>
      <button
        disabled={busy || !scene || !coverage?.attributes}
        onClick={() => {
          if (scene && preset)
            void onEdit({
              kind: "applyPreset",
              id: preset.id,
              sceneId: scene.id,
              fixtureIds: selected,
              attributes,
              linked,
            });
        }}
      >
        {linked ? "引用预设" : "应用数值"}
      </button>
      {preset && (
        <small role="status" title="仅修改匹配灯具的所选属性，其余保持原值">
          匹配 {coverage?.fixtures}/{selected.length} 台 ·{" "}
          {coverage?.attributes} 项
        </small>
      )}
    </section>
  );
}
