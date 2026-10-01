export interface PatchReportExport {
  generation: number;
  path: string | null;
  fixtureCount: number;
  warning: string | null;
}

export interface SequenceReportExport {
  generation: number;
  path: string | null;
  sequenceId: string;
  sequenceName: string;
  stepCount: number;
  warning: string | null;
}
