// Isolated adapter simulation; no filesystem, playback or physical output.
import { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { EffectTemplateFiles } from "../src/components/workbench/EffectTemplateFiles";
import { applicationHost } from "../src/hosts/application-host";
import { createEffect } from "../src/effect-tools";
import type { ImportedEffectTemplate } from "../src/effect-template-types";
import type { FixtureView } from "../src/application-host";
import "../src/base.css";
import "../src/workbench.css";
let finish: (() => void) | null = null;
const fixtures: FixtureView[] = ["甲", "乙"].map((id, i) => ({
  id,
  name: `灯${id}`,
  profileId: id,
  profileName: `档案${id}`,
  domainId: "d",
  domainName: "灯光",
  footprint: 1,
  universe: 1,
  address: 1 + i,
  attributes: [{ key: "dimmer", label: "亮度", defaultValue: 0 }],
}));
const scene = {
  id: "scene",
  name: "演出",
  values: [],
  effects: [
    createEffect("breathe", "effect", ["甲"]),
    createEffect("color", "color", ["甲"]),
  ],
};
function Harness() {
  const [generation, setGeneration] = useState(1),
    [visible, setVisible] = useState(true),
    [selected, setSelected] = useState(["甲", "乙"]);
  const [delay, setDelay] = useState(false),
    [rejectCapture, setRejectCapture] = useState(false),
    [rejectApply, setRejectApply] = useState(false),
    [cancelFile, setCancelFile] = useState(false),
    [keyframes, setKeyframes] = useState(false);
  const [events, setEvents] = useState<string[]>([]),
    [error, setError] = useState("");
  const flags = useRef({ delay, cancelFile, keyframes });
  flags.current = { delay, cancelFile, keyframes };
  const notify = (text: string) => setEvents((e) => [...e, text]);
  const host = useRef({
    ...applicationHost,
    importEffectTemplate: async (
      gen: number,
      sceneId: string,
      ids: string[],
    ) => {
      notify(`导入 ${gen} ${ids.join("→")}`);
      if (flags.current.delay)
        await new Promise<void>((resolve) => {
          finish = resolve;
        });
      if (flags.current.cancelFile) return null;
      const effect = createEffect("breathe", "imported", ids);
      effect.templateSource = {
        sha256: "a".repeat(64),
        template: {
          format: "stagemaster-effect-template",
          formatVersion: 1,
          templateId: "t",
          revision: "r",
          definition: {
            name: "亮度呼吸",
            recipe: {
              kind: "intensity-wave",
              waveform: "smooth",
              low: 0,
              high: 65535,
              dutyPercent: 50,
            },
            timing: {
              periodMs: 2000,
              phaseDegrees: 0,
              spreadDegrees: 0,
              reverseOrder: false,
            },
          },
        },
      };
      if (flags.current.keyframes) {
        const frames = [
          { position: 0, value: 0, transition: "hold" as const },
          { position: 2000, value: 65535, transition: "linear" as const },
          { position: 8000, value: 20000, transition: "smooth" as const },
        ];
        effect.waveform = "keyframes";
        effect.channels = [{ attribute: "dimmer", keyframes: frames }];
        effect.templateSource.template = {
          ...effect.templateSource.template,
          formatVersion: 2,
          definition: {
            ...effect.templateSource.template.definition,
            recipe: { kind: "intensity-keyframes", keyframes: frames },
          },
        };
      }
      return {
        generation: gen,
        token: `token-${gen}-${ids.join("")}`,
        fileName: "亮度.smeffect.json",
        review: {
          sceneId,
          effect,
          usage: {
            attributes: 2,
            steps: 1,
            targetValues: 2,
            effectChannels: 2,
            keyframes: 0,
            valueBufferBytes: 4,
            effectBufferBytes: 96,
          },
        },
      } satisfies ImportedEffectTemplate;
    },
    cancelEffectTemplate: async (token: string) => {
      notify(`取消 ${token}`);
    },
    exportEffectTemplate: async (gen: number, _scene: string, id: string) => {
      notify(`导出 ${id}`);
      return { generation: gen, effectId: id, path: null, warning: null };
    },
  }).current;
  return (
    <main className="workbench" style={{ padding: 20, gap: 12 }}>
      <div>
        <button
          onClick={() => setSelected((s) => (s.length ? [] : ["甲", "乙"]))}
        >
          切换空选择
        </button>
        <button onClick={() => setSelected((s) => [...s].reverse())}>
          反转灯序
        </button>
        <button onClick={() => setVisible((v) => !v)}>切换页面</button>
        <button onClick={() => setGeneration((v) => v + 1)}>修改工程</button>
        <button onClick={() => setDelay((v) => !v)}>
          延迟回执 {String(delay)}
        </button>
        <button
          onClick={() => {
            finish?.();
            finish = null;
          }}
        >
          返回回执
        </button>
        <button onClick={() => setRejectCapture((v) => !v)}>
          拒绝草稿检查 {String(rejectCapture)}
        </button>
        <button onClick={() => setRejectApply((v) => !v)}>
          拒绝应用 {String(rejectApply)}
        </button>
        <button onClick={() => setKeyframes((v) => !v)}>
          关键帧模板 {String(keyframes)}
        </button>
        <button onClick={() => setCancelFile((v) => !v)}>
          取消文件选择 {String(cancelFile)}
        </button>
      </div>
      <p>
        页面 {String(visible)} · 代次 {generation} · 灯序 {selected.join("→")}
      </p>
      <EffectTemplateFiles
        host={host}
        generation={generation}
        scene={scene}
        fixtures={fixtures}
        selected={selected}
        visible={visible}
        busy={false}
        operationError={error}
        capture={async () => {
          setError("");
          return rejectCapture ? null : generation;
        }}
        onApply={async (gen, token) => {
          if (rejectApply) {
            setError("测试：绑定冲突，未应用");
            return false;
          }
          notify(`应用 ${gen} ${token}`);
          setGeneration((v) => v + 1);
          return true;
        }}
      />
      <pre aria-label="操作记录">{events.join("\n") || "无操作"}</pre>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
