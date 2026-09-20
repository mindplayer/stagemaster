import { useEffect, useRef, useState } from "react";
import { PlusIcon, XIcon } from "@phosphor-icons/react";

/** Local monitoring only. The file never leaves the browser. */
export function AudioReference({
  time,
  playing,
  width,
  duration,
  onNotice,
}: {
  time: number;
  playing: boolean;
  width: number;
  duration: number;
  onNotice: (message: string) => void;
}) {
  const input = useRef<HTMLInputElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const audio = useRef<HTMLAudioElement>(null);
  const [source, setSource] = useState<{
    name: string;
    url: string;
    samples: Float32Array;
    duration: number;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  useEffect(
    () => () => {
      if (source) URL.revokeObjectURL(source.url);
    },
    [source],
  );
  useEffect(() => {
    const element = audio.current;
    if (!element || !source) return;
    if (time >= source.duration) {
      element.pause();
      return;
    }
    if (Math.abs(element.currentTime - time) > 0.3) element.currentTime = time;
    if (playing && element.paused)
      void element
        .play()
        .catch(() =>
          onNotice("浏览器暂未允许参考音频播放，请再次点击预览播放"),
        );
    if (!playing) element.pause();
  }, [time, playing, source, onNotice]);
  useEffect(() => {
    const el = canvas.current;
    if (!el || !source) return;
    const context = el.getContext("2d");
    if (!context) return;
    el.width = Math.max(800, width * 1200);
    el.height = 44;
    context.clearRect(0, 0, el.width, 44);
    context.strokeStyle = "#7993ae";
    context.lineWidth = 1;
    const samplesPerSecond = source.samples.length / source.duration;
    for (let x = 0; x < el.width; x += 3) {
      const start = Math.floor((x / el.width) * duration * samplesPerSecond);
      if (start >= source.samples.length) break;
      const end = Math.min(
        source.samples.length,
        Math.floor(((x + 3) / el.width) * duration * samplesPerSecond),
      );
      let peak = 0;
      for (let s = start; s < end; s += 8)
        peak = Math.max(peak, Math.abs(source.samples[s]));
      context.beginPath();
      context.moveTo(x, 22 - peak * 19);
      context.lineTo(x, 22 + peak * 19);
      context.stroke();
    }
  }, [source, width, duration]);
  async function loadFile(file?: File) {
    if (!file) return;
    if (file.size > 30 * 1024 * 1024) {
      onNotice("原型支持 30 兆字节以内的音频参考");
      return;
    }
    setLoading(true);
    let context: AudioContext | undefined;
    try {
      context = new AudioContext();
      const decoded = await context.decodeAudioData(await file.arrayBuffer());
      setSource({
        name: file.name,
        url: URL.createObjectURL(file),
        samples: new Float32Array(decoded.getChannelData(0)),
        duration: decoded.duration,
      });
      onNotice("音乐参考已添加，仅在本机预览监听");
    } catch {
      onNotice("无法读取这份音频，请选择 MP3、WAV 或 M4A 文件");
    } finally {
      void context?.close();
      setLoading(false);
      if (input.current) input.current.value = "";
    }
  }
  return (
    <div className="audio-lane">
      <input
        ref={input}
        className="visually-hidden"
        type="file"
        accept="audio/*"
        aria-label="导入音乐参考"
        onChange={(e) => void loadFile(e.target.files?.[0])}
      />
      {source ? (
        <>
          <canvas ref={canvas} aria-label={"音乐参考波形：" + source.name} />
          <span className="audio-name">{source.name} · 仅本地预览</span>
          <button
            className="icon-button audio-remove"
            aria-label="移除音乐参考"
            onClick={() => setSource(null)}
          >
            <XIcon />
          </button>
          <audio ref={audio} src={source.url} />
        </>
      ) : (
        <button
          className="add-audio"
          onClick={() => input.current?.click()}
          disabled={loading}
        >
          <PlusIcon /> {loading ? "正在读取音频…" : "添加音乐参考"}{" "}
          <span>仅本地预览</span>
        </button>
      )}
    </div>
  );
}
