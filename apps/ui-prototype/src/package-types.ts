import type { CheckLocation } from "./check-types";
export type PackageSelection = { kind: "scene" | "sequence"; id: string };
export interface PackageCandidate extends PackageSelection {
  name: string;
}
export interface PackageProgram {
  name: string;
  location: CheckLocation;
  encodedBytes: number;
  attributes: number;
  steps: number;
  effectChannels: number;
  keyframes: number;
  valueBytes: number;
  residentBytes: number;
  loaderPeakBytes: number;
}
export interface PackageReport {
  projectId: string;
  revisionId: string;
  sourceDigest: string;
  packageDigest: string;
  profile: string;
  bytes: number;
  universe: number;
  catalogResidentBytes: number;
  maxLoaderBytes: number;
  executionSemantics?: number;
  programs: PackageProgram[];
}
export interface PackageResult {
  generation: number;
  token: string | null;
  report: PackageReport | null;
  issues: { message: string; location: CheckLocation | null }[];
}
export interface PackageExport {
  path: string | null;
  warning: string | null;
}
