import { useSyncExternalStore } from "react";
import {
  readPins,
  togglePin,
  persistPins,
  type PinKind,
  type ProjectPins,
} from "./pinned-resources";
const KEY = "stagemaster.resource-pins.v1";
interface PinState {
  projects: ProjectPins[];
  problem: string;
}
let state: PinState | null = null;
const listeners = new Set<() => void>();
function snapshot(): PinState {
  if (!state) {
    try {
      state = { projects: readPins(localStorage.getItem(KEY)), problem: "" };
    } catch {
      state = {
        projects: [],
        problem: "无法读取本机固定项，当前固定仅在本次使用中保留",
      };
    }
  }
  return state;
}
function changed() {
  for (const notify of listeners) notify();
}
function storage(event: StorageEvent) {
  if (event.key !== KEY && event.key !== null) return;
  state = { projects: readPins(event.newValue), problem: "" };
  changed();
}
function subscribe(notify: () => void) {
  if (!listeners.size) window.addEventListener("storage", storage);
  listeners.add(notify);
  return () => {
    listeners.delete(notify);
    if (!listeners.size) window.removeEventListener("storage", storage);
  };
}
export function useResourcePins(projectId: string, kind: PinKind) {
  const data = useSyncExternalStore(subscribe, snapshot);
  return {
    ids: data.projects.find((p) => p.projectId === projectId)?.[kind] ?? [],
    problem: data.problem,
    toggle(id: string, available: readonly string[]) {
      const projects = togglePin(
        snapshot().projects,
        projectId,
        kind,
        id,
        available,
      );
      state = persistPins(projects, (raw) => localStorage.setItem(KEY, raw));
      changed();
    },
  };
}
