/** Grapheme-safe display abbreviation; the original name remains in object metadata. */
export function fitPlanLabel(
  text: string,
  measure: (s: string) => number,
  max = 12,
) {
  const width = measure(text);
  if (width <= max) return { text, width };
  const graphemes = [
    ...new Intl.Segmenter(undefined, { granularity: "grapheme" }).segment(text),
  ].map((s) => s.segment);
  let shown = "";
  for (const g of graphemes) {
    if (measure(shown + g + "…") > max) break;
    shown += g;
  }
  shown += "…";
  return { text: shown, width: Math.min(max, measure(shown)) };
}
