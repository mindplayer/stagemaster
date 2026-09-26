import type { PrevisPlacement } from "./previs-types.ts";
import type { SpatialVector3 } from "./stage-types.ts";

type ViewMessage =
  | { kind: "state"; status: string; selection: string; workLight: string; move: boolean }
  | { kind: "selection"; fixtureId: string }
  | ({ kind: "placement"; requestId: string } & PrevisPlacement);
const record = (v: unknown): v is Record<string, unknown> => !!v && typeof v === "object" && !Array.isArray(v);
const text = (v: unknown, max = 256): v is string => typeof v === "string" && v.length <= max;
const vector = (v: unknown): v is SpatialVector3 => record(v) && Object.keys(v).length === 3 &&
  [v.x, v.y, v.z].every(n => text(n, 32) && /^-?\d+(\.\d{1,6})?$/.test(n) && Number.isFinite(Number(n)));

/** Renderer messages are untrusted proposals; they never bypass Rust validation. */
export function readPrevisMessage(json: string): ViewMessage | null {
  if (json.length > 8192) return null;
  try {
    const v: unknown = JSON.parse(json);
    if (!record(v)) return null;
    if (v.kind === "state" && text(v.status, 4096) && text(v.selection, 1024) && text(v.workLight) && typeof v.move === "boolean")
      return { kind: "state", status: v.status, selection: v.selection, workLight: v.workLight, move: v.move };
    if (v.kind === "selection" && text(v.fixtureId)) return { kind: "selection", fixtureId: v.fixtureId };
    if (v.kind !== "placement" || !text(v.requestId, 36) || !/^[a-f0-9]{32}$/.test(v.requestId) ||
      !Number.isInteger(v.generation) || Number(v.generation) < 0 || Number(v.generation) > 0xffffffff ||
      !text(v.version, 20) || !/^(0|[1-9]\d*)$/.test(v.version) || BigInt(v.version) > 18446744073709551615n) return null;
    const p = v.placement;
    if (!record(p) || Object.keys(p).length !== 4 || !text(p.fixtureId) || !p.fixtureId ||
      !(p.spaceId === null || text(p.spaceId)) || !vector(p.positionMeters) || !vector(p.rotationDegreesXYZ)) return null;
    return { kind: "placement", requestId: v.requestId, generation: Number(v.generation), version: v.version,
      placement: { fixtureId: p.fixtureId, spaceId: p.spaceId, positionMeters: p.positionMeters, rotationDegreesXYZ: p.rotationDegreesXYZ } };
  } catch { return null; }
}
