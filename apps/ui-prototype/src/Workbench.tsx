import { useCallback, useEffect, useRef, useState } from "react";
import {
  PlusIcon,
  FolderOpenIcon,
  FloppyDiskIcon,
  ArrowCounterClockwiseIcon,
  ArrowClockwiseIcon,
  LightbulbIcon,
  StackIcon,
  GearSixIcon,
  XIcon,
} from "@phosphor-icons/react";
import type {
  ApplicationHost,
  EditCommand,
  FixtureView,
  ProjectRequest,
  SceneView,
  Snapshot,
} from "./application-host";
import { ProjectInspector } from "./components/workbench/ProjectInspector";
import type { ProjectForm } from "./components/workbench/ProjectInspector";
import { FixtureTable } from "./components/workbench/FixtureTable";
import { SceneEditor } from "./components/workbench/SceneEditor";
import { DeleteDialog } from "./components/workbench/DeleteDialog";
import "./workbench.css";

const EMPTY: Snapshot = {
  generation: 0,
  project: null,
  fileName: null,
  dirty: false,
  canUndo: false,
  canRedo: false,
};
type Page = "fixtures" | "scenes" | "settings";
function blankProjectForm(): ProjectForm {
  return {
    kind: "info",
    id: "",
    name: "",
    description: "",
    profileId: "",
    domainId: "",
    universe: "1",
    address: "1",
  };
}

