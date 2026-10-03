import { MagnifyingGlassIcon, XIcon } from "@phosphor-icons/react";
import type { DeviceRequest, DeviceSnapshot } from "../../device-types";
import { deviceMatches } from "../../device-tools";
export function DeviceDiscovery({
  snapshot,
  query,
  setQuery,
  selected,
  setSelected,
  startable,
  request,
  connect,
  mode,
  setMode,
  runtimeSupported,
}: {
  snapshot: DeviceSnapshot | null;
  query: string;
  setQuery(value: string): void;
  selected: string;
  setSelected(value: string): void;
  startable: boolean;
  request(value: DeviceRequest): Promise<void>;
  connect(id: string): Promise<void>;
  mode: "installation" | "runtime";
  setMode(value: "installation" | "runtime"): void;
  runtimeSupported: boolean;
}) {
  const phase = snapshot?.phase ?? "idle",
    candidates = snapshot?.candidates ?? [];
  const visible = candidates.filter((candidate) =>
    deviceMatches(candidate, query),
  );
  const target = candidates.find((candidate) => candidate.id === selected);
  return (
    <section className="wb-device-discovery" aria-label="搜索设备">
      {runtimeSupported && (
        <label className="wb-device-mode">
          连接用途
          <select
            aria-label="设备连接用途"
            value={mode}
            disabled={!startable}
            onChange={(e) =>
              setMode(e.target.value as "installation" | "runtime")
            }
          >
            <option value="installation">连接诊断／安装</option>
            <option value="runtime">节目运行</option>
          </select>
        </label>
      )}
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
          <button aria-label="清除设备筛选" onClick={() => setQuery("")}>
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
              {candidate.rssi === null ? "信号未知" : `${candidate.rssi} dBm`}
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
          onClick={() => void connect(target!.id)}
        >
          连接设备
        </button>
      </div>
    </section>
  );
}
