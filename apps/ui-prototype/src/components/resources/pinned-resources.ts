export type PinKind = "groups" | "presets";
export interface ProjectPins {
  projectId: string;
  groups: string[];
  presets: string[];
}
export const pinLimit = 8;
export const projectPinLimit = 20;
const validId = (value: unknown): value is string =>
  typeof value === "string" && value.length > 0 && value.length <= 128;
const ids = (value: unknown) =>
  Array.isArray(value)
    ? [...new Set(value.filter(validId))].slice(0, pinLimit)
    : [];
export function readPins(raw: string | null): ProjectPins[] {
  if (!raw || raw.length > 65536) return [];
  try {
    const data = JSON.parse(raw);
    if (data?.version !== 1 || !Array.isArray(data.projects)) return [];
    const result: ProjectPins[] = [];
    for (const entry of data.projects) {
      if (
        !entry ||
        !validId(entry.projectId) ||
        result.some((p) => p.projectId === entry.projectId)
      )
        continue;
      result.push({
        projectId: entry.projectId,
        groups: ids(entry.groups),
        presets: ids(entry.presets),
      });
      if (result.length === projectPinLimit) break;
    }
    return result;
  } catch {
    return [];
  }
}
export function togglePin(
  projects: readonly ProjectPins[],
  projectId: string,
  kind: PinKind,
  id: string,
  available: readonly string[],
): ProjectPins[] {
  if (!validId(projectId) || !validId(id) || !available.includes(id))
    throw new Error("资源已不可用，请重新选择");
  const original = projects.find((p) => p.projectId === projectId) ?? {
    projectId,
    groups: [],
    presets: [],
  };
  const current = original[kind].filter((id) => available.includes(id));
  if (!current.includes(id) && current.length >= pinLimit)
    throw new Error(`每类最多固定 ${pinLimit} 项，请先取消一个固定项`);
  const updated = {
    ...original,
    [kind]: current.includes(id)
      ? current.filter((value) => value !== id)
      : [...current, id],
  };
  return [updated, ...projects.filter((p) => p.projectId !== projectId)].slice(
    0,
    projectPinLimit,
  );
}

export function persistPins(
  projects: ProjectPins[],
  write: (raw: string) => void,
) {
  let problem = "";
  try {
    write(JSON.stringify({ version: 1, projects }));
  } catch {
    problem = "固定项仅在本次使用中保留，无法保存到本机";
  }
  return { projects, problem };
}
