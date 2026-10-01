import type { ExecutionPosition } from "./execution-position";

export function navigationTargets(
  position: ExecutionPosition,
  sequenceId: string,
  stepIds: readonly string[],
) {
  const ready = position.sequenceId === sequenceId && !position.stale;
  const available = (id: string | null) =>
    ready && id && stepIds.includes(id) ? id : null;
  return {
    current: available(position.currentId),
    next: available(position.nextId),
  };
}

export { rangeScrollTop as stepScrollTop } from "../layout/scroll-range.ts";
