import { useState } from "react";
import type { AudioEdit, AudioTimeline } from "../../audio-types";
import type { ProjectView } from "../../application-host";
import { clipDraft, nextClipGap, type ClipDraft } from "./audio-clip-draft";
export function useAudioClipActions({
  track,
  selected,
  position,
  scenes,
  beforeChange,
  edit,
  onDraft,
  onSelect,
  onProblem,
  readPosition,
}: {
  readPosition(): Promise<number>;
  track: AudioTimeline | null;
  selected: string;
  position: number;
  scenes: ProjectView["scenes"];
  beforeChange(): Promise<boolean>;
  edit(command: AudioEdit): Promise<ProjectView | null>;
  onDraft(value: ClipDraft): void;
  onSelect(id: string): void;
  onProblem(value: string): void;
}) {
  const [removing, setRemoving] = useState<{ id: string; name: string } | null>(
    null,
  );
  const clip = track?.lightingClips?.find((c) => c.id === selected);
  async function add() {
    if (!track || !(await beforeChange())) return;
    const range = nextClipGap(track, position);
    if (!range) {
      onProblem("播放头之后没有可用空隙，请先缩短片段或把播放头移到空隙");
      return;
    }
    onSelect("new-clip");
    onDraft({
      kind: "clip",
      id: null,
      copy: false,
      name: `灯光片段 ${(track.lightingClips?.length ?? 0) + 1}`,
      sceneId: scenes[0]?.id ?? "",
      start: (range.startMs / 1000).toFixed(3),
      end: (range.endMs / 1000).toFixed(3),
      fade: "0",
    });
  }
  function copy() {
    if (!track || !clip) return;
    const range = nextClipGap(track, clip.endMs);
    onDraft({
      ...clipDraft(clip),
      copy: true,
      start: ((range?.startMs ?? position) / 1000).toFixed(3),
    });
  }
  async function convert() {
    if (!(await beforeChange())) return;
    if (await edit({ kind: "convertLightingClips" })) onSelect("");
  }
  async function lock() {
    if (clip && (await beforeChange()))
      await edit({
        kind: "setLightingClipLock",
        id: clip.id,
        locked: !clip.locked,
      });
  }
  async function enabled() {
    if (clip && (await beforeChange()))
      await edit({
        kind: "editLightingClips",
        ids: [clip.id],
        action: { kind: "enabled", enabled: clip.enabled === false },
      });
  }
  async function split(timeMs: number) {
    if (!clip || !(await beforeChange())) return false;
    const previous = new Set(track?.lightingClips?.map((c) => c.id));
    const next = await edit({ kind: "splitLightingClip", id: clip.id, timeMs });
    const right = next?.audio?.lightingClips?.find((c) => !previous.has(c.id));
    if (right) onSelect(right.id);
    return !!right;
  }
  async function resetOffset() {
    if (!clip || !(await beforeChange())) return false;
    return !!(await edit({
      kind: "resetLightingClipEffectOffset",
      id: clip.id,
    }));
  }
  async function remove() {
    if (!removing) return;
    if (await edit({ kind: "removeLightingClip", id: removing.id })) {
      setRemoving(null);
      onSelect("");
    }
  }
  return {
    splitActions: { split, resetOffset, readPosition },
    clip,
    add,
    copy,
    convert,
    lock,
    enabled,
    removing,
    requestRemove: () => clip && setRemoving({ id: clip.id, name: clip.name }),
    cancelRemove: () => setRemoving(null),
    remove,
  };
}
