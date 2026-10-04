import type { ExecutionStatus, ExecutionView } from "./execution-types";

export interface LevelContext {
  status: ExecutionStatus | null;
  available: boolean;
  busy: boolean;
}
export interface LevelIdentity {
  host: string;
  layout: string;
  session: string;
  source: string;
}
export function levelRuntime(status: ExecutionStatus | null | undefined) {
  const runtime = status?.runtime;
  const state = runtime?.observation.snapshot?.state;
  return status?.phase === "connected" &&
    !status.problem &&
    runtime?.controlling &&
    runtime.sessionId &&
    runtime.observation.phase === "running" &&
    !runtime.observation.fault &&
    state &&
    !state.fault &&
    state.owner?.sessionId === runtime.sessionId
    ? runtime
    : null;
}
export function sameLevelSession(
  identity: LevelIdentity,
  runtime: ExecutionView | null,
) {
  return (
    !!runtime &&
    runtime.hostId === identity.host &&
    runtime.catalog.layout === identity.layout &&
    runtime.sessionId === identity.session &&
    runtime.catalog.sources.some(
      (s) => s.id === identity.source && s.selection.kind !== "audioTimeline",
    )
  );
}
export function sourceLevel(runtime: ExecutionView, source: string) {
  return runtime.observation.snapshot?.state.sources.find(
    (s) => s.id === source,
  )?.level;
}
export const levelPercent = (value: number) => Math.round(value / 65.535) / 10;

export function newLevelSerial(
  serial: string | undefined,
  previous: string | undefined,
) {
  return (
    !!serial &&
    /^[1-9][0-9]*$/.test(serial) &&
    (!previous ||
      (/^[1-9][0-9]*$/.test(previous) && BigInt(serial) > BigInt(previous)))
  );
}
