export interface SceneEffect {
  id: string;
  name: string;
  enabled: boolean;
  fixtureIds: string[];
  periodMs: number;
  spreadDegrees: number;
  phaseDegrees: number;
  reverse: boolean;
  waveform:
    "smooth" | "triangle" | "pulse" | "keyframes" | "position" | "worldLine";
  dutyPercent: number;
  channels: EffectChannel[];
  targetPath?: WorldLinePath;
  templateSource?: import("./effect-template-types").EffectTemplateSource;
}
export interface WorldLinePath {
  kind: "line";
  fromMeters: import("./stage-types").SpatialVector3;
  toMeters: import("./stage-types").SpatialVector3;
  branch: "auto" | "front" | "back";
  maxErrorMeters: string;
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
      low?: never;
      high?: never;
      keyframes?: never;
      amplitudeDegrees?: never;
      offsetDegrees?: never;
      phaseDegrees?: never;
    }
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
