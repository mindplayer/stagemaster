import { useRef, useState } from "react";
import type {
  ApplicationHost,
  EditCommand,
  ProjectView,
} from "../../application-host";
import type { EffectHandle } from "./EffectEditor";
import { startScenePreview } from "./scene-preview-action";
import { useEffectSelection, type OpenEffect } from "./useEffectSelection";

/** Effect drafts share the workbench transaction; playback stays in the host. */
export function useEffectWorkspace({
  project,
  sceneId,
  host,
  getProject,
  run,
  edit,
  openPlayback,
  clearError,
}: {
  project: ProjectView | null;
  sceneId: string;
  host: ApplicationHost;
  getProject(): ProjectView | null;
  run(work: () => Promise<void>): Promise<boolean>;
  edit(command: EditCommand): Promise<unknown>;
  openPlayback(): void;
  clearError(): void;
}) {
  const editor = useRef<EffectHandle>(null);
  const [pending, setPending] = useState(false);
  const identity = useRef(sceneId);
  identity.current = sceneId;
  const selection = useEffectSelection(project, sceneId);
  function close() {
    selection.close();
    setPending(false);
    clearError();
  }
  const open: OpenEffect = (effect, isNew, copyFrom) => {
    const current = getProject();
    if (current) selection.open(current, sceneId, effect, isNew, copyFrom);
  };
  function toggle(id: string, enabled: boolean) {
    return run(async () => {
      const scene = getProject()?.scenes.find((s) => s.id === sceneId);
      const effect = scene?.effects.find((e) => e.id === id);
      if (!scene || !effect) throw new Error("效果已被删除，请重新选择");
      await edit({
        op: "effect",
        command: { kind: "put", sceneId, effect: { ...effect, enabled } },
      });
    });
  }
  function preview() {
    return run(async () => {
      const projectId = project?.id;
      if (!projectId || !sceneId) throw new Error("请先选择场景");
      await startScenePreview(
        host,
        sceneId,
        () => getProject()?.id === projectId && identity.current === sceneId,
      );
      openPlayback();
    });
  }
  return {
    editor,
    pending,
    setPending,
    active: selection.active,
    open,
    toggle,
    close,
    apply: () => run(async () => {}),
    preview,
  };
}
