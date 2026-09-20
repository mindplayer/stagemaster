import { useEffect, useRef } from "react";
import type { Clip } from "../demo-session";

/** Data plot of this clip's fade envelope, not a decorative image substitute. */
export function ClipEnvelope({ clip }: { clip: Clip }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const node = canvas.current;
    if (!node) return;
    function draw() {
      if (!node) return;
      const rect = node.getBoundingClientRect();
      const w = Math.max(1, Math.round(rect.width * 2));
      const h = Math.max(1, Math.round(rect.height * 2));
      node.width = w;
      node.height = h;
      const ctx = node.getContext("2d");
      if (!ctx) return;
      const left = (clip.fadeIn / clip.duration) * w;
      const right = (1 - clip.fadeOut / clip.duration) * w;
      ctx.beginPath();
      ctx.moveTo(0, h);
      ctx.lineTo(left, 2);
      ctx.lineTo(right, 2);
      ctx.lineTo(w, h);
      ctx.closePath();
      ctx.fillStyle = "#b5d9ff18";
      ctx.fill();
      ctx.beginPath();
      ctx.moveTo(0, h);
      ctx.lineTo(left, 2);
      ctx.lineTo(right, 2);
      ctx.lineTo(w, h);
      ctx.strokeStyle = "#bdeaffaa";
      ctx.lineWidth = 2;
      ctx.stroke();
    }
    draw();
    const resize = new ResizeObserver(draw);
    resize.observe(node);
    return () => resize.disconnect();
  }, [clip.fadeIn, clip.fadeOut, clip.duration]);
  return <canvas ref={canvas} className="clip-envelope" aria-hidden="true" />;
}
