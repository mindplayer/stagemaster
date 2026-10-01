/** Core DTO. Normalized brightness is independent of native DMX resolution. */
interface TemplateEnvelope {
  format: "stagemaster-effect-template";
  templateId: string;
  revision: string;
}
interface TemplateDefinition {
  name: string;
  timing: {
    periodMs: number;
    phaseDegrees: number;
    spreadDegrees: number;
    reverseOrder: boolean;
  };
}
export type EffectTemplate = TemplateEnvelope &
  (
    | {
        formatVersion: 1;
        definition: TemplateDefinition & {
          recipe: {
            kind: "intensity-wave";
            waveform: "smooth" | "triangle" | "pulse";
            low: number;
            high: number;
            dutyPercent: number;
          };
        };
      }
    | {
        formatVersion: 2;
        definition: TemplateDefinition & {
          recipe: {
            kind: "intensity-keyframes";
            keyframes: import("./effect-types").EffectKeyframe[];
          };
        };
      }
  );
export interface EffectTemplateSource {
  template: EffectTemplate;
  sha256: string;
}
export interface ImportedEffectTemplate {
  generation: number;
  token: string;
  fileName: string;
  review: {
    sceneId: string;
    effect: import("./effect-types").SceneEffect;
    usage: import("./check-types").PlanUsage;
  };
}
export interface ExportedEffectTemplate {
  generation: number;
  effectId: string;
  path: string | null;
  warning: string | null;
}
