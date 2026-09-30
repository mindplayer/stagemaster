import { WorkbenchViewport } from "./components/layout/WorkbenchViewport";
import { useEffectWorkspace } from "./components/workbench/useEffectWorkspace";
import { EffectInspectorPane } from "./components/workbench/EffectInspectorPane";
import { useAudioSceneLink } from "./components/audio/useAudioSceneLink";
import { AudioSceneReturn } from "./components/audio/AudioSceneReturn";
import {
  AudioWorkspace,
  type AudioHandle,
} from "./components/audio/AudioWorkspace";
import { PerformanceLayout } from "./components/layout/PerformanceLayout";
import { DockPane } from "./components/layout/DockPane";
import {
  WorkbenchNavigation,
  type WorkbenchPage,
} from "./components/layout/WorkbenchNavigation";
import { StageViewTabs } from "./components/layout/StageViewTabs";
import { WorkspaceSurface } from "./components/workbench/WorkspaceSurface";
import { useAudio } from "./components/audio/useAudio";
import { AudioPreviewTransport } from "./components/audio/AudioPreviewTransport";
import { ProjectHeader } from "./components/projects/ProjectHeader";
import { ProjectStart } from "./components/projects/ProjectStart";
import { RecentProjectsDialog } from "./components/projects/RecentProjectsDialog";
import { startInstallationReason } from "./installation-tools";
import { useInstallation } from "./components/installation/useInstallation";
import { RecoveryCenter } from "./components/workbench/RecoveryCenter";
import type { RecoveryEntry } from "./recovery-types";
import type { CheckLocation } from "./check-types";
import { PackagePanel } from "./components/workbench/PackagePanel";
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
import { PlusIcon, XIcon } from "@phosphor-icons/react";
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
import { SceneEditingTools } from "./components/workbench/SceneEditingTools";
import { SceneInspector } from "./components/workbench/SceneInspector";
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
import { type SharedPrevisHandle } from "./components/stage/SharedPrevis";

