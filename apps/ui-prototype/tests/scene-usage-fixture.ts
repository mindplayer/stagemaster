import { stageProject } from "./stage-organization-fixture.ts";
export function usageProject() {
  const project = stageProject();
  project.scenes = [
    { id: "s", name: "暖场", values: [], effects: [] },
    { id: "other", name: "暖场", values: [], effects: [] },
  ];
  project.sequences = ["a", "b"].map((id) => ({
    id,
    name: "排练",
    tracking: "isolated" as const,
    repeat: "once" as const,
    steps: Array.from({ length: 25 }, (_, i) => ({
      id: `${id}-${i}`,
      name: `步骤 ${i + 1}`,
      number: `${i + 1}`,
      sceneId: i === 1 ? "other" : "s",
      delayMs: 0,
      fadeMs: 1000,
      waitMs: null,
    })),
  }));
  project.audio = {
    asset: {
      digest: "ab".repeat(32),
      fileName: "音乐.wav",
      extension: "wav",
      durationMs: 100000,
    },
    inMs: 1000,
    outMs: 99000,
    markers: [
      { id: "mark", name: "第一拍", timeMs: 1000, sceneId: "s" },
      { id: "beat", name: "节拍", timeMs: 2000, sceneId: null },
    ],
    lightingClips: [
      {
        id: "clip",
        name: "停用片段",
        startMs: 2000,
        endMs: 4000,
        fadeMs: 0,
        sceneId: "s",
        locked: true,
        enabled: false,
      },
    ],
  };
  return project;
}
