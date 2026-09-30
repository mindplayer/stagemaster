export interface RecentProject {
  id: string;
  name: string;
  path: string;
  openedAtMs: number;
  available: boolean;
}
export type RecentRequest = { kind: "list" } | { kind: "forget"; id: string };
