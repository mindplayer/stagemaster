/** UI-only in-memory adapter. No timing, mixing, transport or device authority. */
export const PALETTE = [
  { id: "white", name: "白光", hex: "#dfedff", hue: 0, saturation: 0 },
  { id: "warm", name: "暖白", hex: "#f4cb91", hue: 175, saturation: 0.7 },
  { id: "blue", name: "冷蓝", hex: "#448afa", hue: 0, saturation: 1 },
  { id: "deep", name: "深蓝", hex: "#305ec6", hue: 15, saturation: 1.4 },
  { id: "purple", name: "紫色", hex: "#b684ee", hue: 65, saturation: 0.85 },
  { id: "pink", name: "品红", hex: "#dc6aaf", hue: 105, saturation: 0.85 },
] as const;
export type ColorId = (typeof PALETTE)[number]["id"];
export type GroupId = "front" | "back" | "wash";
export const GROUPS: {
  id: GroupId;
  name: string;
  prefix: string;
  color: string;
}[] = [
  { id: "front", name: "面光组", prefix: "F", color: "#2a7777" },
  { id: "back", name: "逆光组", prefix: "B", color: "#2c60a1" },
  { id: "wash", name: "染色组", prefix: "W", color: "#71519f" },
];
export type Clip = {
  id: string;
  group: GroupId;
  name: string;
  start: number;
  duration: number;
  intensity: number;
  color: ColorId;
  fadeIn: number;
  fadeOut: number;
};
export type DemoCue = {
  id: number;
  name: string;
  duration: number;
  clips: Clip[];
};
type EditSnapshot = {
  drafts: Record<number, DemoCue>;
  editCueId: number;
  selectedClipId: string | null;
};
export type DemoState = EditSnapshot & {
  saved: Record<number, DemoCue>;
  history: EditSnapshot[];
  future: EditSnapshot[];
  running: { cueId: number; snapshot: DemoCue } | null;
  nextId: number | null;
};
export type Action =
  | { type: "selectCue"; id: number }
  | { type: "selectClip"; id: string }
  | { type: "patchClip"; id: string; patch: Partial<Clip> }
  | { type: "addClip"; group: GroupId; at: number; id: string; color?: ColorId }
  | { type: "deleteClip"; id: string }
  | { type: "save" }
  | { type: "undo" }
  | { type: "redo" }
  | { type: "go" }
  | { type: "standby"; id: number };
const clone = <T>(value: T): T => structuredClone(value);
// UI gesture precision only; this adapter is not the execution clock.
const round = (n: number) => Math.round(n * 1000) / 1000;
export const clamp = (n: number, min: number, max: number) =>
  Math.min(max, Math.max(min, n));

