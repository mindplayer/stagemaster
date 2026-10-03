import { useEffect, useRef, useState, type SetStateAction } from "react";
import type { EditOperation, ProjectView } from "../../application-host";
import type { FixturePlacement } from "../../stage-types";
import type { RiggingPreviewPort } from "../../rigging-preview-types";
import { validateEditorForm } from "../workbench/form-validation";
import {
  beginRigging,
  riggingCommand,
  RiggingInputError,
  type RiggingSession,
} from "./rigging-session";
import {
  RiggingPreviewQueue,
  type RiggingPreviewResult,
} from "./rigging-preview-queue";

export function useRiggingDraft({
  project,
  generation,
  preview,
  onPending,
  onAccepted,
}: {
  project: ProjectView;
  generation: number;
  preview?: RiggingPreviewPort;
  onPending(value: boolean): void;
  onAccepted(ids: string[], placements: FixturePlacement[]): void;
}) {
  const [session, setSession] = useState<RiggingSession | null>(null);
  const current = useRef(session);
  const form = useRef<HTMLFormElement>(null);
  const collected = useRef<FixturePlacement[]>([]);
  const [result, setResult] = useState<RiggingPreviewResult | null>(null);
  const [retry, setRetry] = useState(0);
  const port = useRef(preview);
  port.current = preview;
  const [queue] = useState(
    () =>
      new RiggingPreviewQueue(
        (g, c) =>
          port.current
            ? port.current(g, c)
            : Promise.reject(new Error("当前宿主不支持挂接预览")),
        setResult,
      ),
  );
  function write(value: RiggingSession | null) {
    queue.cancel();
    setResult(null);
    setRetry((n) => n + 1);
    current.current = value;
    setSession(value);
    onPending(value?.dirty ?? false);
  }
  let command = null,
    problem = "";
  if (session) {
    try {
      command = riggingCommand(project, session);
    } catch (e) {
      problem = e instanceof Error ? e.message : String(e);
    }
  }
  const key = command
    ? JSON.stringify([generation, session!.source, command])
    : "";
  const fresh = result?.key === key ? result : null;
  const projection = fresh?.projection ?? null;
  problem ||= fresh?.error ?? "";
  useEffect(() => {
    queue.cancel();
    if (!command) return;
    const timer = window.setTimeout(
      () => queue.submit(key, generation, command),
      90,
    );
    return () => {
      window.clearTimeout(timer);
      queue.cancel();
    };
    // Stable serialized input is the request identity, not the fresh command object.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, retry, queue]);
  function clearValidity() {
    form.current
      ?.querySelectorAll<HTMLInputElement | HTMLSelectElement>("input,select")
      .forEach((field) => field.setCustomValidity(""));
  }
  return {
    session,
    form,
    projection,
    problem,
    computing: !!command && !fresh,
    pending: () => current.current?.dirty ?? false,
    open(ids: string[], rig: string) {
      write(beginRigging(project, ids, rig));
    },
    update(patch: Partial<Omit<RiggingSession, "source" | "dirty">>) {
      if (current.current) {
        clearValidity();
        write({ ...current.current, ...patch, dirty: true });
      }
    },
    setIds(action: SetStateAction<string[]>) {
      if (!current.current) return;
      const ids =
        typeof action === "function" ? action(current.current.ids) : action;
      write({ ...current.current, ids, dirty: true });
    },
    prepareApply() {
      if (current.current) {
        current.current = { ...current.current, dirty: true };
        setSession(current.current);
        onPending(true);
      }
    },
    collect(): EditOperation[] {
      if (!current.current?.dirty) return [];
      try {
        validateEditorForm(form.current);
        const value = riggingCommand(project, current.current);
        const expected = JSON.stringify([
          generation,
          current.current.source,
          value,
        ]);
        if (result?.key !== expected)
          throw new Error("正在计算挂接位置，请稍后应用");
        if (result.error) throw new Error(result.error);
        if (!result.projection) throw new Error("请先完成挂接预览");
        collected.current = result.projection.placements;
        return result.projection.changed
          ? [{ op: "stage", command: value }]
          : [];
      } catch (e) {
        if (e instanceof RiggingInputError)
          form.current
            ?.querySelector<HTMLElement>(`[data-rig-field="${e.field}"]`)
            ?.focus();
        throw e;
      }
    },
    retry() {
      setResult(null);
      setRetry((n) => n + 1);
    },
    cancel() {
      clearValidity();
      collected.current = [];
      write(null);
    },
    accept() {
      const ids = current.current?.dirty ? current.current.ids : null;
      const placements = collected.current;
      collected.current = [];
      clearValidity();
      write(null);
      if (ids?.length) onAccepted(ids, placements);
    },
  };
}
