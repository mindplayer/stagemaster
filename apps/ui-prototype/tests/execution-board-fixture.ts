import type { ExecutionView } from "../src/execution-types";
export function boardFixture(): ExecutionView {
  const sources = Array.from({ length: 62 }, (_, i) => ({
    id: `source-${i}`,
    name: `${String(i + 1).padStart(2, "0")} · ${i % 2 ? "暖金侧光" : "蓝色逆光"}`,
    priority: i,
    selection: {
      kind: i % 8 === 0 ? ("sequence" as const) : ("scene" as const),
      id: `object-${i}`,
    },
    steps: Array.from({ length: i % 8 === 0 ? 20 : 1 }, (_, n) => ({
      id: `step-${i}-${n}`,
      number: String(n + 1),
      name: `段落 ${n + 1}`,
    })),
  }));
  return {
    hostId: "board-test",
    catalog: {
      projectId: "board-test-project",
      layout: "board-layout",
      physicalOutput: false,
      audio: {
        output: "software",
        durationMs: 30000,
        group: "media",
        seekIncludesEnd: true,
      },
      sources: [
        ...sources,
        {
          id: "music",
          name: "演出音乐",
          priority: 0,
          selection: { kind: "audioTimeline" },
          steps: [],
        },
        {
          id: "manual",
          name: "手动编程器",
          priority: 100,
          selection: { kind: "manual" },
          steps: [],
        },
      ],
    },
    controlling: true,
    sessionId: "session",
    pending: false,
    record: null,
    observation: {
      phase: "running",
      fault: null,
      snapshot: {
        cycles: "1",
        missedPeriods: "0",
        state: {
          revision: "1",
          owner: null,
          fault: false,
          audio: {
            output: "software",
            status: "paused",
            positionMs: 2000,
            durationMs: 30000,
            problem: null,
          },
          media: [
            {
              id: "media",
              generation: "1",
              status: "Paused",
              positionMs: 2000,
              control: null,
            },
          ],
          sources: [
            ...sources.map((s, i) => ({
              id: s.id,
              level: 65535,
              status:
                i < 2
                  ? "Running"
                  : i === 2
                    ? "Paused"
                    : i === 3
                      ? "Finished"
                      : "Idle",
              step: i < 4 ? s.steps[0].id : null,
            })),
            { id: "music", level: 65535, status: "Paused", step: null },
            { id: "manual", level: 65535, status: null, step: null },
          ],
        },
      },
    },
  };
}
