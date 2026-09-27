import { useRef, useState } from "react";
import type { ProjectView } from "../../application-host";
import type { RigLayout, StageEdit } from "../../stage-types";
import { canonical } from "../../stage-tools";
import { LibraryDialog } from "../workbench/LibraryDialog";
import { OrderedFixturePicker } from "./OrderedFixturePicker";
export function RigAttachmentDialog({
  project,
  initialIds,
  initialRig,
  busy,
  error,
  onApply,
  onCancel,
}: {
  project: ProjectView;
  initialIds: string[];
  initialRig: string;
  busy: boolean;
  error: string;
  onApply(command: StageEdit): Promise<boolean>;
  onCancel(): void;
}) {
  const root = useRef<HTMLDivElement>(null);
  const [ids, setIds] = useState(initialIds),
    [rig, setRig] = useState(initialRig),
    [layout, setLayout] = useState(true);
  const [draft, setDraft] = useState<RigLayout>({
    startMarginMeters: "0.3",
    endMarginMeters: "0.3",
    dropMeters: "0.1",
  });
  const rigs = project.stage.constructions.filter(
      (c) => c.shape.kind === "rig",
    ),
    selected = rigs.find((c) => c.id === rig);
  const members = project.stage.attachments.filter(
    (a) => a.constructionId === rig,
  );
  const field = (key: keyof RigLayout, label: string) => (
    <label>
      {label}
      <input
        type="number"
        required
        step="any"
        min={0}
        max={1000}
        aria-label={label}
        data-rig-field={key}
        value={draft[key]}
        onChange={(e) => setDraft((d) => ({ ...d, [key]: e.target.value }))}
      />
    </label>
  );
  return (
    <LibraryDialog
      title="批量挂灯"
      submit={`挂接 ${ids.length} 台灯具`}
      busy={busy}
      error={error}
      onCancel={onCancel}
      onSubmit={async () => {
        if (layout && selected?.shape.kind === "rig") {
          const sum =
              Number(draft.startMarginMeters) + Number(draft.endMarginMeters),
            length = Number(selected.shape.lengthMeters);
          if (sum > length || (ids.length > 1 && sum >= length)) {
            const error = "两端余量过大，没有足够长度排列灯具";
            const field = root.current?.querySelector<HTMLInputElement>(
              '[data-rig-field="endMarginMeters"]',
            );
            field?.setCustomValidity(error);
            field?.reportValidity();
            throw new Error(error);
          }
        }
        return onApply({
          op: "attachFixtures",
          constructionId: rig,
          fixtureIds: ids,
          layout: layout
            ? {
                startMarginMeters: canonical(draft.startMarginMeters),
                endMarginMeters: canonical(draft.endMarginMeters),
                dropMeters: canonical(draft.dropMeters),
              }
            : null,
        });
      }}
    >
      <div className="placement-editor" ref={root}>
        <OrderedFixturePicker project={project} ids={ids} setIds={setIds} />
        <section className="placement-settings">
          <label>
            目标支撑体
            <select
              aria-label="目标支撑体"
              required
              value={rig}
              onChange={(e) => setRig(e.target.value)}
            >
              <option value="" disabled>
                选择桁架或灯杆
              </option>
              {rigs.map((r) => (
                <option key={r.id} value={r.id}>
                  {r.name}
                </option>
              ))}
            </select>
          </label>
          <div className="rig-summary">
            {selected?.shape.kind === "rig" && (
              <>
                <strong>
                  {selected.shape.lengthMeters} 米 · 标高{" "}
                  {selected.shape.positionMeters.z} 米
                </strong>
                <span>已挂 {members.length} 台灯具</span>
              </>
            )}
          </div>
          <label>
            挂灯方式
            <select
              aria-label="挂灯方式"
              value={layout ? "layout" : "keep"}
              onChange={(e) => setLayout(e.target.value === "layout")}
            >
              <option value="layout">沿支撑体均布</option>
              <option value="keep">保持当前位置，仅建立挂接</option>
            </select>
          </label>
          {layout && (
            <>
              <div className="stage-pair">
                {field("startMarginMeters", "首端余量（米）")}
                {field("endMarginMeters", "末端余量（米）")}
              </div>
              {field("dropMeters", "下挂距离（米）")}
              <p className="wb-dim">
                按左侧灯序从首端到末端排列，下挂距离从支撑体底部计算。
              </p>
            </>
          )}
          <p className="wb-dim">
            挂灯后跟随支撑体移动、旋转和升降；再次挂接会更换所属支撑体。
          </p>
          {ids.some((id) => members.some((a) => a.fixtureId === id)) && (
            <p className="wb-dim">包含已挂在此处的灯具，将按本次方式更新。</p>
          )}
        </section>
      </div>
    </LibraryDialog>
  );
}
