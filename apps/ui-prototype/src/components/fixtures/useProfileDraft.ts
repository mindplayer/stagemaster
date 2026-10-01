import {
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
  type ForwardedRef,
} from "react";
import type { EditOperation } from "../../application-host";
import type { ProfileView } from "../../fixture-types";
import type { ImportedProfile } from "../../profile-file-types";
import {
  FixtureFieldError,
  profileDefinition,
  profileDraft,
  type ProfileDraft,
} from "../../fixture-tools";

export interface ProfileHandle {
  collect(): EditOperation[];
  accept(): void;
}

/** Authoring drafts share one commit path; imported definitions need explicit acceptance. */
export function useProfileDraft({
  ref,
  profile,
  beforeChange,
  onPending,
  onSelected,
}: {
  ref: ForwardedRef<ProfileHandle>;
  profile: ProfileView | undefined;
  beforeChange(): Promise<boolean>;
  onPending(value: boolean): void;
  onSelected(id: string): void;
}) {
  const [draft, setDraftState] = useState<ProfileDraft | null>(null);
  const [imported, setImported] = useState<ImportedProfile | null>(null);
  const draftRef = useRef<ProfileDraft | null>(null),
    importedRef = useRef<ImportedProfile | null>(null);
  const editingId = useRef<string | null>(null),
    importAllowed = useRef(false);
  const form = useRef<HTMLFormElement>(null);
  useEffect(() => {
    if (imported)
      form.current?.querySelector<HTMLInputElement>("[name=name]")?.focus();
  }, [imported]);
  function source(value: ImportedProfile | null) {
    importedRef.current = value;
    setImported(value);
  }
  function setDraft(value: ProfileDraft | null) {
    draftRef.current = value;
    setDraftState(value);
    onPending(value !== null);
  }
  function cancel() {
    setDraft(null);
    source(null);
  }
  function collect(): EditOperation[] {
    if (!draftRef.current) return [];
    if (importedRef.current && !importAllowed.current) {
      form.current
        ?.querySelector<HTMLButtonElement>("[data-import-save]")
        ?.focus();
      throw new Error("请先保存到工程，或取消导入模式");
    }
    importAllowed.current = false;
    try {
      return [
        {
          op: "fixture",
          command: {
            op: "saveProfile",
            id: editingId.current,
            definition: profileDefinition(draftRef.current),
          },
        },
      ];
    } catch (reason) {
      if (reason instanceof FixtureFieldError) {
        const field = form.current?.elements.namedItem(
          reason.field,
        ) as HTMLInputElement | null;
        field?.focus();
        field?.setCustomValidity?.(reason.message);
        field?.reportValidity?.();
      }
      throw reason;
    }
  }
  useImperativeHandle(ref, () => ({
    collect,
    accept() {
      if (draftRef.current) {
        onSelected(editingId.current ?? "");
        cancel();
      }
    },
  }));
  async function begin(copy: boolean) {
    if (!(await beforeChange())) return;
    source(null);
    editingId.current = copy ? null : (profile?.id ?? null);
    const next = profileDraft(profile);
    if (copy) next.name = `${next.name} 副本`;
    setDraft(next);
  }
  async function beginNew() {
    if (!(await beforeChange())) return;
    source(null);
    editingId.current = null;
    setDraft(profileDraft());
  }
  function importFile(file: ImportedProfile) {
    editingId.current = null;
    source(file);
    setDraft(profileDraft(file.definition));
  }
  async function save() {
    importAllowed.current = true;
    try {
      await beforeChange();
    } finally {
      importAllowed.current = false;
    }
  }
  return {
    editingExisting: editingId.current !== null,
    draft,
    imported,
    form,
    setDraft,
    cancel,
    begin,
    beginNew,
    importFile,
    save,
  };
}
