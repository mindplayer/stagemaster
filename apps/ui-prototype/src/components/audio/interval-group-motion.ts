export interface TimedInterval {
  id: string;
  name: string;
  startMs: number;
  endMs: number;
  locked?: boolean;
}
/** Rendering geometry only. Sorted, disjoint half-open intervals; the host validates edits. */
export function moveIntervalGroup<T extends TimedInterval>(
  items: T[],
  duration: number,
  ids: string[],
  requested: number,
  targets: number[],
  tolerance: number,
  noun: string,
) {
  const chosen = new Set(ids);
  const source = items.filter((c) => chosen.has(c.id));
  const others = items.filter((c) => !chosen.has(c.id));
  const first = source[0]?.startMs ?? 0,
    last = source.at(-1)?.endMs ?? 0;
  const result = {
    ids: source.map((c) => c.id),
    items: source,
    delta: 0,
    destination: first,
    problem: "",
  };
  if (
    !source.length ||
    source.length !== ids.length ||
    !Number.isFinite(requested)
  )
    return { ...result, problem: `请重新选择需要移动的${noun}` };
  const locked = source.find((c) => c.locked);
  if (locked)
    return { ...result, problem: `“${locked.name}”已锁定，整组未移动` };
  const min = -first,
    max = duration - last;
  let delta = Math.max(min, Math.min(max, Math.round(requested)));
  const collision = (offset: number) => {
    let i = 0;
    for (const c of source) {
      const start = c.startMs + offset,
        end = c.endMs + offset;
      while (others[i] && others[i].endMs <= start) i++;
      if (others[i] && others[i].startMs < end) return others[i];
    }
    return undefined;
  };
  if (tolerance > 0) {
    const candidates = [
      ...new Set(targets.flatMap((t) => [t - first, t - last])),
    ]
      .filter((d) => d >= min && d <= max && Math.abs(d - delta) <= tolerance)
      .sort((a, b) => Math.abs(a - delta) - Math.abs(b - delta) || a - b);
    delta = candidates.find((d) => !collision(d)) ?? delta;
  }
  const conflict = collision(delta);
  return {
    ids: result.ids,
    items: source.map((c) => ({
      ...c,
      startMs: c.startMs + delta,
      endMs: c.endMs + delta,
    })),
    delta,
    destination: first + delta,
    problem: conflict ? `目标与“${conflict.name}”重叠，整组未移动` : "",
  };
}
