import { useState } from "react";
import type { ProjectView } from "../../application-host";
import type { RigShape, StageEdit } from "../../stage-types";
import { stageCommand } from "../../stage-tools";
import { RigFields } from "./RigFields";
import { LibraryDialog } from "../workbench/LibraryDialog";
export function RigCreateDialog({
  project,
  initial,
  name: initialName,
  busy,
  error,
  onApply,
  onCancel,
}: {
  project: ProjectView;
  initial: RigShape;
  name: string;
  busy: boolean;
  error: string;
  onApply(command: StageEdit): Promise<boolean>;
  onCancel(): void;
}) {
  const [shape, setShape] = useState(initial),
    [name, setName] = useState(initialName);
  return (
    <LibraryDialog
      title="新建桁架或灯杆"
      submit="创建支撑体"
      busy={busy}
      error={error}
      onCancel={onCancel}
      onSubmit={async () => {
        const command = stageCommand({
          kind: "construction",
          value: { id: "", name, shape },
        });
        if (command.op !== "putConstruction") return false;
        return onApply({ ...command, id: null });
      }}
    >
      <label>
        名称
        <input
          aria-label="支撑体名称"
          required
          maxLength={256}
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </label>
      <RigFields
        value={shape}
        spaces={project.stage.spaces}
        onChange={setShape}
      />
    </LibraryDialog>
  );
}
