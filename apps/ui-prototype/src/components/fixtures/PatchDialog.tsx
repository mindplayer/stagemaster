import { useEffect, useRef, useState } from "react";
import type { EditCommand, ProjectView } from "../../application-host";
import {
  availablePatch,
  compatibleProfile,
  FixtureFieldError,
  patchPlan,
} from "../../fixture-tools";
import { OrderedFixturePicker } from "../stage/OrderedFixturePicker";
import { LibraryDialog } from "../workbench/LibraryDialog";
import { profileExchangeReview } from "../../profile-exchange-review";
import { ProfileExchangeReview } from "./ProfileExchangeReview";
export function PatchDialog({
  project,
  initialIds,
  exchange,
  busy,
  error,
  onCancel,
  onEdit,
}: {
  project: ProjectView;
  initialIds: string[];
  exchange: boolean;
  busy: boolean;
  error: string;
  onCancel(): void;
  onEdit(command: EditCommand): Promise<boolean>;
}) {
  const [ids, setIds] = useState(initialIds),
    [profileId, setProfileId] = useState("");
  const [acceptedReview, setAcceptedReview] = useState("");
  const [readdress, setReaddress] = useState(!exchange),
    [address, setAddress] = useState(
      String(
        project.fixtures.find((f) => f.id === initialIds[0])?.address ?? 1,
      ),
    ),
    [universe, setUniverse] = useState(
      String(
        project.fixtures.find((f) => f.id === initialIds[0])?.universe ?? 1,
      ),
    ),
    [gap, setGap] = useState("0");
  const fields = useRef<HTMLDivElement>(null);
  const fixtures = ids.flatMap((id) =>
    project.fixtures.filter((f) => f.id === id),
  );
  const profiles = project.profiles.filter(
    (p) => p.authorable && compatibleProfile(fixtures, p, true),
  );
  const profile = profiles.find((p) => p.id === profileId);
  const review =
    exchange && profile && fixtures.length
      ? profileExchangeReview(project, fixtures, profile)
      : null;
  const remapAccepted = !!review && acceptedReview === review.signature;
  useEffect(() => setAcceptedReview(""), [review?.signature]);
  const layout = {
    address: Number(address || NaN),
    universe: Number(universe || NaN),
    gap: Number(gap || NaN),
  };
  let preview = "",
    issue = "";
  if (readdress && (!exchange || profile))
    try {
      const rows = patchPlan(project, ids, layout, profile);
      preview = rows
        .map((r) => `${r.fixture.name} → ${r.universe}.${r.address}–${r.end}`)
        .join("\n");
    } catch (e) {
      issue = e instanceof Error ? e.message : String(e);
    }
  else if (exchange && profile)
    preview = fixtures
      .map(
        (f) =>
          `${f.name} → ${profile.name} · ${f.universe ?? "—"}.${f.address ?? "—"}`,
      )
      .join("\n");
  async function submit() {
    try {
      if (exchange && !profile)
        throw new FixtureFieldError("profileId", "请选择属性相容的目标模式");
      if (review?.needsRemap && !remapAccepted)
        throw new FixtureFieldError(
          "colorRemap",
          "请核对色盘差异并选择是否采用新通道值",
        );
      if (readdress) patchPlan(project, ids, layout, profile);
      else if (!ids.length)
        throw new FixtureFieldError("selection", "请先选择灯具");
      return onEdit({
        op: "fixture",
        command: exchange
          ? {
              op: "exchange",
              fixtureIds: ids,
              profileId: profile!.id,
              layout: readdress ? layout : null,
              allowColorSlotRemap: !!review?.needsRemap && remapAccepted,
            }
          : { op: "repatch", fixtureIds: ids, layout },
      });
    } catch (e) {
      if (e instanceof FixtureFieldError) {
        const el = fields.current?.querySelector<
          HTMLInputElement | HTMLSelectElement
        >(`[name="${e.field}"]`);
        el?.focus();
        el?.setCustomValidity(e.message);
        el?.reportValidity();
      }
      throw e;
    }
  }
  return (
    <LibraryDialog
      title={exchange ? "替换灯具模式" : "批量配适"}
      busy={busy}
      error={error}
      onCancel={onCancel}
      onSubmit={submit}
      submit={exchange ? "替换所选灯具" : "应用配适"}
    >
      <div
        ref={fields}
        className="patch-dialog-fields"
        onChangeCapture={(e) => {
          if (
            e.target instanceof HTMLSelectElement ||
            e.target instanceof HTMLInputElement
          )
            e.target.setCustomValidity("");
        }}
      >
        <OrderedFixturePicker
          project={project}
          ids={ids}
          setIds={(next) => {
            setAcceptedReview("");
            setIds(next);
          }}
          purpose="配适"
        />
        <div className="patch-settings">
          {exchange && (
            <>
              <label>
                目标模式
                <select
                  name="profileId"
                  aria-label="目标模式"
                  value={profile?.id ?? ""}
                  onChange={(e) => {
                    setAcceptedReview("");
                    setProfileId(e.target.value);
                  }}
                >
                  <option value="">选择相容的灯具模式</option>
                  {profiles.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name} · {p.footprint} 通道
                      {!compatibleProfile(fixtures, p) ? " · 色盘值待核对" : ""}
                    </option>
                  ))}
                </select>
              </label>
              <p className="wb-dim">
                保留场景、效果、灯组与灯位，只替换所选灯具。名称、色块与默认值采用新模式，未记录或释放的属性可能改变；色盘控制值有差异时须核对。
              </p>
              {review && (
                <ProfileExchangeReview
                  review={review}
                  targetName={profile!.name}
                  accepted={remapAccepted}
                  onAccept={(value) =>
                    setAcceptedReview(value ? review.signature : "")
                  }
                />
              )}
              <label className="patch-check">
                <input
                  type="checkbox"
                  checked={readdress}
                  onChange={(e) => setReaddress(e.target.checked)}
                />
                按当前灯序重新配适
              </label>
            </>
          )}
          {readdress && (
            <>
              <div className="wb-field-pair">
                <label>
                  线路
                  <input
                    name="universe"
                    aria-label="配适线路"
                    type="number"
                    min={1}
                    max={65535}
                    required
                    value={universe}
                    onChange={(e) => setUniverse(e.target.value)}
                  />
                </label>
                <label>
                  起始地址
                  <input
                    name="address"
                    aria-label="配适起始地址"
                    type="number"
                    min={1}
                    max={512}
                    required
                    value={address}
                    onChange={(e) => setAddress(e.target.value)}
                  />
                </label>
              </div>
              <label>
                灯间空余通道
                <input
                  name="gap"
                  aria-label="灯间空余通道"
                  type="number"
                  min={0}
                  max={511}
                  required
                  value={gap}
                  onChange={(e) => setGap(e.target.value)}
                />
              </label>
              <button
                type="button"
                disabled={!ids.length || (exchange && !profile)}
                onClick={() => {
                  const n = availablePatch(
                    project,
                    ids,
                    layout.universe,
                    layout.gap,
                    profile,
                  );
                  if (n !== null) setAddress(String(n));
                  else {
                    const input =
                      fields.current?.querySelector<HTMLInputElement>(
                        '[name="address"]',
                      );
                    input?.setCustomValidity(
                      "当前线路没有足够的连续空间，请减少灯具或调整间隔",
                    );
                    input?.reportValidity();
                  }
                }}
              >
                寻找可用地址
              </button>
            </>
          )}
          {issue && <p className="wb-library-error">{issue}</p>}
          {preview && (
            <pre className="patch-preview" aria-label="配适变更预览">
              {preview}
            </pre>
          )}
        </div>
      </div>
    </LibraryDialog>
  );
}
