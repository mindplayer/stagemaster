import type { ApplicationHost, ProjectView } from "./application-host";
import type { ManualFixture } from "./execution-manual";
export interface ManualCapture {
  generation: number;
  token: string;
  sourceName: string;
  revision: string;
  readings: { fixtureId: string; attribute: string; value: number }[];
  fixtures: ManualFixture[];
  merge?: ManualMergeSummary | null;
}
export type ManualCapturePort = (
  request:
    | {
        kind: "capture";
        generation: number;
        hostId: string;
        source: string;
        selected: string[] | null;
        sceneId?: string | null;
      }
    | { kind: "cancel"; token: string },
) => Promise<ManualCapture | null>;
export interface ManualRecordingContext {
  host: ApplicationHost;
  project: ProjectView;
  generation: number;
  busy: boolean;
  beforeCapture(): Promise<number | null>;
  onRecord(generation: number, token: string, name: string): Promise<void>;
  onMerge(generation: number, token: string, changed: boolean): Promise<void>;
}

export interface ManualMergeSummary {
  sceneId: string;
  sceneName: string;
  added: number;
  replaced: number;
  unchanged: number;
  preserved: number;
  effects: number;
  rows: {
    fixtureId: string;
    attribute: string;
    change: "added" | "replaced" | "unchanged";
    previousMode: string;
    previousValue: number | null;
    previousPreset: string | null;
    effectNames: string[];
  }[];
}
