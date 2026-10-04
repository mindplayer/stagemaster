export interface AudioAsset {
  digest: string;
  fileName: string;
  extension: "wav" | "mp3" | "flac";
  durationMs: number;
}
export interface AudioMarker {
  id: string;
  name: string;
  timeMs: number;
  sceneId: string | null;
  fadeMs?: number;
}
export type ClipFadeMode = "snapshot" | "dynamic";
export interface ClipEntryCrossfade {
  durationMs: number;
  offsetMs: number;
  source: {
    sceneId: string | null;
    effectOffsetMs: number;
    elapsedMs: number;
    entryFade?: ClipEntryFade;
  };
}
export interface AudioLightingClip {
  id: string;
  name: string;
  sceneId: string;
  startMs: number;
  endMs: number;
  fadeMs: number;
  locked: boolean;
  enabled?: boolean;
  effectOffsetMs?: number;
  entryFade?: ClipEntryFade;
  fadeMode?: ClipFadeMode;
  entryCrossfade?: ClipEntryCrossfade;
}
export interface ClipEntryFade {
  durationMs: number;
  offsetMs: number;
  from: Array<{ fixtureId: string; attribute: string; value: number }>;
}
export interface AudioTimeline {
  asset: AudioAsset;
  inMs: number;
  outMs: number;
  markers: AudioMarker[];
  lightingClips?: AudioLightingClip[];
  loopRegions?: AudioLoopRegion[];
}
export interface AudioWaveform {
  durationMs: number;
  bucketMs: number;
  channels: number[][];
}
export interface PreparedAudio {
  asset: AudioAsset;
  waveform: AudioWaveform;
}
export interface AudioLoopRange {
  startMs: number;
  endMs: number;
}
export interface AudioPosition {
  performance?: AudioPerformancePosition | null;
  loopRange?: AudioLoopRange | null;
  volumePercent: number;
  playing: boolean;
  positionMs: number;
  durationMs: number;
  problem: string | null;
}
export type AudioCommand =
  | {
      kind: "exitLoop";
      instance: string;
      regionId: string;
      pass: string;
      requested: boolean;
    }
  | { kind: "setLoop"; range: AudioLoopRange | null }
  | { kind: "volume"; percent: number }
  | { kind: "snapshot" | "play" | "pause" | "stop" }
  | { kind: "seek"; positionMs: number };
export type MarkerGroupAction =
  { kind: "move" | "copy"; destinationMs: number } | { kind: "remove" };
export type LightingClipGroupAction =
  | { kind: "fade"; fadeMs: number; fadeMode?: ClipFadeMode }
  | { kind: "enabled"; enabled: boolean }
  | { kind: "move" | "copy"; destinationMs: number }
  | { kind: "remove" };
export type AudioEdit =
  | { kind: "loopRegions"; command: AudioLoopEdit }
  | {
      kind: "editLightingClips";
      ids: string[];
      action: LightingClipGroupAction;
    }
  | { kind: "splitLightingClip"; id: string; timeMs: number }
  | { kind: "resetLightingClipEffectOffset"; id: string }
  | { kind: "resetLightingClipEntryFade"; id: string }
  | { kind: "sliceLightingClip"; clip: AudioLightingClip }
  | { kind: "convertLightingClips" }
  | {
      kind: "addLightingClip";
      fadeMode?: ClipFadeMode;
      name: string;
      sceneId: string;
      startMs: number;
      endMs: number;
      fadeMs: number;
    }
  | { kind: "trimLightingClip"; clip: AudioLightingClip }
  | { kind: "putLightingClip"; clip: AudioLightingClip }
  | { kind: "copyLightingClip"; id: string; startMs: number }
  | { kind: "removeLightingClip"; id: string }
  | { kind: "setLightingClipLock"; id: string; locked: boolean }
  | { kind: "editMarkers"; ids: string[]; action: MarkerGroupAction }
  | { kind: "setAsset"; asset: AudioAsset }
  | { kind: "clear" }
  | { kind: "trim"; inMs: number; outMs: number }
  | { kind: "putMarker"; marker: AudioMarker }
  | { kind: "removeMarker"; id: string };
import type {
  AudioLoopEdit,
  AudioLoopRegion,
  AudioPerformancePosition,
} from "./audio-performance-types";
