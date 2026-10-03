import { useEffect, useState } from "react";
import type { DeviceProgramKey } from "../../device-runtime-types";
import type { DeviceRuntimeController } from "./useDeviceRuntime";
import { elapsedLabel, programKey, runLabel } from "../../device-runtime-tools";
import "./device-runtime.css";

export function DeviceRuntime({
  runtime: r,
  outputDisabled,
}: {
  runtime: DeviceRuntimeController;
  outputDisabled?: boolean;
}) {
  const [query, setQuery] = useState(""),
    [selected, setSelected] = useState(""),
    [step, setStep] = useState("");
  const [takeover, setTakeover] = useState(false);
  const context = r.view?.peer?.session;
  useEffect(() => {
    setSelected("");
    setStep("");
    setTakeover(false);
  }, [context]);
  useEffect(() => {
    setStep("");
  }, [programKey(r.state?.loaded ?? null)]);
  const online = !!r.view?.peer,
    state = r.state;
  const readable = online && r.ready && !r.busy && !r.view?.pending;
  const operable = readable && r.owned;
  const selectedProgram = r.programs.find(
    (p) => programKey(p.key) === selected,
  );
  const selectedStep = r.steps.find((s) => s.id === step);
  const currentStep = r.steps.find((s) => s.id === state?.step);
  const visible = r.programs.filter((p) =>
    `${p.name} ${p.key.id}`
      .toLocaleLowerCase()
      .includes(query.trim().toLocaleLowerCase()),
  );
  const name = (key: DeviceProgramKey | null) =>
    key
      ? (r.programs.find((p) => programKey(p.key) === programKey(key))?.name ??
        "目录中尚未读取的节目")
      : "无";
  const disabled = !operable || state?.mode !== "operation";
  return (
    <section className="wb-device-runtime" aria-label="设备节目运行">
      <div className="wb-device-section-title">
        <h3>设备节目</h3>
        <button disabled={!online || r.busy} onClick={() => void r.reload()}>
          刷新目录
        </button>
      </div>
      {r.error && (
        <div className="wb-device-error" role="alert">
          {r.error}
        </div>
      )}
      {r.view?.pending && (
        <p role="alert">
          上次操作结果尚未确认。请重新连接并核对节目状态，不要重复执行。
        </p>
      )}
      {!online ? (
        <p>使用“节目运行”连接后，可读取设备节目与执行状态。</p>
      ) : (
        <>
          <div className="wb-run-status">
            <strong role="status">
              {r.ready ? "" : "上次读取："}
              {runLabel(state)}
            </strong>
            {state?.instance && (
              <span>
                本次运行 {state.instance} · {elapsedLabel(state.elapsedMs)}
              </span>
            )}
          </div>
          <dl>
            <div>
              <dt>已绑定节目包</dt>
              <dd>{state?.package ? "已校验绑定" : "无"}</dd>
            </div>
            <div>
              <dt>设备所选</dt>
              <dd>{name(state?.selected ?? null)}</dd>
            </div>
            <div>
              <dt>已载入</dt>
              <dd>{name(state?.loaded ?? null)}</dd>
            </div>
            <div>
              <dt>当前步骤</dt>
              <dd>
                {currentStep
                  ? `${currentStep.number} · ${currentStep.name}`
                  : state?.step
                    ? "步骤尚未读取"
                    : "无"}
              </dd>
            </div>
            <div>
              <dt>物理发送</dt>
              <dd>{outputDisabled ? "设备已禁止输出" : "尚未取得发送回执"}</dd>
            </div>
            <div>
              <dt>控制权</dt>
              <dd>
                {r.owned
                  ? "当前连接持有"
                  : state?.owner
                    ? "其他控制者持有"
                    : "未取得"}
              </dd>
            </div>
          </dl>
          {r.view?.peer?.control && (
            <div className="wb-run-actions">
              {r.owned ? (
                <button
                  disabled={!readable}
                  onClick={() => void r.request({ kind: "release" })}
                >
                  归还控制权
                </button>
              ) : state?.owner ? (
                <button disabled={!readable} onClick={() => setTakeover(true)}>
                  接管控制权…
                </button>
              ) : (
                <button
                  disabled={!state || !readable}
                  onClick={() =>
                    void r.request({ kind: "acquire", takeover: false })
                  }
                >
                  取得控制权
                </button>
              )}
            </div>
          )}
          {!r.view?.peer?.control && <p>当前连接仅允许查看。</p>}
          {takeover && !r.owned && (
            <div
              className="wb-run-confirm"
              role="group"
              aria-label="确认接管设备"
            >
              <p>接管将替换当前控制者，正在运行的节目继续。</p>
              <button
                disabled={!readable}
                onClick={() => {
                  setTakeover(false);
                  void r.request({ kind: "acquire", takeover: true });
                }}
              >
                确认接管
              </button>
              <button onClick={() => setTakeover(false)}>取消</button>
            </div>
          )}
          <div className="wb-device-search">
            <input
              aria-label="筛选设备节目"
              placeholder="搜索已安装节目"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
            {query && (
              <button aria-label="清除节目筛选" onClick={() => setQuery("")}>
                清除
              </button>
            )}
          </div>
          <select
            aria-label="选择设备节目"
            size={Math.max(2, Math.min(5, visible.length))}
            value={
              visible.some((p) => programKey(p.key) === selected)
                ? selected
                : ""
            }
            onChange={(e) => setSelected(e.target.value)}
          >
            <option value="" disabled>
              请选择节目
            </option>
            {visible.map((p) => (
              <option key={programKey(p.key)} value={programKey(p.key)}>
                {p.key.kind === "scene" ? "场景" : "场景列表"} · {p.name}
              </option>
            ))}
          </select>
          {!visible.length && !r.listing && (
            <p>
              {r.programs.length
                ? "没有匹配节目"
                : r.programCount === 0
                  ? "设备目录为空；安装节目后结束维护并刷新目录"
                  : "节目目录尚未读取完整，请刷新目录"}
            </p>
          )}
          {selectedProgram && !visible.includes(selectedProgram) && (
            <p>已选节目被筛选隐藏：{selectedProgram.name}</p>
          )}
          <div className="wb-run-actions">
            <button
              disabled={disabled || !selectedProgram}
              onClick={() => selectedProgram && void r.load(selectedProgram)}
            >
              载入所选节目
            </button>
          </div>
          <label className="wb-run-step">
            起始步骤
            <select
              aria-label="选择设备起始步骤"
              value={step}
              onChange={(e) => setStep(e.target.value)}
            >
              <option value="">请选择步骤</option>
              {r.steps.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.number} · {s.name}
                </option>
              ))}
            </select>
          </label>
          <div className="wb-run-actions">
            <button
              className="wb-primary"
              disabled={disabled || !state?.loaded || !selectedStep}
              onClick={() =>
                selectedStep &&
                void r.request({ kind: "start", step: selectedStep.id })
              }
            >
              执行所选步骤
            </button>
            <button
              disabled={disabled || state?.status !== "running"}
              onClick={() => void r.request({ kind: "pause" })}
            >
              暂停
            </button>
            <button
              disabled={disabled || state?.status !== "paused"}
              onClick={() => void r.request({ kind: "resume" })}
            >
              继续
            </button>
            <button
              disabled={
                disabled ||
                !state?.instance ||
                !["running", "paused"].includes(state?.status ?? "")
              }
              onClick={() => void r.request({ kind: "next" })}
            >
              下一步
            </button>
            <button
              disabled={disabled || !state?.instance}
              onClick={() => void r.request({ kind: "stop" })}
            >
              停止
            </button>
          </div>
          {r.view?.peer?.installation && (
            <details className="wb-run-maintenance">
              <summary>节目安装与维护</summary>
              <p>
                进入维护会停止本次节目。设备确认静默后，断开并使用安装连接；安装完成再回到节目运行，结束维护。
              </p>
              <div className="wb-run-actions">
                {state?.mode === "operation" && (
                  <button
                    disabled={!operable}
                    onClick={() => void r.request({ kind: "beginMaintenance" })}
                  >
                    停止节目并进入维护
                  </button>
                )}
                {state?.mode === "quiescing" && (
                  <button
                    disabled={!operable}
                    onClick={() =>
                      void r.request({ kind: "cancelMaintenance" })
                    }
                  >
                    取消进入维护
                  </button>
                )}
                {state?.mode === "maintenance" && (
                  <button
                    disabled={!operable}
                    onClick={() =>
                      void r.request({ kind: "finishMaintenance" })
                    }
                  >
                    结束维护并读取新节目
                  </button>
                )}
              </div>
            </details>
          )}
        </>
      )}
      {r.listing && (
        <div className="wb-run-actions" role="status">
          <span>{r.listing}</span>
          <button onClick={r.cancel}>取消读取</button>
        </div>
      )}
    </section>
  );
}
