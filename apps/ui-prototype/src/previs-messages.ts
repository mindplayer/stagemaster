import { readPrevisTargets, type PrevisTarget } from "./previs-objects.ts";
import type {
  PrevisPlacement,
  PrevisTranslation,
  PrevisObjectTranslation,
  PrevisTransform,
  PrevisTool,
  MarqueeMode,
} from "./previs-types.ts";
import type { SpatialVector3 } from "./stage-types.ts";

type ViewMessage =
  | {
      kind: "state";
      status: string;
      selection: string;
      workLight: string;
      move: boolean;
      cutaway: boolean;
      interactionVersion: number;
      marqueeSupported: boolean;
      allObjects: boolean;
      marqueeMode: MarqueeMode;
      selectionThrough: boolean;
      vertical: boolean;
      tool: PrevisTool;
    }
  | { kind: "selection"; fixtureId: string }
  | { kind: "selectionGroup"; fixtureIds: string[] }
  | { kind: "selectionTargets"; targets: PrevisTarget[] }
  | ({ kind: "objectTranslation"; requestId: string } & PrevisObjectTranslation)
  | ({ kind: "transform"; requestId: string } & PrevisTransform)
  | ({ kind: "translation"; requestId: string } & PrevisTranslation)
  | ({ kind: "placement"; requestId: string } & PrevisPlacement);
const record = (v: unknown): v is Record<string, unknown> =>
  !!v && typeof v === "object" && !Array.isArray(v);
const text = (v: unknown, max = 256): v is string =>
  typeof v === "string" && v.length <= max;
const vector = (v: unknown): v is SpatialVector3 =>
  record(v) &&
  Object.keys(v).length === 3 &&
  [v.x, v.y, v.z].every(
    (n) =>
      text(n, 32) &&
      /^-?\d+(\.\d{1,6})?$/.test(n) &&
      Number.isFinite(Number(n)),
  );
const identities = (v: unknown, max: number): v is string[] =>
  Array.isArray(v) &&
  v.length <= max &&
  v.every((id) => text(id) && !!id) &&
  new Set(v).size === v.length;