export function Workbench({ host }: { host: ApplicationHost }) {
  const [snapshot, setSnapshot] = useState(EMPTY);
  const current = useRef(EMPTY);
  const [busy, setBusy] = useState(false);
  const lock = useRef(false);
  const [error, setError] = useState("");
  const [page, setPage] = useState<Page>("fixtures");
  const [selected, setSelected] = useState("");
  const [form, setProjectFormState] = useState<ProjectForm | null>(null);
  const formRef = useRef<ProjectForm | null>(null);
  const [pending, setPending] = useState(false);
  const pendingRef = useRef(false);
  const htmlProjectForm = useRef<HTMLFormElement>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const operation = useRef<
    (work: () => Promise<void>, flush?: boolean) => Promise<boolean>
  >(async () => false);
  const project = snapshot.project;
  const dirty = snapshot.dirty || pending;

  function setProjectForm(next: ProjectForm | null, changed = false) {
    formRef.current = next;
    setProjectFormState(next);
    pendingRef.current = changed;
    setPending(changed);
  }
  function changeProjectForm(patch: Partial<ProjectForm>) {
    if (formRef.current) setProjectForm({ ...formRef.current, ...patch }, true);
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
    if (!draft || !pendingRef.current) return;
    if (htmlProjectForm.current && !htmlProjectForm.current.reportValidity())
      throw new Error("请先检查当前填写的内容");
    let command: EditCommand;
    if (draft.kind === "info")
      command = {
        op: "setInfo",
        name: draft.name.trim(),
        description: draft.description,
      };
    else if (draft.kind === "scene")
      command = { op: "renameScene", id: draft.id, name: draft.name.trim() };
    else if (draft.kind === "fixture")
      command = {
        op: "updateFixture",
        id: draft.id,
        name: draft.name.trim(),
        universe: Number(draft.universe),
        address: Number(draft.address),
      };
    else
      command = {
        op: "addFixture",
        name: draft.name.trim(),
        profileId: draft.profileId,
        domainId: draft.domainId,
        universe: Number(draft.universe),
        address: Number(draft.address),
      };
    await edit(command);
    if (draft.kind === "addFixture") {
      const added = current.current.project?.fixtures.at(-1);
      if (added) {
        selectFixture(added);
        setSelected(added.id);
      }
    } else setProjectForm(draft, false);
  }
  async function run(work: () => Promise<void>, flushFirst = true) {
    if (lock.current) return false;
    lock.current = true;
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
      lock.current = false;
      setBusy(false);
    }
  }
  operation.current = run;
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
      .onCloseRequested(() => {
        void operation.current(async () => {
          await request({ kind: "close" });
        });
      })
      .then((fn) => {
        if (active) dispose = fn;
        else fn();
      });
    return () => {
      active = false;
      dispose?.();
    };
  }, [host, request]);
  useEffect(() => {
    const keydown = (event: KeyboardEvent) => {
      if (!(event.metaKey || event.ctrlKey)) return;
      const key = event.key.toLowerCase();
      if (key === "s") {
        event.preventDefault();
        if (current.current.project)
          void operation.current(async () => {
            await request({
              kind: "save",
              generation: current.current.generation,
              saveAs: event.shiftKey,
            });
          });
      }
      if (key === "o" || key === "n") {
        event.preventDefault();
        void operation.current(async () => {
          const previous = current.current.generation;
          await request({
            kind: key === "o" ? "open" : "new",
            generation: current.current.generation,
          });
          if (current.current.generation !== previous) {
            setProjectForm(null);
            setSelected("");
            setPage("fixtures");
          }
        });
      }
      if (
        key === "z" &&
        !(
          event.target instanceof HTMLInputElement ||
          event.target instanceof HTMLTextAreaElement
        )
      ) {
        event.preventDefault();
        void operation.current(async () => {
          await request({
            kind: "history",
            generation: current.current.generation,
            redo: event.shiftKey,
          });
          setProjectForm(null);
        });
      }
    };
    window.addEventListener("keydown", keydown);
    return () => window.removeEventListener("keydown", keydown);
  }, [request]);
  function selectFixture(fixture: FixtureView) {
    setProjectForm({
      ...blankProjectForm(),
      kind: "fixture",
      id: fixture.id,
      name: fixture.name,
      universe: String(fixture.universe ?? 1),
      address: String(fixture.address ?? 1),
    });
  }
  function selectScene(scene: SceneView) {
    setSelected(scene.id);
    setProjectForm({
      ...blankProjectForm(),
      kind: "scene",
      id: scene.id,
      name: scene.name,
    });
  }
  function switchPage(next: Page) {
    void run(async () => {
      setPage(next);
      setSelected("");
      setProjectForm(
        next === "settings" && current.current.project
          ? {
              ...blankProjectForm(),
              kind: "info",
              name: current.current.project.name,
              description: current.current.project.description,
            }
          : null,
      );
    });
  }
  function fileAction(kind: "new" | "open") {
    void run(async () => {
      const previous = current.current.generation;
      await request({ kind, generation: current.current.generation });
      if (current.current.generation !== previous) {
        setProjectForm(null);
        setSelected("");
        setPage("fixtures");
      }
    });
  }
  function addFixture() {
    void run(async () => {
      const p = current.current.project!;
      setSelected("");
      setProjectForm({
        ...blankProjectForm(),
        kind: "addFixture",
        name: `灯具 ${p.fixtures.length + 1}`,
        profileId: p.profiles[0]?.id ?? "",
        domainId: p.domains[0]?.id ?? "",
      });
    });
  }
  function addScene() {
    void run(async () => {
      await edit({
        op: "addScene",
        name: `场景 ${current.current.project!.scenes.length + 1}`,
      });
      const scene = current.current.project!.scenes.at(-1)!;
      selectScene(scene);
    });
  }
  const activeFixture = project?.fixtures.find((item) => item.id === selected);

  return (
    <main className="workbench">
      <header className="workbench-header">
        <div className="wb-brand">
          <span className="wb-logo">
            <LightbulbIcon weight="fill" size={21} />
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
            disabled={busy || host.kind !== "desktop"}
            onClick={() => fileAction("new")}
          >
            <PlusIcon />
            新建
          </button>
          <button
            disabled={busy || host.kind !== "desktop"}
            onClick={() => fileAction("open")}
          >
            <FolderOpenIcon />
            打开
          </button>
          {project && (
            <>
              <button
                disabled={busy || !snapshot.canUndo}
                title="撤销"
                aria-label="撤销"
                onClick={() =>
                  void run(async () => {
                    await request({
                      kind: "history",
                      generation: current.current.generation,
                      redo: false,
                    });
                    setProjectForm(null);
                  })
                }
              >
                <ArrowCounterClockwiseIcon />
              </button>
              <button
                disabled={busy || !snapshot.canRedo}
                title="重做"
                aria-label="重做"
                onClick={() =>
                  void run(async () => {
                    await request({
                      kind: "history",
                      generation: current.current.generation,
                      redo: true,
                    });
                    setProjectForm(null);
                  })
                }
              >
                <ArrowClockwiseIcon />
              </button>
              <button
                disabled={busy}
                onClick={() =>
                  void run(async () => {
                    await request({
                      kind: "save",
                      generation: current.current.generation,
                      saveAs: true,
                    });
                  })
                }
              >
                另存为
              </button>
              <button
                className="wb-primary"
                disabled={busy}
                onClick={() =>
                  void run(async () => {
                    await request({
                      kind: "save",
                      generation: current.current.generation,
                      saveAs: false,
                    });
                  })
                }
              >
                <FloppyDiskIcon />
                保存
              </button>
            </>
          )}
        </div>
      </header>
      {error && (
        <div className="wb-error" role="alert">
          <span>{error}</span>
          <button aria-label="关闭提示" onClick={() => setError("")}>
            <XIcon />
          </button>
        </div>
      )}
      {!project ? (
        <section className="wb-welcome">
          <LightbulbIcon size={48} weight="duotone" />
          <h1>舞台大师</h1>
          <div>
            <button
              className="wb-primary"
              disabled={busy || host.kind !== "desktop"}
              onClick={() => fileAction("new")}
            >
              <PlusIcon />
              新建工程
            </button>
            <button
              disabled={busy || host.kind !== "desktop"}
              onClick={() => fileAction("open")}
            >
              <FolderOpenIcon />
              打开工程
            </button>
          </div>
          {host.kind === "browser" && <p>本地工程请使用桌面应用</p>}
        </section>
      ) : (
        <>
          <nav className="wb-nav" aria-label="工作区">
            <button
              aria-current={page === "fixtures" ? "page" : undefined}
              onClick={() => switchPage("fixtures")}
              disabled={busy}
            >
              <LightbulbIcon />
              灯具配适<span>{project.fixtures.length}</span>
            </button>
            <button
              aria-current={page === "scenes" ? "page" : undefined}
              onClick={() => switchPage("scenes")}
              disabled={busy}
            >
              <StackIcon />
              场景<span>{project.scenes.length}</span>
            </button>
            <button
              aria-current={page === "settings" ? "page" : undefined}
              onClick={() => switchPage("settings")}
              disabled={busy}
            >
              <GearSixIcon />
              工程
            </button>
          </nav>
          <div className="wb-content">
            <section className="wb-main">
              <div className="wb-section-heading">
                <h1>
                  {page === "fixtures"
                    ? "灯具配适"
                    : page === "scenes"
                      ? "场景"
                      : "工程设置"}
                </h1>
                {page === "fixtures" && (
                  <button
                    className="wb-primary"
                    disabled={
                      busy ||
                      !project.profiles.length ||
                      !project.domains.length
                    }
                    onClick={addFixture}
                  >
                    <PlusIcon />
                    添加灯具
                  </button>
                )}
                {page === "scenes" && (
                  <button
                    className="wb-primary"
                    disabled={busy || !project.fixtures.length}
                    onClick={addScene}
                  >
                    <PlusIcon />
                    新建场景
                  </button>
                )}
              </div>
              {page === "fixtures" && (
                <FixtureTable
                  fixtures={project.fixtures}
                  selected={selected}
                  busy={busy}
                  onSelect={(fixture) =>
                    void run(async () => {
                      setSelected(fixture.id);
                      selectFixture(fixture);
                    })
                  }
                />
              )}
              {page === "scenes" && (
                <SceneEditor
                  project={project}
                  selected={selected}
                  busy={busy}
                  onSelect={(item) => void run(async () => selectScene(item))}
                  onEdit={(command) => run(async () => edit(command))}
                />
              )}
              {page === "settings" && (
                <dl className="wb-project-details">
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
            <ProjectInspector
              form={form}
              htmlProjectForm={htmlProjectForm}
              busy={busy}
              pending={pending}
              page={page}
              project={project}
              activeFixture={activeFixture}
              onChange={changeProjectForm}
              onApply={() => {
                if (formRef.current?.kind === "addFixture") {
                  pendingRef.current = true;
                  setPending(true);
                }
                void run(async () => {});
              }}
              onCancel={() => {
                setProjectForm(null);
                setSelected("");
                setError("");
              }}
              onDelete={() => setConfirmDelete(true)}
            />
          </div>
          <footer className="wb-status">
            <span>
              {busy ? "正在处理…" : dirty ? "有未保存的修改" : "已保存"}
            </span>
            <span>
              {snapshot.fileName?.split(/[\\/]/).at(-1) ?? "未命名文件"}
            </span>
          </footer>
        </>
      )}
      {confirmDelete && (
        <DeleteDialog
          name={form?.name ?? ""}
          busy={busy}
          onCancel={() => setConfirmDelete(false)}
          onDelete={() => {
            setConfirmDelete(false);
            const target = formRef.current;
            if (target)
              void run(async () => {
                await edit({
                  op:
                    target.kind === "fixture" ? "removeFixture" : "removeScene",
                  id: target.id,
                });
                setProjectForm(null);
                setSelected("");
              }, false);
          }}
        />
      )}
    </main>
  );
}
