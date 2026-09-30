import { useState } from "react";
import type { ProjectView } from "../../application-host";
import type { LibraryEdit } from "../../library-types";
import type { ResourceDialog } from "./resource-dialog-types";
import { uniqueName } from "../../editor-tools";
import { LibraryDialog } from "./LibraryDialog";
import { PresetUsage } from "./PresetEditor";

export function ManageResource({
  dialog,
  project,
  busy,
  error,
  onCancel,
  onEdit,
}: {
  dialog: Extract<ResourceDialog, { resource: string }>;
  project: ProjectView;
  busy: boolean;
  error: string;
  onCancel(): void;
  onEdit(command: LibraryEdit): Promise<boolean>;
}) {
  const label = dialog.resource === "group" ? "灯组" : "预设";
  const [name, setName] = useState(
    dialog.kind === "duplicate"
      ? uniqueName(
          `${dialog.name} 副本`,
          (dialog.resource === "group" ? project.groups : project.presets).map(
            (r) => r.name,
          ),
        )
      : dialog.name,
  );
  const [keep, setKeep] = useState(false);
  const preset =
    dialog.resource === "preset"
      ? project.presets.find((p) => p.id === dialog.id)
      : undefined;
  const title = `${dialog.kind === "remove" ? "删除" : dialog.kind === "duplicate" ? "复制" : "重命名"}${label}`;
  return (
    <LibraryDialog
      title={title}
      busy={busy}
      error={error}
      onCancel={onCancel}
      submit={title}
      onSubmit={() => {
        if (dialog.kind === "remove")
          return onEdit({
            kind: "remove",
            resource: dialog.resource,
            id: dialog.id,
            keepValues: keep,
          });
        if (!name.trim()) throw new Error(`请填写${label}名称`);
        return onEdit(
          dialog.kind === "duplicate"
            ? {
                kind: "duplicate",
                resource: dialog.resource,
                id: dialog.id,
                name: name.trim(),
              }
            : { kind: "renamePreset", id: dialog.id, name: name.trim() },
        );
      }}
    >
      {dialog.kind !== "remove" ? (
        <label>
          {label}名称
          <input
            autoFocus
            aria-label={`${label}名称`}
            required
            maxLength={256}
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </label>
      ) : (
        <>
          <p>删除“{dialog.name}”</p>
          {preset && (
            <>
              <PresetUsage preset={preset} />
              {!!preset.usedByScenes.length && (
                <label className="wb-check">
                  <input
                    type="checkbox"
                    checked={keep}
                    onChange={(e) => setKeep(e.target.checked)}
                  />
                  保留场景当前数值并删除预设
                </label>
              )}
            </>
          )}
        </>
      )}
    </LibraryDialog>
  );
}
