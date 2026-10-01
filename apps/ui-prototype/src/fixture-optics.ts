import type { ProfileDraft } from "./fixture-tools";
import { nextChannel } from "./fixture-function-draft.ts";
export const opticsLabels: Record<string, string> = {
  zoom: "变焦",
  focus: "调焦",
  iris: "光圈",
};
export function addOpticsChannel(
  draft: ProfileDraft,
  attribute: string,
): ProfileDraft {
  if (
    !Object.hasOwn(opticsLabels, attribute) ||
    draft.channels.some((c) => c.attribute === attribute)
  )
    return draft;
  const coarse = nextChannel(draft);
  return {
    ...draft,
    footprint: String(Math.max(Number(draft.footprint) || 0, coarse)),
    channels: [
      ...draft.channels,
      { attribute, coarse: String(coarse), fine: "", bits: "8", percent: "0" },
    ],
  };
}
