import type { FixturePlacement, StageEdit } from "./stage-types";

export type RiggingCommand = Extract<StageEdit, { op: "attachFixtures" }>;
export interface RiggingProjection {
  generation: number;
  placements: FixturePlacement[];
  changed: boolean;
}
/** Read-only. Apply still uses the ordinary project transaction and validates again. */
export type RiggingPreviewPort = (
  generation: number,
  command: RiggingCommand,
) => Promise<RiggingProjection>;
