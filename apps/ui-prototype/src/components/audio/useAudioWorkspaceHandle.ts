import { useImperativeHandle, useState, type ForwardedRef } from "react";
import type { AudioTimeline } from "../../audio-types";
import type { EditOperation } from "../../application-host";
import { useRevealItem, type RevealItem } from "../layout/useRevealItem";

export interface AudioHandle {
  collect(): EditOperation[];
  accept(): void;
  reveal(kind: "clip" | "marker", id: string): boolean;
}

/** View navigation only; the existing draft transaction remains in its owner. */
export function useAudioWorkspaceHandle(
  ref: ForwardedRef<AudioHandle>,
  props: {
    track: AudioTimeline | null;
    visible: boolean;
    blocked: boolean;
    collect(): EditOperation[];
    cancel(): void;
    select(id: string): void;
    clearBatch(): void;
    clearQuery(): void;
  },
) {
  const [revealRequest, setRevealRequest] = useState<RevealItem | null>(null);
  const root = useRevealItem(revealRequest, props.visible, props.blocked);
  useImperativeHandle(ref, () => ({
    collect: props.collect,
    accept: props.cancel,
    reveal(kind, id) {
      const items =
        kind === "clip" ? props.track?.lightingClips : props.track?.markers;
      if (!items?.some((item) => item.id === id)) return false;
      props.cancel();
      props.select(id);
      props.clearBatch();
      props.clearQuery();
      setRevealRequest((previous) => ({
        id,
        serial: (previous?.serial ?? 0) + 1,
      }));
      return true;
    },
  }));
  return { revealRequest, root };
}
