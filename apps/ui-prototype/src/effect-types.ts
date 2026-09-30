export interface SceneEffect {
  id: string;
  name: string;
  enabled: boolean;
  fixtureIds: string[];
  periodMs: number;
  spreadDegrees: number;
  phaseDegrees: number;
  reverse: boolean;
  waveform: "smooth" | "triangle" | "pulse" | "keyframes" | "position";
  dutyPercent: number;
  channels: EffectChannel[];
}
export type EffectAttribute =
  "dimmer" | "red" | "green" | "blue" | "pan" | "tilt";
export interface EffectKeyframe {
  position: number;
  value: number;
  transition: "hold" | "linear" | "smooth";
}
export type EffectChannel = { attribute: EffectAttribute } & (
  | {
      low: number;
      high: number;
      keyframes?: never;
      amplitudeDegrees?: never;
      offsetDegrees?: never;
      phaseDegrees?: never;
    }
  | {
      keyframes: EffectKeyframe[];
      low?: never;
      high?: never;
      amplitudeDegrees?: never;
      offsetDegrees?: never;
      phaseDegrees?: never;
    }
  | {
      amplitudeDegrees: string;
      offsetDegrees: string;
      phaseDegrees: number;
      low?: never;
      high?: never;
      keyframes?: never;
    }
);
export type EffectEdit =
  | { kind: "put"; sceneId: string; effect: SceneEffect }
  | { kind: "remove"; sceneId: string; id: string };
