export interface PatchReportExport {
  generation: number;
  path: string | null;
  fixtureCount: number;
  warning: string | null;
}
