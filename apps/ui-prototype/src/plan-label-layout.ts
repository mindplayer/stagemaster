/** SVG view coordinates in symbol units. Does not move or filter fixture identities. */
export interface LabelBox {
  x: number;
  y: number;
  width: number;
  height: number;
}
export interface PlanLabelItem {
  id: string;
  x: number;
  y: number;
  width: number;
  priority: number;
  selected: boolean;
  /** False for geometry anchors without a fixture symbol. */
  symbol?: boolean;
}
export interface PlacedPlanLabel extends LabelBox {
  id: string;
}
export function boxesOverlap(a: LabelBox, b: LabelBox): boolean {
  return (
    a.x < b.x + b.width &&
    a.x + a.width > b.x &&
    a.y < b.y + b.height &&
    a.y + a.height > b.y
  );
}
/** Bounded boxes, eight candidate positions, no iterative simulation. */
class CollisionGrid {
  private buckets = new Map<string, LabelBox[]>();
  private keys(b: LabelBox): string[] {
    const keys: string[] = [];
    for (let x = Math.floor(b.x / 8); x <= Math.floor((b.x + b.width) / 8); x++)
      for (
        let y = Math.floor(b.y / 8);
        y <= Math.floor((b.y + b.height) / 8);
        y++
      )
        keys.push(`${x}:${y}`);
    return keys;
  }
  add(b: LabelBox) {
    for (const k of this.keys(b)) {
      const list = this.buckets.get(k) ?? [];
      list.push(b);
      this.buckets.set(k, list);
    }
  }
  hits(b: LabelBox) {
    return this.keys(b).some((k) =>
      this.buckets.get(k)?.some((other) => boxesOverlap(b, other)),
    );
  }
}
const within = (b: LabelBox, v: LabelBox) =>
  b.x >= v.x &&
  b.y >= v.y &&
  b.x + b.width <= v.x + v.width &&
  b.y + b.height <= v.y + v.height;
export function layoutPlanLabels(
  items: readonly PlanLabelItem[],
  viewport?: LabelBox,
) {
  const grid = new CollisionGrid(),
    placed: PlacedPlanLabel[] = [],
    hidden: string[] = [];
  const valid = items.filter(
    (i) =>
      [i.x, i.y, i.width, i.priority].every(Number.isFinite) &&
      Math.abs(i.x) <= 1e9 &&
      Math.abs(i.y) <= 1e9 &&
      i.width > 0 &&
      i.width <= 20,
  );
  const visible = valid.filter(
    (i) =>
      !viewport ||
      boxesOverlap(
        { x: i.x - 1.3, y: i.y - 1.3, width: 2.6, height: 2.6 },
        viewport,
      ),
  );
  for (const i of visible.filter((i) => i.symbol !== false))
    grid.add({
      x: i.x - 1.3,
      y: i.y - (i.selected ? 2.2 : 1.3),
      width: 2.6,
      height: i.selected ? 3.5 : 2.6,
    });
  for (const i of [...visible].sort(
    (a, b) =>
      a.priority - b.priority || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0),
  )) {
    const w = i.width + 0.8,
      h = 1.5;
    const positions = [
      [i.x - w / 2, i.y + 1.7],
      [i.x - w / 2, i.y - (i.selected ? 2.6 : 1.7) - h],
      [i.x + 1.7, i.y - h / 2],
      [i.x - 1.7 - w, i.y - h / 2],
      [i.x + 1.7, i.y + 1.7],
      [i.x - 1.7 - w, i.y + 1.7],
      [i.x + 1.7, i.y - 1.7 - h],
      [i.x - 1.7 - w, i.y - 1.7 - h],
    ];
    const b = positions
      .map(([x, y]) => ({ x, y, width: w, height: h }))
      .find((b) => (!viewport || within(b, viewport)) && !grid.hits(b));
    if (b) {
      grid.add(b);
      placed.push({ id: i.id, ...b });
    } else hidden.push(i.id);
  }
  return { placed, hidden };
}
