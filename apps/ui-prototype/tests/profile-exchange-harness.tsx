import { useState } from "react";
import { createRoot } from "react-dom/client";
import { PatchDialog } from "../src/components/fixtures/PatchDialog";
import type { ProjectView, EditCommand } from "../src/application-host";
import type { ProfileView } from "../src/fixture-types";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/fixtures/fixtures.css";
const source: ProfileView = {
  id: "old",
  revision: "1",
  authorable: true,
  name: "原色盘",
  manufacturer: "验收",
  model: "验收",
  mode: "色盘",
  footprint: 2,
  channels: [
    { attribute: "dimmer", coarse: 1, fine: null, defaultValue: 0 },
    {
      attribute: "color-wheel",
      coarse: 2,
      fine: null,
      defaultValue: { functionKey: "red", position: 0 },
      functions: [
        {
          key: "red",
          name: "实测红色",
          mode: "slot",
          dmxFrom: 10,
          dmxTo: 19,
          dmxDefault: 14,
          appearance: { kind: "color", colors: ["#FF0000"] },
        },
      ],
    },
  ],
};
const target = structuredClone(source);
target.id = "new";
target.name = "校正后的色盘";
target.channels[1].functions![0].dmxDefault = 15;
const project: ProjectView = {
  id: "test",
  name: "色盘差异验收",
  description: "",
  audio: null,
  profiles: [source, target],
  domains: [],
  groups: [],
  presets: [],
  sequences: [],
  scenes: [],
  stage: { spaces: [], placements: [], attachments: [], constructions: [] },
  fixtures: ["甲", "乙"].map((name, i) => ({
    id: name,
    name: `灯${name}`,
    profileId: source.id,
    profileName: source.name,
    domainId: "d",
    domainName: "灯光",
    footprint: 2,
    universe: 1,
    address: 1 + i * 2,
    attributes: [
      { key: "dimmer", label: "亮度", defaultValue: 0 },
      {
        key: "color-wheel",
        label: "色盘",
        defaultValue: 0,
        function: {
          functions: source.channels[1].functions!,
          fine: false,
          default: { functionKey: "red", position: 0 },
        },
      },
    ],
  })),
};
function Harness() {
  const [open, setOpen] = useState(true),
    [commands, setCommands] = useState<EditCommand[]>([]);
  return (
    <main className="workbench">
      <button onClick={() => setOpen(true)}>打开替换</button>
      <p>已提交 {commands.length} 次</p>
      <pre aria-label="已提交命令">{JSON.stringify(commands)}</pre>
      {open && (
        <PatchDialog
          project={project}
          initialIds={["甲"]}
          exchange
          busy={false}
          error=""
          onCancel={() => setOpen(false)}
          onEdit={async (command) => {
            setCommands((v) => [...v, command]);
            return true;
          }}
        />
      )}
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
