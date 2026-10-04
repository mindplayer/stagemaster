import { boardFixture } from "./execution-board-fixture.ts";
import type { ManualFixture } from "../src/execution-manual.ts";
export function manualFixture() {
  const runtime = boardFixture();
  runtime.catalog.sources = runtime.catalog.sources.filter(
    (s) => s.id === "source-0" || s.id === "manual",
  );
  delete runtime.catalog.audio;
  runtime.catalog.capabilities = [
    "manualOwnership",
    "semanticManualPatch",
    "manualValues",
  ];
  runtime.catalog.limits = { manualChanges: 512, requestBytes: 8192 };
  runtime.catalog.fixtures = Array.from(
    { length: 35 },
    (_, index): ManualFixture => ({
      id: `00000000-0000-4000-8000-${String(index + 1).padStart(12, "0")}`,
      name: `测试灯 ${index + 1}`,
      profileName: index === 2 ? "定制色盘" : "标准色盘",
      universe: 1,
      address: 1 + index * 4,
      attributes: [
        { key: "dimmer", label: "亮度", defaultValue: 0 },
        {
          key: "color-wheel",
          label: "色盘",
          defaultValue: 0,
          function: {
            fine: false,
            default: { functionKey: "white", position: 0 },
            functions: [
              {
                key: "white",
                name: "白色",
                mode: "slot",
                dmxFrom: 0,
                dmxTo: 15,
                dmxDefault: 0,
              },
              {
                key: "red",
                name: index === 2 ? "绿色" : "红色",
                mode: "slot",
                dmxFrom: 16,
                dmxTo: 31,
                dmxDefault: 20,
              },
              {
                key: "rotate",
                name: "旋转",
                mode: "range",
                dmxFrom: 32,
                dmxTo: 255,
                dmxDefault: 32,
              },
            ],
          },
        },
      ],
    }),
  );
  const state = runtime.observation.snapshot!.state;
  state.sources = state.sources.filter(
    (s) => s.id === "source-0" || s.id === "manual",
  );
  state.sources.find((s) => s.id === "manual")!.held = [];
  state.sources.find((s) => s.id === "manual")!.heldValues = [];
  delete state.audio;
  delete state.media;
  return runtime;
}
