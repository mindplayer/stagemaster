import { fitSelectionView, type SelectionViewRange } from "./selection-view";
import { audioDisplayTime } from "../../audio-loop-tools";
import { useEffect, useRef, useState, type RefObject } from "react";
import WaveSurfer from "wavesurfer.js";
import Timeline from "wavesurfer.js/dist/plugins/timeline.js";
import Minimap from "wavesurfer.js/dist/plugins/minimap.js";
import type { AudioPosition } from "../../audio-types";
import { audioTime } from "../../audio-tools";
import { zoomAround, type WaveViewport } from "./waveform-data";

export function useWaveSurfer(
  peaks: Float32Array[] | null,
  duration: number,
  sample: RefObject<{ position: AudioPosition; at: number }>,
  gain: number,
  channelHeight = 100,
) {
  const detail = useRef<HTMLDivElement>(null);
  const ruler = useRef<HTMLDivElement>(null);
  const overview = useRef<HTMLDivElement>(null);
  const instance = useRef<WaveSurfer | null>(null);
  const laneCursor = useRef<HTMLDivElement>(null);
  const preview = useRef<number | null>(null);
  const follow = useRef(true);
  const manualUntil = useRef(0);
  const [viewport, setViewport] = useState<WaveViewport>({
    start: 0,
    end: duration,
    width: 800,
  });
  const [zoom, setZoom] = useState(1);
  const [problem, setProblem] = useState("");
  const [ready, setReady] = useState(false);
  const latest = useRef({ sample, duration });
  latest.current = { sample, duration };
  useEffect(() => {
    if (!detail.current || !ruler.current || !overview.current || !peaks)
      return;
    setProblem("");
    setReady(false);
    setZoom(1);
    const minimap = Minimap.create({
      container: overview.current,
      height: 48,
      interact: false,
      waveColor: "#537b88",
      progressColor: "#84cab8",
      cursorColor: "#fff1b6",
      overlayColor: "rgba(100, 213, 195, .16)",
      normalize: false,
      splitChannels: peaks.map(() => ({ overlay: true, height: 48 })),
    });
    const wave = WaveSurfer.create({
      container: detail.current,
      height: channelHeight,
      peaks,
      duration: duration / 1000,
      waveColor: "#67bbaa",
      progressColor: "#acebd7",
      cursorColor: "#fff1b6",
      cursorWidth: 1,
      interact: false,
      autoScroll: false,
      autoCenter: false,
      normalize: false,
      splitChannels: peaks.map((_, i) => ({
        height: channelHeight,
        waveColor: i ? "#6a9cc1" : "#67bbaa",
        progressColor: i ? "#b6d7f0" : "#acebd7",
      })),
      plugins: [
        Timeline.create({
          height: 28,
          container: ruler.current,
          secondaryLabelOpacity: 0.6,
          style: {
            color: "#91a8b7",
            fontSize: "10px",
            fontFamily: "ui-monospace, monospace",
          },
          formatTimeCallback: (seconds) =>
            audioTime(Math.round(seconds * 1000)).replace(/\.000$/, ""),
        }),
        minimap,
      ],
    });
    instance.current = wave;
    const update = () => {
      const width = wave.getWidth();
      const full = wave.getWrapper().clientWidth;
      if (ruler.current?.firstElementChild) {
        (ruler.current.firstElementChild as HTMLElement).style.width =
          `${full}px`;
        ruler.current.scrollLeft = wave.getScroll();
      }
      if (width > 0 && full > 0)
        setViewport({
          start: (wave.getScroll() / full) * duration,
          end: ((wave.getScroll() + width) / full) * duration,
          width,
        });
    };
    const off = [
      wave.on("scroll", update),
      wave.on("redrawcomplete", update),
      wave.on("ready", () => {
        setReady(true);
        update();
      }),
      wave.on("error", () => setProblem("波形显示失败，请重新打开音频工作区")),
    ];
    let frame = 0;
    let previous = -1;
    function tick() {
      if (!detail.current?.isConnected) {
        frame = requestAnimationFrame(tick);
        return;
      }
      const { position, at } = latest.current.sample.current;
      const time =
        preview.current ?? audioDisplayTime(position, performance.now() - at);
      if (laneCursor.current) {
        const x =
          (time / duration) * wave.getWrapper().clientWidth - wave.getScroll();
        laneCursor.current.style.transform = `translateX(${x}px)`;
        laneCursor.current.hidden = x < 0 || x > wave.getWidth();
      }
      if (time !== previous) {
        wave.setTime(time / 1000);
        previous = time;
      }
      if (
        position.playing &&
        preview.current === null &&
        follow.current &&
        performance.now() > manualUntil.current
      ) {
        const px = (time / duration) * wave.getWrapper().clientWidth;
        if (
          px < wave.getScroll() ||
          px > wave.getScroll() + wave.getWidth() * 0.85
        )
          wave.setScroll(Math.max(0, px - wave.getWidth() * 0.2));
      }
      frame = requestAnimationFrame(tick);
    }
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
      off.forEach((fn) => fn());
      wave.destroy();
      instance.current = null;
      setReady(false);
    };
  }, [peaks, duration, channelHeight]);
  useEffect(() => {
    instance.current?.setOptions({ barHeight: gain });
  }, [gain, peaks, duration]);
  function zoomTo(value: number, x?: number) {
    const wave = instance.current;
    if (!wave || !wave.getDecodedData()) return;
    const ratio = Math.max(
      1,
      Math.min(value, Math.max(1, ((duration / 1000) * 400) / wave.getWidth())),
    );
    const localX = x ?? wave.getWidth() / 2;
    const anchor =
      ((wave.getScroll() + localX) / wave.getWrapper().clientWidth) * duration;
    const px = (ratio * wave.getWidth()) / (duration / 1000);
    wave.zoom(px);
    wave.setScroll(zoomAround(anchor, localX, px));
    manualUntil.current = performance.now() + 1500;
    setZoom(ratio);
  }
  function fitSelection(range: SelectionViewRange) {
    const wave = instance.current;
    if (!wave?.getDecodedData()) return false;
    const plan = fitSelectionView(duration, wave.getWidth(), range);
    if (!plan) return false;
    wave.zoom(plan.pixelsPerSecond);
    wave.setScroll(plan.scrollPixels);
    manualUntil.current = performance.now() + 1500;
    setZoom(plan.ratio);
    return true;
  }
  function center(time = sample.current.position.positionMs) {
    const wave = instance.current;
    if (!wave) return;
    wave.setScroll(
      Math.max(
        0,
        (time / duration) * wave.getWrapper().clientWidth - wave.getWidth() / 2,
      ),
    );
    manualUntil.current = performance.now() + 1500;
  }
  function pan(pixels: number) {
    const wave = instance.current;
    if (!wave) return;
    if (pixels !== 0) wave.setScroll(wave.getScroll() + pixels);
    manualUntil.current = performance.now() + 1500;
  }
  return {
    detail,
    ruler,
    overview,
    instance,
    preview,
    laneCursor,
    follow,
    viewport,
    zoom,
    maxZoom: Math.max(1, ((duration / 1000) * 400) / viewport.width),
    zoomTo,
    fitSelection,
    ready,
    center,
    pan,
    problem,
  };
}
