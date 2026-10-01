import { attributeName } from "../../library-tools";
import { useState } from "react";
import type { PreviewSnapshot } from "../../sequence-types";
import { fixtureAppearance, channelWindow } from "../../sequence-tools";

export function PreviewOutput({
  loaded,
}: {
  loaded: NonNullable<PreviewSnapshot["loaded"]>;
}) {
  const [channelPage, setChannelPage] = useState(0);
  const [showChannels, setShowChannels] = useState(false);
  const [channelQuery, setChannelQuery] = useState("");
  const channels = channelWindow(
    loaded.output.slots,
    channelQuery,
    channelPage,
  );
  return (
    <details className="wb-preview-details">
      <summary>输出明细</summary>
      <div className="wb-output-fixtures">
        {loaded.output.fixtures.map((f) => {
          const appearance = fixtureAppearance(f.attributes);
          return (
            <div className="wb-output-fixture" key={f.id}>
              <span
                hidden={f.attributes.some((a) => a.function)}
                className="wb-lamp"
                style={{
                  background: appearance.color,
                  opacity:
                    appearance.level === null
                      ? 1
                      : Math.max(0.08, appearance.level),
                  boxShadow: `0 0 24px ${appearance.color}`,
                }}
              />
              <div>
                <strong>{f.name}</strong>
                <small>
                  地址 {f.address} ·{" "}
                  {appearance.level === null
                    ? "无独立调光"
                    : `${Math.round(appearance.level * 100)}%`}
                </small>
                {f.attributes
                  .filter((a) => a.function)
                  .map((a) => (
                    <small key={a.key}>
                      {attributeName(a.key)}：{a.function!.name}
                      {a.function!.position !== null
                        ? ` · ${Math.round((a.function!.position * 1000) / 65535) / 10}%`
                        : ""}{" "}
                      · 通道值 {a.function!.dmxValue}
                    </small>
                  ))}
              </div>
            </div>
          );
        })}
      </div>
      <button
        className="wb-channel-toggle"
        aria-expanded={showChannels}
        onClick={() => setShowChannels(!showChannels)}
      >
        线路 {loaded.output.universe} · 512 通道{" "}
        {showChannels ? "收起" : "展开"}
      </button>
      {showChannels && (
        <div className="wb-channels-panel">
          <input
            aria-label="筛选通道"
            placeholder="通道或范围，如 1–32"
            value={channelQuery}
            onChange={(e) => {
              setChannelQuery(e.target.value);
              setChannelPage(0);
            }}
            aria-invalid={!channels.valid}
          />
          {!channels.valid && (
            <p className="wb-preview-warning">请输入 1–512 内的通道或范围</p>
          )}
          <div className="wb-channel-pages">
            <button
              disabled={channels.index === 0}
              onClick={() => setChannelPage(channels.index - 1)}
            >
              上一页
            </button>
            <span>
              {channels.rows[0]?.address ?? 0}–
              {channels.rows.at(-1)?.address ?? 0} · 共 {channels.total} 通道
            </span>
            <button
              disabled={channels.index + 1 >= channels.pages}
              onClick={() => setChannelPage(channels.index + 1)}
            >
              下一页
            </button>
          </div>
          <div className="wb-dmx-grid">
            {channels.rows.map(({ address, value }) => (
              <div
                key={address}
                title={`通道 ${address}：${value}`}
                style={{
                  background: `rgba(103,217,212,${0.04 + (value / 255) * 0.2})`,
                }}
              >
                <small>{address}</small>
                <strong>{value}</strong>
              </div>
            ))}
          </div>
        </div>
      )}
    </details>
  );
}
