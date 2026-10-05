import type { ExecutionView } from "./execution-types";
import {
  mediaRequestIdentity,
  mediaSeekReceipt,
} from "./media-seek-receipt.ts";

export type MediaControlNotice =
  | "pending"
  | "applied"
  | "failed"
  | "timedOut"
  | "rejected"
  | "unknown"
  | "unconfirmed"
  | "superseded";

/** Project existing receipts without sending or retargeting any command. */
export function mediaControlNotice(
  view: ExecutionView,
  group: string,
): MediaControlNotice | null {
  if (view.pending) return null;
  const current = view.observation.snapshot?.state.media?.find(
    (m) => m.id === group,
  )?.control;
  const record = view.record;
  if (!record) return current?.status ?? null;
  const outcome = record.outcome;
  if (!outcome) return "unconfirmed";
  if (outcome.kind === "rejected" || outcome.kind === "unknown")
    return outcome.kind;
  if (outcome.kind !== "accepted") return current?.status ?? null;
  const identity = mediaRequestIdentity(view);
  const expected = outcome.state?.media?.find((m) => m.id === group)?.control;
  if (!identity || !expected) return "unconfirmed";
  const receipt = mediaSeekReceipt(identity, view, group);
  if (receipt === "waiting") return "pending";
  if (receipt !== "failed") return receipt;
  return current?.status === "failed" || current?.status === "timedOut"
    ? current.status
    : "unconfirmed";
}
