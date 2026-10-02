import { type ComponentProps } from "react";
import type { ProjectView } from "../../application-host";
import { executionPosition } from "../workbench/execution-position";
import { PreviewPanel } from "../workbench/PreviewPanel";
import { BackgroundExecution } from "./BackgroundExecution";
export function SequenceExecutionPanel({
  project,
  background,
  onBackgroundChange,
  ...props
}: ComponentProps<typeof PreviewPanel> & {
  project: ProjectView;
  background: boolean;
  onBackgroundChange(value: boolean): void;
}) {
  if (!props.execution) return <PreviewPanel {...props} />;
  return (
    <div className="wb-preview execution-area">
      <nav className="execution-buttons" aria-label="执行目标">
        <button
          aria-pressed={!background}
          onClick={() => onBackgroundChange(false)}
        >
          编辑预演
        </button>
        <button
          aria-pressed={background}
          onClick={() => {
            onBackgroundChange(true);
            props.onPosition?.(executionPosition(null));
          }}
        >
          后台执行
        </button>
      </nav>
      {background ? (
        <BackgroundExecution {...props} project={project} />
      ) : (
        <PreviewPanel {...props} />
      )}
    </div>
  );
}
