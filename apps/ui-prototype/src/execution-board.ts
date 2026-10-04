import type { ExecutionSource, ExecutionView } from "./execution-types";

export const sourceKinds = {
  all: "全部类型",
  scene: "场景",
  sequence: "场景列表",
  audioTimeline: "音乐",
  manual: "手动层",
} as const;
export const sourceStates = {
  all: "全部状态",
  Running: "运行中",
  Paused: "已暂停",
  Finished: "已结束",
  Idle: "待执行",
  Preparing: "准备中",
  Failed: "故障",
  Unknown: "状态未知",
} as const;
export type SourceKindFilter = keyof typeof sourceKinds;
export type SourceStateFilter = keyof typeof sourceStates;
export type BoardScope = "all" | "pinned" | "drafts";
export interface BoardFilter {
  query: string;
  kind: SourceKindFilter;
  status: SourceStateFilter;
  scope: BoardScope;
}
export const allSources: BoardFilter = {
  query: "",
  kind: "all",
  status: "all",
  scope: "all",
};
export function sourcePinKey(source: ExecutionSource): string {
  const s = source.selection;
  return "id" in s ? `${s.kind}:${s.id}` : s.kind;
}
export function sourceStatus(
  source: ExecutionSource,
  runtime: ExecutionView,
): Exclude<SourceStateFilter, "all"> | null {
  if (source.selection.kind === "manual") return null;
  const snapshot = runtime.observation.snapshot?.state;
  if (source.selection.kind === "audioTimeline") {
    const status = snapshot?.audio?.status;
    return status
      ? (
          {
            ready: "Idle",
            preparing: "Preparing",
            playing: "Running",
            paused: "Paused",
            stopped: "Idle",
            ended: "Finished",
            failed: "Failed",
          } as const
        )[status]
      : "Unknown";
  }
  const status = snapshot?.sources.find((s) => s.id === source.id)?.status;
  return status && Object.hasOwn(sourceStates, status) && status !== "all"
    ? (status as Exclude<SourceStateFilter, "all">)
    : "Unknown";
}
export function boardRows(
  runtime: ExecutionView,
  filter: BoardFilter,
  pins: readonly string[],
  drafts: ReadonlySet<string>,
) {
  const words = filter.query
    .trim()
    .toLocaleLowerCase()
    .split(/\s+/)
    .filter(Boolean);
  return runtime.catalog.sources
    .map((source, index) => {
      const status = sourceStatus(source, runtime);
      const pin = pins.indexOf(sourcePinKey(source));
      const text =
        `${source.name} ${sourceKinds[source.selection.kind]}`.toLocaleLowerCase();
      return {
        source,
        index,
        status,
        pinned: pin >= 0,
        pin,
        shown:
          words.every((word) => text.includes(word)) &&
          (filter.kind === "all" || source.selection.kind === filter.kind) &&
          (filter.status === "all" || status === filter.status) &&
          (filter.scope === "all" ||
            (filter.scope === "pinned" ? pin >= 0 : drafts.has(source.id))),
      };
    })
    .sort(
      (a, b) =>
        (a.pin < 0 ? 1000 + a.index : a.pin) -
        (b.pin < 0 ? 1000 + b.index : b.pin),
    );
}
