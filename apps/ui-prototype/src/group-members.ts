/** Membership transforms preserve explicit order and never mutate either input. */
export function appendGroupMembers(
  ids: string[],
  additions: string[],
): string[] {
  return [...new Set([...ids, ...additions])];
}
export function removeGroupMembers(
  ids: string[],
  removals: string[],
): string[] {
  const removing = new Set(removals);
  return ids.filter((id) => !removing.has(id));
}
export function groupMembersIssue(
  ids: string[],
  available: { id: string }[],
): string {
  if (!ids.length) return "灯组至少需要一台灯具";
  if (ids.length > 10000) return "灯组最多容纳 10000 台灯具";
  const valid = new Set(available.map((f) => f.id));
  if (ids.some((id) => !valid.has(id)))
    return "灯组包含已删除的灯具，请移出后再保存";
  if (new Set(ids).size !== ids.length) return "灯组不能重复包含同一台灯具";
  return "";
}
