export interface OutputControlSnapshot {
  epoch: number;
  serial: number;
  percent: number;
  blackout: boolean;
  uncontrolledFixtures: number;
}
export type OutputControlRequest =
  | { kind: "snapshot" }
  | {
      kind: "set";
      epoch: number;
      serial: number;
      percent: number;
      blackout: boolean;
    };
export type OutputIntent = Pick<OutputControlSnapshot, "percent" | "blackout">;
