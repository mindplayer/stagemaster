import type { FixturePlacement } from "../../stage-types";
import { useState } from "react";
import type { FixtureView } from "../../application-host";
import type { GroupView, LibraryEdit } from "../../library-types";
import { groupMembersIssue } from "../../group-members";
import { GroupMemberTools } from "./GroupMemberTools";
import { GroupMembers } from "./GroupMembers";
import { LibraryDialog } from "./LibraryDialog";
export function GroupEditor({
  group,
  fixtures,
  placements,
  selected,
  name: initialName,
  busy,
  error,
  onCancel,
  onEdit,
}: {
  group?: GroupView;
  fixtures: FixtureView[];
  placements: FixturePlacement[];
  selected: string[];
  name: string;
  busy: boolean;
  error: string;
  onCancel(): void;
  onEdit(command: LibraryEdit): Promise<boolean>;
}) {
  const [name, setName] = useState(group?.name ?? initialName);
  const [ids, setIds] = useState(group?.fixtureIds ?? selected);
  const issue = groupMembersIssue(ids, fixtures);
  return (
    <LibraryDialog
      title={group ? "编辑灯组" : "记录灯组"}
      busy={busy}
      error={error}
      onCancel={onCancel}
      submitDisabled={!!issue}
      onSubmit={() => {
        if (!name.trim()) throw new Error("请填写灯组名称");
        if (issue) throw new Error(issue);
        return onEdit({
          kind: "saveGroup",
          id: group?.id ?? null,
          name: name.trim(),
          fixtureIds: ids,
        });
      }}
    >
      <label>
        灯组名称
        <input
          autoFocus
          aria-label="灯组名称"
          required
          maxLength={256}
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </label>
      <p>选灯顺序 · {ids.length} 台；整体整理作用于完整灯组。</p>
      <GroupMemberTools
        ids={ids}
        fixtures={fixtures}
        placements={placements}
        selected={selected}
        initial={group?.fixtureIds ?? selected}
        onChange={setIds}
      />
      <GroupMembers fixtures={fixtures} ids={ids} onChange={setIds} />
      {issue && <p role="status">{issue}</p>}
    </LibraryDialog>
  );
}
