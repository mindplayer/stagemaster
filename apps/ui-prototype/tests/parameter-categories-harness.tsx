import { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  ParameterPanel,
  type ParameterHandle,
} from "../src/components/workbench/ParameterPanel";
import type { FixtureView, SceneView } from "../src/application-host";
import { channelLabels } from "../src/fixture-tools";
import { functionLabels } from "../src/fixture-function-types";
import "../src/base.css";
import "../src/workbench.css";

const attributes = [
  "zoom",
  "red",
  "gobo-wheel",
  "shutter",
  "dimmer",
  "green",
  "focus",
  "color-wheel",
  "blue",
  "prism",
  "iris",
  "custom-legacy",
].map((key) => ({
  key,
  label: channelLabels[key] ?? "兼容属性",
  defaultValue: 32768,
  ...(key in functionLabels
    ? {
        function: {
          functions: [
            {
              key: "open",
              name: "常规",
              mode: "slot" as const,
              dmxFrom: 0,
              dmxTo: 63,
              dmxDefault: 0,
            },
            {
              key: "range",
              name: "连续区间",
              mode: "range" as const,
              dmxFrom: 64,
              dmxTo: 255,
              dmxDefault: 128,
            },
          ],
          default: { functionKey: "open", position: 0 },
          fine: false,
        },
      }
    : {}),
}));
const fixture: FixtureView = {
  id: "lamp",
  name: "完整属性灯具",
  profileId: "p",
  profileName: "模式",
  domainId: "d",
  domainName: "灯光",
  universe: 1,
  address: 1,
  footprint: 12,
  attributes,
};
function Harness() {
  const [width, setWidth] = useState(300);
  const [scene, setScene] = useState<SceneView>({
    id: "s",
    name: "灯光",
    effects: [],
    values: [],
  });
  const [selection, setSelection] = useState("full");
  const [report, setReport] = useState("");
  const [reject, setReject] = useState(false);
  const [busy, setBusy] = useState(false);
  const [slow, setSlow] = useState(false);
  const [commits, setCommits] = useState(0);
  const pending = useRef<(() => void) | null>(null);
  const params = useRef<ParameterHandle>(null);
  const beforeChange = async () => {
    try {
      const commands = params.current!.collect();
      if (slow) {
        setBusy(true);
        await new Promise<void>((resolve) => {
          pending.current = resolve;
        });
        setBusy(false);
      }
      if (reject) {
        setReport("提交被拒绝，草稿保留");
        return false;
      }
      if (commands.length) {
        setScene((old) => {
          const values = [...old.values];
          for (const c of commands) {
            if (c.op !== "setSceneValue" && c.op !== "setSceneFunctionValue")
              continue;
            const entry = {
              fixtureId: c.fixtureId,
              attribute: c.attribute,
              value: c.op === "setSceneValue" ? c.value : 0,
              functionValue:
                c.op === "setSceneFunctionValue" ? c.selection : undefined,
              mode: "value" as const,
              presetId: null,
              presetName: null,
            };
            const index = values.findIndex((v) => v.attribute === c.attribute);
            if (index < 0) values.push(entry);
            else values[index] = entry;
          }
          return { ...old, values };
        });
        setCommits((n) => n + 1);
      }
      params.current!.accept();
      setReport("成功");
      return true;
    } catch (e) {
      setReport(String(e));
      return false;
    }
  };
  return (
    <main className="workbench">
      <nav>
        <button onClick={() => setWidth(width === 300 ? 260 : 300)}>
          切换宽度
        </button>
        <label>
          <input
            type="checkbox"
            checked={reject}
            onChange={(e) => setReject(e.target.checked)}
          />
          拒绝提交
        </label>
        <label>
          <input
            type="checkbox"
            checked={slow}
            onChange={(e) => setSlow(e.target.checked)}
          />
          延迟返回
        </label>
        <button
          onClick={() => {
            pending.current?.();
            pending.current = null;
          }}
        >
          完成请求
        </button>
        <select
          aria-label="选择灯具组合"
          value={selection}
          onChange={async (e) => {
            const value = e.target.value;
            if (await beforeChange()) setSelection(value);
          }}
        >
          <option value="full">完整属性</option>
          <option value="dimmer">仅调光</option>
          <option value="empty">未选灯具</option>
        </select>
      </nav>
      <output role="status">
        提交 {commits} · {report}
      </output>
      <div
        className="wb-properties"
        style={{ width, height: 550, overflow: "auto" }}
      >
        <ParameterPanel
          key={selection}
          ref={params}
          fixtures={
            selection === "empty"
              ? []
              : [
                  {
                    ...fixture,
                    attributes:
                      selection === "dimmer"
                        ? attributes.filter((a) => a.key === "dimmer")
                        : attributes,
                  },
                ]
          }
          scene={scene}
          busy={busy}
          beforeChange={beforeChange}
          onApply={() => void beforeChange()}
          onPending={() => {}}
        />
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