export function normalizeClip(clip: Clip, length: number): Clip {
  const duration = round(clamp(clip.duration, 0.2, length));
  const start = round(clamp(clip.start, 0, length - duration));
  const fadeIn = round(clamp(clip.fadeIn, 0, duration));
  const fadeOut = round(clamp(clip.fadeOut, 0, duration - fadeIn));
  return {
    ...clip,
    start,
    duration,
    fadeIn,
    fadeOut,
    intensity: Math.round(clamp(clip.intensity, 0, 100)),
  };
}
function makeCue(id: number, name: string): DemoCue {
  return {
    id,
    name,
    duration: 20,
    clips: [
      {
        id: "front",
        group: "front",
        name: "柔和面光",
        start: 0,
        duration: 20,
        intensity: 80,
        color: "warm",
        fadeIn: 1,
        fadeOut: 1,
      },
      {
        id: "back",
        group: "back",
        name: "桥段逆光",
        start: 4,
        duration: 10,
        intensity: id === 14 ? 65 : 45,
        color: "blue",
        fadeIn: 2,
        fadeOut: 2,
      },
      {
        id: "wash-a",
        group: "wash",
        name: "蓝色铺底",
        start: 0,
        duration: 8,
        intensity: 60,
        color: "blue",
        fadeIn: 1,
        fadeOut: 1,
      },
      {
        id: "wash-b",
        group: "wash",
        name: "紫色过渡",
        start: 8,
        duration: 12,
        intensity: 60,
        color: "purple",
        fadeIn: 1,
        fadeOut: 2,
      },
    ],
  };
}
export function createDemoState(): DemoState {
  const saved = Object.fromEntries(
    ["前奏", "主歌", "副歌", "桥段", "尾声"].map((name, i) => [
      11 + i,
      makeCue(11 + i, name),
    ]),
  );
  return {
    saved,
    drafts: clone(saved),
    editCueId: 14,
    selectedClipId: "back",
    history: [],
    future: [],
    running: { cueId: 12, snapshot: clone(saved[12]) },
    nextId: 13,
  };
}
function editSnapshot(state: DemoState): EditSnapshot {
  return {
    drafts: clone(state.drafts),
    editCueId: state.editCueId,
    selectedClipId: state.selectedClipId,
  };
}
function edited(
  state: DemoState,
  cue: DemoCue,
  selectedClipId = state.selectedClipId,
): DemoState {
  return {
    ...state,
    drafts: { ...state.drafts, [cue.id]: cue },
    selectedClipId,
    history: [...state.history.slice(-49), editSnapshot(state)],
    future: [],
  };
}
export function hasChanges(state: DemoState, id = state.editCueId): boolean {
  return JSON.stringify(state.drafts[id]) !== JSON.stringify(state.saved[id]);
}
export function demoReducer(state: DemoState, action: Action): DemoState {
  const cue = state.drafts[state.editCueId];
  switch (action.type) {
    case "selectCue":
      return state.drafts[action.id]
        ? {
            ...state,
            editCueId: action.id,
            selectedClipId: state.drafts[action.id].clips[0]?.id ?? null,
          }
        : state;
    case "selectClip":
      return cue.clips.some((c) => c.id === action.id)
        ? { ...state, selectedClipId: action.id }
        : state;
    case "patchClip": {
      const existing = cue.clips.find((c) => c.id === action.id);
      if (!existing) return state;
      const { intensity, start, duration, fadeIn, fadeOut, color } =
        action.patch;
      const numbers = [intensity, start, duration, fadeIn, fadeOut];
      if (numbers.some((n) => n !== undefined && !Number.isFinite(n)))
        return state;
      if (color !== undefined && !PALETTE.some((p) => p.id === color))
        return state;
      const patch = Object.fromEntries(
        Object.entries({
          intensity,
          start,
          duration,
          fadeIn,
          fadeOut,
          color,
        }).filter(([, v]) => v !== undefined),
      );
      const next = normalizeClip({ ...existing, ...patch }, cue.duration);
      if (JSON.stringify(existing) === JSON.stringify(next)) return state;
      return edited(state, {
        ...cue,
        clips: cue.clips.map((c) => (c.id === action.id ? next : c)),
      });
    }
    case "addClip": {
      if (
        !GROUPS.some((g) => g.id === action.group) ||
        !Number.isFinite(action.at) ||
        cue.clips.some((c) => c.id === action.id)
      )
        return state;
      if (
        action.color !== undefined &&
        !PALETTE.some((p) => p.id === action.color)
      )
        return state;
      const group = GROUPS.find((g) => g.id === action.group)!;
      const clip = normalizeClip(
        {
          id: action.id,
          group: action.group,
          name: group.name + "片段",
          start: action.at,
          duration: 4,
          intensity: 65,
          color: action.color ?? "blue",
          fadeIn: 0.5,
          fadeOut: 0.5,
        },
        cue.duration,
      );
      return edited(state, { ...cue, clips: [...cue.clips, clip] }, clip.id);
    }
    case "deleteClip":
      if (!cue.clips.some((c) => c.id === action.id)) return state;
      return edited(
        state,
        { ...cue, clips: cue.clips.filter((c) => c.id !== action.id) },
        state.selectedClipId === action.id ? null : state.selectedClipId,
      );
    case "save":
      return hasChanges(state)
        ? { ...state, saved: { ...state.saved, [cue.id]: clone(cue) } }
        : state;
    case "undo": {
      const previous = state.history.at(-1);
      return previous
        ? {
            ...state,
            ...clone(previous),
            history: state.history.slice(0, -1),
            future: [editSnapshot(state), ...state.future],
          }
        : state;
    }
    case "redo": {
      const next = state.future[0];
      return next
        ? {
            ...state,
            ...clone(next),
            history: [...state.history, editSnapshot(state)],
            future: state.future.slice(1),
          }
        : state;
    }
    case "standby":
      return state.saved[action.id] ? { ...state, nextId: action.id } : state;
    case "go": {
      if (state.nextId === null) return state;
      const next = state.nextId;
      const ids = Object.keys(state.saved)
        .map(Number)
        .sort((a, b) => a - b);
      return {
        ...state,
        running: { cueId: next, snapshot: clone(state.saved[next]) },
        nextId: ids[ids.indexOf(next) + 1] ?? null,
      };
    }
  }
}
export function formatTime(value: number): string {
  const ticks = Math.round(Math.max(0, value) * 10);
  return (
    String(Math.floor(ticks / 600)).padStart(2, "0") +
    ":" +
    String(Math.floor(ticks / 10) % 60).padStart(2, "0") +
    "." +
    (ticks % 10)
  );
}
