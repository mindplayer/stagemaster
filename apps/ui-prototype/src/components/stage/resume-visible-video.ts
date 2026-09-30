/** Browsers may pause a live video when its dock is parked outside the document. */
export function resumeVisibleVideo(container: HTMLElement, play: () => void) {
  const video = container.querySelector("video");
  if (!video) return () => {};
  let frame = 0;
  const resume = () => {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      const rect = container.getBoundingClientRect();
      if (
        container.isConnected &&
        rect.width > 0 &&
        rect.height > 0 &&
        video.srcObject &&
        video.paused
      )
        play();
    });
  };
  const observer = new ResizeObserver(resume);
  observer.observe(container);
  video.addEventListener("pause", resume);
  return () => {
    observer.disconnect();
    video.removeEventListener("pause", resume);
    cancelAnimationFrame(frame);
  };
}
