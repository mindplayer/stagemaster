import type { SceneView } from "../../application-host";
import { useMemo, useState, type RefObject } from "react";
import type {
  AudioMarker,
  AudioPosition,
  AudioTimeline,
  AudioWaveform as Wave,
} from "../../audio-types";
import { audioTime } from "../../audio-tools";
import { clipEnvelope } from "./waveform-data";
import { useWaveSurfer } from "./useWaveSurfer";
import { WaveformMarkers } from "./WaveformMarkers";
import "./waveform.css";
export function AudioWaveform({
  track,
  scenes,
  waveform,
  sample,
  selected,
  disabled,
  compact = false,
  requestedPosition = null,
  onSeek,
  onSelect,
  onMove,
}: {
  track: AudioTimeline;
  scenes?: SceneView[];
  waveform: Wave | null;
  sample: RefObject<{ position: AudioPosition; at: number }>;
  selected: string;
  disabled: boolean;
  compact?: boolean;
  requestedPosition?: number | null;
  onSeek(time: number): void;
  onSelect(id: string): void;
  onMove(marker: AudioMarker): void;
}) {
  const duration = track.outMs - track.inMs;
  const prepared = useMemo(() => {
    try {
      return {
        peaks: waveform
          ? clipEnvelope(waveform, track.inMs, track.outMs)
          : null,
        problem: "",
      };
    } catch (error) {
      return {
        peaks: null,
        problem: error instanceof Error ? error.message : "波形数据无效",
      };
    }
  }, [waveform, track.inMs, track.outMs]);
  const [snap, setSnap] = useState(true);
  const [follow, setFollow] = useState(true);
  const [gain, setGain] = useState("1");
  const channelHeight = compact ? 44 : 100;
  const [showOverview, setShowOverview] = useState(!compact);
  const wave = useWaveSurfer(
    prepared.peaks,
    duration,
    sample,
    Number(gain),
    channelHeight,
  );
  const channels = prepared.peaks?.length ?? 1;
  const blocked = disabled || !prepared.peaks;
  function overviewPoint(clientX: number, el: HTMLElement) {
    const rect = el.getBoundingClientRect();
    wave.center(
      Math.max(
        0,
        Math.min(duration, ((clientX - rect.left) / rect.width) * duration),
      ),
    );
  }
  return (
    <section
      className="audio-timeline audio-wave-editor"
      aria-label="音乐时间线"
    >
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
              onChange={(e) => {
                setFollow(e.target.checked);
                wave.follow.current = e.target.checked;
              }}
            />
            跟随
          </label>
          <label>
            <input
              type="checkbox"
              checked={snap}
              onChange={(e) => setSnap(e.target.checked)}
            />
            吸附
          </label>
          <label>
            显示幅度
            <select
              aria-label="波形显示幅度"
              value={gain}
              onChange={(e) => {
                setGain(e.target.value);
              }}
            >
              <option value="1">原始</option>
              <option value="2">×2</option>
              <option value="4">×4</option>
            </select>
          </label>
          <button
            aria-label="缩小波形"
            disabled={!prepared.peaks || wave.zoom <= 1}
            onClick={() => wave.zoomTo(wave.zoom / 1.5)}
          >
            −
          </button>
          <input
            type="range"
            aria-label="波形缩放"
            min="0"
            max={Math.log2(wave.maxZoom) * 15}
            value={Math.log2(wave.zoom) * 15}
            onChange={(e) => wave.zoomTo(2 ** (Number(e.target.value) / 15))}
          />
          <button
            aria-label="放大波形"
            disabled={!prepared.peaks || wave.zoom >= wave.maxZoom}
            onClick={() => wave.zoomTo(wave.zoom * 1.5)}
          >
            ＋
          </button>
          <button onClick={() => wave.zoomTo(1)}>全曲</button>
          <button onClick={() => wave.center()}>播放头</button>
        </div>
      </div>
      {(prepared.problem || wave.problem) && (
        <div role="alert" className="audio-error">
          {prepared.problem || wave.problem}
        </div>
      )}
      <div
        className="audio-wave-body"
        style={{ minHeight: 28 + channels * channelHeight + (scenes ? 98 : 0) }}
      >
        <div ref={wave.ruler} className="audio-wave-ruler" aria-hidden="true" />
        <div
          ref={wave.detail}
          className="audio-wave-detail"
          aria-hidden="true"
        />
        {prepared.peaks && (
          <div className="audio-channel-labels" aria-hidden="true">
            <span style={{ height: channelHeight }}>
              {channels === 2 ? "左" : "单"}
            </span>
            {channels === 2 && (
              <span style={{ height: channelHeight }}>右</span>
            )}
          </div>
        )}
        <WaveformMarkers
          track={track}
          scenes={scenes}
          laneCursor={wave.laneCursor}
          viewport={wave.viewport}
          selected={selected}
          disabled={blocked}
          snap={snap}
          sample={sample}
          preview={wave.preview}
          onMove={onMove}
          onSeek={onSeek}
          onSelect={onSelect}
          onZoom={(factor, x) => wave.zoomTo(wave.zoom * factor, x)}
          onPan={wave.pan}
        />
        {!prepared.peaks && !prepared.problem && (
          <div className="audio-wave-loading">等待音乐波形</div>
        )}
      </div>
      <div className="audio-overview-header">
        <button
          aria-expanded={showOverview}
          onClick={() => setShowOverview((v) => !v)}
        >
          {showOverview ? "收起全曲导航" : "全曲导航"}
        </button>
        <output>
          {audioTime(Math.round(wave.viewport.start))} —{" "}
          {audioTime(Math.min(duration, Math.round(wave.viewport.end)))}
        </output>
      </div>
      <div
        className={`audio-overview${showOverview ? "" : " collapsed"}`}
        inert={!showOverview}
        aria-hidden={!showOverview}
        role="slider"
        tabIndex={0}
        aria-label="全曲导航位置"
        aria-valuemin={0}
        aria-valuemax={duration}
        aria-valuenow={Math.round(
          (wave.viewport.start + wave.viewport.end) / 2,
        )}
        onKeyDown={(e) => {
          if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
            e.preventDefault();
            wave.pan(
              ((e.key === "ArrowLeft" ? -1 : 1) * wave.viewport.width) / 2,
            );
          }
        }}
        onPointerDown={(e) => {
          if (e.button === 0) {
            e.currentTarget.setPointerCapture(e.pointerId);
            overviewPoint(e.clientX, e.currentTarget);
          }
        }}
        onPointerMove={(e) => {
          if (e.currentTarget.hasPointerCapture(e.pointerId))
            overviewPoint(e.clientX, e.currentTarget);
        }}
      >
        <div ref={wave.overview} aria-hidden="true" />
      </div>
      {!compact && (
        <div className="audio-wave-footer">
          <span>横向滚动平移 · Option / Ctrl 滚动缩放 · 左右键微调</span>
          <span>显示幅度不改变音量</span>
        </div>
      )}
      <input
        className="audio-accessible-seek"
        type="range"
        aria-label="音乐播放位置"
        min="0"
        max={duration}
        step="1"
        disabled={blocked}
        value={requestedPosition ?? sample.current.position.positionMs}
        onChange={(e) => onSeek(Number(e.target.value))}
      />
    </section>
  );
}
