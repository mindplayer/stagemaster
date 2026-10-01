export function sameFixtureSelection(
  a: readonly string[],
  b: readonly string[],
) {
  return a.length === b.length && a.every((id, index) => id === b[index]);
}
