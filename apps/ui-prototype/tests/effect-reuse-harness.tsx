// Isolated real-component acceptance; no host, file, device or renderer access.
import { useState } from "react";
import { createRoot } from "react-dom/client";
import type { FixtureView, SceneView } from "../src/application-host";
import { createEffect } from "../src/effect-tools";
import { EffectReuseDialog } from "../src/components/workbench/EffectReuseDialog";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/workbench/effects.css";
const fixtures: FixtureView[] = [
  {
    id: "rgb",
    name: "全彩灯",
    profileId: "rgb",
    profileName: "全彩",
    domainId: "d",
    domainName: "灯光",
    footprint: 4,
    universe: 1,
    address: 1,
    attributes: ["red", "green", "blue", "dimmer"].map((key) => ({
      key,
      label: key,
      defaultValue: 0,
    })),
  },
  {
    id: "dim",
    name: "仅调光灯",
    profileId: "dim",
    profileName: "调光",
    domainId: "d",
    domainName: "灯光",
    footprint: 1,
    universe: 1,
    address: 10,
    attributes: [{ key: "dimmer", label: "亮度", defaultValue: 0 }],
  },
];
const scenes: SceneView[] = [
  {
    id: "one",
    name: "第一幕",
    values: [],
    effects: Array.from({ length: 103 }, (_, i) => ({
      ...createEffect("color", String(i), ["rgb"]),
      name: `色彩 ${String(i + 1).padStart(3, "0")}`,
    })),
  },
  {
    id: "two",
    name: "第二幕",
    values: [],
    effects: [{ ...createEffect("breathe", "0", ["dim"]), name: "呼吸 001" }],
  },
];
function Harness() {
  const [open, setOpen] = useState(false);
  const [result, setResult] = useState("未选择");
  return (
    <main className="workbench" style={{ padding: 24 }}>
      <button onClick={() => setOpen(true)}>复用效果验收</button>
      <output aria-label="复用结果">{result}</output>
      {open && (
        <EffectReuseDialog
          scenes={scenes}
          fixtures={fixtures}
          selected={["dim"]}
          onCancel={() => setOpen(false)}
          onChoose={(effect, ids) =>
            setResult(
              `${effect.name} · ${ids?.join(",") ?? effect.fixtureIds.join(",")}`,
            )
          }
        />
      )}
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
