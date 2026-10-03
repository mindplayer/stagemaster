import type { ExecutionView } from "./execution-types";

export interface MediaRequestIdentity {
  hostId: string;
  sessionId: string;
  serial: string;
}
export function mediaRequestIdentity(
  view: ExecutionView | null | undefined,
): MediaRequestIdentity | null {
  return view?.sessionId && view.record
    ? {
        hostId: view.hostId,
        sessionId: view.sessionId,
        serial: view.record.serial,
      }
    : null;
}

export function mediaSeekReceipt(
  request: MediaRequestIdentity,
  view: ExecutionView,
  group: string,
): "waiting" | "applied" | "failed" | "superseded" {
  if (view.hostId !== request.hostId || view.sessionId !== request.sessionId)
    return "superseded";
  const record = view.record;
  if (!record || BigInt(record.serial) < BigInt(request.serial))
    return "waiting";
  if (record.serial !== request.serial) return "superseded";
  if (!record.outcome) return "waiting";
  if (record.outcome.kind !== "accepted") return "failed";
  const expected = record.outcome.state?.media?.find(
    (m) => m.id === group,
  )?.control;
  if (!expected) return "failed";
  const current = view.observation.snapshot?.state.media?.find(
    (m) => m.id === group,
  )?.control;
  if (!current || BigInt(current.request) < BigInt(expected.request))
    return "waiting";
  if (current.request !== expected.request) return "superseded";
  if (current.status === "applied") return "applied";
  return current.status === "pending" ? "waiting" : "failed";
}
