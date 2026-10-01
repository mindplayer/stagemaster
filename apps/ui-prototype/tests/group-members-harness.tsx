import { useState } from "react";
import { createRoot } from "react-dom/client";
import type { FixtureView } from "../src/application-host";
import { GroupEditor } from "../src/components/workbench/GroupEditor";
import "../src/base.css";
import "../src/workbench.css";
const fixtures: FixtureView[] = Array.from({ length: 123 }, (_, i) => ({
  id: `f${i + 1}`,
  name: `${i % 3 === 0 ? "染色灯" : "光束灯"} ${String(i + 1).padStart(3, "0")}`,
  profileId: "p",
  profileName: i % 3 === 0 ? "染色模式" : "光束模式",
  domainId: "d",
  domainName: "灯光",
  universe: 1 + Math.floor(i / 50),
  address: (i % 50) * 10 + 1,
  footprint: 1,
  attributes: [{ key: "dimmer", label: "亮度", defaultValue: 0 }],
}));
const group = {
  id: "g",
  name: "主灯组",
  fixtureIds: fixtures.slice(0, 61).map((f) => f.id),
};
function Harness() {
  const [open, setOpen] = useState(false),
    [reject, setReject] = useState(false),
    [error, setError] = useState("");
  const [report, setReport] = useState(""),
    [commits, setCommits] = useState(0);
  return (
    <main className="workbench" style={{ padding: 24 }}>
      <label>
        <input
          type="checkbox"
          checked={reject}
          onChange={(e) => setReject(e.target.checked)}
        />
        拒绝提交
      </label>
      <button
        onClick={() => {
          setError("");
          setOpen(true);
        }}
      >
        打开灯组验收
      </button>
      <output>
        提交 {commits} · {report}
      </output>
      {open && (
        <GroupEditor
          placements={fixtures.slice(0, 60).map((f, i) => ({
            fixtureId: f.id,
            spaceId: null,
            positionMeters: {
              x: String(60 - i),
              y: String(i % 3),
              z: String(i % 2),
            },
            rotationDegreesXYZ: { x: "0", y: "0", z: "0" },
          }))}
          group={group}
          fixtures={fixtures}
          selected={["f123", "f62", "f5"]}
          name="主灯组"
          busy={false}
          error={error}
          onCancel={() => setOpen(false)}
          onEdit={async (command) => {
            setReport(JSON.stringify(command));
            if (reject) {
              setError("验收拒绝：保留当前灯序");
              return false;
            }
            setCommits((n) => n + 1);
            return true;
          }}
        />
      )}
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
