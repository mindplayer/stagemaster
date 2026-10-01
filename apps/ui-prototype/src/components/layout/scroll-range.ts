/** Minimal scrolling inside one viewport. Prefer current + next when both fit. */
export function rangeScrollTop(
  scrollTop: number,
  viewportHeight: number,
  scrollHeight: number,
  first: { top: number; bottom: number },
  next?: { top: number; bottom: number },
  center = false,
) {
  const height = Math.max(0, viewportHeight);
  let bottom = first.bottom;
  if (next && next.top >= first.top && next.bottom - first.top <= height - 16)
    bottom = next.bottom;
  let result = scrollTop;
  if (center) result = (first.top + first.bottom - height) / 2;
  else if (first.top < scrollTop + 8 || bottom - first.top > height - 16)
    result = first.top - 8;
  else if (bottom > scrollTop + height - 8) result = bottom - height + 8;
  return Math.max(0, Math.min(Math.max(0, scrollHeight - height), result));
}
