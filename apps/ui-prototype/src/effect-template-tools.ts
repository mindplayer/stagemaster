import type { SceneEffect } from "./effect-types";
export function canExportEffectTemplate(effect: SceneEffect) {
  return (
    ["smooth", "triangle", "pulse"].includes(effect.waveform) &&
    effect.channels.length === 1 &&
    effect.channels[0].attribute === "dimmer" &&
    typeof effect.channels[0].low === "number" &&
    typeof effect.channels[0].high === "number" &&
    !effect.targetPath
  );
}
/** Order matters; visibility transitions are tracked by the caller's epoch. */
export function effectTemplateContext(
  sceneId: string,
  selected: string[],
  visible: boolean,
) {
  return JSON.stringify([sceneId, selected, visible]);
}
