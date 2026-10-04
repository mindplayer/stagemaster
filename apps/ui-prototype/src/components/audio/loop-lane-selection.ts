import type { AudioLoopRegion } from "../../audio-performance-types";
import type { LoopMotion } from "./performance-loop-motion";
export interface LoopLaneSelection {
  active: boolean;
  ids: string[];
  blocked: boolean;
  onPick(id: string, range: boolean): void;
  onClear(): void;
  onMove(ids: string[], destinationMs: number): void;
  onEdit(
    original: AudioLoopRegion,
    next: AudioLoopRegion,
    mode: LoopMotion,
  ): void;
}
