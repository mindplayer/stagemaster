import { ProfileMetadata } from "./ProfileMetadata";
import { ProfileMotionFields } from "./ProfileMotionFields";
import { ProfileLinearChannels } from "./ProfileLinearChannels";
import { ProfileFunctionChannels } from "./ProfileFunctionChannels";
import { ChannelStrip } from "./ProfileChannelStrip";
import { WorkspaceSurface } from "../workbench/WorkspaceSurface";
import { forwardRef, useImperativeHandle, useRef, useState } from "react";
import type {
  EditCommand,
  EditOperation,
  ProjectView,
} from "../../application-host";
import {
  FixtureFieldError,
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
        field?.setCustomValidity?.(reason.message);
        field?.reportValidity?.();
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
                if (
                  e.target instanceof HTMLInputElement ||
                  e.target instanceof HTMLSelectElement
                )
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
                    <ProfileMetadata value={value} setDraft={setDraft} />
                    <ProfileMotionFields value={value} setDraft={setDraft} />
                    <ProfileLinearChannels value={value} setDraft={setDraft} />
                    <ProfileFunctionChannels
                      value={value}
                      setDraft={setDraft}
                    />
                  </fieldset>
                  <p className="wb-dim">
                    未映射通道输出
                    0。复位、灯泡控制和外部电机不属于当前模式编辑范围。
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
