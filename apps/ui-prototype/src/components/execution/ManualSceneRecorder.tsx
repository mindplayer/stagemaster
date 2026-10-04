import { useEffect, useRef, useState } from "react";
import type { ExecutionSource, ExecutionView } from "../../execution-types";
import type {
  ManualCapture,
  ManualRecordingContext,
} from "../../manual-capture-types";
import { uniqueName } from "../../editor-tools";
import { ManualSceneReview } from "./ManualSceneReview";
import { ManualSceneDestination } from "./ManualSceneDestination";
import "./manual-scene-recorder.css";

export function ManualSceneRecorder({
  context,
  runtime,
  source,
  active,
  observed,
  pending,
  selected,
}: {
  context: ManualRecordingContext;
  runtime: ExecutionView;
  source: ExecutionSource;
  active: boolean;
  observed: boolean;
  pending: boolean;
  selected: string[];
}) {
  const [capture, setCapture] = useState<ManualCapture | null>(null);
  const [scope, setScope] = useState<"all" | "selected">("all");
  const [mode, setMode] = useState<"new" | "merge">("new");
  const [sceneId, setSceneId] = useState("");
  const [name, setName] = useState("");
  const [problem, setProblem] = useState("");
  const [notice, setNotice] = useState("");
  const [working, setWorking] = useState(false);
  const requestId = useRef(0);
  const collectButton = useRef<HTMLButtonElement>(null);
  const hadReview = useRef(false);
  useEffect(() => {
    if (!capture && hadReview.current && active) collectButton.current?.focus();
    hadReview.current = !!capture;
  }, [capture, active]);
  const ticket = useRef<ManualCapture | null>(null);
  const current = useRef({ context, runtime, active });
  current.current = { context, runtime, active };
  const port = context.host.manualCapture;
  const held =
    runtime.observation.snapshot?.state.sources.find((s) => s.id === source.id)
      ?.held ?? [];
  const available =
    port &&
    runtime.catalog.projectId === context.project.id &&
    runtime.catalog.capabilities?.includes("manualValues");
  const count = held.filter(
    (t) => scope === "all" || selected.includes(t.fixtureId),
  ).length;
  const disabled =
    !available ||
    !active ||
    !observed ||
    pending ||
    runtime.pending ||
    working ||
    context.busy ||
    !count ||
    (mode === "merge" && !context.project.scenes.some((s) => s.id === sceneId));
  function discard() {
    requestId.current++;
    const old = ticket.current;
    ticket.current = null;
    setCapture(null);
    setWorking(false);
    if (old) void port?.({ kind: "cancel", token: old.token }).catch(() => {});
  }
  useEffect(() => {
    if (!active) discard();
  }, [active]);
  useEffect(() => {
    if (ticket.current && ticket.current.generation !== context.generation) {
      discard();
      setProblem("工程已变化，请重新采集后录入");
    }
  }, [context.generation]);
  useEffect(
    () => () => {
      requestId.current++;
      const old = ticket.current;
      if (old)
        void port?.({ kind: "cancel", token: old.token }).catch(() => {});
    },
    [runtime.hostId, source.id, port],
  );

  async function collect() {
    if (disabled || !port) return;
    setWorking(true);
    setProblem("");
    setNotice("");
    const id = ++requestId.current;
    const hostId = runtime.hostId;
    try {
      const generation = await context.beforeCapture();
      if (
        generation === null ||
        id !== requestId.current ||
        !current.current.active
      )
        return;
      const result = await port({
        kind: "capture",
        generation,
        hostId,
        source: source.id,
        selected: scope === "all" ? null : [...selected],
        sceneId: mode === "merge" ? sceneId : null,
      });
      if (!result) throw Error("后台未返回手动记录");
      if (
        id !== requestId.current ||
        !current.current.active ||
        current.current.context.generation !== result.generation ||
        current.current.runtime.hostId !== hostId
      ) {
        await port({ kind: "cancel", token: result.token });
        return;
      }
      ticket.current = result;
      setCapture(result);
      setName(
        uniqueName(
          "现场手动场景",
          context.project.scenes.map((s) => s.name),
        ),
      );
    } catch (e) {
      if (id === requestId.current)
        setProblem(e instanceof Error ? e.message : String(e));
    } finally {
      if (id === requestId.current) setWorking(false);
    }
  }
  async function record() {
    if (!capture || working || context.busy || !active) return;
    const trimmed = name.trim();
    if (
      !capture.merge &&
      (!trimmed ||
        [...trimmed].length > 256 ||
        /[\u0000-\u001f\u007f]/.test(trimmed))
    ) {
      setProblem("场景名称需要 1–256 个有效字符");
      return;
    }
    if (
      !capture.merge &&
      context.project.scenes.some((s) => s.name === trimmed)
    ) {
      setProblem("已有同名场景，请换一个名称");
      return;
    }
    const id = ++requestId.current;
    setWorking(true);
    setProblem("");
    try {
      if (capture.merge)
        await context.onMerge(
          capture.generation,
          capture.token,
          capture.merge.added + capture.merge.replaced > 0,
        );
      else await context.onRecord(capture.generation, capture.token, trimmed);
      // A successful project update advances generation and clears the review independently.
      ticket.current = null;
      setCapture(null);
      setProblem("");
      setNotice(
        capture.merge
          ? capture.merge.added + capture.merge.replaced > 0
            ? `已合并到“${capture.merge.sceneName}”，后台运行版本保持，可撤销恢复。`
            : "记录值相同，场景未修改。"
          : `已录入“${trimmed}”，手动层保持，可撤销恢复。`,
      );
    } catch (e) {
      if (id === requestId.current)
        setProblem(e instanceof Error ? e.message : String(e));
    } finally {
      setWorking(false);
    }
  }
  return (
    <section className="manual-scene-recorder" aria-label="录入手动场景">
      {!capture ? (
        <div className="manual-scene-setup">
          <ManualSceneDestination
            mode={mode}
            sceneId={sceneId}
            scenes={context.project.scenes}
            disabled={working}
            onMode={setMode}
            onScene={setSceneId}
          />
          <div className="execution-buttons">
            <label>
              录入范围
              <select
                aria-label="手动录入范围"
                disabled={working}
                value={scope}
                onChange={(e) => setScope(e.target.value as typeof scope)}
              >
                <option value="all">全部持有属性</option>
                <option value="selected">所选灯具的持有属性</option>
              </select>
            </label>
            <button
              ref={collectButton}
              disabled={disabled}
              onClick={() => void collect()}
            >
              {working
                ? "正在采集"
                : `${mode === "merge" ? "审阅合并" : "录入新场景"}（${count} 项）`}
            </button>
          </div>
        </div>
      ) : (
        <ManualSceneReview
          capture={capture}
          project={context.project}
          name={name}
          working={working}
          disabled={
            working ||
            context.busy ||
            !active ||
            context.generation !== capture.generation
          }
          onName={(value) => {
            setName(value);
            setProblem("");
          }}
          onConfirm={() => void record()}
          onCancel={() => {
            discard();
            setProblem("");
          }}
        />
      )}
      {pending && !capture && (
        <p>有未应用输入，请先应用或取消，再录入实际手动值。</p>
      )}
      {runtime.catalog.projectId !== context.project.id && (
        <p>后台属于另一工程，不能录入当前工程。</p>
      )}
      {problem && <p role="alert">{problem}</p>}
      {notice && <p role="status">{notice}</p>}
    </section>
  );
}
