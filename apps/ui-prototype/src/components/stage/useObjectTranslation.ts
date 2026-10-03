import {
  objectTranslationOperations,
  translationSource,
  type ObjectTranslationDraft,
} from "./object-translation-session";
export type { ObjectTranslationDraft } from "./object-translation-session";
import { useRef, useState } from "react";
import type { ProjectView } from "../../application-host";
import type { SpatialVector3, StageSelection } from "../../stage-types";
import { zeroTranslation } from "./object-translation";
import { validateEditorForm } from "../workbench/form-validation";
export function useObjectTranslation(
  project: ProjectView,
  targets: StageSelection[],
  onPending: (value: boolean) => void,
) {
  const [draft, setDraft] = useState<ObjectTranslationDraft | null>(null);
  const current = useRef(draft);
  const form = useRef<HTMLFormElement>(null);
  function clearValidity() {
    form.current
      ?.querySelectorAll<HTMLInputElement>("input")
      .forEach((field) => field.setCustomValidity(""));
  }
  let problem = "";
  try {
    if (draft) objectTranslationOperations(project, draft);
  } catch (e) {
    problem = (e as Error).message;
  }
  return {
    draft,
    form,
    problem,
    update(axis: keyof SpatialVector3, value: string) {
      clearValidity();
      const old = current.current ?? {
        source: translationSource(project),
        targets: structuredClone(targets),
        delta: zeroTranslation(),
      };
      const next = { ...old, delta: { ...old.delta, [axis]: value } };
      current.current = next;
      setDraft(next);
      onPending(true);
    },
    collect() {
      if (!current.current) return [];
      validateEditorForm(form.current);
      try {
        return objectTranslationOperations(project, current.current);
      } catch (e) {
        const message = (e as Error).message;
        const axis = message.match(/^([XYZ]) 位移/)?.[1].toLowerCase();
        const field = axis
          ? form.current?.querySelector<HTMLInputElement>(
              `[data-translation-axis="${axis}"]`,
            )
          : null;
        field?.setCustomValidity(message);
        field?.reportValidity();
        throw e;
      }
    },
    pending: () => current.current !== null,
    cancel() {
      clearValidity();
      current.current = null;
      setDraft(null);
      onPending(false);
    },
  };
}
