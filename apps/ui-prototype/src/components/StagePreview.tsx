import { useRef } from "react";
import { PlayIcon, PauseIcon, CornersOutIcon } from "@phosphor-icons/react";
import { PALETTE, GROUPS, formatTime, clamp } from "../demo-session";
import type { Clip, DemoCue } from "../demo-session";

export function StagePreview({
  cue,
  clip,
  time,
  playing,
  onSeek,
  onPlay,
}: {
  cue: DemoCue;
  clip?: Clip;
  time: number;
  playing: boolean;
  onSeek: (n: number) => void;
  onPlay: () => void;
}) {
  const viewport = useRef<HTMLDivElement>(null);
  const color = PALETTE.find((p) => p.id === clip?.color) ?? PALETTE[2];
  const group = GROUPS.find((g) => g.id === clip?.group);
  const local = clip ? time - clip.start : -1;
  const active = clip && local >= 0 && local < clip.duration;
  const envelope = active
    ? Math.min(
        1,
        clip.fadeIn ? local / clip.fadeIn : 1,
        clip.fadeOut ? (clip.duration - local) / clip.fadeOut : 1,
      )
    : 0;
  const level = clip ? (clamp(envelope, 0, 1) * clip.intensity) / 100 : 0;
  return (
    <section className="preview panel" aria-label="离线舞台预览">
      <div className="section-heading">
        <span>
          预演画面 <i />{" "}
          <span className="subtle">
            场景 {cue.id} {cue.name}
          </span>
        </span>
        <span className="preview-mode">离线预览 · 不改变现场</span>
      </div>
      <div className="stage-viewport" ref={viewport}>
        <img
          src="/assets/stage-blue.png"
          alt="八台逆光灯的舞台场景示意"
          draggable={false}
          style={{
            filter:
              "brightness(" +
              (0.14 + level * 1.32) +
              ") saturate(" +
              color.saturation +
              ") hue-rotate(" +
              color.hue +
              "deg)",
          }}
        />
        <div
          className="fixture-labels"
          aria-label={group ? group.name + "，8 台" : "未选择灯组"}
        >
          {Array.from({ length: 8 }, (_, i) => (
            <span key={i}>
              {group?.prefix ?? "B"}
              {String(i + 1).padStart(2, "0")}
            </span>
          ))}
        </div>
        <span className="scene-note">场景示意 · 非光学仿真</span>
        {clip && !active && (
          <span className="inactive-preview">所选片段不在当前预览时间内</span>
        )}
      </div>
      <div className="preview-transport">
        <button
          className="icon-button preview-play"
          aria-label={playing ? "暂停预览" : "播放预览"}
          onClick={onPlay}
        >
          {playing ? <PauseIcon weight="fill" /> : <PlayIcon weight="fill" />}
        </button>
        <span>预览播放</span>
        <span className="timecode">
          {formatTime(time)}{" "}
          <span className="muted">/ {formatTime(cue.duration)}</span>
        </span>
        <input
          className="seek-slider"
          type="range"
          min={0}
          max={cue.duration}
          step="any"
          value={time}
          aria-label="预览位置"
          onChange={(e) => onSeek(Number(e.target.value))}
        />
        <button
          className="icon-button fullscreen"
          title="放大舞台预览"
          aria-label="放大舞台预览"
          onClick={() => {
            if (document.fullscreenElement) void document.exitFullscreen();
            else void viewport.current?.requestFullscreen().catch(() => {});
          }}
        >
          <CornersOutIcon />
        </button>
      </div>
    </section>
  );
}
