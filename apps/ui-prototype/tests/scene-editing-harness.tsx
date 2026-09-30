// Dev-only component regression surface. No file, device, audio or renderer access.
import { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import type { FixtureView, ProjectView } from "../src/application-host";
import { applicationHost } from "../src/hosts/application-host";
import { SceneInspector } from "../src/components/workbench/SceneInspector";
import {
  ParameterPanel,
  type ParameterHandle,
} from "../src/components/workbench/ParameterPanel";
import { SceneEditingTools } from "../src/components/workbench/SceneEditingTools";
import "../src/base.css";
import "../src/workbench.css";

const fixtures: FixtureView[] = Array.from({ length: 24 }, (_, i) => ({
  id: String(i),
  name: `光束摇头灯 ${i + 1} · 后区逆光（长名称边界）`,
  profileId: "p",
  profileName: "通用模式",
  domainId: "d",
  domainName: "舞台",
  universe: 1,
  footprint: 4,
  address: i * 4 + 1,
  attributes: ["dimmer", "red", "green", "blue"].map((key, index) => ({
    key,
    label: ["亮度", "红色", "绿色", "蓝色"][index],
    defaultValue: 0,
  })),
}));
const project: ProjectView = {
  id: "fixture",
  name: "隔离验收",
  description: "",
  audio: null,
  profiles: [],
  domains: [],
  fixtures,
  scenes: [{ id: "scene", name: "场景", effects: [], values: [] }],
  presets: [],
  sequences: [],
  groups: [
    { id: "all", name: "全部摇头灯", fixtureIds: fixtures.map((f) => f.id) },
  ],
  stage: { spaces: [], constructions: [], placements: [], attachments: [] },
};
const host = {
  ...applicationHost,
  preview: async () => ({ epoch: 0, controlSerial: 0, loaded: null }),
};
function Harness() {
  const [selected, setSelected] = useState(fixtures.map((f) => f.id));
  const [width, setWidth] = useState(1100);
  const [query, setQuery] = useState("");
  const [only, setOnly] = useState(false);
  const [report, setReport] = useState("等待检查");
  const params = useRef<ParameterHandle>(null);
  const beforeChange = async () => {
    try {
      params.current?.collect();
      params.current?.accept();
      setReport("验证通过");
      return true;
    } catch (e) {
      setReport(String(e));
      return false;
    }
  };
  return (
    <main className="workbench" style={{ width, maxWidth: "100%" }}>
      <nav>
        <button onClick={() => setWidth(1100)}>1100 像素</button>
        <button onClick={() => setWidth(820)}>820 像素</button>
        <output>{report}</output>
      </nav>
      <div
        className="performance-layout"
        style={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1fr) 260px",
          height: 440,
          minHeight: 440,
        }}
      >
        <div>舞台区域（隔离测试无渲染器）</div>
        <div className="wb-properties" style={{ overflow: "auto" }}>
          <SceneInspector
            active
            busy={false}
            hasPosition={false}
            beforeChange={beforeChange}
            light={
              <ParameterPanel
                key={selected.join(",")}
                ref={params}
                fixtures={fixtures.filter((f) => selected.includes(f.id))}
                scene={project.scenes[0]}
                busy={false}
                onApply={() => void beforeChange()}
                onPending={() => {}}
              />
            }
            position={null}
            scene={
              <input aria-label="场景备注测试" defaultValue="保留上下文" />
            }
          />
        </div>
      </div>
      <div style={{ height: 320, minHeight: 320 }}>
        <SceneEditingTools
          host={host}
          project={project}
          scene={project.scenes[0]}
          selected={selected}
          fixtureQuery={query}
          onlySelected={only}
          generation={1}
          visible
          busy={false}
          error=""
          beforeChange={beforeChange}
          onQuery={setQuery}
          onFilter={setOnly}
          onSelect={async (ids) => {
            if (!(await beforeChange())) return false;
            setSelected(ids);
            return true;
          }}
          onEdit={async () => false}
          onLibraryEdit={async () => null}
          onView3d={() => {}}
          onAddScene={() => {}}
          onAddFixtures={() => {}}
        />
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
