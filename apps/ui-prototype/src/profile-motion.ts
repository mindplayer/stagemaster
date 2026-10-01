import type { ProfileDraft } from "./fixture-tools";

export function hasMotion(draft: ProfileDraft): boolean {
  return draft.channels.some(
    (c) => c.attribute === "pan" || c.attribute === "tilt",
  );
}

/** Channel identity and physical interpretation have independent lifecycles. */
export function withMotion(
  draft: ProfileDraft,
  enabled: boolean,
): ProfileDraft {
  if (enabled && hasMotion(draft)) return draft;
  const channels = draft.channels.filter(
    (c) => c.attribute !== "pan" && c.attribute !== "tilt",
  );
  if (!enabled) return { ...draft, positioning: null, channels };
  let offset = Math.max(
    Number(draft.footprint) || channels.length,
    ...channels.flatMap((c) => [
      Number(c.coarse) || 0,
      c.bits === "16" ? Number(c.fine) || 0 : 0,
    ]),
  );
  for (const attribute of ["pan", "tilt"]) {
    channels.push({
      attribute,
      coarse: String(++offset),
      fine: String(++offset),
      bits: "16",
      percent: "50",
    });
  }
  return { ...draft, channels, footprint: String(offset), positioning: null };
}

export function withPositionModel(
  draft: ProfileDraft,
  enabled: boolean,
): ProfileDraft {
  if (!enabled) return { ...draft, positioning: null };
  if (!hasMotion(draft) || draft.positioning) return draft;
  return {
    ...draft,
    positioning: {
      kind: "intersectingOrthogonal",
      pan: { minDegrees: "", maxDegrees: "", reversed: false },
      tilt: { minDegrees: "", maxDegrees: "", reversed: false },
    },
  };
}
