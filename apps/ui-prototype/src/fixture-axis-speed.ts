import type { ProfileDraft } from "./fixture-tools";
import { FixtureFieldError } from "./fixture-field-error.ts";
import { nextChannel } from "./fixture-function-draft.ts";

export const axisSpeedKey = "pan-tilt-speed";
export const axisSpeedLabel = "两轴速度控制";

export function hasBothAxes(draft: ProfileDraft) {
  return ["pan", "tilt"].every((key) =>
    draft.channels.some((c) => c.attribute === key),
  );
}

/** No implicit fastest/slowest value when a manual has not supplied the direction. */
export function addAxisSpeedChannel(draft: ProfileDraft): ProfileDraft {
  if (
    !hasBothAxes(draft) ||
    draft.channels.some((c) => c.attribute === axisSpeedKey)
  )
    return draft;
  const coarse = nextChannel(draft);
  return {
    ...draft,
    footprint: String(Math.max(Number(draft.footprint) || 0, coarse)),
    channels: [
      ...draft.channels,
      {
        attribute: axisSpeedKey,
        coarse: String(coarse),
        fine: "",
        bits: "8",
        percent: "",
      },
    ],
  };
}

export function validateAxisSpeedDraft(draft: ProfileDraft) {
  if (
    draft.channels.some((c) => c.attribute === axisSpeedKey) &&
    !hasBothAxes(draft)
  )
    throw new FixtureFieldError(
      "family",
      "两轴速度控制须同时具备水平和垂直通道",
    );
}
