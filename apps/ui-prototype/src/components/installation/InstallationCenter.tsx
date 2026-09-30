import { useEffect, useRef } from "react";
import type { InstallationController } from "./useInstallation";
import { installationLabels } from "../../installation-tools";
import { InstallationTask } from "./InstallationTask";
import "./installation.css";
export function InstallationCenter({
  controller,
}: {
  controller: InstallationController;
}) {
  const {
    view,
    open,
    setOpen,
    busy,
    error,
    communicationError,
    request,
    available,
  } = controller;
  const button = useRef<HTMLButtonElement>(null);
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    if (open) heading.current?.focus();
  }, [open]);
  const task = view?.installation.task;
  const label = communicationError
    ? "状态不可用"
    : task
      ? installationLabels[task.phase]
      : "节目安装";
  function close() {
    setOpen(false);
    button.current?.focus();
  }
  return (
    <>
      <button
        ref={button}
        className="wb-install-toggle"
        disabled={!available}
        aria-expanded={open}
        aria-controls="installation-center"
        aria-label={`下发节目：${label}`}
        onClick={() => (open ? close() : setOpen(true))}
      >
        下发节目{task ? ` · ${label}` : ""}
      </button>
      {open && (
        <aside
          id="installation-center"
          className="wb-install-center"
          aria-labelledby="installation-heading"
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              event.preventDefault();
              event.stopPropagation();
              close();
            }
          }}
        >
          <header>
            <h2 id="installation-heading" tabIndex={-1} ref={heading}>
              下发节目
            </h2>
            <button onClick={close} aria-label="关闭安装面板">
              关闭
            </button>
          </header>
          <div className="wb-install-body">
            <section className="wb-install-target" aria-label="当前连接设备">
              <strong>{view?.destination.name || "尚未连接设备"}</strong>
              <p>
                {view?.destination.allowed && !communicationError
                  ? "当前连接具备安装权限"
                  : communicationError ||
                    view?.destination.reason ||
                    "正在读取设备状态…"}
              </p>
            </section>
            {(error || communicationError) && (
              <div className="wb-install-error" role="alert">
                {communicationError || error}
                <button
                  disabled={busy}
                  onClick={() => void request({ kind: "status" })}
                >
                  重新读取状态
                </button>
              </div>
            )}
            {task ? (
              <InstallationTask task={task} controller={controller} />
            ) : (
              <p>在工程的“播放包”中选择节目，生成后下发到当前连接设备。</p>
            )}
          </div>
          <footer>收起面板后继续处理；安装不会启动灯光输出。</footer>
        </aside>
      )}
    </>
  );
}
