import {
  CubeIcon,
  GearSixIcon,
  LightbulbIcon,
  ListNumbersIcon,
  MusicNotesIcon,
  StackIcon,
} from "@phosphor-icons/react";
import type { ProjectView } from "../../application-host";
export type WorkbenchPage =
  | "profiles"
  | "stage"
  | "fixtures"
  | "scenes"
  | "sequences"
  | "settings"
  | "audio";
export function WorkbenchNavigation({
  page,
  project,
  busy,
  onSelect,
}: {
  page: WorkbenchPage;
  project: ProjectView;
  busy: boolean;
  onSelect(page: WorkbenchPage): void;
}) {
  const items = [
    {
      id: "scenes",
      name: "场景编辑",
      Icon: StackIcon,
      count: project.scenes.length,
    },
    {
      id: "sequences",
      name: "执行步骤",
      Icon: ListNumbersIcon,
      count: project.sequences.length,
    },
    {
      id: "audio",
      name: "音乐时间线",
      Icon: MusicNotesIcon,
      count: project.audio?.markers.length ?? 0,
    },
    {
      id: "stage",
      name: "场地布置",
      Icon: CubeIcon,
      count: project.stage.spaces.length,
    },
    {
      id: "fixtures",
      name: "灯具",
      Icon: LightbulbIcon,
      count: project.fixtures.length,
    },
    { id: "settings", name: "工程", Icon: GearSixIcon, count: undefined },
  ] as const;
  return (
    <nav className="wb-nav" aria-label="工作区">
      {items.map(({ id, name, Icon, count }) => {
        const active =
          page === id || (id === "fixtures" && page === "profiles");
        return (
          <button
            key={id}
            className={active ? "active" : ""}
            aria-pressed={active}
            disabled={busy}
            onClick={() => onSelect(id)}
          >
            <Icon />
            {name}
            {count !== undefined && <span>{count}</span>}
          </button>
        );
      })}
    </nav>
  );
}
