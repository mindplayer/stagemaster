import type { PositionModel } from "./position-types";
export interface ProfileChannel {
  attribute: string;
  coarse: number;
  fine: number | null;
  defaultValue: number;
}
export interface ProfileDefinition {
  positioning?: PositionModel | null;
  name: string;
  manufacturer: string;
  model: string;
  mode: string;
  footprint: number;
  channels: ProfileChannel[];
}
export interface ProfileView extends ProfileDefinition {
  id: string;
  revision: string;
  authorable: boolean;
}
export interface Repatch {
  universe: number;
  address: number;
  gap: number;
}
export type FixtureEdit =
  | { op: "saveProfile"; id: string | null; definition: ProfileDefinition }
  | { op: "removeProfile"; id: string }
  | { op: "repatch"; fixtureIds: string[]; layout: Repatch }
  | {
      op: "exchange";
      fixtureIds: string[];
      profileId: string;
      layout: Repatch | null;
    };
