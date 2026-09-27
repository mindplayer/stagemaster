export interface SceneEffect {
  id: string;
  name: string;
  enabled: boolean;
  fixtureIds: string[];
  periodMs: number;
  spreadDegrees: number;
  phaseDegrees: number;
  reverse: boolean;
  waveform: "smooth" | "triangle" | "pulse" | "keyframes";
  dutyPercent: number;
  channels: EffectChannel[];
}
export type EffectAttribute = "dimmer" | "red" | "green" | "blue";
export interface EffectKeyframe {
  position: number;
  value: number;
  transition: "hold" | "linear" | "smooth";
}
export type EffectChannel = { attribute: EffectAttribute } & (
  | { low: number; high: number; keyframes?: never }
  | { keyframes: EffectKeyframe[]; low?: never; high?: never }
);
export type EffectEdit =
  | { kind: "put"; sceneId: string; effect: SceneEffect }
  | { kind: "remove"; sceneId: string; id: string };
