import { WorkspaceSurface } from "../workbench/WorkspaceSurface";
import { forwardRef, useImperativeHandle, useRef, useState } from "react";
import type {
  EditCommand,
  EditOperation,
  ProjectView,
} from "../../application-host";
import {
  withMotion,
  FixtureFieldError,
  channelLabels,
  profileDefinition,
  profileDraft,
  profileMatches,
  type ProfileDraft,
} from "../../fixture-tools";
import { DeleteDialog } from "../workbench/DeleteDialog";
import "./fixtures.css";
export interface ProfileHandle {
  collect(): EditOperation[];
  accept(): void;
}
export const ProfileWorkspace = forwardRef<
  ProfileHandle,
  {
    project: ProjectView;
    visible: boolean;
    busy: boolean;
    error: string;
    onPending(pending: boolean): void;
    beforeChange(): Promise<boolean>;
    onEdit(command: EditCommand): Promise<ProjectView | null>;
    onBack(): void;
  }
>(function ProfileWorkspace(
  { project, visible, busy, error, onPending, beforeChange, onEdit, onBack },
  ref,
) {
  const [selectedId, setSelectedId] = useState(""),
    [query, setQuery] = useState("");
  const [draft, setDraftState] = useState<ProfileDraft | null>(null),
    draftRef = useRef<ProfileDraft | null>(null);
  const editingId = useRef<string | null>(null),
    form = useRef<HTMLFormElement>(null);
  const [remove, setRemove] = useState(false);
  const profile =
    project.profiles.find((p) => p.id === selectedId) ??
    project.profiles.at(-1);
  const used = project.fixtures.filter((f) => f.profileId === profile?.id);
  function setDraft(value: ProfileDraft | null) {
    draftRef.current = value;
    setDraftState(value);
    onPending(value !== null);
  }
  function cancel() {
    setDraft(null);
  }
  function collect(): EditOperation[] {
    if (!draftRef.current) return [];
    try {
      return [
        {
          op: "fixture",
          command: {
            op: "saveProfile",
            id: editingId.current,
            definition: profileDefinition(draftRef.current),
          },
        },
      ];
    } catch (reason) {
      if (reason instanceof FixtureFieldError) {
        const field = form.current?.elements.namedItem(
          reason.field,
        ) as HTMLInputElement | null;
        field?.focus();
        field?.setCustomValidity(reason.message);
        field?.reportValidity();
      }
      throw reason;
    }
  }
  useImperativeHandle(ref, () => ({
    collect,
    accept() {
      if (draftRef.current) {
        setSelectedId(editingId.current ?? "");
        setDraft(null);
      }
    },
  }));
  async function begin(copy: boolean) {
    if (!(await beforeChange())) return;
    editingId.current = copy ? null : (profile?.id ?? null);
    const next = profileDraft(profile);
    if (copy) next.name = `${next.name} 副本`;
    setDraft(next);
  }
  const shown = project.profiles.filter((p) => profileMatches(p, query));
  const value = draft ?? (profile ? profileDraft(profile) : null);
  const numberInput = (
    name: string,
    label: string,
    raw: string,
    min: number,
    max: number,
    change: (v: string) => void,
  ) => (
    <label>
      {label}
      <input
        name={name}
        aria-label={label}
        type="number"
        min={min}
        max={max}
        step={name.endsWith("percent") ? "any" : 1}
        value={raw}
        required
        onChange={(e) => change(e.target.value)}
      />
    </label>
  );
  return (
    <WorkspaceSurface
      visible={visible}
      className="fixture-library"
      label="工程灯库"
    >
      <header>
        <div>
          <span className="wb-eyebrow">灯具管理</span>
          <h1>工程灯库</h1>
        </div>
        <button disabled={busy} onClick={onBack}>
          返回灯具配适
        </button>
        <button
          className="wb-primary"
          disabled={busy}
          onClick={async () => {
            if (await beforeChange()) {
              editingId.current = null;
              setDraft(profileDraft());
            }
          }}
        >
          新建模式
        </button>
      </header>
      <div className="fixture-library-body">
        <aside>
          <input
            type="search"
            aria-label="搜索灯具模式"
            placeholder="搜索厂家、型号或模式"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <p className="wb-dim">{shown.length} 个模式</p>
          <div className="profile-list">
            {shown.map((p) => (
              <button
                key={p.id}
                aria-pressed={profile?.id === p.id}
                className={profile?.id === p.id ? "selected" : ""}
                disabled={busy}
                onClick={async () => {
                  if (await beforeChange()) setSelectedId(p.id);
                }}
              >
                <strong>{p.name}</strong>
                <span>
                  {p.manufacturer} · {p.model}
                </span>
                <small>
                  {p.mode} · {p.footprint} 通道 ·{" "}
                  {project.fixtures.filter((f) => f.profileId === p.id).length}{" "}
                  台
                </small>
              </button>
            ))}
            {!shown.length && (
              <p className="wb-dim">
                没有匹配模式
                {query && (
                  <button onClick={() => setQuery("")}>清除搜索</button>
                )}
              </p>
            )}
          </div>
        </aside>
        <main>
          {value ? (
            <form
              ref={form}
              noValidate
              onSubmit={(e) => {
                e.preventDefault();
                void beforeChange();
              }}
              onInputCapture={(e) => {
                if (e.target instanceof HTMLInputElement)
                  e.target.setCustomValidity("");
              }}
              onKeyDown={(e) => {
                if (e.key === "Escape" && draft && !busy) {
                  e.preventDefault();
                  cancel();
                }
              }}
            >
              <div className="profile-title">
                <div>
                  <h2>
                    {draft
                      ? editingId.current
                        ? "编辑模式"
                        : "新建模式"
                      : profile?.name}
                  </h2>
                  <p className="wb-dim">
                    {draft
                      ? "粗调、细调按灯具说明书填写，通道从 1 开始。"
                      : `${used.length} 台灯具使用 · ${profile?.manufacturer} / ${profile?.model}`}
                  </p>
                </div>
                {!draft && (
                  <div className="profile-actions">
                    <button
                      type="button"
                      disabled={busy || !profile?.authorable || used.length > 0}
                      title={
                        used.length
                          ? "使用中请复制后替换灯具"
                          : "编辑未使用模式"
                      }
                      onClick={() => void begin(false)}
                    >
                      编辑模式
                    </button>
                    <button
                      type="button"
                      disabled={busy || !profile?.authorable}
                      onClick={() => void begin(true)}
                    >
                      复制模式
                    </button>
                    <button
                      type="button"
                      disabled={busy || used.length > 0}
                      onClick={() => setRemove(true)}
                    >
                      删除模式
                    </button>
                  </div>
                )}
              </div>
              {!profile?.authorable && !draft ? (
                <p>此模式包含当前编辑器尚未支持的属性，保留原始定义。</p>
              ) : (
                <>
                  <fieldset disabled={busy || !draft}>
                    <div className="profile-meta">
                      {(
                        [
                          ["name", "模式名称"],
                          ["manufacturer", "厂家"],
                          ["model", "型号"],
                          ["mode", "模式标识"],
                        ] as const
                      ).map(([key, label]) => (
                        <label key={key}>
                          {label}
                          <input
                            name={key}
                            aria-label={label}
                            required
                            maxLength={256}
                            value={value[key]}
                            onChange={(e) =>
                              setDraft({ ...value, [key]: e.target.value })
                            }
                          />
                        </label>
                      ))}
                      {numberInput(
                        "footprint",
                        "占用通道数",
                        value.footprint,
                        1,
                        512,
                        (v) => setDraft({ ...value, footprint: v }),
                      )}
                      <label>
                        功能组合
                        <select
                          name="family"
                          aria-label="功能组合"
                          value={
                            value.channels.filter(
                              (c) =>
                                c.attribute !== "pan" && c.attribute !== "tilt",
                            ).length === 1
                              ? "dimmer"
                              : value.channels.filter(
                                    (c) =>
                                      c.attribute !== "pan" &&
                                      c.attribute !== "tilt",
                                  ).length === 3
                                ? "rgb"
                                : "rgbd"
                          }
                          onChange={(e) => {
                            const keys =
                              e.target.value === "dimmer"
                                ? ["dimmer"]
                                : e.target.value === "rgb"
                                  ? ["red", "green", "blue"]
                                  : ["dimmer", "red", "green", "blue"];
                            const next: ProfileDraft = {
                              ...value,
                              footprint: String(keys.length),
                              channels: keys.map((attribute, i) => ({
                                attribute,
                                coarse: String(i + 1),
                                fine: "",
                                bits: "8",
                                percent: "0",
                              })),
                            };
                            const physical = value.positioning;
                            const moved = physical
                              ? withMotion(next, true)
                              : next;
                            setDraft({ ...moved, positioning: physical });
                          }}
                        >
                          <option value="dimmer">调光</option>
                          <option value="rgb">RGB 三原色</option>
                          <option value="rgbd">调光与 RGB</option>
                        </select>
                      </label>
                    </div>
                    <label className="profile-motion-toggle">
                      <input
                        type="checkbox"
                        checked={Boolean(value.positioning)}
                        onChange={(e) =>
                          setDraft(withMotion(value, e.target.checked))
                        }
                      />
                      两轴摇头灯
                    </label>
                    {value.positioning && (
                      <section className="profile-motion">
                        <h3>轴行程与输出方向</h3>
                        <p className="wb-dim">
                          按厂家通道表填写物理角度。零角光束沿灯具局部下方；安装朝向在舞台设置。此模型仅支持相交正交两轴。
                        </p>
                        {(["pan", "tilt"] as const).map((axis) => (
                          <div key={axis} className="profile-meta">
                            {(["minDegrees", "maxDegrees"] as const).map(
                              (key) => (
                                <label key={key}>
                                  {channelLabels[axis]}
                                  {key === "minDegrees" ? "最小" : "最大"}
                                  角度（°）
                                  <input
                                    name={`${axis}-${key}`}
                                    aria-label={`${channelLabels[axis]}${key === "minDegrees" ? "最小" : "最大"}角度`}
                                    value={value.positioning![axis][key]}
                                    onChange={(e) =>
                                      setDraft({
                                        ...value,
                                        positioning: {
                                          ...value.positioning!,
                                          [axis]: {
                                            ...value.positioning![axis],
                                            [key]: e.target.value,
                                          },
                                        },
                                      })
                                    }
                                  />
                                </label>
                              ),
                            )}
                            <label>
                              {channelLabels[axis]}输出映射
                              <select
                                aria-label={`${channelLabels[axis]}输出映射`}
                                value={
                                  value.positioning![axis].reversed
                                    ? "reverse"
                                    : "forward"
                                }
                                onChange={(e) =>
                                  setDraft({
                                    ...value,
                                    positioning: {
                                      ...value.positioning!,
                                      [axis]: {
                                        ...value.positioning![axis],
                                        reversed: e.target.value === "reverse",
                                      },
                                    },
                                  })
                                }
                              >
                                <option value="forward">低值 → 最小角度</option>
                                <option value="reverse">低值 → 最大角度</option>
                              </select>
                            </label>
                          </div>
                        ))}
                      </section>
                    )}
                    <h3>属性与物理通道</h3>
                    <div className="profile-channels">
                      {value.channels.map((c, i) => {
                        const change = (patch: Partial<typeof c>) =>
                          setDraft({
                            ...value,
                            channels: value.channels.map((old, index) =>
                              index === i ? { ...old, ...patch } : old,
                            ),
                          });
                        return (
                          <div className="profile-channel" key={c.attribute}>
                            <strong>
                              {channelLabels[c.attribute] ?? c.attribute}
                            </strong>
                            <label>
                              精度
                              <select
                                aria-label={`${channelLabels[c.attribute]}精度`}
                                value={c.bits}
                                onChange={(e) =>
                                  change({ bits: e.target.value as "8" | "16" })
                                }
                              >
                                <option value="8">8 位</option>
                                <option value="16">16 位</option>
                              </select>
                            </label>
                            {numberInput(
                              `channel-${i}-coarse`,
                              `${channelLabels[c.attribute]}粗调通道`,
                              c.coarse,
                              1,
                              512,
                              (v) => change({ coarse: v }),
                            )}
                            {c.bits === "16" ? (
                              numberInput(
                                `channel-${i}-fine`,
                                `${channelLabels[c.attribute]}细调通道`,
                                c.fine,
                                1,
                                512,
                                (v) => change({ fine: v }),
                              )
                            ) : (
                              <span className="wb-dim">—</span>
                            )}
                            {numberInput(
                              `channel-${i}-percent`,
                              `${channelLabels[c.attribute]}默认值（%）`,
                              c.percent,
                              0,
                              100,
                              (v) => change({ percent: v }),
                            )}
                          </div>
                        );
                      })}
                    </div>
                  </fieldset>
                  <p className="wb-dim">
                    未映射通道输出 0。仅适用于全范围线性调光和
                    RGB，以及已定义行程的水平／垂直轴；频闪、复位、色盘和外部电机需专用定义。
                  </p>
                  <ChannelStrip draft={value} />
                </>
              )}
              {error && (
                <p className="wb-library-error" role="alert">
                  {error}
                </p>
              )}
              {draft && (
                <div className="profile-actions profile-footer">
                  <button type="button" disabled={busy} onClick={cancel}>
                    取消编辑
                  </button>
                  <button className="wb-primary" disabled={busy}>
                    保存模式
                  </button>
                </div>
              )}
              {!draft && used.length > 0 && (
                <section className="profile-usage">
                  <h3>使用此模式</h3>
                  {used.map((f) => (
                    <span key={f.id}>
                      {f.name} · {f.universe ?? "—"}.{f.address ?? "—"}
                    </span>
                  ))}
                </section>
              )}
            </form>
          ) : (
            <div className="wb-empty">
              <h2>尚无灯具模式</h2>
              <p>新建模式后即可添加灯具。</p>
            </div>
          )}
        </main>
      </div>
      {remove && profile && (
        <DeleteDialog
          name={profile.name}
          description={`删除“${profile.name}”？可通过撤销恢复。`}
          busy={busy}
          error={error}
          onCancel={() => setRemove(false)}
          onDelete={async () => {
            if (
              await onEdit({
                op: "fixture",
                command: { op: "removeProfile", id: profile.id },
              })
            ) {
              setRemove(false);
              setSelectedId("");
            }
          }}
        />
      )}
    </WorkspaceSurface>
  );
});
function ChannelStrip({ draft }: { draft: ProfileDraft }) {
  const width = Number(draft.footprint);
  if (!Number.isInteger(width) || width < 1 || width > 512) return null;
  const slots = Array.from({ length: width }, (_, i) =>
    draft.channels.flatMap((c) => [
      ...(Number(c.coarse) === i + 1
        ? [`${channelLabels[c.attribute]}粗调`]
        : []),
      ...(c.bits === "16" && Number(c.fine) === i + 1
        ? [`${channelLabels[c.attribute]}细调`]
        : []),
    ]),
  );
  return (
    <section className="profile-strip" aria-label="模式通道分布">
      <h3>通道分布</h3>
      <div>
        {slots.map((labels, i) => (
          <span
            key={i}
            className={
              labels.length > 1 ? "conflict" : labels.length ? "mapped" : ""
            }
            title={labels.join(" / ") || "未映射，输出 0"}
          >
            <b>{i + 1}</b>
            <small>{labels.join(" / ") || "空余"}</small>
          </span>
        ))}
      </div>
    </section>
  );
}
