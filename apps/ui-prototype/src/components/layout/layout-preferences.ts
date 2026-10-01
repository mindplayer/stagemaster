export interface LayoutPreferences {
  library: number;
  inspector: number;
  editor: number;
  showLibrary: boolean;
  showInspector: boolean;
  focusedTasks: LayoutTask[];
}
export const layoutTasks = ["scenes", "sequences", "audio"] as const;
export type LayoutTask = (typeof layoutTasks)[number];
export const defaultLayout: LayoutPreferences = {
  library: 240,
  inspector: 300,
  editor: 320,
  showLibrary: true,
  showInspector: true,
  focusedTasks: [],
};
export const panelBounds = {
  library: [180, 380],
  inspector: [240, 420],
  editor: [180, 560],
} as const;
export function panelSize(panel: keyof typeof panelBounds, value: number) {
  const [min, max] = panelBounds[panel];
  return Number.isFinite(value)
    ? Math.max(min, Math.min(max, value))
    : defaultLayout[panel];
}
export function readLayout(raw: string | null): LayoutPreferences {
  try {
    const data = JSON.parse(raw ?? "null");
    if (!data || typeof data !== "object")
      return { ...defaultLayout, focusedTasks: [] };
    return {
      library: panelSize("library", data.library),
      inspector: panelSize("inspector", data.inspector),
      editor: panelSize("editor", data.editor),
      showLibrary:
        typeof data.showLibrary === "boolean" ? data.showLibrary : true,
      showInspector:
        typeof data.showInspector === "boolean" ? data.showInspector : true,
      focusedTasks: layoutTasks.filter(
        (task) =>
          Array.isArray(data.focusedTasks) && data.focusedTasks.includes(task),
      ),
    };
  } catch {
    return { ...defaultLayout, focusedTasks: [] };
  }
}
