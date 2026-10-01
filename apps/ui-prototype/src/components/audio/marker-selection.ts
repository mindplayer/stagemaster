export interface MarkerLaneSelection {
  active: boolean;
  ids: string[];
  blocked: boolean;
  onMode(): void;
  onPick(id: string, range: boolean): void;
  onClear(): void;
}
