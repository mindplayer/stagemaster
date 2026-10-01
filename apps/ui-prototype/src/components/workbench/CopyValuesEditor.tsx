import { useState } from "react";
import type { FixtureView, SceneView } from "../../application-host";
import type { LibraryEdit } from "../../library-types";
import { attributeName } from "../../library-tools";
import {
  copyValuesPreview,
  copySubmissionIssue,
} from "../../copy-values-preview";
import { LibraryDialog } from "./LibraryDialog";
import { AttributeMask } from "./AttributeMask";
import { ValueTable } from "./PresetEditor";
import { ResourcePicker } from "../resources/ResourcePicker";
import { CopyValueTargets } from "./CopyValueTargets";
import "./copy-values.css";

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
  const source = fixtures.find((f) => f.id === sourceId);
  const available = source?.attributes ?? [];
  const [attributes, setAttributes] = useState(available.map((a) => a.key));
  const [selected, setSelected] = useState(fixtures.map((f) => f.id));
  const preview = copyValuesPreview(scene, source, fixtures, attributes);
  const selectedSet = new Set(selected);
  const ids = preview.targets
    .filter((t) => selectedSet.has(t.fixture.id))
    .map((t) => t.fixture.id);
  const problem = copySubmissionIssue(preview, ids);
  return (
    <LibraryDialog
      title="复制灯具属性"
      busy={busy}
      error={error}
      onCancel={onCancel}
      submit="复制独立数值"
      submitDisabled={!!problem}
      onSubmit={() => {
        if (problem) throw new Error(problem);
        return onEdit({
          kind: "copyValues",
          sceneId: scene.id,
          sourceId,
          fixtureIds: ids,
          attributes: preview.keys,
        });
      }}
    >
      <p>来源场景：{scene.name}</p>
      <ResourcePicker
        label="来源灯具"
        placeholder="选择来源灯具"
        value={sourceId}
        disabled={busy}
        options={fixtures.map((f) => ({
          id: f.id,
          label: f.name,
          detail: `${f.profileName} · ${f.universe ?? "—"}.${f.address ?? "—"}`,
        }))}
        onSelect={(id) => {
          setSourceId(id);
          const keys = new Set(
            fixtures.find((f) => f.id === id)?.attributes.map((a) => a.key),
          );
          setAttributes((old) => old.filter((key) => keys.has(key)));
        }}
      />
      <AttributeMask
        available={available}
        selected={attributes}
        onChange={setAttributes}
      />
      <p>
        来源已记录 {preview.values.length} 项 · 已选目标 {ids.length} 台，对应{" "}
        {preview.values.length * ids.length} 项
      </p>
      {preview.skipped.length > 0 && (
        <p className="wb-dim">
          跳过释放或未记录：{preview.skipped.map(attributeName).join("、")}
        </p>
      )}
      <details>
        <summary>查看来源内容</summary>
        <ValueTable values={preview.values} fixtures={fixtures} />
      </details>
      <CopyValueTargets
        ready={!preview.sourceIssue}
        targets={preview.targets}
        selected={selected}
        onChange={setSelected}
      />
      <p className="wb-dim">
        目标保存独立值，不建立预设引用，其余属性不变。功能按定义匹配，不转换不同灯型的颜色或光学效果。
      </p>
      {problem && (
        <p className="copy-value-issue" role="status">
          {problem}
        </p>
      )}
    </LibraryDialog>
  );
}
