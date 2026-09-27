import { WorkspaceSurface } from "./components/workbench/WorkspaceSurface";
import { RecoveryCenter } from "./components/workbench/RecoveryCenter";
import type { RecoveryEntry } from "./recovery-types";
import type { CheckLocation } from "./check-types";
import { ProjectCheckPanel } from "./components/workbench/ProjectCheckPanel";
import {
  PositionPanel,
  type PositionHandle,
} from "./components/fixtures/PositionPanel";
import {
  ProfileWorkspace,
  type ProfileHandle,
} from "./components/fixtures/ProfileWorkspace";
import { PatchDialog } from "./components/fixtures/PatchDialog";
import { PatchMap } from "./components/fixtures/PatchMap";
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
  CubeIcon,
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
import {
  StageWorkspace,
  type StageHandle,
} from "./components/stage/StageWorkspace";
import { ResourcePool } from "./components/workbench/ResourcePool";
import { PrevisPanel } from "./components/stage/PrevisPanel";
import { EffectRack } from "./components/workbench/EffectRack";
import { PreviewPanel } from "./components/workbench/PreviewPanel";

const EMPTY: Snapshot = {
  generation: 0,
  project: null,
  fileName: null,
  dirty: false,
  canUndo: false,
  canRedo: false,
  recovery: { state: "clean", capturedAtMs: null, problem: null },
};
type Page =
  | "profiles"
  | "stage"
  | "fixtures"
  | "scenes"
  | "sequences"
  | "settings";
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
  const [showMonitor, setShowMonitor] = useState(false);
  const [showRecovery, setShowRecovery] = useState(false);
  const [followScene, setFollowScene] = useState(true);
  const [patchId, setPatchId] = useState("");
  const [patchSelection, setPatchSelection] = useState<string[]>([]);
  const [patchDialog, setPatchDialog] = useState<"repatch" | "exchange" | null>(
    null,
  );
  const profiles = useRef<ProfileHandle>(null);
  const [profilePending, setProfilePending] = useState(false);
  const [sceneId, setSceneId] = useState("");
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [patchQuery, setPatchQuery] = useState("");
  const [patchOnlySelected, setPatchOnlySelected] = useState(false);
  const [fixtureQuery, setFixtureQuery] = useState("");
  const [sceneQuery, setSceneQuery] = useState("");
  const [onlySelected, setOnlySelected] = useState(false);
  const [form, setFormState] = useState<ProjectForm | null>(null);
  const formRef = useRef<ProjectForm | null>(null);
  const [pending, setPending] = useState(false);
  const pendingRef = useRef(false);
  const [parameterPending, setParameterPending] = useState(false);
  const parameters = useRef<ParameterHandle>(null);
  const positions = useRef<PositionHandle>(null);
  const [positionPending, setPositionPending] = useState(false);
  const sequences = useRef<SequenceHandle>(null);
  const stage = useRef<StageHandle>(null);
  const [stagePending, setStagePending] = useState(false);
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
  const hasDrafts =
    pending ||
    positionPending ||
    parameterPending ||
    sequencePending ||
    stagePending ||
    profilePending;
  const dirty = snapshot.dirty || hasDrafts;

  function setForm(next: ProjectForm | null, changed = false) {
    formRef.current = next;
    setFormState(next);
    pendingRef.current = changed;
    setPending(changed);
  }
  const request = useCallback(
    async (value: ProjectRequest) => {
      try {
        const next = await host.request(value);
        current.current = next;
        setSnapshot(next);
        return next;
      } catch (reason) {
        // A native save prompt may have committed a revision before a subsequent
        // open/retirement failed. Keep the generation current without discarding drafts.
        if (value.kind !== "snapshot") {
          try {
            const next = await host.request({ kind: "snapshot" });
            current.current = next;
            setSnapshot(next);
          } catch {
            /* Preserve the original actionable error. */
          }
        }
        throw reason;
      }
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
    commands.push(...(positions.current?.collect() ?? []));
    commands.push(...(sequences.current?.collect() ?? []));
    commands.push(...(stage.current?.collect() ?? []));
    commands.push(...(profiles.current?.collect() ?? []));
    if (!commands.length) {
      sequences.current?.accept();
      stage.current?.accept();
      profiles.current?.accept();
      return;
    }
    if (commands.length > 256)
      throw new Error("一次最多修改 256 项，请先应用部分修改");
    await edit({ op: "batch", commands });
    parameters.current?.accept();
    positions.current?.accept();
    sequences.current?.accept();
    stage.current?.accept();
    profiles.current?.accept();
    setParameterPending(false);
    setNotice("修改已应用");
    if (draft && pendingRef.current) {
      if (draft.kind === "addFixture") {
        const added = current.current.project!.fixtures.at(-1)!;
        setPatchId(added.id);
        setPatchSelection([added.id]);
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
      nextPage === "profiles" ||
        nextPage === "sequences" ||
        nextPage === "stage"
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
        resetWorkspace(next);
        setNotice(kind === "new" ? "已创建工程" : "已打开工程");
      }
    });
  }
  async function recover(entry: RecoveryEntry): Promise<boolean> {
    let recovered = false;
    const ok = await run(async () => {
      const previousGeneration = current.current.generation;
      const next = await request({
        kind: "recover",
        generation: previousGeneration,
        id: entry.id,
        token: entry.token,
      });
      if (next.project && next.generation !== previousGeneration) {
        resetWorkspace(next);
        setShowRecovery(false);
        setNotice("已恢复为未保存副本，请选择位置保存");
        recovered = true;
      }
    });
    return ok && recovered;
  }
  function resetWorkspace(next: Snapshot) {
    if (next.project) {
      setPage(next.project.fixtures.length ? "scenes" : "fixtures");
      setShowMonitor(false);
      setFollowScene(true);
      setPatchId("");
      setPatchSelection([]);
      setProfilePending(false);
      setPositionPending(false);
      setSceneId(next.project.scenes[0]?.id ?? "");
      setSelectedIds([]);
      setPatchQuery("");
      setPatchOnlySelected(false);
      setFixtureQuery("");
      setSceneQuery("");
      setOnlySelected(false);
      setParameterPending(false);
      setForm(
        next.project.scenes[0] && next.project.fixtures.length
          ? sceneForm(next.project.scenes[0])
          : null,
      );
      setStagePending(false);
      setSequencePending(false);
    }
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
  async function captureCheck(): Promise<number | null> {
    let generation: number | null = null;
    const ok = await run(async () => {
      generation = current.current.generation;
    });
    return ok ? generation : null;
  }
  function locateCheck(location: CheckLocation, generation: number) {
    return run(async () => {
      if (current.current.generation !== generation)
        throw new Error("检查报告已过期，请重新检查后定位");
      const p = current.current.project!;
      switch (location.kind) {
        case "fixtures":
        case "fixture": {
          const fixture =
            location.kind === "fixture"
              ? p.fixtures.find((f) => f.id === location.id)
              : undefined;
          setPage("fixtures");
          setPatchQuery("");
          setPatchOnlySelected(false);
          setPatchId(fixture?.id ?? "");
          setPatchSelection(fixture ? [fixture.id] : []);
          setForm(fixture ? fixtureForm(fixture) : null);
          break;
        }
        case "scenes":
        case "scene": {
          const scene =
            location.kind === "scene"
              ? p.scenes.find((s) => s.id === location.id)
              : p.scenes[0];
          setPage("scenes");
          setSceneQuery("");
          setSceneId(scene?.id ?? "");
          setForm(scene ? sceneForm(scene) : null);
          break;
        }
        case "sequence":
          setPage("sequences");
          setForm(null);
          sequences.current?.reveal(location.id);
          break;
        case "placement":
          setPage("stage");
          setForm(null);
          stage.current?.revealFixture(location.id);
          break;
      }
      setNotice("已定位检查对象；修复后回到工程重新检查");
    });
  }
  function openSceneMonitor(playback = false) {
    void run(async () => {
      const scene = current.current.project?.scenes.find(
        (s) => s.id === sceneId,
      );
      if (!scene) return;
      await host.previs({
        kind: "source",
        generation: current.current.generation,
        source: playback
          ? { kind: "playback" }
          : { kind: "scene", sceneId: scene.id },
      });
      const status = await host.previs({ kind: "status" });
      if (!status.enabled) await host.previs({ kind: "enable" });
      setFollowScene(!playback);
      setShowMonitor(true);
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
          <button
            disabled={busy || host.kind !== "desktop"}
            onClick={() => {
              setError("");
              setShowRecovery(true);
            }}
          >
            恢复
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
                !positionPending &&
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
              positionPending ||
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
      {project && (dirty || snapshot.recovery.problem) && (
        <div
          className={`wb-recovery-status${snapshot.recovery.problem ? " warning" : ""}`}
          role="status"
        >
          <span>
            {snapshot.recovery.problem
              ? `恢复保护需要处理：${snapshot.recovery.problem}`
              : hasDrafts
                ? "输入框中有未应用修改；应用后将更新恢复点"
                : snapshot.recovery.state === "protected"
                  ? `恢复点已更新 · ${new Date(snapshot.recovery.capturedAtMs!).toLocaleTimeString("zh-CN", { hour12: false })} · 工程尚未保存`
                  : "尚未建立恢复点，请保存工程"}
          </span>
          {snapshot.recovery.problem && (
            <button
              disabled={busy}
              onClick={() =>
                void run(async () => {
                  await request({ kind: "snapshot" });
                }, false)
              }
            >
              重试保护
            </button>
          )}
          {snapshot.recovery.problem && (
            <button disabled={busy} onClick={() => setShowRecovery(true)}>
              管理副本
            </button>
          )}
        </div>
      )}
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
            <button
              disabled={host.kind !== "desktop" || busy}
              onClick={() => setShowRecovery(true)}
            >
              恢复工程
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
              className={
                page === "fixtures" || page === "profiles" ? "active" : ""
              }
              aria-pressed={page === "fixtures" || page === "profiles"}
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
            <button
              className={page === "stage" ? "active" : ""}
              aria-pressed={page === "stage"}
              disabled={busy}
              onClick={() => switchPage("stage")}
            >
              <CubeIcon />
              舞台<span>{project.stage.spaces.length}</span>
            </button>
          </nav>
          <ProfileWorkspace
            key={`profiles:${project.id}`}
            ref={profiles}
            project={project}
            visible={page === "profiles"}
            busy={busy}
            error={error}
            onPending={(value) => {
              setProfilePending(value);
              if (!value) setError("");
            }}
            beforeChange={() => run(async () => {})}
            onBack={() => switchPage("fixtures")}
            onEdit={async (command) => {
              const ok = await run(async () => {
                await edit(command);
                setNotice("灯具模式已更新，可撤销恢复");
              });
              return ok ? current.current.project : null;
            }}
          />
          <StageWorkspace
            key={`stage:${project.id}`}
            ref={stage}
            previs={(selection) => (
              <PrevisPanel
                host={host}
                scenes={project.scenes}
                busy={busy}
                {...selection}
                onPrepareMove={() => run(async () => {})}
                onPlacement={(proposal, isActive) =>
                  run(async () => {
                    if (!isActive())
                      throw new Error("三维视窗已关闭，灯位未修改");
                    await request({ kind: "previsPlacement", ...proposal });
                    setNotice("灯位已更新，可撤销恢复");
                  })
                }
                generation={() => current.current.generation}
                run={(work) => run(work)}
              />
            )}
            project={project}
            visible={page === "stage"}
            busy={busy}
            error={error}
            beforeChange={() => run(async () => {})}
            onPending={(value) => {
              setStagePending(value);
              if (!value) setError("");
            }}
            onEdit={async (command) => {
              const ok = await run(async () => {
                await edit(command);
                setNotice("场地已更新，可撤销恢复");
              });
              return ok ? current.current.project : null;
            }}
          />
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
          <WorkspaceSurface
            visible={
              page === "scenes" || page === "fixtures" || page === "settings"
            }
            label={
              page === "scenes"
                ? "编排工作区"
                : page === "fixtures"
                  ? "灯具工作区"
                  : "工程工作区"
            }
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
            <section
              className={`wb-content ${page === "scenes" && showMonitor ? "wb-content-monitored" : ""}`}
            >
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
                        : "工程概览"}
                  </h1>
                </div>
                {page === "fixtures" && (
                  <button
                    className="wb-primary"
                    disabled={busy || !project.profiles.length}
                    onClick={addFixture}
                  >
                    <PlusIcon />
                    添加灯具
                  </button>
                )}
                {page === "scenes" && activeScene && (
                  <div className="wb-scene-actions">
                    <span className="wb-dim">
                      {activeScene.values.length} 项记录
                    </span>
                    <button
                      disabled={busy}
                      aria-expanded={showMonitor}
                      onClick={() => {
                        if (showMonitor)
                          void run(async () => setShowMonitor(false));
                        else openSceneMonitor();
                      }}
                    >
                      <CubeIcon />
                      {showMonitor ? "收起三维" : "显示三维"}
                    </button>
                  </div>
                )}
              </div>
              {page === "scenes" && activeScene && showMonitor && (
                <div className="wb-scene-monitor">
                  <PrevisPanel
                    host={host}
                    scenes={project.scenes}
                    busy={busy}
                    currentScene={activeScene}
                    followCurrent={followScene}
                    onFollowCurrent={setFollowScene}
                    generation={() => current.current.generation}
                    run={(work) => run(work)}
                    selectedId={selected.at(-1) ?? ""}
                    onSelect={(id) =>
                      run(async () => {
                        if (
                          id &&
                          !current.current.project?.fixtures.some(
                            (f) => f.id === id,
                          )
                        )
                          throw new Error("所选灯具已不存在");
                        setSelectedIds(id ? [id] : []);
                      })
                    }
                    allowPlacement={false}
                    onPrepareMove={async () => false}
                    onPlacement={async () => false}
                  />
                </div>
              )}
              <div className="wb-editing-content">
                {page === "fixtures" && (
                  <>
                    <div className="patch-actions">
                      <button
                        disabled={busy}
                        onClick={() => switchPage("profiles")}
                      >
                        灯具模式库
                      </button>
                      <button
                        disabled={busy || !project.fixtures.length}
                        onClick={() =>
                          void run(async () => {
                            setPatchDialog("repatch");
                          })
                        }
                      >
                        批量配适
                      </button>
                      <button
                        disabled={busy || !project.fixtures.length}
                        onClick={() =>
                          void run(async () => {
                            setPatchDialog("exchange");
                          })
                        }
                      >
                        替换模式
                      </button>
                      <span>按住 ⌘ 或 Shift 多选灯具</span>
                    </div>
                    <PatchMap
                      project={project}
                      selected={patchSelection}
                      busy={busy}
                      onSelect={(id) =>
                        void run(async () => {
                          const f = current.current.project!.fixtures.find(
                            (f) => f.id === id,
                          );
                          if (f) {
                            setPatchSelection([id]);
                            setPatchQuery("");
                            setPatchOnlySelected(false);
                            setPatchId(id);
                            setForm(fixtureForm(f));
                          }
                        })
                      }
                    />
                    <FixtureBrowser
                      fixtures={project.fixtures}
                      selected={patchSelection.filter((id) =>
                        project.fixtures.some((f) => f.id === id),
                      )}
                      query={patchQuery}
                      onlySelected={patchOnlySelected}
                      busy={busy}
                      onQuery={setPatchQuery}
                      onFilter={setPatchOnlySelected}
                      onSelect={(ids) => {
                        void run(async () => {
                          const fixture =
                            current.current.project!.fixtures.find(
                              (f) => f.id === ids[0],
                            );
                          setPatchSelection(ids);
                          if (!fixture) {
                            setPatchId("");
                            setForm(null);
                          }
                          if (fixture) {
                            setPatchId(fixture.id);
                            setForm(fixtureForm(fixture));
                          }
                        });
                      }}
                      table
                    />
                  </>
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
                {page === "scenes" && activeScene && (
                  <>
                    <EffectRack
                      key={activeScene.id}
                      scene={activeScene}
                      fixtures={project.fixtures}
                      scenes={project.scenes}
                      selected={selected}
                      busy={busy}
                      error={error}
                      beforeChange={() => run(async () => {})}
                      onEdit={(commands) =>
                        run(async () => {
                          await edit({ op: "batch", commands });
                          setNotice(
                            "效果已更新，可撤销恢复；重新载入预览可查看变化",
                          );
                        })
                      }
                    />
                    <PreviewPanel
                      host={host}
                      scene={activeScene}
                      stepId={activeScene.id}
                      generation={snapshot.generation}
                      busy={busy}
                      beforeAction={() => run(async () => {})}
                      visible={page === "scenes"}
                      onView3d={() => openSceneMonitor(true)}
                    />
                  </>
                )}
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
                  <details className="wb-file-details">
                    <summary>
                      文件信息 ·{" "}
                      {snapshot.fileName?.split(/[\\/]/).at(-1) ?? "尚未保存"}
                    </summary>
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
                  </details>
                )}
                <ProjectCheckPanel
                  key={`check:${project.id}`}
                  host={host}
                  projectId={project.id}
                  generation={snapshot.generation}
                  hasDrafts={
                    pending ||
                    parameterPending ||
                    positionPending ||
                    sequencePending ||
                    stagePending ||
                    profilePending
                  }
                  visible={page === "settings"}
                  busy={busy}
                  capture={captureCheck}
                  onLocate={locateCheck}
                />
              </div>
            </section>
            <div className="wb-properties">
              {page === "scenes" && activeScene && (
                <PositionPanel
                  key={`position:${project.id}:${activeScene.id}:${selected.join(",")}`}
                  ref={positions}
                  project={project}
                  fixtures={selected.map(
                    (id) => project.fixtures.find((f) => f.id === id)!,
                  )}
                  scene={activeScene}
                  busy={busy}
                  onPending={(value) => {
                    setPositionPending(value);
                    if (!value) setError("");
                  }}
                  onApply={() => {
                    void run(async () => {});
                  }}
                  beforeChange={() => run(async () => {})}
                />
              )}
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
                page={
                  page === "profiles" ||
                  page === "sequences" ||
                  page === "stage"
                    ? "scenes"
                    : page
                }
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
          </WorkspaceSurface>
          <footer className="wb-status">
            <span role="status">
              {busy
                ? "正在处理…"
                : error
                  ? "修改未完成"
                  : pending ||
                      positionPending ||
                      parameterPending ||
                      sequencePending ||
                      profilePending
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
      {patchDialog && project && (
        <PatchDialog
          project={project}
          initialIds={patchSelection.filter((id) =>
            project.fixtures.some((f) => f.id === id),
          )}
          exchange={patchDialog === "exchange"}
          busy={busy}
          error={error}
          onCancel={() => {
            setPatchDialog(null);
            setError("");
          }}
          onEdit={(command) =>
            run(async () => {
              await edit(command);
              restoreForm();
              setNotice("灯具配适已更新，可撤销恢复");
            })
          }
        />
      )}
      {showRecovery && (
        <RecoveryCenter
          host={host}
          operationError={error}
          onClose={() => setShowRecovery(false)}
          onRestore={recover}
        />
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
