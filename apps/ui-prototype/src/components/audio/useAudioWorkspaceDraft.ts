import { useEffect, useRef, useState } from "react";
import type { AudioTimeline } from "../../audio-types";
import type { EditOperation } from "../../application-host";
import {
  AudioDraftError,
  collectAudioDraft,
  type AudioDraft,
} from "./audio-inspector-draft";
export function useAudioWorkspaceDraft(
  track: AudioTimeline | null,
  onPending: (value: boolean) => void,
  onProblem: (value: string) => void,
  onSelect: (id: string) => void,
) {
  const [draft, setDraft] = useState<AudioDraft | null>(null);
  const draftRef = useRef<AudioDraft | null>(null);
  const form = useRef<HTMLFormElement>(null);
  const created = useRef<{ kind: "clip" | "loop"; ids: string[] } | null>(null);
  useEffect(() => {
    if (!created.current || draft) return;
    const items = created.current.kind === "clip" ? track?.lightingClips : track?.loopRegions;
    const next = items?.find((c) => !created.current!.ids.includes(c.id));
    if (next) onSelect(next.id);
    created.current = null;
  }, [track, draft, onSelect]);
  function change(value: AudioDraft) {
    draftRef.current = value;
    setDraft(value);
    onPending(true);
    onProblem("");
  }
  function cancel() {
    draftRef.current = null;
    setDraft(null);
    onPending(false);
    onProblem("");
  }
  function collect(): EditOperation[] {
    const value = draftRef.current;
    if (!value || !track) return [];
    try {
      if (!form.current?.reportValidity())
        throw new Error("请修正音频属性中的输入");
      const command = collectAudioDraft(value, track);
      if (
        command.kind === "addLightingClip" ||
        command.kind === "copyLightingClip"
      )
        created.current = {
          kind: "clip", ids: track.lightingClips?.map((c) => c.id) ?? [],
        };
      if (command.kind === "loopRegions" && command.command.kind === "add")
        created.current = {
          kind: "loop", ids: track.loopRegions?.map((r) => r.id) ?? [],
        };
      return [{ op: "audio", command }];
    } catch (error) {
      onProblem(error instanceof Error ? error.message : String(error));
      const field =
        error instanceof AudioDraftError
          ? error.field
          : value.kind === "performanceLoop"
            ? "loopName"
            : value.kind === "clipGroupFade"
            ? "clipGroupFade"
            : value.kind === "marker"
              ? "markerName"
              : value.kind === "clip"
                ? "clipStart"
                : "trimStart";
      requestAnimationFrame(() =>
        form.current
          ?.querySelector<HTMLInputElement>(`[name="${field}"]`)
          ?.focus(),
      );
      throw error;
    }
  }
  return { draft, form, change, cancel, collect };
}
