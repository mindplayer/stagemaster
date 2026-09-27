export interface RecoveryStatus {
  state: "clean" | "protected" | "unprotected";
  capturedAtMs: number | null;
  problem: string | null;
}
export interface RecoveryEntry {
  id: string;
  token: string;
  projectName: string | null;
  capturedAtMs: number | null;
  sourceFile: string | null;
  state: "ready" | "active" | "damaged";
  older: boolean;
  problem: string | null;
  canDiscard: boolean;
}
export interface RecoveryCatalog {
  entries: RecoveryEntry[];
  omitted: number;
}
export type RecoveryRequest =
  | { kind: "list" }
  | { kind: "discard"; id: string; token: string };
