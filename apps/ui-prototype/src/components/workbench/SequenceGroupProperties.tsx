import { forwardRef, useImperativeHandle, useRef, useState } from "react";
import type { StepView } from "../../sequence-types";
import {
  SequenceGroupTiming,
  type GroupTimingHandle,
} from "./SequenceGroupTiming";
import { SequenceGroupScript } from "./SequenceGroupScript";
export type GroupPropertiesHandle = GroupTimingHandle;

/** A local cancel may clear one editor, never the other editor's pending work. */
export const SequenceGroupProperties = forwardRef<
  GroupPropertiesHandle,
  {
    sequenceId: string;
    steps: StepView[];
    busy: boolean;
    beforeChange(): Promise<boolean>;
    onPending(value: boolean): void;
  }
>(function SequenceGroupProperties({ onPending, ...props }, ref) {
  const timing = useRef<GroupTimingHandle>(null),
    script = useRef<GroupTimingHandle>(null);
  const pending = useRef({ timing: false, script: false });
  const [both, setBoth] = useState(false);
  const change = (kind: "timing" | "script", value: boolean) => {
    pending.current[kind] = value;
    setBoth(pending.current.timing && pending.current.script);
    onPending(pending.current.timing || pending.current.script);
  };
  useImperativeHandle(ref, () => ({
    collect: () => [
      ...(timing.current?.collect() ?? []),
      ...(script.current?.collect() ?? []),
    ],
    accept() {
      timing.current?.accept();
      script.current?.accept();
    },
  }));
  return (
    <section aria-label="步骤批量属性">
      {both && <p className="wb-dim">本次应用包含时间和剧本提示修改。</p>}
      <SequenceGroupTiming
        {...props}
        ref={timing}
        onPending={(value) => change("timing", value)}
      />
      <SequenceGroupScript
        {...props}
        ref={script}
        onPending={(value) => change("script", value)}
      />
    </section>
  );
});
