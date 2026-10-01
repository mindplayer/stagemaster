import type { ProfileDefinition } from "./fixture-types";
export interface ImportedProfile {
  generation: number;
  fileName: string;
  source: { profileId: string; revision: string };
  definition: ProfileDefinition;
}
export interface ExportedProfile {
  generation: number;
  profileId: string;
  revision: string;
  path: string | null;
  warning: string | null;
}
