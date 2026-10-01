import type { ComponentProps } from "react";
import { PackagePanel } from "../workbench/PackagePanel";
import { ProjectCheckPanel } from "../workbench/ProjectCheckPanel";
import { PatchReportExport } from "./PatchReportExport";

/** Project handoff and checks stay mounted across workspace navigation. */
export function ProjectDocuments({
  fileName,
  packageProps,
  onSaveResources,
}: {
  fileName: string | null;
  packageProps: ComponentProps<typeof PackagePanel>;
  onSaveResources(generation: number): Promise<boolean>;
}) {
  const {
    host,
    project,
    generation,
    hasDrafts,
    visible,
    busy,
    capture,
    onLocate,
  } = packageProps;
  return (
    <>
      {visible && (
        <details className="wb-file-details">
          <summary>
            文件信息 · {fileName?.split(/[\\/]/).at(-1) ?? "尚未保存"}
          </summary>
          <dl className="wb-project-summary">
            <dt>工程名称</dt>
            <dd>{project.name}</dd>
            <dt>文件位置</dt>
            <dd>{fileName ?? "尚未保存"}</dd>
            <dt>灯具</dt>
            <dd>{project.fixtures.length} 台</dd>
            <dt>场景</dt>
            <dd>{project.scenes.length} 个</dd>
          </dl>
        </details>
      )}
      <PatchReportExport
        {...{ host, project, generation, hasDrafts, visible, busy, capture }}
      />
      <PackagePanel {...packageProps} />
      <ProjectCheckPanel
        {...{
          host,
          generation,
          hasDrafts,
          visible,
          busy,
          capture,
          onLocate,
          onSaveResources,
        }}
        projectId={project.id}
      />
    </>
  );
}
