import { LightbulbIcon } from "@phosphor-icons/react";
import type { ApplicationHost } from "../../application-host";
import { RecentProjects } from "./RecentProjects";
export function ProjectStart({
  host,
  busy,
  onNew,
  onOpen,
  onRecover,
  onRecent,
}: {
  host: ApplicationHost;
  busy: boolean;
  onNew(): void;
  onOpen(): void;
  onRecover(): void;
  onRecent(id: string): Promise<boolean>;
}) {
  return (
    <div className="project-start">
      <section className="project-start-intro">
        <LightbulbIcon size={44} weight="duotone" />
        <h1>开始编排</h1>
        <p>
          {host.kind === "browser"
            ? "请使用桌面应用打开本地工程"
            : "创建工程，或继续已有编排"}
        </p>
        <div>
          <button
            className="wb-primary"
            disabled={busy || host.kind !== "desktop"}
            onClick={onNew}
          >
            新建工程
          </button>
          <button disabled={busy || host.kind !== "desktop"} onClick={onOpen}>
            打开工程
          </button>
          <button
            disabled={busy || host.kind !== "desktop"}
            onClick={onRecover}
          >
            恢复工程
          </button>
        </div>
      </section>
      {host.kind === "desktop" && (
        <RecentProjects
          host={host}
          busy={busy}
          onOpen={onRecent}
          onBrowse={onOpen}
        />
      )}
    </div>
  );
}
