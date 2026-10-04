export interface ExecutionPins {
  projectId: string;
  sources: string[];
}
export const executionPinLimit = 16;
const validProject = (v: unknown): v is string =>
  typeof v === "string" && v.length > 0 && v.length <= 128;
const validKey = (v: unknown): v is string =>
  typeof v === "string" &&
  /^(scene:.{1,128}|sequence:.{1,128}|audioTimeline|manual)$/.test(v);
export function readExecutionPins(raw: string | null): ExecutionPins[] {
  if (!raw || raw.length > 65536) return [];
  try {
    const data = JSON.parse(raw);
    if (data?.version !== 1 || !Array.isArray(data.projects)) return [];
    const result: ExecutionPins[] = [];
    for (const item of data.projects.slice(0, 20)) {
      if (
        !item ||
        !validProject(item.projectId) ||
        result.some((p) => p.projectId === item.projectId)
      )
        continue;
      result.push({
        projectId: item.projectId,
        sources: Array.isArray(item.sources)
          ? [
              ...new Set<string>(
                item.sources.slice(0, executionPinLimit).filter(validKey),
              ),
            ]
          : [],
      });
    }
    return result;
  } catch {
    return [];
  }
}
export function editExecutionPins(
  projects: readonly ExecutionPins[],
  projectId: string,
  available: readonly string[],
  action:
    { kind: "toggle" | "earlier" | "later"; key: string } | { kind: "prune" },
): ExecutionPins[] {
  if (!validProject(projectId)) throw new Error("后台工程身份不可用");
  let sources = [
    ...(projects.find((p) => p.projectId === projectId)?.sources ?? []),
  ];
  if (action.kind === "prune")
    sources = sources.filter((k) => available.includes(k));
  else {
    if (!validKey(action.key) || !available.includes(action.key))
      throw new Error("节目已不在当前后台目录中");
    const index = sources.indexOf(action.key);
    if (action.kind === "toggle") {
      if (index >= 0) sources.splice(index, 1);
      else {
        if (sources.length >= executionPinLimit)
          throw new Error(
            `每个工程最多固定 ${executionPinLimit} 个节目，请取消固定或清理未载入项`,
          );
        sources.push(action.key);
      }
    } else {
      const present = sources.filter((k) => available.includes(k));
      const at = present.indexOf(action.key);
      const neighbor = present[at + (action.kind === "earlier" ? -1 : 1)];
      if (index >= 0 && at >= 0 && neighbor) {
        const other = sources.indexOf(neighbor);
        [sources[index], sources[other]] = [sources[other], sources[index]];
      }
    }
  }
  return [
    { projectId, sources },
    ...projects.filter((p) => p.projectId !== projectId),
  ].slice(0, 20);
}
export function saveExecutionPins(
  projects: ExecutionPins[],
  write: (raw: string) => void,
): string {
  try {
    write(JSON.stringify({ version: 1, projects }));
    return "";
  } catch {
    return "常用节目仅在本次使用中保留，无法保存到本机";
  }
}
