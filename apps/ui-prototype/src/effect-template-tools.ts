import type { SceneEffect } from "./effect-types";
export function canExportEffectTemplate(effect: SceneEffect) {
  if (effect.targetPath || effect.channels.length !== 1) return false;
  const channel = effect.channels[0];
  if (channel.attribute !== "dimmer") return false;
  if (effect.waveform === "keyframes") return Array.isArray(channel.keyframes);
  return (
    ["smooth", "triangle", "pulse"].includes(effect.waveform) &&
    typeof channel.low === "number" &&
    typeof channel.high === "number"
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
