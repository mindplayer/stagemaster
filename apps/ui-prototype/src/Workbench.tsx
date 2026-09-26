import { useCallback, useEffect, useRef, useState } from "react";
import {
  PlusIcon,
  FolderOpenIcon,
  FloppyDiskIcon,
  ArrowCounterClockwiseIcon,
  ArrowClockwiseIcon,
  LightbulbIcon,
  StackIcon,
  ListNumbersIcon,
  GearSixIcon,
  XIcon,
} from "@phosphor-icons/react";
import type {
  ApplicationHost,
  EditCommand,
  EditOperation,
  FixtureView,
  ProjectRequest,
  ProjectView,
  SceneView,
  Snapshot,
} from "./application-host";
import { ProjectInspector } from "./components/workbench/ProjectInspector";
import type { ProjectForm } from "./components/workbench/ProjectInspector";
import { FixtureBrowser } from "./components/workbench/FixtureBrowser";
import { SceneLibrary } from "./components/workbench/SceneLibrary";
import { ParameterPanel } from "./components/workbench/ParameterPanel";
import type { ParameterHandle } from "./components/workbench/ParameterPanel";
import { DeleteDialog } from "./components/workbench/DeleteDialog";
import { availableAddress, fixtureMatches, uniqueName } from "./editor-tools";
import { validateEditorForm } from "./components/workbench/form-validation";
import {
  SequenceWorkspace,
  type SequenceHandle,
} from "./components/workbench/SequenceWorkspace";
import "./workbench.css";
import { ResourcePool } from "./components/workbench/ResourcePool";

const EMPTY: Snapshot = {
  generation: 0,
  project: null,
  fileName: null,
  dirty: false,
  canUndo: false,
  canRedo: false,
};
type Page = "fixtures" | "scenes" | "sequences" | "settings";
const blank = (): ProjectForm => ({
  kind: "info",
  id: "",
  name: "",
  description: "",
  profileId: "",
  domainId: "",
  universe: "1",
  address: "1",
  count: "1",
});
const fixtureForm = (f: FixtureView): ProjectForm => ({
  ...blank(),
  kind: "fixture",
  id: f.id,
  name: f.name,
  universe: String(f.universe ?? 1),
  address: String(f.address ?? 1),
});
const sceneForm = (s: SceneView): ProjectForm => ({
  ...blank(),
  kind: "scene",
  id: s.id,
  name: s.name,
});
const infoForm = (p: ProjectView): ProjectForm => ({
  ...blank(),
  name: p.name,
  description: p.description,
});

