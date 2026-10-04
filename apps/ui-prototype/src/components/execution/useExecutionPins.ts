import { useState } from "react";
import {
  editExecutionPins,
  readExecutionPins,
  saveExecutionPins,
} from "../../execution-pins";
const storageKey = "stagemaster.executionPins.v1";
export function useExecutionPins(projectId: string, available: string[]) {
  const [projects, setProjects] = useState(() => {
    try {
      return readExecutionPins(localStorage.getItem(storageKey));
    } catch {
      return [];
    }
  });
  const [problem, setProblem] = useState("");
  const pins = projects.find((p) => p.projectId === projectId)?.sources ?? [];
  function edit(action: Parameters<typeof editExecutionPins>[3]) {
    try {
      const next = editExecutionPins(projects, projectId, available, action);
      setProjects(next);
      setProblem(
        saveExecutionPins(next, (raw) => localStorage.setItem(storageKey, raw)),
      );
    } catch (e) {
      setProblem(e instanceof Error ? e.message : String(e));
    }
  }
  return {
    pins,
    problem,
    edit,
    missing: pins.filter((k) => !available.includes(k)).length,
  };
}
