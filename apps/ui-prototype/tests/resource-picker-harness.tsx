// Isolated UI regression: no device, renderer, file or application-host access.
import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { ResourcePicker } from "../src/components/resources/ResourcePicker";
import "../src/base.css";
import "../src/workbench.css";
const options = Array.from({ length: 1000 }, (_, index) => ({
  id: String(index),
  label: `灯组 ${String(index + 1).padStart(4, "0")} · 长名称舞台左侧染色摇头灯`,
  detail: `${(index % 24) + 1} 台灯具`,
  keywords: "RGB 蓝色",
}));
function Harness() {
  const [tick, setTick] = useState(0);
  const [selected, setSelected] = useState("0");
  const [commits, setCommits] = useState(0);
  useEffect(() => {
    const timer = setInterval(() => setTick((v) => v + 1), 60);
    return () => clearInterval(timer);
  }, []);
  return (
    <main className="workbench" style={{ padding: 20 }}>
      <ResourcePicker
        label="灯组选择验收"
        placeholder="选择灯组"
        value={selected}
        options={options}
        disabled={false}
        onSelect={(id) => {
          setSelected(id);
          setCommits((v) => v + 1);
        }}
      />
      <output aria-label="选择结果">
        {selected} · 提交 {commits}
      </output>
      <output aria-label="播放轮询">{tick}</output>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
