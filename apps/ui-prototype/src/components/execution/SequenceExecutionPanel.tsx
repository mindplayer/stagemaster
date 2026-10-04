import { useEffect, useState, type ComponentProps } from "react";
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
  const showBackground = !!props.execution && background;
  const [opened, setOpened] = useState(showBackground);
  useEffect(() => {
    if (showBackground) setOpened(true);
  }, [showBackground]);
  return (
    <div
      className={
        props.execution
          ? "wb-preview execution-area"
          : "wb-preview execution-retained"
      }
    >
      {props.execution && (
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
      )}
      {!showBackground && <PreviewPanel {...props} />}
      <div className="execution-retained" hidden={!showBackground}>
        {(opened || showBackground) && (
          <BackgroundExecution
            {...props}
            project={project}
            visible={props.visible && showBackground}
          />
        )}
      </div>
    </div>
  );
}
