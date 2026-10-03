import { useCommittedStageAction } from "./useCommittedStageAction";
import type { FixturePlacement } from "../../stage-types";
import type { RiggingPreviewPort } from "../../rigging-preview-types";
import { useRiggingDraft } from "./useRiggingDraft";
import type { ProjectView } from "../../application-host";
import type { StageEdit, StageObject } from "../../stage-types";
import { RigAttachmentInspector } from "./RigAttachmentInspector";
export function useStageRigging({
  project,
  object,
  liveIds,
  beforeChange,
  edit,
  onResult,
  busy,
  visible,
  onError,
  error,
  generation,
  preview,
  onPending,
  onAccepted,
  onOpen,
}: {
  project: ProjectView;
  object: StageObject | null;
  liveIds: string[];
  beforeChange(): Promise<boolean>;
  edit(command: StageEdit): Promise<ProjectView | null>;
  onResult(ids: string[], next: ProjectView): void;
  busy: boolean;
  visible: boolean;
  onError(message: string): void;
  error: string;
  generation: number;
  preview?: RiggingPreviewPort;
  onPending(value: boolean): void;
  onAccepted(ids: string[], placements: FixturePlacement[]): void;
  onOpen(): void;
}) {
  const draft = useRiggingDraft({
    project,
    generation,
    preview,
    onPending,
    onAccepted,
  });
  const open = useCommittedStageAction<{ ids: string[]; rig: string }>({
    scopeId: project.id,
    active: visible,
    busy,
    beforeChange,
    onError,
    execute(opening) {
      const rig =
        opening.rig ||
        project.stage.attachments.find((a) => opening.ids.includes(a.fixtureId))
          ?.constructionId ||
        project.stage.constructions.find((c) => c.shape.kind === "rig")?.id ||
        "";
      const ids = opening.ids.length
        ? opening.ids
        : project.stage.attachments
            .filter((a) => a.constructionId === rig)
            .map((a) => a.fixtureId);
      draft.open(ids, rig);
      onOpen();
    },
  });
  function hang() {
    return open({
      ids: [...liveIds],
      rig:
        object?.kind === "construction" && object.value.shape.kind === "rig"
          ? object.value.id
          : "",
    });
  }
  async function detach(ids: string[]) {
    if (!(await beforeChange())) return;
    const next = await edit({
      op: "attachFixtures",
      constructionId: null,
      fixtureIds: ids,
      layout: null,
    });
    if (next) onResult(ids, next);
  }
  const inspector = draft.session && (
    <RigAttachmentInspector
      project={project}
      draft={draft}
      busy={busy}
      error={error}
      onApply={() => {
        draft.prepareApply();
        void beforeChange();
      }}
    />
  );
  return { hang, detach, draft, inspector };
}
