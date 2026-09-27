export interface SceneEffect {
  id: string;
  name: string;
  enabled: boolean;
  fixtureIds: string[];
  periodMs: number;
  spreadDegrees: number;
  phaseDegrees: number;
  reverse: boolean;
  waveform: "smooth" | "triangle" | "pulse";
  dutyPercent: number;
  channels: {
    attribute: "dimmer" | "red" | "green" | "blue";
    low: number;
    high: number;
  }[];
}
export type EffectEdit =
  | { kind: "put"; sceneId: string; effect: SceneEffect }
  | { kind: "remove"; sceneId: string; id: string };
