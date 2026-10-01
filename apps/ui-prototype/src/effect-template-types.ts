/** Core v1 DTO. Normalized intensity uses the same 16-bit scale as scene values. */
export interface EffectTemplate {
  format: "stagemaster-effect-template";
  formatVersion: 1;
  templateId: string;
  revision: string;
  definition: {
    name: string;
    recipe: {
      kind: "intensity-wave";
      waveform: "smooth" | "triangle" | "pulse";
      low: number;
      high: number;
      dutyPercent: number;
    };
    timing: {
      periodMs: number;
      phaseDegrees: number;
      spreadDegrees: number;
      reverseOrder: boolean;
    };
  };
}
export interface EffectTemplateSource {
  template: EffectTemplate;
  sha256: string;
}
