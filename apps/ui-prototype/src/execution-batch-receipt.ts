import type { ExecutionStatus, ExecutionView } from "./execution-types";
import type { BatchReview } from "./execution-batch";
import { newLevelSerial } from "./execution-level-context.ts";

export function batchReply(
  review: BatchReview,
  before: string | undefined,
  result: ExecutionStatus | undefined,
) {
  const next = result?.runtime;
  return next &&
    result.phase === "connected" &&
    next.hostId === review.request.hostId &&
    next.sessionId === review.session &&
    next.catalog.layout === review.layout &&
    next.catalog.projectId === review.project &&
    newLevelSerial(next.record?.serial, before)
    ? next.record
    : null;
}
export function batchReceiptText(record: NonNullable<ExecutionView["record"]>) {
  if (record.status !== "complete")
    return "批量操作结果待确认；只查询原回执，不重复发送。";
  if (record.outcome?.kind === "applied")
    return "批量操作已确认；实际状态见各节目。";
  return `批量操作未确认生效：${record.outcome?.message || record.outcome?.kind || "结果未知"}`;
}
