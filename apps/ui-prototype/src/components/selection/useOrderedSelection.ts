import { useEffect, useState } from "react";
import {
  currentOrderedIds,
  toggleOrderedRange,
  type Identified,
  type OrderedSelection,
} from "./ordered-selection";
/** Local identity-scoped selection; shared by list and time views, never persisted. */
export function useOrderedSelection<T extends Identified>(
  identity: string,
  items: T[] | undefined,
): OrderedSelection<T> {
  const [state, setState] = useState({
    identity,
    ids: [] as string[],
    anchor: null as string | null,
  });
  const current = items ?? [];
  const ids =
    state.identity === identity ? currentOrderedIds(current, state.ids) : [];
  useEffect(() => {
    setState((s) => {
      if (s.identity !== identity) return { identity, ids: [], anchor: null };
      const next = currentOrderedIds(current, s.ids);
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
  }, [identity, items]);
  return {
    ids,
    replace(next) {
      setState({ identity, ids: [...new Set(next)], anchor: null });
    },
    toggle(id, visible, range) {
      setState((s) => ({
        identity,
        ids: currentOrderedIds(
          current,
          toggleOrderedRange(
            s.identity === identity ? currentOrderedIds(current, s.ids) : [],
            visible,
            id,
            s.identity === identity ? s.anchor : null,
            range,
          ),
        ),
        anchor: id,
      }));
    },
  };
}
