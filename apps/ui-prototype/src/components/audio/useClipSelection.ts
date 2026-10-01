import { useEffect, useState } from "react";
import type { AudioLightingClip } from "../../audio-types";
import { currentClipIds, type ClipSelection } from "./clip-selection";
import { toggleClipRange } from "./clip-group-tools";
/** One selection owner shared by library and timeline; not persisted in the show. */
export function useClipSelection(
  identity: string,
  clips: AudioLightingClip[] | undefined,
): ClipSelection {
  const [state, setState] = useState({
    identity,
    ids: [] as string[],
    anchor: null as string | null,
  });
  const items = clips ?? [];
  const ids =
    state.identity === identity ? currentClipIds(items, state.ids) : [];
  useEffect(() => {
    setState((s) => {
      if (s.identity !== identity) return { identity, ids: [], anchor: null };
      const next = currentClipIds(items, s.ids);
      if (
        next.length === s.ids.length &&
        next.every((id, i) => id === s.ids[i])
      )
        return s;
      return {
        ...s,
        ids: next,
        anchor: next.includes(s.anchor ?? "") ? s.anchor : null,
      };
    });
  }, [identity, clips]);
  return {
    ids,
    replace(next) {
      setState({ identity, ids: [...new Set(next)], anchor: null });
    },
    toggle(id, visible, range) {
      setState((s) => {
        const current =
          s.identity === identity ? currentClipIds(items, s.ids) : [];
        const anchor = s.identity === identity ? s.anchor : null;
        return {
          identity,
          ids: currentClipIds(
            items,
            toggleClipRange(current, visible, id, anchor, range),
          ),
          anchor: id,
        };
      });
    },
  };
}