export function Workbench({ host }: { host: ApplicationHost }) {
  const [snapshot, setSnapshot] = useState(EMPTY);
  const current = useRef(EMPTY);
  const [busy, setBusy] = useState(false);
  const queue = useRef(Promise.resolve(true));
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [page, setPage] = useState<Page>("fixtures");
  const [patchId, setPatchId] = useState("");
  const [sceneId, setSceneId] = useState("");
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [patchQuery, setPatchQuery] = useState("");
  const [fixtureQuery, setFixtureQuery] = useState("");
  const [sceneQuery, setSceneQuery] = useState("");
  const [onlySelected, setOnlySelected] = useState(false);
  const [form, setFormState] = useState<ProjectForm | null>(null);
  const formRef = useRef<ProjectForm | null>(null);
  const [pending, setPending] = useState(false);
  const pendingRef = useRef(false);
  const [parameterPending, setParameterPending] = useState(false);
  const parameters = useRef<ParameterHandle>(null);
  const sequences = useRef<SequenceHandle>(null);
  const [sequencePending, setSequencePending] = useState(false);
  const htmlProjectForm = useRef<HTMLFormElement>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const actions = useRef({
    close: () => {},
    save: (_as: boolean) => {},
    file: (_kind: "new" | "open") => {},
    history: (_redo: boolean) => {},
    duplicate: () => {},
    selectAll: () => {},
  });
  const project = snapshot.project;
  const selected = selectedIds.filter((id) =>
    project?.fixtures.some((f) => f.id === id),
  );
  const activeScene = project?.scenes.find((s) => s.id === sceneId);
  const activeFixture = project?.fixtures.find((f) => f.id === patchId);
  const dirty =
    snapshot.dirty || pending || parameterPending || sequencePending;

  function setForm(next: ProjectForm | null, changed = false) {
    formRef.current = next;
    setFormState(next);
    pendingRef.current = changed;
    setPending(changed);
  }
  const request = useCallback(
    async (value: ProjectRequest) => {
      const next = await host.request(value);
      current.current = next;
      setSnapshot(next);
      return next;
    },
    [host],
  );
  async function edit(command: EditCommand) {
    await request({
      kind: "edit",
      generation: current.current.generation,
      command,
    });
  }
  async function flush() {
    const draft = formRef.current;
    const commands: EditOperation[] = [];
    if (draft && pendingRef.current) {
      validateEditorForm(htmlProjectForm.current);
      if (!draft.name.trim()) throw new Error("名称不能为空");
      if (draft.kind === "info")
        commands.push({
          op: "setInfo",
          name: draft.name.trim(),
          description: draft.description,
        });
      else if (draft.kind === "scene")
        commands.push({
          op: "renameScene",
          id: draft.id,
          name: draft.name.trim(),
        });
      else if (draft.kind === "fixture")
        commands.push({
          op: "updateFixture",
          id: draft.id,
          name: draft.name.trim(),
          universe: Number(draft.universe),
          address: Number(draft.address),
        });
      else {
        const p = current.current.project!;
        const footprint = p.profiles.find(
          (profile) => profile.id === draft.profileId,
        )?.footprint;
        if (!footprint) throw new Error("请选择灯具模式");
        const count = Number(draft.count);
        if (!Number.isInteger(count) || count < 1 || count > 128)
          throw new Error("一次可添加 1–128 台灯具");
        if (Number(draft.address) + count * footprint - 1 > 512)
          throw new Error(
            "这批灯具超出本线路的 512 个地址，请减少数量或调整起始地址",
          );
        const names = p.fixtures.map((f) => f.name);
        for (let i = 0; i < count; i++) {
          const name = uniqueName(
            count > 1 ? `${draft.name.trim()} ${i + 1}` : draft.name.trim(),
            names,
          );
          names.push(name);
          commands.push({
            op: "addFixture",
            name,
            profileId: draft.profileId,
            domainId: draft.domainId,
            universe: Number(draft.universe),
            address: Number(draft.address) + i * footprint,
          });
        }
      }
    }
    commands.push(...(parameters.current?.collect() ?? []));
    commands.push(...(sequences.current?.collect() ?? []));
    if (!commands.length) {
      sequences.current?.accept();
      return;
    }
    if (commands.length > 256)
      throw new Error("一次最多修改 256 项，请先应用部分修改");
    await edit({ op: "batch", commands });
    parameters.current?.accept();
    sequences.current?.accept();
    setParameterPending(false);
    setNotice("修改已应用");
    if (draft && pendingRef.current) {
      if (draft.kind === "addFixture") {
        const added = current.current.project!.fixtures.at(-1)!;
        setPatchId(added.id);
        setForm(fixtureForm(added));
        setNotice(`已添加 ${draft.count} 台灯具`);
      } else setForm({ ...draft, name: draft.name.trim() });
    }
  }
  function run(work: () => Promise<void>, flushFirst = true): Promise<boolean> {
    const next = queue.current.then(async () => {
      const focused = document.activeElement;
      setBusy(true);
      setError("");
      try {
        if (flushFirst) await flush();
        await work();
        return true;
      } catch (reason) {
        setError(reason instanceof Error ? reason.message : String(reason));
        return false;
      } finally {
        setBusy(false);
        requestAnimationFrame(() => {
          if (
            focused instanceof HTMLElement &&
            focused.isConnected &&
            (document.activeElement === document.body ||
              document.activeElement === document.documentElement)
          ) {
            focused.focus({ preventScroll: true });
          }
        });
      }
    });
    queue.current = next;
    return next;
  }
  function restoreForm(nextPage = page) {
    const p = current.current.project;
    if (!p) {
      setForm(null);
      return;
    }
    const fixture = p.fixtures.find((f) => f.id === patchId);
    const scene = p.scenes.find((s) => s.id === sceneId);
    setForm(
      nextPage === "sequences"
        ? null
        : nextPage === "settings"
          ? infoForm(p)
          : nextPage === "fixtures"
            ? fixture
              ? fixtureForm(fixture)
              : null
            : scene
              ? sceneForm(scene)
              : null,
    );
  }
  function fileAction(kind: "new" | "open") {
    void run(async () => {
      const previousGeneration = current.current.generation;
      const next = await request({
        kind,
        generation: current.current.generation,
      });
      if (next.project && next.generation !== previousGeneration) {
        setPage(next.project.fixtures.length ? "scenes" : "fixtures");
        setPatchId("");
        setSceneId(next.project.scenes[0]?.id ?? "");
        setSelectedIds([]);
        setPatchQuery("");
        setFixtureQuery("");
        setSceneQuery("");
        setOnlySelected(false);
        setParameterPending(false);
        setForm(
          next.project.scenes[0] && next.project.fixtures.length
            ? sceneForm(next.project.scenes[0])
            : null,
        );
        setNotice(kind === "new" ? "已创建工程" : "已打开工程");
      }
    });
  }
  function save(saveAs = false) {
    void run(async () => {
      const next = await request({
        kind: "save",
        generation: current.current.generation,
        saveAs,
      });
      if (!next.dirty && next.fileName) setNotice("工程已保存");
    });
  }
  function history(redo: boolean) {
    void run(async () => {
      await request({
        kind: "history",
        generation: current.current.generation,
        redo,
      });
      restoreForm();
      setNotice(redo ? "已重做" : "已撤销");
    });
  }
  function switchPage(next: Page) {
    void run(async () => {
      setPage(next);
      restoreForm(next);
    });
  }
  function addFixture() {
    void run(async () => {
      const p = current.current.project!;
      const profile = p.profiles[0];
      const domain = p.domains[0];
      setForm({
        ...blank(),
        kind: "addFixture",
        name: "灯具",
        profileId: profile?.id ?? "",
        domainId: domain?.id ?? "",
        address: String(
          availableAddress(p, domain?.id ?? "", 1, profile?.footprint ?? 1) ??
            1,
        ),
      });
    });
  }
  function chooseScene(scene: SceneView) {
    void run(async () => {
      setSceneId(scene.id);
      setForm(sceneForm(scene));
    });
  }
  function addScene() {
    void run(async () => {
      await edit({
        op: "addScene",
        name: uniqueName(
          "场景",
          current.current.project!.scenes.map((s) => s.name),
        ),
      });
      const scene = current.current.project!.scenes.at(-1)!;
      setSceneId(scene.id);
      setForm(sceneForm(scene));
      setSceneQuery("");
      setNotice("已创建场景");
    });
  }
  function duplicateScene() {
    if (!activeScene) return;
    void run(async () => {
      const scene = current.current.project!.scenes.find(
        (s) => s.id === sceneId,
      )!;
      await edit({
        op: "duplicateScene",
        id: scene.id,
        name: uniqueName(
          `${scene.name} 副本`,
          current.current.project!.scenes.map((s) => s.name),
        ),
      });
      const copy = current.current.project!.scenes.at(-1)!;
      setSceneId(copy.id);
      setForm(sceneForm(copy));
      setSceneQuery("");
      setNotice("已复制场景");
    });
  }
  actions.current = {
    close: () => {
      const dialog = document.querySelector<HTMLDialogElement>("dialog[open]");
      if (dialog) {
        dialog.querySelector<HTMLElement>("input,button")?.focus();
        setError("请先保存或取消当前编辑，再关闭窗口");
        return;
      }
      void run(async () => {
        await request({ kind: "close" });
      });
    },
    save,
    file: fileAction,
    history,
    duplicate: () => {
      if (page === "scenes") duplicateScene();
    },
    selectAll: () => {
      if (page === "scenes" && project)
        void run(async () => {
          setSelectedIds(
            project.fixtures
              .filter(
                (f) =>
                  fixtureMatches(f, fixtureQuery) &&
                  (!onlySelected || selected.includes(f.id)),
              )
              .map((f) => f.id),
          );
        });
    },
  };
  useEffect(() => {
    let active = true;
    if (host.kind === "desktop")
      void host
        .request({ kind: "snapshot" })
        .then((next) => {
          if (active) {
            current.current = next;
            setSnapshot(next);
          }
        })
        .catch((reason) => {
          if (active) setError(String(reason));
        });
    let dispose: (() => void) | undefined;
    void host
      .onCloseRequested(() => actions.current.close())
      .then((fn) => {
        if (active) dispose = fn;
        else fn();
      });
    return () => {
      active = false;
      dispose?.();
    };
  }, [host]);
  useEffect(() => {
    const keydown = (event: KeyboardEvent) => {
      if (
        !(event.metaKey || event.ctrlKey) ||
        host.kind !== "desktop" ||
        document.querySelector("dialog[open]")
      )
        return;
      const key = event.key.toLowerCase();
      const typing =
        (event.target instanceof HTMLInputElement &&
          !["range", "color", "checkbox", "radio", "button"].includes(
            event.target.type,
          )) ||
        event.target instanceof HTMLTextAreaElement ||
        (event.target instanceof HTMLElement && event.target.isContentEditable);
      if (key === "s" && current.current.project) {
        event.preventDefault();
        actions.current.save(event.shiftKey);
      } else if (key === "o" || key === "n") {
        event.preventDefault();
        actions.current.file(key === "o" ? "open" : "new");
      } else if (key === "z" && !typing) {
        event.preventDefault();
        actions.current.history(event.shiftKey);
      } else if (key === "d" && !typing) {
        event.preventDefault();
        actions.current.duplicate();
      } else if (key === "a" && !typing) {
        event.preventDefault();
        actions.current.selectAll();
      }
    };
    window.addEventListener("keydown", keydown);
    return () => window.removeEventListener("keydown", keydown);
  }, [host]);

  return (
    <main className="workbench">
      <header className="workbench-header">
        <div className="wb-brand">
          <span className="wb-logo">
            <LightbulbIcon weight="fill" size={22} />
          </span>
          舞台大师
        </div>
        {project && (
          <div className="wb-project-title">
            <strong>{project.name}</strong>
            <span className={dirty ? "wb-unsaved" : ""}>
              {dirty ? "未保存" : "已保存"}
            </span>
          </div>
        )}
        <div className="wb-file-actions">
          <button
            title="新建工程（⌘N / Ctrl+N）"
            disabled={busy || host.kind !== "desktop"}
            onClick={() => fileAction("new")}
          >
            <PlusIcon />
            新建
          </button>
          <button
            title="打开工程（⌘O / Ctrl+O）"
            disabled={busy || host.kind !== "desktop"}
            onClick={() => fileAction("open")}
          >
            <FolderOpenIcon />
            打开
          </button>
          <span className="wb-toolbar-separator" />
          <button
            aria-label="撤销"
            title="撤销（⌘Z / Ctrl+Z）"
            disabled={
              busy ||
              (!snapshot.canUndo &&
                !pending &&
                !parameterPending &&
                !sequencePending)
            }
            onClick={() => history(false)}
          >
            <ArrowCounterClockwiseIcon />
          </button>
          <button
            aria-label="重做"
            title="重做（⇧⌘Z / Ctrl+Shift+Z）"
            disabled={
              busy ||
              !snapshot.canRedo ||
              pending ||
              parameterPending ||
              sequencePending
            }
            onClick={() => history(true)}
          >
            <ArrowClockwiseIcon />
          </button>
          <button disabled={busy || !project} onClick={() => save(true)}>
            另存为
          </button>
          <button
            className="wb-primary"
            title="保存工程（⌘S / Ctrl+S）"
            disabled={busy || !project || !dirty}
            onClick={() => save()}
          >
            <FloppyDiskIcon />
            保存
          </button>
        </div>
      </header>
      {error && (
        <div className="wb-error" role="alert">
          <span>{error}</span>
          <button aria-label="关闭错误提示" onClick={() => setError("")}>
            <XIcon />
          </button>
        </div>
      )}
      {!project ? (
        <div className="wb-welcome">
          <LightbulbIcon size={52} weight="duotone" />
          <h1>开始编排</h1>
          <p>
            {host.kind === "browser"
              ? "请使用桌面应用打开本地工程"
              : "创建工程，或继续已有编排"}
          </p>
          <div>
            <button
              className="wb-primary"
              disabled={host.kind !== "desktop" || busy}
              onClick={() => fileAction("new")}
            >
              新建工程
            </button>
            <button
              disabled={host.kind !== "desktop" || busy}
              onClick={() => fileAction("open")}
            >
              打开工程
            </button>
          </div>
        </div>
      ) : (
        <>
          <nav className="wb-nav" aria-label="工作区">
            <button
              className={page === "scenes" ? "active" : ""}
              aria-pressed={page === "scenes"}
              disabled={busy}
              onClick={() => switchPage("scenes")}
            >
              <StackIcon />
              编排<span>{project.scenes.length}</span>
            </button>
            <button
              className={page === "sequences" ? "active" : ""}
              aria-pressed={page === "sequences"}
              disabled={busy}
              onClick={() => switchPage("sequences")}
            >
              <ListNumbersIcon />
              列表与预览<span>{project.sequences.length}</span>
            </button>
            <button
              className={page === "fixtures" ? "active" : ""}
              aria-pressed={page === "fixtures"}
              disabled={busy}
              onClick={() => switchPage("fixtures")}
            >
              <LightbulbIcon />
              灯具<span>{project.fixtures.length}</span>
            </button>
            <button
              className={page === "settings" ? "active" : ""}
              aria-pressed={page === "settings"}
              disabled={busy}
              onClick={() => switchPage("settings")}
            >
              <GearSixIcon />
              工程
            </button>
          </nav>
          <SequenceWorkspace
            key={project.id}
            ref={sequences}
            project={project}
            host={host}
            generation={snapshot.generation}
            busy={busy}
            visible={page === "sequences"}
            beforeChange={() => run(async () => {})}
            onPending={(value) => {
              setSequencePending(value);
              if (!value) setError("");
            }}
            onEdit={async (command) => {
              const ok = await run(async () => {
                await edit(command);
                setNotice("列表已更新，可撤销恢复");
              });
              return ok ? current.current.project : null;
            }}
          />
          <div
            hidden={page === "sequences"}
            className={`wb-layout ${page === "scenes" ? "wb-arrangement" : ""}`}
          >
            {page === "scenes" && (
              <SceneLibrary
                scenes={project.scenes}
                selected={activeScene?.id ?? ""}
                query={sceneQuery}
                busy={busy}
                canCreate={project.fixtures.length > 0}
                onQuery={setSceneQuery}
                onSelect={chooseScene}
                onAdd={addScene}
                onDuplicate={duplicateScene}
              />
            )}
            <section className="wb-content">
              <div className="wb-content-heading">
                <div>
                  <span className="wb-eyebrow">
                    {page === "scenes"
                      ? "灯光编排"
                      : page === "fixtures"
                        ? "灯具管理"
                        : "工程管理"}
                  </span>
                  <h1>
                    {page === "scenes"
                      ? (activeScene?.name ?? "场景编排")
                      : page === "fixtures"
                        ? "灯具配适"
                        : "工程信息"}
                  </h1>
                </div>
                {page === "fixtures" && (
                  <button
                    className="wb-primary"
                    disabled={busy}
                    onClick={addFixture}
                  >
                    <PlusIcon />
                    添加灯具
                  </button>
                )}
                {page === "scenes" && activeScene && (
                  <span className="wb-dim">
                    {activeScene.values.length} 项记录
                  </span>
                )}
              </div>
              {page === "fixtures" && (
                <FixtureBrowser
                  fixtures={project.fixtures}
                  selected={activeFixture ? [activeFixture.id] : []}
                  query={patchQuery}
                  onlySelected={false}
                  busy={busy}
                  onQuery={setPatchQuery}
                  onFilter={() => {}}
                  onSelect={(ids) => {
                    void run(async () => {
                      const fixture = current.current.project!.fixtures.find(
                        (f) => f.id === ids[0],
                      );
                      if (fixture) {
                        setPatchId(fixture.id);
                        setForm(fixtureForm(fixture));
                      }
                    });
                  }}
                  table
                />
              )}
              {page === "scenes" &&
                (activeScene ? (
                  <FixtureBrowser
                    fixtures={project.fixtures}
                    selected={selected}
                    scene={activeScene}
                    query={fixtureQuery}
                    onlySelected={onlySelected}
                    busy={busy}
                    onQuery={setFixtureQuery}
                    onFilter={setOnlySelected}
                    onSelect={(ids) => {
                      void run(async () => {
                        setSelectedIds(ids);
                      });
                    }}
                  />
                ) : (
                  <div className="wb-empty">
                    <StackIcon size={40} />
                    <h2>
                      {project.scenes.length
                        ? "选择一个场景"
                        : "创建第一个场景"}
                    </h2>
                    {project.fixtures.length ? (
                      <button
                        className="wb-primary"
                        disabled={busy}
                        onClick={addScene}
                      >
                        新建场景
                      </button>
                    ) : (
                      <button
                        disabled={busy}
                        onClick={() => switchPage("fixtures")}
                      >
                        添加灯具
                      </button>
                    )}
                  </div>
                ))}
              <ResourcePool
                key={project.id}
                project={project}
                scene={activeScene}
                selected={selected}
                busy={busy}
                error={error}
                visible={page === "scenes"}
                beforeChange={() => run(async () => {})}
                onEdit={async (command) => {
                  const ok = await run(async () => {
                    await edit({ op: "library", command });
                    setNotice("资源已更新，可撤销恢复");
                  });
                  return ok ? current.current.project : null;
                }}
                onSelect={(ids) =>
                  run(async () => {
                    setSelectedIds(ids);
                  })
                }
              />
              {page === "settings" && (
                <dl className="wb-project-summary">
                  <dt>工程名称</dt>
                  <dd>{project.name}</dd>
                  <dt>文件位置</dt>
                  <dd>{snapshot.fileName ?? "尚未保存"}</dd>
                  <dt>灯具</dt>
                  <dd>{project.fixtures.length} 台</dd>
                  <dt>场景</dt>
                  <dd>{project.scenes.length} 个</dd>
                </dl>
              )}
            </section>
            <div className="wb-properties">
              {page === "scenes" && activeScene && (
                <ParameterPanel
                  key={`${project.id}:${activeScene.id}:${selected.join(",")}`}
                  ref={parameters}
                  scene={activeScene}
                  fixtures={selected.map(
                    (id) => project.fixtures.find((f) => f.id === id)!,
                  )}
                  busy={busy}
                  onApply={() => {
                    void run(async () => {});
                  }}
                  onPending={(value) => {
                    setParameterPending(value);
                    if (!value) setError("");
                  }}
                />
              )}
              <ProjectInspector
                form={form}
                htmlProjectForm={htmlProjectForm}
                busy={busy}
                pending={pending}
                page={page === "sequences" ? "scenes" : page}
                project={project}
                activeFixture={page === "fixtures" ? activeFixture : undefined}
                onChange={(patch) => {
                  if (formRef.current)
                    setForm({ ...formRef.current, ...patch }, true);
                }}
                onApply={() => {
                  if (formRef.current?.kind === "addFixture") {
                    pendingRef.current = true;
                    setPending(true);
                  }
                  void run(async () => {});
                }}
                onCancel={() => {
                  restoreForm();
                  setError("");
                }}
                onDelete={() => {
                  setError("");
                  setConfirmDelete(true);
                }}
              />
            </div>
          </div>
          <footer className="wb-status">
            <span role="status">
              {busy
                ? "正在处理…"
                : error
                  ? "修改未完成"
                  : pending || parameterPending || sequencePending
                    ? "有待应用的修改"
                    : notice || (dirty ? "有未保存的修改" : "已保存")}
            </span>
            <span>
              {snapshot.fileName?.split(/[\\/]/).at(-1) ?? "尚未保存到文件"}
              <i />
              {project.fixtures.length} 台灯具 · {project.scenes.length} 个场景
            </span>
          </footer>
        </>
      )}
      {confirmDelete && (
        <DeleteDialog
          name={form?.name ?? ""}
          error={error}
          busy={busy}
          onCancel={() => setConfirmDelete(false)}
          onDelete={() => {
            const target = formRef.current;
            void run(async () => {
              if (
                !target ||
                (target.kind !== "fixture" && target.kind !== "scene")
              )
                return;
              try {
                await edit({
                  op:
                    target.kind === "fixture" ? "removeFixture" : "removeScene",
                  id: target.id,
                });
              } catch (reason) {
                if (
                  target.kind === "fixture" &&
                  String(reason).includes("引用")
                ) {
                  throw new Error(
                    "这台灯具仍被场景、分组或预设使用。请先移除相关记录，再删除灯具。",
                  );
                }
                throw reason;
              }
              setForm(null);
              setParameterPending(false);
              setConfirmDelete(false);
              setNotice("已删除，可撤销恢复");
            }, false);
          }}
        />
      )}
    </main>
  );
}
