import type { FixtureView, ProjectView } from "./application-host";
import type { ProfileView } from "./fixture-types";
import type { FunctionDefinition } from "./fixture-function-types";
import { sameAppearance } from "./wheel-appearance.ts";

export interface ColorSlotChange {
  before: FunctionDefinition;
  after: FunctionDefinition;
  remap: boolean;
}
export interface ProfileExchangeReview {
  signature: string;
  needsRemap: boolean;
  groups: {
    sourceId: string;
    sourceName: string;
    fixtures: FixtureView[];
    changes: ColorSlotChange[];
  }[];
}
/** Caller first checks compatibleProfile; this only describes supported changes. */
export function profileExchangeReview(
  project: ProjectView,
  fixtures: FixtureView[],
  target: ProfileView,
): ProfileExchangeReview {
  const targetFunctions = target.channels.find(
    (c) => c.attribute === "color-wheel",
  )?.functions;
  const groups = [...new Set(fixtures.map((f) => f.profileId))].map(
    (sourceId) => {
      const members = fixtures.filter((f) => f.profileId === sourceId);
      const functions = members[0].attributes.find(
        (a) => a.key === "color-wheel",
      )?.function?.functions;
      const changes = (functions ?? []).flatMap((before) => {
        const after = targetFunctions?.find((f) => f.key === before.key);
        if (!after) return [];
        const remap =
          before.dmxFrom !== after.dmxFrom ||
          before.dmxTo !== after.dmxTo ||
          before.dmxDefault !== after.dmxDefault;
        return remap ||
          before.name !== after.name ||
          !sameAppearance(before.appearance, after.appearance)
          ? [{ before, after, remap }]
          : [];
      });
      return {
        sourceId,
        sourceName: members[0].profileName,
        fixtures: members,
        changes,
      };
    },
  );
  return {
    // A changed selection or profile revision cannot inherit an earlier review.
    signature: JSON.stringify([
      target.id,
      target.revision,
      fixtures.map((f) => [
        f.id,
        f.profileId,
        project.profiles.find((p) => p.id === f.profileId)?.revision,
      ]),
    ]),
    needsRemap: groups.some((g) => g.changes.some((c) => c.remap)),
    groups,
  };
}
