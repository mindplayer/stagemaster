export function WaveformToolbar({
  channels,
  follow,
  snap,
  gain,
  zoom,
  maxZoom,
  ready,
  canFit,
  fitTitle,
  onFollow,
  onSnap,
  onGain,
  onZoom,
  onCenter,
  onFit,
}: {
  channels: number;
  follow: boolean;
  snap: boolean;
  gain: string;
  zoom: number;
  maxZoom: number;
  ready: boolean;
  canFit: boolean;
  fitTitle: string;
  onFollow(value: boolean): void;
  onSnap(value: boolean): void;
  onGain(value: string): void;
  onZoom(value: number): void;
  onCenter(): void;
  onFit(): void;
}) {
  return (
    <div className="audio-wave-toolbar">
      <div className="audio-wave-identity">
        <span className="audio-track-number">01</span>
        <strong>音乐</strong>
        <span>{channels === 2 ? "立体声" : "单声道"}</span>
      </div>
      <div className="audio-wave-controls">
        <label>
          <input
            type="checkbox"
            checked={follow}
            onChange={(e) => onFollow(e.target.checked)}
          />
          跟随
        </label>
        <label>
          <input
            type="checkbox"
            checked={snap}
            onChange={(e) => onSnap(e.target.checked)}
          />
          吸附
        </label>
        <label>
          显示幅度
          <select
            aria-label="波形显示幅度"
            value={gain}
            onChange={(e) => {
              onGain(e.target.value);
            }}
          >
            <option value="1">原始</option>
            <option value="2">×2</option>
            <option value="4">×4</option>
          </select>
        </label>
        <button
          aria-label="缩小波形"
          disabled={!ready || zoom <= 1}
          onClick={() => onZoom(zoom / 1.5)}
        >
          −
        </button>
        <input
          type="range"
          disabled={!ready}
          aria-label="波形缩放"
          min="0"
          max={Math.log2(maxZoom) * 15}
          value={Math.log2(zoom) * 15}
          onChange={(e) => onZoom(2 ** (Number(e.target.value) / 15))}
        />
        <button
          aria-label="放大波形"
          disabled={!ready || zoom >= maxZoom}
          onClick={() => onZoom(zoom * 1.5)}
        >
          ＋
        </button>
        <button disabled={!ready} onClick={() => onZoom(1)}>
          全曲
        </button>
        <button disabled={!ready} onClick={() => onCenter()}>
          播放头
        </button>
        <button disabled={!canFit} title={fitTitle} onClick={onFit}>
          适应所选
        </button>
      </div>
    </div>
  );
}
