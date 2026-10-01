import type {
  ProjectView,
  FixtureView,
  SceneView,
} from "../../application-host";
import { attributeState } from "../../editor-tools";
import { axisReadout } from "../../position-tools";
export function PositionReadout({
  project,
  heads,
  scene,
}: {
  project: ProjectView;
  heads: FixtureView[];
  scene: SceneView;
}) {
  return (
    <details className="position-readout-details">
      <summary>逐灯角度 · {heads.length} 台</summary>
      <div className="position-readout">
        {heads.map((f) => (
          <p key={f.id}>
            <strong>{f.name}</strong>
            <span>
              {(["pan", "tilt"] as const)
                .map((key) => {
                  const a = attributeState(scene, [f], key),
                    channel = project.profiles
                      .find((p) => p.id === f.profileId)
                      ?.channels.find((c) => c.attribute === key);
                  const status =
                    a.mode === "release"
                      ? "释放"
                      : a.mode === "absent"
                        ? "未记录"
                        : a.mode === "preset"
                          ? `预设 · ${a.presetName}`
                          : "已记录";
                  return `${key === "pan" ? "水平" : "垂直"} ${axisReadout(f.positioning![key], a.value, Boolean(channel?.fine)).toFixed(2)}°（${status}）`;
                })
                .join(" · ")}
            </span>
          </p>
        ))}
      </div>
    </details>
  );
}
