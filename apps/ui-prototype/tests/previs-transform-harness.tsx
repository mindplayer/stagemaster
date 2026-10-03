// Real controls and SDK coordinate translator; no UE, devices or project writes.
import { createRoot } from "react-dom/client";
import { useEffect, useRef, useState } from "react";
import { InputCoordTranslator } from "../node_modules/@epicgames-ps/lib-pixelstreamingfrontend-ue5.8/src/Util/InputCoordTranslator";
import { PrevisMoveTools } from "../src/components/stage/PrevisMoveTools";
import { observePrevisInputGeometry } from "../src/components/stage/previs-input-geometry";
import type { PrevisTool } from "../src/previs-types";
import "../src/base.css";
import "../src/components/stage/stage.css";
import "../src/workbench.css";

function Harness() {
  const [tool, setTool] = useState<PrevisTool>("rotate");
  const [key, setKey] = useState(0),
    [busy, setBusy] = useState(false);
  const [height, setHeight] = useState(360),
    [observing, setObserving] = useState(true);
  const [received, setReceived] = useState(""),
    [calls, setCalls] = useState(0);
  const [geometry, setGeometry] = useState("");
  const area = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!observing || !area.current) return;
    const element = area.current;
    const translator = new InputCoordTranslator();
    return observePrevisInputGeometry(element, () => {
      const width = element.clientWidth,
        height = element.clientHeight;
      translator.reconfigure({ width, height }, { width: 1920, height: 1080 });
      const expectedVideoHeight = (width * 1080) / 1920;
      const atQuarter = translator.translateUnsigned(
        width / 2,
        (height - expectedVideoHeight) / 2 + expectedVideoHeight / 4,
      );
      setGeometry(
        JSON.stringify({ width, height, x: atQuarter.x, y: atQuarter.y }),
      );
      setCalls((n) => n + 1);
    });
  }, [observing]);
  return (
    <main className="workbench" style={{ height: "auto", padding: 20 }}>
      <div className="previs-tools">
        <PrevisMoveTools
          moving
          tool={tool}
          contextKey={String(key)}
          disabled={busy}
          onAction={(action) => {
            if (action === "rotate" || action === "scale") setTool(action);
            if (action === "cancel") setReceived("取消");
          }}
          onExact={(yaw, scale) => setReceived(JSON.stringify({ yaw, scale }))}
        />
      </div>
      <div>
        <button onClick={() => setKey((n) => n + 1)}>切换选择</button>
        <button onClick={() => setBusy((v) => !v)}>切换忙状态</button>
        <button onClick={() => setHeight((v) => (v === 360 ? 440 : 360))}>
          调整视窗高度
        </button>
        <button onClick={() => setHeight(0)}>隐藏视窗</button>
        <button onClick={() => setObserving(false)}>停止监听</button>
      </div>
      <output aria-label="提交参数">{received}</output>
      <output aria-label="尺寸更新次数">{calls}</output>
      <output aria-label="实际坐标换算">{geometry}</output>
      <div ref={area} style={{ width: 640, height, background: "#17222c" }}>
        真实布局尺寸变更测试
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
