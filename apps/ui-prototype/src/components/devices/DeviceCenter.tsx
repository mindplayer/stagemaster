import { useEffect, useRef, useState } from "react";
import { BluetoothIcon, XIcon } from "@phosphor-icons/react";
import type { ApplicationHost } from "../../application-host";
import type { DeviceRequest, DeviceSnapshot } from "../../device-types";
import {
  canStartDeviceOperation,
  deviceDeadline,
  deviceError,
  deviceLabels,
  newerDeviceSnapshot,
} from "../../device-tools";
import { DeviceIdentity } from "./DeviceIdentity";
import "./devices.css";
import { DeviceDiscovery } from "./DeviceDiscovery";
import { DeviceRuntime } from "./DeviceRuntime";
import { useDeviceRuntime } from "./useDeviceRuntime";

// This component stays mounted when the panel closes. The native service owns
// connectivity even if the entire webview disappears.
export function DeviceCenter({
  host,
  dismiss = false,
  onOpen,
}: {
  host: ApplicationHost;
  dismiss?: boolean;
  onOpen?(): void;
}) {
  const [open, setOpen] = useState(false);
  useEffect(() => {
    if (dismiss) setOpen(false);
  }, [dismiss]);
  const [snapshot, setSnapshot] = useState<DeviceSnapshot | null>(null);
  const runtime = useDeviceRuntime(host.deviceRuntime, snapshot);
  const [mode, setMode] = useState<"installation" | "runtime">("installation");
  const [transportError, setTransportError] = useState("");
  const [actionError, setActionError] = useState("");
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState("");
  const [busy, setBusy] = useState(false);
  const pending = useRef(false);
  const mounted = useRef(false);
  const button = useRef<HTMLButtonElement>(null);
  const heading = useRef<HTMLHeadingElement>(null);
  function accept(value: DeviceSnapshot) {
    setSnapshot((current) => newerDeviceSnapshot(current, value));
  }
  useEffect(() => {
    mounted.current = true;
    let stopped = false;
    let timer: ReturnType<typeof setTimeout>;
    async function poll() {
      try {
        const value = await deviceDeadline(host.device({ kind: "status" }));
        if (!stopped) {
          accept(value);
          setTransportError("");
        }
      } catch (error) {
        if (!stopped) setTransportError(deviceError(error));
      }
      if (!stopped) timer = setTimeout(() => void poll(), 700);
    }
    if (host.kind === "desktop") void poll();
    return () => {
      stopped = true;
      mounted.current = false;
      clearTimeout(timer);
    };
  }, [host]);
  useEffect(() => {
    if (open) heading.current?.focus();
  }, [open]);
  async function request(value: DeviceRequest) {
    if (pending.current) return;
    pending.current = true;
    setBusy(true);
    setActionError("");
    try {
      const next = await deviceDeadline(host.device(value));
      if (mounted.current) {
        accept(next);
        setTransportError("");
      }
    } catch (error) {
      if (mounted.current) setActionError(deviceError(error));
    } finally {
      pending.current = false;
      if (mounted.current) setBusy(false);
    }
  }
  async function connect(id: string) {
    if (mode === "installation" || !host.deviceRuntime) {
      await request({ kind: "connect", epoch: snapshot!.epoch, id });
      return;
    }
    if (pending.current || !snapshot) return;
    pending.current = true;
    setBusy(true);
    setActionError("");
    try {
      await host.deviceRuntime({ kind: "connect", epoch: snapshot.epoch, id });
      const next = await host.device({ kind: "status" });
      if (mounted.current) accept(next);
    } catch (error) {
      if (mounted.current) setActionError(deviceError(error));
    } finally {
      pending.current = false;
      if (mounted.current) setBusy(false);
    }
  }
  function close() {
    setOpen(false);
    button.current?.focus();
  }
  const phase = snapshot?.phase ?? "idle";
  const startable =
    snapshot && canStartDeviceOperation(phase) && !busy && !transportError;
  const cancellable =
    snapshot &&
    ["preparing", "scanning", "connecting", "connected"].includes(phase) &&
    !busy &&
    !transportError;
  const current = snapshot?.selected;
  const diagnostics =
    transportError || phase !== "connected" ? null : snapshot?.diagnostics;
  const label = transportError
    ? "状态不可用"
    : phase === "fault" && !current
      ? "搜索未完成"
      : phase === "stopping" && !current
        ? "正在结束搜索"
        : deviceLabels[phase];
  return (
    <>
      <button
        ref={button}
        className="wb-device-toggle"
        aria-label={`设备连接：${label}`}
        aria-expanded={open}
        aria-controls="device-center"
        disabled={host.kind !== "desktop"}
        title={
          host.kind === "browser"
            ? "请使用桌面应用连接蓝牙设备"
            : `设备连接 · ${label}`
        }
        onClick={() => {
          if (open) close();
          else {
            onOpen?.();
            setOpen(true);
          }
        }}
      >
        <BluetoothIcon />
        <span>设备</span>
        <i
          className={!transportError && phase === "connected" ? "online" : ""}
        />
        <span>{label}</span>
      </button>
      {
        <aside
          hidden={!open}
          id="device-center"
          className="wb-device-center"
          aria-labelledby="device-heading"
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              event.stopPropagation();
              event.preventDefault();
              close();
            }
          }}
        >
          <header>
            <div>
              <h2 ref={heading} tabIndex={-1} id="device-heading">
                设备连接
              </h2>
              <p>低功耗蓝牙</p>
            </div>
            <button aria-label="关闭设备面板" onClick={close}>
              <XIcon />
            </button>
          </header>
          <div className="wb-device-body">
            <section
              className={`wb-device-summary ${phase === "connected" && !transportError ? "online" : ""}`}
            >
              <div>
                <strong role="status">{label}</strong>
                <span>{current?.name ?? "尚未选择设备"}</span>
              </div>
              {cancellable && (
                <button
                  onClick={() =>
                    void request({ kind: "cancel", epoch: snapshot.epoch })
                  }
                >
                  {phase === "connected"
                    ? "断开连接"
                    : phase === "scanning"
                      ? "结束搜索"
                      : phase === "preparing"
                        ? "取消搜索"
                        : "取消连接"}
                </button>
              )}
              {phase === "stopping" && <span>请稍候…</span>}
            </section>
            {phase === "preparing" && (
              <p role="status">
                正在等待系统蓝牙；首次使用请完成系统访问确认。
              </p>
            )}
            {(transportError || actionError || snapshot?.problem) && (
              <div className="wb-device-error" role="alert">
                {transportError || actionError || snapshot?.problem?.message}
                {transportError && (
                  <button
                    disabled={busy}
                    onClick={() => void request({ kind: "status" })}
                  >
                    重新读取状态
                  </button>
                )}
              </div>
            )}
            {diagnostics && (
              <section aria-label="设备诊断" className="wb-device-diagnostics">
                <dl>
                  <div>
                    <dt>设备自检</dt>
                    <dd>{diagnostics.selfTest ? "通过" : "未通过"}</dd>
                  </div>
                  <div>
                    <dt>灯光输出</dt>
                    <dd>
                      {diagnostics.outputDisabled
                        ? "设备已禁止输出"
                        : "设备未禁止输出"}
                    </dd>
                  </div>
                  <div>
                    <dt>连接保活</dt>
                    <dd>{snapshot!.heartbeatCount} 次</dd>
                  </div>
                  <div>
                    <dt>最近通信</dt>
                    <dd>
                      {snapshot!.roundTripMs == null
                        ? "尚未测得"
                        : `${snapshot!.roundTripMs} 毫秒`}
                    </dd>
                  </div>
                </dl>
                <DeviceIdentity description={snapshot?.description ?? null} />
              </section>
            )}
            {host.deviceRuntime && (
              <DeviceRuntime
                runtime={runtime}
                outputDisabled={diagnostics?.outputDisabled}
              />
            )}
            <DeviceDiscovery
              snapshot={snapshot}
              query={query}
              setQuery={setQuery}
              selected={selected}
              setSelected={setSelected}
              startable={!!startable}
              request={request}
              connect={connect}
              mode={mode}
              setMode={setMode}
              runtimeSupported={!!host.deviceRuntime}
            />
            <details className="wb-device-details">
              <summary>连接详情</summary>
              <dl>
                <div>
                  <dt>连接标识</dt>
                  <dd>{current?.id ?? "—"}</dd>
                </div>
                <div>
                  <dt>最近有效回复</dt>
                  <dd>
                    {snapshot?.lastReplyAgeMs == null || transportError
                      ? "—"
                      : `${(snapshot.lastReplyAgeMs / 1000).toFixed(1)} 秒前`}
                  </dd>
                </div>
                <div>
                  <dt>设备运行</dt>
                  <dd>
                    {diagnostics
                      ? `${(diagnostics.uptimeMs / 1000).toFixed(0)} 秒`
                      : "—"}
                  </dd>
                </div>
                <div>
                  <dt>内核推进</dt>
                  <dd>{diagnostics?.ticks ?? "—"}</dd>
                </div>
                <div>
                  <dt>内部堆占用</dt>
                  <dd>
                    {diagnostics
                      ? `${(diagnostics.heapUsed / 1024).toFixed(1)} / ${((diagnostics.heapUsed + diagnostics.heapFree) / 1024).toFixed(1)} KiB`
                      : "—"}
                  </dd>
                </div>
              </dl>
              {snapshot?.problem?.detail && (
                <pre>{snapshot.problem.detail}</pre>
              )}
              <p>连接标识随系统和设备启动变化，不作为设备身份认证。</p>
            </details>
          </div>
          <footer>收起面板后保持连接；退出应用后断开。</footer>
        </aside>
      }
    </>
  );
}
