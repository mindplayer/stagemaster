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
}
export interface AudioTimeline {
  asset: AudioAsset;
  inMs: number;
  outMs: number;
  markers: AudioMarker[];
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
export interface AudioPosition {
  volumePercent: number;
  playing: boolean;
  positionMs: number;
  durationMs: number;
  problem: string | null;
}
export type AudioCommand =
  | { kind: "volume"; percent: number }
  | { kind: "snapshot" | "play" | "pause" | "stop" }
  | { kind: "seek"; positionMs: number };
export type AudioEdit =
  | { kind: "setAsset"; asset: AudioAsset }
  | { kind: "clear" }
  | { kind: "trim"; inMs: number; outMs: number }
  | { kind: "putMarker"; marker: AudioMarker }
  | { kind: "removeMarker"; id: string };
