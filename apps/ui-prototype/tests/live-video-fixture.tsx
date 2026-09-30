import { useEffect, useRef } from "react";
import { resumeVisibleVideo } from "../src/components/stage/resume-visible-video";

/** Local generated stream: exercises browser pause on DOM parking without UE. */
export function LiveVideoFixture() {
  const parent = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const container = parent.current!;
    const video = container.querySelector("video")!;
    const canvas = document.createElement("canvas");
    canvas.width = 160;
    canvas.height = 90;
    const ctx = canvas.getContext("2d")!;
    const timer = setInterval(() => {
      ctx.fillStyle = "#123f52";
      ctx.fillRect(0, 0, 160, 90);
      ctx.fillStyle = "#80c6bc";
      ctx.fillRect(Date.now() % 140, 20, 20, 50);
    }, 40);
    const stream = canvas.captureStream(25);
    video.srcObject = stream;
    const play = () => {
      void video.play().catch(() => {});
    };
    const cleanup = resumeVisibleVideo(container, play);
    play();
    return () => {
      cleanup();
      clearInterval(timer);
      stream.getTracks().forEach((track) => track.stop());
      video.srcObject = null;
    };
  }, []);
  return (
    <div ref={parent} style={{ minHeight: 90 }}>
      <video
        aria-label="唯一视频节点"
        muted
        playsInline
        style={{ width: 160, height: 90 }}
      />
    </div>
  );
}