const EMPTY: Snapshot = {
  generation: 0,
  project: null,
  fileName: null,
  dirty: false,
  canUndo: false,
  canRedo: false,
  recovery: { state: "clean", capturedAtMs: null, problem: null },
};
type Page = WorkbenchPage;
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
  const installation = useInstallation(host);
  const [snapshot, setSnapshot] = useState(EMPTY);
  const current = useRef(EMPTY);
  const [busy, setBusy] = useState(false);
  const queue = useRef(Promise.resolve(true));
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [page, setPage] = useState<Page>("fixtures");
  const sharedPrevis = useRef<SharedPrevisHandle>(null);
  const [stageView, setStageView] = useState<"plan" | "three">("plan");
  const [stageSelected, setStageSelected] = useState("");
  const [monitorVisible, setMonitorVisible] = useState(false);
  const audioSession = useAudio(
    host,
    () => current.current.generation,
    snapshot.project?.audio ?? null,
    page === "audio" || monitorVisible,
    snapshot.project?.id ?? "",
    page === "audio",
  );
  const [showRecovery, setShowRecovery] = useState(false);
  const [showRecent, setShowRecent] = useState(false);
  const [patchId, setPatchId] = useState("");
  const [patchSelection, setPatchSelection] = useState<string[]>([]);
  const [patchDialog, setPatchDialog] = useState<"repatch" | "exchange" | null>(
    null,
  );
  const audio = useRef<AudioHandle>(null);
  const [audioPending, setAudioPending] = useState(false);
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
  const [sequenceExecution, setSequenceExecution] = useState(false);
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
  const effectWorkspace = useEffectWorkspace({
    project,
    sceneId,
    host,
    getProject: () => current.current.project,
    run,
    edit,
    openPlayback: () => sharedPrevis.current?.openPlayback(),
    clearError: () => setError(""),
  });
  const audioSceneLink = useAudioSceneLink(
    project,
    () => current.current.project,
    run,
    (scene) => {
      setSceneId(scene.id);
      setSceneQuery("");
      setForm(sceneForm(scene));
      setPage("scenes");
    },
    () => {
      setPage("audio");
      restoreForm("audio");
    },
  );
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
    profilePending ||
    audioPending ||
    effectWorkspace.pending;
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
    commands.push(...(audio.current?.collect() ?? []));
    commands.push(...(effectWorkspace.editor.current?.collect() ?? []));
    if (!commands.length) {
      sequences.current?.accept();
      stage.current?.accept();
      profiles.current?.accept();
      audio.current?.accept();
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
    audio.current?.accept();
    effectWorkspace.editor.current?.accept();
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
  async function fileAction(
    kind: "new" | "open" | "openRecent",
    id?: string,
  ): Promise<boolean> {
    let opened = false;
    const ok = await run(async () => {
      const previousGeneration = current.current.generation;
      const value: ProjectRequest =
        kind === "openRecent"
          ? { kind, generation: previousGeneration, id: id! }
          : { kind, generation: previousGeneration };
      const next = await request(value);
      if (next.project && next.generation !== previousGeneration) {
        resetWorkspace(next);
        setShowRecent(false);
        setNotice(kind === "new" ? "已创建工程" : "已打开工程");
        opened = true;
      }
    });
    return ok && opened;
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
    effectWorkspace.close();
    audioSceneLink.reset();
    if (next.project) {
      setPage(next.project.fixtures.length ? "scenes" : "fixtures");
      setStageSelected("");
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
  function save(saveAs = false, expectedGeneration?: number) {
    return run(async () => {
      if (
        expectedGeneration !== undefined &&
        current.current.generation !== expectedGeneration
      )
        throw new Error("结果已过期，请重新检查后补齐资源");
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
        throw new Error("结果已过期，请刷新结果后定位");
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
        case "audio":
          setPage("audio");
          setForm(null);
          break;
        case "placement":
          setPage("stage");
          setForm(null);
          stage.current?.revealFixture(location.id);
          break;
      }
      setNotice("已定位对象；修复后回到工程刷新结果");
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
      const dialog = document.querySelector<HTMLDialogElement>(
        "dialog[open]:not([data-navigation-dialog])",
      );
      if (dialog) {
        dialog.querySelector<HTMLElement>("input,button")?.focus();
        setError("请先保存或取消当前编辑，再关闭窗口");
        return;
      }
      setShowRecent(false);
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
      <ProjectHeader
        host={host}
        snapshot={snapshot}
        dirty={dirty}
        busy={busy}
        hasDrafts={hasDrafts}
        installation={installation}
        onNew={() => void fileAction("new")}
        onOpen={() => void fileAction("open")}
        onRecent={() => {
          setError("");
          setShowRecent(true);
        }}
        onRecover={() => {
          setError("");
          setShowRecovery(true);
        }}
        onHistory={history}
        onSave={save}
      />
      {showRecent && (
        <RecentProjectsDialog
          host={host}
          busy={busy}
          error={error}
          onOpen={(id) => fileAction("openRecent", id)}
          onBrowse={() => void fileAction("open")}
          onClose={() => setShowRecent(false)}
        />
      )}
      {snapshot.recentProblem && (
        <div className="wb-recovery-status warning" role="status">
          最近工程记录未更新：{snapshot.recentProblem}
        </div>
      )}
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
        <ProjectStart
          host={host}
          busy={busy}
          onNew={() => void fileAction("new")}
          onOpen={() => void fileAction("open")}
          onRecover={() => setShowRecovery(true)}
          onRecent={(id) => fileAction("openRecent", id)}
        />
      ) : (
        <>
          <PerformanceLayout
            revealInspector={effectWorkspace.active?.token}
            beforeChange={() => run(async () => {})}
            busy={busy}
            mode={
              page === "sequences" && sequenceExecution ? "execution" : page
            }
            toolbar={
              <WorkbenchNavigation
                page={page}
                project={project}
                busy={busy}
                onSelect={switchPage}
              />
            }
          >
            <WorkbenchViewport
              key={`viewport:${project.id}`}
              ref={sharedPrevis}
              project={project}
              page={page}
              stageView={stageView}
              onStageView={(value) => void run(async () => setStageView(value))}
              query={fixtureQuery}
              onlySelected={onlySelected}
              selected={selected}
              onQuery={setFixtureQuery}
              onFilter={setOnlySelected}
              onSelect={(ids) =>
                run(async () => {
                  if (current.current.project?.id !== project.id)
                    throw new Error("工程已变化，请重新选择灯具");
                  const valid = current.current.project.fixtures.map(
                    (f) => f.id,
                  );
                  setSelectedIds(ids.filter((id) => valid.includes(id)));
                })
              }
              preview={{
                onVisibilityChange: setMonitorVisible,
                transport: (
                  <AudioPreviewTransport
                    session={audioSession}
                    track={
                      page === "audio" || audioSession.position.playing
                        ? project.audio
                        : null
                    }
                    busy={busy}
                  />
                ),
                host,
                scenes: project.scenes,
                currentScene: page === "scenes" ? activeScene : undefined,
                allowPlacement: page === "stage" && stageView === "three",
                busy,
                generation: () => current.current.generation,
                run: (work) => run(work),
                selectedId:
                  page === "stage"
                    ? stageSelected
                    : page === "scenes"
                      ? (selected.at(-1) ?? "")
                      : "",
                onSelect: (id) => {
                  if (page === "stage")
                    return (
                      stage.current?.selectFixture(id) ?? Promise.resolve(false)
                    );
                  if (page !== "scenes") return Promise.resolve(false);
                  return run(async () => {
                    if (
                      id &&
                      !current.current.project?.fixtures.some(
                        (f) => f.id === id,
                      )
                    )
                      throw new Error("所选灯具已不存在");
                    setSelectedIds(id ? [id] : []);
                  });
                },
                onPrepareMove: () => run(async () => {}),
                onPlacement: (proposal, isActive) =>
                  run(async () => {
                    if (!isActive())
                      throw new Error("三维编辑上下文已变化，灯位未修改");
                    await request({ kind: "previsPlacement", ...proposal });
                    setNotice("灯位已更新，可撤销恢复");
                  }),
              }}
            />
            <DockPane region="full" visible={page === "profiles"}>
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
            </DockPane>
            <StageWorkspace
              key={`stage:${project.id}`}
              ref={stage}
              canvasVisible={stageView === "plan"}
              viewControls={
                <StageViewTabs
                  value={stageView}
                  busy={busy}
                  onChange={(value) =>
                    void run(async () => setStageView(value))
                  }
                />
              }
              onSelectedFixture={setStageSelected}
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
            <AudioWorkspace
              key={`audio:${project.id}`}
              ref={audio}
              session={audioSession}
              onEditScene={audioSceneLink.edit}
              onView3d={() => sharedPrevis.current?.openPlayback()}
              sharedTransport
              project={project}
              host={host}
              generation={() => current.current.generation}
              visible={page === "audio"}
              busy={busy}
              beforeChange={() => run(async () => {})}
              onPending={(value) => {
                setAudioPending(value);
                if (!value) setError("");
              }}
              onEdit={async (command) => {
                const ok = await run(async () => {
                  await edit(command);
                  setNotice("音频编排已更新，可撤销恢复");
                });
                return ok ? current.current.project : null;
              }}
            />
            <SequenceWorkspace
              key={project.id}
              ref={sequences}
              onView3d={() => sharedPrevis.current?.openPlayback()}
              execution={sequenceExecution}
              onExecution={setSequenceExecution}
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
            <DockPane
              region="full"
              visible={page === "fixtures" || page === "settings"}
              passthrough={page === "scenes"}
            >
              <WorkspaceSurface
                visible={
                  page === "scenes" ||
                  page === "fixtures" ||
                  page === "settings"
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
                  <DockPane region="library" visible={page === "scenes"}>
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
                  </DockPane>
                )}
                <DockPane
                  region="editor"
                  visible={page === "scenes"}
                  passthrough={page !== "scenes"}
                >
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
                        </div>
                      )}
                    </div>
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
                                const f =
                                  current.current.project!.fixtures.find(
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
                      <SceneEditingTools
                        context={
                          audioSceneLink.marker &&
                          audioSceneLink.sceneId === activeScene?.id ? (
                            <AudioSceneReturn
                              marker={audioSceneLink.marker}
                              busy={busy}
                              onBack={() => void audioSceneLink.back()}
                            />
                          ) : null
                        }
                        key={project.id}
                        host={host}
                        project={project}
                        scene={activeScene}
                        selected={selected}
                        fixtureQuery={fixtureQuery}
                        onlySelected={onlySelected}
                        generation={snapshot.generation}
                        visible={page === "scenes"}
                        busy={busy}
                        error={error}
                        beforeChange={() => run(async () => {})}
                        onQuery={setFixtureQuery}
                        onFilter={setOnlySelected}
                        onSelect={(ids) =>
                          run(async () => {
                            setSelectedIds(ids);
                          })
                        }
                        onEdit={(command) =>
                          run(async () => {
                            await edit(command);
                            setNotice(
                              "效果已更新，可撤销恢复；预演当前场景可查看变化",
                            );
                          })
                        }
                        onLibraryEdit={async (command) => {
                          const ok = await run(async () => {
                            await edit({ op: "library", command });
                            setNotice("资源已更新，可撤销恢复");
                          });
                          return ok ? current.current.project : null;
                        }}
                        onView3d={() => sharedPrevis.current?.openPlayback()}
                        onOpenEffect={effectWorkspace.open}
                        onToggleEffect={effectWorkspace.toggle}
                        onAddScene={addScene}
                        onAddFixtures={() => switchPage("fixtures")}
                      />
                      {page === "settings" && (
                        <details className="wb-file-details">
                          <summary>
                            文件信息 ·{" "}
                            {snapshot.fileName?.split(/[\\/]/).at(-1) ??
                              "尚未保存"}
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
                      <PackagePanel
                        key={`package:${project.id}`}
                        host={host}
                        project={project}
                        generation={snapshot.generation}
                        hasDrafts={hasDrafts}
                        visible={page === "settings"}
                        busy={busy}
                        capture={captureCheck}
                        onLocate={locateCheck}
                        onInstall={installation.start}
                        installReason={startInstallationReason(
                          installation.view,
                          installation.communicationError,
                        )}
                      />
                      <ProjectCheckPanel
                        key={`check:${project.id}`}
                        host={host}
                        projectId={project.id}
                        generation={snapshot.generation}
                        hasDrafts={hasDrafts}
                        visible={page === "settings"}
                        busy={busy}
                        capture={captureCheck}
                        onLocate={locateCheck}
                        onSaveResources={(generation) =>
                          save(false, generation)
                        }
                      />
                    </div>
                  </section>
                </DockPane>
                <DockPane
                  region="inspector"
                  visible={page === "scenes"}
                  passthrough={page !== "scenes"}
                >
                  <div className="wb-properties">
                    <EffectInspectorPane
                      selection={
                        page === "scenes" ? effectWorkspace.active : null
                      }
                      editor={effectWorkspace.editor}
                      fixtures={project.fixtures}
                      selected={selected}
                      busy={busy}
                      error={error}
                      onCancel={effectWorkspace.close}
                      onPending={effectWorkspace.setPending}
                      onApply={effectWorkspace.apply}
                      onPreview={effectWorkspace.preview}
                    >
                      <SceneInspector
                        active={page === "scenes" && !!activeScene}
                        busy={busy}
                        beforeChange={() => run(async () => {})}
                        hasPosition={selected.some(
                          (id) =>
                            project.fixtures.find((f) => f.id === id)
                              ?.positioning,
                        )}
                        position={
                          <>
                            {page === "scenes" && activeScene && (
                              <PositionPanel
                                key={`position:${project.id}:${activeScene.id}:${selected.join(",")}`}
                                ref={positions}
                                project={project}
                                fixtures={selected.map((id) =>
                                  project.fixtures.find((f) => f.id === id)!,
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
                          </>
                        }
                        light={
                          <>
                            {page === "scenes" && activeScene && (
                              <ParameterPanel
                                key={`${project.id}:${activeScene.id}:${selected.join(",")}`}
                                ref={parameters}
                                scene={activeScene}
                                fixtures={selected.map((id) =>
                                  project.fixtures.find((f) => f.id === id)!,
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
                          </>
                        }
                        scene={
                          <>
                            <ProjectInspector
                              form={form}
                              htmlProjectForm={htmlProjectForm}
                              busy={busy}
                              pending={pending}
                              page={
                                page === "profiles" ||
                                page === "sequences" ||
                                page === "audio" ||
                                page === "stage"
                                  ? "scenes"
                                  : page
                              }
                              project={project}
                              activeFixture={
                                page === "fixtures" ? activeFixture : undefined
                              }
                              onChange={(patch) => {
                                if (formRef.current)
                                  setForm(
                                    { ...formRef.current, ...patch },
                                    true,
                                  );
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
                          </>
                        }
                      />
                    </EffectInspectorPane>
                  </div>
                </DockPane>
              </WorkspaceSurface>
            </DockPane>
          </PerformanceLayout>
          <footer className="wb-status">
            <span role="status">
              {busy
                ? "正在处理…"
                : error
                  ? "修改未完成"
                  : hasDrafts
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
