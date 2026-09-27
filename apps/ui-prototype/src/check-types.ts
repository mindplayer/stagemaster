export type CheckLocation =
  | { kind: "fixtures" | "scenes" }
  | { kind: "fixture" | "placement" | "scene" | "sequence"; id: string };
export interface CheckIssue {
  code: string;
  severity: "error" | "warning";
  message: string;
  location: CheckLocation;
}
export interface PlanLimits {
  attributes: number;
  steps: number;
  targetValues: number;
  effectChannels: number;
  keyframes: number;
}
export interface PlanUsage extends PlanLimits {
  valueBufferBytes: number;
  effectBufferBytes: number;
}
export interface ProgramCheck {
  name: string;
  location: CheckLocation;
  status: "passed" | "failed" | "blocked";
  usage: PlanUsage | null;
}
export interface ProjectCheck {
  generation: number;
  deviceRelease: "unavailable";
  report: {
    projectId: string;
    revisionId: string;
    desktopReady: boolean;
    issues: CheckIssue[];
    programs: ProgramCheck[];
    limits: PlanLimits;
  };
}
