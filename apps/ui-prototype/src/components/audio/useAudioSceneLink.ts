import { useState } from "react";
import type { ProjectView, SceneView } from "../../application-host";

/** Navigation context only; AudioWorkspace keeps the marker, zoom and filter alive. */
export function useAudioSceneLink(
  project: ProjectView | null,
  current: () => ProjectView | null,
  run: (work: () => Promise<void>) => Promise<boolean>,
  openScene: (scene: SceneView) => void,
  openAudio: () => void,
) {
  const [origin, setOrigin] = useState<{
    projectId: string;
    markerId: string;
    sceneId: string;
  } | null>(null);
  const marker =
    origin?.projectId === project?.id
      ? project?.audio?.markers.find((m) => m.id === origin?.markerId)
      : null;
  return {
    sceneId: origin?.sceneId,
    marker,
    reset: () => setOrigin(null),
    edit: (markerId: string) => {
      const projectId = project?.id;
      return run(async () => {
        const p = current();
        if (!p || p.id !== projectId) return;
        const m = p.audio?.markers.find((m) => m.id === markerId);
        const scene = p.scenes.find((s) => s.id === m?.sceneId);
        if (!m || !scene) throw new Error("此卡点尚未关联可编辑的灯光场景");
        setOrigin({ projectId: p.id, markerId, sceneId: scene.id });
        openScene(scene);
      });
    },
    back: () =>
      run(async () => {
        openAudio();
        setOrigin(null);
      }),
  };
}
