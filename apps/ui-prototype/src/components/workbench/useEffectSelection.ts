import { useEffect, useState } from "react";
import type { ProjectView } from "../../application-host";
import type { SceneEffect } from "../../effect-types";
import { uniqueName } from "../../editor-tools";

export interface EffectSelection {
  projectId: string;
  sceneId: string;
  effect: SceneEffect;
  isNew: boolean;
  token: string;
}
export type OpenEffect = (
  effect: SceneEffect,
  isNew: boolean,
  copyFrom?: string,
) => void;

export function useEffectSelection(
  project: ProjectView | null,
  sceneId: string,
) {
  const [selection, setSelection] = useState<EffectSelection | null>(null);
  const scene =
    selection?.projectId === project?.id && selection?.sceneId === sceneId
      ? project?.scenes.find((s) => s.id === sceneId)
      : null;
  const saved = scene?.effects.find((e) => e.id === selection?.effect.id);
  useEffect(() => {
    if (saved && selection?.isNew)
      setSelection((s) =>
        s && s.token === selection.token
          ? { ...s, isNew: false, effect: saved }
          : s,
      );
  }, [saved, selection]);
  const active =
    selection && scene && (saved || selection.isNew)
      ? { ...selection, effect: saved ?? selection.effect, isNew: !saved }
      : null;
  function open(
    current: ProjectView,
    sceneId: string,
    effect: SceneEffect,
    isNew: boolean,
    copyFrom?: string,
  ) {
    const scene = current.scenes.find((s) => s.id === sceneId);
    if (!scene) return;
    const source = scene.effects.find((e) => e.id === (copyFrom ?? effect.id));
    if ((!isNew || copyFrom) && !source) return;
    const value =
      copyFrom && source
        ? {
            ...structuredClone(source),
            id: effect.id,
            enabled: false,
            name: uniqueName(
              `${source.name} 副本`,
              scene.effects.map((e) => e.name),
            ),
          }
        : !isNew
          ? source!
          : effect;
    setSelection({
      projectId: current.id,
      sceneId,
      effect: value,
      isNew,
      token: crypto.randomUUID(),
    });
  }
  return { active, open, close: () => setSelection(null) };
}
