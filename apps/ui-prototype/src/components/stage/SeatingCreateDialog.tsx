import { useState } from "react";
import type { ProjectView } from "../../application-host";
import type { SeatingShape, StageEdit } from "../../stage-types";
import { stageCommand } from "../../stage-tools";
import { SeatingFields } from "./SeatingFields";
import { LibraryDialog } from "../workbench/LibraryDialog";
export function SeatingCreateDialog({
  project,
  initial,
  name: initialName,
  busy,
  error,
  onApply,
  onCancel,
}: {
  project: ProjectView;
  initial: SeatingShape;
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
      title="新建观众座区"
      submit="创建座区"
      busy={busy}
      error={error}
      onCancel={onCancel}
      onSubmit={async () => {
        const command = stageCommand({
          kind: "construction",
          value: { id: "", name, shape },
        });
        return (
          command.op === "putConstruction" && onApply({ ...command, id: null })
        );
      }}
    >
      <label>
        名称
        <input
          aria-label="座区名称"
          required
          maxLength={256}
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </label>
      <SeatingFields
        value={shape}
        spaces={project.stage.spaces}
        onChange={setShape}
      />
    </LibraryDialog>
  );
}
