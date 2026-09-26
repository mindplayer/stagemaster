import type { SceneView } from "./application-host";
export interface GroupView {
  id: string;
  name: string;
  fixtureIds: string[];
}
export interface PresetView {
  id: string;
  name: string;
  values: SceneView["values"];
  usedByScenes: { id: string; name: string }[];
  usedBySequences: { id: string; name: string }[];
}
export type ResourceKind = "group" | "preset";
export type PresetUpdate = "existing" | "merge" | "replace";
export interface CaptureScope {
  sceneId: string;
  fixtureIds: string[];
  attributes: string[];
}
export type LibraryEdit =
  | { kind: "saveGroup"; id: string | null; name: string; fixtureIds: string[] }
  | { kind: "duplicate"; resource: ResourceKind; id: string; name: string }
  | { kind: "remove"; resource: ResourceKind; id: string; keepValues: boolean }
  | { kind: "renamePreset"; id: string; name: string }
  | ({ kind: "recordPreset"; name: string } & CaptureScope)
  | ({ kind: "updatePreset"; id: string; mode: PresetUpdate } & CaptureScope)
  | ({ kind: "applyPreset"; id: string; linked: boolean } & CaptureScope)
  | ({ kind: "detach" } & CaptureScope)
  | ({ kind: "copyValues"; sourceId: string } & CaptureScope);