/** Renderer messages are untrusted proposals; they never bypass Rust validation. */
export function readPrevisMessage(json: string): ViewMessage | null {
  if (json.length > 65536) return null;
  try {
    const v: unknown = JSON.parse(json);
    if (!record(v)) return null;
    if (
      v.kind === "state" &&
      text(v.status, 4096) &&
      text(v.selection, 1024) &&
      text(v.workLight) &&
      typeof v.move === "boolean" &&
      (v.allObjects === undefined || typeof v.allObjects === "boolean") &&
      (v.cutaway === undefined || typeof v.cutaway === "boolean") &&
      (v.tool === undefined ||
        (typeof v.tool === "string" &&
          ["horizontal", "vertical", "rotate", "scale"].includes(v.tool))) &&
      (v.marqueeMode === undefined ||
        (typeof v.marqueeMode === "string" &&
          ["replace", "add", "remove"].includes(v.marqueeMode))) &&
      (v.marqueeSupported === undefined ||
        typeof v.marqueeSupported === "boolean") &&
      (v.selectionThrough === undefined ||
        typeof v.selectionThrough === "boolean") &&
      (v.vertical === undefined || typeof v.vertical === "boolean") &&
      (v.interactionVersion === undefined ||
        Number.isSafeInteger(v.interactionVersion))
    )
      return {
        kind: "state",
        status: v.status,
        selection: v.selection,
        workLight: v.workLight,
        move: v.move,
        cutaway: v.cutaway === true,
        interactionVersion: Number(v.interactionVersion ?? 1),
        marqueeSupported: v.marqueeSupported === true,
        allObjects: v.allObjects === true,
        marqueeMode: (v.marqueeMode ?? "replace") as MarqueeMode,
        selectionThrough: v.selectionThrough === true,
        vertical: v.vertical === true,
        tool: (v.tool ??
          (v.vertical ? "vertical" : "horizontal")) as PrevisTool,
      };
    if (v.kind === "selectionTargets" && Object.keys(v).length === 2) {
      const targets = readPrevisTargets(v.targets);
      return targets ? { kind: "selectionTargets", targets } : null;
    }
    if (v.kind === "selection" && text(v.fixtureId))
      return { kind: "selection", fixtureId: v.fixtureId };
    if (
      v.kind === "selectionGroup" &&
      Object.keys(v).length === 2 &&
      identities(v.fixtureIds, 1024)
    )
      return { kind: "selectionGroup", fixtureIds: v.fixtureIds };
    if (
      !["placement", "translation", "transform", "objectTranslation"].includes(
        String(v.kind),
      ) ||
      !text(v.requestId, 36) ||
      !/^[a-f0-9]{32}$/.test(v.requestId) ||
      !Number.isInteger(v.generation) ||
      Number(v.generation) < 0 ||
      Number(v.generation) > 0xffffffff ||
      !text(v.version, 20) ||
      !/^(0|[1-9]\d*)$/.test(v.version) ||
      BigInt(v.version) > 18446744073709551615n
    )
      return null;
    if (v.kind === "objectTranslation") {
      const targets = readPrevisTargets(v.targets, 256);
      if (
        Object.keys(v).length !== 6 ||
        !targets?.length ||
        !vector(v.deltaMeters) ||
        Object.values(v.deltaMeters).some((n) => Math.abs(Number(n)) > 200000)
      )
        return null;
      return {
        kind: "objectTranslation",
        requestId: v.requestId,
        generation: Number(v.generation),
        version: v.version,
        targets,
        deltaMeters: v.deltaMeters,
      };
    }
    if (v.kind === "transform") {
      const scalar = (n: unknown, min: number, max: number) =>
        text(n, 32) &&
        /^-?\d+(\.\d{1,6})?$/.test(n) &&
        Number(n) >= min &&
        Number(n) <= max;
      if (
        Object.keys(v).length !== 7 ||
        !identities(v.fixtureIds, 256) ||
        !v.fixtureIds.length ||
        !scalar(v.yawDegrees, -360, 360) ||
        !scalar(v.spacingScale, 0.01, 100)
      )
        return null;
      return {
        kind: "transform",
        requestId: v.requestId,
        generation: Number(v.generation),
        version: v.version,
        fixtureIds: v.fixtureIds,
        yawDegrees: v.yawDegrees as string,
        spacingScale: v.spacingScale as string,
      };
    }
    if (v.kind === "translation") {
      if (
        Object.keys(v).length !== 6 ||
        !identities(v.fixtureIds, 256) ||
        !v.fixtureIds.length ||
        !vector(v.deltaMeters) ||
        Object.values(v.deltaMeters).some((n) => Math.abs(Number(n)) > 200000)
      )
        return null;
      return {
        kind: "translation",
        requestId: v.requestId,
        generation: Number(v.generation),
        version: v.version,
        fixtureIds: v.fixtureIds,
        deltaMeters: v.deltaMeters,
      };
    }
    const p = v.placement;
    if (
      !record(p) ||
      Object.keys(p).length !== 4 ||
      !text(p.fixtureId) ||
      !p.fixtureId ||
      !(p.spaceId === null || text(p.spaceId)) ||
      !vector(p.positionMeters) ||
      !vector(p.rotationDegreesXYZ)
    )
      return null;
    return {
      kind: "placement",
      requestId: v.requestId,
      generation: Number(v.generation),
      version: v.version,
      placement: {
        fixtureId: p.fixtureId,
        spaceId: p.spaceId,
        positionMeters: p.positionMeters,
        rotationDegreesXYZ: p.rotationDegreesXYZ,
      },
    };
  } catch {
    return null;
  }
}
