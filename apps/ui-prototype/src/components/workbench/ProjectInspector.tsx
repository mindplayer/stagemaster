import type { RefObject } from "react";
import { TrashIcon } from "@phosphor-icons/react";
import type { ProjectView, FixtureView } from "../../application-host";
export type ProjectForm = {
  kind: "addFixture" | "fixture" | "info" | "scene";
  id: string;
  name: string;
  description: string;
  profileId: string;
  domainId: string;
  universe: string;
  address: string;
};

export function ProjectInspector({
  form,
  htmlProjectForm,
  busy,
  pending,
  page,
  project,
  activeFixture,
  onChange,
  onApply,
  onCancel,
  onDelete,
}: {
  form: ProjectForm | null;
  htmlProjectForm: RefObject<HTMLFormElement | null>;
  busy: boolean;
  pending: boolean;
  page: "fixtures" | "scenes" | "settings";
  project: ProjectView;
  activeFixture: FixtureView | undefined;
  onChange: (patch: Partial<ProjectForm>) => void;
  onApply: () => void;
  onCancel: () => void;
  onDelete: () => void;
}) {
  return (
    <aside className="wb-inspector">
      {form ? (
        <form
          ref={htmlProjectForm}
          onSubmit={(event) => {
            event.preventDefault();
            onApply();
          }}
        >
          <h2>
            {form.kind === "addFixture"
              ? "添加灯具"
              : form.kind === "fixture"
                ? "灯具参数"
                : form.kind === "scene"
                  ? "场景参数"
                  : "工程信息"}
          </h2>
          <fieldset disabled={busy}>
            <label>
              名称
              <input
                aria-label="名称"
                required
                maxLength={256}
                value={form.name}
                onChange={(event) => onChange({ name: event.target.value })}
              />
            </label>
            {form.kind === "info" && (
              <label>
                备注
                <textarea
                  aria-label="备注"
                  maxLength={8192}
                  rows={6}
                  value={form.description}
                  onChange={(event) =>
                    onChange({ description: event.target.value })
                  }
                />
              </label>
            )}
            {form.kind === "addFixture" && (
              <>
                <label>
                  灯具模式
                  <select
                    aria-label="灯具模式"
                    value={form.profileId}
                    onChange={(event) =>
                      onChange({ profileId: event.target.value })
                    }
                  >
                    {project.profiles.map((profile) => (
                      <option key={profile.id} value={profile.id}>
                        {profile.name}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  输出域
                  <select
                    aria-label="输出域"
                    value={form.domainId}
                    onChange={(event) =>
                      onChange({ domainId: event.target.value })
                    }
                  >
                    {project.domains.map((domain) => (
                      <option key={domain.id} value={domain.id}>
                        {domain.name}
                      </option>
                    ))}
                  </select>
                </label>
              </>
            )}
            {(form.kind === "fixture" || form.kind === "addFixture") && (
              <>
                <div className="wb-field-pair">
                  <label>
                    线路
                    <input
                      type="number"
                      aria-label="线路"
                      required
                      min={1}
                      max={65535}
                      step={1}
                      value={form.universe}
                      onChange={(event) =>
                        onChange({ universe: event.target.value })
                      }
                    />
                  </label>
                  <label>
                    起始地址
                    <input
                      type="number"
                      aria-label="起始地址"
                      required
                      min={1}
                      max={512}
                      step={1}
                      value={form.address}
                      onChange={(event) =>
                        onChange({ address: event.target.value })
                      }
                    />
                  </label>
                </div>
                {activeFixture && (
                  <p className="wb-dim">{activeFixture.profileName}</p>
                )}
              </>
            )}
            <div className="wb-form-actions">
              <button
                type="submit"
                className="wb-primary"
                disabled={!pending && form.kind !== "addFixture"}
              >
                {form.kind === "addFixture" ? "添加" : "应用"}
              </button>
              <button type="button" onClick={onCancel}>
                取消
              </button>
            </div>
            {(form.kind === "fixture" || form.kind === "scene") && (
              <button type="button" className="wb-delete" onClick={onDelete}>
                <TrashIcon />
                删除{form.kind === "fixture" ? "灯具" : "场景"}
              </button>
            )}
          </fieldset>
        </form>
      ) : (
        <div className="wb-inspector-empty">
          {page === "fixtures"
            ? "选择灯具"
            : page === "scenes"
              ? "选择场景"
              : "工程信息"}
        </div>
      )}
    </aside>
  );
}
