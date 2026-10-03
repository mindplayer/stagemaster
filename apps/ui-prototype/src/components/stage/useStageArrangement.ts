import { useRef, useState, type SetStateAction } from "react";
import type { ProjectView } from "../../application-host";
import type { FixturePlacement, StageSpace } from "../../stage-types";
import {
  PlacementInputError,
  type ArrangementDraft,
} from "../../placement-tools";
import { validateEditorForm } from "../workbench/form-validation";
import {
  beginArrangement,
  arrangementPlacements,
  arrangementOperations,
  type ArrangementSession,
} from "./arrangement-session";

/** One draft participates in the existing workbench transaction, including save and navigation. */
export function useStageArrangement({
  project,
  onPending,
  onAccepted,
}: {
  project: ProjectView;
  onPending(value: boolean): void;
  onAccepted(ids: string[], placements: FixturePlacement[]): void;
}) {
  const [session, setSession] = useState<ArrangementSession | null>(null);
  const current = useRef(session);
  const collected = useRef<FixturePlacement[]>([]);
  const form = useRef<HTMLFormElement>(null);
  function write(next: ArrangementSession | null) {
    current.current = next;
    setSession(next);
    onPending(next?.dirty ?? false);
  }
  function clearValidity() {
    form.current
      ?.querySelectorAll<HTMLInputElement | HTMLSelectElement>("input,select")
      .forEach((field) => field.setCustomValidity(""));
  }
  let preview = null,
    problem = "";
  if (session) {
    try {
      preview = arrangementPlacements(project, session);
    } catch (e) {
      problem = e instanceof Error ? e.message : String(e);
    }
  }
  function collect() {
    const value = current.current;
    if (!value?.dirty) return [];
    try {
      validateEditorForm(form.current);
      const operations = arrangementOperations(project, value);
      collected.current = arrangementPlacements(project, value);
      return operations;
    } catch (e) {
      if (e instanceof PlacementInputError)
        form.current
          ?.querySelector<HTMLInputElement>(
            `[data-placement-field="${e.field}"]`,
          )
          ?.focus();
      throw e;
    }
  }
  return {
    session,
    form,
    preview,
    problem,
    collect,
    pending: () => current.current?.dirty ?? false,
    open(ids: string[], space: StageSpace | undefined, selected: boolean) {
      write(beginArrangement(project, ids, space, selected));
    },
    update(patch: Partial<ArrangementDraft>) {
      if (!current.current) return;
      clearValidity();
      write({
        ...current.current,
        dirty: true,
        draft: { ...current.current.draft, ...patch },
      });
    },
    setIds(action: SetStateAction<string[]>) {
      if (!current.current) return;
      const ids =
        typeof action === "function" ? action(current.current.ids) : action;
      write({ ...current.current, dirty: true, ids });
    },
    prepareApply() {
      if (current.current) write({ ...current.current, dirty: true });
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
