import type { AudioLightingClip } from "../../audio-types";
import { useOrderedSelection } from "../selection/useOrderedSelection";
export function useClipSelection(
  identity: string,
  clips: AudioLightingClip[] | undefined,
) {
  return useOrderedSelection(identity, clips);
}
