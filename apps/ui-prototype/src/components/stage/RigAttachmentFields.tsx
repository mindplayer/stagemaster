import type { ProjectView } from "../../application-host";
import type { RigLayout } from "../../stage-types";
import type { RiggingSession } from "./rigging-session";
import { displayMeters } from "./stage-display";

export function RigAttachmentFields({
  project,
  session,
  update,
}: {
  project: ProjectView;
  session: RiggingSession;
  update(patch: Partial<Omit<RiggingSession, "source" | "dirty">>): void;
}) {
  const rigs = project.stage.constructions.filter(
    (c) => c.shape.kind === "rig",
  );
  const selected = rigs.find((c) => c.id === session.rig);
  const members = project.stage.attachments.filter(
    (a) => a.constructionId === session.rig,
  );
  const replacing = project.stage.attachments.filter(
    (a) =>
      session.ids.includes(a.fixtureId) && a.constructionId !== session.rig,
  ).length;
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
        value={session.draft[key]}
        onChange={(e) =>
          update({ draft: { ...session.draft, [key]: e.target.value } })
        }
      />
    </label>
  );
  return (
    <section className="rigging-fields placement-settings">
      <label>
        目标支撑体
        <select
          aria-label="目标支撑体"
          data-rig-field="rig"
          required
          value={session.rig}
          onChange={(e) => update({ rig: e.target.value })}
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
      {selected?.shape.kind === "rig" && (
        <div className="rig-summary">
          <strong>
            {displayMeters(Number(selected.shape.lengthMeters))} 米 · 标高{" "}
            {displayMeters(Number(selected.shape.positionMeters.z))} 米
          </strong>
          <span>
            已挂 {members.length} 台 · 本次选择 {session.ids.length} 台
          </span>
          {replacing > 0 && <span>{replacing} 台将从其他支撑体换挂至此</span>}
        </div>
      )}
      <label>
        挂灯方式
        <select
          aria-label="挂灯方式"
          value={session.layout ? "layout" : "keep"}
          onChange={(e) => update({ layout: e.target.value === "layout" })}
        >
          <option value="keep">保持当前位置，仅建立挂接</option>
          <option value="layout">沿支撑体均布</option>
        </select>
      </label>
      {session.layout && (
        <>
          <div className="stage-pair">
            {field("startMarginMeters", "首端余量（米）")}
            {field("endMarginMeters", "末端余量（米）")}
          </div>
          {field("dropMeters", "下挂距离（米）")}
          <p className="wb-dim">
            按灯序从首端到末端均布，下挂距离从支撑体底部计算。
          </p>
        </>
      )}
    </section>
  );
}
