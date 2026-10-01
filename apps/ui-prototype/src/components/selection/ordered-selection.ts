export interface Identified {
  id: string;
}
export interface OrderedSelection<T extends Identified> {
  ids: string[];
  replace(ids: string[]): void;
  toggle(id: string, visible: T[], range: boolean): void;
}
export function currentOrderedIds(items: Identified[], ids: string[]) {
  const chosen = new Set(ids);
  return items.filter((item) => chosen.has(item.id)).map((item) => item.id);
}
export function toggleOrderedRange(
  ids: string[],
  visible: Identified[],
  id: string,
  anchor: string | null,
  range: boolean,
) {
  const from = visible.findIndex((item) => item.id === anchor);
  const to = visible.findIndex((item) => item.id === id);
  if (to < 0) return ids;
  if (range && from >= 0)
    return [
      ...new Set([
        ...ids,
        ...visible
          .slice(Math.min(from, to), Math.max(from, to) + 1)
          .map((item) => item.id),
      ]),
    ];
  return ids.includes(id) ? ids.filter((value) => value !== id) : [...ids, id];
}
