import { useState } from "react";
import type { ProjectView } from "../../application-host";
import type { StageEdit, StageObject } from "../../stage-types";
import { RigAttachmentDialog } from "./RigAttachmentDialog";
export function useStageRigging({
  project,
  object,
  liveIds,
  beforeChange,
  edit,
  onResult,
  busy,
  error,
}: {
  project: ProjectView;
  object: StageObject | null;
  liveIds: string[];
  beforeChange(): Promise<boolean>;
  edit(command: StageEdit): Promise<ProjectView | null>;
  onResult(ids: string[], next: ProjectView): void;
  busy: boolean;
  error: string;
}) {
  const [hanging, setHanging] = useState<{ ids: string[]; rig: string } | null>(
    null,
  );
  async function hang() {
    if (!(await beforeChange())) return;
    const rig =
      object?.kind === "construction" && object.value.shape.kind === "rig"
        ? object.value.id
        : (project.stage.attachments.find((a) => liveIds.includes(a.fixtureId))
            ?.constructionId ??
          project.stage.constructions.find((c) => c.shape.kind === "rig")?.id ??
          "");
    setHanging({
      rig,
      ids: liveIds.length
        ? liveIds
        : project.stage.attachments
            .filter((a) => a.constructionId === rig)
            .map((a) => a.fixtureId),
    });
  }
  async function detach(ids: string[]) {
    if (!(await beforeChange())) return;
    await edit({
      op: "attachFixtures",
      constructionId: null,
      fixtureIds: ids,
      layout: null,
    });
  }
  const dialog = (
    <>
      {hanging && (
        <RigAttachmentDialog
          project={project}
          initialIds={hanging.ids}
          initialRig={hanging.rig}
          busy={busy}
          error={error}
          onCancel={() => setHanging(null)}
          onApply={async (command) => {
            const next = await edit(command);
            if (!next) return false;
            if (command.op === "attachFixtures")
              onResult(command.fixtureIds, next);
            return true;
          }}
        />
      )}
    </>
  );
  return { hang, detach, dialog };
}
