import type { AudioEdit, AudioTimeline } from "../../audio-types.ts";
import type {
  AudioLoopRegion,
  AudioLoopGroupAction,
} from "../../audio-performance-types.ts";
import { audioMilliseconds } from "../../audio-tools.ts";
import { AudioDraftError } from "./audio-draft-error.ts";
import { sameLoopRegion } from "./performance-loop-draft.ts";
export function guardLoopGroup(
  track: AudioTimeline,
  original: AudioLoopRegion[],
) {
  if (
    original.some((r) => {
      const current = track.loopRegions?.find((next) => next.id === r.id);
      return !current || !sameLoopRegion(r, current);
    })
  )
    throw new AudioDraftError(
      "所选区段已变化，请取消输入后重新编辑",
      "loopDestination",
    );
}
export type LoopGroupOperation =
  "move" | "copy" | "remove" | "enable" | "disable" | "lock" | "unlock";
export function loopGroupSelection(
  regions: AudioLoopRegion[],
  ids: string[],
  visible = regions,
) {
  const chosen = new Set(ids),
    shown = new Set(visible.map((r) => r.id));
  const items = regions.filter((r) => chosen.has(r.id));
  return {
    items,
    first: items[0]?.startMs ?? 0,
    last: items.at(-1)?.endMs ?? 0,
    hidden: items.filter((r) => !shown.has(r.id)).length,
    locked: items.filter((r) => r.locked).length,
    inactive: items.filter((r) => !r.enabled).length,
  };
}
export function loopGroupCommand(
  track: AudioTimeline,
  ids: string[],
  kind: LoopGroupOperation,
  value = "",
): AudioEdit {
  try {
    const { items, locked } = loopGroupSelection(track.loopRegions ?? [], ids);
    if (
      !ids.length ||
      ids.length > 128 ||
      new Set(ids).size !== ids.length ||
      items.length !== ids.length
    )
      throw new Error("请重新选择循环区段");
    if (locked && !["copy", "lock", "unlock"].includes(kind))
      throw new Error("选中区段包含锁定项，整组未修改");
    const action: AudioLoopGroupAction =
      kind === "move" || kind === "copy"
        ? { kind, destinationMs: audioMilliseconds(value, "区段组目标起点") }
        : kind === "lock" || kind === "unlock"
          ? { kind: "locked", locked: kind === "lock" }
          : kind === "enable" || kind === "disable"
            ? { kind: "enabled", enabled: kind === "enable" }
            : { kind: "remove" };
    return {
      kind: "loopRegions",
      command: { kind: "edit", ids: items.map((r) => r.id), action },
    };
  } catch (error) {
    throw new AudioDraftError(
      error instanceof Error ? error.message : String(error),
      "loopDestination",
    );
  }
}
