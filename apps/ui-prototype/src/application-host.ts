import type {
  PositionEdit,
  PositionModel,
  FixtureZero,
} from "./position-types";
import type { FixtureEdit, ProfileView } from "./fixture-types";
import type { StageEdit, StageView } from "./stage-types";
import type { EffectEdit, SceneEffect } from "./effect-types";
import type {
  PrevisRequest,
  PrevisStatus,
  PrevisPlacement,
} from "./previs-types";
import type { GroupView, PresetView, LibraryEdit } from "./library-types";
import type {
  SequenceEdit,
  SequenceView,
  PreviewRequest,
  PreviewSnapshot,
} from "./sequence-types";
export type EditCommand =
  | EditOperation
  | { op: "batch"; commands: EditOperation[] };
export type EditOperation =
  | { op: "position"; command: PositionEdit }
  | { op: "fixture"; command: FixtureEdit }
  | { op: "effect"; command: EffectEdit }
  | { op: "stage"; command: StageEdit }
  | { op: "library"; command: LibraryEdit }
  | { op: "sequence"; command: SequenceEdit }
  | { op: "setInfo"; name: string; description: string }
  | {
      op: "addFixture";
      name: string;
      profileId: string;
      domainId: string;
      universe: number;
      address: number;
    }
  | {
      op: "updateFixture";
      id: string;
      name: string;
      universe: number;
      address: number;
    }
  | { op: "removeFixture"; id: string }
  | { op: "addScene"; name: string }
  | { op: "duplicateScene"; id: string; name: string }
  | { op: "renameScene"; id: string; name: string }
  | { op: "removeScene"; id: string }
  | {
      op: "setSceneValue";
      sceneId: string;
      fixtureId: string;
      attribute: string;
      mode: "literal" | "release" | "remove";
      value: number;
    };
export interface FixtureView {
  positioning?: PositionModel | null;
  zeroCorrection?: FixtureZero | null;
  profileId: string;
  id: string;
  name: string;
  profileName: string;
  domainName: string;
  domainId: string;
  footprint: number;
  universe: number | null;
  address: number | null;
  attributes: { key: string; label: string; defaultValue: number }[];
}
export interface SceneView {
  id: string;
  name: string;
  effects: SceneEffect[];
  values: {
    fixtureId: string;
    attribute: string;
    mode: string;
    value: number | null;
    presetName: string | null;
    presetId: string | null;
  }[];
}
export interface ProjectView {
  id: string;
  name: string;
  description: string;
  profiles: ProfileView[];
  domains: { id: string; name: string }[];
  fixtures: FixtureView[];
  scenes: SceneView[];
  groups: GroupView[];
  presets: PresetView[];
  sequences: SequenceView[];
  stage: StageView;
}
export interface Snapshot {
  generation: number;
  project: ProjectView | null;
  fileName: string | null;
  dirty: boolean;
  canUndo: boolean;
  canRedo: boolean;
  recovery: import("./recovery-types").RecoveryStatus;
}
export type ProjectRequest =
  | { kind: "snapshot" | "close" }
  | { kind: "new" | "open"; generation: number }
  | { kind: "recover"; generation: number; id: string; token: string }
  | { kind: "save"; generation: number; saveAs: boolean }
  | { kind: "edit"; generation: number; command: EditCommand }
  | ({ kind: "previsPlacement" } & PrevisPlacement)
  | { kind: "history"; generation: number; redo: boolean };
export interface ApplicationHost {
  kind: "desktop" | "browser";
  recovery(
    request: import("./recovery-types").RecoveryRequest,
  ): Promise<import("./recovery-types").RecoveryCatalog>;
  check(generation: number): Promise<import("./check-types").ProjectCheck>;
  request(request: ProjectRequest): Promise<Snapshot>;
  preview(request: PreviewRequest): Promise<PreviewSnapshot>;
  previs(request: PrevisRequest): Promise<PrevisStatus>;
  onCloseRequested(handler: () => void): Promise<() => void>;
}
