import { useEffect, useRef, useState } from "react";
import {
  BluetoothIcon,
  MagnifyingGlassIcon,
  XIcon,
} from "@phosphor-icons/react";
import type { ApplicationHost } from "../../application-host";
import type { DeviceRequest, DeviceSnapshot } from "../../device-types";
import {
  canStartDeviceOperation,
  deviceDeadline,
  deviceError,
  deviceLabels,
  deviceMatches,
  newerDeviceSnapshot,
} from "../../device-tools";
import { DeviceIdentity } from "./DeviceIdentity";
import "./devices.css";

// This component stays mounted when the panel closes. The native service owns
// connectivity even if the entire webview disappears.
export function DeviceCenter({ host }: { host: ApplicationHost }) {
  const [open, setOpen] = useState(false);
  const [snapshot, setSnapshot] = useState<DeviceSnapshot | null>(null);
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
  const candidates = snapshot?.candidates ?? [];
  const visible = candidates.filter((candidate) =>
    deviceMatches(candidate, query),
  );
  const target = candidates.find((candidate) => candidate.id === selected);
  const current = snapshot?.selected;
  const diagnostics = transportError ? null : snapshot?.diagnostics;
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
        onClick={() => (open ? close() : setOpen(true))}
      >
        <BluetoothIcon />
        <span>设备</span>
        <i
          className={!transportError && phase === "connected" ? "online" : ""}
        />
        <span>{label}</span>
      </button>
      {open && (
        <aside
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
                    <dd>{snapshot!.roundTripMs} 毫秒</dd>
                  </div>
                </dl>
                <DeviceIdentity description={snapshot?.description ?? null} />
              </section>
            )}
            <section className="wb-device-discovery" aria-label="搜索设备">
              <div className="wb-device-section-title">
                <h3>
                  附近设备 <span>{candidates.length}</span>
                </h3>
                <button
                  className="wb-primary"
                  disabled={!startable}
                  onClick={() => {
                    setSelected("");
                    void request({ kind: "scan", epoch: snapshot!.epoch });
                  }}
                >
                  <MagnifyingGlassIcon />
                  {candidates.length ? "重新搜索" : "搜索设备"}
                </button>
              </div>
              <div className="wb-device-search">
                <input
                  aria-label="筛选设备"
                  placeholder="搜索名称或连接标识"
                  value={query}
                  onChange={(event) => setQuery(event.target.value)}
                />
                {query && (
                  <button
                    aria-label="清除设备筛选"
                    onClick={() => setQuery("")}
                  >
                    <XIcon />
                  </button>
                )}
              </div>
              <div
                className="wb-device-list"
                role="radiogroup"
                aria-label="选择蓝牙设备"
              >
                {visible.map((candidate) => (
                  <label
                    key={candidate.id}
                    className={selected === candidate.id ? "selected" : ""}
                  >
                    <input
                      type="radio"
                      name="device-selection"
                      value={candidate.id}
                      checked={selected === candidate.id}
                      onChange={() => setSelected(candidate.id)}
                    />
                    <div>
                      <strong>{candidate.name || "未命名设备"}</strong>
                      <span>{candidate.id}</span>
                    </div>
                    <span>
                      {candidate.rssi === null
                        ? "信号未知"
                        : `${candidate.rssi} dBm`}
                    </span>
                  </label>
                ))}
                {!visible.length && (
                  <p className="wb-device-empty">
                    {query && candidates.length
                      ? "没有匹配设备，可清除筛选"
                      : phase === "preparing"
                        ? "蓝牙就绪后开始搜索"
                        : phase === "scanning"
                          ? "正在查找附近的播放设备…"
                          : snapshot?.scanPerformed
                            ? "未发现设备，请检查供电与距离后重新搜索"
                            : "搜索并选择要连接的播放设备"}
                  </p>
                )}
              </div>
              {snapshot?.truncated && (
                <p>设备较多，仅显示前 32 台；请缩小搜索范围后重试。</p>
              )}
              {target && !deviceMatches(target, query) && (
                <p>已选择的设备被筛选隐藏：{target.name}</p>
              )}
              <div className="wb-device-connect">
                <span>{target ? `已选：${target.name}` : "请选择设备"}</span>
                <button
                  className="wb-primary"
                  disabled={!startable || !target}
                  onClick={() =>
                    void request({
                      kind: "connect",
                      epoch: snapshot!.epoch,
                      id: target!.id,
                    })
                  }
                >
                  连接设备
                </button>
              </div>
            </section>
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
                      ? `${(diagnostics.heapUsed / 1024).toFixed(1)} / 128 KiB`
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
      )}
    </>
  );
}
