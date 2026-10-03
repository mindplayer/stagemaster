/** The streaming SDK watches window resize, but our dock also resizes inside a fixed window. */
export function observePrevisInputGeometry(
  container: HTMLElement,
  update: () => void,
) {
  let frame = 0;
  let previous = "";
  const observer = new ResizeObserver(() => {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      const { clientWidth: width, clientHeight: height } = container;
      const size = `${width}:${height}`;
      if (
        !container.isConnected ||
        width <= 0 ||
        height <= 0 ||
        size === previous
      )
        return;
      previous = size;
      update();
    });
  });
  observer.observe(container);
  return () => {
    observer.disconnect();
    cancelAnimationFrame(frame);
  };
}
